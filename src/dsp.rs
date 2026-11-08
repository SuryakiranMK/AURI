// AURI Loudness Normalization & Soft Peak Limiter
// Implements YouTube Music target loudness (-7.0 LUFS) and peak limiter

/// Loudness normalization target matching YouTube Music desktop (-7.0 LUFS)
pub const TARGET_LUFS: f64 = -7.0;

/// Maximum volume boost in dB allowed for quiet masters to preserve dynamic headroom
pub const MAX_BOOST_DB: f64 = 3.0;

/// Minimum attenuation in dB for excessively loud tracks
pub const MIN_CUT_DB: f64 = -24.0;

/// Calculate required gain adjustment in dB from YouTube Music's metadata `loudnessDb`.
///
/// YouTube Music delivers `loudnessDb` relative to -14 LUFS.
/// To reach TARGET_LUFS (-7.0 LUFS):
/// gain = TARGET_LUFS - (loudness_db - 14.0)
pub fn calculate_loudness_gain_db(loudness_db: Option<f64>) -> Option<f64> {
    let raw = loudness_db?;
    // Compute required gain
    let gain = (TARGET_LUFS - (raw - 14.0)).clamp(MIN_CUT_DB, MAX_BOOST_DB);
    // Ignore imperceptible changes below 0.05 dB
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loudness_gain_calculation() {
        // "As It Was" with loudnessDb 7.77 -> -6.23 LUFS (0.77 dB over -7 target) -> -0.77 dB gain
        let gain = calculate_loudness_gain_db(Some(7.77)).unwrap();
        assert!((gain - (-0.77)).abs() < 0.01);

        // Track exactly on target (7.0 dB) -> returns None (no modification needed)
        assert_eq!(calculate_loudness_gain_db(Some(7.0)), None);

        // Quiet master (loudnessDb 6.35 -> -7.65 LUFS) -> +0.65 dB gain
        let boost = calculate_loudness_gain_db(Some(6.35)).unwrap();
        assert!((boost - 0.65).abs() < 0.01);

        // Extremely quiet track capped at MAX_BOOST_DB (+3.0 dB)
        let capped = calculate_loudness_gain_db(Some(0.11)).unwrap();
        assert_eq!(capped, MAX_BOOST_DB);

        // No metadata -> None
        assert_eq!(calculate_loudness_gain_db(None), None);
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
}

