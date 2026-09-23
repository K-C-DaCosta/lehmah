use serde::Deserialize;
use std::{fmt::Debug, fs, io, path::PathBuf};

#[derive(Deserialize, Debug)]
pub struct EnvConfig {
    pub oxypress_web_root: PathBuf,
    pub oxypress_web_enable_logging: bool,
    pub oxypress_web_http_port: u32,
    pub oxypress_web_https_port: u32,
    pub oxypress_web_cert_directory: PathBuf,
    pub oxypress_database_url: String,
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
            ("OXYPRESS_ENV_DIR", "./.env.local.template"),
            ("OXYPRESS_WEB_CWD_RELATIVE", "./oxypress_web_server"),
            ("OXYPRESS_WEB_ENABLE_LOGGING", "false"),
            ("OXYPRESS_WEB_HTTP_PORT", "8080"),
            ("OXYPRESS_WEB_HTTPS_PORT", "8081"),
            ("OXYPRESS_WEB_CERT_DIRECTORY", "./generated_local_certs"),
            ("OXYPRESS_WEB_HOSTNAME", "khadeemdacosta.ca"),
            ("OXYPRESS_WEB_ALT_HOSTNAMES", "khadeemdacosta.ca"),
            (
                "OXYPRESS_DATABASE_URL",
                "postgres://testdev:testdev@oxypress_web_server_db:5432/oxypress_dev",
            ),
        ]
        .iter()
        .copied()
        .collect::<HashMap<_, _>>();

        unsafe {
            for (key, val) in config_template {
                std::env::set_var(key, val);
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
