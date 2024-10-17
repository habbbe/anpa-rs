use core::borrow::Borrow;
/// Common trait for types that can be safely converted to `char`.
pub trait CharLike: Copy {
    fn as_char(self) -> char;
}

macro_rules! impl_CharLike {
    ($t:ty) => {
        impl CharLike for $t {
            #[inline]
            fn as_char(self) -> char {
                *self.borrow() as char
            }
        }
    };
}

impl_CharLike!(u8);
impl_CharLike!(&u8);
impl_CharLike!(char);
impl_CharLike!(&char);