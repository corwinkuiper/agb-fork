//! An example of using baked text

#![no_std]
#![no_main]

use agb::{
    bake,
    display::{
        Rgb15,
        font::{Font, bake::BakedText},
        tiled::{RegularBackground, VRAM_MANAGER},
    },
    fixnum::vec2,
    include_font,
};

static FONT: Font = include_font!("examples/font/ark-pixel-10px-proportional-ja.ttf", 10);
static HELLO: BakedText = bake!(FONT, "This is my example text");

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();

    let mut bg = RegularBackground::new(
        agb::display::Priority::P0,
        agb::display::tiled::RegularBackgroundSize::Background32x32,
        agb::display::tiled::TileFormat::FourBpp,
    );

    VRAM_MANAGER.set_background_palette_colour(0, 0, Rgb15::BLACK);
    VRAM_MANAGER.set_background_palette_colour(0, 1, Rgb15::WHITE);

    HELLO.set_bg_tiles(&mut bg, vec2(2, 4));

    loop {
        let mut frame = gfx.frame();

        bg.show(&mut frame);
        frame.commit();
    }
}
