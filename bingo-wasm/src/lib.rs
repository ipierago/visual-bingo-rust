use bingo_core::{GenerateRequest, ImageData, PdfOptions, generate, generate_pdf};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Takes a JSON GenerateRequest, returns a JSON GenerateResponse
#[wasm_bindgen]
pub fn generate_cards(request_json: &str) -> Result<String, JsValue> {
    let request: GenerateRequest =
        serde_json::from_str(request_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let response = generate(&request);

    serde_json::to_string(&response).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Takes a JSON GenerateRequest and a JS array of image objects,
/// returns PDF bytes as a Uint8Array
#[wasm_bindgen]
pub fn generate_pdf_wasm(request_json: &str, images_json: &str) -> Result<Vec<u8>, JsValue> {
    let request: GenerateRequest =
        serde_json::from_str(request_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let image_entries: Vec<WasmImageData> =
        serde_json::from_str(images_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let response = generate(&request);

    let image_data: Vec<ImageData> = image_entries
        .into_iter()
        .map(|e| ImageData {
            id: e.id,
            bytes: e.bytes,
            is_png: e.is_png,
        })
        .collect();

    let pdf_bytes = generate_pdf(
        &PdfOptions {
            cards: response.cards,
            call_list: response.call_list,
        },
        &image_data,
    );

    Ok(pdf_bytes)
}

#[derive(serde::Deserialize)]
struct WasmImageData {
    id: String,
    bytes: Vec<u8>,
    is_png: bool,
}
