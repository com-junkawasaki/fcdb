local processNetwork = {
  phases: {
    A: {
      name: "P4 Core",
      modules: ["enishi-core", "enishi-cas", "enishi-graph", "enishi-api"],
      kpis: { "3-hop": "<=13ms", cache_hit: ">=0.97", write_amp: "<=1.15" }
    }
  },
  currentPhase: "A"
};

{
  processNetwork: processNetwork,
  executionOrder: ["A"]
}