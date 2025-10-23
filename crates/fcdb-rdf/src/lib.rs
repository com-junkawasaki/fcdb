//! fcdb-rdf: RDF projection for FCDB GraphDB
//! Merkle DAG: fcdb_rdf -> mapping, sparql (optional)

mod mapping;

#[cfg(feature = "sparql")]
mod sparql;

pub use mapping::{ExportFormat, RdfExporter, RdfNode, Triple};

#[cfg(feature = "sparql")]
pub use sparql::{SparqlQueryKind, SparqlRunner};

#[cfg(test)]
mod tests {
    use super::*;
    use fcdb_graph::GraphDB;
    use fcdb_cas::PackCAS;

    #[tokio::test]
    async fn test_rdf_exporter_basic() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cas = PackCAS::open(temp_dir.path()).await.unwrap();
        let graph = GraphDB::new(cas).await;

        // Create a test node
        let node_data = b"test node";
        let rid = graph.create_node(node_data).await.unwrap();

        // Create RDF exporter
        let exporter = RdfExporter::new(&graph, "https://example.org/");

        // Export to N-Triples
        let ntriples = exporter.export_ntriples().await.unwrap();

        // Verify output contains expected triples
        assert!(ntriples.contains("<https://example.org/node/"));
        assert!(ntriples.contains("<https://example.org/prop/data>"));
        assert!(ntriples.contains("test node"));
        assert!(ntriples.contains(" ."));
    }

    #[tokio::test]
    async fn test_rdf_exporter_with_edges() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cas = PackCAS::open(temp_dir.path()).await.unwrap();
        let graph = GraphDB::new(cas).await;

        // Create test nodes
        let node1_data = b"node1";
        let node2_data = b"node2";
        let rid1 = graph.create_node(node1_data).await.unwrap();
        let rid2 = graph.create_node(node2_data).await.unwrap();

        // Create edge
        let edge_props = b"edge properties";
        graph.create_edge(rid1, rid2, 1u32.into(), edge_props).await.unwrap();

        // Create RDF exporter
        let exporter = RdfExporter::new(&graph, "https://example.org/");

        // Export to N-Triples
        let ntriples = exporter.export_ntriples().await.unwrap();

        // Verify output contains node and edge triples
        assert!(ntriples.contains("<https://example.org/node/"));
        assert!(ntriples.contains("<https://example.org/rel/"));
        assert!(ntriples.contains("node1"));
        assert!(ntriples.contains("node2"));
    }

    #[cfg(feature = "sparql")]
    #[tokio::test]
    async fn test_sparql_basic_select() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cas = PackCAS::open(temp_dir.path()).await.unwrap();
        let graph = GraphDB::new(cas).await;

        // Create test data
        let rid = graph.create_node(br#"{"name": "Alice", "age": 30}"#).await.unwrap();

        // Create RDF exporter and SPARQL runner
        let exporter = RdfExporter::new(&graph, "https://example.org/");
        let runner = SparqlRunner::new(exporter);

        // Simple SELECT query
        let query = r#"
            SELECT ?s ?p ?o
            WHERE {
                ?s ?p ?o .
            }
            LIMIT 1
        "#;

        let result = runner.execute(query).await.unwrap();
        assert!(!result.is_empty());
        // Basic validation that we get some result
        assert!(result.contains("s") || result.contains("results"));
    }

    #[cfg(feature = "sparql")]
    #[tokio::test]
    async fn test_sparql_ask_query() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cas = PackCAS::open(temp_dir.path()).await.unwrap();
        let graph = GraphDB::new(cas).await;

        // Create test data
        graph.create_node(br#"{"type": "Person"}"#).await.unwrap();

        // Create RDF exporter and SPARQL runner
        let exporter = RdfExporter::new(&graph, "https://example.org/");
        let runner = SparqlRunner::new(exporter);

        // ASK query
        let query = r#"
            ASK {
                ?s ?p ?o .
            }
        "#;

        let result = runner.execute(query).await.unwrap();
        assert!(result.contains("boolean"));
        assert!(result.contains("true"));
    }

    #[test]
    fn test_rdf_node_creation() {
        let node = RdfNode("http://example.org/test".to_string());
        assert_eq!(node.0, "http://example.org/test");
    }

    #[test]
    fn test_triple_creation() {
        let s = RdfNode("http://example.org/subject".to_string());
        let p = "http://example.org/predicate".to_string();
        let o = "http://example.org/object".to_string();

        let triple = Triple { s, p, o };

        assert_eq!(triple.s.0, "http://example.org/subject");
        assert_eq!(triple.p, "http://example.org/predicate");
        assert_eq!(triple.o, "http://example.org/object");
    }

    #[test]
    fn test_export_format() {
        assert_eq!(format!("{:?}", ExportFormat::NTriples), "NTriples");
    }
}


