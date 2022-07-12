// BSL 1.0 License

use tinyvec::ArrayVec;

use crate::{Channel, ChannelValue};

/// An RGBA color tuple.
///
/// This is never encoded into the image directly, but is used in certain
/// logical use cases, such as solid colors.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Rgba {
    /// The red component of the color.
    pub red: u16,
    /// The green component of the color.
    pub green: u16,
    /// The blue component of the color.
    pub blue: u16,
    /// The alpha component of the color.
    pub alpha: u16,
}

impl Rgba {
    /// Get the channels for this color.
    pub(crate) fn channel_values(self) -> ArrayVec<[ChannelValue; 4]> {
        use Channel::*;

        ArrayVec::from([
            ChannelValue::new(Alpha, shift(self.alpha)),
            ChannelValue::new(Red, shift(self.red)),
            ChannelValue::new(Green, shift(self.green)),
            ChannelValue::new(Blue, shift(self.blue)),
        ])
    }
}

const fn shift(value: u16) -> u8 {
    (value >> 8) as u8
}
