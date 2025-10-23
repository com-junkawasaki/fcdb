use fcdb_graph::{GraphDB, Rid, LabelId, AdjEntry};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RdfError {
    #[error("graph error: {0}")]
    Graph(String),
    #[error("io error: {0}")]
    Io(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct RdfNode(pub String); // IRI or blank node id

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Triple {
    pub s: RdfNode,
    pub p: String,
    pub o: String,
}

#[derive(Clone, Copy, Debug)]
pub enum ExportFormat {
    NTriples,
}

pub struct RdfExporter<'a> {
    pub graph: &'a GraphDB,
    pub base_iri: &'a str,
}

impl<'a> RdfExporter<'a> {
    pub fn new(graph: &'a GraphDB, base_iri: &'a str) -> Self {
        Self { graph, base_iri }
    }

    pub async fn export_ntriples(&self) -> Result<String, RdfError> {
        let rids = self.graph.list_rids().await;
        let mut out = String::new();

        for rid in rids {
            let subj = self.iri_for_rid(rid);
            // node data triple
            if let Ok(Some(bytes)) = self.graph.get_node(rid).await {
                let data = escape_literal(&String::from_utf8_lossy(&bytes));
                out.push_str(&format!("<{}> <{}data> \"{}\" .\n", subj, self.base_iri, data));
            }

            // edges
            let edges = self.graph.get_edges_from(rid).await;
            for e in edges {
                let pred = format!("{}rel/{}", self.base_iri, e.label.0);
                let obj = self.iri_for_rid(e.target);
                out.push_str(&format!("<{}> <{}> <{}> .\n", subj, pred, obj));
            }
        }

        Ok(out)
    }

    fn iri_for_rid(&self, rid: Rid) -> String {
        format!("{}node/{}", self.base_iri, rid.0)
    }
}

fn escape_literal(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}


