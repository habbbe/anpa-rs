/// Common trait for types that can be safely converted to `char`.
pub trait CharLike: Copy {
    fn as_char(self) -> char;
}

impl<C: CharLike> CharLike for &C {
    #[inline(always)]
    fn as_char(self) -> char {
        (*self).as_char()
    }
}

macro_rules! impl_CharLike {
    ($t:ty) => {
        impl CharLike for $t {
            #[inline(always)]
            fn as_char(self) -> char {
                self as char
            }
        }
    };
}

impl_CharLike!(u8);
impl_CharLike!(char);