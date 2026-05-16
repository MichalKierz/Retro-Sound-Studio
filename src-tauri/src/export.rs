use crate::audio::{
    frequency_to_midi_note, render_melody, render_sound, MelodyProject, RenderedAudio,
};
use crate::synth::SynthParams;
use midly::{Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};
use mp3lame_encoder::{Bitrate, Builder, FlushNoGap, InterleavedPcm, Mode, MonoPcm, Quality};
use std::fs::{self, File};
use std::io::Write;
use std::num::{NonZeroU32, NonZeroU8};
use std::path::{Path, PathBuf};
use vorbis_rs::{VorbisBitrateManagementStrategy, VorbisEncoderBuilder};

#[derive(Clone, serde::Serialize)]
struct ExportProgress {
    kind: String,
    percent: u8,
    label: String,
}

#[derive(Clone, serde::Deserialize)]
pub struct ExportOptions {
    wav_bit_depth: Option<String>,
    normalize_peak: Option<bool>,
    mp3_bitrate: Option<u16>,
    mp3_quality: Option<u8>,
    ogg_quality: Option<u8>,
    ogg_mode: Option<String>,
    ogg_bitrate: Option<u32>,
    midi_ticks_per_beat: Option<u16>,
    midi_note_beats: Option<f32>,
    midi_velocity_scale: Option<f32>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            wav_bit_depth: Some("16".to_string()),
            normalize_peak: Some(true),
            mp3_bitrate: Some(192),
            mp3_quality: Some(2),
            ogg_quality: Some(6),
            ogg_mode: Some("quality".to_string()),
            ogg_bitrate: Some(128000),
            midi_ticks_per_beat: Some(480),
            midi_note_beats: Some(2.0),
            midi_velocity_scale: Some(100.0),
        }
    }
}

#[tauri::command]
pub fn export_sound(
    window: tauri::Window,
    params: SynthParams,
    format: String,
    filename: String,
    folder: Option<String>,
    options: Option<ExportOptions>,
) -> Result<String, String> {
    let options = options.unwrap_or_default().normalized();
    emit_progress(&window, "sounds", 5, "Preparing");
    let format = normalize_format(&format)?;
    let export_dir = ensure_export_dir("sounds", folder)?;
    let path = export_dir.join(format!("{}.{}", safe_filename(&filename), format));
    emit_progress(&window, "sounds", 20, "Rendering");

    if format == "midi" {
        emit_progress(&window, "sounds", 70, "Writing MIDI");
        write_single_note_midi(
            &path,
            params.frequency,
            options.midi_ticks(),
            options.midi_note_beats(),
        )?;
    } else {
        let rendered = render_sound(params, 0.85);
        write_audio_file(&path, &format, rendered, &options, |percent, label| {
            emit_progress(&window, "sounds", percent, label)
        })?;
    }

    emit_progress(&window, "sounds", 100, "Done");
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn export_melody(
    window: tauri::Window,
    project: MelodyProject,
    format: String,
    filename: String,
    folder: Option<String>,
    options: Option<ExportOptions>,
) -> Result<String, String> {
    let options = options.unwrap_or_default().normalized();
    emit_progress(&window, "melodies", 5, "Preparing");
    let format = normalize_format(&format)?;
    let export_dir = ensure_export_dir("melodies", folder)?;
    let path = export_dir.join(format!("{}.{}", safe_filename(&filename), format));
    emit_progress(&window, "melodies", 20, "Rendering");

    if format == "midi" {
        emit_progress(&window, "melodies", 70, "Writing MIDI");
        write_melody_midi(
            &path,
            project,
            options.midi_ticks(),
            options.midi_velocity_scale(),
        )?;
    } else {
        let rendered = render_melody(project);
        write_audio_file(&path, &format, rendered, &options, |percent, label| {
            emit_progress(&window, "melodies", percent, label)
        })?;
    }

    emit_progress(&window, "melodies", 100, "Done");
    Ok(path.to_string_lossy().to_string())
}

fn write_audio_file<F>(
    path: &Path,
    format: &str,
    mut rendered: RenderedAudio,
    options: &ExportOptions,
    mut progress: F,
) -> Result<(), String>
where
    F: FnMut(u8, &'static str),
{
    if options.normalize_peak.unwrap_or(true) {
        normalize_peak(&mut rendered.samples);
    }
    match format {
        "wav" => write_wav(path, rendered, options.wav_bit_depth(), progress),
        "mp3" => {
            progress(70, "Encoding MP3");
            write_mp3(path, rendered, options.mp3_bitrate(), options.mp3_quality())?;
            progress(95, "Writing MP3");
            Ok(())
        }
        "ogg" => {
            progress(70, "Encoding OGG");
            write_ogg(path, rendered, options.ogg_strategy())?;
            progress(95, "Writing OGG");
            Ok(())
        }
        _ => Err(format!("Unsupported audio format: {}", format)),
    }
}

fn write_wav<F>(
    path: &Path,
    rendered: RenderedAudio,
    bit_depth: WavBitDepth,
    mut progress: F,
) -> Result<(), String>
where
    F: FnMut(u8, &'static str),
{
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: rendered.sample_rate,
        bits_per_sample: bit_depth.bits(),
        sample_format: bit_depth.sample_format(),
    };
    let mut writer = hound::WavWriter::create(path, spec).map_err(|error| error.to_string())?;
    let channels = rendered.channels.max(1);
    let samples = if channels == 2 {
        rendered.samples
    } else {
        rendered
            .samples
            .into_iter()
            .flat_map(|sample| [sample, sample])
            .collect()
    };
    let total = samples.len().max(1);
    let step = (total / 24).max(1);
    for (index, sample) in samples.into_iter().enumerate() {
        if index % step == 0 {
            let percent = 55 + ((index as f32 / total as f32) * 40.0).round() as u8;
            progress(percent.min(95), "Writing WAV");
        }
        match bit_depth {
            WavBitDepth::Pcm16 => {
                let sample = f32_to_i16(sample);
                writer
                    .write_sample(sample)
                    .map_err(|error| error.to_string())?;
            }
            WavBitDepth::Pcm24 => {
                let sample = f32_to_i24(sample);
                writer
                    .write_sample(sample)
                    .map_err(|error| error.to_string())?;
            }
            WavBitDepth::Float32 => {
                let sample = sample.clamp(-1.0, 1.0);
                writer
                    .write_sample(sample)
                    .map_err(|error| error.to_string())?;
            }
        }
    }
    progress(95, "Writing WAV");
    writer.finalize().map_err(|error| error.to_string())
}

fn write_mp3(
    path: &Path,
    rendered: RenderedAudio,
    bitrate: Bitrate,
    quality: Quality,
) -> Result<(), String> {
    let mut builder = Builder::new().ok_or_else(|| "MP3 builder error".to_string())?;
    builder
        .set_sample_rate(rendered.sample_rate)
        .map_err(|error| format!("MP3 sample rate error: {:?}", error))?;
    let channels = rendered.channels.clamp(1, 2);
    builder
        .set_num_channels(channels as u8)
        .map_err(|error| format!("MP3 channel error: {:?}", error))?;
    builder
        .set_mode(if channels == 2 {
            Mode::JointStereo
        } else {
            Mode::Mono
        })
        .map_err(|error| format!("MP3 mode error: {:?}", error))?;
    builder
        .set_brate(bitrate)
        .map_err(|error| format!("MP3 bitrate error: {:?}", error))?;
    builder
        .set_quality(quality)
        .map_err(|error| format!("MP3 quality error: {:?}", error))?;
    let mut encoder = builder
        .build()
        .map_err(|error| format!("MP3 encoder error: {:?}", error))?;
    let pcm: Vec<i16> = samples_for_channels(&rendered, channels)
        .iter()
        .map(|sample| f32_to_i16(*sample))
        .collect();
    let mut encoded = Vec::new();
    let frame_count = if channels == 2 {
        pcm.len() / 2
    } else {
        pcm.len()
    };
    encoded.reserve(mp3lame_encoder::max_required_buffer_size(frame_count));
    let encoded_size = if channels == 2 {
        encoder
            .encode(InterleavedPcm(&pcm), encoded.spare_capacity_mut())
            .map_err(|error| format!("MP3 encode error: {:?}", error))?
    } else {
        encoder
            .encode(MonoPcm(&pcm), encoded.spare_capacity_mut())
            .map_err(|error| format!("MP3 encode error: {:?}", error))?
    };
    unsafe {
        encoded.set_len(encoded_size);
    }

    let mut file = File::create(path).map_err(|error| error.to_string())?;
    file.write_all(&encoded)
        .map_err(|error| error.to_string())?;

    let mut flushed = Vec::new();
    flushed.reserve(7200);
    let flush_size = encoder
        .flush::<FlushNoGap>(flushed.spare_capacity_mut())
        .map_err(|error| format!("MP3 flush error: {:?}", error))?;
    unsafe {
        flushed.set_len(flush_size);
    }
    file.write_all(&flushed).map_err(|error| error.to_string())
}

fn write_ogg(
    path: &Path,
    rendered: RenderedAudio,
    strategy: VorbisBitrateManagementStrategy,
) -> Result<(), String> {
    let sample_rate =
        NonZeroU32::new(rendered.sample_rate).ok_or_else(|| "Invalid sample rate".to_string())?;
    let channel_count = rendered.channels.clamp(1, 2);
    let channels =
        NonZeroU8::new(channel_count as u8).ok_or_else(|| "Invalid channel count".to_string())?;
    let file = File::create(path).map_err(|error| error.to_string())?;
    let mut builder = VorbisEncoderBuilder::new_with_serial(sample_rate, channels, file, 12345);
    builder.bitrate_management_strategy(strategy);
    let mut encoder = builder.build().map_err(|error| error.to_string())?;
    let channel_samples = planar_samples_for_channels(&rendered, channel_count);
    encoder
        .encode_audio_block(channel_samples)
        .map_err(|error| error.to_string())?;
    encoder
        .finish()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn write_single_note_midi(
    path: &Path,
    frequency: f32,
    ticks_per_beat: u16,
    note_beats: f32,
) -> Result<(), String> {
    let note = frequency_to_midi_note(frequency);
    let note_ticks = (note_beats.max(0.25) * ticks_per_beat as f32)
        .round()
        .max(1.0) as u32;
    let track = vec![
        tempo_event(0, 500000),
        note_event(0, 0, note, 100, true),
        note_event(note_ticks, 0, note, 0, false),
        end_event(0),
    ];
    save_midi(path, ticks_per_beat, vec![track])
}

fn write_melody_midi(
    path: &Path,
    project: MelodyProject,
    ticks_per_beat: u16,
    velocity_scale: f32,
) -> Result<(), String> {
    let project = project.normalized();
    let tempo = project.tempo.unwrap_or(120.0);
    let micros_per_quarter = (60_000_000.0 / tempo).round().clamp(250000.0, 1500000.0) as u32;
    let ticks_per_beat_f32 = ticks_per_beat as f32;
    let mut events = Vec::new();

    for layer in project.layers {
        let channel = layer.id.saturating_sub(1).min(15) as u8;
        for note in layer.notes {
            let start = (note.start * ticks_per_beat_f32).round().max(0.0) as u32;
            let length = (note.duration * ticks_per_beat_f32).round().max(1.0) as u32;
            let velocity = (note.velocity.unwrap_or(0.85) * velocity_scale * 127.0)
                .round()
                .clamp(1.0, 127.0) as u8;
            events.push((start, channel, note.pitch, velocity, true));
            events.push((start + length, channel, note.pitch, 0, false));
        }
    }

    events.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| left.4.cmp(&right.4))
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.2.cmp(&right.2))
    });

    let mut track = Vec::with_capacity(events.len() + 2);
    track.push(tempo_event(0, micros_per_quarter));
    let mut cursor = 0;
    for (tick, channel, pitch, velocity, on) in events {
        let delta = tick.saturating_sub(cursor);
        cursor = tick;
        track.push(note_event(delta, channel, pitch, velocity, on));
    }
    track.push(end_event(0));
    save_midi(path, ticks_per_beat, vec![track])
}

fn save_midi(
    path: &Path,
    ticks_per_beat: u16,
    tracks: Vec<Vec<TrackEvent<'static>>>,
) -> Result<(), String> {
    let smf = Smf {
        header: Header::new(Format::SingleTrack, Timing::Metrical(ticks_per_beat.into())),
        tracks,
    };
    smf.save(path).map_err(|error| error.to_string())
}

fn tempo_event(delta: u32, micros_per_quarter: u32) -> TrackEvent<'static> {
    TrackEvent {
        delta: delta.into(),
        kind: TrackEventKind::Meta(MetaMessage::Tempo(micros_per_quarter.into())),
    }
}

fn note_event(delta: u32, channel: u8, pitch: u8, velocity: u8, on: bool) -> TrackEvent<'static> {
    let message = if on {
        MidiMessage::NoteOn {
            key: pitch.into(),
            vel: velocity.into(),
        }
    } else {
        MidiMessage::NoteOff {
            key: pitch.into(),
            vel: velocity.into(),
        }
    };
    TrackEvent {
        delta: delta.into(),
        kind: TrackEventKind::Midi {
            channel: channel.into(),
            message,
        },
    }
}

fn end_event(delta: u32) -> TrackEvent<'static> {
    TrackEvent {
        delta: delta.into(),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    }
}

#[derive(Clone, Copy)]
enum WavBitDepth {
    Pcm16,
    Pcm24,
    Float32,
}

impl WavBitDepth {
    fn bits(self) -> u16 {
        match self {
            Self::Pcm16 => 16,
            Self::Pcm24 => 24,
            Self::Float32 => 32,
        }
    }

    fn sample_format(self) -> hound::SampleFormat {
        match self {
            Self::Float32 => hound::SampleFormat::Float,
            _ => hound::SampleFormat::Int,
        }
    }
}

impl ExportOptions {
    fn normalized(mut self) -> Self {
        self.wav_bit_depth = Some(match self.wav_bit_depth.as_deref() {
            Some("24") => "24".to_string(),
            Some("32f") => "32f".to_string(),
            _ => "16".to_string(),
        });
        self.mp3_bitrate = Some(self.mp3_bitrate.unwrap_or(192).clamp(128, 320));
        self.mp3_quality = Some(self.mp3_quality.unwrap_or(2).min(9));
        self.ogg_quality = Some(self.ogg_quality.unwrap_or(6).min(10));
        self.ogg_mode = Some(
            if self.ogg_mode.as_deref() == Some("bitrate") {
                "bitrate"
            } else {
                "quality"
            }
            .to_string(),
        );
        self.ogg_bitrate = Some(self.ogg_bitrate.unwrap_or(128000).clamp(64000, 320000));
        self.midi_ticks_per_beat = Some(match self.midi_ticks_per_beat.unwrap_or(480) {
            240 => 240,
            960 => 960,
            _ => 480,
        });
        self.midi_note_beats = Some(self.midi_note_beats.unwrap_or(2.0).clamp(0.25, 16.0));
        self.midi_velocity_scale =
            Some((self.midi_velocity_scale.unwrap_or(100.0) / 100.0).clamp(0.5, 1.4));
        self.normalize_peak = Some(self.normalize_peak.unwrap_or(true));
        self
    }

    fn wav_bit_depth(&self) -> WavBitDepth {
        match self.wav_bit_depth.as_deref() {
            Some("24") => WavBitDepth::Pcm24,
            Some("32f") => WavBitDepth::Float32,
            _ => WavBitDepth::Pcm16,
        }
    }

    fn mp3_bitrate(&self) -> Bitrate {
        match self.mp3_bitrate.unwrap_or(192) {
            128 => Bitrate::Kbps128,
            256 => Bitrate::Kbps256,
            320 => Bitrate::Kbps320,
            _ => Bitrate::Kbps192,
        }
    }

    fn mp3_quality(&self) -> Quality {
        match self.mp3_quality.unwrap_or(2) {
            0 => Quality::Best,
            1 => Quality::SecondBest,
            3 => Quality::VeryNice,
            4 => Quality::Nice,
            5 => Quality::Good,
            6 => Quality::Decent,
            7 => Quality::Ok,
            8 => Quality::SecondWorst,
            9 => Quality::Worst,
            _ => Quality::NearBest,
        }
    }

    fn ogg_strategy(&self) -> VorbisBitrateManagementStrategy {
        if self.ogg_mode.as_deref() == Some("bitrate") {
            VorbisBitrateManagementStrategy::Vbr {
                target_bitrate: NonZeroU32::new(self.ogg_bitrate.unwrap_or(128000))
                    .unwrap_or(NonZeroU32::new(128000).unwrap()),
            }
        } else {
            VorbisBitrateManagementStrategy::QualityVbr {
                target_quality: self.ogg_quality.unwrap_or(6) as f32 / 10.0,
            }
        }
    }

    fn midi_ticks(&self) -> u16 {
        self.midi_ticks_per_beat.unwrap_or(480)
    }

    fn midi_note_beats(&self) -> f32 {
        self.midi_note_beats.unwrap_or(2.0)
    }

    fn midi_velocity_scale(&self) -> f32 {
        self.midi_velocity_scale.unwrap_or(1.0)
    }
}

#[tauri::command]
pub fn default_export_dir(kind: String) -> Result<String, String> {
    let kind = normalize_export_kind(&kind)?;
    let dir = ensure_export_dir(&kind, None)?;
    Ok(dir.to_string_lossy().to_string())
}

fn emit_progress(window: &tauri::Window, kind: &str, percent: u8, label: &str) {
    let _ = window.emit(
        "export-progress",
        ExportProgress {
            kind: kind.to_string(),
            percent,
            label: label.to_string(),
        },
    );
}

fn ensure_export_dir(kind: &str, folder: Option<String>) -> Result<PathBuf, String> {
    let dir = if let Some(folder) = folder.filter(|value| !value.trim().is_empty()) {
        PathBuf::from(folder)
    } else {
        portable_root().join("exports").join(kind)
    };
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    Ok(dir)
}

fn portable_root() -> PathBuf {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if current_dir.join("presets").exists() || current_dir.join("exports").exists() {
        return current_dir;
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return dir.to_path_buf();
        }
    }
    current_dir
}

fn normalize_format(format: &str) -> Result<String, String> {
    let format = format.trim().to_ascii_lowercase();
    match format.as_str() {
        "wav" | "mp3" | "ogg" | "midi" => Ok(format),
        _ => Err(format!("Unsupported format: {}", format)),
    }
}

fn normalize_export_kind(kind: &str) -> Result<String, String> {
    let kind = kind.trim().to_ascii_lowercase();
    match kind.as_str() {
        "sounds" | "melodies" => Ok(kind),
        _ => Err(format!("Unsupported export kind: {}", kind)),
    }
}

fn safe_filename(filename: &str) -> String {
    let cleaned: String = filename
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = cleaned.trim_matches('_');
    if trimmed.is_empty() {
        "retro_export".to_string()
    } else {
        trimmed.chars().take(64).collect()
    }
}

fn f32_to_i16(sample: f32) -> i16 {
    let amplitude = i16::MAX as f32;
    (sample.clamp(-1.0, 1.0) * amplitude) as i16
}

fn f32_to_i24(sample: f32) -> i32 {
    let amplitude = 8_388_607.0;
    (sample.clamp(-1.0, 1.0) * amplitude) as i32
}

fn samples_for_channels(rendered: &RenderedAudio, channels: u16) -> Vec<f32> {
    if channels == 2 {
        if rendered.channels == 2 {
            rendered.samples.clone()
        } else {
            rendered
                .samples
                .iter()
                .flat_map(|sample| [*sample, *sample])
                .collect()
        }
    } else {
        rendered.mono_samples()
    }
}

fn planar_samples_for_channels(rendered: &RenderedAudio, channels: u16) -> Vec<Vec<f32>> {
    if channels == 2 {
        let interleaved = samples_for_channels(rendered, 2);
        let mut left = Vec::with_capacity(interleaved.len() / 2);
        let mut right = Vec::with_capacity(interleaved.len() / 2);
        for frame in interleaved.chunks_exact(2) {
            left.push(frame[0]);
            right.push(frame[1]);
        }
        vec![left, right]
    } else {
        vec![rendered.mono_samples()]
    }
}

fn normalize_peak(samples: &mut [f32]) {
    let peak = samples
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0_f32, f32::max);
    if peak > 0.0 {
        let gain = 0.98 / peak;
        for sample in samples {
            *sample = (*sample * gain).clamp(-1.0, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::{MelodyLayer, MelodyNote};

    #[test]
    fn safe_filename_removes_path_characters() {
        assert_eq!(safe_filename("../bad:name"), "bad_name");
    }

    #[test]
    fn midi_export_writes_parseable_file() {
        let path = std::env::temp_dir().join("retro_sound_studio_test_sound.mid");
        write_single_note_midi(&path, 440.0, 480, 2.0).unwrap();
        let bytes = fs::read(&path).unwrap();
        let parsed = Smf::parse(&bytes).unwrap();
        assert_eq!(parsed.tracks.len(), 1);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn melody_midi_export_writes_notes() {
        let path = std::env::temp_dir().join("retro_sound_studio_test_melody.mid");
        let project = MelodyProject {
            name: None,
            description: None,
            tempo: Some(120.0),
            sample_rate: Some(44100),
            layers: vec![MelodyLayer {
                id: 1,
                name: None,
                sound: None,
                muted: false,
                volume: 1.0,
                sound_sample: None,
                sound_file_path: None,
                sound_label: None,
                melody_preset_id: None,
                notes: vec![MelodyNote {
                    pitch: 60,
                    start: 0.0,
                    duration: 1.0,
                    velocity: Some(0.8),
                }],
            }],
        };
        write_melody_midi(&path, project, 480, 1.0).unwrap();
        let bytes = fs::read(&path).unwrap();
        let parsed = Smf::parse(&bytes).unwrap();
        assert_eq!(parsed.tracks.len(), 1);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn wav_export_writes_identical_stereo_channels() {
        let path = std::env::temp_dir().join("retro_sound_studio_test_stereo.wav");
        let _ = fs::remove_file(&path);
        let rendered = RenderedAudio {
            samples: vec![0.25, -0.5, 0.75],
            sample_rate: 44100,
            channels: 1,
        };
        write_wav(&path, rendered, WavBitDepth::Pcm16, |_, _| {}).unwrap();
        let mut reader = hound::WavReader::open(&path).unwrap();
        assert_eq!(reader.spec().channels, 2);
        let samples = reader
            .samples::<i16>()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(samples.len(), 6);
        for frame in samples.chunks_exact(2) {
            assert_eq!(frame[0], frame[1]);
        }
        let _ = fs::remove_file(path);
    }

    #[test]
    fn wav_export_preserves_stereo_rendered_audio() {
        let path = std::env::temp_dir().join("retro_sound_studio_test_pan_stereo.wav");
        let _ = fs::remove_file(&path);
        let rendered = RenderedAudio {
            samples: vec![0.8, 0.1, -0.6, -0.2],
            sample_rate: 44100,
            channels: 2,
        };
        write_wav(&path, rendered, WavBitDepth::Pcm16, |_, _| {}).unwrap();
        let mut reader = hound::WavReader::open(&path).unwrap();
        assert_eq!(reader.spec().channels, 2);
        let samples = reader
            .samples::<i16>()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_ne!(samples[0], samples[1]);
        assert_ne!(samples[2], samples[3]);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn compressed_export_helpers_preserve_stereo_frames() {
        let rendered = RenderedAudio {
            samples: vec![0.8, 0.1, -0.6, -0.2],
            sample_rate: 44100,
            channels: 2,
        };
        let interleaved = samples_for_channels(&rendered, 2);
        assert_eq!(interleaved, rendered.samples);
        let planar = planar_samples_for_channels(&rendered, 2);
        assert_eq!(planar.len(), 2);
        assert_eq!(planar[0], vec![0.8, -0.6]);
        assert_eq!(planar[1], vec![0.1, -0.2]);

        let mono = samples_for_channels(&rendered, 1);
        assert!((mono[0] - 0.45).abs() < 0.0001);
        assert!((mono[1] + 0.4).abs() < 0.0001);
    }

    #[test]
    fn mono_render_can_be_promoted_to_stereo_for_compressed_export() {
        let rendered = RenderedAudio {
            samples: vec![0.25, -0.5],
            sample_rate: 44100,
            channels: 1,
        };
        assert_eq!(
            samples_for_channels(&rendered, 2),
            vec![0.25, 0.25, -0.5, -0.5]
        );
        assert_eq!(
            planar_samples_for_channels(&rendered, 2),
            vec![vec![0.25, -0.5], vec![0.25, -0.5]]
        );
    }

    #[test]
    fn normalize_export_kind_accepts_supported_folders() {
        assert_eq!(normalize_export_kind("sounds").unwrap(), "sounds");
        assert_eq!(normalize_export_kind(" Melodies ").unwrap(), "melodies");
        assert!(normalize_export_kind("other").is_err());
    }

    #[test]
    fn ensure_export_dir_uses_selected_folder() {
        let path = std::env::temp_dir().join("retro_sound_studio_selected_export");
        let _ = fs::remove_dir_all(&path);
        let resolved =
            ensure_export_dir("sounds", Some(path.to_string_lossy().to_string())).unwrap();
        assert_eq!(resolved, path);
        assert!(resolved.exists());
        let _ = fs::remove_dir_all(resolved);
    }

    #[test]
    fn default_sound_export_dir_points_to_sound_exports() {
        let dir = default_export_dir("sounds".to_string()).unwrap();
        assert!(dir.ends_with("exports/sounds") || dir.ends_with("exports\\sounds"));
        assert!(PathBuf::from(dir).exists());
    }

    #[test]
    fn export_options_normalize_to_supported_values() {
        let options = ExportOptions {
            wav_bit_depth: Some("bad".to_string()),
            normalize_peak: None,
            mp3_bitrate: Some(999),
            mp3_quality: Some(99),
            ogg_quality: Some(99),
            ogg_mode: Some("bitrate".to_string()),
            ogg_bitrate: Some(1),
            midi_ticks_per_beat: Some(123),
            midi_note_beats: Some(0.0),
            midi_velocity_scale: Some(500.0),
        }
        .normalized();
        assert!(matches!(options.wav_bit_depth(), WavBitDepth::Pcm16));
        assert_eq!(options.midi_ticks(), 480);
        assert_eq!(options.midi_note_beats(), 0.25);
        assert_eq!(options.midi_velocity_scale(), 1.4);
    }

    #[test]
    fn normalize_peak_sets_consistent_headroom() {
        let mut samples = vec![0.25, -0.5, 0.125];
        normalize_peak(&mut samples);
        let peak = samples
            .iter()
            .map(|sample| sample.abs())
            .fold(0.0_f32, f32::max);
        assert!((peak - 0.98).abs() < 0.001);
    }
}
