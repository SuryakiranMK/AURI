// Audio Tone & Test Stream Synthesizer
// Provides offline, zero-dependency audio verification and tone testing.

use std::f32::consts::PI;

pub struct ToneGenerator {
    frequency_hz: f32,
    sample_rate: u32,
    phase: f32,
    total_frames: usize,
    rendered_frames: usize,
}

impl ToneGenerator {
    pub fn new(frequency_hz: f32, sample_rate: u32, duration_secs: f32) -> Self {
        let total_frames = (sample_rate as f32 * duration_secs) as usize;
        Self {
            frequency_hz,
            sample_rate,
            phase: 0.0,
            total_frames,
            rendered_frames: 0,
        }
    }

    /// Generates the next chunk of stereo interleaved f32 audio samples.
    pub fn next_chunk(&mut self, max_frames: usize) -> Option<Vec<f32>> {
        if self.rendered_frames >= self.total_frames {
            return None;
        }

        let frames_to_render = max_frames.min(self.total_frames - self.rendered_frames);
        let mut buffer = Vec::with_capacity(frames_to_render * 2);

        let phase_step = 2.0 * PI * self.frequency_hz / (self.sample_rate as f32);

        for _ in 0..frames_to_render {
            // Apply a smooth attack/decay envelope to prevent clicks
            let progress = self.rendered_frames as f32 / self.total_frames as f32;
            let envelope = if progress < 0.05 {
                progress / 0.05
            } else if progress > 0.95 {
                (1.0 - progress) / 0.05
            } else {
                1.0
            };

            let sample = self.phase.sin() * 0.25 * envelope;

            // Stereo: Left and Right channels
            buffer.push(sample);
            buffer.push(sample);

            self.phase += phase_step;
            if self.phase > 2.0 * PI {
                self.phase -= 2.0 * PI;
            }
            self.rendered_frames += 1;
        }

        Some(buffer)
    }

    pub fn duration_secs(&self) -> f64 {
        self.total_frames as f64 / self.sample_rate as f64
    }

    pub fn current_time_secs(&self) -> f64 {
        self.rendered_frames as f64 / self.sample_rate as f64
    }

    pub fn seek(&mut self, seconds: f64) {
        let target_frame = (seconds * self.sample_rate as f64) as usize;
        self.rendered_frames = target_frame.min(self.total_frames);
        let phase_step = 2.0 * PI * self.frequency_hz / (self.sample_rate as f32);
        self.phase = (self.rendered_frames as f32 * phase_step) % (2.0 * PI);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tone_generator_frame_count() {
        let mut gen = ToneGenerator::new(440.0, 48000, 1.0);
        let mut total_samples = 0;
        while let Some(chunk) = gen.next_chunk(1024) {
            total_samples += chunk.len();
        }
        // 1 second * 48000 frames * 2 channels = 96000 samples
        assert_eq!(total_samples, 96000);
    }

    #[test]
    fn test_tone_generator_sample_bounds() {
        let mut gen = ToneGenerator::new(440.0, 48000, 0.5);
        while let Some(chunk) = gen.next_chunk(512) {
            for &sample in &chunk {
                assert!(sample >= -0.26 && sample <= 0.26, "Sample out of bounds: {}", sample);
            }
        }
    }

    #[test]
    fn test_tone_generator_seek() {
        let mut gen = ToneGenerator::new(440.0, 48000, 2.0);
        gen.seek(1.0);
        assert!((gen.current_time_secs() - 1.0).abs() < 0.001);
    }
}
