use std::io::{self, Cursor};

use lazy_static::lazy_static;

const STAMP_DATA_BROTLI: &[u8] = include_bytes!("stamps.bin.br");

fn decompress_brotli(compressed_data: &[u8]) -> Result<Vec<u8>, io::Error> {
    let mut input = Cursor::new(compressed_data);
    let mut output = Vec::new();

    brotli::BrotliDecompress(&mut input, &mut output)?;
    Ok(output)
}

lazy_static! {
    static ref STAMP_DATA: Vec<u8> =
        decompress_brotli(STAMP_DATA_BROTLI).expect("Failed to decompress stamp data");
}

pub struct Color<'a> {
    pub r: &'a u8,
    pub g: &'a u8,
    pub b: &'a u8,
}

pub const CELL_W: usize = 10;
pub const CELL_H: usize = 20;

pub fn access_data(char_id: u8, bg_id: u8, fg_id: u8, y: u8, x: u8) -> Option<Color<'static>> {
    const COLUMN_SELECTOR: usize = 3;
    const ROW_SELECTOR: usize = COLUMN_SELECTOR * CELL_W;
    const FG_SELECTOR: usize = ROW_SELECTOR * CELL_H;
    const BG_SELECTOR: usize = FG_SELECTOR * 16;
    const CHAR_SELECTOR: usize = BG_SELECTOR * 16;

    // Bounds checking
    if char_id >= 95 || bg_id >= 16 || fg_id >= 16 || y >= 20 || x >= 10 {
        return None;
    }

    let index = char_id as usize * CHAR_SELECTOR
        + bg_id as usize * BG_SELECTOR
        + fg_id as usize * FG_SELECTOR
        + y as usize * ROW_SELECTOR
        + x as usize * COLUMN_SELECTOR;

    // Additional bounds check for the array access
    if index + 2 >= STAMP_DATA.len() {
        return None;
    }

    Some(Color {
        r: &STAMP_DATA[index],
        g: &STAMP_DATA[index + 1],
        b: &STAMP_DATA[index + 2],
    })
}
