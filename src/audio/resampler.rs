// High-efficiency, zero-allocation linear resampler
// Converts source sample rate (e.g. 44.1 kHz, 96 kHz) to WASAPI output rate (e.g. 48 kHz).

pub struct LinearResampler {
    pub src_rate: u32,
    pub dst_rate: u32,
    pub channels: usize,
    phase: f64,
    last_frame: Vec<f32>,
}

impl LinearResampler {
    pub fn new(src_rate: u32, dst_rate: u32, channels: usize) -> Self {
        Self {
            src_rate,
            dst_rate,
            channels: channels.max(1),
            phase: 0.0,
            last_frame: vec![0.0; channels.max(1)],
        }
    }

    /// Resamples an interleaved input slice `[s0_ch0, s0_ch1, s1_ch0, s1_ch1, ...]`
    /// and appends resampled frames into `output`.
    pub fn resample(&mut self, input: &[f32], output: &mut Vec<f32>) {
        if self.src_rate == self.dst_rate {
            output.extend_from_slice(input);
            if input.len() >= self.channels {
                let start = input.len() - self.channels;
                self.last_frame.copy_from_slice(&input[start..]);
            }
            return;
        }

        let num_input_frames = input.len() / self.channels;
        if num_input_frames == 0 {
            return;
        }

        let ratio = self.src_rate as f64 / self.dst_rate as f64;

        while self.phase < num_input_frames as f64 {
            let idx0 = self.phase.floor() as usize;
            let frac = (self.phase - idx0 as f64) as f32;
            let idx1 = idx0 + 1;

            for ch in 0..self.channels {
                let s0 = input[idx0 * self.channels + ch];
                let s1 = if idx1 < num_input_frames {
                    input[idx1 * self.channels + ch]
                } else {
                    s0
                };
                let sample = s0 + frac * (s1 - s0);
                output.push(sample);
            }

            self.phase += ratio;
        }

        self.phase -= num_input_frames as f64;
        let start = input.len() - self.channels;
        self.last_frame.copy_from_slice(&input[start..]);
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.last_frame.fill(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resampler_passthrough() {
        let mut resampler = LinearResampler::new(48000, 48000, 2);
        let input = vec![0.1, -0.1, 0.5, -0.5];
        let mut output = Vec::new();
        resampler.resample(&input, &mut output);
        assert_eq!(output, input);
    }

    #[test]
    fn test_resampler_upsample_ratio() {
        let mut resampler = LinearResampler::new(44100, 48000, 2);
        // 441 frames stereo = 882 samples
        let input = vec![0.2; 882];
        let mut output = Vec::new();
        resampler.resample(&input, &mut output);
        // 441 * (48000 / 44100) = 480 frames = 960 stereo samples (approx)
        let out_frames = output.len() / 2;
        assert!(out_frames >= 479 && out_frames <= 481, "Expected ~480 frames, got {}", out_frames);
    }

    #[test]
    fn test_resampler_downsample_ratio() {
        let mut resampler = LinearResampler::new(96000, 48000, 2);
        let input = vec![0.3; 960 * 2];
        let mut output = Vec::new();
        resampler.resample(&input, &mut output);
        let out_frames = output.len() / 2;
        assert_eq!(out_frames, 480);
    }
}
