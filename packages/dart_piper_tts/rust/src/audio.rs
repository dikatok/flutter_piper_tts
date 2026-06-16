use std::sync::{
    Arc, Mutex, OnceLock, RwLock,
    atomic::{AtomicBool, AtomicU8, Ordering},
    mpsc,
};

use log::{debug, warn};
use ringbuf::{
    HeapProd, HeapRb,
    traits::{Consumer, Producer, Split},
};
use tinyaudio::{OutputDeviceParameters, run_output_device};

static AUDIO_PLAYER: OnceLock<Mutex<AudioPlayer>> = OnceLock::new();

// Clones of the Arc atomics stored outside the mutex so that
// pause / stop / resume can take effect immediately without
// waiting for play_internal to release the lock.
static AUDIO_COMMAND: OnceLock<Arc<AtomicU8>> = OnceLock::new();
static AUDIO_DRAIN: OnceLock<Arc<AtomicBool>> = OnceLock::new();

pub type DartPort = i64;
pub type CompletionCallback = unsafe extern "C" fn(port: DartPort);
static AUDIO_COMPLETION_CB: RwLock<Option<Mutex<CompletionCallback>>> = RwLock::new(None);

const SAMPLE_RATE: u32 = 22050;

pub(crate) struct AudioPlayer {
    ring_buffer: HeapProd<AudioSlot>,
    command: Arc<AtomicU8>,
    drain: Arc<AtomicBool>,
}

impl AudioPlayer {
    pub(crate) fn init(config: AudioPlayerConfig) {
        AUDIO_PLAYER.get_or_init(|| {
            let mut cb_guard = AUDIO_COMPLETION_CB.write().unwrap();
            *cb_guard = Some(Mutex::new(config.completion_callback));
            Mutex::new(AudioPlayer::init_internal(config))
        });
    }

    pub(crate) fn play(samples: &[f32]) {
        debug!("playing {} samples", samples.len());
        AUDIO_PLAYER
            .get()
            .expect("audio player not initialized")
            .lock()
            .unwrap()
            .play_internal(samples);
    }

    pub(crate) fn mark_end_of_speech(dart_port: i64) {
        debug!("marking end of speech on port {}", dart_port);
        AUDIO_PLAYER
            .get()
            .expect("audio player not initialized")
            .lock()
            .unwrap()
            .mark_end_of_speech_internal(dart_port);
    }

    // These three bypass the AUDIO_PLAYER mutex so they take effect
    // immediately, even while play_internal is spinning inside play().
    pub(crate) fn resume() {
        debug!("resuming audio playback");
        if let Some(cmd) = AUDIO_COMMAND.get() {
            cmd.store(AudioPlayerCommand::Play as u8, Ordering::SeqCst);
        }
    }

    pub(crate) fn pause() {
        debug!("pausing audio playback");
        if let Some(cmd) = AUDIO_COMMAND.get() {
            cmd.store(AudioPlayerCommand::Pause as u8, Ordering::SeqCst);
        }
    }

    pub(crate) fn stop() {
        debug!("stopping audio playback");
        // Write command first so the callback sees Pause before drain=true.
        if let Some(cmd) = AUDIO_COMMAND.get() {
            cmd.store(AudioPlayerCommand::Pause as u8, Ordering::SeqCst);
        }
        if let Some(drain) = AUDIO_DRAIN.get() {
            drain.store(true, Ordering::SeqCst);
        }
    }

    fn init_internal(config: AudioPlayerConfig) -> Self {
        let ring_buffer = HeapRb::<AudioSlot>::new(config.ring_buffer_capacity());
        let (producer, mut consumer) = ring_buffer.split();

        let command = Arc::new(AtomicU8::new(AudioPlayerCommand::Play as u8));
        let drain = Arc::new(AtomicBool::new(false));

        // Publish Arc clones before spawning threads so pause/stop/resume
        // can reach the atomics without going through the mutex.
        AUDIO_COMMAND.set(Arc::clone(&command)).ok();
        AUDIO_DRAIN.set(Arc::clone(&drain)).ok();

        let command_cb = Arc::clone(&command);
        let drain_cb = Arc::clone(&drain);

        let (completion_tx, completion_rx) = mpsc::channel::<i64>();

        let mut frac: f32 = 0.0;
        let mut held_sample: f32 = 0.0;

        let device = run_output_device(
            OutputDeviceParameters {
                channels_count: 1,
                sample_rate: config.sample_rate as usize,
                channel_sample_count: 4410,
            },
            move |data| {
                let cmd = command_cb.load(Ordering::Acquire);

                for out in data.iter_mut() {
                    if cmd != AudioPlayerCommand::Play as u8 {
                        if drain_cb.load(Ordering::Acquire) {
                            while let Some(s) = consumer.try_pop() {
                                match s {
                                    AudioSlot::Sample(_) => {}
                                    AudioSlot::Complete(port) => {
                                        let _ = completion_tx.send(port);
                                    }
                                }
                            }
                            // Reset held_sample so no stale audio leaks
                            // if playback resumes later.
                            held_sample = 0.0;
                            frac = 0.0;
                            drain_cb.store(false, Ordering::Release);
                        }
                        *out = 0.0;
                        continue;
                    }

                    frac += 1.0;

                    while frac >= 1.0 {
                        if let Some(s) = consumer.try_pop() {
                            match s {
                                AudioSlot::Sample(s) => held_sample = s,
                                AudioSlot::Complete(port) => {
                                    let _ = completion_tx.send(port);
                                }
                            }
                        }
                        frac -= 1.0;
                    }

                    *out = held_sample;
                }
            },
        )
        .expect("failed to open audio device");

        std::thread::spawn(move || {
            let _keep_alive = device;

            while let Ok(port) = completion_rx.recv() {
                if port != -1 {
                    debug!("calling completion callback on port: {}", port);
                    let cb_guard = AUDIO_COMPLETION_CB.read().unwrap();
                    if let Some(ref mutex) = *cb_guard {
                        let cb = mutex.lock().unwrap();
                        unsafe {
                            cb(port);
                        }
                    } else {
                        warn!("completion callback not initialized");
                    }
                }
            }
        });

        Self {
            ring_buffer: producer,
            command,
            drain,
        }
    }

    fn play_internal(&mut self, samples: &[f32]) {
        // If stop() set a drain, keep cmd=Pause and wait for the audio callback
        // to empty the ring buffer before pushing new samples. Cancelling drain
        // early (our previous approach) left stale samples in the buffer,
        // causing old speech to bleed through before the new one started.
        while self.drain.load(Ordering::Acquire) {
            std::thread::yield_now();
        }

        self.command
            .store(AudioPlayerCommand::Play as u8, Ordering::SeqCst);

        let mut remaining = samples;

        while !remaining.is_empty() {
            if self.command.load(Ordering::Acquire) != AudioPlayerCommand::Play as u8 {
                break;
            }

            let slots: Vec<AudioSlot> = remaining.iter().map(|&s| AudioSlot::Sample(s)).collect();
            let pushed = self.ring_buffer.push_slice(&slots);
            remaining = &remaining[pushed..];

            if !remaining.is_empty() {
                std::thread::yield_now();
            }
        }
    }

    fn mark_end_of_speech_internal(&mut self, dart_port: i64) {
        loop {
            if self
                .ring_buffer
                .try_push(AudioSlot::Complete(dart_port))
                .is_ok()
            {
                break;
            }
            std::thread::yield_now();
        }
    }
}

#[repr(u8)]
pub(crate) enum AudioPlayerCommand {
    Play = 1,
    Pause = 2,
}

pub struct AudioPlayerConfig {
    sample_rate: u32,
    buffer_duration_secs: u32,
    completion_callback: CompletionCallback,
}

impl AudioPlayerConfig {
    pub(crate) fn new(
        sample_rate: Option<u32>,
        buffer_duration_secs: Option<u32>,
        completion_callback: CompletionCallback,
    ) -> Self {
        Self {
            sample_rate: sample_rate.unwrap_or(SAMPLE_RATE),
            buffer_duration_secs: buffer_duration_secs.unwrap_or(10),
            completion_callback,
        }
    }
}

impl AudioPlayerConfig {
    pub fn ring_buffer_capacity(&self) -> usize {
        (self.buffer_duration_secs * self.sample_rate) as usize
    }
}

#[derive(Copy, Clone)]
enum AudioSlot {
    Sample(f32),
    Complete(i64),
}
