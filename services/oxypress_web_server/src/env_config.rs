use serde::Deserialize;
use std::{fmt::Debug, fs, io, path::PathBuf};

#[derive(Deserialize, Debug, Clone)]
pub struct EnvConfig {
    pub oxypress_web_root: PathBuf,
    pub oxypress_web_enable_logging: bool,
    pub oxypress_web_http_port: u16,
    pub oxypress_web_https_port: u16,
    pub oxypress_web_cert_directory: PathBuf,
    pub oxypress_web_database_url: String,
    pub oxypress_web_hostname: String,
}

impl EnvConfig {
    pub fn get_config_variables() -> Self {
        envy::from_env::<Self>().expect("Found unexpected enviroment variables.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn config_example_values() {
        let config_template = [
            (
                "OXYPRESS_WEB_ROOT",
                "./services/oxypress_web_server/resources",
            ),
            ("OXYPRESS_WEB_ENABLE_LOGGING", "false"),
            ("OXYPRESS_WEB_HTTP_PORT", "8080"),
            ("OXYPRESS_WEB_HTTPS_PORT", "8081"),
            ("OXYPRESS_WEB_CERT_DIRECTORY", "./generated_local_certs"),
            ("OXYPRESS_WEB_HOSTNAME", "khadeemdacosta.ca"),
            (
                "OXYPRESS_WEB_DATABASE_URL",
                "postgres://testdev:testdev@oxypress_web_server_db:5432/oxypress_dev",
            ),
        ]
        .iter()
        .copied()
        .collect::<HashMap<_, _>>();

        unsafe {
            for (env_var_ident, env_ident_val) in config_template {
                std::env::set_var(env_var_ident, env_ident_val);
            }
        }

        let cfg = EnvConfig::get_config_variables();
        assert_eq!(
            cfg.oxypress_web_root
                .as_os_str()
                .to_str()
                .expect("failed to convert to string"),
            "./services/oxypress_web_server/resources"
        );
    }
}
