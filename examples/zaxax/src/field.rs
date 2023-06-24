use agb::{
    display::{bitmap3::Bitmap3, HEIGHT, WIDTH},
    fixnum::{Rect, Vector2D},
};

use crate::Colour;

const CIRCLE: &[(i32, i32)] = &[(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)];

const CIRCLE_REMOVE_AREA: &[(i32, i32)] = &[
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (0, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

pub struct CollisionField<'gba> {
    bitmap: Bitmap3<'gba>,
}

impl<'gba> CollisionField<'gba> {
    pub fn new(mut bitmap: Bitmap3<'gba>) -> Self {
        bitmap.clear(0);

        for x in 0..WIDTH {
            bitmap.draw_point(x, 0, 0xFFFF);
            bitmap.draw_point(x, HEIGHT - 1, 0xFFFF);
        }
        for y in 0..HEIGHT {
            bitmap.draw_point(0, y, 0xFFFF);
            bitmap.draw_point(WIDTH - 1, y, 0xFFFF);
        }

        Self { bitmap }
    }
}

impl CollisionField<'_> {
    fn read_pixel(&self, position: Vector2D<i32>) -> u16 {
        self.bitmap.read_point(position.x, position.y)
    }

    fn set_pixel(&mut self, position: Vector2D<i32>, colour: u16) {
        self.bitmap.draw_point(position.x, position.y, colour);
    }

    fn set_pixel_checked(&mut self, position: Vector2D<i32>, colour: u16) {
        if point_is_within_field(position) {
            self.set_pixel(position, colour);
        }
    }

    pub(crate) fn init_position(&mut self, position: Vector2D<i32>, colour: Colour) {
        for pos in CIRCLE
            .iter()
            .map(|x| Vector2D::new(x.0, x.1))
            .map(|x| x + position)
        {
            self.set_pixel(pos, colour.colour());
        }
    }

    pub(crate) fn update_position(
        &mut self,
        previous_position: Vector2D<i32>,
        position: Vector2D<i32>,
        colour: Colour,
    ) -> bool {
        let previous_position = CIRCLE
            .iter()
            .map(|x| Vector2D::new(x.0, x.1))
            .map(|x| x + previous_position);

        let next_position_check_area: alloc::vec::Vec<_> = CIRCLE_REMOVE_AREA
            .iter()
            .map(|x| Vector2D::new(x.0, x.1))
            .map(|x| x + position)
            .collect();

        for pos in previous_position.filter(|x| !next_position_check_area.contains(x)) {
            self.set_pixel_checked(pos, (1 << 15) | colour.colour());
        }

        let pixel = self.read_pixel(position);
        let found_collision = pixel != 0 && pixel != colour.colour();

        let next_position: alloc::vec::Vec<_> = CIRCLE
            .iter()
            .map(|x| Vector2D::new(x.0, x.1))
            .map(|x| x + position)
            .collect();

        if !found_collision {
            for pos in next_position.iter().copied() {
                let current = self.read_pixel(pos);
                if !(current != 0 && current != colour.colour()) {
                    self.set_pixel_checked(pos, colour.colour());
                }
            }
        } else {
            for pos in next_position.iter().copied() {
                self.set_pixel_checked(pos, (1 << 15) | colour.colour());
            }
        }

        found_collision
    }
}

fn field_rectangle() -> Rect<i32> {
    Rect::new((0, 0).into(), (WIDTH - 1, HEIGHT - 1).into())
}

fn point_is_within_field(position: Vector2D<i32>) -> bool {
    field_rectangle().contains_point(position)
}
