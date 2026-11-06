// AURI High-Fidelity Audio Resampler
// Implements high-order cubic Hermite interpolation for crisp frequency reproduction without linear aliasing

#[derive(Debug, Clone)]
pub struct CubicResampler {
    from_rate: u32,
    to_rate: u32,
    channels: usize,
    ratio: f64,
}

impl CubicResampler {
    pub fn new(from_rate: u32, to_rate: u32, channels: usize) -> Self {
        let from = from_rate.max(1);
        let to = to_rate.max(1);
        let ratio = from as f64 / to as f64;
        Self {
            from_rate: from,
            to_rate: to,
            channels: channels.max(1),
            ratio,
        }
    }

    #[inline]
    pub fn is_passthrough(&self) -> bool {
        self.from_rate == self.to_rate
    }

    /// Resample interleaved multi-channel PCM buffer from `from_rate` to `to_rate`.
    pub fn resample(&self, input: &[f32], output: &mut Vec<f32>) {
        if self.is_passthrough() {
            output.extend_from_slice(input);
            return;
        }

        let in_frames = input.len() / self.channels;
        if in_frames < 4 {
            // Not enough points for cubic interpolation, pass as-is
            output.extend_from_slice(input);
            return;
        }

        let out_frames = (in_frames as f64 / self.ratio).floor() as usize;
        output.reserve(out_frames * self.channels);

        let ch = self.channels;

        for out_idx in 0..out_frames {
            let in_pos = out_idx as f64 * self.ratio;
            let i = in_pos.floor() as usize;
            let frac = (in_pos - i as f64) as f32;

            // Clamped 4-point neighborhood indices for cubic Hermite
            let i0 = if i > 0 { i - 1 } else { 0 };
            let i1 = i.min(in_frames - 1);
            let i2 = (i + 1).min(in_frames - 1);
            let i3 = (i + 2).min(in_frames - 1);

            for c in 0..ch {
                let y0 = input[i0 * ch + c];
                let y1 = input[i1 * ch + c];
                let y2 = input[i2 * ch + c];
                let y3 = input[i3 * ch + c];

                // Catmull-Rom cubic interpolation formula
                let a = -0.5 * y0 + 1.5 * y1 - 1.5 * y2 + 0.5 * y3;
                let b = y0 - 2.5 * y1 + 2.0 * y2 - 0.5 * y3;
                let c_coeff = -0.5 * y0 + 0.5 * y2;
                let d = y1;

                let sample = ((a * frac + b) * frac + c_coeff) * frac + d;
                output.push(sample.clamp(-1.0, 1.0));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passthrough() {
        let resampler = CubicResampler::new(48000, 48000, 2);
        assert!(resampler.is_passthrough());
        let input = vec![0.1, -0.2, 0.3, -0.4];
        let mut output = Vec::new();
        resampler.resample(&input, &mut output);
        assert_eq!(input, output);
    }

    #[test]
    fn test_upsample_ratio() {
        let resampler = CubicResampler::new(44100, 48000, 1);
        assert!(!resampler.is_passthrough());
        let input: Vec<f32> = (0..441).map(|i| (i as f32 * 0.05).sin()).collect();
        let mut output = Vec::new();
        resampler.resample(&input, &mut output);
        // Output frames should be approximately 441 * 48000 / 44100 = 480
        assert!(output.len() >= 475 && output.len() <= 485);
    }
}

