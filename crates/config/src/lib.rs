use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub storage: StorageSettings,
    pub processing: ProcessingSettings,
    pub llm: LlmSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseSettings {
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageSettings {
    pub upload_dir: String,
    pub max_file_size_mb: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProcessingSettings {
    pub max_concurrent: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmSettings {
    pub model_path: String,
    pub model_name: String,
    pub context_size: u32,
    pub threads: u32,
    pub gpu_layers: u32,
}

impl Settings {
    pub fn load() -> Result<Self, config::ConfigError> {
        dotenvy::dotenv().ok();

        let config = config::Config::builder()
            .add_source(config::File::with_name("config/default"))
            .add_source(
                config::Environment::with_prefix("HARVEX")
                    .prefix_separator("__")
                    .separator("__"),
            )
            .build()?;

        config.try_deserialize()
    }
}
