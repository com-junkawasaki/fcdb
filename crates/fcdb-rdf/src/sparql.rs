#[cfg(feature = "sparql")]
use oxigraph::{
    io::{GraphFormat, GraphParser},
    model::{GraphName, Quad},
    sparql::{Query, QueryResults},
    store::Store,
};

use super::RdfExporter;

#[derive(Clone, Copy, Debug)]
pub enum SparqlQueryKind {
    Select,
    Construct,
}

pub struct SparqlRunner<'a> {
    pub exporter: RdfExporter<'a>,
}

impl<'a> SparqlRunner<'a> {
    pub fn new(exporter: RdfExporter<'a>) -> Self { Self { exporter } }

    pub async fn execute(&self, query: &str) -> Result<String, String> {
        // Project current view into an in-memory store
        let store = Store::new().map_err(|e| e.to_string())?;
        let ntriples = self.exporter.export_ntriples().await.map_err(|e| e.to_string())?;
        let parser = GraphParser::from_format(GraphFormat::NTriples);
        for t in parser.read_triples(ntriples.as_bytes()) {
            let t = t.map_err(|e| e.to_string())?;
            let q = Quad::new(t.subject, t.predicate, t.object, GraphName::DefaultGraph);
            store.insert(&q).map_err(|e| e.to_string())?;
        }

        // Run query
        let q = Query::parse(query, None).map_err(|e| e.to_string())?;
        let results = store.query(q).map_err(|e| e.to_string())?;

        // Return JSON serialization for SELECT, N-Triples for CONSTRUCT
        match results {
            QueryResults::Solutions(mut s) => {
                let mut rows = vec![];
                while let Some(sol) = s.next().transpose().map_err(|e| e.to_string())? {
                    let mut row = serde_json::Map::new();
                    for (var, val) in sol.iter() {
                        row.insert(var.as_str().to_string(), serde_json::Value::String(val.to_string()));
                    }
                    rows.push(serde_json::Value::Object(row));
                }
                Ok(serde_json::Value::Array(rows).to_string())
            }
            QueryResults::Graph(g) => {
                let mut nt = String::new();
                for t in g {
                    let t = t.map_err(|e| e.to_string())?;
                    nt.push_str(&format!("{:?}\n", t));
                }
                Ok(nt)
            }
            QueryResults::Boolean(b) => Ok(serde_json::json!({"boolean": b}).to_string()),
        }
    }
}


