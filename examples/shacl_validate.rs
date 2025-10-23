//! SHACL Validation Example for FCDB
//! Merkle DAG: shacl_validate -> shape_parsing -> data_validation -> validation_report

use fcdb_graph::GraphDB;
use fcdb_shacl::{validate_shapes, ValidationConfig};
use fcdb_cas::PackCAS;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("FCDB SHACL Validation Example");
    println!("============================");

    // Initialize FCDB components
    let cas = PackCAS::open("./data").await?;
    let graph = GraphDB::new(cas).await;

    // SHACL shapes in Turtle format
    let shapes_turtle = r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
        @prefix foaf: <http://xmlns.com/foaf/0.1/> .

        # Person shape
        foaf:PersonShape a sh:NodeShape ;
            sh:targetClass foaf:Person ;
            sh:property [
                sh:path foaf:name ;
                sh:datatype xsd:string ;
                sh:minCount 1 ;
                sh:maxCount 1 ;
            ] ;
            sh:property [
                sh:path foaf:age ;
                sh:datatype xsd:integer ;
                sh:minInclusive 0 ;
                sh:maxInclusive 150 ;
            ] .

        # Email shape
        foaf:EmailShape a sh:NodeShape ;
            sh:targetClass foaf:Person ;
            sh:property [
                sh:path foaf:mbox ;
                sh:nodeKind sh:IRI ;
                sh:pattern "^mailto:" ;
            ] .
    "#;

    // Validation configuration
    let config = ValidationConfig {
        max_violations: 100,
        strict_mode: false,
    };

    println!("SHACL Shapes:");
    println!("{}", shapes_turtle);

    println!("\nValidation Config:");
    println!("- Max violations: {}", config.max_violations);
    println!("- Strict mode: {}", config.strict_mode);

    // Perform validation
    match validate_shapes(&graph, shapes_turtle, config).await {
        Ok(report) => {
            println!("\nValidation Report:");
            println!("- Conforms: {}", report.conforms);
            println!("- Number of results: {}", report.results.len());
            println!("- Shapes validated: {}", report.shapes.len());

            for (i, result) in report.results.iter().enumerate() {
                println!("\nResult {}:", i + 1);
                println!("  - Focus node: {}", result.focus_node);
                println!("  - Shape ID: {}", result.shape_id);
                println!("  - Result: {}", result.result);
                println!("  - Violations: {}", result.violations.len());

                for violation in &result.violations {
                    println!("    * {}: {}", violation.constraint, violation.message);
                    if let Some(value) = &violation.value {
                        println!("      Value: {}", value);
                    }
                    if let Some(expected) = &violation.expected {
                        println!("      Expected: {}", expected);
                    }
                }
            }
        }
        Err(e) => {
            println!("Validation error: {}", e);
        }
    }

    println!("\nSHACL validation example completed!");
    Ok(())
}
