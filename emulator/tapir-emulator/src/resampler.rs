use std::{
    collections::VecDeque,
    ops::Deref,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use crossbeam::queue::SegQueue;
use sdl2::audio::AudioCallback;

pub trait Resampler {
    fn set_input_frequency(&mut self, frequency: f64);

    fn len(&self) -> usize;
    fn write_sample(&mut self, sample: f64);
    fn read_sample(&mut self) -> Option<f64>;
}

pub struct CubicResampler {
    buffered_samples: VecDeque<f64>,
    history: [f64; 4],
    fraction: f64,
    input_frequency: f64,
    output_frequency: f64,
    frequency_ratio: f64,
}

#[derive(Default)]
pub struct AudioQueue {
    queue: SegQueue<i16>,
    sample_rate: AtomicUsize,
}

#[derive(Default, Clone)]
pub struct SharedAudioQueue(Arc<AudioQueue>);

impl Deref for SharedAudioQueue {
    type Target = AudioQueue;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AudioQueue {
    pub fn set_sample_rate(&self, sample_rate: usize) {
        self.sample_rate.store(sample_rate, Ordering::SeqCst);
    }

    pub fn sample_rate(&self) -> usize {
        self.sample_rate.load(Ordering::SeqCst)
    }

    pub fn push(&self, sample: [i16; 2]) {
        for sample in sample {
            self.queue.push(sample);
        }
    }

    pub fn samples(&self) -> usize {
        self.queue.len() / 2
    }
}

impl AudioCallback for SharedAudioQueue {
    type Channel = i16;

    fn callback(&mut self, samples_output: &mut [Self::Channel]) {
        for sample_output in samples_output.iter_mut() {
            let Some(sample) = self.queue.pop() else {
                return;
            };

            *sample_output = sample;
        }
    }
}

pub fn calculate_dynamic_rate_ratio(
    maximum_buffer_size: usize,
    current_buffer_fill: usize,
    maximum_drift: f64,
) -> f64 {
    let maximum_buffer_size = maximum_buffer_size as f64;
    let current_buffer_fill = current_buffer_fill as f64;

    1. - maximum_drift + 2. * current_buffer_fill / maximum_buffer_size * maximum_drift
}

impl CubicResampler {
    pub fn new(output_frequency: f64, input_frequency: f64) -> Self {
        Self {
            buffered_samples: Default::default(),
            history: Default::default(),
            fraction: 0.,
            input_frequency,
            output_frequency,
            frequency_ratio: input_frequency / output_frequency,
        }
    }
}

impl Resampler for CubicResampler {
    fn set_input_frequency(&mut self, frequency: f64) {
        self.input_frequency = frequency;
        self.frequency_ratio = self.input_frequency / self.output_frequency;
    }

    fn len(&self) -> usize {
        self.buffered_samples.len()
    }

    fn write_sample(&mut self, sample: f64) {
        self.history[0] = self.history[1];
        self.history[1] = self.history[2];
        self.history[2] = self.history[3];
        self.history[3] = sample;

        let [s0, s1, s2, s3] = self.history;

        while self.fraction <= 1. {
            let t = self.fraction;

            let a = s3 - s2 - s0 + s1;
            let b = s0 - s1 - a;
            let c = s2 - s0;
            let d = s1;

            self.buffered_samples
                .push_back(a * t * t * t + b * t * t + c * t + d);

            self.fraction += self.frequency_ratio;
        }

        self.fraction -= 1.;
    }

    fn read_sample(&mut self) -> Option<f64> {
        self.buffered_samples.pop_front()
    }
}
