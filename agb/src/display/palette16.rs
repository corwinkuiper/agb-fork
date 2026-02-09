#![warn(missing_docs)]
use super::Rgb15;

/// Represents a palette of 16 colours.
///
/// The Game Boy Advance can have up to 16, 16 colour palettes active at once. For
/// objects, these are loaded dynamically as needed, but for backgrounds you will
/// need to manually load the palettes using
/// [`VRamManager::set_background_palette`](crate::display::tiled::VRamManager::set_background_palette)
#[repr(C)]
#[derive(Clone)]
pub struct Palette16 {
    pub(crate) colours: [Rgb15; 16],
}

impl Palette16 {
    /// Create a new palette with the given 16 colours.
    #[must_use]
    pub const fn new(colours: [Rgb15; 16]) -> Self {
        Palette16 { colours }
    }

    /// Set the colour at given `index` to the given colour.
    ///
    /// Index must be less than 16 or this function will panic.
    #[track_caller]
    pub const fn update_colour(&mut self, index: usize, colour: Rgb15) {
        self.colours[index] = colour;
    }

    /// Gets the colour for a given index.
    ///
    /// Index must be less than 16 or this function will panic.
    #[must_use]
    #[track_caller]
    pub const fn colour(&self, index: usize) -> Rgb15 {
        self.colours[index]
    }

    /// Extends the palette with the given colours, skipping duplicates.
    ///
    /// New colours are appended after the last non-black colour in the palette.
    /// Colours already present in the palette are not added again.
    /// If the palette is full, panics.
    pub const fn extend(&self, colours: &[Rgb15]) -> Palette16 {
        let mut output = Palette16 {
            colours: self.colours,
        };

        // scan backwards to find colour
        let mut insert_idx = 15;
        while output.colour(insert_idx).0 == Rgb15::BLACK.0 && insert_idx != 0 {
            insert_idx -= 1;
        }
        if output.colour(insert_idx).0 != Rgb15::BLACK.0 {
            insert_idx += 1;
        }

        // insert colour if it doesn't exist
        let mut colour_idx = 0;
        'outer: while colour_idx < colours.len() {
            let colour = colours[colour_idx];
            colour_idx += 1;

            let mut search_idx = 0;
            while search_idx < insert_idx {
                if output.colour(search_idx).0 == colour.0 {
                    continue 'outer;
                }
                search_idx += 1;
            }

            if insert_idx >= 16 {
                panic!("Unable to extend with colours given");
            }

            output.update_colour(insert_idx, colour);
            insert_idx += 1;
        }

        output
    }

    /// Finds the index of each given colour in the palette.
    ///
    /// Returns an array of indices corresponding to each input colour's
    /// position in the palette. Panics if any colour is not found.
    pub const fn find<const N: usize>(&self, colours: [Rgb15; N]) -> [u8; N] {
        let mut indexes = [0; N];
        let mut i = 0;
        while i < N {
            let mut j = 0;
            let colour = colours[i];
            while j < 16 {
                if self.colours[j].0 == colour.0 {
                    indexes[i] = j as u8;
                    break;
                }
                j += 1;
            }
            if j == 16 {
                panic!("Colour not found");
            }
            i += 1;
        }
        indexes
    }
}

#[cfg(test)]
mod tests {
    use crate::display::{Palette16, Rgb15};

    #[test_case]
    fn check_extend_works(_: &mut crate::Gba) {
        let palette = Palette16::new([const { Rgb15::new(0x0) }; 16]);
        let palette = palette.extend(&[Rgb15::new(1)]);
        assert_eq!(palette.colour(0), Rgb15::new(1));
        let palette = palette.extend(&[Rgb15::new(2), Rgb15::new(1)]);

        assert_eq!(palette.colour(0), Rgb15::new(1));
        assert_eq!(palette.colour(1), Rgb15::new(2));
        assert_eq!(palette.colour(2), Rgb15::new(0));
    }

    #[test_case]
    fn check_find_works(_: &mut crate::Gba) {
        let mut palette = Palette16::new([const { Rgb15::new(0x0) }; 16]);
        palette.update_colour(0, Rgb15::new(10));
        palette.update_colour(1, Rgb15::new(20));
        palette.update_colour(2, Rgb15::new(30));

        assert_eq!(palette.find([Rgb15::new(20)]), [1]);
        assert_eq!(palette.find([Rgb15::new(30), Rgb15::new(10)]), [2, 0]);
        assert_eq!(palette.find([Rgb15::new(10), Rgb15::new(10)]), [0, 0]);
    }
}
