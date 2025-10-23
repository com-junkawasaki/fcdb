use horned_owl::model::Ontology;
use fcdb_rdf::Triple;
use std::collections::{HashMap, HashSet};

/// Subset reasoner implementing RDFS and basic OWL-RL rules
/// Merkle DAG: fcdb_owl -> SubsetReasoner::apply_rules(data_triples) -> inferred_triples
pub struct SubsetReasoner {
    ontology: Ontology,
}

impl SubsetReasoner {
    pub fn new(ontology: Ontology) -> Self {
        Self { ontology }
    }

    /// Apply RDFS/OWL-RL rules to infer new triples
    /// Merkle DAG: fcdb_owl -> apply_rules(data) -> inferences
    pub fn apply_rules(&self, data_triples: Vec<Triple>) -> Result<Vec<Triple>, String> {
        let mut inferred = HashSet::new();
        let mut all_triples = data_triples.clone();

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

        // Apply RDFS rules (fixed-point iteration)
        let mut changed = true;
        while changed {
            changed = false;
            let current_size = all_triples.len();

            // Rule 1: subPropertyOf transitivity
            if let Some(new_triples) = self.apply_subproperty_transitivity(&all_triples, &subproperty_of) {
                for triple in new_triples {
                    if inferred.insert(triple.clone()) {
                        all_triples.push(triple);
                        changed = true;
                    }
                }
            }

            // Rule 2: domain inference
            if let Some(new_triples) = self.apply_domain_inference(&all_triples, &domain_of) {
                for triple in new_triples {
                    if inferred.insert(triple.clone()) {
                        all_triples.push(triple);
                        changed = true;
                    }
                }
            }

            // Rule 3: range inference
            if let Some(new_triples) = self.apply_range_inference(&all_triples, &range_of) {
                for triple in new_triples {
                    if inferred.insert(triple.clone()) {
                        all_triples.push(triple);
                        changed = true;
                    }
                }
            }

            // Rule 4: subclass inheritance
            if let Some(new_triples) = self.apply_subclass_inference(&all_triples, &subclass_of) {
                for triple in new_triples {
                    if inferred.insert(triple.clone()) {
                        all_triples.push(triple);
                        changed = true;
                    }
                }
            }
        }

        // Return only the inferred triples
        Ok(inferred.into_iter().collect())
    }

    fn apply_subproperty_transitivity(
        &self,
        triples: &[Triple],
        subproperty_of: &HashMap<String, Vec<String>>,
    ) -> Option<Vec<Triple>> {
        let mut new_triples = Vec::new();

        for triple in triples {
            if let Some(supers) = subproperty_of.get(&triple.p) {
                for super_prop in supers {
                    new_triples.push(Triple {
                        s: triple.s.clone(),
                        p: super_prop.clone(),
                        o: triple.o.clone(),
                    });
                }
            }
        }

        if new_triples.is_empty() {
            None
        } else {
            Some(new_triples)
        }
    }

    fn apply_domain_inference(
        &self,
        triples: &[Triple],
        domain_of: &HashMap<String, String>,
    ) -> Option<Vec<Triple>> {
        let mut new_triples = Vec::new();

        for triple in triples {
            if let Some(domain_class) = domain_of.get(&triple.p) {
                new_triples.push(Triple {
                    s: triple.s.clone(),
                    p: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),
                    o: domain_class.clone(),
                });
            }
        }

        if new_triples.is_empty() {
            None
        } else {
            Some(new_triples)
        }
    }

    fn apply_range_inference(
        &self,
        triples: &[Triple],
        range_of: &HashMap<String, String>,
    ) -> Option<Vec<Triple>> {
        let mut new_triples = Vec::new();

        for triple in triples {
            if let Some(range_class) = range_of.get(&triple.p) {
                new_triples.push(Triple {
                    s: fcdb_rdf::RdfNode(triple.o.clone()),
                    p: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),
                    o: range_class.clone(),
                });
            }
        }

        if new_triples.is_empty() {
            None
        } else {
            Some(new_triples)
        }
    }

    fn apply_subclass_inference(
        &self,
        triples: &[Triple],
        subclass_of: &HashMap<String, Vec<String>>,
    ) -> Option<Vec<Triple>> {
        let mut new_triples = Vec::new();

        for triple in triples {
            if triple.p == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" {
                if let Some(supers) = subclass_of.get(&triple.o) {
                    for super_class in supers {
                        new_triples.push(Triple {
                            s: triple.s.clone(),
                            p: triple.p.clone(),
                            o: super_class.clone(),
                        });
                    }
                }
            }
        }

        if new_triples.is_empty() {
            None
        } else {
            Some(new_triples)
        }
    }
}
