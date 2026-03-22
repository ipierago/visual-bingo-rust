use image::GenericImageView;
use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref, Str};
use std::collections::HashMap;

use crate::{BingoCard, ImageItem};

// A4 in PDF points (1 point = 1/72 inch)
const A4_W: f32 = 595.0;
const A4_H: f32 = 842.0;
const MARGIN: f32 = 20.0;
const GRID: usize = 5;
const CALL_LIST_COLS: usize = 3;
const CALL_LIST_ROWS: usize = 8;
const CALL_LIST_PER_PAGE: usize = CALL_LIST_COLS * CALL_LIST_ROWS;

pub struct ImageData {
    pub id: String,
    pub bytes: Vec<u8>,
    pub is_png: bool,
}

pub struct PdfOptions {
    pub cards: Vec<BingoCard>,
    pub call_list: Vec<ImageItem>,
}

// Decoded image dimensions + pixel data for embedding
struct DecodedImage {
    width: u32,
    height: u32,
    // RGB bytes (no alpha — we convert to RGB)
    samples: Vec<u8>,
}

fn decode_image(data: &ImageData) -> Option<DecodedImage> {
    let img = image::load_from_memory(&data.bytes).ok()?;
    let (width, height) = img.dimensions();

    // Flatten alpha against white background
    let img_rgba = img.to_rgba8();
    let mut samples = Vec::with_capacity((width * height * 3) as usize);

    for pixel in img_rgba.pixels() {
        let alpha = pixel[3] as f32 / 255.0;
        let r = (pixel[0] as f32 * alpha + 255.0 * (1.0 - alpha)) as u8;
        let g = (pixel[1] as f32 * alpha + 255.0 * (1.0 - alpha)) as u8;
        let b = (pixel[2] as f32 * alpha + 255.0 * (1.0 - alpha)) as u8;
        samples.push(r);
        samples.push(g);
        samples.push(b);
    }

    Some(DecodedImage {
        width,
        height,
        samples,
    })
}

pub fn generate_pdf(options: &PdfOptions, image_data: &[ImageData]) -> Vec<u8> {
    let mut pdf = Pdf::new();

    let catalog_id = Ref::new(1);
    let page_tree_id = Ref::new(2);

    // Decode and index all images
    let image_map: HashMap<&str, &ImageData> =
        image_data.iter().map(|d| (d.id.as_str(), d)).collect();

    let mut decoded: HashMap<&str, DecodedImage> = HashMap::new();
    for (id, data) in &image_map {
        if let Some(dec) = decode_image(data) {
            decoded.insert(id, dec);
        }
    }

    let card_count = options.cards.len();
    let call_list_pages = call_list_page_count(&options.call_list);
    let total_pages = card_count + call_list_pages;

    // Ref layout:
    // 1          = catalog
    // 2          = page tree
    // 3..P+2     = pages (cards then call list)
    // P+3..P+3+C = content streams (one per page)
    // P+3+C..    = image xobjects
    let first_page_ref = 3i32;
    let first_content_ref = first_page_ref + total_pages as i32;
    let first_image_ref = first_content_ref + total_pages as i32;

    // After first_image_ref calculation
    let font_ref = Ref::new(first_image_ref + decoded.len() as i32);

    // Assign a stable ref to each unique image
    let mut image_refs: HashMap<&str, Ref> = HashMap::new();
    for (i, id) in decoded.keys().enumerate() {
        image_refs.insert(id, Ref::new(first_image_ref + i as i32));
    }

    // Catalog + page tree
    pdf.catalog(catalog_id).pages(page_tree_id);
    let page_ids: Vec<Ref> = (0..total_pages)
        .map(|i| Ref::new(first_page_ref + i as i32))
        .collect();
    pdf.pages(page_tree_id)
        .kids(page_ids.iter().copied())
        .count(total_pages as i32);

    // Card pages
    for (i, card) in options.cards.iter().enumerate() {
        let page_id = Ref::new(first_page_ref + i as i32);
        let content_id = Ref::new(first_content_ref + i as i32);
        write_card_page(
            &mut pdf,
            page_id,
            page_tree_id,
            content_id,
            card,
            &image_refs,
            &decoded,
        );
    }

    // Call list pages
    let per_page = 3 * 8;
    for (pi, chunk) in options.call_list.chunks(CALL_LIST_PER_PAGE).enumerate() {
        let page_id = Ref::new(first_page_ref + card_count as i32 + pi as i32);
        let content_id = Ref::new(first_content_ref + card_count as i32 + pi as i32);
        write_call_list_page(
            &mut pdf,
            page_id,
            page_tree_id,
            content_id,
            font_ref,
            chunk,
            pi,
        );
    }
    // Image XObjects
    for (id, dec) in &decoded {
        let img_ref = image_refs[id];
        let mut image = pdf.image_xobject(img_ref, &dec.samples);
        image.width(dec.width as i32);
        image.height(dec.height as i32);
        image.color_space().device_rgb();
        image.bits_per_component(8);
        image.finish();
    }

    // After writing image XObjects
    pdf.type1_font(font_ref).base_font(Name(b"Helvetica"));

    pdf.finish()
}

fn write_card_page(
    pdf: &mut Pdf,
    page_id: Ref,
    page_tree_id: Ref,
    content_id: Ref,
    card: &BingoCard,
    image_refs: &HashMap<&str, Ref>,
    decoded: &HashMap<&str, DecodedImage>,
) {
    let cell_w = (A4_W - MARGIN * 2.0) / GRID as f32;
    let cell_h = (A4_H - MARGIN * 2.0) / GRID as f32;
    let pad = 4.0;

    // Collect image names used on this page for resources
    let used: Vec<(&str, Ref)> = card
        .cells
        .iter()
        .filter_map(|cell| {
            image_refs
                .get(cell.id.as_str())
                .map(|r| (cell.id.as_str(), *r))
        })
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    // Page
    let mut page = pdf.page(page_id);
    page.parent(page_tree_id)
        .media_box(Rect::new(0.0, 0.0, A4_W, A4_H));

    let mut resources = page.resources();
    let mut x_objects = resources.x_objects();
    for (id, img_ref) in &used {
        let name = image_name(id);
        x_objects.pair(Name(name.as_bytes()), *img_ref);
    }
    x_objects.finish();
    resources.finish();
    page.contents(content_id);
    page.finish();

    // Content stream
    let mut content = Content::new();

    for (idx, cell) in card.cells.iter().enumerate() {
        let col = idx % GRID;
        let row = idx / GRID;
        let x = MARGIN + col as f32 * cell_w;
        let y = A4_H - MARGIN - (row + 1) as f32 * cell_h;

        // Draw cell border
        content
            .set_stroke_rgb(0.7, 0.7, 0.7)
            .set_line_width(0.5)
            .rect(x, y, cell_w, cell_h)
            .stroke();

        // Draw image if available
        if let Some(dec) = decoded.get(cell.id.as_str()) {
            let max_w = cell_w - pad * 2.0;
            let max_h = cell_h - pad * 2.0;
            let scale = (max_w / dec.width as f32).min(max_h / dec.height as f32);
            let img_w = dec.width as f32 * scale;
            let img_h = dec.height as f32 * scale;
            let img_x = x + (cell_w - img_w) / 2.0;
            let img_y = y + (cell_h - img_h) / 2.0;

            let name = image_name(&cell.id);
            content
                .save_state()
                .transform([img_w, 0.0, 0.0, img_h, img_x, img_y])
                .x_object(Name(name.as_bytes()))
                .restore_state();
        }
    }

    pdf.stream(content_id, &content.finish());
}

fn write_call_list_page(
    pdf: &mut Pdf,
    page_id: Ref,
    page_tree_id: Ref,
    content_id: Ref,
    font_ref: Ref,
    items: &[ImageItem],
    page_index: usize,
) {
    let col_w = (A4_W - MARGIN * 2.0) / CALL_LIST_COLS as f32;
    let row_h = (A4_H - MARGIN * 2.0) / CALL_LIST_ROWS as f32;
    let font_size = 11.0f32;

    let mut page = pdf.page(page_id);
    page.parent(page_tree_id)
        .media_box(Rect::new(0.0, 0.0, A4_W, A4_H));

    let mut resources = page.resources();
    resources.fonts().pair(Name(b"F1"), font_ref);
    resources.finish();

    page.contents(content_id);
    page.finish();

    let mut content = Content::new();
    content.set_fill_rgb(0.1, 0.1, 0.1);

    for (i, item) in items.iter().enumerate() {
        let col = i % CALL_LIST_COLS;
        let row = i / CALL_LIST_COLS;
        let x = MARGIN + col as f32 * col_w;
        let y = A4_H - MARGIN - (row + 1) as f32 * row_h + row_h / 2.0;

        let number = page_index * CALL_LIST_PER_PAGE + i + 1;
        let label = format!("{}. {}", number, item.label);

        content
            .begin_text()
            .set_font(Name(b"F1"), font_size)
            .set_text_matrix([1.0, 0.0, 0.0, 1.0, x, y])
            .show(Str(label.as_bytes()))
            .end_text();
    }

    pdf.stream(content_id, &content.finish());
}

fn call_list_page_count(call_list: &[ImageItem]) -> usize {
    call_list.len().div_ceil(CALL_LIST_PER_PAGE)
}
// Convert an image id to a safe PDF resource name like "Im_cat"
fn image_name(id: &str) -> String {
    let safe = id.replace(['/', ' ', '-'], "_");
    format!("Im_{}", safe)
}
