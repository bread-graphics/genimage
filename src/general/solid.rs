// BSL 1.0 License

use crate::{ColorType, Endianness, Format, Pixel, Rgba};
use ordered_float::NotNan;

/// An image made up entirely of a solid color.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SolidColorImage {
    width: usize,
    height: usize,
    bytes_per_scanline: usize,
    repeat: bool,
    pixel: Pixel,
}

impl SolidColorImage {
    pub(crate) fn with_bytes_per_line(
        width: usize,
        height: usize,
        bytes_per_scanline: usize,
        repeat: bool,
        pixel: Pixel,
    ) -> Self {
        Self {
            width,
            height,
            bytes_per_scanline,
            repeat,
            pixel,
        }
    }
}
