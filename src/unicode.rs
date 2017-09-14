use unicode_width::*;
use std::char;
use std::str;


/// The trait provides a method `chunks`.
pub trait UnicodeStrExt {
    /// Returns an iterator of its sub-strings splitted by `width`.
    fn chunks<'a>(&'a self, width: usize) -> Chunks<'a>;
}

impl UnicodeStrExt for str {
    fn chunks<'a>(&'a self, width: usize) -> Chunks<'a> {
        Chunks {
            bytes: self.as_bytes(),
            pos: 0,
            width,
        }
    }
}


pub struct Chunks<'a> {
    bytes: &'a [u8],
    pos: usize,
    width: usize,
}

impl<'a> Chunks<'a> {
    fn next_chunk(&self) -> &'a str {
        let mut width = 0;
        let mut pos = self.pos;

        while pos < self.bytes.len() {
            let (ch, len) = unsafe { next_utf8_char(&self.bytes[pos..]) };
            let w = ch.width().unwrap_or(0) as usize;

            if width + w > self.width {
                break;
            }

            width += w;
            pos += len;
        }

        let s = &self.bytes[self.pos..pos];
        unsafe { str::from_utf8_unchecked(s) }
    }
}

impl<'a> Iterator for Chunks<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == self.bytes.len() {
            return None;
        }

        let s = self.next_chunk();
        self.pos += s.len();
        Some(s)
    }
}


/// Reads a UTF-8 character from a slice of bytes,
/// and returns itself and how many bytes will be consumed.
///
/// # Safety
/// This function assumes that the slice is not empty and
/// the value is a valid UTF-8 sequence.
#[inline]
unsafe fn next_utf8_char(bytes: &[u8]) -> (char, usize) {
    debug_assert!(bytes.len() > 0);

    let (len, ch) = match *bytes.get_unchecked(0) {
        x if x < 0x80 => {
            (1, x as u32)
        },
        x if x >= 0xC2 && x < 0xE0 => {
            (2, u32::from(x & 0b0001_1111) << 6
                | u32::from(*bytes.get_unchecked(1) & 0x3F))
        },
        x if x >= 0xE0 && x < 0xF0 => {
            (3, u32::from(x & 0b0000_1111) << 12
                | u32::from(*bytes.get_unchecked(1) & 0x3F) << 6
                | u32::from(*bytes.get_unchecked(2) & 0x3F))
        }
        x if x >= 0xF0 && x < 0xF5 => {
            (4, u32::from(x & 0b0000_1111) << 18
                | u32::from(*bytes.get_unchecked(1) & 0x3F) << 12
                | u32::from(*bytes.get_unchecked(2) & 0x3F) << 6
                | u32::from(*bytes.get_unchecked(3) & 0x3F))
        }
        _ => {
            (0, 0u32)
        }
    };

    (char::from_u32_unchecked(ch), len)
}

#[cfg(test)]
mod tests {
    use super::next_utf8_char;
    use super::UnicodeStrExt;

    #[test]
    fn test_next_utf8_char() {
        assert_eq!(unsafe { next_utf8_char(b"a") }, ('a', 1));
        assert_eq!(unsafe { next_utf8_char("あ".as_bytes()) }, ('あ', 3));
        assert_eq!(unsafe { next_utf8_char("🍣".as_bytes()) }, ('🍣', 4));
    }

    #[test]
    fn test_chunks_en() {
        let i = "123456789";
        let c = ["12345", "6789"];
        let w = 5;
        let o: Vec<_> = i.chunks(w).collect();
        assert_eq!(&c[..], &o[..]);
    }

    #[test]
    fn test_chunks_cjk() {
        let i = "やきにくがたべたい…しかし😫";
        let c = ["やきにくが", "たべたい…", "しかし😫"];
        let w = 10;

        let o: Vec<_> = i.chunks(w).collect();
        assert_eq!(&c[..], &o[..]);
    }
}
