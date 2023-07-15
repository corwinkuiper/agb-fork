#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

extern crate alloc;

use agb::{
    display::{Font, HEIGHT, WIDTH},
    fixnum::{num, Num, Vector2D},
    include_font,
    input::{Button, Tri},
};
use field::CollisionField;
use text::CenteredTextRender;

mod field;
mod text;

use core::fmt::Write;

const YOSTER: Font = include_font!("fnt/yoster.ttf", 12);

#[derive(Clone, Copy, PartialEq, Eq)]
struct Colour(u16);

impl Colour {
    fn new<U: Into<u32>>(r: U, g: U, b: U) -> Self {
        fn into(r: u32, g: u32, b: u32) -> Colour {
            let max = 1 << 5;
            assert!(r < max);
            assert!(g < max);
            assert!(b < max);
            let c = (b << 10) | (g << 5) | (r);
            Colour(c as u16)
        }

        into(r.into(), g.into(), b.into())
    }

    fn colour(self) -> u16 {
        self.0
    }
}

struct Snake {
    angle: Num<i32, 8>,
    position: Vector2D<Num<i32, 8>>,
    speed: Num<i32, 8>,
    rotation_speed: Num<i32, 8>,
    colour: Colour,
    has_lost: bool,
}

impl Snake {
    fn process_frame(&mut self, direction: Tri, field: &mut CollisionField) -> bool {
        self.angle = (self.angle + self.rotation_speed * direction as i32).rem_euclid(1.into());

        let angle_unit_vector = Vector2D::new_from_angle(self.angle);
        let displacement = angle_unit_vector * self.speed;
        let previous_position = self.position;
        self.position += displacement.try_change_base().unwrap();

        self.has_lost |= field.update_position(
            previous_position.floor(),
            self.position.floor(),
            self.colour,
        );

        self.has_lost
    }
}

pub fn entry(gba: &mut agb::Gba) {
    let vblank = agb::interrupt::VBlank::get();
    let mut input = agb::input::ButtonController::new();

    loop {
        let mut field = CollisionField::new(gba.display.video.bitmap3());

        let mut snake = Snake {
            angle: 0.into(),
            position: (WIDTH / 4, HEIGHT / 2).into(),
            speed: num!(0.5),
            rotation_speed: num!(0.015),
            colour: Colour::new(0, 0, 31_u32),
            has_lost: false,
        };

        let mut snake2 = Snake {
            angle: num!(0.5),
            position: (3 * WIDTH / 4, HEIGHT / 2).into(),
            speed: num!(0.5),
            rotation_speed: num!(0.015),
            colour: Colour::new(31_u32, 0, 0),
            has_lost: false,
        };

        loop {
            vblank.wait_for_vblank();
            input.update();

            let direction = input.x_tri();
            let direction2 = Tri::from((input.is_pressed(Button::B), input.is_pressed(Button::A)));

            if snake.process_frame(direction, &mut field)
                || snake2.process_frame(direction2, &mut field)
            {
                let winner = [("Blue", snake.has_lost), ("Red", snake2.has_lost)]
                    .into_iter()
                    .find(|x| !x.1)
                    .map(|x| x.0);

                let mut text_render = CenteredTextRender::new(&YOSTER, field.bitmap(), 0xFF_FF);

                if let Some(winner) = winner {
                    let _ = writeln!(text_render, "{} wins!", winner);
                } else {
                    let _ = writeln!(text_render, "There is no winner");
                }

                text_render.render_line_centered((WIDTH / 2, HEIGHT / 3).into());

                for _ in 0..100 {
                    vblank.wait_for_vblank();
                }
                break;
            }
        }
    }
}
