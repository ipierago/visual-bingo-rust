mod config;

use anyhow::{Context, Result};
use bingo_core::{generate, GenerateRequest, ImageItem};
use config::Config;
use std::{env, fs};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let config_path = args.get(1).map(String::as_str).unwrap_or("bingo.toml");

    let config_str = fs::read_to_string(config_path)
        .with_context(|| format!("Could not read config file: {}", config_path))?;

    let config: Config = toml::from_str(&config_str)
        .with_context(|| "Failed to parse config file")?;

    println!("Seed:       {}", config.settings.seed);
    println!("Cards:      {}", config.settings.card_count);
    println!("Images:     {}", config.images.len());
    println!("Output:     {}", config.settings.output);

    let images: Vec<ImageItem> = config.images.iter().map(|e| ImageItem {
        id: e.label.clone(),
        label: e.label.clone(),
        url: e.path.clone(),
    }).collect();

    let req = GenerateRequest {
        images,
        seed: config.settings.seed.clone(),
        card_count: config.settings.card_count,
    };

    let resp = generate(&req);

    println!("\nGenerated {} cards", resp.cards.len());
    println!("Call list has {} images", resp.call_list.len());

    println!("\nCard 1 preview:");
    for (i, cell) in resp.cards[0].cells.iter().enumerate() {
        print!("  {:>2}. {:<20}", i + 1, cell.label);
        if (i + 1) % 5 == 0 { println!() }
    }

    println!("\nDone. (PDF output coming soon)");
    Ok(())
}