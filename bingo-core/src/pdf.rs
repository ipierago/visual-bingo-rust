use image::GenericImageView;
use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref};
use std::collections::HashMap;

use crate::{BingoCard, ImageItem};

// A4 in PDF points (1 point = 1/72 inch)
const A4_W: f32 = 595.0;
const A4_H: f32 = 842.0;
const MARGIN: f32 = 20.0;
const GRID: usize = 5;

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
    let samples = img.to_rgb8().into_raw();
    Some(DecodedImage {
        width,
        height,
        samples,
    })
}

pub fn generate_pdf(options: &PdfOptions, image_data: &[ImageData]) -> Vec<u8> {
    let mut pdf = Pdf::new();

    // Allocate reference IDs
    // IDs: 1 = catalog, 2 = page tree, 3..N = pages, then images
    let catalog_id = Ref::new(1);
    let page_tree_id = Ref::new(2);

    let page_count = options.cards.len() + call_list_page_count(&options.call_list);
    let first_page_id = 3i32;

    // Decode and index all images
    let image_map: HashMap<&str, &ImageData> =
        image_data.iter().map(|d| (d.id.as_str(), d)).collect();

    let mut decoded: HashMap<&str, DecodedImage> = HashMap::new();
    for (id, data) in &image_map {
        if let Some(dec) = decode_image(data) {
            decoded.insert(id, dec);
        }
    }

    // Assign a Ref to each unique image
    let mut image_refs: HashMap<&str, Ref> = HashMap::new();
    let mut next_id = first_page_id + page_count as i32;
    for id in decoded.keys() {
        image_refs.insert(id, Ref::new(next_id));
        next_id += 1;
    }

    // Write catalog
    pdf.catalog(catalog_id).pages(page_tree_id);

    // Write page tree
    let page_ids: Vec<Ref> = (0..page_count)
        .map(|i| Ref::new(first_page_id + i as i32))
        .collect();
    pdf.pages(page_tree_id)
        .kids(page_ids.iter().copied())
        .count(page_count as i32);

    // Write card pages
    for (i, card) in options.cards.iter().enumerate() {
        let page_id = Ref::new(first_page_id + i as i32);
        let content_id = next_id;
        next_id += 1;
        write_card_page(
            &mut pdf,
            page_id,
            page_tree_id,
            card,
            &image_refs,
            &decoded,
            content_id,
        );
    }
    // Write call list pages
    let call_pages = options.cards.len();
    let cols = 3usize;
    let rows = 8usize;
    let per_page = cols * rows;
    for (pi, chunk) in options.call_list.chunks(per_page).enumerate() {
        let page_id = Ref::new(first_page_id + call_pages as i32 + pi as i32);
        write_call_list_page(&mut pdf, page_id, page_tree_id, chunk, pi, per_page);
    }

    // Write image XObjects
    for (id, dec) in &decoded {
        let img_ref = image_refs[id];
        let mut image = pdf.image_xobject(img_ref, &dec.samples);
        image.width(dec.width as i32);
        image.height(dec.height as i32);
        image.color_space().device_rgb();
        image.bits_per_component(8);
        image.finish();
    }

    pdf.finish()
}

fn write_card_page(
    pdf: &mut Pdf,
    page_id: Ref,
    page_tree_id: Ref,
    card: &BingoCard,
    image_refs: &HashMap<&str, Ref>,
    decoded: &HashMap<&str, DecodedImage>,
    content_id: i32,
) {
    let content_ref = Ref::new(content_id);

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
    page.contents(content_ref);
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

    pdf.stream(content_ref, &content.finish());
}

fn write_call_list_page(
    pdf: &mut Pdf,
    page_id: Ref,
    page_tree_id: Ref,
    items: &[ImageItem],
    page_index: usize,
    per_page: usize,
) {
    let content_ref = Ref::new(page_id.get() + 10000); // offset to avoid collision
    let cols = 3usize;
    let rows = 8usize;
    let col_w = (A4_W - MARGIN * 2.0) / cols as f32;
    let row_h = (A4_H - MARGIN * 2.0) / rows as f32;

    let mut page = pdf.page(page_id);
    page.parent(page_tree_id)
        .media_box(Rect::new(0.0, 0.0, A4_W, A4_H));
    page.contents(content_ref);
    page.finish();

    let mut content = Content::new();

    content
        .set_fill_rgb(0.1, 0.1, 0.1)
        .begin_text()
        .set_font(Name(b"F1"), 11.0);

    for (i, item) in items.iter().enumerate() {
        let col = i % cols;
        let row = i / cols;
        let x = MARGIN + col as f32 * col_w;
        let y = A4_H - MARGIN - (row + 1) as f32 * row_h + row_h / 2.0;

        let number = page_index * per_page + i + 1;
        let label = format!("{}. {}", number, item.label);

        content
            .next_line(x, y)
            .show(pdf_writer::Str(label.as_bytes()));
    }

    content.end_text();
    pdf.stream(content_ref, &content.finish());
}

fn call_list_page_count(call_list: &[ImageItem]) -> usize {
    let per_page = 3 * 8;
    call_list.len().div_ceil(per_page)
}

// Convert an image id to a safe PDF resource name like "Im_cat"
fn image_name(id: &str) -> String {
    let safe = id.replace(['/', ' ', '-'], "_");
    format!("Im_{}", safe)
}
