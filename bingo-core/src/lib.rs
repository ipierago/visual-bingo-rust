mod generate;
mod shuffle;

pub use generate::{generate, generate_call_list, generate_cards};

use serde::{Deserialize, Serialize};

/// A single image in the library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageItem {
    pub id: String,
    pub label: String,
    pub url: String,
}

/// A single bingo card — always 25 cells
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BingoCard {
    pub cells: Vec<ImageItem>,
}

/// Input to the generator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub images: Vec<ImageItem>,
    pub seed: String,
    pub card_count: usize,
}

/// Output from the generator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResponse {
    pub cards: Vec<BingoCard>,
    pub call_list: Vec<ImageItem>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_images(n: usize) -> Vec<ImageItem> {
        (0..n)
            .map(|i| ImageItem {
                id: format!("img-{}", i),
                label: format!("image {}", i),
                url: format!("http://example.com/{}.jpg", i),
            })
            .collect()
    }

    #[test]
    fn generates_correct_card_count() {
        let req = GenerateRequest {
            images: make_images(30),
            seed: "test-seed".into(),
            card_count: 5,
        };
        let resp = generate(&req);
        assert_eq!(resp.cards.len(), 5);
    }

    #[test]
    fn each_card_has_25_cells() {
        let req = GenerateRequest {
            images: make_images(30),
            seed: "test-seed".into(),
            card_count: 3,
        };
        let resp = generate(&req);
        for card in &resp.cards {
            assert_eq!(card.cells.len(), 25);
        }
    }

    #[test]
    fn deterministic_with_same_seed() {
        let images = make_images(30);
        let req = GenerateRequest {
            images: images.clone(),
            seed: "same-seed".into(),
            card_count: 2,
        };
        let r1 = generate(&req);
        let r2 = generate(&req);
        assert_eq!(r1.cards[0].cells[0].id, r2.cards[0].cells[0].id);
        assert_eq!(r1.cards[1].cells[24].id, r2.cards[1].cells[24].id);
    }

    #[test]
    fn different_seeds_give_different_cards() {
        let images = make_images(30);
        let r1 = generate(&GenerateRequest {
            images: images.clone(),
            seed: "seed-a".into(),
            card_count: 1,
        });
        let r2 = generate(&GenerateRequest {
            images: images.clone(),
            seed: "seed-b".into(),
            card_count: 1,
        });
        assert_ne!(r1.cards[0].cells[0].id, r2.cards[0].cells[0].id);
    }

    #[test]
    fn no_duplicate_cells_on_a_card() {
        let req = GenerateRequest {
            images: make_images(30),
            seed: "test-seed".into(),
            card_count: 1,
        };
        let resp = generate(&req);
        let ids: std::collections::HashSet<_> = resp.cards[0].cells.iter().map(|c| &c.id).collect();
        assert_eq!(ids.len(), 25);
    }
}
