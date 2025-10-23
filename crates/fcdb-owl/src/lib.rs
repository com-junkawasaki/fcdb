//! fcdb-owl: OWL processing and reasoning for FCDB
//! Merkle DAG: fcdb_owl -> parser, reasoner, materializer

pub mod parser;
pub mod reasoner;

use fcdb_graph::GraphDB;
use fcdb_rdf::{RdfExporter, RdfNode, Triple};
use horned_owl::model::Ontology;
use std::collections::HashSet;

/// Classify ontology and materialize inferred triples
/// Merkle DAG: fcdb_owl -> classify_ontology(owl_input, graph) -> inferred_triples
pub async fn classify_ontology(
    owl_input: &str,
    graph: &GraphDB,
) -> Result<Vec<Triple>, OwlError> {
    // Parse OWL ontology
    let ontology = parser::parse_owl(owl_input)?;

    // Create reasoner with RDFS/OWL-RL subset
    let reasoner = reasoner::SubsetReasoner::new(ontology);

    // Get current graph as RDF triples
    let exporter = RdfExporter::new(graph, "https://enishi.local/");
    let current_triples = exporter.export_ntriples().await
        .map_err(|e| OwlError::Graph(e.to_string()))?;

    // Parse current triples
    let data_triples = parse_ntriples(&current_triples)?;

    // Apply reasoning rules
    let inferred_triples = reasoner.apply_rules(data_triples)?;

    Ok(inferred_triples)
}

/// Parse N-Triples format into Triple structs
fn parse_ntriples(ntriples: &str) -> Result<Vec<Triple>, OwlError> {
    let mut triples = Vec::new();

    for line in ntriples.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Simple N-Triples parser (subject predicate object .)
        if let Some(dot_pos) = line.rfind('.') {
            let triple_str = &line[..dot_pos].trim();
            let parts: Vec<&str> = triple_str.split_whitespace().collect();

            if parts.len() >= 3 {
                let subject = if parts[0].starts_with('<') && parts[0].ends_with('>') {
                    RdfNode::from(&parts[0][1..parts[0].len()-1])
                } else {
                    RdfNode::from("_:blank") // Simplified
                };

                let predicate = if parts[1].starts_with('<') && parts[1].ends_with('>') {
                    parts[1][1..parts[1].len()-1].to_string()
                } else {
                    parts[1].to_string()
                };

                let object = if parts[2].starts_with('<') && parts[2].ends_with('>') {
                    RdfNode::from(&parts[2][1..parts[2].len()-1])
                } else if parts[2].starts_with('"') {
                    // Literal
                    RdfNode::from("literal") // Simplified
                } else {
                    RdfNode::from("_:blank") // Simplified
                };

                triples.push(Triple {
                    s: subject,
                    p: predicate,
                    o: object.0,
                });
            }
        }
    }

    Ok(triples)
}

impl From<&str> for RdfNode {
    fn from(s: &str) -> Self {
        RdfNode(s.to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum OwlError {
    #[error("OWL parsing error: {0}")]
    Parse(String),
    #[error("Reasoning error: {0}")]
    Reasoning(String),
    #[error("RDF error: {0}")]
    Rdf(String),
    #[error("Graph error: {0}")]
    Graph(String),
}
