// Minimized version of https://github.com/mariuszs/friendly_id/blob/master/src/base62.rs
// The crate depends on `failure`, which is unmaintained. As we only need to generate some random IDs,
// we can make do by reimplementing it.
pub fn friendly_id() -> String {
    const BASE: u128 = 62;
    const ALPHABET: [u8; BASE as usize] = [
        b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L',
        b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h',
        b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z',
    ];

    let mut num: u128 = rand::random();

    let mut bytes = Vec::new();
    while num > 0 {
        bytes.push(ALPHABET[(num % BASE) as usize]);
        num /= BASE;
    }
    bytes.reverse();

    String::from_utf8(bytes).unwrap()
}
