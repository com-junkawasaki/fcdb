//! fcdb-shacl: SHACL Core subset validator for FCDB
//! Merkle DAG: fcdb_shacl -> shapes, validator, report

mod shapes;
mod validator;
mod report;

pub use shapes::{Shape, NodeShape, PropertyShape, Constraint, ConstraintComponent};
pub use validator::{ShaclValidator, ValidationConfig};
pub use report::{ValidationReport, ValidationResult, Violation};

/// Core SHACL validation function
/// Merkle DAG: fcdb_shacl -> validate_shapes(data_graph, shape_graph) -> report
pub async fn validate_shapes(
    data_graph: &fcdb_graph::GraphDB,
    shapes_input: &str,
    config: ValidationConfig,
) -> Result<ValidationReport, ShaclError> {
    let validator = ShaclValidator::new(config);
    validator.validate(data_graph, shapes_input).await
}

#[derive(Debug, thiserror::Error)]
pub enum ShaclError {
    #[error("RDF parsing error: {0}")]
    RdfParse(String),
    #[error("Shape parsing error: {0}")]
    ShapeParse(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Graph error: {0}")]
    Graph(String),
}
