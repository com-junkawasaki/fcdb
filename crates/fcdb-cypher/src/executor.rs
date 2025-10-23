use crate::ast::*;
use crate::parser::parse_query;
use crate::planner::{ExecutionPlan, QueryPlanner, MatchPlan, TraversalStep, WherePlan, ReturnPlan, ValueRef};
use fcdb_graph::{GraphDB, Rid};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Cypher query executor
pub struct CypherExecutor<'a> {
    graph: &'a GraphDB,
    planner: QueryPlanner<'a>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<HashMap<String, serde_json::Value>>,
    pub stats: QueryStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStats {
    pub nodes_created: u32,
    pub nodes_deleted: u32,
    pub relationships_created: u32,
    pub relationships_deleted: u32,
    pub labels_added: u32,
    pub labels_removed: u32,
    pub properties_set: u32,
    pub execution_time_ms: u64,
}

impl<'a> CypherExecutor<'a> {
    pub fn new(graph: &'a GraphDB) -> Self {
        Self {
            graph,
            planner: QueryPlanner::new(graph),
        }
    }

    /// Execute a Cypher query
    /// Merkle DAG: fcdb_cypher -> execute(query) -> result
    pub async fn execute(&mut self, query: &str) -> Result<QueryResult, crate::CypherError> {
        let start_time = std::time::Instant::now();

        // Parse query
        let ast = parse_query(query)
            .map_err(crate::CypherError::Parse)?;

        // Plan execution
        let plan = self.planner.plan_query(&ast).await
            .map_err(crate::CypherError::Planning)?;

        // Execute plan
        let result = self.execute_plan(plan).await?;

        let execution_time = start_time.elapsed().as_millis() as u64;

        // Update stats
        let stats = QueryStats {
            nodes_created: 0, // Not implemented yet
            nodes_deleted: 0,
            relationships_created: 0,
            relationships_deleted: 0,
            labels_added: 0,
            labels_removed: 0,
            properties_set: 0,
            execution_time_ms: execution_time,
        };

        Ok(QueryResult {
            columns: result.columns,
            rows: result.rows,
            stats,
        })
    }

    async fn execute_plan(&self, plan: ExecutionPlan) -> Result<QueryResult, crate::CypherError> {
        // Execute MATCH
        let matches = self.execute_match(&plan.match_plan).await?;

        // Apply WHERE filtering
        let filtered_matches = if let Some(where_plan) = &plan.where_plan {
            self.apply_where(matches, where_plan).await?
        } else {
            matches
        };

        // Apply RETURN projection
        let result = self.apply_return(filtered_matches, &plan.return_plan).await?;

        Ok(result)
    }

    async fn execute_match(&self, match_plan: &MatchPlan) -> Result<Vec<MatchResult>, crate::CypherError> {
        let mut results = Vec::new();

        // For each start node, execute traversals
        for &start_rid in &match_plan.start_nodes {
            let mut current_bindings = HashMap::new();
            current_bindings.insert("start".to_string(), start_rid);

            let result = self.execute_traversals(start_rid, &match_plan.traversals, current_bindings).await?;
            results.extend(result);
        }

        Ok(results)
    }

    async fn execute_traversals(
        &self,
        start_rid: Rid,
        traversals: &[TraversalStep],
        initial_bindings: HashMap<String, Rid>,
    ) -> Result<Vec<MatchResult>, crate::CypherError> {
        let mut results = vec![MatchResult {
            bindings: initial_bindings,
        }];

        for traversal in traversals {
            let mut new_results = Vec::new();

            for result in &results {
                if let Some(&from_rid) = result.bindings.get(&traversal.from_variable) {
                    // Execute traversal
                    let traversal_result = self.graph.traverse(
                        from_rid,
                        Some(&traversal.relationship_types),
                        traversal.max_hops.unwrap_or(10) as usize,
                        None, // No temporal filtering for now
                    ).await.map_err(|e| crate::CypherError::Execution(e.to_string()))?;

                    for (to_rid, _depth) in traversal_result {
                        let mut new_bindings = result.bindings.clone();
                        new_bindings.insert(traversal.to_variable.clone(), to_rid);
                        new_results.push(MatchResult {
                            bindings: new_bindings,
                        });
                    }
                }
            }

            results = new_results;
        }

        Ok(results)
    }

    async fn apply_where(
        &self,
        matches: Vec<MatchResult>,
        where_plan: &WherePlan,
    ) -> Result<Vec<MatchResult>, crate::CypherError> {
        let mut filtered = Vec::new();

        for match_result in matches {
            let mut passes = true;

            for condition in &where_plan.conditions {
                if !self.evaluate_condition(&match_result, condition).await? {
                    passes = false;
                    break;
                }
            }

            if passes {
                filtered.push(match_result);
            }
        }

        Ok(filtered)
    }

    async fn evaluate_condition(
        &self,
        match_result: &MatchResult,
        condition: &crate::planner::Condition,
    ) -> Result<bool, crate::CypherError> {
        let left_value = self.resolve_value_ref(match_result, &condition.left).await?;
        let right_value = self.resolve_value_ref(match_result, &condition.right).await?;

        match condition.op {
            crate::ast::BinaryOperator::Equal => Ok(left_value == right_value),
            crate::ast::BinaryOperator::NotEqual => Ok(left_value != right_value),
            // Add other operators as needed
            _ => Err(crate::CypherError::Execution("Unsupported operator".to_string())),
        }
    }

    async fn resolve_value_ref(
        &self,
        match_result: &MatchResult,
        value_ref: &ValueRef,
    ) -> Result<serde_json::Value, crate::CypherError> {
        match value_ref {
            ValueRef::Variable(var) => {
                if let Some(&rid) = match_result.bindings.get(var) {
                    if let Ok(Some(data)) = self.graph.get_node(rid).await {
                        Ok(serde_json::from_slice(&data)
                            .unwrap_or(serde_json::Value::Null))
                    } else {
                        Ok(serde_json::Value::Null)
                    }
                } else {
                    Ok(serde_json::Value::Null)
                }
            }
            ValueRef::Property { variable, property } => {
                if let Some(&rid) = match_result.bindings.get(variable) {
                    if let Ok(Some(data)) = self.graph.get_node(rid).await {
                        let json: serde_json::Value = serde_json::from_slice(&data)
                            .unwrap_or(serde_json::Value::Null);
                        Ok(json.get(property).cloned().unwrap_or(serde_json::Value::Null))
                    } else {
                        Ok(serde_json::Value::Null)
                    }
                } else {
                    Ok(serde_json::Value::Null)
                }
            }
            ValueRef::Literal(lit) => {
                let value = match lit {
                    Literal::String(s) => serde_json::Value::String(s.clone()),
                    Literal::Integer(i) => serde_json::Value::Number((*i).into()),
                    Literal::Float(f) => serde_json::Value::Number(serde_json::Number::from_f64(*f).unwrap()),
                    Literal::Boolean(b) => serde_json::Value::Bool(*b),
                    Literal::Null => serde_json::Value::Null,
                };
                Ok(value)
            }
        }
    }

    async fn apply_return(
        &self,
        matches: Vec<MatchResult>,
        return_plan: &ReturnPlan,
    ) -> Result<QueryResult, crate::CypherError> {
        let mut columns = Vec::new();
        let mut rows = Vec::new();

        // Determine column names
        for item in &return_plan.items {
            match item {
                ReturnItem::Variable(var) => columns.push(var.clone()),
                ReturnItem::Property { variable, property } => {
                    columns.push(format!("{}.{}", variable, property));
                }
                ReturnItem::Count => columns.push("count".to_string()),
            }
        }

        // Process each match result
        for match_result in matches {
            let mut row = HashMap::new();

            for (i, item) in return_plan.items.iter().enumerate() {
                let value = match item {
                    ReturnItem::Variable(var) => {
                        self.resolve_value_ref(&match_result, &ValueRef::Variable(var.clone())).await?
                    }
                    ReturnItem::Property { variable, property } => {
                        self.resolve_value_ref(&match_result, &ValueRef::Property {
                            variable: variable.clone(),
                            property: property.clone(),
                        }).await?
                    }
                    ReturnItem::Count => serde_json::Value::Number(1.into()),
                };

                row.insert(columns[i].clone(), value);
            }

            rows.push(row);

            // Apply LIMIT
            if let Some(limit) = return_plan.limit {
                if rows.len() >= limit as usize {
                    break;
                }
            }
        }

        // Apply SKIP
        if let Some(skip) = return_plan.skip {
            rows = rows.into_iter().skip(skip as usize).collect();
        }

        Ok(QueryResult {
            columns,
            rows,
            stats: QueryStats {
                nodes_created: 0,
                nodes_deleted: 0,
                relationships_created: 0,
                relationships_deleted: 0,
                labels_added: 0,
                labels_removed: 0,
                properties_set: 0,
                execution_time_ms: 0,
            },
        })
    }
}

/// Internal match result representation
#[derive(Debug, Clone)]
struct MatchResult {
    bindings: HashMap<String, Rid>,
}
