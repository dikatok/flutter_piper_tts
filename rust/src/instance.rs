use std::{fs::File, path::Path};

use espeak_ng::Translator;
use log::debug;
use ort::session::{Session, builder::SessionBuilder};

use crate::{
    AUDIO_PLAYER, ESPEAK_DATA_DIR,
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
        debug!("creating new instance");

        let config_file = File::open(config_path)?;
        let config: ModelConfig = serde_json::from_reader(config_file)?;
        debug!("config fetched from {}", config_path.display());

        let ort_session = SessionBuilder::new()?.commit_from_file(model_path)?;
        debug!("ort session created from {}", model_path.display());

        let lang = config
            .language
            .family
            .clone()
            .or_else(|| Some("en".to_string()))
            .unwrap();
        debug!("espeak language: {}", lang);
        let espeak_translator = match Translator::new(
            lang.as_str(),
            Some(Path::new(ESPEAK_DATA_DIR.read().unwrap().as_str())),
        ) {
            Ok(translator) => translator,
            Err(err) => {
                return Err(TTSError {
                    message: format!("Failed to create translator: {}", err),
                });
            }
        };
        debug!("espeak translator created");

        Ok(Instance {
            config,
            ort_session,
            espeak_translator,
        })
    }

    pub(crate) fn speak(&mut self, text: &str) -> TTSResult<()> {
        debug!("speak");

        let clauses = self
            .espeak_translator
            .read_clauses(text)
            .map_err(|err| TTSError {
                message: format!("Failed to read clauses: {}", err),
            });

        let mut player = AUDIO_PLAYER.get().expect("").lock().unwrap();

        for clause in clauses? {
            debug!("clause: {}", clause.text);
            let phonemes = self
                .espeak_translator
                .text_to_ipa(&clause.text)
                .map_err(|err| TTSError {
                    message: format!("Failed to convert text to phonemes: {}", err),
                })?;
            debug!("phonemes: {}", phonemes);
            let samples = infer(&mut self.ort_session, &self.config, &phonemes.as_str())?;
            player.play(&samples);
        }

        Ok(())
    }

    pub(crate) fn pause(&mut self) {
        debug!("pause");
        AUDIO_PLAYER.get().expect("").lock().unwrap().pause();
    }

    pub(crate) fn resume(&mut self) {
        debug!("resume");
        AUDIO_PLAYER.get().expect("").lock().unwrap().resume();
    }

    pub(crate) fn stop(&mut self) {
        debug!("stop");
        AUDIO_PLAYER.get().expect("").lock().unwrap().stop();
    }
}
