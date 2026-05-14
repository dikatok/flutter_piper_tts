use std::path::Path;

use log::debug;
use ndarray::ArrayViewD;
use ort::{
    session::{Session, builder::SessionBuilder},
    value::Value,
};
use unaccent::unaccent;
use unicode_segmentation::UnicodeSegmentation;

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

    pub fn phonemize(
        &mut self,
        lang: &str,
        text: &str,
        chunk_size: Option<usize>,
        max_len: Option<usize>,
    ) -> Result<String, TTSError> {
        let cleaned_text = unaccent(text)
            .chars()
            .filter(|c| c.is_alphanumeric() || ".!? ".contains(*c))
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        let chunks = self.chunk_text(cleaned_text.as_str(), chunk_size.unwrap_or(100));

        let mut phonemes: Vec<String> = Vec::new();

        for chunk in chunks {
            debug!("processing chunk: {}", chunk.text_for_model);
            let input_ids = encode(lang, chunk.text_for_model.as_str());
            let seq_len = input_ids.len();
            let input_ids_arr = Value::from_array(
                ndarray::Array2::from_shape_vec((1, seq_len), input_ids).unwrap(),
            )?;
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

                if next_token_id == EOS_TOKEN_ID {
                    break;
                }

                generated.push(next_token_id);
                decoder_ids.push(next_token_id);
            }

            let decoded_ipa = decode(&generated);
            let final_phonemes = format!("{}{}", decoded_ipa, chunk.original_punctuation);
            phonemes.push(final_phonemes);
        }

        Ok(phonemes.join(" "))
    }

    fn chunk_text(&self, text: &str, max_chars: usize) -> Vec<PhonemeChunk> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();

        for sentence in text.unicode_sentences() {
            if current_chunk.len() + sentence.len() > max_chars && !current_chunk.is_empty() {
                chunks.push(current_chunk.trim().to_string());
                current_chunk = String::new();
            }
            current_chunk.push_str(sentence);
            current_chunk.push(' ');
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk.trim().to_string());
        }

        chunks
            .iter()
            .map(|c| PhonemeChunk::new(c.as_str()))
            .collect()
    }
}

struct PhonemeChunk {
    text_for_model: String,       // Canonicalized (ends in '.')
    original_punctuation: String, // The actual mark (!, ?, ,)
}

impl PhonemeChunk {
    fn new(text: &str) -> Self {
        let trimmed = text.trim();

        if let Some(last) = trimmed.chars().last()
            && last.is_ascii_punctuation()
        {
            return Self {
                text_for_model: trimmed[..trimmed.len() - last.len_utf8()].to_string(),
                original_punctuation: last.to_string(),
            };
        }

        Self {
            text_for_model: trimmed.to_string(),
            original_punctuation: "".to_string(),
        }
    }
}
