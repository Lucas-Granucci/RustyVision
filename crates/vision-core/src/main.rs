mod config;
use config::Config;

fn main() {
    println!("RustyVision waking up...");

    let config = match Config::load_default() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Could not find configuration file: {}", e);
            eprintln!("Using default configuration");
            Config::default()
        }
    };
}
