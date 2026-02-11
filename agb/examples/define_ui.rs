//! Example showing define_ui! macro for retained-mode UI with dirty tracking.
//!
//! Displays a score counter, an HP bar (current/max), and a cycling description.
//! Only dirty elements are re-rendered each frame.

#![no_std]
#![no_main]

extern crate alloc;

use agb::{
    define_ui,
    display::{
        Palette16, Priority, Rgb15,
        font::{Font, bake::BakeSettings},
        tiled::{RegularBackground, RegularBackgroundSize, TileFormat, VRAM_MANAGER},
    },
    include_background_gfx, include_font,
};

include_background_gfx!(mod ui_gfx, UI => "examples/gfx/ui.aseprite");
static FONT: Font = include_font!("fnt/ark-pixel-10px-proportional-latin.ttf", 10);

const PALETTE: Palette16 = ui_gfx::PALETTES[0].extend(&[Rgb15::BLACK, Rgb15::WHITE]);

define_ui! {
    ScoreHud,
    ui_tiles: &ui_gfx::UI,
    font: &FONT,
    bake_settings: BakeSettings::new()
        .with_colours(&PALETTE.find([Rgb15::WHITE]))
        .with_backdrop(PALETTE.find([Rgb15::BLACK])[0]),
    layout: {
        rect(1, 1, 16, 6) {
            row(1, 1) {
                static_text("Score: "),
                number score(6, right),
            }
            row(1, 3) {
                static_text("HP: "),
                number current_hp(3, right),
                static_text("/"),
                number max_hp(3, left),
            }
        }
        rect(1, 8, 20, 4) {
            row(1, 1) {
                dynamic_text description(18, 2),
            }
        }
    }
}

static DESCRIPTIONS: &[&str] = &["Hello world!", "agb UI system", "Dirty tracking"];

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    VRAM_MANAGER.set_background_palette(0, &PALETTE);

    let mut gfx = gba.graphics.get();

    let mut bg = RegularBackground::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    let mut hud = ScoreHud::new();
    hud.set_score(0);
    hud.set_current_hp(123);
    hud.set_max_hp(456);
    hud.set_description(DESCRIPTIONS[0]);

    let mut frame_count: i32 = 0;

    loop {
        frame_count = frame_count.wrapping_add(1);
        hud.set_score(frame_count);

        // HP ticks down every 60 frames, wrapping back to max
        let hp = 456 - (frame_count / 60) % 457;
        hud.set_current_hp(hp);

        let desc_idx = (frame_count / 120) as usize % DESCRIPTIONS.len();
        hud.set_description(DESCRIPTIONS[desc_idx]);

        hud.show(&mut bg);

        let mut frame = gfx.frame();
        bg.show(&mut frame);
        frame.commit();
    }
}
