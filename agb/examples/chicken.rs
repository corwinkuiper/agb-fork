#![no_std]
#![no_main]

use agb::{
    display::tiled::{TileFormat, TileSet, TileSetting, TiledMap},
    display::{
        object::{OamDisplay, ObjectUnmanaged, Size, Sprite, SpriteLoader, SpriteVram},
        palette16::Palette16,
        tiled::RegularBackgroundSize,
        HEIGHT, WIDTH,
    },
    input::Button,
};
use agb_fixnum::{num, Num, Vector2D};

#[derive(PartialEq, Eq)]
enum State {
    Ground,
    Upwards,
    Flapping,
}

struct Character {
    sprite: SpriteVram,
    position: Vector2D<Num<i32, 8>>,
    velocity: Vector2D<Num<i32, 8>>,
    flipped: bool,
}

impl OamDisplay for &Character {
    fn set_in(
        self,
        oam: &mut agb::display::object::OamIterator,
    ) -> agb::display::object::OamDisplayResult {
        ObjectUnmanaged::new(self.sprite.clone())
            .set_position((self.position + (num!(0.5), num!(0.5)).into()).floor() - (4, 4).into())
            .set_hflip(self.flipped)
            .set_in(oam)
    }
}

fn tile_is_collidable(tile: u16) -> bool {
    let masked = tile & 0b0000001111111111;
    masked == 0 || masked == 4
}

fn position_is_colliding(tiles: &[[u16; 32]; 32], position: Vector2D<Num<i32, 8>>) -> bool {
    fn inner_check(tiles: &[[u16; 32]; 32], position: Vector2D<Num<i32, 8>>) -> Option<bool> {
        let position = position.floor() / 8;
        Some(tile_is_collidable(
            *tiles.get(position.y as usize)?.get(position.x as usize)?,
        ))
    }

    inner_check(tiles, position).unwrap_or(true)
}

fn frame_ranger(count: u32, start: u32, end: u32, delay: u32) -> usize {
    (((count / delay) % (end + 1 - start)) + start) as usize
}

#[agb::entry]
fn entry(gba: agb::Gba) -> ! {
    main(gba);
}

fn main(mut gba: agb::Gba) -> ! {
    let map_as_grid: &[[u16; 32]; 32] = unsafe {
        (&MAP_MAP as *const [u16; 1024] as *const [[u16; 32]; 32])
            .as_ref()
            .unwrap()
    };

    let (gfx, mut vram) = gba.display.video.tiled0();
    let vblank = agb::interrupt::VBlank::get();
    let mut input = agb::input::ButtonController::new();

    vram.set_background_palette_raw(&MAP_PALETTE);
    let tileset = TileSet::new(&MAP_TILES, TileFormat::FourBpp);

    let mut background = gfx.background(
        agb::display::Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    for (i, &tile) in MAP_MAP.iter().enumerate() {
        let i = i as u16;
        background.set_tile(
            &mut vram,
            (i % 32, i / 32).into(),
            &tileset,
            TileSetting::from_raw(tile),
        );
    }

    background.show();
    background.commit(&mut vram);

    let (mut oam, mut sprite_loader) = gba.display.object.get();

    let sprite = sprite_loader.get_vram_sprite(&CHICKEN_SPRITES[0]);
    let mut chicken = Character {
        sprite,
        position: Vector2D {
            x: (6 * 8).into(),
            y: ((7 * 8) - 4).into(),
        },
        velocity: Vector2D {
            x: 0.into(),
            y: 0.into(),
        },
        flipped: false,
    };

    let acceleration = num!(0.0625);
    let gravity = num!(0.0625);
    let flapping_gravity = gravity / 3;
    let jump_velocity = num!(2.);
    let mut frame_count = 0;
    let mut frames_off_ground = 0;

    let terminal_velocity = num!(0.5);

    loop {
        frame_count += 1;

        input.update();

        // Horizontal movement
        chicken.velocity.x += acceleration * (input.x_tri() as i32);
        chicken.velocity.x = (chicken.velocity.x * 61) / 64;

        // Update position based on collision detection
        let state = handle_collision(
            &mut chicken,
            map_as_grid,
            gravity,
            flapping_gravity,
            terminal_velocity,
        );

        if state != State::Ground {
            frames_off_ground += 1;
        } else {
            frames_off_ground = 0;
        }

        // Jumping code
        if frames_off_ground < 10 && input.is_just_pressed(Button::A) {
            frames_off_ground = 200;
            chicken.velocity.y = -jump_velocity;
        }

        restrict_to_screen(&mut chicken);
        update_chicken_object(&mut chicken, &mut sprite_loader, state, frame_count);
        vblank.wait_for_vblank();
        chicken.set_in(&mut oam.iter());
    }
}

fn update_chicken_object(
    chicken: &mut Character,
    sprite_loader: &mut SpriteLoader,
    state: State,
    frame_count: u32,
) {
    match chicken.velocity.x.to_raw().signum() {
        1 => chicken.flipped = false,
        -1 => chicken.flipped = true,
        _ => {}
    }

    match state {
        State::Ground => {
            if chicken.velocity.x.abs() > num!(0.0625) {
                chicken.sprite = sprite_loader
                    .get_vram_sprite(&CHICKEN_SPRITES[frame_ranger(frame_count, 1, 3, 10)]);
            } else {
                chicken.sprite = sprite_loader.get_vram_sprite(&CHICKEN_SPRITES[0]);
            }
        }
        State::Upwards => {}
        State::Flapping => {
            chicken.sprite =
                sprite_loader.get_vram_sprite(&CHICKEN_SPRITES[frame_ranger(frame_count, 4, 5, 5)]);
        }
    }
}

fn restrict_to_screen(chicken: &mut Character) {
    if chicken.position.x > (WIDTH - 8 + 4).into() {
        chicken.velocity.x = 0.into();
        chicken.position.x = (WIDTH - 8 + 4).into();
    } else if chicken.position.x < 4.into() {
        chicken.velocity.x = 0.into();
        chicken.position.x = 4.into();
    }
    if chicken.position.y > (HEIGHT - 8 + 4).into() {
        chicken.velocity.y = 0.into();
        chicken.position.y = (HEIGHT - 8 + 4).into();
    } else if chicken.position.y < 4.into() {
        chicken.velocity.y = 0.into();
        chicken.position.y = 4.into();
    }
}

fn handle_collision(
    chicken: &mut Character,
    map_as_grid: &[[u16; 32]; 32],
    gravity: Num<i32, 8>,
    flapping_gravity: Num<i32, 8>,
    terminal_velocity: Num<i32, 8>,
) -> State {
    let mut new_chicken_positon = chicken.position + chicken.velocity;

    if (chicken.velocity.x < 0.into()
        && position_is_colliding(map_as_grid, new_chicken_positon - (4, 0).into()))
        || (chicken.velocity.x > 0.into()
            && position_is_colliding(map_as_grid, new_chicken_positon + (4, 0).into()))
    {
        new_chicken_positon.x = (new_chicken_positon.x.floor() / 8 * 8 + 4).into();
        chicken.velocity.x = 0.into();
    }

    if (chicken.velocity.y < 0.into()
        && position_is_colliding(map_as_grid, new_chicken_positon - (0, 4).into()))
        || (chicken.velocity.y > 0.into()
            && position_is_colliding(map_as_grid, new_chicken_positon + (0, 4).into()))
    {
        new_chicken_positon.y = (new_chicken_positon.y.floor() / 8 * 8 + 4).into();
        chicken.velocity.y = 0.into();
    }

    let mut air_animation = State::Ground;

    if !position_is_colliding(map_as_grid, new_chicken_positon + (0, 4).into()) {
        if chicken.velocity.y < 0.into() {
            air_animation = State::Upwards;
            chicken.velocity.y += gravity;
        } else {
            air_animation = State::Flapping;
            chicken.velocity.y += flapping_gravity;
            if chicken.velocity.y > terminal_velocity {
                chicken.velocity.y = terminal_velocity;
            }
        }
    }

    chicken.position = new_chicken_positon;

    air_animation
}

// Below is the data for the sprites

static CHICKEN_PALETTE: Palette16 =
    Palette16::new([0x7C1E, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

static CHICKEN_SPRITES: &[Sprite] = unsafe {
    &[
        Sprite::new(
            &CHICKEN_PALETTE,
            &[
                0x00, 0x00, 0x10, 0x01, 0x00, 0x00, 0x10, 0x11, 0x10, 0x00, 0x10, 0x01, 0x10, 0x11,
                0x11, 0x01, 0x10, 0x11, 0x11, 0x01, 0x00, 0x10, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00,
                0x00, 0x10, 0x01, 0x00,
            ],
            Size::S8x8,
        ),
        Sprite::new(
            &CHICKEN_PALETTE,
            &[
                0x00, 0x00, 0x10, 0x01, 0x00, 0x00, 0x10, 0x11, 0x10, 0x00, 0x10, 0x01, 0x10, 0x11,
                0x11, 0x01, 0x10, 0x11, 0x11, 0x01, 0x00, 0x01, 0x01, 0x00, 0x00, 0x01, 0x10, 0x00,
                0x10, 0x00, 0x00, 0x00,
            ],
            Size::S8x8,
        ),
        Sprite::new(
            &CHICKEN_PALETTE,
            &[
                0x00, 0x00, 0x10, 0x01, 0x00, 0x00, 0x10, 0x11, 0x10, 0x00, 0x10, 0x01, 0x10, 0x11,
                0x11, 0x01, 0x10, 0x11, 0x11, 0x01, 0x00, 0x10, 0x01, 0x00, 0x10, 0x01, 0x10, 0x00,
                0x00, 0x00, 0x10, 0x00,
            ],
            Size::S8x8,
        ),
        Sprite::new(
            &CHICKEN_PALETTE,
            &[
                0x00, 0x00, 0x10, 0x01, 0x00, 0x00, 0x10, 0x11, 0x10, 0x00, 0x10, 0x01, 0x10, 0x11,
                0x11, 0x01, 0x10, 0x11, 0x11, 0x01, 0x00, 0x10, 0x01, 0x00, 0x00, 0x11, 0x01, 0x00,
                0x00, 0x10, 0x00, 0x00,
            ],
            Size::S8x8,
        ),
        Sprite::new(
            &CHICKEN_PALETTE,
            &[
                0x00, 0x00, 0x10, 0x01, 0x00, 0x11, 0x11, 0x11, 0x10, 0x10, 0x11, 0x01, 0x10, 0x11,
                0x11, 0x01, 0x10, 0x11, 0x11, 0x01, 0x00, 0x10, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ],
            Size::S8x8,
        ),
        Sprite::new(
            &CHICKEN_PALETTE,
            &[
                0x00, 0x00, 0x10, 0x01, 0x00, 0x00, 0x10, 0x11, 0x10, 0x11, 0x11, 0x01, 0x10, 0x11,
                0x11, 0x01, 0x10, 0x11, 0x11, 0x01, 0x00, 0x10, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ],
            Size::S8x8,
        ),
    ]
};

static MAP_TILES: [u8; 8 * 17 * 4] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x01, 0x11, 0x11, 0x11, 0x01, 0x11, 0x11, 0x11, 0x01,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x11, 0x11, 0x00, 0x00, 0x10, 0x11,
    0x00, 0x01, 0x00, 0x11, 0x00, 0x11, 0x00, 0x10, 0x00, 0x11, 0x01, 0x00, 0x00, 0x11, 0x11, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x11, 0x01, 0x11, 0x00, 0x10, 0x01,
    0x11, 0x01, 0x00, 0x01, 0x11, 0x11, 0x00, 0x00, 0x11, 0x11, 0x01, 0x00, 0x11, 0x11, 0x11, 0x00,
    0x11, 0x11, 0x11, 0x00, 0x11, 0x11, 0x11, 0x00, 0x11, 0x11, 0x11, 0x00, 0x11, 0x11, 0x11, 0x00,
    0x11, 0x11, 0x11, 0x00, 0x11, 0x11, 0x11, 0x00, 0x11, 0x11, 0x00, 0x00, 0x11, 0x01, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x10, 0x11, 0x10, 0x10, 0x01,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x10, 0x10, 0x01,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x10, 0x11, 0x11, 0x01, 0x11, 0x10,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x10, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x01, 0x11, 0x11, 0x11, 0x11, 0x10, 0x11, 0x11, 0x11, 0x10, 0x11, 0x10,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x10, 0x11, 0x10,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x10, 0x11, 0x11, 0x10, 0x10, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x01, 0x11, 0x11, 0x01, 0x01, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x01, 0x11, 0x01, 0x01, 0x11,
];

static MAP_MAP: [u16; 1024] = [
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0002, 0x0003, 0x0003, 0x0003, 0x0003,
    0x0003, 0x0003, 0x0003, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0004,
    0x0404, 0x0004, 0x0404, 0x0004, 0x0404, 0x0004, 0x0404, 0x0004, 0x0404, 0x0004, 0x0404, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0804, 0x0C04, 0x0804, 0x0C04, 0x0804,
    0x0C04, 0x0804, 0x0C04, 0x0804, 0x0C04, 0x0804, 0x0C04, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0005, 0x0405, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0005, 0x0405, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0006,
    0x0406, 0x0002, 0x0003, 0x0003, 0x0003, 0x0003, 0x0003, 0x0003, 0x0001, 0x0006, 0x0406, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0000, 0x0000, 0x0004, 0x0404, 0x0004,
    0x0404, 0x0004, 0x0404, 0x0004, 0x0404, 0x0000, 0x0000, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0000, 0x0000, 0x0007, 0x0007, 0x0007, 0x0007, 0x0007, 0x0007, 0x0007,
    0x0007, 0x0000, 0x0000, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0008, 0x0009, 0x000A, 0x0000,
    0x0000, 0x000B, 0x000C, 0x000D, 0x000B, 0x000E, 0x0008, 0x0009, 0x000A, 0x0000, 0x0000, 0x000B,
    0x000B, 0x000C, 0x000D, 0x000B, 0x000E, 0x0008, 0x0009, 0x000A, 0x000F, 0x0010, 0x000B, 0x000C,
    0x000D, 0x000B, 0x000E, 0x0008, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000,
];

static MAP_PALETTE: [u16; 2] = [0x0000, 0x6A2F];
