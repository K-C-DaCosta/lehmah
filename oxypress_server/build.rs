use oxypress_core::{log::ConsoleColors, loggy};
use serde::Serialize;
use std::{
    fs::{self, File},
    io::Write,
    os::unix::process::CommandExt,
    path::Path,
    process::Command,
};
use tera::{Context, Tera};

const CONFIG_TEMPLATE_SRC: &str = "./config_templates";
const CONFIG_TEMPLATE_DST: &str = "./generated_configs";

struct EtcHostRecord<'a> {
    _ip: &'a str,
    hostname: &'a str,
}

#[derive(Serialize, Clone, Copy)]
struct WebServerSelfSignedSSLConfigContext<'a> {
    oxypress_web_server_hostname: &'a str,
    oxypress_web_server_local_country: &'a str,
    oxypress_web_server_local_state: &'a str,
    oxypress_web_server_local_locality: &'a str,
    oxypress_web_server_local_organization_description: &'a str,
}



fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    // TODO: Hints aren't working. No idea why. Low priority. Investigate later.
    send_hints_to_rerun_this_script_for_all_files_in_expected_directory(CONFIG_TEMPLATE_SRC);
    generate_config_files();
    generate_local_certs();
}

fn send_hints_to_rerun_this_script_for_all_files_in_expected_directory(expected_dir: &str) {
    fs::read_dir(expected_dir)
        .expect("expected dir should exist")
        .filter_map(|possible_dir| possible_dir.ok())
        .filter_map(|dir| dir.path().is_file().then_some(dir))
        .for_each(|file| {
            println!("cargo:rerun-if-changed={}", file.path().display());
        });
}

fn generate_config_files() {
    let template_engine = instantiate_terra_engine_read_all_files_from_config_directory(
        format!("./{}/*", CONFIG_TEMPLATE_SRC).as_str(),
    );
    log_all_compiled_templates(&template_engine);
    generate_oxypress_webservers_local_ssl_configs(&template_engine);
}

fn instantiate_terra_engine_read_all_files_from_config_directory(directory: &str) -> Tera {
    Tera::new(directory).expect("Tera failed to instantiate")
}

fn log_all_compiled_templates(template_engine: &Tera) {
    loggy!(ConsoleColors::GREEN, "Templates compiled:");
    template_engine.get_template_names().for_each(|name| {
        loggy!(ConsoleColors::GREEN, "Name: {}", name);
    });
}

fn generate_oxypress_webservers_local_ssl_configs(template_engine: &Tera) {
    let generated_config_directory = Path::new(CONFIG_TEMPLATE_DST);

    // TODO: hardcoded context. make customizeable with env-vars
    let local_cert_ctx = WebServerSelfSignedSSLConfigContext {
        oxypress_web_server_hostname: "khadeemdacosta.ca",
        oxypress_web_server_local_country: "CA",
        oxypress_web_server_local_state: "ON",
        oxypress_web_server_local_locality: "TO",
        oxypress_web_server_local_organization_description: "Description",
    };
    generate_config_file_at_destination(
        template_engine,
        "local.openssl.cnf.terra",
        tera::Context::from_serialize(local_cert_ctx).unwrap(),
        generated_config_directory,
    );
    #[cfg(target_os = "linux")]
    {
        omit_warning_if_ctx_isnt_fount_in_etc_hosts(&local_cert_ctx);
    }
    #[cfg(not(target_os = "linux"))]
    {
        println!(
            "cargo:warning={}",
            "Unsupported operating system.Build scripts could break"
        );
    }
}

fn omit_warning_if_ctx_isnt_fount_in_etc_hosts(ctx: &WebServerSelfSignedSSLConfigContext) {
    let canonical_hostname = ctx.oxypress_web_server_hostname;
    let conanical_name_found = process_hosts_file(|iterator| {
        for EtcHostRecord { hostname, .. } in iterator {
            if hostname.contains(canonical_hostname) {
                return true;
            }
        }
        false
    });
    if !conanical_name_found {
        let warning = r"
        The canonical hostname wasn't found anywhere in
        /etc/hosts/ make sure it's configured properly otherwise 
        the auto-generated self-signed cert won't work as expected.
        ";
        println!(
            "cargo:warning={}",
            warning
                .trim()
                .lines()
                .flat_map(|line| line
                    .split(char::is_whitespace)
                    .filter(|word| !word.is_empty()))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
}

fn process_hosts_file<F, Out>(callback: F) -> Out
where
    F: FnOnce(&mut dyn Iterator<Item = EtcHostRecord>) -> Out,
{
    let file_contents = std::fs::read_to_string("/etc/hosts").expect("file expected");

    let mut host_iterator = file_contents
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .flat_map(|line| {
            let mut parts = line.split_whitespace();
            let ip = parts.next().unwrap();
            let hostnames = parts;
            hostnames.map(move |hostname| EtcHostRecord { _ip: ip, hostname })
        });

    callback(&mut host_iterator)
}

fn generate_local_certs() {
    let execution_result = Command::new("openssl")
        .args([
            "req",
            "-x509",
            "-newkey",
            "rsa:4096",
            "-nodes",
            "-keyout",
            "./generated_local_certs/key.pem",
            "-out",
            "./generated_local_certs/cert.pem",
            "-days",
            "365",
            "-config",
        ])
        .arg(Path::new(CONFIG_TEMPLATE_DST).join("local.openssl.cnf"))
        .exec();
    println!("keyvalue pairs result: {}", execution_result);
}

fn generate_config_file_at_destination<OutDir: AsRef<Path>>(
    template_engine: &Tera,
    terra_template_file_name: &str,
    terra_template_context: Context,
    output_directory: OutDir,
) {
    let output_file_path = output_directory
        .as_ref()
        .join(Path::new(terra_template_file_name).file_stem().unwrap());

    let mut output_file = File::create(output_file_path)
        .map_err(|err| {
            loggy!(
                ConsoleColors::RED,
                "Failed to create file for {} .terra",
                terra_template_file_name
            );
            loggy!(ConsoleColors::DEFAULT, "Reason:{:?}", err);
        })
        .unwrap();
    let rendered_template = template_engine
        .render(terra_template_file_name, &terra_template_context)
        .unwrap();
    output_file.write_all(rendered_template.as_bytes()).unwrap();
}
