// BSL 1.0 License

// TODO: builder pattern

mod bits;
use bits::BitsImage;

mod builder;
pub use builder::Builder;

mod solid;
use solid::SolidColorImage;

use alloc::vec::Vec;

use crate::{Endianness, Format, Rgba};

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
    Buffered(BitsImage<Vec<u8>>),
    /// An image that is a solid color.
    Solid(SolidColorImage),
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

impl GeneralImage<Nothing> {
    /// Creates an image made up of a solid color.
    pub fn solid_color(color: Rgba, width: usize, height: usize, endianness: Endianness) -> Self {
        todo!()
    }
}
