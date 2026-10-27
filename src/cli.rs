//! Command-line interface: argument parsing and the command table.
//!
//! No arguments starts the server (see `main.rs`). Parsing is separate from running so it can be
//! tested without loading the dictionary or opening a socket.

use crate::api::{self, AppState};
use crate::config;
use crate::romanizer::{Loanwords, Romanizer};
use crate::utils::Result;
use std::io::BufRead;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;

pub const USAGE: &str = "pronounce-api\n\n\
Usage: pronounce-api [command]\n\n\
(no command)                           Start the server (same as `serve`)\n\
serve                                  Start the server\n\
    --config <path>      Config file (default: config.toml in the current directory;\n\
                         a file that does not exist means the defaults)\n\
    --host <address>     Address to listen on (default: [server] host, 127.0.0.1)\n\
    --port <port>        Port to listen on (default: [server] port, 8000)\n\
convert [text]...                      Print the romaji of the text. Without text, every line of the\n\
                                       standard input is converted and printed as a line\n\
    --loanwords <romaji|english>       How to write loanwords spelled in hiragana (default: romaji)\n\
    --config <path>      Config file, for [exceptions]\n\
--help, -h                             Show this help\n\n\
GET /pronounce?q=<text>[&loanwords=romaji|english]   the romaji as text/plain\n\
GET /health                                          ok";

#[derive(Debug, PartialEq)]
pub enum Command {
    Serve { config: Option<PathBuf>, host: Option<String>, port: Option<u16> },
    Convert { texts: Vec<String>, loanwords: Loanwords, config: Option<PathBuf> },
    Help,
}

pub fn parse(args: &[String]) -> Result<Command> {
    let Some(first) = args.first() else {
        return Ok(Command::Serve { config: None, host: None, port: None });
    };
    let rest = &args[1..];

    match first.as_str() {
        "--help" | "-h" | "help" => Ok(Command::Help),
        "serve" => parse_serve(rest),
        "convert" => parse_convert(rest),
        // options with no command are the options of `serve`
        arg if arg.starts_with("--") => parse_serve(args),
        other => Err(format!("unknown command: {other}\n\n{USAGE}").into()),
    }
}

fn parse_serve(args: &[String]) -> Result<Command> {
    let (mut config, mut host, mut port) = (None, None, None);
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--config" => config = Some(PathBuf::from(value(&mut iter, arg)?)),
            "--host" => host = Some(value(&mut iter, arg)?),
            "--port" => {
                let text = value(&mut iter, arg)?;
                port = Some(text.parse::<u16>().map_err(|_| format!("--port needs a port number, got {text}"))?);
            }
            other => return Err(format!("unknown option: {other}\n\n{USAGE}").into()),
        }
    }

    Ok(Command::Serve { config, host, port })
}

fn parse_convert(args: &[String]) -> Result<Command> {
    let (mut texts, mut loanwords, mut config) = (Vec::new(), Loanwords::Romaji, None);
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--loanwords" => {
                loanwords = match value(&mut iter, arg)?.as_str() {
                    "romaji" => Loanwords::Romaji,
                    "english" => Loanwords::English,
                    other => return Err(format!("--loanwords is romaji or english, got {other}").into()),
                }
            }
            "--config" => config = Some(PathBuf::from(value(&mut iter, arg)?)),
            other => texts.push(other.to_string()),
        }
    }

    Ok(Command::Convert { texts, loanwords, config })
}

fn value(iter: &mut std::slice::Iter<String>, option: &str) -> Result<String> {
    iter.next().cloned().ok_or_else(|| format!("{option} needs a value").into())
}

pub async fn run(command: Command) -> Result<()> {
    match command {
        Command::Help => {
            println!("{USAGE}");
            Ok(())
        }
        Command::Serve { config, host, port } => serve(config, host, port).await,
        Command::Convert { texts, loanwords, config } => convert(texts, loanwords, config),
    }
}

async fn serve(config: Option<PathBuf>, host: Option<String>, port: Option<u16>) -> Result<()> {
    let config = config::load(&config.unwrap_or_else(config::default_path))?;
    let host = host.unwrap_or(config.server.host.clone());
    let port = port.unwrap_or(config.server.port);

    log::info!("loading the dictionary");
    let state = Arc::new(AppState {
        romanizer: Arc::new(Romanizer::new(&config.exceptions)?),
        max_query_chars: config.convert.max_query_chars,
    });

    // an IPv6 address has to be in brackets
    let address = if host.contains(':') { format!("[{host}]:{port}") } else { format!("{host}:{port}") };
    let listener = TcpListener::bind(&address)
        .await
        .map_err(|e| format!("cannot listen on {address}: {e}"))?;
    log::info!("listening on http://{}", listener.local_addr()?);

    axum::serve(listener, api::router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    log::info!("stopped");
    Ok(())
}

fn convert(texts: Vec<String>, loanwords: Loanwords, config: Option<PathBuf>) -> Result<()> {
    let config = config::load(&config.unwrap_or_else(config::default_path))?;
    let romanizer = Romanizer::new(&config.exceptions)?;

    if texts.is_empty() {
        for line in std::io::stdin().lock().lines() {
            println!("{}", romanizer.romanize(&line?, loanwords)?);
        }
    } else {
        for text in texts {
            println!("{}", romanizer.romanize(&text, loanwords)?);
        }
    }

    Ok(())
}

/// Ctrl+C, and on Unix SIGTERM too (what `systemctl stop` and `docker stop` send).
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.ok();
    };

    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut signal) = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            signal.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn no_arguments_starts_the_server() {
        assert_eq!(parse(&[]).unwrap(), Command::Serve { config: None, host: None, port: None });
    }

    #[test]
    fn serve_options() {
        let command = parse(&args(&["serve", "--host", "0.0.0.0", "--port", "9000", "--config", "a.toml"])).unwrap();
        assert_eq!(
            command,
            Command::Serve {
                config: Some(PathBuf::from("a.toml")),
                host: Some("0.0.0.0".into()),
                port: Some(9000)
            }
        );
        // options alone are the options of `serve`
        assert_eq!(
            parse(&args(&["--port", "1"])).unwrap(),
            Command::Serve { config: None, host: None, port: Some(1) }
        );
    }

    #[test]
    fn convert_options() {
        assert_eq!(
            parse(&args(&["convert", "--loanwords", "english", "あ", "い"])).unwrap(),
            Command::Convert { texts: args(&["あ", "い"]), loanwords: Loanwords::English, config: None }
        );
        assert_eq!(
            parse(&args(&["convert"])).unwrap(),
            Command::Convert { texts: vec![], loanwords: Loanwords::Romaji, config: None }
        );
    }

    #[test]
    fn mistakes_are_errors() {
        assert!(parse(&args(&["nope"])).is_err());
        assert!(parse(&args(&["serve", "--port"])).is_err());
        assert!(parse(&args(&["serve", "--port", "x"])).is_err());
        assert!(parse(&args(&["serve", "--port", "70000"])).is_err());
        assert!(parse(&args(&["serve", "--bogus"])).is_err());
        assert!(parse(&args(&["convert", "--loanwords", "latin"])).is_err());
    }

    #[test]
    fn help() {
        assert_eq!(parse(&args(&["--help"])).unwrap(), Command::Help);
        assert_eq!(parse(&args(&["-h"])).unwrap(), Command::Help);
    }
}
