//! Inventory UI example using define_ui! with a pageable item list,
//! a description panel, and a sprite-based selection cursor.

#![no_std]
#![no_main]

extern crate alloc;

use agb::{
    define_ui,
    display::{
        Palette16, Priority, Rgb15,
        font::{Font, bake::BakeSettings},
        object::Object,
        tiled::{RegularBackground, RegularBackgroundSize, TileFormat, VRAM_MANAGER},
    },
    fixnum::vec2,
    include_aseprite, include_background_gfx, include_font,
    input::{Button, ButtonController},
};

include_background_gfx!(mod ui_gfx, UI => "examples/gfx/ui.aseprite");
include_aseprite!(mod cursor_gfx, "examples/gfx/cursor.aseprite");

static FONT: Font = include_font!("examples/font/Dungeon Puzzler Font.ttf", 8);

const PALETTE: Palette16 = ui_gfx::PALETTES[0].extend(&[Rgb15::BLACK, Rgb15::WHITE]);

const ITEMS_PER_PAGE: usize = 5;

struct Item {
    name: &'static str,
    description: &'static str,
}

static ITEMS: &[Item] = &[
    Item {
        name: "Iron Sword",
        description: "A sturdy blade forged from iron.",
    },
    Item {
        name: "Wooden Shield",
        description: "Blocks weak attacks.",
    },
    Item {
        name: "Health Potion",
        description: "Restores 50 HP when used.",
    },
    Item {
        name: "Fire Scroll",
        description: "Casts a fireball that hits all enemies.",
    },
    Item {
        name: "Leather Armor",
        description: "Light armor, easy to move in.",
    },
    Item {
        name: "Magic Ring",
        description: "Boosts magic power by 10.",
    },
    Item {
        name: "Steel Bow",
        description: "A powerful bow with long range.",
    },
    Item {
        name: "Elixir",
        description: "Fully restores HP and MP.",
    },
    Item {
        name: "Thunder Rod",
        description: "Calls lightning from the sky.",
    },
    Item {
        name: "Silver Amulet",
        description: "Protects against curses.",
    },
    Item {
        name: "Dragon Scale",
        description: "A rare crafting material.",
    },
    Item {
        name: "Antidote",
        description: "Cures poison.",
    },
];

define_ui! {
    InventoryUi,
    ui_tiles: &ui_gfx::UI,
    font: &FONT,
    bake_settings: BakeSettings::new()
        .with_colours(&PALETTE.find([Rgb15::WHITE]))
        .with_backdrop(PALETTE.find([Rgb15::BLACK])[0]),
    layout: {
        rect(0, 0, 14, 18) {
            row(1, 1) {
                static_text("INVENTORY"),
            }
            column(2, 3) {
                dynamic_text item0(10, 1),
                dynamic_text item1(10, 1),
                dynamic_text item2(10, 1),
                dynamic_text item3(10, 1),
                dynamic_text item4(10, 1),
            }
            row(1, 15) {
                dynamic_text page_indicator(12, 1),
            }
        }
        rect(15, 0, 15, 18) {
            row(1, 1) {
                dynamic_text selected_name(13, 1),
            }
            row(1, 3) {
                dynamic_text description(13, 4),
            }
            row(1, 8) {
                static_text("ATK: "),
                number atk(3, left),
            }
            row(1, 10) {
                static_text("DEF: "),
                number def(3, left),
            }
        }
    }
}

struct InventoryState {
    selected: usize,
    page: usize,
}

impl InventoryState {
    fn new() -> Self {
        Self {
            selected: 0,
            page: 0,
        }
    }

    fn total_pages(&self) -> usize {
        (ITEMS.len() + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE
    }

    fn page_start(&self) -> usize {
        self.page * ITEMS_PER_PAGE
    }

    fn page_item_count(&self) -> usize {
        let start = self.page_start();
        let remaining = ITEMS.len().saturating_sub(start);
        remaining.min(ITEMS_PER_PAGE)
    }

    fn selected_item(&self) -> &'static Item {
        &ITEMS[self.page_start() + self.selected]
    }

    fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    fn move_down(&mut self) {
        if self.selected + 1 < self.page_item_count() {
            self.selected += 1;
        }
    }

    fn next_page(&mut self) {
        if self.page + 1 < self.total_pages() {
            self.page += 1;
            self.selected = 0;
        }
    }

    fn prev_page(&mut self) {
        if self.page > 0 {
            self.page -= 1;
            self.selected = 0;
        }
    }
}

fn update_ui(ui: &mut InventoryUi, state: &InventoryState) {
    let start = state.page_start();
    let count = state.page_item_count();

    let item_setters: [fn(&mut InventoryUi, &str); ITEMS_PER_PAGE] = [
        InventoryUi::set_item0,
        InventoryUi::set_item1,
        InventoryUi::set_item2,
        InventoryUi::set_item3,
        InventoryUi::set_item4,
    ];

    for i in 0..ITEMS_PER_PAGE {
        if i < count {
            item_setters[i](ui, ITEMS[start + i].name);
        } else {
            item_setters[i](ui, "");
        }
    }

    let item = state.selected_item();
    ui.set_selected_name(item.name);
    ui.set_description(item.description);

    // Fake stats derived from item index
    let item_idx = (start + state.selected) as i32;
    ui.set_atk(10 + item_idx * 3);
    ui.set_def(5 + item_idx * 2);

    use alloc::format;
    let page_text = format!("Page {}/{}", state.page + 1, state.total_pages());
    ui.set_page_indicator(&page_text);
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    VRAM_MANAGER.set_background_palette(0, &PALETTE);

    let mut gfx = gba.graphics.get();
    let mut input = ButtonController::new();

    let mut bg = RegularBackground::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    let mut ui = InventoryUi::new();
    let mut state = InventoryState::new();

    // Cursor sprite — position in pixels
    // Each item row is 1 tile (8px) tall, starting at tile row 3 inside the rect at tile (0,0)
    // So first item is at pixel y = (0 + 3) * 8 = 24, each subsequent item +8
    let cursor_sprite = cursor_gfx::CURSOR.sprite(0);

    // Smooth cursor animation
    let mut cursor_y_current: i32 = 3 * 8;
    let mut cursor_y_target: i32;

    // Initial UI state
    update_ui(&mut ui, &state);

    loop {
        input.update();

        let mut changed = false;

        if input.is_just_pressed(Button::Up) {
            state.move_up();
            changed = true;
        }
        if input.is_just_pressed(Button::Down) {
            state.move_down();
            changed = true;
        }
        if input.is_just_pressed(Button::Right) {
            state.next_page();
            changed = true;
        }
        if input.is_just_pressed(Button::Left) {
            state.prev_page();
            changed = true;
        }

        if changed {
            update_ui(&mut ui, &state);
        }

        // Smooth cursor: target y based on selected index
        // Items start at tile row 3 in the rect at tile (0,0), so pixel y = (0 + 3 + selected) * 8
        cursor_y_target = (3 + state.selected as i32) * 8;

        // Lerp cursor towards target (move 2px per frame for smooth feel)
        if cursor_y_current < cursor_y_target {
            cursor_y_current = (cursor_y_current + 2).min(cursor_y_target);
        } else if cursor_y_current > cursor_y_target {
            cursor_y_current = (cursor_y_current - 2).max(cursor_y_target);
        }

        ui.show(&mut bg);

        let mut frame = gfx.frame();

        // Show cursor sprite at the left edge of the item list
        Object::new(cursor_sprite)
            .set_pos(vec2(0, cursor_y_current))
            .set_priority(Priority::P0)
            .show(&mut frame);

        bg.show(&mut frame);
        frame.commit();
    }
}
