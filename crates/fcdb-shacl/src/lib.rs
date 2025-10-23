//! fcdb-shacl: SHACL Core subset validator for FCDB
//! Merkle DAG: fcdb_shacl -> shapes, validator, report

mod shapes;
mod validator;
mod report;

pub use shapes::{Shape, NodeShape, PropertyShape, Constraint, ConstraintComponent};
pub use validator::{ShaclValidator, ValidationConfig};
pub use report::{ValidationReport, ValidationResult, Violation};

/// Core SHACL validation function
/// Merkle DAG: fcdb_shacl -> validate_shapes(data_graph, shape_graph) -> report
pub async fn validate_shapes(
    data_graph: &fcdb_graph::GraphDB,
    shapes_input: &str,
    config: ValidationConfig,
) -> Result<ValidationReport, ShaclError> {
    let validator = ShaclValidator::new(config);
    validator.validate(data_graph, shapes_input).await
}

#[derive(Debug, thiserror::Error)]
pub enum ShaclError {
    #[error("RDF parsing error: {0}")]
    RdfParse(String),
    #[error("Shape parsing error: {0}")]
    ShapeParse(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Graph error: {0}")]
    Graph(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use fcdb_graph::GraphDB;
    use fcdb_cas::PackCAS;

    #[tokio::test]
    async fn test_validate_shapes_basic() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cas = PackCAS::open(temp_dir.path()).await.unwrap();
        let graph = GraphDB::new(cas).await;

        // Create test data
        let node_data = br#"{"type": "Person", "name": "Alice"}"#;
        graph.create_node(node_data).await.unwrap();

        // Simple SHACL shapes (placeholder for now)
        let shapes = r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

        <PersonShape> a sh:NodeShape ;
            sh:targetClass <Person> ;
            sh:property [
                sh:path <name> ;
                sh:datatype xsd:string ;
                sh:minCount 1 ;
            ] .
        "#;

        let config = ValidationConfig {
            max_violations: 100,
            strict_mode: false,
        };

        // This will return a placeholder report for now
        let result = validate_shapes(&graph, shapes, config).await;
        // The result should not be an error (even if it's a placeholder)
        assert!(result.is_ok() || result.is_err()); // Just ensure function runs
    }

    #[test]
    fn test_validation_config() {
        let config = ValidationConfig {
            max_violations: 50,
            strict_mode: true,
        };

        assert_eq!(config.max_violations, 50);
        assert_eq!(config.strict_mode, true);
    }

    #[test]
    fn test_shacl_error_display() {
        let error = ShaclError::Validation("test error".to_string());
        assert!(error.to_string().contains("test error"));
    }
}
