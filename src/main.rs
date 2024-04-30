use std::env;
use std::path::PathBuf;

use clap::Parser;
use home::home_dir;
use path_clean::PathClean;

use crate::app::App;
use crate::constants::CONFIG_DIR;

mod events;
mod config;
mod app;
mod logs;
mod hash;
mod auth;
mod helpers;
mod constants;
mod server;
mod files;

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_missing_value = None)]
    config: Option<String>,
}

fn resolve_config_dir(config: Option<String>) -> PathBuf {
    match config {
        Some(config) => {
            let path = PathBuf::from(config);
            if path.is_absolute() {
                path
            } else {
                env::current_dir().unwrap().join(path)
            }.clean()
        }
        None => home_dir().unwrap().join(CONFIG_DIR)
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let args = Args::parse();

    let config_dir = resolve_config_dir(args.config);

    let app = App::new(&config_dir).await;
    if app.is_err() { return Err("Demon start failed".to_string()); }
    let mut app = app.unwrap();

    app.listen().await;

    Ok(())
}
