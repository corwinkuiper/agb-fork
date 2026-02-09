use agb_fixnum::{Rect, Vector2D, vec2};

use crate::display::{
    tile_data::TileData,
    tiled::{DynamicTile16, DynamicTile256, RegularBackground, TileSet, TileSetting},
};

use super::tiled::TileFormat;

pub struct UiRectangle<'background> {
    tiles: &'static TileData,
    background: &'background mut RegularBackground,
    rectangle: Rect<i32>,
}

const TOP_LEFT_CORNER: usize = 0;
const TOP_EDGE: usize = 1;
const TOP_RIGHT_CORNER: usize = 2;
const LEFT_EDGE: usize = 5;
pub const CENTRE: usize = 6;
const RIGHT_EDGE: usize = 7;
const BOTTOM_LEFT_CORNER: usize = 10;
const BOTTOM_EDGE: usize = 11;
const BOTTOM_RIGHT_CORNER: usize = 12;

const TOP_CORNER: usize = 3;
const BOTTOM_CORNER: usize = 8;
const VERTICAL: usize = 9;

const HORIZONTAL: usize = 4;
const LEFT_CORNER: usize = 13;
const RIGHT_CORNER: usize = 14;

impl<'background> UiRectangle<'background> {
    pub fn new(
        rectangle: Rect<i32>,
        tiles: &'static TileData,
        background: &'background mut RegularBackground,
    ) -> Self {
        Self {
            tiles,
            background,
            rectangle,
        }
    }

    fn get_tile(&self, position: Vector2D<i32>) -> usize {
        if self.rectangle.size.x == 1 {
            if position.y == 0 {
                TOP_CORNER
            } else if position.y == self.rectangle.size.y - 1 {
                BOTTOM_CORNER
            } else {
                VERTICAL
            }
        } else if self.rectangle.size.y == 1 {
            if position.x == 0 {
                LEFT_CORNER
            } else if position.x == self.rectangle.size.x - 1 {
                RIGHT_CORNER
            } else {
                HORIZONTAL
            }
        } else {
            if position.x == 0 {
                if position.y == 0 {
                    TOP_LEFT_CORNER
                } else if position.y == self.rectangle.size.y - 1 {
                    BOTTOM_LEFT_CORNER
                } else {
                    LEFT_EDGE
                }
            } else if position.x == self.rectangle.size.x - 1 {
                if position.y == 0 {
                    TOP_RIGHT_CORNER
                } else if position.y == self.rectangle.size.y - 1 {
                    BOTTOM_RIGHT_CORNER
                } else {
                    RIGHT_EDGE
                }
            } else {
                if position.y == 0 {
                    TOP_EDGE
                } else if position.y == self.rectangle.size.y - 1 {
                    BOTTOM_EDGE
                } else {
                    CENTRE
                }
            }
        }
    }

    pub fn draw(&mut self) -> &mut Self {
        if self.rectangle.size.x == 1 {
            self.background.set_tile(
                self.rectangle.position,
                &self.tiles.tiles,
                self.tiles.tile_settings[TOP_CORNER],
            );
            for y in 1..self.rectangle.size.y - 1 {
                self.background.set_tile(
                    self.rectangle.position + vec2(0, y),
                    &self.tiles.tiles,
                    self.tiles.tile_settings[VERTICAL],
                );
            }
            self.background.set_tile(
                self.rectangle.position + self.rectangle.size - vec2(1, 1),
                &self.tiles.tiles,
                self.tiles.tile_settings[BOTTOM_CORNER],
            );
        } else if self.rectangle.size.y == 1 {
            self.background.set_tile(
                self.rectangle.position,
                &self.tiles.tiles,
                self.tiles.tile_settings[LEFT_CORNER],
            );
            for x in 1..self.rectangle.size.x - 1 {
                self.background.set_tile(
                    self.rectangle.position + vec2(x, 0),
                    &self.tiles.tiles,
                    self.tiles.tile_settings[HORIZONTAL],
                );
            }
            self.background.set_tile(
                self.rectangle.position + self.rectangle.size - vec2(1, 1),
                &self.tiles.tiles,
                self.tiles.tile_settings[RIGHT_CORNER],
            );
        } else {
            self.background.set_tile(
                self.rectangle.position,
                &self.tiles.tiles,
                self.tiles.tile_settings[TOP_LEFT_CORNER],
            );
            self.background.set_tile(
                self.rectangle.position + vec2(self.rectangle.size.x - 1, 0),
                &self.tiles.tiles,
                self.tiles.tile_settings[TOP_RIGHT_CORNER],
            );
            self.background.set_tile(
                self.rectangle.position + vec2(0, self.rectangle.size.y - 1),
                &self.tiles.tiles,
                self.tiles.tile_settings[BOTTOM_LEFT_CORNER],
            );
            self.background.set_tile(
                self.rectangle.position + self.rectangle.size - vec2(1, 1),
                &self.tiles.tiles,
                self.tiles.tile_settings[BOTTOM_RIGHT_CORNER],
            );

            for x in 1..self.rectangle.size.x - 1 {
                self.background.set_tile(
                    self.rectangle.position + vec2(x, 0),
                    &self.tiles.tiles,
                    self.tiles.tile_settings[TOP_EDGE],
                );
                self.background.set_tile(
                    self.rectangle.position + vec2(x, self.rectangle.size.y - 1),
                    &self.tiles.tiles,
                    self.tiles.tile_settings[BOTTOM_EDGE],
                );
            }

            for y in 1..self.rectangle.size.y - 1 {
                self.background.set_tile(
                    self.rectangle.position + vec2(0, y),
                    &self.tiles.tiles,
                    self.tiles.tile_settings[LEFT_EDGE],
                );
                self.background.set_tile(
                    self.rectangle.position + vec2(self.rectangle.size.x - 1, y),
                    &self.tiles.tiles,
                    self.tiles.tile_settings[RIGHT_EDGE],
                );
            }

            for y in 1..self.rectangle.size.y - 1 {
                for x in 1..self.rectangle.size.x - 1 {
                    self.background.set_tile(
                        self.rectangle.position + vec2(x, y),
                        &self.tiles.tiles,
                        self.tiles.tile_settings[CENTRE],
                    );
                }
            }
        }

        self
    }

    pub fn draw_image_overlay(&mut self, position: Vector2D<i32>, tile: &[u32]) -> &mut Self {
        let inner_tile_idx = self.get_tile(position);
        let inner_tile = self.tiles.tiles.get_tile_data(inner_tile_idx as u16);
        match self.background.tile_format() {
            TileFormat::FourBpp => {
                let mut dynamic = DynamicTile16::new();
                dynamic.data_mut().copy_from_slice(inner_tile);
                agb::display::utils::blit_16_colour(dynamic.data_mut(), tile);
                self.background.set_tile_dynamic16(
                    position + self.rectangle.position,
                    &dynamic,
                    *self.tiles.tile_settings[inner_tile_idx]
                        .clone()
                        .tile_effect(),
                );
            }
            TileFormat::EightBpp => {
                let mut dynamic = DynamicTile256::new();
                dynamic.data_mut().copy_from_slice(inner_tile);
                agb::display::utils::blit_256_colour(dynamic.data_mut(), tile);
                self.background.set_tile_dynamic256(
                    position + self.rectangle.position,
                    &dynamic,
                    *self.tiles.tile_settings[inner_tile_idx]
                        .clone()
                        .tile_effect(),
                );
            }
        }

        self
    }

    pub fn draw_tileset_overlay(
        &mut self,
        position: Vector2D<i32>,
        tiledata: &TileData,
    ) -> &mut Self {
        for y in 0..tiledata.height as i32 {
            for x in 0..tiledata.width as i32 {
                let setting = tiledata.tile_settings[x as usize + y as usize * tiledata.width];
                let tile = tiledata.tiles.get_tile_data(setting.tile_id());
                self.draw_image_overlay(position + vec2(x, y), tile);
            }
        }

        self
    }

    pub fn draw_image(
        &mut self,
        pos: Vector2D<i32>,
        tileset: &TileSet,
        tile_setting: TileSetting,
    ) -> &mut Self {
        self.background
            .set_tile(pos + self.rectangle.top_left(), tileset, tile_setting);

        self
    }

    pub fn draw_tiles(&mut self, position: Vector2D<i32>, tiledata: &TileData) -> &mut Self {
        for y in 0..tiledata.height {
            for x in 0..tiledata.width {
                let setting = tiledata.tile_settings[x + y * tiledata.width];
                self.draw_image(
                    vec2(x as i32, y as i32) + position,
                    &tiledata.tiles,
                    setting,
                );
            }
        }

        self
    }
}

#[macro_export]
macro_rules! ui_blit {
    ($ui: expr, $sample: expr) => {{
        use $crate::display::tile_data::TileData;
        use $crate::display::tiled::{TileEffect, TileFormat, TileSet, TileSetting};
        use $crate::display::ui::CENTRE;
        use $crate::display::utils::*;

        const SAMPLE: &TileData = $sample;
        const UI: &TileData = $ui;

        const NUM_TILES: usize = SAMPLE.tiles.tiles().len() / 8;

        const CENTRE_TILE: &[u32] = UI.tiles.get_tile_data(CENTRE as u16);

        static TILES: &[u32] = &const {
            let mut output_tiles = [0u32; NUM_TILES * 8];

            let mut tile_idx = 0;
            while tile_idx < NUM_TILES {
                let mut copy_idx = 0;
                while copy_idx < 8 {
                    output_tiles[tile_idx * 8 + copy_idx] = CENTRE_TILE[copy_idx];
                    copy_idx += 1;
                }
                tile_idx += 1;
            }

            blit_16_colour(&mut output_tiles, SAMPLE.tiles.tiles());

            output_tiles
        };

        const {
            TileData::new(
                unsafe { TileSet::new(cast_u32_to_bytes(TILES), TileFormat::FourBpp) },
                SAMPLE.tile_settings,
                SAMPLE.width,
                SAMPLE.height,
            )
        }
    }};
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::{
        display::{
            Priority,
            tiled::{RegularBackground, RegularBackgroundSize, TileFormat, VRAM_MANAGER},
        },
        include_background_gfx,
        test_runner::assert_image_output,
    };

    include_background_gfx!(mod ui, UI => "examples/gfx/ui.aseprite", CRAB_UNBLIT => "examples/gfx/crab.aseprite");

    static CRAB_BLIT: TileData = ui_blit!(&ui::UI, &ui::CRAB_UNBLIT);

    #[test_case]
    fn check_basic_ui(gba: &mut crate::Gba) {
        VRAM_MANAGER.set_background_palettes(ui::PALETTES);
        let mut graphics = gba.graphics.get();
        let mut frame = graphics.frame();
        let mut bg = RegularBackground::new(
            Priority::P0,
            RegularBackgroundSize::Background32x32,
            TileFormat::FourBpp,
        );
        UiRectangle::new(Rect::new(vec2(2, 2), vec2(5, 5)), &ui::UI, &mut bg).draw();
        UiRectangle::new(Rect::new(vec2(10, 2), vec2(2, 2)), &ui::UI, &mut bg).draw();
        UiRectangle::new(Rect::new(vec2(15, 2), vec2(5, 1)), &ui::UI, &mut bg).draw();
        UiRectangle::new(Rect::new(vec2(10, 5), vec2(1, 5)), &ui::UI, &mut bg).draw();

        bg.show(&mut frame);
        frame.commit();
        assert_image_output("gfx/test_output/ui/basic.png");
    }

    #[test_case]
    fn check_crab_overlay(gba: &mut crate::Gba) {
        VRAM_MANAGER.set_background_palettes(ui::PALETTES);
        let mut graphics = gba.graphics.get();
        let mut frame = graphics.frame();
        let mut bg = RegularBackground::new(
            Priority::P0,
            RegularBackgroundSize::Background32x32,
            TileFormat::FourBpp,
        );

        UiRectangle::new(
            Rect::new(
                vec2(2, 2),
                vec2(CRAB_BLIT.width as i32, CRAB_BLIT.height as i32),
            ),
            &ui::UI,
            &mut bg,
        )
        .draw()
        .draw_tiles(vec2(0, 0), &CRAB_BLIT);

        bg.show(&mut frame);
        frame.commit();
        assert_image_output("gfx/test_output/ui/crab_overlay.png");
    }
}
