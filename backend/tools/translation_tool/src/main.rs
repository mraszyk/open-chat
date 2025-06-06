use clap::Parser;
use std::process;
use translation_tool::Config;
use translation_tool::run;

#[tokio::main]
async fn main() {
    let config = Config::parse();

    if let Err(e) = run(config).await {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
