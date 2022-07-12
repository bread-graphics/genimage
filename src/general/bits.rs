// BSL 1.0 License

use crate::{Endianness, Format};
use core::cmp;

/// An image that stores all of its bits in a buffer, like a traditional
/// image.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct BitsImage<Storage: ?Sized> {
    width: usize,
    height: usize,
    format: Format,
    endianness: Endianness,
    bytes_per_scanline: usize,
    storage: Storage,
}

impl<Storage> BitsImage<Storage> {
    pub(crate) fn with_bytes_per_line(
        width: usize,
        height: usize,
        format: Format,
        endianness: Endianness,
        bytes_per_scanline: usize,
        storage: Storage,
    ) -> Self {
        BitsImage {
            width,
            height,
            format,
            endianness,
            bytes_per_scanline,
            storage,
        }
    }
}

impl<Storage: AsRef<[u8]> + AsMut<[u8]> + ?Sized> BitsImage<Storage> {
    fn storage(&self) -> &[u8] {
        self.storage.as_ref()
    }

    fn storage_mut(&mut self) -> &mut [u8] {
        self.storage.as_mut()
    }

    fn calculate_posn(&self, x: usize, y: usize, len: usize) -> (usize, usize) {
        let line_start = y.saturating_mul(self.bytes_per_scanline);
        let index_start = x.saturating_mul(self.format.bpp() as usize);
        let index_start = cmp::min(index_start, self.bytes_per_scanline);
        let index_end = index_start.saturating_add(len);
        let index_end = cmp::min(index_end, self.bytes_per_scanline);

        let begin = line_start.saturating_add(index_start);
        let end = line_start.saturating_add(index_end);

        (begin, end)
    }

    pub(crate) fn scanline(&self, x: usize, y: usize, scanline: &mut [u8]) -> usize {
        // calculate the index into the bytes we need to go
        let (begin, end) = self.calculate_posn(x, y, scanline.len());

        // memcpy the slice over
        let bytes = &self.storage()[begin..end];
        scanline.copy_from_slice(bytes);
        end.saturating_sub(begin)
    }

    pub(crate) fn set_scanline(&mut self, x: usize, y: usize, scanline: &[u8]) -> usize {
        // calculate the index into the bytes we need to go
        let (begin, end) = self.calculate_posn(x, y, scanline.len());

        // memcpy the slice over
        let bytes = &mut self.storage_mut()[begin..end];
        bytes.copy_from_slice(scanline);
        end.saturating_sub(begin)
    }

    #[inline]
    pub(crate) fn format(&self) -> Format {
        self.format
    }

    #[inline]
    pub(crate) fn endianness(&self) -> Endianness {
        self.endianness
    }

    #[inline]
    pub(crate) fn width(&self) -> usize {
        self.width
    }

    #[inline]
    pub(crate) fn height(&self) -> usize {
        self.height
    }

    #[inline]
    pub(crate) fn bytes_per_scanline(&self) -> usize {
        self.bytes_per_scanline
    }
}
