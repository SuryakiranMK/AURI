// AURI Loudness Normalization & Soft Peak Limiter
// Implements YouTube Music target loudness (-7.0 LUFS) and peak limiter

/// Loudness normalization target matching YouTube Music desktop broadcast standard (-14.0 LUFS)
#[allow(dead_code)]
pub const TARGET_LUFS: f64 = -14.0;

/// Maximum volume boost in dB allowed for quiet masters to preserve dynamic headroom
pub const MAX_BOOST_DB: f64 = 3.0;

/// Minimum attenuation in dB for excessively loud tracks
pub const MIN_CUT_DB: f64 = -24.0;

/// Calculate required gain adjustment in dB from YouTube Music's metadata `loudnessDb`.
///
/// YouTube Music delivers `loudnessDb` relative to -14.0 LUFS.
/// To normalize to TARGET_LUFS (-14.0 LUFS), the required gain is `-raw`.
/// For example, a modern loud master with loudnessDb = +7.0 dB is attenuated by -7.0 dB.
pub fn calculate_loudness_gain_db(loudness_db: Option<f64>) -> Option<f64> {
    let gain = match loudness_db {
        Some(raw) => (-raw).clamp(MIN_CUT_DB, MAX_BOOST_DB),
        None => -4.5, // Intelligent fallback for tracks without loudness metadata
    };
    if gain.abs() >= 0.05 {
        Some(gain)
    } else {
        None
    }
}

/// Convert dB to linear amplitude multiplier
#[inline]
pub fn db_to_linear(gain_db: f64) -> f32 {
    (10.0_f64.powf(gain_db / 20.0)) as f32
}

/// A zero-allocation, soft-knee peak limiter.
///
/// When lifting quiet tracks, audio peaks can occasionally exceed full scale (1.0).
/// This soft limiter transparently compresses peaks above `threshold` (0.95)
/// towards 1.0 without introducing digital hard-clipping distortion.
#[derive(Debug, Clone)]
pub struct SoftLimiter {
    threshold: f32,
}

impl Default for SoftLimiter {
    fn default() -> Self {
        Self { threshold: 0.95 }
    }
}

impl SoftLimiter {
    #[allow(dead_code)]
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold: threshold.clamp(0.5, 0.99),
        }
    }

    /// Process a single PCM sample through the limiter
    #[inline]
    pub fn process_sample(&self, sample: f32) -> f32 {
        let abs = sample.abs();
        if abs <= self.threshold {
            sample
        } else {
            // Soft-knee compression above threshold
            let excess = abs - self.threshold;
            let headroom = 1.0 - self.threshold;
            let compressed = self.threshold + headroom * (excess / (excess + headroom));
            if sample >= 0.0 {
                compressed.min(1.0)
            } else {
                (-compressed).max(-1.0)
            }
        }
    }

    /// Process an interleaved slice of PCM audio samples in-place
    pub fn process_buffer(&self, buffer: &mut [f32], gain_multiplier: f32) {
        if (gain_multiplier - 1.0).abs() < 0.001 {
            // Unity gain - only limit if needed
            for s in buffer.iter_mut() {
                if s.abs() > self.threshold {
                    *s = self.process_sample(*s);
                }
            }
        } else {
            // Apply gain and limit
            for s in buffer.iter_mut() {
                let amplified = *s * gain_multiplier;
                *s = self.process_sample(amplified);
            }
        }
    }
}

/// Standard 10-band ISO octave center frequencies in Hz
pub const EQ_FREQUENCIES: [f32; 10] = [
    31.0, 63.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0,
];

/// Human-readable frequency band labels for UI
pub const EQ_BAND_LABELS: [&str; 10] = [
    "31Hz", "63Hz", "125Hz", "250Hz", "500Hz", "1kHz", "2kHz", "4kHz", "8kHz", "16kHz",
];

/// Presets for the 10-band equalizer
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EqPreset {
    Flat,
    FpsFootsteps,
    BassPunch,
    VocalClarity,
    Cinematic,
    Custom,
}

impl Default for EqPreset {
    fn default() -> Self {
        Self::Flat
    }
}

impl EqPreset {
    pub const ALL: [EqPreset; 6] = [
        EqPreset::Flat,
        EqPreset::FpsFootsteps,
        EqPreset::BassPunch,
        EqPreset::VocalClarity,
        EqPreset::Cinematic,
        EqPreset::Custom,
    ];

    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            EqPreset::Flat => "Audiophile Flat",
            EqPreset::FpsFootsteps => "FPS Footsteps & Cues",
            EqPreset::BassPunch => "Deep Bass Punch",
            EqPreset::VocalClarity => "Vocal Clarity",
            EqPreset::Cinematic => "Cinematic Rumble",
            EqPreset::Custom => "Custom Profile",
        }
    }

    pub fn gains(&self) -> [f32; 10] {
        match self {
            EqPreset::Flat => [0.0; 10],
            // Boost 2kHz-4kHz for footsteps and weapon reloads, cut sub-rumble
            EqPreset::FpsFootsteps => [-2.5, -3.0, -1.5, 0.0, 1.0, 2.5, 4.0, 5.0, 3.0, 1.5],
            // Bass thump & punch with gentle treble rise
            EqPreset::BassPunch => [5.5, 6.0, 4.0, 2.0, 0.5, -0.5, 0.0, 1.0, 2.0, 2.0],
            // Vocal presence curve
            EqPreset::VocalClarity => [-2.0, -2.5, -1.0, 1.0, 2.5, 4.5, 3.5, 2.0, 1.0, 0.5],
            // V-shaped dynamic movie rumble
            EqPreset::Cinematic => [6.0, 4.5, 2.0, 0.0, -1.0, 0.5, 2.0, 3.5, 4.5, 4.0],
            EqPreset::Custom => [0.0; 10],
        }
    }
}

/// Second-order IIR peaking filter coefficients (Robert Bristow-Johnson Audio EQ Cookbook)
#[derive(Debug, Clone, Copy, Default)]
pub struct BiquadCoeffs {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    identity: bool,
}

impl BiquadCoeffs {
    pub fn peaking(sample_rate: f32, f0: f32, gain_db: f32, q: f32) -> Self {
        if gain_db.abs() < 0.05 {
            return Self {
                b0: 1.0,
                b1: 0.0,
                b2: 0.0,
                a1: 0.0,
                a2: 0.0,
                identity: true,
            };
        }

        let nyquist = sample_rate * 0.495;
        let f0 = f0.clamp(10.0, nyquist);

        let a = 10.0_f32.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f32::consts::PI * f0 / sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q);

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_w0;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha / a;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            identity: false,
        }
    }
}

/// Direct Form II Transposed filter state
#[derive(Debug, Clone, Copy, Default)]
pub struct BiquadState {
    s1: f32,
    s2: f32,
}

impl BiquadState {
    #[inline]
    pub fn process(&mut self, coeffs: &BiquadCoeffs, x: f32) -> f32 {
        if coeffs.identity {
            return x;
        }
        let y = coeffs.b0 * x + self.s1;
        self.s1 = coeffs.b1 * x - coeffs.a1 * y + self.s2;
        self.s2 = coeffs.b2 * x - coeffs.a2 * y;
        // Avoid denormals
        if self.s1.abs() < 1e-15 {
            self.s1 = 0.0;
        }
        if self.s2.abs() < 1e-15 {
            self.s2 = 0.0;
        }
        y
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.s1 = 0.0;
        self.s2 = 0.0;
    }
}

/// 10-Band Graphic Equalizer with stereo channel state
#[derive(Debug, Clone)]
pub struct Equalizer {
    coeffs: [BiquadCoeffs; 10],
    states: [[BiquadState; 2]; 10], // [band][channel: 0=L, 1=R]
    gains: [f32; 10],
    sample_rate: u32,
    active: bool,
}

impl Default for Equalizer {
    fn default() -> Self {
        Self::new(48_000)
    }
}

impl Equalizer {
    pub fn new(sample_rate: u32) -> Self {
        let mut eq = Self {
            coeffs: [BiquadCoeffs::default(); 10],
            states: [[BiquadState::default(); 2]; 10],
            gains: [0.0; 10],
            sample_rate,
            active: false,
        };
        eq.recalculate();
        eq
    }

    pub fn set_sample_rate(&mut self, rate: u32) {
        if self.sample_rate != rate && rate > 0 {
            self.sample_rate = rate;
            self.recalculate();
        }
    }

    pub fn set_gains(&mut self, gains: [f32; 10]) {
        let mut changed = false;
        let mut active = false;
        for (i, &g) in gains.iter().enumerate() {
            let clamped = g.clamp(-12.0, 12.0);
            if (self.gains[i] - clamped).abs() >= 0.05 {
                self.gains[i] = clamped;
                changed = true;
            }
            if clamped.abs() >= 0.05 {
                active = true;
            }
        }
        self.active = active;
        if changed {
            self.recalculate();
        }
    }

    fn recalculate(&mut self) {
        let rate = self.sample_rate as f32;
        // Q = 1.414 for 1-octave bandwidth
        let q = 1.414_f32;
        for i in 0..10 {
            self.coeffs[i] = BiquadCoeffs::peaking(rate, EQ_FREQUENCIES[i], self.gains[i], q);
        }
    }

    #[allow(dead_code)]
    pub fn reset_states(&mut self) {
        for band in &mut self.states {
            band[0].reset();
            band[1].reset();
        }
    }

    /// Process interleaved audio buffer in-place
    pub fn process_interleaved(&mut self, buffer: &mut [f32], channels: usize, sample_rate: u32) {
        self.set_sample_rate(sample_rate);
        if !self.active || channels == 0 {
            return;
        }

        if channels == 1 {
            // Mono
            for sample in buffer.iter_mut() {
                let mut s = *sample;
                for b in 0..10 {
                    s = self.states[b][0].process(&self.coeffs[b], s);
                }
                *sample = s;
            }
        } else {
            // Stereo or multi-channel (processes first 2 channels)
            for frame in buffer.chunks_mut(channels) {
                let mut left = frame[0];
                for b in 0..10 {
                    left = self.states[b][0].process(&self.coeffs[b], left);
                }
                frame[0] = left;

                if frame.len() > 1 {
                    let mut right = frame[1];
                    for b in 0..10 {
                        right = self.states[b][1].process(&self.coeffs[b], right);
                    }
                    frame[1] = right;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loudness_gain_calculation() {
        // Track with loudnessDb 7.77 -> -7.77 dB gain to normalize to -14 LUFS
        let gain = calculate_loudness_gain_db(Some(7.77)).unwrap();
        assert!((gain - (-7.77)).abs() < 0.01);

        // Track already at -14 LUFS (0.0 dB) -> returns None (no modification needed)
        assert_eq!(calculate_loudness_gain_db(Some(0.0)), None);

        // Quiet master (loudnessDb -1.5 dB) -> +1.5 dB boost
        let boost = calculate_loudness_gain_db(Some(-1.5)).unwrap();
        assert!((boost - 1.5).abs() < 0.01);

        // Extremely quiet track capped at MAX_BOOST_DB (+3.0 dB)
        let capped = calculate_loudness_gain_db(Some(-10.0)).unwrap();
        assert_eq!(capped, MAX_BOOST_DB);

        // No metadata -> fallback -4.5 dB
        assert_eq!(calculate_loudness_gain_db(None), Some(-4.5));
    }

    #[test]
    fn test_soft_limiter_bounds() {
        let limiter = SoftLimiter::default();
        // Values under threshold (0.95) are untouched
        assert_eq!(limiter.process_sample(0.5), 0.5);
        assert_eq!(limiter.process_sample(-0.8), -0.8);

        // Values exceeding 1.0 are soft-compressed below 1.0
        let limited = limiter.process_sample(1.5);
        assert!(limited < 1.0 && limited >= 0.95);

        let limited_neg = limiter.process_sample(-2.0);
        assert!(limited_neg > -1.0 && limited_neg <= -0.95);
    }

    #[test]
    fn test_equalizer_flat_passthrough() {
        let mut eq = Equalizer::default();
        let mut buffer = vec![0.5, -0.5, 0.25, -0.25];
        let original = buffer.clone();
        eq.process_interleaved(&mut buffer, 2, 48_000);
        // With all gains at 0.0, signal is identical
        assert_eq!(buffer, original);
    }

    #[test]
    fn test_equalizer_boost_and_presets() {
        let mut eq = Equalizer::default();
        let preset = EqPreset::FpsFootsteps;
        eq.set_gains(preset.gains());
        assert!(eq.active);

        // Process a 1kHz tone or noise
        let mut buffer = vec![0.1_f32; 100];
        eq.process_interleaved(&mut buffer, 2, 48_000);
        // Audio should be non-zero and finite
        for &s in &buffer {
            assert!(s.is_finite());
        }
    }
}

