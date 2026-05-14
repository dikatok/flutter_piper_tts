use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU8, Ordering},
    mpsc,
};

use log::{debug, warn};
use ringbuf::{
    HeapProd, HeapRb,
    traits::{Consumer, Producer, Split},
};
use tinyaudio::{OutputDeviceParameters, run_output_device};

use crate::COMPLETION_CB;

const SAMPLE_RATE: u32 = 22050;

#[repr(u8)]
pub(crate) enum AudioPlayerCommand {
    Play = 1,
    Pause = 2,
}

pub struct AudioPlayerConfig {
    pub sample_rate: u32,
    pub buffer_duration_secs: u32,
}

impl Default for AudioPlayerConfig {
    fn default() -> Self {
        Self {
            sample_rate: SAMPLE_RATE,
            buffer_duration_secs: 10,
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
    Complete(i64), // dart port, -1 = no notification needed
}

pub(crate) struct AudioPlayer {
    ring_buffer: HeapProd<AudioSlot>,
    command: Arc<AtomicU8>,
    drain: Arc<AtomicBool>,
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new(AudioPlayerConfig::default())
    }
}

impl AudioPlayer {
    pub(crate) fn new(config: AudioPlayerConfig) -> Self {
        let ring_buffer = HeapRb::<AudioSlot>::new(config.ring_buffer_capacity());
        let (producer, mut consumer) = ring_buffer.split();

        let command = Arc::new(AtomicU8::new(AudioPlayerCommand::Play as u8));
        let drain = Arc::new(AtomicBool::new(false));

        let command_cb = Arc::clone(&command);
        let drain_cb = Arc::clone(&drain);

        let (completion_tx, completion_rx) = mpsc::channel::<i64>();

        // Fractional position accumulator lives inside the callback closure
        let mut frac: f32 = 0.0;
        // Small local buffer for speed < 1.0 (we need to repeat samples)
        let mut held_sample: f32 = 0.0;

        let device = run_output_device(
            OutputDeviceParameters {
                channels_count: 1,
                sample_rate: config.sample_rate as usize,
                channel_sample_count: 4410, // ~100ms chunks
            },
            move |data| {
                let cmd = command_cb.load(Ordering::Relaxed);

                for out in data.iter_mut() {
                    if cmd != AudioPlayerCommand::Play as u8 {
                        if drain_cb.load(Ordering::Relaxed) {
                            while let Some(s) = consumer.try_pop() {
                                match s {
                                    AudioSlot::Sample(s) => *out = s,
                                    AudioSlot::Complete(port) => {
                                        let _ = completion_tx.send(port);
                                    }
                                }
                            }
                            drain_cb.store(false, Ordering::Relaxed);
                        }
                        *out = 0.0;
                        continue;
                    }

                    frac += 1.0;

                    // Pop as many samples as frac has accumulated
                    while frac >= 1.0 {
                        if let Some(s) = consumer.try_pop() {
                            match s {
                                AudioSlot::Sample(s) => held_sample = s,
                                AudioSlot::Complete(port) => {
                                    let _ = completion_tx.send(port);
                                }
                            }
                        }
                        // If ring is empty: output silence (underrun)
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
                    let cb_guard = COMPLETION_CB.read().unwrap();
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

    pub fn play(&mut self, samples: &[f32]) {
        self.command
            .store(AudioPlayerCommand::Play as u8, Ordering::Relaxed);

        let mut remaining = samples;

        while !remaining.is_empty() {
            let slots: Vec<AudioSlot> = remaining.iter().map(|&s| AudioSlot::Sample(s)).collect();

            let pushed = self.ring_buffer.push_slice(&slots);

            remaining = &remaining[pushed..];

            if !remaining.is_empty() {
                std::thread::yield_now();
            }
        }
    }

    pub fn mark_end_of_speech(&mut self, dart_port: i64) {
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

    pub fn resume(&self) {
        self.command
            .store(AudioPlayerCommand::Play as u8, Ordering::Relaxed);
    }

    pub fn pause(&self) {
        self.command
            .store(AudioPlayerCommand::Pause as u8, Ordering::Relaxed);
    }

    pub fn stop(&mut self) {
        self.command
            .store(AudioPlayerCommand::Pause as u8, Ordering::Relaxed);
        self.drain.store(true, Ordering::Relaxed);
    }
}
