//! 4-character paste ids, drawn from an alphabet with no lookalikes.

pub const LEN: usize = 4;
const ALPHABET: &[u8] = b"abcdefghjkmnpqrstuvwxyz23456789";

pub fn random_id() -> String {
    let mut out = String::with_capacity(LEN);
    for _ in 0..LEN {
        let i = (rand::random::<u32>() % ALPHABET.len() as u32) as usize;
        out.push(ALPHABET[i] as char);
    }
    out
}

pub fn is_id_char(c: char) -> bool {
    c.is_ascii() && ALPHABET.contains(&(c as u8))
}

pub fn is_valid_prefix(s: &str) -> bool {
    !s.is_empty() && s.chars().all(is_id_char)
}
