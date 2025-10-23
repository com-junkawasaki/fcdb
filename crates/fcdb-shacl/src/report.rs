use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// SHACL validation report
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationReport {
    pub conforms: bool,
    pub results: Vec<ValidationResult>,
    pub shapes: Vec<String>, // Shape IDs that were validated
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub result: bool,
    pub violations: Vec<Violation>,
    pub focus_node: String,  // The node that was validated
    pub shape_id: String,    // The shape that was used
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Violation {
    pub constraint: String,      // Description of the constraint
    pub message: String,         // Human-readable error message
    pub value: Option<String>,   // The actual value that violated
    pub expected: Option<String>, // What was expected
    pub path: Option<String>,    // Property path if applicable
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            conforms: true,
            results: vec![],
            shapes: vec![],
        }
    }

    pub fn add_result(&mut self, result: ValidationResult) {
        if !result.result {
            self.conforms = false;
        }
        self.results.push(result);
    }

    pub fn add_shape(&mut self, shape_id: &str) {
        self.shapes.push(shape_id.to_string());
    }

    pub fn is_conformant(&self) -> bool {
        self.conforms
    }
}

impl ValidationResult {
    pub fn new(focus_node: String, shape_id: String) -> Self {
        Self {
            result: true,
            violations: vec![],
            focus_node,
            shape_id,
        }
    }

    pub fn add_violation(&mut self, violation: Violation) {
        self.result = false;
        self.violations.push(violation);
    }

    pub fn is_valid(&self) -> bool {
        self.result
    }
}

impl Violation {
    pub fn new(constraint: String, message: String) -> Self {
        Self {
            constraint,
            message,
            value: None,
            expected: None,
            path: None,
        }
    }

    pub fn with_value(mut self, value: String) -> Self {
        self.value = Some(value);
        self
    }

    pub fn with_expected(mut self, expected: String) -> Self {
        self.expected = Some(expected);
        self
    }

    pub fn with_path(mut self, path: String) -> Self {
        self.path = Some(path);
        self
    }
}
