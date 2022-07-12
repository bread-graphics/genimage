// BSL 1.0 License

use crate::{Endianness, Format, Pixel};

/// Convert pixels of one format to another.
pub(crate) fn convert_to_format(pixel: Pixel, format: Format, endian: Endianness) -> Pixel {
    // if the formats are equal, no need to convert
    if pixel.format() == format && pixel.endianness() == endian {
        return pixel;
    }

    Pixel::collect_channels(endian, format, pixel.channel_info())
}
