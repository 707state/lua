const ALPHA: u8 = 1 << 0;
const DIGIT: u8 = 1 << 1;
const PRINT: u8 = 1 << 2;
const SPACE: u8 = 1 << 3;
const XDIGIT: u8 = 1 << 4;

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

#[unsafe(no_mangle)]
pub static luai_ctype_: [u8; 258] = build_ctype();

#[cfg(test)]
mod tests {
    use super::luai_ctype_;

    #[test]
    fn ctype_table_matches_expected_ascii_classes() {
        assert_eq!(luai_ctype_[0], 0x00);
        assert_eq!(luai_ctype_[b'0' as usize + 1], 0x16);
        assert_eq!(luai_ctype_[b'A' as usize + 1], 0x15);
        assert_eq!(luai_ctype_[b'G' as usize + 1], 0x05);
        assert_eq!(luai_ctype_[b'a' as usize + 1], 0x15);
        assert_eq!(luai_ctype_[b'_' as usize + 1], 0x05);
        assert_eq!(luai_ctype_[b' ' as usize + 1], 0x0c);
        assert_eq!(luai_ctype_[0x7f + 1], 0x00);
        assert_eq!(luai_ctype_[255 + 1], 0x00);
    }
}
