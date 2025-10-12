# Enishi Database Design

## Overview
Own+CFA-Enishi: Graph database with PackCAS, temporal support, and capability-based security.

## Architecture
- **enishi-core**: Cid, Cap, Monoid, path_sig, class_sig, trace NF
- **enishi-cas**: PackCAS, cidx, bloom filters, WAL, GC
- **enishi-graph**: RID→CID, adj, posting, temporal
- **enishi-api**: GraphQL, gRPC
- **enishi-tools**: benchmarks, validators

## Phase A (Current)
- PackCAS with bloom filters
- Basic graph operations
- GraphQL API
- Performance targets: 3-hop ≤13ms, H≥0.97, WA≤1.15