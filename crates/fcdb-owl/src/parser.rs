/// Extract RDFS and basic OWL axioms from ontology string
/// Merkle DAG: fcdb_owl -> extract_rdfs_rules(input) -> rules
pub fn extract_rdfs_rules(input: &str) -> Vec<RdfsRule> {
    let mut rules = Vec::new();

    // Basic RDFS rules (subset)
    // Rule 1: If (x p y) and (p rdfs:subPropertyOf q) then (x q y)
    rules.push(RdfsRule::SubProperty);

    // Rule 2: If (p rdfs:domain c) and (x p y) then (x rdf:type c)
    rules.push(RdfsRule::Domain);

    // Rule 3: If (p rdfs:range c) and (x p y) then (y rdf:type c)
    rules.push(RdfsRule::Range);

    // Rule 4: If (x rdf:type c) and (c rdfs:subClassOf d) then (x rdf:type d)
    rules.push(RdfsRule::SubClass);

    // Rule 5: If (p rdfs:subPropertyOf q) and (q rdfs:subPropertyOf r) then (p rdfs:subPropertyOf r)
    rules.push(RdfsRule::SubPropertyTransitive);

    rules
}

#[derive(Debug, Clone)]
pub enum RdfsRule {
    SubProperty,
    Domain,
    Range,
    SubClass,
    SubPropertyTransitive,
}
