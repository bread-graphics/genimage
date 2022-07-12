// BSL 1.0 License

//! A library for dealing with generic, byte-based images.
//!
//! The [`image`] crate is a fantastic crate, for many reasons.
//! Combined with the [`imageproc`] crate, it can preform just about
//! any logical image manipulation possible.
//!
//! However, the real world is not always logical. When working with
//! images on a low level (e.g. for a graphics framework), it is
//! often useful to pretend that images are just two-dimensional
//! arrays of "pixels" that may be bytes, floats, or anything in
//! between. In addition, it is often desirable to be able to
//! store images in a one-dimensional vector of bytes for use in
//! transmission across framework boundaries. This crate aims to
//! provide traits and structures to allow this to become significantly
//! easier.
//!
//! ## The [`Image`] trait
//!
//! [`Image`] is the main trait in this crate. It defines several
//! elementary properties and operations on images. Notable properties
//! include:
//!
//! - The width and height of the image.
//! - The [`Format`] of the image. This structure aims to store the
//!   format of the information in the image. For instance, the
//!   [`ARGB32`] format is an image made up of 32-bit quantums
//!   (represented as `u32`) in the order of `[A, R, G, B]`.
//! - Filling a buffer with a [`scanline`] from the image.
//! - Filling the image's buffer via [`set_scanline`].
//!
//! The [`Image`] trait does not allow direct access to a byte buffer.
//! This allows non-standard images to be used. For instance, an image
//! that consists entirely of a solid color could be used without
//! allocating an entire buffer. The [`scanline`] function would be
//! implemented by just filling the input buffer with the given color.
//!
//! Much like the [`image`] crate, it is also possible to preform
//! pixel-level manipuations. See [`pixel`], [`set_pixel`], and the
//! [`Pixel`] structure.
//!
//! ## The [`GeneralImage`] Structure
//!
//! The [`GeneralImage`] structure provides a general-purpose implementation
//! of [`Image`] that is intended to be usable in many situations. Internally,
//! it is implemented as a sum type over several different types of images.
//! For instance, you can create a [`GeneralImage`] that doesn't allocate and
//! only uses a solid color using the [`solid_color`] function.
//!
//! When you need a buffer-oriented image, you can use the [`from_buffer`]
//! constructor. [`GeneralImage`] can act as a wrapper around any implementor
//! of [`AsRef<[u8]>`], including static slices, arrays and [`Vec`]s.
//!
//! With the `alloc` feature enabled, if a [`GeneralImage`] that is backed by
//! a non-buffer is edited, it will allocate a buffer and copy the data into
//! there. This is useful for when you need to edit an image that is not
//! backed by a buffer. Without the `alloc` feature, mutating this image will
//! panic.
//!
//! ## [`image`] compatibility
//!
//! With the `image` feature enabled, types that implement
//! [`GenericImageView`] implement the [`Image`] trait, for compatibility's
//! sake. However, it will only be implemented if the image's `Pixel`
//! implements the [`CompatiblePixel`] trait.
//!
//! [`image`]: https://crates.io/crates/image
//! [`imageproc`]: https://crates.io/crates/imageproc
//! [`ARGB32`]: crate::Format::ARGB32
//! [`Image`]: crate::Image
//! [`Format`]: crate::Format
//! [`scanline`]: crate::Image::scanline
//! [`set_scanline`]: crate::Image::set_scanline
//! [`pixel`]: crate::Image::pixel
//! [`set_pixel`]: crate::Image::set_pixel
//! [`Pixel`]: crate::Pixel
//! [`GeneralImage`]: crate::GeneralImage
//! [`solid_color`]: crate::GeneralImage::solid_color
//! [`from_buffer`]: crate::GeneralImage::from_buffer
//! [`AsRef<[u8]>`]: std::convert::AsRef
//! [`Vec`]: std::vec::Vec
//! [`GenericImageView`]: image::GenericImageView

#![forbid(unsafe_code, future_incompatible, rust_2018_idioms)]
#![no_std]

#[cfg(feature = "general_image")]
extern crate alloc;

pub(crate) mod array;
pub(crate) mod assert_exact_size;

mod color;
pub use color::Rgba;

mod format;
pub(crate) use format::MAX_BYTES_PER_PIXEL;
pub use format::{Channel, ColorType, Format};

mod pixel;
pub use pixel::{ChannelValue, Pixel};

#[cfg(feature = "general_image")]
mod general;
#[cfg(feature = "general_image")]
pub use general::{Builder, GeneralImage, Nothing};

/// The centerpiece trait for this library.
///
/// This trait represents a byte-oriented two-dimensional array of
/// values generally understood to represent the pixels of an image.
/// See the [crate level documentation] for more information.
///
/// [crate level documentation]: index.html
pub trait Image {
    /// The format of the image.
    ///
    /// This function should return a structure that defines how the
    /// bytes of the image should be interpreted. This format includes:
    ///
    /// - The number of bits needed to represent a pixel.
    /// - The colors channels contained in the pixel.
    /// - How many bytes each channel uses.
    ///
    /// For more information, see the [`Format`] structure.
    ///
    /// ## Example
    ///
    /// todo
    ///
    /// [`Format`]: crate::Format
    fn format(&self) -> Format;

    /// The endianness of the image.
    ///
    /// Physically, images consist of bytes. However, most formats
    /// group these bytes into "quantums" which can be represented as
    /// 16-bit or 32-bit values. This function returns the endianness
    /// of the quantums.
    ///
    /// It is optimal to deal with images in the format of
    /// [`Endianness::NATIVE`]. However, the situation may arise where
    /// you need to deal with images in a different endianness, such as
    /// if they are sent from another computer.
    fn endianness(&self) -> Endianness;

    /// Logical dimensions of the image.
    ///
    /// This describes the width and the height for the image.
    /// The height is the number of scanlines that the image has.
    /// The width is the number of pixels per scanline.
    fn dimensions(&self) -> (usize, usize);

    /// Logical width of the image.
    fn width(&self) -> usize {
        let (width, _) = self.dimensions();
        width
    }

    /// Logical height of the image.
    fn height(&self) -> usize {
        let (_, height) = self.dimensions();
        height
    }

    /// Number of bytes per scanline.
    ///
    /// In many cases, while you only have a given number of pixels in
    /// a given scanline, it is desirable to pad it to a multiple of
    /// some number that the system likes. This function returns the
    /// number of bytes that can be expected in a given scanline.
    ///
    /// ## Notes
    ///
    /// This number is expected to be a multiple of [`self.format().bytes()`].
    /// If it isn't, logical errors will occur.
    ///
    /// [`self.format().bytes()`]: crate::Image::format
    fn bytes_per_scanline(&self) -> usize;

    /// Fill a scanline with bytes from this image.
    ///
    /// The `y` coordinate is the logical scanline index. The `x` coordinate
    /// describes where to start the scanline. It should be guaranteed that the
    /// first byte of `scanline` will contain the pixel at `x`. For formats where
    /// a pixel's size is greater than or equal to one byte, the first byte of
    /// `scanline` will be the first byte of the pixel.
    ///
    /// The desired number of bytes is determined by the length of the `scanline`
    /// parameter.
    ///
    /// This function should return the number of bytes written to `scanline`.
    /// This number may be less than or equal to the length of `scanline`.
    ///
    /// It is up to the user to interpret the bytes of `scanline` into whatever
    /// form they desire, depending on the result of [`format`] and [`endianness`].
    /// For instance:
    ///
    /// - For formats where the pixel's size is less than one byte, the first
    ///   byte of the result should be sliced appropriately until the desired
    ///   bits are extracted.
    /// - For formats where the pixels' size is one byte, no further interpretation
    ///   is necessary.
    /// - For formats where the pixels' size is 16 or 32 bytes, the array should
    ///   be cast to an array of `u16`s or `u32`s, respectively. `bytemuck`'s
    ///   [`cast_slice`] function provides a safe way of doing this. Note that
    ///   the `scanline` should be aligned to the `u16` or `u32` boundary in this
    ///   case. This can be done be instantiating it as an array of `u32`, and then
    ///   using [`bytes_of`] to cast it into an array of bytes for this function.
    /// - For formats where the pixels are floats, the above steps should be taken,
    ///   but the array should be cast to an array of `f32`s.
    ///
    /// The [`pixel()`] function does all of the above.
    ///
    /// [`pixel()`]: crate::Image::pixel
    /// [`format`]: crate::Image::format
    /// [`endianness`]: crate::Image::endianness
    /// [`bytes_of`]: bytemuck::bytes_of
    /// [`cast_slice`]: bytemuck::cast_slice
    fn scanline(&self, x: usize, y: usize, scanline: &mut [u8]) -> usize;
    /// Store a scanline into this image.
    fn set_scanline(&mut self, x: usize, y: usize, scanline: &[u8]) -> usize;

    /// Fetch the pixel at the given location.
    fn pixel(&self, x: usize, y: usize) -> Pixel {
        // read into a buffer
        let mut bytes = [0u32; MAX_BYTES_PER_PIXEL / 4];
        let index = match self.format().bpp() {
            1 => x % 8,
            4 => x % 2,
            _ => 0,
        };

        let read = self.scanline(x, y, bytemuck::bytes_of_mut(&mut bytes));
        debug_assert_eq!(
            read,
            self.format().bytes() as usize,
            "Did not read entire pixel"
        );

        if self.format().involves_float() {
            Pixel::from_float_bytes(bytemuck::cast(bytes), self.endianness(), self.format())
        } else {
            Pixel::from_bytes(
                bytes[0].to_ne_bytes(),
                index as u8,
                self.endianness(),
                self.format(),
            )
        }
    }

    /// Set the pixel at the given location.
    fn set_pixel(&mut self, x: usize, y: usize, pixel: Pixel) {
        // read one pixel's worth to a buffer, insert it, and then write it back
        // TODO: convert pixel to this format
        let mut buffer = [0u8; MAX_BYTES_PER_PIXEL];
        let len: usize = self.format().bytes().into();
        self.scanline(x, y, &mut buffer[..len]);
        pixel.insert(&mut buffer[..len]);
        self.set_scanline(x, y, &buffer[..len]);
    }
}

/// The endianness for an image.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Endianness {
    /// Little endian.
    Little,
    /// Big endian.
    Big,
}

impl Endianness {
    /// The native endianness for this system.
    pub const NATIVE: Endianness = Self::NATIVE_;

    #[cfg(target_endian = "little")]
    const NATIVE_: Endianness = Endianness::Little;
    #[cfg(target_endian = "big")]
    const NATIVE_: Endianness = Endianness::Big;

    /// Does this endianness match the native endianness?
    pub fn is_native(self) -> bool {
        self == Self::NATIVE
    }

    /// Create a u32 from bytes of this endianness.
    pub(crate) fn make_u32(self, bytes: [u8; 4]) -> u32 {
        match self {
            Endianness::Little => u32::from_le_bytes(bytes),
            Endianness::Big => u32::from_be_bytes(bytes),
        }
    }

    /// Create a u16 from bytes of this endianness.
    pub(crate) fn make_u16(self, bytes: [u8; 2]) -> u16 {
        match self {
            Endianness::Little => u16::from_le_bytes(bytes),
            Endianness::Big => u16::from_be_bytes(bytes),
        }
    }
}
