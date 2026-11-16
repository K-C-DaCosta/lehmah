use std::{
    error,
    process::{self, Command},
};

const DEFAULT_PG_PORT: usize = 5432;
const SKIP_PROGRAM_NAME: usize = 1;
const EXIT_CODE_FAILURE: i32 = 1;
const LOCALHOST: &str = "127.0.0.1";
const TEST_STD_OUT_ENABLED: bool = false;

fn main() -> Result<(), Box<dyn error::Error>> {
    let postgres_results = check_postgres_availability();

    if let Err(reason) = &postgres_results {
        eprintln!("postgres unavailable ({reason}); compiling against .sqlx cache");
    }

    let test_runner_exit_stats = Command::new("cargo")
        .args(["test", "--target=x86_64-unknown-linux-gnu"])
        .args(std::env::args().skip(SKIP_PROGRAM_NAME))
        .args(if TEST_STD_OUT_ENABLED {
            &["--", "--no-capture"][..]
        } else {
            &[][..]
        })
        .env("SQLX_OFFLINE", postgres_results.is_err().to_string())
        .status()?;
    process::exit(test_runner_exit_stats.code().unwrap_or(EXIT_CODE_FAILURE))
}

pub fn check_postgres_availability() -> Result<(), Box<dyn error::Error>> {
    Command::new("pg_isready")
        .args([
            "-q",
            "-h",
            LOCALHOST,
            "-p",
            DEFAULT_PG_PORT.to_string().as_str(),
        ])
        .status()
        .map_err(|e| {
            format!(
                "Warning: the program pg_isready failed to spawn. It may not actually exist. Details: {:?}",
                e
            )
        })?
        .success()
        .then_some(())
        .ok_or("pg_isready can't find a database".to_string().into())
}
