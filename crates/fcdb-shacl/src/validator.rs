use crate::shapes::*;
use crate::report::*;
use fcdb_graph::{GraphDB, Rid, LabelId};
use regex::Regex;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ValidationConfig {
    pub max_violations: usize,  // Maximum violations to report
    pub strict_mode: bool,      // Fail fast on first violation
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_violations: 100,
            strict_mode: false,
        }
    }
}

pub struct ShaclValidator {
    config: ValidationConfig,
}

impl ShaclValidator {
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Main validation entry point
    /// Merkle DAG: fcdb_shacl -> validate(data_graph, shapes_input) -> report
    pub async fn validate(
        &self,
        data_graph: &GraphDB,
        shapes_input: &str,
    ) -> Result<ValidationReport, crate::ShaclError> {
        // Parse shapes from RDF input
        let shapes = if shapes_input.trim().is_empty() {
            // Use example shapes for testing
            create_example_shapes()
        } else {
            parse_shapes_from_rdf(shapes_input)
                .map_err(|e| crate::ShaclError::ShapeParse(e))?
        };

        let mut report = ValidationReport::new();

        // Get all nodes to validate
        let rids = data_graph.list_rids().await;

        for shape in &shapes {
            report.add_shape(&shape.id());

            match shape {
                Shape::Node(node_shape) => {
                    self.validate_node_shape(data_graph, node_shape, &rids, &mut report).await?;
                }
                Shape::Property(prop_shape) => {
                    self.validate_property_shape(data_graph, prop_shape, &rids, &mut report).await?;
                }
            }

            if self.config.strict_mode && !report.is_conformant() {
                break;
            }
        }

        Ok(report)
    }

    async fn validate_node_shape(
        &self,
        data_graph: &GraphDB,
        shape: &NodeShape,
        rids: &[Rid],
        report: &mut ValidationReport,
    ) -> Result<(), crate::ShaclError> {
        // Determine target nodes
        let target_rids = self.get_target_nodes(data_graph, shape, rids).await?;

        for &rid in &target_rids {
            let focus_node = format!("{}", rid.0);
            let mut result = ValidationResult::new(focus_node.clone(), shape.id.clone());

            for constraint in &shape.constraints {
                self.validate_node_constraint(data_graph, rid, constraint, &focus_node, &mut result).await?;
            }

            report.add_result(result);

            if report.results.len() >= self.config.max_violations {
                break;
            }
        }

        Ok(())
    }

    async fn validate_property_shape(
        &self,
        data_graph: &GraphDB,
        shape: &PropertyShape,
        rids: &[Rid],
        report: &mut ValidationReport,
    ) -> Result<(), crate::ShaclError> {
        for &rid in rids {
            let focus_node = format!("{}", rid.0);
            let mut result = ValidationResult::new(focus_node.clone(), shape.id.clone());

            // Get values for the property path
            let values = self.get_property_values(data_graph, rid, &shape.path).await?;

            for constraint in &shape.constraints {
                self.validate_property_constraint(values.clone(), constraint, &shape.path, &focus_node, &mut result)?;
            }

            if !result.is_valid() {
                report.add_result(result);
            }

            if report.results.len() >= self.config.max_violations {
                break;
            }
        }

        Ok(())
    }

    async fn validate_node_constraint(
        &self,
        data_graph: &GraphDB,
        rid: Rid,
        constraint: &Constraint,
        focus_node: &str,
        result: &mut ValidationResult,
    ) -> Result<(), crate::ShaclError> {
        match &constraint.component {
            ConstraintComponent::Datatype { datatype } => {
                if let Ok(Some(data)) = data_graph.get_node(rid).await {
                    let data_str = String::from_utf8_lossy(&data);
                    if !self.validate_datatype(&data_str, datatype) {
                        result.add_violation(
                            Violation::new(
                                "sh:datatype".to_string(),
                                format!("Value does not match datatype {}", datatype),
                            )
                            .with_value(data_str.to_string())
                            .with_expected(datatype.clone())
                        );
                    }
                }
            }
            // Other node constraints would be implemented here
            _ => {} // Placeholder for other constraint types
        }
        Ok(())
    }

    fn validate_property_constraint(
        &self,
        values: Vec<String>,
        constraint: &Constraint,
        path: &PropertyPath,
        focus_node: &str,
        result: &mut ValidationResult,
    ) -> Result<(), crate::ShaclError> {
        match &constraint.component {
            ConstraintComponent::MinCount { min } => {
                if values.len() < *min {
                    result.add_violation(
                        Violation::new(
                            "sh:minCount".to_string(),
                            format!("Expected at least {} values, found {}", min, values.len()),
                        )
                        .with_path(format!("{:?}", path))
                    );
                }
            }
            ConstraintComponent::MaxCount { max } => {
                if values.len() > *max {
                    result.add_violation(
                        Violation::new(
                            "sh:maxCount".to_string(),
                            format!("Expected at most {} values, found {}", max, values.len()),
                        )
                        .with_path(format!("{:?}", path))
                    );
                }
            }
            ConstraintComponent::Datatype { datatype } => {
                for value in &values {
                    if !self.validate_datatype(value, datatype) {
                        result.add_violation(
                            Violation::new(
                                "sh:datatype".to_string(),
                                format!("Property value does not match datatype {}", datatype),
                            )
                            .with_value(value.clone())
                            .with_expected(datatype.clone())
                            .with_path(format!("{:?}", path))
                        );
                    }
                }
            }
            ConstraintComponent::Pattern { pattern, flags } => {
                let regex = match flags {
                    Some(f) if f.contains('i') => Regex::new(&format!("(?i){}", pattern)),
                    _ => Regex::new(pattern),
                }.map_err(|e| crate::ShaclError::Validation(format!("Invalid regex pattern: {}", e)))?;

                for value in &values {
                    if !regex.is_match(value) {
                        result.add_violation(
                            Violation::new(
                                "sh:pattern".to_string(),
                                format!("Value does not match pattern {}", pattern),
                            )
                            .with_value(value.clone())
                            .with_path(format!("{:?}", path))
                        );
                    }
                }
            }
            ConstraintComponent::In { values: allowed_values } => {
                for value in &values {
                    if !allowed_values.contains(value) {
                        result.add_violation(
                            Violation::new(
                                "sh:in".to_string(),
                                format!("Value not in allowed set: {:?}", allowed_values),
                            )
                            .with_value(value.clone())
                            .with_path(format!("{:?}", path))
                        );
                    }
                }
            }
            // Other property constraints would be implemented here
            _ => {} // Placeholder
        }
        Ok(())
    }

    fn validate_datatype(&self, value: &str, datatype: &str) -> bool {
        match datatype {
            "http://www.w3.org/2001/XMLSchema#string" => true, // All strings are valid
            "http://www.w3.org/2001/XMLSchema#integer" => value.parse::<i64>().is_ok(),
            "http://www.w3.org/2001/XMLSchema#boolean" => matches!(value, "true" | "false" | "1" | "0"),
            // Add more datatype validations as needed
            _ => true, // Unknown datatypes are assumed valid
        }
    }

    async fn get_target_nodes(
        &self,
        data_graph: &GraphDB,
        shape: &NodeShape,
        rids: &[Rid],
    ) -> Result<Vec<Rid>, crate::ShaclError> {
        if let Some(target_class) = &shape.target_class {
            // For now, return all nodes - proper class-based targeting would need OWL reasoning
            Ok(rids.to_vec())
        } else if !shape.target_node.is_empty() {
            // Target specific nodes by ID
            let mut targets = vec![];
            for node_id in &shape.target_node {
                if let Ok(rid) = node_id.parse::<u64>() {
                    targets.push(Rid(rid));
                }
            }
            Ok(targets)
        } else {
            // Default: all nodes
            Ok(rids.to_vec())
        }
    }

    async fn get_property_values(
        &self,
        data_graph: &GraphDB,
        rid: Rid,
        path: &PropertyPath,
    ) -> Result<Vec<String>, crate::ShaclError> {
        match path {
            PropertyPath::Predicate(predicate) => {
                // For now, treat predicate as label ID
                if let Ok(label_id) = predicate.parse::<u32>() {
                    let edges = data_graph.get_edges_from(rid).await;
                    let mut values = vec![];

                    for edge in edges {
                        if edge.label.0 == label_id {
                            if let Ok(Some(data)) = data_graph.get_node(edge.target).await {
                                values.push(String::from_utf8_lossy(&data).to_string());
                            }
                        }
                    }

                    Ok(values)
                } else {
                    Ok(vec![]) // Unknown predicate
                }
            }
        }
    }
}

impl Shape {
    fn id(&self) -> String {
        match self {
            Shape::Node(ns) => ns.id.clone(),
            Shape::Property(ps) => ps.id.clone(),
        }
    }
}
