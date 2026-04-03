use crate::runtime::{ALPHA, DIGIT, PRINT, SPACE, XDIGIT};

const fn ctype_byte(byte: u8) -> u8 {
    let mut bits = 0;

    if matches!(byte, b'\t' | b'\n' | 0x0B | 0x0C | b'\r' | b' ') {
        bits |= SPACE;
    }
    if byte >= 0x20 && byte <= 0x7e {
        bits |= PRINT;
    }
    if byte >= b'0' && byte <= b'9' {
        bits |= DIGIT | XDIGIT;
    }
    if (byte >= b'A' && byte <= b'Z') || (byte >= b'a' && byte <= b'z') || byte == b'_' {
        bits |= ALPHA;
    }
    if (byte >= b'A' && byte <= b'F') || (byte >= b'a' && byte <= b'f') {
        bits |= XDIGIT;
    }

    bits
}

const fn build_ctype() -> [u8; 258] {
    let mut out = [0u8; 258];
    let mut i = 0usize;
    while i < 256 {
        out[i + 1] = ctype_byte(i as u8);
        i += 1;
    }
    out
}

pub(crate) static LUAI_CTYPE: [u8; 258] = build_ctype();

#[cfg(test)]
mod tests {
    use super::LUAI_CTYPE;

    #[test]
    fn ctype_table_matches_expected_ascii_classes() {
        assert_eq!(LUAI_CTYPE[0], 0x00);
        assert_eq!(LUAI_CTYPE[b'0' as usize + 1], 0x16);
        assert_eq!(LUAI_CTYPE[b'A' as usize + 1], 0x15);
        assert_eq!(LUAI_CTYPE[b'G' as usize + 1], 0x05);
        assert_eq!(LUAI_CTYPE[b'a' as usize + 1], 0x15);
        assert_eq!(LUAI_CTYPE[b'_' as usize + 1], 0x05);
        assert_eq!(LUAI_CTYPE[b' ' as usize + 1], 0x0c);
        assert_eq!(LUAI_CTYPE[0x7f + 1], 0x00);
        assert_eq!(LUAI_CTYPE[255 + 1], 0x00);
    }
}
