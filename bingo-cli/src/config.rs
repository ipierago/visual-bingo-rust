use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub settings: Settings,
    pub images: Vec<ImageEntry>,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub seed: String,
    pub card_count: usize,
    pub output: String,
}

#[derive(Debug, Deserialize)]
pub struct ImageEntry {
    pub path: String,
    pub label: String,
}
