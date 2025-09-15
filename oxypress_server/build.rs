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

#[derive(Serialize)]
struct WebServerSSLConfigContext<'a> {
    oxypress_web_server_hostname: &'a str,
    oxypress_web_server_local_country: &'a str,
    oxypress_web_server_local_state: &'a str,
    oxypress_web_server_local_locality: &'a str,
    oxypress_web_server_local_organization_description: &'a str,
}

const CONFIG_TEMPLATE_SRC: &str = "./config_templates";
const CONFIG_TEMPLATE_DST: &str = "./generated_configs";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    // TODO: Hints aren't working. No idea why. Low priority. Investigate later.
    send_hints_to_rerun_this_script_for_all_files_in_expected_directory(CONFIG_TEMPLATE_SRC);
    generate_config_files();

    //openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -config openssl.cnf
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
    println!("{}", execution_result);
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
    output_all_compiled_templates(&template_engine);
    generate_oxypress_webservers_local_ssl_configs(&template_engine);
}

fn instantiate_terra_engine_read_all_files_from_config_directory(directory: &str) -> Tera {
    Tera::new(directory).expect("Tera failed to instantiate")
}

fn output_all_compiled_templates(template_engine: &Tera) {
    loggy!(ConsoleColors::GREEN, "Templates compiled:");
    template_engine.get_template_names().for_each(|name| {
        loggy!(ConsoleColors::GREEN, "Name: {}", name);
    });
}

fn generate_oxypress_webservers_local_ssl_configs(template_engine: &Tera) {
    let generated_config_directory = Path::new(CONFIG_TEMPLATE_DST);

    // TODO: hardcoded context. make customizeable with env-vars
    generate_config_file_at_destination(
        template_engine,
        "local.openssl.cnf.terra",
        tera::Context::from_serialize(WebServerSSLConfigContext {
            oxypress_web_server_hostname: "khadeemdacosta.ca",
            oxypress_web_server_local_country: "CA",
            oxypress_web_server_local_state: "ON",
            oxypress_web_server_local_locality: "TO",
            oxypress_web_server_local_organization_description: "Description",
        })
        .unwrap(),
        generated_config_directory,
    );
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
