use crate::shuffle::shuffle;
use crate::{BingoCard, GenerateRequest, GenerateResponse, ImageItem};

const CELLS_PER_CARD: usize = 25;

pub fn generate_cards(req: &GenerateRequest) -> Vec<BingoCard> {
    assert!(
        req.images.len() >= CELLS_PER_CARD,
        "Need at least {} images, got {}",
        CELLS_PER_CARD,
        req.images.len()
    );

    (0..req.card_count)
        .map(|i| {
            let card_seed = format!("{}-{}", req.seed, i);
            let cells = shuffle(&req.images, &card_seed)
                .into_iter()
                .take(CELLS_PER_CARD)
                .collect();
            BingoCard { cells }
        })
        .collect()
}

pub fn generate_call_list(req: &GenerateRequest) -> Vec<ImageItem> {
    let call_seed = format!("{}-calllist", req.seed);
    shuffle(&req.images, &call_seed)
}

pub fn generate(req: &GenerateRequest) -> GenerateResponse {
    GenerateResponse {
        cards: generate_cards(req),
        call_list: generate_call_list(req),
    }
}
