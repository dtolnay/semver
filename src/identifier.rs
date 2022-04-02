// This module implements Identifier, a short-optimized string allowed to
// contain only the ASCII characters hyphen, dot, 0-9, A-Z, a-z.

use core::str;

pub(crate) enum Identifier {
    Empty,
    // Invariant: The array MUST contain valid UTF-8
    Inline([u8; 8], u8),
    Heap(Box<str>),
}

impl Identifier {
    pub(crate) const fn empty() -> Self {
        Identifier::Empty
    }

    pub(crate) fn new(string: &str) -> Self {
        let len = string.len();
        match len {
            0 => Identifier::Empty,
            1..=8 => {
                let mut bytes = [0u8; 8];
                bytes[0..len].copy_from_slice(string.as_bytes());
                Identifier::Inline(bytes, len as u8)
            }
            _ => {
                let boxed = string.to_owned().into();
                Identifier::Heap(boxed)
            }
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        match self {
            Identifier::Empty => true,
            _ => false,
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        match self {
            Identifier::Empty => "",
            Identifier::Inline(arr, len) => {
                // SAFETY: This type must always have valid UTF-8
                unsafe { str::from_utf8_unchecked(&arr[0..*len as usize]) }
            }
            Identifier::Heap(boxed) => boxed,
        }
    }
}

impl Clone for Identifier {
    fn clone(&self) -> Self {
        match self {
            Identifier::Empty => Identifier::Empty,
            Identifier::Inline(arr, len) => Identifier::Inline(*arr, *len),
            Identifier::Heap(boxed) => Identifier::Heap(boxed.clone()),
        }
    }
}

impl PartialEq for Identifier {
    fn eq(&self, rhs: &Self) -> bool {
        self.as_str() == rhs.as_str()
    }
}
