//! SPARQL Query Example for FCDB
//! Merkle DAG: sparql_query -> rdf_projection -> oxigraph_store -> sparql_execution

use fcdb_graph::GraphDB;
use fcdb_rdf::{RdfExporter, SparqlRunner};
use fcdb_cas::PackCAS;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("FCDB SPARQL Query Example");
    println!("=========================");

    // Initialize FCDB components
    let cas = PackCAS::open("./data").await?;
    let graph = GraphDB::new(cas).await;

    // Add some sample data (would be loaded from RDF in real usage)
    // For demo, we'll assume some RDF data is already in the graph

    // Create RDF exporter and SPARQL runner
    let exporter = RdfExporter::new(&graph, "https://example.org/");
    let runner = SparqlRunner::new(exporter);

    // Example SPARQL SELECT query
    let select_query = r#"
        PREFIX foaf: <http://xmlns.com/foaf/0.1/>
        SELECT ?name ?email
        WHERE {
            ?person foaf:name ?name .
            ?person foaf:mbox ?email .
        }
        LIMIT 10
    "#;

    println!("\n1. SELECT Query:");
    println!("Query: {}", select_query);
    match runner.execute(select_query).await {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {}", e),
    }

    // Example SPARQL CONSTRUCT query
    let construct_query = r#"
        PREFIX foaf: <http://xmlns.com/foaf/0.1/>
        PREFIX vcard: <http://www.w3.org/2001/vcard-rdf/3.0#>
        CONSTRUCT {
            ?person vcard:FN ?name .
        }
        WHERE {
            ?person foaf:name ?name .
        }
    "#;

    println!("\n2. CONSTRUCT Query:");
    println!("Query: {}", construct_query);
    match runner.execute(construct_query).await {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {}", e),
    }

    // Example SPARQL ASK query
    let ask_query = r#"
        PREFIX foaf: <http://xmlns.com/foaf/0.1/>
        ASK {
            ?person foaf:name "Alice" .
        }
    "#;

    println!("\n3. ASK Query:");
    println!("Query: {}", ask_query);
    match runner.execute(ask_query).await {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {}", e),
    }

    println!("\nSPARQL query example completed!");
    Ok(())
}
