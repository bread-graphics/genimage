// BSL 1.0 License

use crate::{
    assert_exact_size::AssertExactSize, format::ChannelInfo, Channel, Endianness, Format, Rgba,
};
use core::{cmp, fmt, iter::FusedIterator};
use ordered_float::{NotNan, OrderedFloat};
use tinyvec::ArrayVec;

mod convert_format;

/// A pixel in an image.
///
/// This contains the data of the pixel, as well as information regarding
/// how that pixel should be interpreted, including its format and endianness.
///
/// `Pixels` can be created using the `Pixel::new` function, but are mostly
/// created through `Image::pixel()`.
#[derive(Copy, Clone)]
pub struct Pixel {
    /// The pixel's data.
    value: Value,
    /// The format of the pixel.
    format: Format,
    /// The endianness of this pixel.
    ///
    /// The values in `value` are assumed to be in this endianness.
    /// It is stored here mostly for usage in converting pixels between
    /// endiannesses.
    endianness: Endianness,
}

impl fmt::Debug for Pixel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct AsHex(u32);

        impl fmt::Debug for AsHex {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:08X}", self.0)
            }
        }

        f.debug_tuple("Pixel")
            .field(&AsHex(self.raw_u32()))
            .finish()
    }
}

impl cmp::PartialEq for Pixel {
    fn eq(&self, other: &Self) -> bool {
        if let (
            Value::NonFloat {
                data: data1,
                index: index1,
            },
            Value::NonFloat {
                data: data2,
                index: index2,
            },
        ) = (self.value, other.value)
        {
            return data1 == data2 && index1 == index2;
        }

        self.components_float()
            .map(OrderedFloat)
            .eq(other.components_float().map(OrderedFloat))
    }
}

impl cmp::Eq for Pixel {}

impl cmp::PartialOrd for Pixel {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::Ord for Pixel {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.components_float()
            .map(OrderedFloat)
            .cmp(other.components_float().map(OrderedFloat))
    }
}

impl core::hash::Hash for Pixel {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        for flt in self.components_float() {
            state.write_u32(flt.to_bits());
        }
    }
}

#[derive(Copy, Clone)]
enum Value {
    NonFloat {
        /// The pixel's data.
        ///
        /// It is implied that the data is stored in the correct
        /// endianness.
        data: u32,
        /// For sub-byte channels, the index into the first byte of the pixel.
        index: u8,
    },
    Float {
        /// Float data.
        ///
        /// It is implied to be in the correct endianness. The format
        /// field defines how many are valid.
        data: [f32; 4],
    },
}

impl Pixel {
    /// Create a new pixel from raw bytes, endianness and format.
    ///
    /// `index` is used for sub-byte formats to determine where in the
    /// first byte the pixel is. It can normally be zero for other
    /// formats.
    pub(crate) fn from_bytes(
        bytes: [u8; 4],
        index: u8,
        endian: Endianness,
        format: Format,
    ) -> Self {
        debug_assert!(!format.involves_float());

        // depending on the quantum, make a new value
        let data = match format.bytes() {
            1 => bytes[0] as u32,
            2 => endian.make_u16([bytes[0], bytes[1]]) as u32,
            3..=4 => endian.make_u32(bytes),
            bytes => panic!("has {} bytes, expected 1..=4", bytes),
        };

        Self {
            format,
            value: Value::NonFloat { data, index },
            endianness: endian,
        }
    }

    /// Create a new pixel from the raw bytes for a float.
    pub(crate) fn from_float_bytes(bytes: [u8; 16], endian: Endianness, format: Format) -> Self {
        debug_assert!(format.involves_float());

        let data: [[u8; 4]; 4] = bytemuck::cast(bytes);
        let data = crate::array::map(data, |arr| f32::from_bits(endian.make_u32(arr)));

        Self {
            format,
            value: Value::Float { data },
            endianness: endian,
        }
    }

    /// Create a new pixel from the raw bytes, endianness, format and,
    /// if applicable, index into the bytes that the pixel exists at.
    pub fn with_index(bytes: &[u8], index: u8, endian: Endianness, format: Format) -> Self {
        if format.involves_float() {
            // create a float
            let mut buffer = [0u8; 16];
            let cnt = format.bytes() as usize;
            buffer[..cnt].copy_from_slice(&bytes[..cnt]);

            Self::from_float_bytes(buffer, endian, format)
        } else {
            // create a raw
            let mut buffer = [0u8; 4];
            let cnt = format.bytes() as usize;
            buffer[..cnt].copy_from_slice(&bytes[..cnt]);

            Self::from_bytes(buffer, index, endian, format)
        }
    }

    /// Create a new pixel from a format, endianness and iterator over
    /// color values.
    ///
    /// This allows a pixel to be created from a collection of color
    /// channels.
    pub fn collect_channels(
        endianness: Endianness,
        format: Format,
        channels: impl IntoIterator<Item = ChannelValue>,
    ) -> Self {
        // there will be at most 4 channels
        let our_channels: ArrayVec<[ChannelInfo; 4]> = format.channels().collect();
        let non_native_endian = !endianness.is_native();

        if format.involves_float() {
            // we're dealing with floats here
            let mut data = [0f32; 4];
            channels.into_iter().for_each(|channel_value| {
                if let Some(posn) = our_channels
                    .iter()
                    .position(|channel_info| channel_value.channel_type == channel_info.channel)
                {
                    let mut val = channel_value.float_value();
                    if non_native_endian {
                        val = f32::from_bits(val.to_bits().swap_bytes());
                    }
                    data[posn] = val;
                }
            });

            Self {
                format,
                endianness,
                value: Value::Float { data },
            }
        } else {
            // we're dealing with raw values here
            let mut data = 0u32;
            channels.into_iter().for_each(|channel_value| {
                if let Some(channel_info) = our_channels
                    .iter()
                    .find(|channel_info| channel_info.channel == channel_value.channel_type)
                {
                    let val = channel_value.value() as u32;
                    data |= (val & LOW_BIT_MASKS[channel_info.bits as usize])
                        << (channel_info.shift as u32);
                }
            });

            if non_native_endian {
                data = data.swap_bytes();
            }

            Self {
                format,
                endianness,
                value: Value::NonFloat { data, index: 0 },
            }
        }
    }

    /// Create a new pixel from an RGBA color.
    pub fn from_rgba(rgba: Rgba, format: Format, endian: Endianness) -> Self {
        Self::collect_channels(endian, format, rgba.channel_values())
    }

    /// Create a new pixel from raw bytes, endianness and format.
    pub fn new(bytes: &[u8], endianness: Endianness, format: Format) -> Self {
        Self::with_index(bytes, 0, endianness, format)
    }

    /// The format for this pixel.
    pub const fn format(self) -> Format {
        self.format
    }

    /// The endianness for the pixel.
    pub const fn endianness(self) -> Endianness {
        self.endianness
    }

    /// Get the components of this channel as floating point values.
    pub(crate) fn components_float(
        self,
    ) -> impl ExactSizeIterator<Item = f32> + FusedIterator + DoubleEndedIterator {
        // get an array of floats
        let floats = match self.value {
            Value::Float { data } => {
                ArrayVec::from_array_len(data, self.format.bpp() as usize / 32)
            }
            Value::NonFloat { data, index } => {
                // manual channel conversion
                iter_channels(data, index, self.format)
                    .map(|x| {
                        let x: f32 = x as f32;
                        x / (core::u8::MAX as f32)
                    })
                    .collect()
            }
        };

        AssertExactSize(floats.into_iter())
    }

    /// Create the raw `u32` that could be used to represent this pixel.
    ///
    /// Although this is a basic arithmetic operation for raw pixels, for
    /// pixels involving floats it will try to compute the pixel from scratch.
    pub fn raw_u32(self) -> u32 {
        match self.value {
            Value::NonFloat { data, index } => data << (index as u32),
            Value::Float { .. } => {
                // manually construct it
                let mut data = 0u32;
                self.components_float().for_each(|x| {
                    data <<= 8;
                    let component = (x * (core::u8::MAX as f32)) as u32;
                    data |= component;
                });
                data
            }
        }
    }

    /// Get channel information for this pixel.
    pub fn channel_info(
        self,
    ) -> impl ExactSizeIterator<Item = ChannelValue> + DoubleEndedIterator + FusedIterator {
        let values: ArrayVec<[ChannelValue; 4]> = match self.value {
            Value::Float { .. } => {
                // iterate over channels and calculate the values
                self.components_float()
                    .zip(self.format.color_type().channels())
                    .map(|(x, channel)| ChannelValue::new_with_float(channel, x))
                    .collect()
            }
            Value::NonFloat { data, index } => iter_channels(data, index, self.format)
                .zip(self.format.color_type().channels())
                .map(|(x, channel)| ChannelValue::new(channel, x))
                .collect(),
        };

        // for some reason, ArrayVecIterator doesn't implement ExactSizeIterator
        // despite having a fixed consistent size
        AssertExactSize(values.into_iter())
    }

    /// Convert this `Pixel` to the same value but in a new format.
    ///
    /// When converting from a higher-resolution format to a lower
    /// resolution format, information may be lost.
    pub fn into_new_format(self, endian: Endianness, format: Format) -> Self {
        convert_format::convert_to_format(self, format, endian)
    }

    /// Insert this `Pixel` into the corresponding bytes.
    ///
    /// Assumes that the bytes and this pixel are of the same format.
    pub(crate) fn insert(self, bytes: &mut [u8]) {
        if let Value::NonFloat { data, index } = self.value {
            // if the format involves sub-bytes, we need to use bit
            // masking to mutate the bytes
            if self.format().subbyte() {
                let mask = (LOW_BIT_MASKS[self.format().bpp() as usize] as u8) << index;
                let data = data as u8 & mask;
                bytes[0] = (bytes[0] & !mask) | data;
                return;
            }
        }

        // otherwise, it's a pretty simple byte-wise copy
        let data_bytes = match self.value {
            Value::NonFloat { ref data, .. } => bytemuck::bytes_of(data),
            Value::Float { ref data } => bytemuck::bytes_of(data),
        };

        let cnt = self.format().bytes() as usize;
        bytes[..cnt].copy_from_slice(&data_bytes[..cnt]);
    }

    /// Fill a row of bytes with this pixel.
    ///
    /// Returns the number of bytes written.
    pub(crate) fn fill_row(self, bytes: &mut [u8]) -> usize {
        match self.format().bpp() {
            1 => {
                // only one bit per pixel
                let raw = self.raw_u32();
                if raw == 0 {
                    bytes.iter_mut().for_each(|x| *x = 0);
                } else {
                    debug_assert_eq!(raw, 1);
                    bytes.iter_mut().for_each(|x| *x = 0xFF);
                }

                bytes.len()
            }
            4 => {
                // it's a nibble
                let raw_u8 = self.raw_u32() as u8;
                let byte = (raw_u8 << 4) | raw_u8;
                bytes.iter_mut().for_each(|x| *x = byte);

                bytes.len()
            }
            8 => {
                // it's a byte
                let byte = self.raw_u32() as u8;
                bytes.iter_mut().for_each(|x| *x = byte);

                bytes.len()
            }
            16 => {
                // it's a word
                let word = self.raw_u32() as u16;
                let word_bytes = word.to_ne_bytes();
                bytes
                    .chunks_exact_mut(2)
                    .map(|chunk| {
                        chunk.copy_from_slice(&word_bytes);
                    })
                    .count()
                    * 2
            }
            32 => {
                // it's a double word
                let word = self.raw_u32() as u32;
                let word_bytes = word.to_ne_bytes();
                bytes
                    .chunks_exact_mut(4)
                    .map(|chunk| {
                        chunk.copy_from_slice(&word_bytes);
                    })
                    .count()
                    * 4
            }
            bpp => {
                // just call insert multiple times
                let bcount: usize = (bpp / 8).into();
                bytes
                    .chunks_exact_mut(bcount)
                    .map(|chunk| {
                        self.insert(chunk);
                    })
                    .count()
                    * bcount
            }
        }
    }
}

fn iter_channels(
    mut data: u32,
    index: u8,
    format: Format,
) -> impl ExactSizeIterator<Item = u8> + FusedIterator + DoubleEndedIterator {
    // shift it over by index
    data >>= index as u32;

    // iterate over channels
    format.channels().map(move |channel_info| {
        // shift and mask data
        let channel =
            (data >> (channel_info.shift as u32)) & LOW_BIT_MASKS[channel_info.bits as usize];
        channel as u8
    })
}

/// The value of a channel combined with the type of the channel.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ChannelValue {
    /// The type of the channel.
    channel_type: Channel,
    /// The value of the channel.
    value: u8,
    /// The floating point value of this channel.
    float_value: Option<NotNan<f32>>,
}

impl ChannelValue {
    /// Create a new `ChannelValue` from a channel and a `u8` value.
    pub const fn new(channel_type: Channel, value: u8) -> Self {
        Self {
            channel_type,
            value,
            float_value: None,
        }
    }

    /// Create a new `ChannelValue` from a channel and a `f32` value.
    ///
    /// `value` is expected to be between `0.0` and `1.0` inclusive. All
    /// other values will lead to logic errors.
    pub fn new_with_float(channel_type: Channel, value: f32) -> Self {
        Self {
            channel_type,
            value: (value * (core::u8::MAX as f32)) as u8,
            float_value: NotNan::new(value).ok(),
        }
    }

    /// The type of the channel.
    pub const fn channel_type(self) -> Channel {
        self.channel_type
    }

    /// The value of the channel.
    pub const fn value(self) -> u8 {
        self.value
    }

    /// The floating point value of the channel.
    pub fn float_value(self) -> f32 {
        self.float_value.map_or_else(
            || self.value as f32 / (core::u8::MAX as f32),
            |x| x.into_inner(),
        )
    }
}

/// The mask for getting the `n` lowest bits of a `u32`.
const LOW_BIT_MASKS: [u32; 33] = {
    let mut low_bit_masks = [0u32; 33];
    let mut i = 0;
    let mut current = 0u32;

    while i < 33 {
        low_bit_masks[i as usize] = current;
        current = (current << 1) | 1;
        i += 1;
    }

    low_bit_masks
};

#[cfg(all(feature = "alloc", test))]
mod tests {
    use alloc::vec::Vec;
    use core::hash::{Hash, Hasher};

    use super::*;

    /// Ready-bake pixels for use in testing.
    fn test_pixels() -> Vec<Pixel> {
        alloc::vec![
            Pixel::from_bytes([255, 255, 255, 255], 0, Endianness::NATIVE, Format::ARGB32),
            Pixel::from_float_bytes(
                bytemuck::cast([1.0f32, 1.0, 1.0, 1.0]),
                Endianness::NATIVE,
                Format::ARGB_F32,
            ),
            Pixel::from_bytes([255, 255, 255, 0], 0, Endianness::NATIVE, Format::ARGB32),
            Pixel::from_bytes([255, 255, 255, 0], 0, Endianness::NATIVE, Format::RGB24),
        ]
    }

    #[test]
    fn partial_eq_implies_hash_eq() {
        let hash_pixel = |pixel: Pixel| {
            let mut hasher = ahash::AHasher::new_with_keys(69, 420);
            pixel.hash(&mut hasher);
            hasher.finish()
        };

        let make_comparison = |px1: Pixel, px2: Pixel| {
            if hash_pixel(px1) == hash_pixel(px2) {
                assert_eq!(px1, px2);
            } else {
                assert_ne!(px1, px2);
            }
        };

        let iter1 = test_pixels().into_iter();
        let iter2 = test_pixels().into_iter();

        for (left, right) in itertools::iproduct!(iter1, iter2) {
            make_comparison(left, right);
        }
    }
}
