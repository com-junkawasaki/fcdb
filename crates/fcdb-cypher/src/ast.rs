use serde::{Deserialize, Serialize};

/// Cypher query AST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    Match(MatchClause),
    Where(WhereClause),
    Return(ReturnClause),
}

/// MATCH clause
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchClause {
    pub pattern: Pattern,
}

/// Graph pattern in MATCH clause
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub elements: Vec<PatternElement>,
}

/// Pattern element (node or relationship)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternElement {
    Node(NodePattern),
    Relationship(RelationshipPattern),
}

/// Node pattern in MATCH
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePattern {
    pub variable: Option<String>,
    pub labels: Vec<String>,
    pub properties: Vec<Property>,
}

/// Relationship pattern in MATCH
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipPattern {
    pub variable: Option<String>,
    pub types: Vec<String>,
    pub direction: Direction,
    pub length: Option<PathLength>,
    pub properties: Vec<Property>,
}

/// Relationship direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Direction {
    Outgoing,    // ->
    Incoming,    // <-
    Bidirectional, // -
}

/// Path length specification (*, *1..3, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathLength {
    Any,                    // *
    Range(u32, Option<u32>), // *min..max
}

/// Property in node/relationship patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub key: String,
    pub value: Expression,
}

/// WHERE clause
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhereClause {
    pub condition: Expression,
}

/// RETURN clause
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnClause {
    pub items: Vec<ReturnItem>,
    pub distinct: bool,
    pub limit: Option<u32>,
    pub skip: Option<u32>,
}

/// Return item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReturnItem {
    Variable(String),
    Property { variable: String, property: String },
    Count,
}

/// Expression in WHERE or property values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    Variable(String),
    Literal(Literal),
    PropertyAccess { variable: String, property: String },
    BinaryOp { left: Box<Expression>, op: BinaryOperator, right: Box<Expression> },
    In { left: Box<Expression>, list: Vec<Expression> },
}

/// Binary operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BinaryOperator {
    Equal,        // =
    NotEqual,     // <>
    LessThan,     // <
    GreaterThan,  // >
    LessEqual,    // <=
    GreaterEqual, // >=
}

/// Literal values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Literal {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
}
