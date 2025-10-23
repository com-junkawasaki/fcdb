use fcdb_rdf::Triple;
use std::collections::{HashMap, HashSet};

/// Apply RDFS rules to infer new triples
/// Merkle DAG: fcdb_owl -> apply_rdfs_rules(data_triples, rules) -> inferred_triples
pub fn apply_rdfs_rules(data_triples: Vec<Triple>, _rules: Vec<super::parser::RdfsRule>) -> Result<Vec<Triple>, String> {
    let mut inferred = HashSet::new();

    // Build indexes for efficient lookup
    let mut subproperty_of = HashMap::new();
    let mut domain_of = HashMap::new();
    let mut range_of = HashMap::new();
    let mut subclass_of = HashMap::new();

    // Extract schema triples (simplified)
    for triple in &data_triples {
        match triple.p.as_str() {
            "http://www.w3.org/2000/01/rdf-schema#subPropertyOf" => {
                subproperty_of.entry(triple.s.0.clone())
                    .or_insert_with(Vec::new)
                    .push(triple.o.clone());
            }
            "http://www.w3.org/2000/01/rdf-schema#domain" => {
                domain_of.insert(triple.s.0.clone(), triple.o.clone());
            }
            "http://www.w3.org/2000/01/rdf-schema#range" => {
                range_of.insert(triple.s.0.clone(), triple.o.clone());
            }
            "http://www.w3.org/2000/01/rdf-schema#subClassOf" => {
                subclass_of.entry(triple.s.0.clone())
                    .or_insert_with(Vec::new)
                    .push(triple.o.clone());
            }
            _ => {}
        }
    }

    // Apply RDFS rules
    // Rule 1: domain inference
    for triple in &data_triples {
        if let Some(domain_class) = domain_of.get(&triple.p) {
            let new_triple = Triple {
                s: triple.s.clone(),
                p: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),
                o: domain_class.clone(),
            };
            inferred.insert(new_triple);
        }
    }

    // Rule 2: range inference
    for triple in &data_triples {
        if let Some(range_class) = range_of.get(&triple.p) {
            let new_triple = Triple {
                s: fcdb_rdf::RdfNode(triple.o.clone()),
                p: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),
                o: range_class.clone(),
            };
            inferred.insert(new_triple);
        }
    }

    // Rule 3: subclass inheritance
    for triple in &data_triples {
        if triple.p == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" {
            if let Some(supers) = subclass_of.get(&triple.o) {
                for super_class in supers {
                    let new_triple = Triple {
                        s: triple.s.clone(),
                        p: triple.p.clone(),
                        o: super_class.clone(),
                    };
                    inferred.insert(new_triple);
                }
            }
        }
    }

    // Return only the inferred triples
    Ok(inferred.into_iter().collect())
}
