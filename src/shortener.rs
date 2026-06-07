const BASE62: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

pub fn encode_base62(mut n: u64) -> String {
    if n == 0 {
        return String::from("0");
    }
    let mut buf = Vec::with_capacity(11);
    while n > 0 {
        buf.push(BASE62[(n % 62) as usize]);
        n /= 62;
    }
    buf.reverse();
    String::from_utf8(buf).unwrap()
}

pub fn decode_base62(s: &str) -> u64 {
    let mut n: u64 = 0;
    for &c in s.as_bytes() {
        let idx = match c {
            b'0'..=b'9' => c - b'0',
            b'A'..=b'Z' => c - b'A' + 10,
            b'a'..=b'z' => c - b'a' + 36,
            _ => return 0,
        };
        n = n.wrapping_mul(62).wrapping_add(idx as u64);
    }
    n
}

pub fn generate_short_code(id: u64) -> String {
    let mut scrambled = id;
    scrambled = (scrambled & 0x5555555555555555) << 1
        | (scrambled & 0xAAAAAAAAAAAAAAAA) >> 1;
    scrambled = (scrambled & 0x3333333333333333) << 2
        | (scrambled & 0xCCCCCCCCCCCCCCCC) >> 2;
    scrambled = (scrambled & 0x0F0F0F0F0F0F0F0F) << 4
        | (scrambled & 0xF0F0F0F0F0F0F0F0) >> 4;
    encode_base62(scrambled)
}

pub fn is_valid_short_code(code: &str) -> bool {
    if code.is_empty() || code.len() > 16 {
        return false;
    }
    code.bytes().all(|c| matches!(c, b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip() {
        let ids = [0, 1, 61, 62, 12345, 999999999, 1234567890123];
        for &id in &ids {
            let code = encode_base62(id);
            let decoded = decode_base62(&code);
            assert_eq!(decoded, id, "round-trip failed for {}", id);
        }
    }

    #[test]
    fn test_unique_codes() {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for i in 1..=10000 {
            let code = generate_short_code(i);
            assert!(seen.insert(code), "duplicate code at {}", i);
        }
    }

    #[test]
    fn test_valid_short_code() {
        assert!(is_valid_short_code("abc123"));
        assert!(is_valid_short_code("ABCxyz789"));
        assert!(is_valid_short_code("0abcDEF123xyz"));
        assert!(!is_valid_short_code(""));
        assert!(!is_valid_short_code("abc-def"));
        assert!(!is_valid_short_code("abc def"));
    }
}
