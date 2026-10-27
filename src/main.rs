mod api;
mod cli;
mod config;
mod romaji;
mod romanizer;
#[cfg(test)]
mod testing;
mod tokenizer;
mod utils;

use std::env;
use std::process::exit;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let result = match cli::parse(&args) {
        Ok(command) => cli::run(command).await,
        Err(e) => Err(e),
    };
    if let Err(e) = result {
        eprintln!("error: {}", e);
        exit(1);
    }
}
