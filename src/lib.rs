mod masking;
mod path_hint;
mod util;

pub type Masking = masking::Masking;

#[derive(Debug, Clone)]
pub struct Config {
    pub api_key: String,
    pub api_id: String,
    pub version_id: String,
    pub port: i32,
}

#[derive(Debug, Clone)]
pub struct SpeakeasySdk {
    config: Config,
    masking: Masking,
}

impl SpeakeasySdk {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            masking: Default::default(),
        }
    }
}
