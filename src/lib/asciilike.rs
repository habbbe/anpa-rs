/// Common trait for types that can be treated like ASCII characters.
pub trait AsciiLike: Copy + PartialEq {
    type PlainType: Copy + PartialEq;
    const MINUS: Self::PlainType;
    const PERIOD: Self::PlainType;

    /// Attempt converting this item to a digit.
    fn as_digit(self) -> Option<u8>;

    /// Check if two items are equal.
    fn equal(self, other: Self::PlainType) -> bool;
}

impl AsciiLike for char {
    const MINUS: Self::PlainType = '-';
    const PERIOD: Self::PlainType = '.';
    type PlainType = char;

    #[inline(always)]
    fn as_digit(self) -> Option<u8> {
        self.to_digit(10).map(|d| d as u8)
    }

    #[inline(always)]
    fn equal(self, other: Self::PlainType) -> bool {
        self == other
    }
}

impl AsciiLike for &char {
    const MINUS: Self::PlainType = char::MINUS;
    const PERIOD: Self::PlainType = char::PERIOD;
    type PlainType = char;

    #[inline(always)]
    fn as_digit(self) -> Option<u8> {
        self.to_digit(10).map(|d| d as u8)
    }

    #[inline(always)]
    fn equal(self, other: Self::PlainType) -> bool {
        *self == other
    }
}

impl AsciiLike for u8 {
    const MINUS: Self::PlainType = b'-';
    const PERIOD: Self::PlainType = b'.';
    type PlainType = u8;

    #[inline(always)]
    fn as_digit(self) -> Option<u8> {
        self.is_ascii_digit().then_some(self - b'0')
    }

    #[inline(always)]
    fn equal(self, other: Self::PlainType) -> bool {
        self == other
    }
}

impl AsciiLike for &u8 {
    const MINUS: Self::PlainType = u8::MINUS;
    const PERIOD: Self::PlainType = u8::PERIOD;
    type PlainType = u8;

    #[inline(always)]
    fn as_digit(self) -> Option<u8> {
        (*self).as_digit()
    }

    #[inline(always)]
    fn equal(self, other: Self::PlainType) -> bool {
        (*self).equal(other)
    }
}