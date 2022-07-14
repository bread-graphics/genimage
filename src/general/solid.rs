// BSL 1.0 License

use crate::{Endianness, Format, Pixel};
use core::cmp;

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

    pub(crate) fn scanline(&self, x: usize, y: usize, scanline: &mut [u8]) -> usize {
        // if we're logically outside of the image bounds, return
        if !self.repeat && y >= self.height {
            return 0;
        }

        // determine how many bytes to fill
        let fill = if self.repeat {
            scanline.len()
        } else {
            let byte_index = x
                .saturating_mul(self.format().bpp() as usize)
                .saturating_div(8);
            cmp::min(scanline.len(), self.bytes_per_scanline() - byte_index)
        };

        // fill the scanline with the solid color
        self.pixel.fill_row(&mut scanline[..fill])
    }

    pub(crate) fn format(&self) -> Format {
        self.pixel.format()
    }

    pub(crate) fn endianness(&self) -> Endianness {
        self.pixel.endianness()
    }

    pub(crate) fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub(crate) fn repeat(&self) -> bool {
        self.repeat
    }

    pub(crate) fn bytes_per_scanline(&self) -> usize {
        self.bytes_per_scanline
    }
}
