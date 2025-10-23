use crate::ast::*;
use fcdb_graph::{GraphDB, Rid, LabelId, Timestamp};

/// Query execution plan
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub match_plan: MatchPlan,
    pub where_plan: Option<WherePlan>,
    pub return_plan: ReturnPlan,
}

#[derive(Debug, Clone)]
pub struct MatchPlan {
    pub start_nodes: Vec<Rid>,
    pub traversals: Vec<TraversalStep>,
}

#[derive(Debug, Clone)]
pub struct TraversalStep {
    pub from_variable: String,
    pub to_variable: String,
    pub relationship_types: Vec<LabelId>,
    pub direction: Direction,
    pub min_hops: u32,
    pub max_hops: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct WherePlan {
    pub conditions: Vec<Condition>,
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub left: ValueRef,
    pub op: BinaryOperator,
    pub right: ValueRef,
}

#[derive(Debug, Clone)]
pub enum ValueRef {
    Variable(String),
    Property { variable: String, property: String },
    Literal(Literal),
}

#[derive(Debug, Clone)]
pub struct ReturnPlan {
    pub items: Vec<ReturnItem>,
    pub distinct: bool,
    pub limit: Option<u32>,
    pub skip: Option<u32>,
}

pub struct QueryPlanner<'a> {
    graph: &'a GraphDB,
}

impl<'a> QueryPlanner<'a> {
    pub fn new(graph: &'a GraphDB) -> Self {
        Self { graph }
    }

    /// Plan a Cypher query execution
    /// Merkle DAG: fcdb_cypher -> plan_query(query) -> execution_plan
    pub async fn plan_query(&self, query: &Query) -> Result<ExecutionPlan, String> {
        let mut match_plan = None;
        let mut where_plan = None;
        let mut return_plan = None;

        for statement in &query.statements {
            match statement {
                Statement::Match(match_clause) => {
                    match_plan = Some(self.plan_match(&match_clause.pattern).await?);
                }
                Statement::Where(where_clause) => {
                    where_plan = Some(self.plan_where(&where_clause.condition)?);
                }
                Statement::Return(return_clause) => {
                    return_plan = Some(self.plan_return(return_clause)?);
                }
            }
        }

        let match_plan = match_plan.ok_or("No MATCH clause found")?;
        let return_plan = return_plan.ok_or("No RETURN clause found")?;

        Ok(ExecutionPlan {
            match_plan,
            where_plan,
            return_plan,
        })
    }

    async fn plan_match(&self, pattern: &Pattern) -> Result<MatchPlan, String> {
        let mut start_nodes = Vec::new();
        let mut traversals = Vec::new();

        // For now, start from all nodes if no specific start is given
        // In a full implementation, we'd analyze the pattern to find optimal starting points
        if pattern.elements.is_empty() {
            return Err("Empty pattern".to_string());
        }

        // Find start nodes (nodes without incoming relationships in the pattern)
        let mut node_variables = std::collections::HashMap::new();
        let mut rel_sources: std::collections::HashSet<String> = std::collections::HashSet::new();

        for element in &pattern.elements {
            match element {
                PatternElement::Node(node) => {
                    if let Some(var) = &node.variable {
                        node_variables.insert(var.clone(), node.clone());
                    }
                }
                PatternElement::Relationship(rel) => {
                    // This is a simplified approach - we'd need to track variable bindings
                    // For now, just collect all nodes
                }
            }
        }

        // If we have specific node patterns, use them as start points
        for (var, node_pattern) in &node_variables {
            if !node_pattern.labels.is_empty() {
                // For now, assume all nodes are potential matches
                // In a real implementation, we'd filter by labels
                start_nodes.extend(self.graph.list_rids().await);
                break;
            }
        }

        // If no specific patterns, start from all nodes
        if start_nodes.is_empty() {
            start_nodes = self.graph.list_rids().await;
        }

        // Plan traversals for relationships
        for element in &pattern.elements {
            if let PatternElement::Relationship(rel) = element {
                // This is a simplified traversal planning
                // In a full implementation, we'd need to properly track variable bindings
                let from_var = "start".to_string(); // Simplified
                let to_var = "end".to_string();     // Simplified

                let relationship_types = rel.types.iter()
                    .map(|t| LabelId(t.parse().unwrap_or(0)))
                    .collect();

                let (min_hops, max_hops) = match &rel.length {
                    Some(PathLength::Any) => (0, None),
                    Some(PathLength::Range(min, max)) => (*min, *max),
                    None => (1, Some(1)),
                };

                traversals.push(TraversalStep {
                    from_variable: from_var,
                    to_variable: to_var,
                    relationship_types,
                    direction: rel.direction.clone(),
                    min_hops,
                    max_hops,
                });
            }
        }

        Ok(MatchPlan {
            start_nodes,
            traversals,
        })
    }

    fn plan_where(&self, condition: &Expression) -> Result<WherePlan, String> {
        let conditions = self.extract_conditions(condition)?;
        Ok(WherePlan { conditions })
    }

    fn extract_conditions(&self, expr: &Expression) -> Result<Vec<Condition>, String> {
        match expr {
            Expression::BinaryOp { left, op, right } => {
                let left_ref = self.expr_to_value_ref(left)?;
                let right_ref = self.expr_to_value_ref(right)?;
                Ok(vec![Condition {
                    left: left_ref,
                    op: op.clone(),
                    right: right_ref,
                }])
            }
            Expression::In { left, list } => {
                let left_ref = self.expr_to_value_ref(left)?;
                let mut conditions = Vec::new();

                // Convert IN to multiple OR conditions
                for item in list {
                    let right_ref = self.expr_to_value_ref(item)?;
                    conditions.push(Condition {
                        left: left_ref.clone(),
                        op: BinaryOperator::Equal,
                        right: right_ref,
                    });
                }

                Ok(conditions)
            }
            _ => Err("Unsupported WHERE expression".to_string()),
        }
    }

    fn expr_to_value_ref(&self, expr: &Expression) -> Result<ValueRef, String> {
        match expr {
            Expression::Variable(var) => Ok(ValueRef::Variable(var.clone())),
            Expression::PropertyAccess { variable, property } => {
                Ok(ValueRef::Property {
                    variable: variable.clone(),
                    property: property.clone(),
                })
            }
            Expression::Literal(lit) => Ok(ValueRef::Literal(lit.clone())),
            _ => Err("Unsupported expression in condition".to_string()),
        }
    }

    fn plan_return(&self, return_clause: &ReturnClause) -> Result<ReturnPlan, String> {
        Ok(ReturnPlan {
            items: return_clause.items.clone(),
            distinct: return_clause.distinct,
            limit: return_clause.limit,
            skip: return_clause.skip,
        })
    }
}
