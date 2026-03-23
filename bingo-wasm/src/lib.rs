use bingo_core::{generate, generate_pdf, GenerateRequest, ImageData, PdfOptions};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn generate_cards(request_json: &str) -> Result<String, JsValue> {
    let request: GenerateRequest =
        serde_json::from_str(request_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let response = generate(&request);

    serde_json::to_string(&response).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn generate_pdf_wasm(
    request_json: &str,
    image_ids_json: &str,
    image_is_png_json: &str,
    image_data: &[u8],
    image_offsets: &[u32],
    image_lengths: &[u32],
) -> Result<Vec<u8>, JsValue> {
    let request: GenerateRequest =
        serde_json::from_str(request_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let ids: Vec<String> =
        serde_json::from_str(image_ids_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let is_png: Vec<bool> =
        serde_json::from_str(image_is_png_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let image_data_list: Vec<ImageData> = ids
        .into_iter()
        .zip(is_png)
        .enumerate()
        .map(|(i, (id, is_png))| {
            let offset = image_offsets[i] as usize;
            let length = image_lengths[i] as usize;
            ImageData {
                id,
                bytes: image_data[offset..offset + length].to_vec(),
                is_png,
            }
        })
        .collect();

    let response = generate(&request);

    let pdf_bytes = generate_pdf(
        &PdfOptions {
            cards: response.cards,
            call_list: response.call_list,
        },
        &image_data_list,
    );

    Ok(pdf_bytes)
}
