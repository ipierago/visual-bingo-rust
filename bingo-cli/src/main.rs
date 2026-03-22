mod config;

use anyhow::{Context, Result};
use bingo_core::{generate, generate_pdf, GenerateRequest, ImageData, ImageItem, PdfOptions};
use config::Config;
use std::{env, fs};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let config_path = args.get(1).map(String::as_str).unwrap_or("bingo.toml");

    let config_str = fs::read_to_string(config_path)
        .with_context(|| format!("Could not read config file: {}", config_path))?;

    let config: Config =
        toml::from_str(&config_str).with_context(|| "Failed to parse config file")?;

    println!("Seed:    {}", config.settings.seed);
    println!("Cards:   {}", config.settings.card_count);
    println!("Images:  {}", config.images.len());
    println!("Output:  {}", config.settings.output);

    // Load image bytes from disk
    let image_data: Vec<ImageData> = config
        .images
        .iter()
        .map(|e| {
            let bytes = fs::read(&e.path)
                .with_context(|| format!("Could not read image: {}", e.path))
                .unwrap();
            ImageData {
                id: e.label.clone(),
                bytes,
                is_png: e.path.to_lowercase().ends_with(".png"),
            }
        })
        .collect();

    let images: Vec<ImageItem> = config
        .images
        .iter()
        .map(|e| ImageItem {
            id: e.label.clone(),
            label: e.label.clone(),
            url: e.path.clone(),
        })
        .collect();

    let req = GenerateRequest {
        images,
        seed: config.settings.seed.clone(),
        card_count: config.settings.card_count,
    };

    let resp = generate(&req);

    println!("\nGenerating PDF...");

    let pdf_bytes = generate_pdf(
        &PdfOptions {
            cards: resp.cards,
            call_list: resp.call_list,
        },
        &image_data,
    );

    fs::write(&config.settings.output, pdf_bytes)
        .with_context(|| format!("Could not write output: {}", config.settings.output))?;

    println!("Written to {}", config.settings.output);
    Ok(())
}
