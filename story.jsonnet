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
    },
    D: {
      name: "Own+CFA Final",
      modules: ["enishi-concur", "enishi-core", "enishi-derive"],
      kpis: { "3-hop": "9.3-9.8ms", cache_hit: "0.988-0.989", write_amp: "1.05-1.10" }
    }
  },
  currentPhase: "D",  // Moving to Phase D
  executionOrder: ["A", "B", "C", "D"]
};

{
  processNetwork: processNetwork,
  executionOrder: ["A", "B", "C", "D"]
}