use mgba::MCore;

use crate::resampler::{calculate_dynamic_rate_ratio, CubicResampler, Resampler, SharedAudioQueue};

pub struct Emulator {
    mgba: MCore,
    resamplers: [CubicResampler; 2],
    audio_sample_rate: f64,
}

impl Emulator {
    pub fn new(rom: Vec<u8>, sample_rate: f64) -> Result<Self, anyhow::Error> {
        let mut mgba_core =
            mgba::MCore::new().ok_or_else(|| anyhow::anyhow!("Failed to initialise mgba core"))?;
        mgba_core.load_rom(mgba::MemoryBacked::new(rom));
        mgba_core.set_audio_frequency(sample_rate);

        Ok(Self {
            mgba: mgba_core,
            resamplers: [
                CubicResampler::new(sample_rate, sample_rate),
                CubicResampler::new(sample_rate, sample_rate),
            ],
            audio_sample_rate: sample_rate,
        })
    }

    pub fn frame(&mut self, keys: u32, frame_time: f64, sample_queue: &SharedAudioQueue) {
        self.mgba.set_keys(keys);
        self.mgba.frame();

        let mut audio_buffer = Vec::new();

        self.mgba.read_audio(&mut audio_buffer);

        let queue_length = sample_queue.samples();

        let desired_buffer_size = (self.audio_sample_rate * 3. * frame_time) as usize;
        let desired_buffer_size = desired_buffer_size.max(10); // make sure it's not zero!
        let ratio = calculate_dynamic_rate_ratio(desired_buffer_size, queue_length, 0.005);

        let rate = self.audio_sample_rate * ratio;

        for resampler in self.resamplers.iter_mut() {
            resampler.set_input_frequency(rate);
        }

        for sample in audio_buffer.chunks_exact(2) {
            let sample_l = sample[0];
            let sample_r = sample[1];
            self.resamplers[0].write_sample(sample_l as f64);
            self.resamplers[1].write_sample(sample_r as f64);
        }

        while let (Some(a), Some(b)) = (
            self.resamplers[0].read_sample(),
            self.resamplers[1].read_sample(),
        ) {
            sample_queue.push([a as i16, b as i16]);
        }
    }

    pub fn copy_video_buffer_to_texture(&mut self, texture: &mut [u8]) {
        let mgba_buffer = self.mgba.video_buffer();
        for (i, data) in mgba_buffer.iter().enumerate() {
            texture[(i * 4)..((i + 1) * 4)].copy_from_slice(&data.to_ne_bytes());
        }
    }
}
