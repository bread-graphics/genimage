// BSL 1.0 License

use core::{
    borrow::{Borrow, BorrowMut},
    ops::{Deref, DerefMut},
};

/// A wrapper around any `AsRef<[u32]>` that casts it into an `AsRef<[u8]>`.
///
/// Useful for usage as a buffer for images.
#[repr(transparent)]
pub struct U32Buf<T: ?Sized>(pub T);

impl<T> From<T> for U32Buf<T> {
    fn from(item: T) -> Self {
        U32Buf(item)
    }
}

impl<T: ?Sized> Deref for U32Buf<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: ?Sized> DerefMut for U32Buf<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: AsRef<[u32]> + ?Sized> AsRef<[u8]> for U32Buf<T> {
    fn as_ref(&self) -> &[u8] {
        bytemuck::cast_slice(self.0.as_ref())
    }
}

impl<T: AsMut<[u32]> + ?Sized> AsMut<[u8]> for U32Buf<T> {
    fn as_mut(&mut self) -> &mut [u8] {
        bytemuck::cast_slice_mut(self.0.as_mut())
    }
}

impl<T: Borrow<[u32]> + ?Sized> Borrow<[u8]> for U32Buf<T> {
    fn borrow(&self) -> &[u8] {
        bytemuck::cast_slice(self.0.borrow())
    }
}

impl<T: BorrowMut<[u32]> + ?Sized> BorrowMut<[u8]> for U32Buf<T> {
    fn borrow_mut(&mut self) -> &mut [u8] {
        bytemuck::cast_slice_mut(self.0.borrow_mut())
    }
}
