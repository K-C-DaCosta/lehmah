use rcgen::{generate_simple_self_signed, CertifiedKey};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cert_dir = Path::new("../certs/dev");
    if cert_dir.join("localhost.pem").exists() {
        println!("Dev certs already exist.");
        return Ok(());
    }

    fs::create_dir_all(cert_dir)?;

    let CertifiedKey { cert, key_pair } = generate_simple_self_signed(
        ["localhost", "127.0.0.1"]
            .iter()
            .copied()
            .map(String::from)
            .collect::<Vec<_>>(),
    )?;
    fs::write(cert_dir.join("localhost.pem"), cert.pem())?;
    fs::write(cert_dir.join("localhost-key.pem"), key_pair.serialize_pem())?;
    println!("Successfully generated dev certs in ./certs/dev/");
    Ok(())
}
