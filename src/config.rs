use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_database_url")]
    pub database_url: String,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8050
}

fn default_database_url() -> String {
    "postgres://postgres:postgres@localhost:1311/postgres".to_string()
}
