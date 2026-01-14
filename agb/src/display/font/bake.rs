use core::str;

use agb_fixnum::{Vector2D, vec2};

use crate::display::{
    font::{Font, special::AGB_PRIVATE_USE_RANGE},
    tiled::{RegularBackground, TileEffect, TileSet, TileSetting},
};

#[macro_export]
macro_rules! bake {
    ($font: expr, $text: expr) => {{
        use $crate::display::font::Font;
        use $crate::display::font::bake::*;
        const THIS_FONT: &Font = &$font;
        const THIS_TEXT: &str = $text;
        const TILE_SIZE: (usize, usize) = const {
            let length = calculate_length(THIS_FONT, THIS_TEXT);
            let height = calculate_height(THIS_FONT, THIS_TEXT);

            let tile_length = length.div_ceil(8);
            let tile_height = height.div_ceil(8);

            (tile_length as usize, tile_height as usize)
        };
        const NUMBER_OF_U32S: usize = const { TILE_SIZE.0 * TILE_SIZE.1 * 8 };

        const TILES: &[u32] = &const {
            let mut tiles = [0; NUMBER_OF_U32S];

            let mut tiles_collection = TileCollection::new(&mut tiles, TILE_SIZE.0 as usize);

            bake_inner(THIS_FONT, THIS_TEXT, &mut tiles_collection);

            tiles
        };

        const { BakedText::new(TILES, TILE_SIZE.0, TILE_SIZE.1) }
    }};
}

struct Chars<'a> {
    text: &'a str,
}

const fn str_to_char(s: &str) -> char {
    const CONTINUATION_MASK: u32 = 0b00111111;

    let code_point = match *s.as_bytes() {
        [a] => a as u32,
        [a, b] => ((a as u32 & 0b00011111) << 6) | (b as u32 & CONTINUATION_MASK),
        [a, b, c] => {
            ((a as u32 & 0b00001111) << 12)
                | ((b as u32 & CONTINUATION_MASK) << 6)
                | (c as u32 & CONTINUATION_MASK)
        }
        [a, b, c, d] => {
            ((a as u32 & 0b00000111) << 18)
                | ((b as u32 & CONTINUATION_MASK) << 12)
                | ((c as u32 & CONTINUATION_MASK) << 6)
                | (d as u32 & CONTINUATION_MASK)
        }
        _ => panic!("Str is not a char"),
    };

    char::from_u32(code_point).expect("conversion should be correct")
}

impl<'a> Chars<'a> {
    const fn new(text: &'a str) -> Self {
        Self { text }
    }

    const fn next(&mut self) -> Option<char> {
        if self.text.is_empty() {
            return None;
        }

        let mut idx = 1;
        while !self.text.is_char_boundary(idx) {
            idx += 1;
        }

        let (c, rest) = &self.text.split_at(idx);

        self.text = rest;

        Some(str_to_char(c))
    }

    const fn next_back(&mut self) -> Option<char> {
        if self.text.is_empty() {
            return None;
        }

        let mut idx = self.text.len() - 1;
        while !self.text.is_char_boundary(idx) {
            idx -= 1;
        }

        let (rest, c) = &self.text.split_at(idx);

        self.text = rest;

        Some(str_to_char(c))
    }
}

pub const fn calculate_length(font: &Font, text: &str) -> u32 {
    let mut previous_character = None;
    let mut chars = Chars::new(text);
    let mut width = 0;
    while let Some(c) = chars.next() {
        // rust analyzer gets this type wrong, so for now do this up here so I can have correct hints elsewhere
        let c: char = c;
        let l = font.letter_const(c);
        let kern = if let Some(previous) = previous_character {
            l.kerning_amount_const(previous)
        } else {
            0
        };

        if c == '\n' {
            panic!("Text contains newline");
        }

        if (c as u32) >= AGB_PRIVATE_USE_RANGE.start && (c as u32) < AGB_PRIVATE_USE_RANGE.end {
            continue;
        }

        previous_character = Some(c);

        width += l.advance_width as i32 + kern;
    }

    let mut chars = Chars::new(text);

    while let Some(c) = chars.next_back() {
        let c: char = c;
        if (c as u32) >= AGB_PRIVATE_USE_RANGE.start && (c as u32) < AGB_PRIVATE_USE_RANGE.end {
            continue;
        }

        let l = font.letter_const(c);

        let difference = l.xmin as i32 + l.width as i32 - l.advance_width as i32;
        if difference > 0 {
            width += difference;
        }

        break;
    }

    width as u32
}

pub const fn calculate_height(font: &Font, text: &str) -> u32 {
    let mut chars = Chars::new(text);
    let mut height = i32::MIN;
    while let Some(c) = chars.next() {
        let c: char = c;
        let l = font.letter_const(c);
        let this_height = font.ascent() - l.ymin as i32;

        if this_height > height {
            height = this_height;
        }
    }

    height as u32
}

pub struct TileCollection<'a> {
    tiles: &'a mut [u32],
    width_tiles: usize,
}

impl<'a> TileCollection<'a> {
    pub const fn new(tiles: &'a mut [u32], width_tiles: usize) -> Self {
        Self { tiles, width_tiles }
    }

    const fn set_pixel(&mut self, x: i32, y: i32, colour: u32) {
        if x < 0
            || x > (self.width_tiles as i32 * 8)
            || y < 0
            || y > ((self.tiles.len() / self.width_tiles) as i32)
        {
            panic!("Pixel out of bounds");
        }

        let x = x as usize;
        let y = y as usize;

        let x_pixel = x % 8;
        let y_pixel = y % 8;
        let x_tile = x / 8;
        let y_tile = y / 8;

        let mask = 0xF << (x_pixel * 4);
        let idx = (x_tile + y_tile * self.width_tiles) * 8 + y_pixel;
        self.tiles[idx] = (self.tiles[idx] & !mask) | (colour << (x_pixel * 4));
    }
}

pub const fn bake_inner(font: &Font, text: &str, tiles: &mut TileCollection) {
    let mut previous_character = None;
    let mut chars = Chars::new(text);

    let mut cursor = 0;
    let mut colour = 1;

    while let Some(c) = chars.next() {
        // rust analyzer gets this type wrong, so for now do this up here so I can have correct hints elsewhere
        let c: char = c;
        let l = font.letter_const(c);
        let kern = if let Some(previous) = previous_character {
            l.kerning_amount_const(previous)
        } else {
            0
        };

        if c == '\n' {
            panic!("Text contains newline");
        }

        if (c as u32) >= AGB_PRIVATE_USE_RANGE.start && (c as u32) < AGB_PRIVATE_USE_RANGE.end {
            continue;
        }

        previous_character = Some(c);
        cursor += kern;

        let mut y = 0;

        let y_start = font.ascent() as i32 - l.height as i32 - l.ymin as i32;
        let x_start = cursor + l.xmin as i32;
        while y < l.height as i32 {
            let mut x = 0;
            while x < l.width as i32 {
                if l.bit_absolute(x as usize, y as usize) {
                    tiles.set_pixel(x_start + x, y_start + y, colour);
                }

                x += 1;
            }
            y += 1;
        }

        cursor += l.advance_width as i32;
    }
}

pub struct BakedText {
    width: usize,
    height: usize,
    tiles: &'static [u32],
}

const fn cast_u32_to_bytes(a: &[u32]) -> &[u8] {
    unsafe { core::slice::from_raw_parts(a.as_ptr().cast(), a.len() * 4) }
}

impl BakedText {
    pub const fn new(tiles: &'static [u32], width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            tiles,
        }
    }

    pub const fn data(&self) -> &'static [u32] {
        self.tiles
    }

    pub const fn data_bytes(&self) -> &'static [u8] {
        cast_u32_to_bytes(self.tiles)
    }

    pub const fn tile_set(&self) -> TileSet {
        unsafe {
            TileSet::new(
                self.data_bytes(),
                crate::display::tiled::TileFormat::FourBpp,
            )
        }
    }

    pub const fn width(&self) -> usize {
        self.width
    }

    pub const fn height(&self) -> usize {
        self.height
    }

    pub fn set_bg_tiles(&self, bg: &mut RegularBackground, position: impl Into<Vector2D<i32>>) {
        let position = position.into();
        for y in 0..self.height {
            for x in 0..self.width {
                bg.set_tile(
                    position + vec2(x as i32, y as i32),
                    &self.tile_set(),
                    TileSetting::new((x + y * self.width) as u16, TileEffect::default()),
                );
            }
        }
    }
}
