local processNetwork = {
  phases: {
    A: {
      name: "P4 Core",
      modules: ["enishi-core", "enishi-cas", "enishi-graph", "enishi-api"],
      kpis: { "3-hop": "<=13ms", cache_hit: ">=0.97", write_amp: "<=1.15" }
    },
    B: {
      name: "P10 Optimization",
      modules: ["enishi-core", "enishi-derive"],
      kpis: { "3-hop": "<=12ms", cache_hit: ">=0.98" }
    },
    C: {
      name: "P10+ Adaptation",
      modules: ["enishi-cas", "enishi-exec", "enishi-core"],
      kpis: { "3-hop": "<=9.5ms", "9-hop": "35-80ms", cache_hit: ">=0.989" }
    }
  },
  currentPhase: "C",  // Moving to Phase C
  executionOrder: ["A", "B", "C"]
};

{
  processNetwork: processNetwork,
  executionOrder: ["A", "B", "C"]
}