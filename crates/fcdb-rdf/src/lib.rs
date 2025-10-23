//! fcdb-rdf: RDF projection for FCDB GraphDB
//! Merkle DAG: fcdb_rdf -> mapping, sparql (optional)

mod mapping;

#[cfg(feature = "sparql")]
mod sparql;

pub use mapping::{ExportFormat, RdfExporter, RdfNode, Triple};

#[cfg(feature = "sparql")]
pub use sparql::{SparqlQueryKind, SparqlRunner};


