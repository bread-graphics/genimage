// BSL 1.0 License

use super::{BitsImage, GeneralImage, Innards, SolidColorImage};
use crate::{divide_rounding_up, Endianness, Format, Nothing, Pixel, Rgba};
use const_fn::const_fn;

/// A builder that allows the user to construct images.
#[derive(Debug)]
pub struct Builder<Storage> {
    width: usize,
    height: usize,
    bytes_per_scanline: usize,
    repeat: bool,
    variant: Variant<Storage>,
}

#[derive(Debug)]
enum Variant<Storage> {
    Bits {
        storage: Storage,
        format: Format,
        endianness: Endianness,
    },
    SolidColor {
        pixel: Pixel,
    },
}

impl<Storage> Builder<Storage> {
    const fn new_with_variant(
        width: usize,
        height: usize,
        format: Format,
        variant: Variant<Storage>,
    ) -> Self {
        Self {
            width,
            height,
            bytes_per_scanline: bytes_per_scanline(width, format.bpp()),
            repeat: false,
            variant,
        }
    }

    /// Create a new image builder for an image that wraps
    /// around a byte buffer.
    pub const fn from_buffer(
        width: usize,
        height: usize,
        format: Format,
        storage: Storage,
    ) -> Self {
        Self::new_with_variant(
            width,
            height,
            format,
            Variant::Bits {
                storage,
                format,
                endianness: Endianness::NATIVE,
            },
        )
    }

    /// Use a custom number of bytes per scanline.
    ///
    /// Sometimes, it is useful to use a custom number of bytes per scanline.
    /// For instance, if extra padding is expected by the system for each line.
    ///
    /// # Panics
    ///
    /// - If the value is less than `width * bpp / 8` rounded up, this function
    ///   will panic, since there is not enough room for `width` pixels in a scanline.
    /// - If the value is not a multiple of `bpp / 8` rounded up, this function
    ///   will panic, since the scanline is not aligned to the pixel.
    #[const_fn("1.57")]
    pub const fn with_bytes_per_scanline(mut self, value: usize) -> Self {
        assert!(
            value >= self.width * self.bytes_per_scanline,
            "The number of bytes per scanline must be at least the number of bytes per pixel times the width of the image."
        );
        assert!(
            value % self.bytes_per_scanline == 0,
            "The number of bytes per scanline must be a multiple of the number of bytes per pixel."
        );
        self.bytes_per_scanline = value;
        self
    }

    /// Use a different endianness for the image.
    pub fn with_endianness(mut self, endianness: Endianness) -> Self {
        self.variant = self.variant.with_endianness(endianness);
        self
    }

    /// Repeat this image.
    pub const fn repeat(mut self) -> Self {
        self.repeat = true;
        self
    }

    /// Finish building the image.
    pub fn finish(self) -> GeneralImage<Storage> {
        // disassemble the builder
        let Self {
            width,
            height,
            repeat,
            bytes_per_scanline,
            variant,
        } = self;

        let innards = match variant {
            Variant::Bits {
                storage,
                format,
                endianness,
            } => {
                // create the bits image
                let bits = BitsImage::with_bytes_per_line(
                    width,
                    height,
                    format,
                    endianness,
                    bytes_per_scanline,
                    repeat,
                    storage,
                );
                Innards::Bits(bits)
            }
            Variant::SolidColor { pixel } => {
                let solid = SolidColorImage::with_bytes_per_line(
                    width,
                    height,
                    bytes_per_scanline,
                    repeat,
                    pixel,
                );
                Innards::Solid(solid)
            }
        };

        innards.into()
    }
}

impl Builder<Nothing> {
    /// Create a new image builder for an image consisting entirely
    /// of a solid color.
    pub const fn from_solid_color(width: usize, height: usize, pixel: Pixel) -> Self {
        Self::new_with_variant(width, height, pixel.format(), Variant::SolidColor { pixel })
    }

    /// Create a new image builder for a solid color image of the given
    /// color.
    pub fn from_solid_color_rgba(width: usize, height: usize, format: Format, rgba: Rgba) -> Self {
        Self::from_solid_color(
            width,
            height,
            Pixel::from_rgba(rgba, format, Endianness::NATIVE),
        )
    }
}

impl<Storage> Variant<Storage> {
    fn with_endianness(mut self, endian: Endianness) -> Self {
        match self {
            Variant::Bits {
                ref mut endianness, ..
            } => {
                *endianness = endian;
            }
            Variant::SolidColor { ref mut pixel, .. } => {
                *pixel = pixel.into_new_format(endian, pixel.format());
            }
        }

        self
    }
}

const fn bytes_per_scanline(width: usize, bpp: u8) -> usize {
    divide_rounding_up(width * bpp as usize, 8)
}
