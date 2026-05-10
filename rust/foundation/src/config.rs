use config::{Config as ConfigBuilder, Environment, File};
use std::path::{Path, PathBuf};

use crate::error::Result;

#[derive(Debug, Clone)]
pub struct Config {
    pub data_dir: PathBuf,
    pub souls_dir: PathBuf,
    pub archive_dir: PathBuf,
    pub db_path: PathBuf,
    pub registry_path: PathBuf,
    pub call_records_path: PathBuf,
    pub server_host: String,
    pub server_port: u16,
    pub nextjs_port: u16,
    pub searxng_url: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let builder = ConfigBuilder::builder()
            .add_source(File::from(Path::new("config/default.yaml")))
            .add_source(File::from(Path::new("config/local.yaml")).required(false))
            .add_source(Environment::with_prefix("WANMINFAN"));

        let cfg = builder.build()?;

        let data_dir = cfg.get_string("data_dir").unwrap_or_else(|_| "./data".into());

        Ok(Config {
            souls_dir: PathBuf::from(&data_dir).join("souls"),
            archive_dir: PathBuf::from(&data_dir).join("archive"),
            db_path: PathBuf::from(&data_dir).join("wanminfan.db"),
            registry_path: PathBuf::from(&data_dir).join("registry.yaml"),
            call_records_path: PathBuf::from(&data_dir).join("call-records.yaml"),
            data_dir: PathBuf::from(data_dir),
            server_host: cfg.get_string("server_host").unwrap_or_else(|_| "127.0.0.1".into()),
            server_port: cfg.get_int("server_port").map(|p| p as u16).unwrap_or(3001),
            nextjs_port: cfg.get_int("nextjs_port").map(|p| p as u16).unwrap_or(3000),
            searxng_url: cfg.get_string("searxng_url").unwrap_or_else(|_| "http://127.0.0.1:8080".into()),
        })
    }
}
