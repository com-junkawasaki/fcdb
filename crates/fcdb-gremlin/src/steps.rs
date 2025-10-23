use fcdb_graph::Rid;
use serde_json;

/// Gremlin traversal steps
#[derive(Debug, Clone)]
pub enum Step {
    /// Start from vertices (g.V())
    V(Option<Rid>),

    /// Traverse outgoing edges (out())
    Out(Option<String>),

    /// Traverse incoming edges (in())
    In(Option<String>),

    /// Filter by property (has())
    Has(String, serde_json::Value),

    /// Get property values (values())
    Values(String),

    /// Get traversal path (path())
    Path,

    /// Filter by label (hasLabel())
    HasLabel(String),

    /// Limit results (limit())
    Limit(usize),

    /// Count elements (count())
    Count,

    /// Group by key (group().by())
    GroupBy(String),

    /// Order by property (order().by())
    OrderBy(String, OrderDirection),
}

#[derive(Debug, Clone)]
pub enum OrderDirection {
    Asc,
    Desc,
}
