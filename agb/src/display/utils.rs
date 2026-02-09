//! Utilities for graphics.

/// Copies the content of `src` into `target` but skipping transparent pixels.
///
/// Assumes 1 pixel is 4 bits, so useful for copying into 4-bit dynamic tiles
/// like in [`DynamicTile16`](crate::display::tiled::DynamicTile16) or
/// [`DynamicSprite16`](crate::display::object::DynamicSprite16).
///
/// Normally you shouldn't need to use this method, as you should be using
/// [`Object`s](crate::display::object::Object) or backgrounds. Only use this
/// if you specifically need dynamic sprites or tiles.
///
/// # Examples
///
/// ```
/// # #![no_main]
/// # #![no_std]
/// # #[agb::doctest]
/// # fn test(_: agb::Gba) {
/// use agb::display::utils::blit_16_colour;
///
/// let a = &mut [0x89abcdef];
/// let b = &[0x01030507];
///
/// blit_16_colour(a, b);
/// assert_eq!(a[0], 0x81a3c5e7);
/// # }
/// ```
pub const fn blit_16_colour(target: &mut [u32], src: &[u32]) {
    assert!(
        target.len() == src.len(),
        "Target and source must have the same length"
    );

    let mut idx = 0;
    let len = target.len();

    while idx < len {
        let a = &mut target[idx];
        let b = src[idx];

        let hi = b & 0x8888_8888;
        let lo = b & 0x7777_7777;

        let set_nybbles = (hi | ((lo + 0x7777_7777) & 0x8888_8888)) >> 3;
        let mask = set_nybbles * 0xf;

        *a = (*a & !mask) | b;

        idx += 1;
    }
}

/// Copies the content of `src` into `target` but skipping transparent pixels.
///
/// Assumes 1 pixel is 1 byte, so useful for copying into 8-bit dynamic tile
/// like in [`DynamicTile256`](crate::display::tiled::DynamicTile256) or
/// [`DynamicSprite256`](crate::display::object::DynamicSprite256).
///
/// Normally you shouldn't need to use this method, as you should be using
/// [`Object`s](crate::display::object::Object) or backgrounds. Only use this
/// if you specifically need dynamic sprites or tiles.
///
/// # Examples
///
/// ```
/// # #![no_main]
/// # #![no_std]
/// # #[agb::doctest]
/// # fn test(_: agb::Gba) {
/// use agb::display::utils::blit_256_colour;
///
/// let a = &mut [0x89abcdef];
/// let b = &[0xabcd0000];
///
/// blit_256_colour(a, b);
/// assert_eq!(a[0], 0xabcdcdef);
/// # }
/// ```
pub const fn blit_256_colour(target: &mut [u32], src: &[u32]) {
    assert!(
        target.len() == src.len(),
        "Target and source must have the same length"
    );

    let mut idx = 0;
    let len = target.len();

    while idx < len {
        let a = &mut target[idx];
        let b = src[idx];

        let hi = b & 0x8080_8080;
        let lo = b & 0x7f7f_7f7f;

        let set_bytes = (hi | ((lo + 0x7f7f_7f7f) & 0x8080_8080)) >> 7;
        let mask = set_bytes * 0xff;

        *a = (*a & !mask) | b;

        idx += 1;
    }
}

/// Converts a slice of `u32` values into a slice of `u8` values.
///
/// This function provides a way to reinterpret a slice of 32-bit values
/// as a slice of 8-bit values, without copying the data.
pub const fn cast_u32_to_bytes(a: &[u32]) -> &[u8] {
    unsafe { core::slice::from_raw_parts(a.as_ptr().cast(), a.len() * 4) }
}

/// Converts a slice of `u8` values into a slice of `u32` values.
///
/// This function provides a way to reinterpret a slice of 8-bit values
/// as a slice of 32-bit values, without copying the data.
///
/// # Safety
///
/// * The caller must guarantee that the length of `a` is a multiple of 4.
/// * The caller must guarantee that the slice `a` is 4-byte aligned.
pub const unsafe fn cast_bytes_to_u32(a: &[u8]) -> &[u32] {
    unsafe { core::slice::from_raw_parts(a.as_ptr().cast(), a.len() / 4) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn blit_16_simple(_: &mut crate::Gba) {
        let a = &mut [0x89abcdef];
        let b = &[0x01030507];
        blit_16_colour(a, b);
        assert_eq!(a[0], 0x81a3c5e7);
    }

    #[test_case]
    fn blit_256_simple(_: &mut crate::Gba) {
        let a = &mut [0x89abcdef];
        let b = &[0xabcd0000];
        blit_256_colour(a, b);
        assert_eq!(a[0], 0xabcdcdef);
    }
}
