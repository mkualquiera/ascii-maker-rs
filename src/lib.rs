mod stamps;

use crate::stamps::{CELL_H, CELL_W};
use image::{GenericImageView, ImageReader};
use std::io::Cursor;
use wasm_bindgen::prelude::*;

// Set up panic hook for better error messages
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

// Add logging macro
macro_rules! console_log {
    ($($t:tt)*) => (web_sys::console::log_1(&format_args!($($t)*).to_string().into()))
}

#[wasm_bindgen]
pub fn process_image(
    image: Vec<u8>,
    cols: u32,
    invert: bool,
    callback: &js_sys::Function,
) -> String {
    // Validate inputs first
    if image.is_empty() {
        return "Error: Empty image data".to_string();
    }

    if cols == 0 {
        return "Error: cols must be greater than 0".to_string();
    }

    console_log!(
        "Starting image processing with {} bytes, {} cols",
        image.len(),
        cols
    );

    let img_data = Cursor::new(&image);
    let img = match ImageReader::new(img_data).with_guessed_format() {
        Ok(reader) => {
            console_log!("Successfully guessed image format");
            reader
        }
        Err(e) => {
            let error_msg = format!("Error guessing format: {}", e);
            console_log!("{}", error_msg);
            return error_msg;
        }
    };

    let img = match img.decode() {
        Ok(decoded) => {
            console_log!(
                "Successfully decoded image: {}x{}",
                decoded.width(),
                decoded.height()
            );
            decoded
        }
        Err(e) => {
            let error_msg = format!("Error decoding image: {}", e);
            console_log!("{}", error_msg);
            return error_msg;
        }
    };

    // Match the C code logic exactly
    let final_cwidth = cols;
    let final_iwidth = final_cwidth * CELL_W as u32;
    let scaling_ratio = final_iwidth as f64 / img.width() as f64;
    let final_iheight = (img.height() as f64 * scaling_ratio) as u32;
    let final_cheight = final_iheight / CELL_H as u32;

    console_log!("Original image: {}x{}", img.width(), img.height());
    console_log!(
        "Target cell dimensions: {}x{} cells",
        final_cwidth,
        final_cheight
    );
    console_log!(
        "Target pixel dimensions: {}x{} pixels",
        final_iwidth,
        final_iheight
    );
    console_log!("Scaling ratio: {}", scaling_ratio);

    // Resize the image to match the C code behavior using Lanczos3 filter
    let resized_img = img.resize(
        final_iwidth + 1,
        final_iheight + 1,
        image::imageops::FilterType::Lanczos3,
    );
    console_log!(
        "Resized image to: {}x{}",
        resized_img.width(),
        resized_img.height()
    );

    let mut resized_img = resized_img;
    if invert {
        resized_img.invert();
    }

    // Update rows to match what we actually got
    let rows = final_cheight;

    // Process each cell
    for row in 0..rows {
        for col in 0..cols {
            // Calculate pixel boundaries for this cell
            let start_x = col * CELL_W as u32;
            let start_y = row * CELL_H as u32;
            let end_x = start_x + CELL_W as u32;
            let end_y = start_y + CELL_H as u32;

            // Bounds checking - this should never happen with proper resizing
            if end_x > resized_img.width() || end_y > resized_img.height() {
                panic!(
                    "Cell at ({}, {}) extends beyond image bounds. Cell ends at ({}, {}), image is {}x{}",
                    col,
                    row,
                    end_x,
                    end_y,
                    resized_img.width(),
                    resized_img.height()
                );
            }

            let mut best_candidate = None;
            let mut best_score = f32::MAX;

            for candidate in 0..95 {
                let mut score: f32 = 0.0;

                for x in 0..CELL_W as u32 {
                    for y in 0..CELL_H as u32 {
                        let pixel_x = start_x + x;
                        let pixel_y = start_y + y;

                        // Bounds check before pixel access
                        if pixel_x >= resized_img.width() || pixel_y >= resized_img.height() {
                            panic!(
                                "Pixel access out of bounds: ({}, {}) vs image size ({}, {})",
                                pixel_x,
                                pixel_y,
                                resized_img.width(),
                                resized_img.height()
                            );
                        }

                        let pixel = resized_img.get_pixel(pixel_x, pixel_y);
                        let r = pixel[0] as f32 / 255.0;
                        let g = pixel[1] as f32 / 255.0;
                        let b = pixel[2] as f32 / 255.0;

                        // Bounds check for stamps access
                        if y as u8 >= CELL_H as u8 || x as u8 >= CELL_W as u8 {
                            panic!(
                                "Stamp access out of bounds: ({}, {}) vs cell size ({}, {})",
                                x, y, CELL_W, CELL_H
                            );
                        }

                        match stamps::access_data(candidate, 0, 7, y as u8, x as u8) {
                            Some(color) => {
                                let dr = r - (*color.r as f32 / 255.0);
                                let dg = g - (*color.g as f32 / 255.0);
                                let db = b - (*color.b as f32 / 255.0);
                                score += dr * dr + dg * dg + db * db;
                            }
                            None => {
                                panic!(
                                    "stamps::access_data failed for candidate {}, pos ({}, {})",
                                    candidate, y, x
                                );
                            }
                        }
                    }
                }

                if score < best_score {
                    best_score = score;
                    best_candidate = Some(candidate);
                }
            }

            match best_candidate {
                Some(best_candidate) => {
                    let actual_char_id = best_candidate + 32;
                    if let Some(ch) = char::from_u32(actual_char_id as u32) {
                        let as_str = ch.to_string();
                        if let Err(e) =
                            callback.call1(&JsValue::UNDEFINED, &JsValue::from_str(&as_str))
                        {
                            panic!("Failed to call callback during character output: {:?}", e);
                        }
                    } else {
                        panic!("Invalid character ID: {}", actual_char_id);
                    }
                }
                None => {
                    panic!(
                        "No candidate found for cell ({}, {}) - this should never happen",
                        col, row
                    );
                }
            }
        }

        // Add newline
        if let Err(e) = callback.call1(&JsValue::UNDEFINED, &JsValue::from_str("\n")) {
            panic!("Failed to call callback for newline: {:?}", e);
        }

        // Optional: Progress reporting
        if row % 10 == 0 {
            console_log!("Processed {} of {} rows", row + 1, rows);
        }
    }

    console_log!("Image processing completed successfully");
    "Success".to_string()
}
