/// ByT5 tokenizer
/// Token IDs: 0=PAD, 1=EOS, 2=UNK, 3..258 = UTF-8 byte values 0..255

pub(crate) const PAD_TOKEN_ID: i64 = 0;
pub(crate) const EOS_TOKEN_ID: i64 = 1;

/// Encode "<lang>: text" into ByT5 token IDs.
pub(crate) fn encode(lang: &str, text: &str) -> Vec<i64> {
    let input = format!("<{lang}>: {text}");
    input.bytes().map(|b| b as i64 + 3).collect()
}

/// Decode ByT5 token IDs back to a UTF-8 string.
pub(crate) fn decode(token_ids: &[i64]) -> String {
    let bytes: Vec<u8> = token_ids.iter().map(|&t| (t - 3) as u8).collect();
    String::from_utf8_lossy(&bytes).into_owned()
}
