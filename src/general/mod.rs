// BSL 1.0 License

// TODO: builder pattern

mod bits;
use bits::BitsImage;

mod builder;
pub use builder::Builder;

mod solid;
use solid::SolidColorImage;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::{Endianness, Format, Image, Rgba, U32Buf};

/// A general-purpose image that fits many use cases.
pub struct GeneralImage<Storage> {
    innards: Innards<Storage>,
}

/// A storage type that evaluates to nothing.
///
/// This aims to be a ZST replacement that can be easily used in
/// a [`GeneralImage`] without using any extra memory, in cases
/// where a backing buffer isn't needed.
///
/// [`GeneralImage`]: crate::GeneralImage
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Nothing;

/// Keeping this enum internal means that any changes do not become
/// breaking changes.
enum Innards<Storage> {
    /// An image made up of bits.
    Bits(BitsImage<Storage>),
    /// An image made up of bits that uses a `Vec<u8>` as a backing store.
    ///
    /// For when other types of images are edited, this is used to convert
    /// the image to an editable form.
    #[cfg(feature = "alloc")]
    Buffered(BitsImage<U32Buf<Vec<u32>>>),
    /// An image that is a solid color.
    Solid(SolidColorImage),
}

macro_rules! dispatch {
    (&$self: expr, $fnname: ident $($args: tt)*) => {{
        match ($self).innards {
            Innards::Bits(ref bits) => bits.$fnname $($args)*,
            #[cfg(feature = "alloc")]
            Innards::Buffered(ref bits) => bits.$fnname $($args)*,
            Innards::Solid(ref solid) => solid.$fnname $($args)*,
        }
    }};
    (&mut $self: expr, $fnname: ident $($args: tt)*) => {{
        loop {
            match &mut ($self).innards {
                Innards::Bits(ref mut bits) => return bits.$fnname $($args)*,
                #[cfg(feature = "alloc")]
                Innards::Buffered(ref mut bits) => return bits.$fnname $($args)*,
                _ => {
                    cfg_if::cfg_if! {
                        if #[cfg(feature = "alloc")] {
                            ($self).make_buffered();
                        } else {
                            panic!(
                                concat!(
                                    "Cannot call ",
                                    stringify!($fnname),
                                    " on a non-buffered image"
                                )
                            )
                        }
                    }
                }
            }
        }
    }};
}

impl<Storage> From<Innards<Storage>> for GeneralImage<Storage> {
    fn from(innards: Innards<Storage>) -> Self {
        GeneralImage { innards }
    }
}

impl<Storage> GeneralImage<Storage> {
    /// Create a new image that wraps around a byte buffer.
    pub fn from_buffer(width: usize, height: usize, format: Format, buffer: Storage) -> Self {
        Builder::from_buffer(width, height, format, buffer).finish()
    }
}

impl<Storage: AsRef<[u8]> + AsMut<[u8]>> GeneralImage<Storage> {
    pub fn repeat(&self) -> bool {
        dispatch!(&self, repeat())
    }

    /// Make this buffered.
    #[cfg(feature = "alloc")]
    fn make_buffered(&mut self) {
        use crate::divide_rounding_up;

        // create a heap buffer with enough space to store the
        // current image data
        let heap_buffer_size = self.height() * self.bytes_per_scanline();
        // divide by 4 rounding up
        let heap_buffer_size = divide_rounding_up(heap_buffer_size, 4);

        // ensure it's aligned to a 32-bit boundary
        // divide rounding up by 4 to get u32 size
        let heap_buffer = alloc::vec![0u32; heap_buffer_size];
        let mut bits = BitsImage::with_bytes_per_line(
            self.width(),
            self.height(),
            self.format(),
            self.endianness(),
            self.bytes_per_scanline(),
            self.repeat(),
            U32Buf(heap_buffer),
        );

        let mut line_buffer = alloc::vec![0u8; self.bytes_per_scanline()];

        // copy scanlines from our image to the new one
        for y in 0..self.height() {
            self.scanline(0, y, &mut line_buffer);
            bits.set_scanline(0, y, &line_buffer);
        }

        // set self to the bits
        self.innards = Innards::Buffered(bits);
    }
}

impl GeneralImage<Nothing> {
    /// Creates an image made up of a solid color.
    pub fn solid_color(width: usize, height: usize, format: Format, rgba: Rgba) -> Self {
        Builder::from_solid_color_rgba(width, height, format, rgba).finish()
    }
}

impl<Storage: AsRef<[u8]> + AsMut<[u8]>> Image for GeneralImage<Storage> {
    fn format(&self) -> Format {
        dispatch!(&self, format())
    }

    fn endianness(&self) -> Endianness {
        dispatch!(&self, endianness())
    }

    fn dimensions(&self) -> (usize, usize) {
        dispatch!(&self, dimensions())
    }

    fn bytes_per_scanline(&self) -> usize {
        dispatch!(&self, bytes_per_scanline())
    }

    fn scanline(&self, x: usize, y: usize, scanline: &mut [u8]) -> usize {
        dispatch!(&self, scanline(x, y, scanline))
    }

    fn set_scanline(&mut self, x: usize, y: usize, scanline: &[u8]) -> usize {
        dispatch!(&mut self, set_scanline(x, y, scanline))
    }
}
