use core::str;

use crate::display::font::{AlignmentKind, Font, FontLetter, special::AGB_PRIVATE_USE_RANGE};

#[macro_export]
macro_rules! bake {
    ($font: expr, $text: expr) => {
        $crate::bake!(
            $font,
            $text,
            $crate::display::font::bake::BakeSettings::new()
        )
    };
    ($font: expr, $text: expr, $settings: expr) => {{
        use $crate::display::font::Font;
        use $crate::display::font::bake::*;
        use $crate::display::tile_data::TileData;
        use $crate::display::tiled::{TileEffect, TileFormat, TileSet, TileSetting};
        use $crate::display::utils::*;

        const SETTINGS: BakeSettings = $settings;

        const THIS_FONT: &Font = &$font;
        const THIS_TEXT: &str = $text;
        const TILE_SIZE: (usize, usize, usize) = const {
            let length = calculate_length(THIS_FONT, THIS_TEXT)
                + if SETTINGS.backdrop().is_some() { 1 } else { 0 };
            let height = calculate_height(THIS_FONT, THIS_TEXT);

            let tile_length = length.div_ceil(8);
            let tile_height = height.div_ceil(8);

            (tile_length as usize, tile_height as usize, length as usize)
        };
        const NUMBER_OF_TILES: usize = const { TILE_SIZE.0 * TILE_SIZE.1 };
        const NUMBER_OF_U32S: usize = const { TILE_SIZE.0 * TILE_SIZE.1 * 8 };

        static TILES: &[u32] = &const {
            let mut tiles = [0; NUMBER_OF_U32S];

            let mut tiles_collection =
                TileCollection::new(&mut tiles, TILE_SIZE.0 as usize, TILE_SIZE.2);

            bake_inner(THIS_FONT, THIS_TEXT, &mut tiles_collection, &SETTINGS);

            tiles
        };

        const TILE_EFFECT: TileEffect = const { TileEffect::new(false, false, 0) };

        static TILE_SETTINGS: &[TileSetting] = &const {
            let mut tiles = [const { TileSetting::new(0, TILE_EFFECT) }; NUMBER_OF_TILES];

            let mut idx = 0;
            while idx < NUMBER_OF_TILES {
                tiles[idx] = TileSetting::new(idx as u16, TILE_EFFECT);
                idx += 1;
            }

            tiles
        };

        &const {
            TileData::new(
                unsafe { TileSet::new(cast_u32_to_bytes(TILES), TileFormat::FourBpp) },
                TILE_SETTINGS,
                TILE_SIZE.0,
                TILE_SIZE.1,
            )
        }
    }};
}

pub struct BakeSettings {
    colours: [u8; 16],
    backdrop: Option<u8>,
    background: u8,
    alignment: AlignmentKind,
}

impl BakeSettings {
    pub const fn new() -> BakeSettings {
        let colours = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

        BakeSettings {
            colours,
            backdrop: None,
            background: 0,
            alignment: AlignmentKind::Left,
        }
    }

    pub const fn with_colours(mut self, colours: &[u8]) -> BakeSettings {
        let mut idx = 0;
        while idx < colours.len() {
            self.colours[idx + 1] = colours[idx];
            idx += 1;
        }

        self
    }

    pub const fn with_background(mut self, background_idx: u8) -> BakeSettings {
        self.background = background_idx;
        self
    }

    pub const fn with_backdrop(mut self, backdrop_idx: u8) -> BakeSettings {
        self.backdrop = Some(backdrop_idx);

        self
    }

    pub const fn with_alignment(mut self, alignment: AlignmentKind) -> BakeSettings {
        self.alignment = alignment;

        self
    }

    pub const fn backdrop(&self) -> Option<u8> {
        self.backdrop
    }

    pub const fn alignment(&self) -> AlignmentKind {
        self.alignment
    }

    pub const fn palette_index(&self) -> u8 {
        self.colours[1]
    }
}

struct Chars<'a> {
    text: &'a str,
}

const fn pixel_width(letter: &FontLetter) -> i32 {
    let mut max_x = 0;

    let mut y = 0;
    while y < letter.height as i32 {
        let mut x = max_x;
        while x < letter.width as i32 {
            if letter.bit_absolute(x as usize, y as usize) {
                max_x = x + 1;
                break;
            }
            x += 1;
        }
        y += 1;
    }

    max_x
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

        width -= l.advance_width as i32;
        width += l.xmin as i32 + pixel_width(l);

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
    width_pixels: usize,
}

impl<'a> TileCollection<'a> {
    pub const fn new(tiles: &'a mut [u32], width_tiles: usize, width_pixels: usize) -> Self {
        Self {
            tiles,
            width_tiles,
            width_pixels,
        }
    }

    const fn fill(&mut self, colour: u8) {
        let colour = colour as u32;
        let colour = colour | colour << 4;
        let colour = colour | colour << 8;
        let colour = colour | colour << 16;

        let mut idx = 0;
        while idx < self.tiles.len() {
            self.tiles[idx] = colour;
            idx += 1;
        }
    }

    const fn set_pixel(&mut self, x: i32, y: i32, colour: u8) -> Option<()> {
        let colour = colour as u32;

        if x < 0
            || x >= (self.width_tiles as i32 * 8)
            || y < 0
            || y >= ((self.tiles.len() / self.width_tiles) as i32)
        {
            return None;
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

        Some(())
    }
}

pub const fn bake_inner(
    font: &Font,
    text: &str,
    tiles: &mut TileCollection,
    settings: &BakeSettings,
) {
    let mut previous_character = None;
    let mut chars = Chars::new(text);

    tiles.fill(settings.background);

    let mut cursor = match settings.alignment {
        AlignmentKind::Left | AlignmentKind::None | AlignmentKind::Justify => 0,
        AlignmentKind::Right => tiles.width_pixels % 8,
        AlignmentKind::Centre => (tiles.width_pixels % 8).div_ceil(2),
    } as i32;
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
                    let xx = x_start + x;
                    let yy = y_start + y;
                    tiles
                        .set_pixel(xx, yy, settings.colours[colour])
                        .expect("Pixel should be in range");
                    if let Some(backdrop) = settings.backdrop {
                        tiles.set_pixel(xx + 1, yy + 1, backdrop);
                    }
                }

                x += 1;
            }
            y += 1;
        }

        cursor += l.advance_width as i32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static FONT: Font = include_font!("examples/font/ark-pixel-10px-proportional-ja.ttf", 10);

    #[test_case]
    fn check_chars(_: &mut crate::Gba) {
        let mut chars = Chars::new("Hello, 世界!");
        assert_eq!(chars.next(), Some('H'));
        assert_eq!(chars.next(), Some('e'));
        assert_eq!(chars.next(), Some('l'));
        assert_eq!(chars.next(), Some('l'));
        assert_eq!(chars.next(), Some('o'));
        assert_eq!(chars.next(), Some(','));
        assert_eq!(chars.next(), Some(' '));
        assert_eq!(chars.next(), Some('世'));
        assert_eq!(chars.next(), Some('界'));
        assert_eq!(chars.next(), Some('!'));
        assert_eq!(chars.next(), None);
    }

    #[test_case]
    fn check_chars_next_back(_: &mut crate::Gba) {
        let mut chars = Chars::new("Hello, 世界!");
        assert_eq!(chars.next_back(), Some('!'));
        assert_eq!(chars.next_back(), Some('界'));
        assert_eq!(chars.next_back(), Some('世'));
        assert_eq!(chars.next_back(), Some(' '));
        assert_eq!(chars.next_back(), Some(','));
        assert_eq!(chars.next_back(), Some('o'));
        assert_eq!(chars.next_back(), Some('l'));
        assert_eq!(chars.next_back(), Some('l'));
        assert_eq!(chars.next_back(), Some('e'));
        assert_eq!(chars.next_back(), Some('H'));
        assert_eq!(chars.next_back(), None);
    }

    #[test_case]
    fn check_pixel_width(_: &mut crate::Gba) {
        assert_eq!(pixel_width(FONT.letter('t')), 4);
    }
}
