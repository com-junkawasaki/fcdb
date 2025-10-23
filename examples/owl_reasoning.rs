//! OWL Reasoning Example for FCDB
//! Merkle DAG: owl_reasoning -> ontology_parsing -> rdfs_rules -> rdf_materialization

use fcdb_graph::GraphDB;
use fcdb_owl::classify_ontology;
use fcdb_cas::PackCAS;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("FCDB OWL Reasoning Example");
    println!("==========================");

    // Initialize FCDB components
    let cas = PackCAS::open("./data").await?;
    let graph = GraphDB::new(cas).await;

    // Example OWL ontology with RDFS vocabulary
    let owl_ontology = r#"
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
        @prefix : <http://example.org/> .

        # Class hierarchy
        :Person rdfs:subClassOf :Agent .
        :Student rdfs:subClassOf :Person .
        :Professor rdfs:subClassOf :Person .

        # Property domains and ranges
        :hasName rdfs:domain :Agent .
        :hasName rdfs:range <http://www.w3.org/2001/XMLSchema#string> .

        :studiesUnder rdfs:domain :Student .
        :studiesUnder rdfs:range :Professor .

        :teaches rdfs:domain :Professor .
        :teaches rdfs:range :Student .
    "#;

    println!("OWL Ontology:");
    println!("{}", owl_ontology);

    // Perform OWL classification/reasoning
    println!("\nPerforming OWL reasoning...");
    match classify_ontology(owl_ontology, &graph).await {
        Ok(inferred_triples) => {
            println!("\nInferred {} triples:", inferred_triples.len());

            for (i, triple) in inferred_triples.iter().enumerate() {
                println!("{}. <{}> <{}> <{}> .",
                    i + 1,
                    triple.s.0,
                    triple.p,
                    triple.o
                );
            }

            // Group inferences by type
            let mut type_inferences = Vec::new();
            let mut domain_inferences = Vec::new();
            let mut range_inferences = Vec::new();

            for triple in &inferred_triples {
                if triple.p.contains("type") {
                    type_inferences.push(triple);
                } else if triple.p.contains("domain") {
                    // This would be meta-level, but simplified here
                } else if triple.p.contains("range") {
                    // This would be meta-level, but simplified here
                }
            }

            println!("\nInference Summary:");
            println!("- rdf:type inferences: {}", type_inferences.len());
            println!("- Total inferences: {}", inferred_triples.len());

            if !type_inferences.is_empty() {
                println!("\nSample rdf:type inferences:");
                for triple in type_inferences.iter().take(3) {
                    println!("  {} rdf:type {}", triple.s.0, triple.o);
                }
            }
        }
        Err(e) => {
            println!("OWL reasoning error: {}", e);
        }
    }

    // Example with different ontology
    let simple_ontology = r#"
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        <http://example.org/Animal> rdfs:subClassOf <http://example.org/LivingThing> .
        <http://example.org/Dog> rdfs:subClassOf <http://example.org/Animal> .
    "#;

    println!("\n\nSimple Class Hierarchy Example:");
    println!("Ontology: {}", simple_ontology);

    match classify_ontology(simple_ontology, &graph).await {
        Ok(inferred_triples) => {
            println!("Inferred triples: {}", inferred_triples.len());
            // In a real scenario with actual graph data, this would infer:
            // - Dog instances are also Animals and LivingThings
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    println!("\nSupported OWL Reasoning Features:");
    println!("- rdfs:subClassOf inheritance (transitive)");
    println!("- rdfs:domain inference (property -> type)");
    println!("- rdfs:range inference (property -> type)");
    println!("- Basic RDFS vocabulary support");

    println!("\nNote: This implements a subset of RDFS/OWL-RL reasoning.");
    println!("For full OWL 2 reasoning, consider integrating with HermiT, Pellet, or FaCT++.");

    println!("\nOWL reasoning example completed!");
    Ok(())
}
