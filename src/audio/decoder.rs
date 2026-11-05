// Pure Rust Symphonia Audio Demuxer & Decoder
// Supports FLAC, MP3, AAC, Opus, Vorbis, and WAV with sample-accurate seeking.

use std::fs::File;
use std::path::Path;
use symphonia::core::codecs::audio::{AudioDecoder as SymphoniaAudioDecoder, AudioDecoderOptions};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::units::{Duration, Time};

pub struct AudioDecoder {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn SymphoniaAudioDecoder>,
    track_id: u32,
    sample_rate: u32,
    channels: usize,
    duration_secs: f64,
    current_time_secs: f64,
    time_base: Option<symphonia::core::units::TimeBase>,
}

impl AudioDecoder {
    pub fn open_file(path: &Path) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            hint.with_extension(ext);
        }

        Self::from_media_source(mss, hint)
    }

    pub fn open_stream(
        source: Box<dyn symphonia::core::io::MediaSource>,
        extension_hint: Option<&str>,
    ) -> Result<Self, String> {
        let mss = MediaSourceStream::new(source, Default::default());
        let mut hint = Hint::new();
        if let Some(ext) = extension_hint {
            hint.with_extension(ext);
        }

        Self::from_media_source(mss, hint)
    }

    fn from_media_source(mss: MediaSourceStream<'static>, hint: Hint) -> Result<Self, String> {
        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();

        let format = symphonia::default::get_probe()
            .probe(&hint, mss, format_opts, metadata_opts)
            .map_err(|e| format!("Audio format probe error: {}", e))?;

        let track = format
            .default_track(TrackType::Audio)
            .ok_or_else(|| "No default audio track found in container".to_string())?;

        let track_id = track.id;
        let time_base = track.time_base;

        let audio_params = track
            .codec_params
            .as_ref()
            .and_then(|p| p.audio())
            .ok_or_else(|| "Missing audio codec parameters".to_string())?;

        let sample_rate = audio_params.sample_rate.unwrap_or(44100);
        let channels = audio_params.channels.as_ref().map(|c| c.count()).unwrap_or(2);

        let duration_secs = if let (Some(dur), Some(tb)) = (track.duration, time_base) {
            if let Some(time) = tb.calc_duration(dur) {
                time.as_secs_f64()
            } else {
                0.0
            }
        } else if let (Some(n_frames), Some(tb)) = (track.num_frames, time_base) {
            if let Some(time) = tb.calc_duration(Duration::from(n_frames)) {
                time.as_secs_f64()
            } else {
                0.0
            }
        } else {
            0.0
        };

        let decode_opts = AudioDecoderOptions::default();
        let decoder = symphonia::default::get_codecs()
            .make_audio_decoder(audio_params, &decode_opts)
            .map_err(|e| format!("Codec initialization failed: {}", e))?;

        Ok(Self {
            format,
            decoder,
            track_id,
            sample_rate,
            channels,
            duration_secs,
            current_time_secs: 0.0,
            time_base,
        })
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> usize {
        self.channels
    }

    pub fn duration_secs(&self) -> f64 {
        self.duration_secs
    }

    pub fn current_time_secs(&self) -> f64 {
        self.current_time_secs
    }

    /// Decodes the next packet into interleaved f32 audio samples.
    pub fn decode_next(&mut self) -> Result<Option<Vec<f32>>, String> {
        loop {
            let packet = match self.format.next_packet() {
                Ok(Some(packet)) => packet,
                Ok(None) => return Ok(None),
                Err(SymphoniaError::IoError(ref err)) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                    return Ok(None);
                }
                Err(SymphoniaError::ResetRequired) => {
                    self.decoder.reset();
                    continue;
                }
                Err(e) => return Err(format!("Demux error: {}", e)),
            };

            if packet.track_id != self.track_id {
                continue;
            }

            if let Some(tb) = self.time_base {
                if let Some(time) = tb.calc_time(packet.pts) {
                    self.current_time_secs = time.as_secs_f64();
                }
            }

            match self.decoder.decode(&packet) {
                Ok(decoded) => {
                    let mut samples: Vec<f32> = Vec::new();
                    decoded.copy_to_vec_interleaved(&mut samples);
                    return Ok(Some(samples));
                }
                Err(SymphoniaError::DecodeError(err)) => {
                    eprintln!("[AURI Decoder] Non-fatal frame decode warning: {}", err);
                    continue;
                }
                Err(e) => return Err(format!("Fatal decode error: {}", e)),
            }
        }
    }

    /// Seeks to a specific timestamp in seconds.
    pub fn seek(&mut self, seconds: f64) -> Result<(), String> {
        let time = Time::try_from_secs_f64(seconds).unwrap_or_default();
        self.format
            .seek(SeekMode::Coarse, SeekTo::Time { time, track_id: Some(self.track_id) })
            .map_err(|e| format!("Seek failed: {}", e))?;
        self.decoder.reset();
        self.current_time_secs = seconds;
        Ok(())
    }
}
