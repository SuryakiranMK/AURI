// AURI Symphonia Audio Decoder Pipeline
// Pure Rust safe decoding for FLAC, MP3, AAC, Opus, Vorbis, and WAV

use std::fs::File;
use std::path::Path;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{CodecRegistry, Decoder, DecoderOptions};
use symphonia::core::errors::Error as DecodeError;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::{Time, TimeBase};

pub struct DecodedAudioPacket {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: usize,
    pub duration_secs: f64,
}

pub struct AudioDecoder {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    _time_base: Option<TimeBase>,
    sample_buf: Option<SampleBuffer<f32>>,
    duration_secs: f64,
}

impl AudioDecoder {
    /// Open and initialize decoder from a local file path
    pub fn open_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let file = File::open(path.as_ref())
            .map_err(|e| format!("Failed to open audio file: {}", e))?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = path.as_ref().extension().and_then(|s| s.to_str()) {
            hint.with_extension(ext);
        }

        Self::open_source(mss, hint)
    }

    /// Open and initialize decoder from an arbitrary MediaSource (e.g. streaming sliding buffer)
    pub fn open_stream(source: Box<dyn MediaSource>, extension_hint: &str) -> Result<Self, String> {
        let mss = MediaSourceStream::new(source, Default::default());
        let mut hint = Hint::new();
        hint.with_extension(extension_hint);
        Self::open_source(mss, hint)
    }

    fn open_source(mss: MediaSourceStream, hint: Hint) -> Result<Self, String> {
        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .map_err(|e| format!("Failed to probe audio format: {}", e))?;

        let format = probed.format;
        let track = format
            .default_track()
            .ok_or_else(|| "No default audio track found in media container".to_string())?;

        let track_id = track.id;
        let time_base = track.codec_params.time_base;

        let duration_secs = track
            .codec_params
            .n_frames
            .and_then(|frames| {
                track
                    .codec_params
                    .sample_rate
                    .map(|rate| frames as f64 / rate as f64)
            })
            .unwrap_or(0.0);

        let decoder_opts = DecoderOptions::default();
        let mut codecs = CodecRegistry::new();
        symphonia::default::register_enabled_codecs(&mut codecs);
        codecs.register_all::<super::opus::OpusStream>();
        let decoder = codecs
            .make(&track.codec_params, &decoder_opts)
            .map_err(|e| format!("Failed to create audio decoder: {}", e))?;

        Ok(Self {
            format,
            decoder,
            track_id,
            _time_base: time_base,
            sample_buf: None,
            duration_secs,
        })
    }

    pub fn duration_secs(&self) -> f64 {
        self.duration_secs
    }

    /// Decode the next audio packet. Returns Ok(Some(packet)) on packet, Ok(None) on EOF.
    pub fn next_packet(&mut self) -> Result<Option<DecodedAudioPacket>, String> {
        loop {
            let packet = match self.format.next_packet() {
                Ok(packet) => packet,
                Err(DecodeError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    return Ok(None);
                }
                Err(DecodeError::ResetRequired) => {
                    self.decoder.reset();
                    continue;
                }
                Err(e) => return Err(format!("Demuxer packet error: {}", e)),
            };

            if packet.track_id() != self.track_id {
                continue;
            }

            let decoded = match self.decoder.decode(&packet) {
                Ok(decoded) => decoded,
                Err(DecodeError::DecodeError(_)) => continue,
                Err(e) => return Err(format!("Decoder error: {}", e)),
            };

            let spec = *decoded.spec();
            let channels = spec.channels.count();
            let rate = spec.rate;

            let sample_buf = match &mut self.sample_buf {
                Some(buf) if buf.capacity() >= decoded.frames() * channels => buf,
                slot => slot.insert(SampleBuffer::new(decoded.frames() as u64, spec)),
            };

            sample_buf.copy_interleaved_ref(decoded);
            let samples = sample_buf.samples().to_vec();

            return Ok(Some(DecodedAudioPacket {
                samples,
                sample_rate: rate,
                channels,
                duration_secs: self.duration_secs,
            }));
        }
    }

    /// Seek to a specified time in seconds
    pub fn seek(&mut self, time_secs: f64) -> Result<(), String> {
        let time = Time::from(time_secs);
        self.format
            .seek(
                SeekMode::Accurate,
                SeekTo::Time {
                    time,
                    track_id: Some(self.track_id),
                },
            )
            .map_err(|e| format!("Seek failed: {}", e))?;
        self.decoder.reset();
        Ok(())
    }
}

