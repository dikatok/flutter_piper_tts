use std::{
    fs::File,
    path::Path,
    sync::mpsc::{self, Sender},
};

use log::{debug, error};
use ort::session::builder::SessionBuilder;

use crate::{
    AUDIO_PLAYER, PHONEMIZER_SESSION,
    config::ModelConfig,
    error::{TTSError, TTSResult},
    inference::infer,
};

enum SpeechTask {
    Play {
        text: String,
        dart_port: i64,
        is_phonemes: bool,
    },
    Resume,
    Pause,
    Stop,
}

pub(crate) struct Instance {
    speech_tasks: Sender<SpeechTask>,
}

unsafe impl Send for Instance {}
unsafe impl Sync for Instance {}

impl Instance {
    pub(crate) fn new(model_path: &Path, config_path: &Path) -> TTSResult<Self> {
        let config_file = File::open(config_path)?;
        let config: ModelConfig = serde_json::from_reader(config_file)?;

        debug!("config loaded from {}", config_path.display());

        let mut ort_session = SessionBuilder::new()?.commit_from_file(model_path)?;

        debug!("ort session created from {}", model_path.display());

        let lang = config
            .language
            .code
            .clone()
            .or_else(|| Some("en".to_string()))
            .unwrap()
            .replace("_", "-");

        debug!("language: {}", lang);

        let (tx, rx) = mpsc::channel::<SpeechTask>();

        std::thread::spawn(move || {
            while let Ok(task) = rx.recv() {
                match task {
                    SpeechTask::Play {
                        text,
                        dart_port,
                        is_phonemes,
                    } => {
                        debug!("processing play (is_phonemes: {}): {}", is_phonemes, text);

                        let phonemes = if is_phonemes {
                            text
                        } else {
                            match PHONEMIZER_SESSION
                                .get()
                                .unwrap()
                                .lock()
                                .unwrap()
                                .phonemize(&lang, &text, None, None)
                            {
                                Ok(c) => c,
                                Err(e) => {
                                    error!("failed to read clauses: {}", e);
                                    continue;
                                }
                            }
                        };

                        debug!("phonemes: {}", phonemes);

                        let samples = match infer(&mut ort_session, &config, &phonemes) {
                            Ok(s) => s,
                            Err(e) => {
                                error!("inference failed: {}", e);
                                continue;
                            }
                        };

                        AUDIO_PLAYER
                            .get()
                            .expect("audio player not initialized")
                            .lock()
                            .unwrap()
                            .play(&samples);

                        AUDIO_PLAYER
                            .get()
                            .expect("audio player not initialized")
                            .lock()
                            .unwrap()
                            .mark_end_of_speech(dart_port);
                    }
                    SpeechTask::Resume => AUDIO_PLAYER
                        .get()
                        .expect("audio player not initialized")
                        .lock()
                        .unwrap()
                        .resume(),
                    SpeechTask::Pause => AUDIO_PLAYER
                        .get()
                        .expect("audio player not initialized")
                        .lock()
                        .unwrap()
                        .pause(),
                    SpeechTask::Stop => AUDIO_PLAYER
                        .get()
                        .expect("audio player not initialized")
                        .lock()
                        .unwrap()
                        .stop(),
                }
            }

            debug!("speech task worker exiting");
        });

        Ok(Instance { speech_tasks: tx })
    }

    pub(crate) fn speak(&mut self, text: &str, is_phonemes: bool, dart_port: i64) -> TTSResult<()> {
        self.speech_tasks
            .send(SpeechTask::Play {
                text: text.to_string(),
                is_phonemes,
                dart_port,
            })
            .map_err(|err| TTSError {
                message: format!("Failed to send speech task: {}", err),
            })?;

        Ok(())
    }

    pub(crate) fn pause(&self) {
        self.speech_tasks
            .send(SpeechTask::Pause)
            .expect("Failed to send speech task");
    }

    pub(crate) fn resume(&self) {
        self.speech_tasks
            .send(SpeechTask::Resume)
            .expect("Failed to send speech task");
    }

    pub(crate) fn stop(&self) {
        self.speech_tasks
            .send(SpeechTask::Stop)
            .expect("Failed to send speech task");
    }
}
