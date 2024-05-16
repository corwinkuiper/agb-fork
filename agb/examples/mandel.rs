#![no_std]
#![no_main]

use core::ops::{Add, Mul};

use agb::display::{bitmap3::Bitmap3, HEIGHT, WIDTH};
use agb_fixnum::{num, Num};

type BaseNumber = Num<i32, 12>;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
struct Complex {
    real: BaseNumber,
    imaginary: BaseNumber,
}

impl Complex {
    fn new(real: BaseNumber, imaginary: BaseNumber) -> Self {
        Self { real, imaginary }
    }
}

impl Mul for Complex {
    type Output = Complex;

    fn mul(self, rhs: Self) -> Self::Output {
        Complex {
            real: self.real * rhs.real - self.imaginary * rhs.imaginary,
            imaginary: self.real * rhs.imaginary + self.imaginary * rhs.real,
        }
    }
}

impl Add for Complex {
    type Output = Complex;

    fn add(self, rhs: Self) -> Self::Output {
        Complex {
            real: self.real + rhs.real,
            imaginary: self.imaginary + rhs.imaginary,
        }
    }
}

const MAX_ITERATIONS: usize = 32;

fn do_iterations(c: Complex) -> Option<usize> {
    let mut current = c;
    for iterations in 0..MAX_ITERATIONS {
        if current.real * current.real + current.imaginary * current.imaginary > (2 * 2).into() {
            return Some(iterations);
        }

        current = current * current + c;
    }

    None
}

fn draw_mandel(bitmap: &mut Bitmap3) {
    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            let xx = (BaseNumber::new(x) - num!(0.8) * WIDTH) * num!(3.) / WIDTH;
            let yy = (BaseNumber::new(y) - num!(0.5) * HEIGHT) * num!(2.5) / HEIGHT;

            if let Some(iterations) = do_iterations(Complex::new(xx, yy)) {
                bitmap.draw_point(
                    x,
                    y,
                    0xFFFF - 0b0000010000100001 * (iterations * 32 / MAX_ITERATIONS) as u16,
                );
            } else {
                bitmap.draw_point(x, y, 0);
            }
        }
    }
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut bitmap = gba.display.video.bitmap3();

    draw_mandel(&mut bitmap);
    loop {
        agb::syscall::halt();
    }
}
