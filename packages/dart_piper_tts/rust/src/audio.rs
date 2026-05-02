use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU8, Ordering},
};

use ringbuf::{
    HeapProd, HeapRb,
    traits::{Consumer, Producer, Split},
};
use tinyaudio::{OutputDevice, OutputDeviceParameters, run_output_device};

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

pub(crate) struct AudioPlayer {
    ring_buffer: HeapProd<f32>,
    command: Arc<AtomicU8>,
    _device: OutputDevice, // keep this alive
    drain: Arc<AtomicBool>,
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new(AudioPlayerConfig::default())
    }
}

impl AudioPlayer {
    pub(crate) fn new(config: AudioPlayerConfig) -> Self {
        let ring_buffer = HeapRb::<f32>::new(config.ring_buffer_capacity());
        let (producer, mut consumer) = ring_buffer.split();

        let command = Arc::new(AtomicU8::new(AudioPlayerCommand::Play as u8));
        let drain = Arc::new(AtomicBool::new(false));

        let command_cb = Arc::clone(&command);
        let drain_cb = Arc::clone(&drain);

        // Fractional position accumulator lives inside the callback closure
        let mut frac: f32 = 0.0;
        // Small local buffer for speed < 1.0 (we need to repeat samples)
        let mut held_sample: f32 = 0.0;

        let _device = run_output_device(
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
                            while consumer.try_pop().is_some() {}
                            drain_cb.store(false, Ordering::Relaxed);
                        }
                        *out = 0.0;
                        continue;
                    }

                    frac += 1.0;

                    // Pop as many samples as frac has accumulated
                    while frac >= 1.0 {
                        if let Some(s) = consumer.try_pop() {
                            held_sample = s;
                        }
                        // If ring is empty: output silence (underrun)
                        frac -= 1.0;
                    }

                    *out = held_sample;
                }
            },
        )
        .expect("failed to open audio device");

        println!("Audio device opened");

        Self {
            ring_buffer: producer,
            command,
            _device,
            drain,
        }
    }

    pub fn play(&mut self, samples: &[f32]) {
        self.command
            .store(AudioPlayerCommand::Play as u8, Ordering::Relaxed);
        let mut remaining = samples;
        while !remaining.is_empty() {
            let pushed = self.ring_buffer.push_slice(remaining);
            remaining = &remaining[pushed..];
            if !remaining.is_empty() {
                std::thread::yield_now();
            }
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
