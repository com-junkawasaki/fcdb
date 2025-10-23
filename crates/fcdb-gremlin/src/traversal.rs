use crate::steps::Step;
use fcdb_graph::Rid;

/// Gremlin traversal representation
#[derive(Debug, Clone)]
pub struct Traversal {
    pub steps: Vec<Step>,
}

impl Traversal {
    pub fn new() -> Self {
        Self { steps: vec![] }
    }

    pub fn add_step(mut self, step: Step) -> Self {
        self.steps.push(step);
        self
    }

    pub fn V() -> Self {
        Self::new().add_step(Step::V(None))
    }

    pub fn V_id(id: u64) -> Self {
        Self::new().add_step(Step::V(Some(Rid(id))))
    }

    pub fn out(mut self, label: Option<String>) -> Self {
        self.steps.push(Step::Out(label));
        self
    }

    pub fn in_(mut self, label: Option<String>) -> Self {
        self.steps.push(Step::In(label));
        self
    }

    pub fn has(mut self, key: String, value: serde_json::Value) -> Self {
        self.steps.push(Step::Has(key, value));
        self
    }

    pub fn values(mut self, key: String) -> Self {
        self.steps.push(Step::Values(key));
        self
    }

    pub fn path(mut self) -> Self {
        self.steps.push(Step::Path);
        self
    }

    pub fn has_label(mut self, label: String) -> Self {
        self.steps.push(Step::HasLabel(label));
        self
    }

    pub fn limit(mut self, count: usize) -> Self {
        self.steps.push(Step::Limit(count));
        self
    }

    pub fn count(mut self) -> Self {
        self.steps.push(Step::Count);
        self
    }
}

/// Traverser represents an element moving through the graph during traversal
#[derive(Debug, Clone)]
pub struct Traverser {
    pub current: Rid,
    pub path: Vec<Rid>,
    pub bulk: u64,  // Number of traversers represented by this one
    pub side_effects: std::collections::HashMap<String, serde_json::Value>,
}

impl Traverser {
    pub fn new(rid: Rid) -> Self {
        Self {
            current: rid,
            path: vec![rid],
            bulk: 1,
            side_effects: std::collections::HashMap::new(),
        }
    }

    pub fn new_with_path(rid: Rid, path: Vec<Rid>) -> Self {
        Self {
            current: rid,
            path,
            bulk: 1,
            side_effects: std::collections::HashMap::new(),
        }
    }

    pub fn get_path(&self) -> &[Rid] {
        &self.path
    }

    pub fn get_current(&self) -> Rid {
        self.current
    }

    pub fn attach_side_effect(&mut self, key: String, value: serde_json::Value) {
        self.side_effects.insert(key, value);
    }

    pub fn get_side_effect(&self, key: &str) -> Option<&serde_json::Value> {
        self.side_effects.get(key)
    }
}
