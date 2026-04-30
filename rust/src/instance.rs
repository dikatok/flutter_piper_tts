use std::{fs::File, path::Path};

use espeak_ng::Translator;
use log::info;
use ort::session::{Session, builder::SessionBuilder};
use tinyaudio::{OutputDeviceParameters, run_output_device};

use crate::{
    ESPEAK_DATA_DIR,
    config::ModelConfig,
    error::{TTSError, TTSResult},
    inference::infer,
};

pub(crate) struct Instance {
    config: ModelConfig,
    ort_session: Session,
    espeak_translator: Translator,
}

impl Instance {
    pub(crate) fn new(model_path: &Path, config_path: &Path) -> TTSResult<Self> {
        info!("Creating new instance");
        let config_file = File::open(config_path)?;
        let config: ModelConfig = serde_json::from_reader(config_file)?;
        info!("Config created");

        let ort_session = SessionBuilder::new()?.commit_from_file(model_path)?;
        info!("Session created");

        let espeak_translator = match Translator::new(
            &config.language.family,
            Some(Path::new(ESPEAK_DATA_DIR.read().unwrap().as_str())),
        ) {
            Ok(translator) => translator,
            Err(err) => {
                return Err(TTSError {
                    message: format!("Failed to create translator: {}", err),
                });
            }
        };
        info!("Translator created");

        Ok(Instance {
            config,
            ort_session,
            espeak_translator,
        })
    }

    pub(crate) fn speak(&mut self, text: &str) -> TTSResult<()> {
        let clauses = self
            .espeak_translator
            .read_clauses(text)
            .map_err(|err| TTSError {
                message: format!("Failed to read clauses: {}", err),
            });

        let binding = clauses.unwrap();
        let clause = binding.get(0).unwrap();

        // for clause in clauses? {
        println!("Clause: {:?}", clause.clause_type);
        let phonemes = self
            .espeak_translator
            .text_to_ipa(&clause.text)
            .map_err(|err| TTSError {
                message: format!("Failed to convert text to phonemes: {}", err),
            })?;
        println!("Phonemes: {}", phonemes);
        let samples = infer(&mut self.ort_session, &self.config, phonemes.as_str())?;
        println!("Samples: {}", samples.len());
        // }
        let mut pos = 0;

        let gain = 2.0_f32; // try 2.0–4.0
        let amplified: Vec<f32> = samples
            .iter()
            .map(|s| (s * gain).clamp(-1.0, 1.0))
            .collect();

        let _device = run_output_device(
            OutputDeviceParameters {
                channels_count: 1,                                   // Piper outputs mono
                sample_rate: self.config.audio.sample_rate as usize, // Piper's default sample rate
                channel_sample_count: 4410,
            },
            move |data| {
                for out in data.iter_mut() {
                    *out = if pos < amplified.len() {
                        let s = amplified[pos];
                        pos += 1;
                        s
                    } else {
                        0.0
                    };
                }
            },
        )
        .unwrap();
        std::thread::sleep(std::time::Duration::from_secs_f64(3.0));
        println!("Samples appended");
        println!("Samples played");
        Ok(())
    }
}
