use std::env::{VarError, var};
use anyhow::anyhow;

pub struct Config {
    pub db_path:     String,
    pub socket_path: String,
    pub live_port:   u16,
    pub query_port:  u16,
}

impl Config {
    pub fn import() -> anyhow::Result<Self> {
        let db_path     = import_var("QITECH_TELEMETRY_DB_PATH")?;
        let socket_path = import_var("QITECH_TELEMETRY_SOCKET_PATH")?;
        let live_port   = import_var("QITECH_TELEMETRY_LIVE_PORT")?.parse::<u16>()?;
        let query_port  = import_var("QITECH_TELEMETRY_QUERY_PORT")?.parse::<u16>()?;

        Ok(Self { 
            db_path, 
            socket_path, 
            live_port, 
            query_port 
        })
    }
}

fn import_var(name: &str) -> anyhow::Result<String> {
    match var(name) {
        Ok(v) => Ok(v),
        Err(VarError::NotPresent) => {
            Err(anyhow!("Env Var {name} not provided."))
        },
        Err(VarError::NotUnicode(_)) => {
            Err(anyhow!("Env Var {name} not unicode."))
        },
    }
}