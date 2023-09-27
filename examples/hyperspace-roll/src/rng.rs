pub fn roll_dice(number_of_dice: u32, bits_per_dice: u32) -> u32 {
    assert!(
        32 % bits_per_dice == 0,
        "the number of bits per dice should be a multiple of 32"
    );

    assert!(
        number_of_dice % (32 / bits_per_dice) == 0,
        "number of dice should be a multiple of 32 / bits per dice"
    );

    fn roll_dice_inner(number_of_random_values: u32, bits_per_dice: u32) -> u32 {
        let mut count = 0;
        let bit_mask = 1u32.wrapping_shl(bits_per_dice).wrapping_sub(1);

        for _ in 0..number_of_random_values {
            let n = agb::rng::gen() as u32;
            for idx in 0..(32 / bits_per_dice) {
                count += (n >> (bits_per_dice * idx)) & bit_mask;
            }
        }

        count
    }

    roll_dice_inner(number_of_dice / (32 / bits_per_dice), bits_per_dice)
}

// uses multiple dice rolls to generate a random value with a specified mean and width
pub fn roll_dice_scaled(number_of_dice: u32, bits_per_dice: u32, width: u32) -> i32 {
    let dice = roll_dice(number_of_dice, bits_per_dice) as i32;

    let current_width = (number_of_dice * ((1 << bits_per_dice) - 1)) as i32;
    let current_mean = current_width / 2;

    let dice_around_zero = dice - current_mean;

    fn divide_nearest(numerator: i32, denominator: i32) -> i32 {
        if (numerator < 0) ^ (denominator < 0) {
            (numerator - denominator / 2) / denominator
        } else {
            (numerator + denominator / 2) / denominator
        }
    }
    divide_nearest(dice_around_zero * width as i32, current_width)
}

pub fn default_roll(width: u32) -> i32 {
    roll_dice_scaled(2, 16, width)
}

pub fn gen() -> i32 {
    agb::rng::gen()
}
