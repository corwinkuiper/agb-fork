use core::fmt::Write;

use agb::{
    display::{bitmap3::Bitmap3, Font},
    fixnum::Vector2D,
};
use alloc::collections::VecDeque;

pub struct BitmapTextRender<'bitmap, 'gba> {
    head_position: Vector2D<i32>,
    font: &'static Font,
    start_x: i32,
    bitmap: &'bitmap mut Bitmap3<'gba>,
    colour: u16,
}

pub struct CenteredTextRender<'bitmap, 'gba> {
    inner: BitmapTextRender<'bitmap, 'gba>,
    buffer: VecDeque<char>,
}

impl<'bitmap, 'gba> BitmapTextRender<'bitmap, 'gba> {
    pub fn new(
        font: &'static Font,
        bitmap: &'bitmap mut Bitmap3<'gba>,
        position: Vector2D<i32>,
        start_colour: u16,
    ) -> Self {
        Self {
            font,
            head_position: position,
            start_x: position.x,
            bitmap,
            colour: start_colour,
        }
    }

    fn render_letter(&mut self, letter: char) {
        let letter = self.font.letter(letter);

        self.head_position.x += letter.xmin() as i32;

        let y_position_start = self.head_position.y + self.font.ascent()
            - letter.height() as i32
            - letter.ymin() as i32;

        for y in 0..letter.height() as usize {
            for x in 0..letter.width() as usize {
                let rendered = letter.bit_absolute(x, y);
                if rendered {
                    self.bitmap.draw_point(
                        x as i32 + self.head_position.x,
                        y as i32 + y_position_start,
                        self.colour,
                    )
                }
            }
        }

        self.head_position.x += letter.advance_width() as i32;
    }

    fn render_char(&mut self, c: char) {
        match c {
            '\n' => {
                self.head_position.x = self.start_x;
                self.head_position.y += self.font.line_height();
            }
            ' ' => {
                self.head_position.x += self.font.letter(' ').advance_width() as i32;
            }
            letter => self.render_letter(letter),
        }
    }
}

impl<'bitmap, 'gba> CenteredTextRender<'bitmap, 'gba> {
    pub fn new(font: &'static Font, bitmap: &'bitmap mut Bitmap3<'gba>, start_colour: u16) -> Self {
        Self {
            inner: BitmapTextRender::new(font, bitmap, (0, 0).into(), start_colour),
            buffer: VecDeque::new(),
        }
    }
    pub fn render_line_centered(&mut self, position: Vector2D<i32>) {
        let mut line_width = 0;
        let mut last_idx = 0;

        for (idx, c) in self.buffer.iter().copied().enumerate() {
            last_idx = idx;

            match c {
                '\n' => {
                    break;
                }
                letter => {
                    let letter = self.inner.font.letter(letter);
                    line_width += letter.advance_width() as i32 - letter.xmin() as i32;
                }
            }
        }

        self.inner.head_position = position - (line_width / 2, 0).into();

        for c in self.buffer.drain(0..last_idx) {
            self.inner.render_char(c);
        }
    }
}

impl<'bitmap, 'gba> Write for BitmapTextRender<'bitmap, 'gba> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.render_char(c)
        }

        Ok(())
    }
}

impl<'bitmap, 'gba> Write for CenteredTextRender<'bitmap, 'gba> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.buffer.push_back(c)
        }

        Ok(())
    }
}
