use std::{fs::File, num::NonZero, path::Path};

use espeak_ng::Translator;
use ort::session::{Session, builder::SessionBuilder};
use rodio::{MixerDeviceSink, Player, buffer::SamplesBuffer, nz};

use crate::{
    config::ModelConfig,
    error::{TTSError, TTSResult},
    inference::infer,
};

#[allow(dead_code)]
pub(crate) struct Instance {
    config: ModelConfig,
    ort_session: Session,
    sink: MixerDeviceSink, // keep the sink alive
    rodio_player: Player,
    espeak_translator: Translator,
}

impl Instance {
    pub(crate) fn new(model_path: &Path, config_path: &Path) -> TTSResult<Self> {
        let config_file = File::open(config_path)?;
        let config: ModelConfig = serde_json::from_reader(config_file)?;

        let ort_session = SessionBuilder::new()?.commit_from_file(model_path)?;

        let sink = rodio::DeviceSinkBuilder::open_default_sink().unwrap();
        let rodio_player = rodio::Player::connect_new(&sink.mixer());

        let espeak_translator = match Translator::new(
            &config.language.family,
            Some(
                sysdirs::config_dir()
                    .unwrap()
                    .join("espeak-ng-data")
                    .as_path(),
            ),
        ) {
            Ok(translator) => translator,
            Err(err) => {
                return Err(TTSError {
                    message: format!("Failed to create translator: {}", err),
                });
            }
        };

        Ok(Instance {
            config,
            sink,
            ort_session,
            rodio_player,
            espeak_translator,
        })
    }

    pub(crate) fn speak(&mut self, text: &str) -> TTSResult<()> {
        let phonemes = self
            .espeak_translator
            .text_to_ipa(text)
            .map_err(|err| TTSError {
                message: format!("Failed to convert text to phonemes: {}", err),
            })?;
        println!("Phonemes: {}", phonemes);
        let samples = infer(&mut self.ort_session, &self.config, phonemes.as_str())?;
        println!("Samples: {}", samples.len());
        self.rodio_player.append(SamplesBuffer::new(
            nz!(1),
            NonZero::new(self.config.audio.sample_rate).unwrap(),
            samples,
        ));
        println!("Samples appended");
        self.rodio_player.sleep_until_end();
        println!("Samples played");
        Ok(())
    }
}
