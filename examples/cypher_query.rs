//! Cypher Query Example for FCDB
//! Merkle DAG: cypher_query -> parse_ast -> plan_execution -> graph_traversal -> results

use fcdb_graph::GraphDB;
use fcdb_cypher::execute_cypher;
use fcdb_cas::PackCAS;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("FCDB Cypher Query Example");
    println!("=========================");

    // Initialize FCDB components
    let cas = PackCAS::open("./data").await?;
    let graph = GraphDB::new(cas).await;

    // Example Cypher queries (subset supported)
    let queries = vec![
        "MATCH (n) RETURN count(n) as node_count",
        "MATCH (n)-[r]->(m) RETURN count(r) as edge_count",
        "MATCH (n) WHERE n.type = 'Person' RETURN n.name",
        "MATCH (p:Person)-[:KNOWS]->(friend) RETURN p.name, friend.name",
        "MATCH (p:Person) WHERE p.age > 25 RETURN p.name, p.age",
    ];

    for (i, query) in queries.iter().enumerate() {
        println!("\n{}. Cypher Query: {}", i + 1, query);

        match execute_cypher(query, &graph).await {
            Ok(result) => {
                println!("Columns: {:?}", result.columns);
                println!("Rows: {} rows", result.rows.len());
                println!("Stats: nodes_created={}, nodes_deleted={}, relationships_created={}",
                    result.stats.nodes_created,
                    result.stats.nodes_deleted,
                    result.stats.relationships_created
                );
                println!("Execution time: {}ms", result.stats.execution_time_ms);

                // Show first few rows
                for (j, row) in result.rows.iter().take(3).enumerate() {
                    println!("  Row {}: {:?}", j + 1, row);
                }
                if result.rows.len() > 3 {
                    println!("  ... and {} more rows", result.rows.len() - 3);
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    // Show supported Cypher features
    println!("\nSupported Cypher Features:");
    println!("- MATCH patterns with nodes and relationships");
    println!("- WHERE clauses with equality and basic comparisons");
    println!("- RETURN with property access and aggregation");
    println!("- Basic path traversal");

    println!("\nNote: This is a subset implementation focused on core graph traversal patterns.");
    println!("For full Cypher support, consider integrating with Neo4j or similar.");

    println!("\nCypher query example completed!");
    Ok(())
}
