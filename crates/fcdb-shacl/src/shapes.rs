use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// SHACL Shape types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Shape {
    Node(NodeShape),
    Property(PropertyShape),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeShape {
    pub id: String,
    pub target_class: Option<String>, // sh:targetClass
    pub target_node: Vec<String>,     // sh:targetNode
    pub constraints: Vec<Constraint>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PropertyShape {
    pub id: String,
    pub path: PropertyPath,           // sh:path
    pub constraints: Vec<Constraint>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PropertyPath {
    Predicate(String),     // Simple predicate path
    // Extended: Sequence, Alternative, etc. (simplified for now)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Constraint {
    pub component: ConstraintComponent,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConstraintComponent {
    Datatype { datatype: String },           // sh:datatype
    MinCount { min: usize },                // sh:minCount
    MaxCount { max: usize },                // sh:maxCount
    Pattern { pattern: String, flags: Option<String> }, // sh:pattern, sh:flags
    In { values: Vec<String> },             // sh:in
    Class { class: String },                // sh:class
    NodeKind { kind: NodeKind },            // sh:nodeKind
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum NodeKind {
    IRI,           // sh:IRI
    BlankNode,     // sh:BlankNode
    Literal,       // sh:Literal
    IRIOrLiteral,  // sh:IRIOrLiteral
    BlankNodeOrIRI, // sh:BlankNodeOrIRI
    BlankNodeOrLiteral, // sh:BlankNodeOrLiteral
}

/// Parse SHACL shapes from RDF input (simplified)
pub fn parse_shapes_from_rdf(rdf_input: &str) -> Result<Vec<Shape>, String> {
    // For now, return empty vec - will be implemented with RDF parsing
    // This would use fcdb-rdf to parse Turtle/JSON-LD shapes
    Ok(vec![])
}

/// Create example shapes for testing (temporary)
pub fn create_example_shapes() -> Vec<Shape> {
    vec![
        Shape::Node(NodeShape {
            id: "PersonShape".to_string(),
            target_class: Some("http://example.org/Person".to_string()),
            target_node: vec![],
            constraints: vec![
                Constraint {
                    component: ConstraintComponent::Datatype {
                        datatype: "http://www.w3.org/2001/XMLSchema#string".to_string(),
                    },
                },
            ],
        }),
        Shape::Property(PropertyShape {
            id: "PersonNameShape".to_string(),
            path: PropertyPath::Predicate("http://example.org/name".to_string()),
            constraints: vec![
                Constraint {
                    component: ConstraintComponent::MinCount { min: 1 },
                },
                Constraint {
                    component: ConstraintComponent::MaxCount { max: 1 },
                },
                Constraint {
                    component: ConstraintComponent::Datatype {
                        datatype: "http://www.w3.org/2001/XMLSchema#string".to_string(),
                    },
                },
            ],
        }),
    ]
}
