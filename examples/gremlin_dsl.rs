//! Gremlin DSL Example for FCDB
//! Merkle DAG: gremlin_dsl -> traversal_builder -> graph_traversal -> results

use fcdb_graph::GraphDB;
use fcdb_gremlin::{execute_traversal, g};
use fcdb_cas::PackCAS;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("FCDB Gremlin DSL Example");
    println!("========================");

    // Initialize FCDB components
    let cas = PackCAS::open("./data").await?;
    let graph = GraphDB::new(cas).await;

    // Example Gremlin traversals using Rust DSL
    println!("1. Get all vertices:");
    let traversal1 = g().V().build();
    match execute_traversal(&graph, traversal1).await {
        Ok(result) => {
            println!("Found {} vertices", result.traversers.len());
            for traverser in result.traversers.iter().take(3) {
                println!("  Vertex ID: {}", traverser.current.0);
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n2. Traverse outgoing edges:");
    let traversal2 = g().V().out(None).build();
    match execute_traversal(&graph, traversal2).await {
        Ok(result) => {
            println!("Traversal result: {} traversers", result.traversers.len());
            for traverser in result.traversers.iter().take(3) {
                println!("  Path: {:?}", traverser.path.iter().map(|r| r.0).collect::<Vec<_>>());
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n3. Filter by property and get values:");
    let traversal3 = g()
        .V()
        .has("type".to_string(), serde_json::json!("Person"))
        .values("name".to_string())
        .build();
    match execute_traversal(&graph, traversal3).await {
        Ok(result) => {
            println!("Found {} matching vertices", result.traversers.len());
            for traverser in result.traversers.iter().take(3) {
                if let Some(value) = traverser.get_side_effect("value") {
                    println!("  Name: {}", value);
                }
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n4. Path traversal:");
    let traversal4 = g().V().out(None).path().build();
    match execute_traversal(&graph, traversal4).await {
        Ok(result) => {
            println!("Found {} paths", result.traversers.len());
            for traverser in result.traversers.iter().take(3) {
                if let Some(path_value) = traverser.get_side_effect("value") {
                    println!("  Path: {}", path_value);
                }
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n5. Complex traversal (Person -> knows -> Person):");
    let traversal5 = g()
        .V()
        .has("type".to_string(), serde_json::json!("Person"))
        .out(Some("knows".to_string()))
        .has("type".to_string(), serde_json::json!("Person"))
        .values("name".to_string())
        .build();
    match execute_traversal(&graph, traversal5).await {
        Ok(result) => {
            println!("Found {} friend connections", result.traversers.len());
            for traverser in result.traversers.iter().take(5) {
                if let Some(name) = traverser.get_side_effect("value") {
                    println!("  Friend: {}", name);
                }
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\nSupported Gremlin Steps:");
    println!("- V(): Start from vertices");
    println!("- out(label): Traverse outgoing edges");
    println!("- in(label): Traverse incoming edges");
    println!("- has(key, value): Filter by property");
    println!("- values(key): Extract property values");
    println!("- path(): Get traversal path");

    println!("\nNote: This implements a Gremlin-inspired DSL in Rust.");
    println!("For full Gremlin language support, consider integrating with Apache TinkerPop.");

    println!("\nGremlin DSL example completed!");
    Ok(())
}
