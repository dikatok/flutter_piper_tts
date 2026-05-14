use std::path::Path;

use ndarray::ArrayViewD;
use ort::{
    session::{Session, builder::SessionBuilder},
    value::Value,
};

use crate::{
    error::TTSError,
    tokenizer::{EOS_TOKEN_ID, PAD_TOKEN_ID, decode, encode},
};

pub(crate) struct Phonemizer {
    session: Session,
}

impl Phonemizer {
    pub fn load(model_path: &Path) -> Result<Self, TTSError> {
        let session = SessionBuilder::new()?.commit_from_file(model_path)?;
        Ok(Self { session })
    }

    /// Convert text to IPA phonemes.
    ///
    /// `lang`    — language code: "en", "de", "ja", "zh", "fr", etc.
    /// `text`    — input text in that language
    /// `max_len` — max output tokens (128 is enough for a sentence)
    pub fn phonemize(
        &mut self,
        lang: &str,
        text: &str,
        max_len: Option<i64>,
    ) -> Result<String, TTSError> {
        let input_ids = encode(lang, text);
        let seq_len = input_ids.len();
        let input_ids_arr =
            Value::from_array(ndarray::Array2::from_shape_vec((1, seq_len), input_ids).unwrap())?;
        let attention_mask_arr = Value::from_array(ndarray::Array2::<i64>::ones((1, seq_len)))?;

        let mut decoder_ids: Vec<i64> = vec![PAD_TOKEN_ID];
        let mut generated: Vec<i64> = Vec::new();

        for _ in 0..max_len.unwrap_or(512).clamp(1, 512) {
            let decoder_ids_arr = Value::from_array(ndarray::Array2::from_shape_vec(
                (1, decoder_ids.len()),
                decoder_ids.clone(),
            )?)?;

            let outputs = self.session.run(ort::inputs![
                "input_ids"         => input_ids_arr.clone(),
                "attention_mask"    => attention_mask_arr.clone(),
                "decoder_input_ids" => decoder_ids_arr
            ])?;

            let (shape, data) = outputs[0].try_extract_tensor::<f32>()?;

            let dims: Vec<usize> = shape.iter().map(|&d| d as usize).collect();

            let logits_view = ArrayViewD::from_shape(dims, data)?;

            let shape = logits_view.shape();
            let seq_len = shape[1];

            let next_token_id = logits_view
                .slice(ndarray::s![0, seq_len - 1, ..])
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i as i64)
                .ok_or_else(|| TTSError::from("No next token".to_string()))?;

            generated.push(next_token_id);

            if next_token_id == EOS_TOKEN_ID {
                break;
            }

            decoder_ids.push(next_token_id);
        }

        Ok(decode(&generated))
    }
}
