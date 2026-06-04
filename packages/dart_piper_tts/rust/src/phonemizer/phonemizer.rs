use std::str::FromStr;

use unaccent::unaccent;

use crate::{
    error::TTSError,
    phonemizer::{
        cmudict::CmudictPhonemizer, epitran::EpitranPhonemizer, neural::NeuralPhonemizer,
    },
};

pub(crate) struct Phonemizer {}

impl Phonemizer {
    pub(crate) fn init() {
        CmudictPhonemizer::init();
        EpitranPhonemizer::init();
        NeuralPhonemizer::init();
    }

    pub(crate) fn phonemize(
        lang: &str,
        text: &str,
        strategy: PhonemizationStrategy,
    ) -> Result<String, TTSError> {
        let cleaned_text = unaccent(text)
            .chars()
            .filter(|c| c.is_alphanumeric() || ".!? ".contains(*c))
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        match strategy {
            PhonemizationStrategy::NeuralOnly => {
                NeuralPhonemizer::phonemize(lang, cleaned_text.as_str(), None, None)
            }
            _ => {
                // 1. Allocate the cleaned string and give it a long life
                let intermediate_string = unaccent(text)
                    .chars()
                    .filter(|c| c.is_alphanumeric() || ".!? ".contains(*c))
                    .collect::<String>();

                // 2. Now you can safely split it into a Vec<&str>
                let cleaned_text = intermediate_string.split_whitespace().collect::<Vec<_>>();
                let mut result: Vec<String> = Vec::new();

                if lang.to_lowercase()[..2] == *"en" {
                    for word in cleaned_text {
                        if let Some(lookup) = CmudictPhonemizer::lookup(word) {
                            result.push(lookup);
                        } else {
                            result.push(NeuralPhonemizer::phonemize(lang, word, None, None)?);
                        }
                    }
                } else {
                    for word in cleaned_text {
                        if let Some(lookup) = EpitranPhonemizer::lookup(lang, word) {
                            result.push(lookup);
                        } else {
                            result.push(NeuralPhonemizer::phonemize(lang, word, None, None)?);
                        }
                    }
                }

                if matches!(strategy, PhonemizationStrategy::DictionaryWithOmitUnknown) {
                    Ok(result.join(" "))
                } else {
                    Ok(result.join(" "))
                }
            }
        }
    }
}

pub enum PhonemizationStrategy {
    NeuralOnly,
    DictionaryWithNeuralFallback,
    DictionaryWithOmitUnknown,
}

impl FromStr for PhonemizationStrategy {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "neural" => Ok(PhonemizationStrategy::NeuralOnly),
            "dict_neural" => Ok(PhonemizationStrategy::DictionaryWithNeuralFallback),
            "dict_omit" => Ok(PhonemizationStrategy::DictionaryWithOmitUnknown),
            _ => Ok(PhonemizationStrategy::DictionaryWithNeuralFallback),
        }
    }
}
