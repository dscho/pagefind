use pagefind::{PagefindInboundConfig, SearchOptions, SearchState};
use std::time::Instant;
use twelf::reexports::clap::CommandFactory;
use twelf::Layer;

const CONFIGS: &[&str] = &[
    "pagefind.json",
    "pagefind.yml",
    "pagefind.yaml",
    "pagefind.toml",
];

#[tokio::main]
async fn main() {
    let start = Instant::now();

    let matches = PagefindInboundConfig::command()
        // .ignore_errors(true)
        .get_matches();

    let mut config_layers = vec![];

    let configs: Vec<&str> = CONFIGS
        .iter()
        .filter(|c| std::path::Path::new(c).exists())
        .cloned()
        .collect();
    if configs.len() > 1 {
        eprintln!(
            "Found multiple possible config files: [{}]",
            configs.join(", ")
        );
        eprintln!("Pagefind only supports loading one configuration file format, please ensure only one file exists.");
        return;
    }

    for config in configs {
        let layer_fn = if config.ends_with("json") {
            Layer::Json
        } else if config.ends_with("toml") {
            Layer::Toml
        } else if config.ends_with("yaml") || config.ends_with("yml") {
            Layer::Yaml
        } else {
            panic!("Unknown config file format");
        };
        config_layers.push(layer_fn(config.into()));
    }

    config_layers.push(Layer::Env(Some("PAGEFIND_".to_string())));
    config_layers.push(Layer::Clap(matches));

    match PagefindInboundConfig::with_layers(&config_layers) {
        Ok(config) => {
            if let Ok(options) = SearchOptions::load(config) {
                let mut runner = SearchState::new(options);

                runner.run().await;

                let duration = start.elapsed();
                println!(
                    "Finished in {}.{} seconds",
                    duration.as_secs(),
                    duration.subsec_millis()
                );
            }
        }
        Err(e) => {
            eprintln!("Error loading Pagefind config:");
            match e {
                twelf::Error::Io(e) => {
                    eprintln!("{}", e);
                }
                twelf::Error::Envy(e) => {
                    eprintln!("{}", e);
                }
                twelf::Error::Json(e) => {
                    eprintln!("{}", e);
                }
                twelf::Error::Toml(e) => {
                    eprintln!("{}", e);
                }
                twelf::Error::Yaml(e) => {
                    eprintln!("{}", e);
                }
                twelf::Error::Deserialize(e) => {
                    eprintln!("{}", e);
                }
                _ => {
                    eprintln!("Unknown Error");
                }
            }
        }
    }
}
