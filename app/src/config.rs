pub enum GraphEngine {
    Falkor,
    EmitOnly,
}

pub enum LogEngine {
    GCP,
    Fake,
}

pub struct Config {
    pub graph_engine: GraphEngine,
    pub log_engine: LogEngine,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            graph_engine: GraphEngine::Falkor,
            log_engine: LogEngine::GCP,
        }
    }
}

impl Config {
    pub fn new(graph_engine: GraphEngine, log_engine: LogEngine) -> Self {
        Self {
            graph_engine,
            log_engine,
        }
    }
}
