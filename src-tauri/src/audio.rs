use crate::synth::{RetroSynth, SynthParams};

#[derive(Clone)]
pub struct RenderedAudio {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

impl RenderedAudio {
    pub fn mono_samples(&self) -> Vec<f32> {
        if self.channels <= 1 {
            return self.samples.clone();
        }
        self.samples
            .chunks(self.channels as usize)
            .map(|frame| frame.iter().sum::<f32>() / frame.len().max(1) as f32)
            .collect()
    }
}

#[derive(Clone, serde::Serialize, Debug)]
pub struct WaveformEnvelope {
    pub sample_rate: u32,
    pub duration_seconds: f32,
    pub points_per_second: f32,
    pub points: Vec<WaveformPoint>,
}

#[derive(Clone, serde::Serialize, Debug)]
pub struct WaveformPoint {
    pub min: f32,
    pub max: f32,
    pub rms: f32,
}

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
pub struct MelodyProject {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tempo: Option<f32>,
    pub sample_rate: Option<u32>,
    pub layers: Vec<MelodyLayer>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
pub struct MelodyLayer {
    pub id: u32,
    pub name: Option<String>,
    pub sound: Option<SynthParams>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub muted: bool,
    #[serde(
        default = "default_layer_volume",
        skip_serializing_if = "is_default_layer_volume"
    )]
    pub volume: f32,
    #[serde(
        default,
        rename = "soundSample",
        skip_serializing_if = "Option::is_none"
    )]
    pub sound_sample: Option<AudioSample>,
    #[serde(
        default,
        rename = "soundFilePath",
        skip_serializing_if = "Option::is_none"
    )]
    pub sound_file_path: Option<String>,
    #[serde(
        default,
        rename = "soundLabel",
        skip_serializing_if = "Option::is_none"
    )]
    pub sound_label: Option<String>,
    #[serde(
        default,
        rename = "melodyPresetId",
        skip_serializing_if = "Option::is_none"
    )]
    pub melody_preset_id: Option<String>,
    pub notes: Vec<MelodyNote>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
pub struct AudioSample {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
pub struct MelodyNote {
    pub pitch: u8,
    pub start: f32,
    pub duration: f32,
    pub velocity: Option<f32>,
}

impl MelodyProject {
    pub fn normalized(mut self) -> Self {
        let sample_rate = self.sample_rate.unwrap_or(44100).clamp(8000, 192000);
        self.sample_rate = Some(sample_rate);
        self.tempo = Some(self.tempo.unwrap_or(120.0).clamp(40.0, 240.0));
        self.layers = self
            .layers
            .into_iter()
            .map(|mut layer| {
                layer.id = layer.id.max(1);
                layer.sound = layer.sound.map(|sound| sound.normalized());
                layer.sound_sample = layer.sound_sample.map(|sample| sample.normalized());
                layer.sound_file_path = layer
                    .sound_file_path
                    .filter(|value| !value.trim().is_empty());
                layer.sound_label = layer.sound_label.filter(|value| !value.trim().is_empty());
                layer.melody_preset_id = layer
                    .melody_preset_id
                    .filter(|value| !value.trim().is_empty());
                layer.volume = layer.volume.clamp(0.0, 1.0);
                if layer.volume <= f32::EPSILON {
                    layer.muted = true;
                }
                layer.notes = layer.notes.into_iter().filter_map(normalize_note).collect();
                layer
            })
            .collect();
        self
    }
}

pub fn render_sound(params: SynthParams, hold_seconds: f32) -> RenderedAudio {
    let params = params.normalized();
    let synth = RetroSynth::new_with_hold(params, hold_seconds);
    synth.render()
}

pub fn analyze_waveform(
    samples: &[f32],
    sample_rate: u32,
    requested_points: usize,
) -> WaveformEnvelope {
    let points_target = requested_points.clamp(512, 16384).min(samples.len().max(1));
    let bucket_size = ((samples.len().max(1) as f32) / (points_target as f32))
        .ceil()
        .max(1.0) as usize;
    let points = if samples.is_empty() {
        vec![WaveformPoint {
            min: 0.0,
            max: 0.0,
            rms: 0.0,
        }]
    } else {
        samples
            .chunks(bucket_size)
            .map(|chunk| {
                let mut min = 1.0_f32;
                let mut max = -1.0_f32;
                let mut energy = 0.0_f32;
                for sample in chunk {
                    let sample = sample.clamp(-1.0, 1.0);
                    min = min.min(sample);
                    max = max.max(sample);
                    energy += sample * sample;
                }
                WaveformPoint {
                    min,
                    max,
                    rms: (energy / chunk.len().max(1) as f32).sqrt().clamp(0.0, 1.0),
                }
            })
            .collect()
    };
    let duration_seconds = if sample_rate == 0 {
        0.0
    } else {
        samples.len() as f32 / sample_rate as f32
    };
    let points_per_second = if duration_seconds > 0.0 {
        points.len() as f32 / duration_seconds
    } else {
        0.0
    };
    WaveformEnvelope {
        sample_rate,
        duration_seconds,
        points_per_second,
        points,
    }
}

pub fn render_melody(project: MelodyProject) -> RenderedAudio {
    let project = project.normalized();
    let sample_rate = project.sample_rate.unwrap_or(44100);
    let tempo = project.tempo.unwrap_or(120.0);
    let seconds_per_beat = 60.0 / tempo;
    let audible_layers = project.layers.iter().filter(|layer| {
        !layer.muted
            && layer.volume > f32::EPSILON
            && (layer.sound.is_some() || layer.sound_sample.is_some())
    });
    let length_beats = audible_layers
        .clone()
        .flat_map(|layer| layer.notes.iter().map(|note| note.start + note.duration))
        .fold(0.0_f32, f32::max);
    let has_notes = audible_layers.clone().any(|layer| !layer.notes.is_empty());
    let output_channels = if audible_layers.clone().any(|layer| {
        layer.sound.as_ref().is_some_and(|sound| {
            sound.pan.abs() > f32::EPSILON
                || (sound.auto_pan_speed > f32::EPSILON && sound.auto_pan_depth > f32::EPSILON)
        })
    }) {
        2
    } else {
        1
    };
    let tail_seconds = project
        .layers
        .iter()
        .filter(|layer| !layer.muted)
        .filter_map(|layer| {
            layer
                .sound
                .as_ref()
                .map(|sound| sound_tail_seconds(&sound.clone().normalized()))
        })
        .fold(0.0_f32, f32::max);
    let total_seconds = if has_notes {
        aligned_four_beat_length(length_beats) * seconds_per_beat + tail_seconds
    } else {
        0.25
    };
    let total_frames = (total_seconds * sample_rate as f32).ceil() as usize;
    let mut mix = vec![0.0; total_frames.max(1) * output_channels as usize];

    for layer in project.layers {
        if layer.muted || layer.volume <= f32::EPSILON {
            continue;
        }
        let layer_volume = layer.volume.clamp(0.0, 1.0);
        if let Some(sample) = layer.sound_sample {
            mix_sample_layer(
                &mut mix,
                sample,
                layer.notes,
                seconds_per_beat,
                sample_rate,
                layer_volume,
                output_channels,
            );
            continue;
        }
        let Some(layer_sound) = layer.sound.map(|sound| sound.normalized()) else {
            continue;
        };
        for note in layer.notes {
            let frequency = midi_note_to_frequency(note.pitch);
            let hold_seconds = (note.duration * seconds_per_beat).max(0.03);
            let mut params = layer_sound.clone().with_frequency(frequency);
            params.sample_rate = sample_rate;
            let rendered = render_sound(params, hold_seconds);
            let start = (note.start * seconds_per_beat * sample_rate as f32).max(0.0) as usize;
            let velocity = note.velocity.unwrap_or(0.85).clamp(0.0, 1.0);
            let rendered_channels = rendered.channels.max(1) as usize;
            for (frame_index, frame) in rendered.samples.chunks(rendered_channels).enumerate() {
                let target = start + frame_index;
                if target >= total_frames {
                    break;
                }
                if output_channels == 2 {
                    let left = frame.first().copied().unwrap_or(0.0);
                    let right = frame.get(1).copied().unwrap_or(left);
                    let base = target * 2;
                    mix[base] += left * velocity * layer_volume * 0.55;
                    mix[base + 1] += right * velocity * layer_volume * 0.55;
                } else {
                    let sample = frame.iter().sum::<f32>() / frame.len().max(1) as f32;
                    if target < mix.len() {
                        mix[target] += sample * velocity * layer_volume * 0.55;
                    }
                }
            }
        }
    }

    for sample in &mut mix {
        *sample = sample.tanh().clamp(-1.0, 1.0);
    }

    RenderedAudio {
        samples: mix,
        sample_rate,
        channels: output_channels,
    }
}

fn aligned_four_beat_length(length_beats: f32) -> f32 {
    if length_beats <= f32::EPSILON {
        0.0
    } else {
        (length_beats / 4.0).ceil() * 4.0
    }
}

fn sound_tail_seconds(sound: &SynthParams) -> f32 {
    (sound.release + sound.delay_time * sound.delay_mix.max(0.0) + 0.08).clamp(0.0, 10.0)
}

impl AudioSample {
    pub(crate) fn normalized(mut self) -> Self {
        self.sample_rate = self.sample_rate.clamp(8000, 192000);
        self.samples = self
            .samples
            .into_iter()
            .filter(|sample| sample.is_finite())
            .map(|sample| sample.clamp(-1.0, 1.0))
            .take(self.sample_rate as usize * 30)
            .collect();
        self
    }
}

fn mix_sample_layer(
    mix: &mut [f32],
    sample: AudioSample,
    notes: Vec<MelodyNote>,
    seconds_per_beat: f32,
    target_sample_rate: u32,
    layer_volume: f32,
    output_channels: u16,
) {
    let sample = sample.normalized();
    if sample.samples.is_empty() {
        return;
    }
    for note in notes {
        let start = (note.start * seconds_per_beat * target_sample_rate as f32).max(0.0) as usize;
        let max_len =
            (note.duration * seconds_per_beat * target_sample_rate as f32).max(1.0) as usize;
        let velocity = note.velocity.unwrap_or(0.85).clamp(0.0, 1.0);
        for index in 0..max_len {
            let source = index as f32 * sample.sample_rate as f32 / target_sample_rate as f32;
            let source_index = source.floor() as usize;
            if source_index >= sample.samples.len() {
                break;
            }
            let target = start + index;
            let frame_count = mix.len() / output_channels as usize;
            if target >= frame_count {
                break;
            }
            let value = sample.samples[source_index] * velocity * layer_volume * 0.65;
            if output_channels == 2 {
                let base = target * 2;
                mix[base] += value;
                mix[base + 1] += value;
            } else {
                mix[target] += value;
            }
        }
    }
}

#[cfg(test)]
pub fn default_layer_sound(layer: u32) -> SynthParams {
    let mut params = SynthParams::default();
    match ((layer.saturating_sub(1)) % 4) + 1 {
        2 => {
            params.waveform = "triangle".to_string();
            params.sub_waveform = "square".to_string();
            params.sub_level = 0.45;
            params.filter_type = "lowpass".to_string();
            params.filter_cutoff = 900.0;
            params.attack = 0.01;
            params.decay = 0.18;
            params.sustain = 0.7;
            params.release = 0.12;
        }
        3 => {
            params.waveform = "square".to_string();
            params.pulse_width = 0.25;
            params.bit_depth = 8;
            params.sample_rate_reduction = 2;
            params.attack = 0.0;
            params.decay = 0.08;
            params.sustain = 0.2;
            params.release = 0.08;
        }
        4 => {
            params.waveform = "noise".to_string();
            params.noise_color = "white".to_string();
            params.filter_type = "highpass".to_string();
            params.filter_cutoff = 2500.0;
            params.attack = 0.0;
            params.decay = 0.09;
            params.sustain = 0.0;
            params.release = 0.05;
        }
        _ => {
            params.waveform = "square".to_string();
            params.pulse_width = 0.5;
            params.delay_time = 0.12;
            params.delay_feedback = 0.18;
            params.delay_mix = 0.12;
        }
    }
    params.normalized()
}

pub fn midi_note_to_frequency(note: u8) -> f32 {
    440.0 * f32::powf(2.0, (note as f32 - 69.0) / 12.0)
}

pub fn frequency_to_midi_note(frequency: f32) -> u8 {
    let safe_frequency = frequency.clamp(20.0, 20000.0);
    (69.0 + 12.0 * (safe_frequency / 440.0).log2())
        .round()
        .clamp(0.0, 127.0) as u8
}

fn normalize_note(mut note: MelodyNote) -> Option<MelodyNote> {
    if !note.start.is_finite() || !note.duration.is_finite() || note.duration <= 0.0 {
        return None;
    }
    note.start = note.start.max(0.0);
    note.duration = note.duration.clamp(0.0625, 64.0);
    note.pitch = note.pitch.clamp(12, 108);
    note.velocity = Some(note.velocity.unwrap_or(0.85).clamp(0.0, 1.0));
    Some(note)
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn default_layer_volume() -> f32 {
    1.0
}

fn is_default_layer_volume(value: &f32) -> bool {
    (*value - 1.0).abs() <= f32::EPSILON
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_sound_produces_samples() {
        let rendered = render_sound(SynthParams::default(), 0.2);
        assert!(rendered.samples.len() > 1000);
        assert!(rendered.samples.iter().all(|sample| sample.is_finite()));
    }

    #[test]
    fn waveform_analysis_preserves_min_max_and_rms() {
        let mut samples = Vec::new();
        for index in 0..1024 {
            samples.push(if index % 2 == 0 { -1.0 } else { 1.0 });
        }
        let waveform = analyze_waveform(&samples, 1024, 512);
        assert_eq!(waveform.points.len(), 512);
        assert_eq!(waveform.duration_seconds, 1.0);
        assert!(waveform.points.iter().any(|point| point.min < -0.9));
        assert!(waveform.points.iter().any(|point| point.max > 0.9));
        assert!(waveform.points.iter().all(|point| point.rms >= 0.0));
    }

    #[test]
    fn render_melody_mixes_layers() {
        let project = MelodyProject {
            name: Some("Test".to_string()),
            description: Some("Renderer test melody".to_string()),
            tempo: Some(120.0),
            sample_rate: Some(44100),
            layers: vec![
                MelodyLayer {
                    id: 1,
                    name: None,
                    sound: Some(default_layer_sound(1)),
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
                },
                MelodyLayer {
                    id: 2,
                    name: None,
                    sound: Some(default_layer_sound(2)),
                    muted: false,
                    volume: 1.0,
                    sound_sample: None,
                    sound_file_path: None,
                    sound_label: None,
                    melody_preset_id: None,
                    notes: vec![MelodyNote {
                        pitch: 48,
                        start: 0.0,
                        duration: 1.0,
                        velocity: Some(0.8),
                    }],
                },
            ],
        };
        let rendered = render_melody(project);
        assert!(rendered.samples.len() > 10000);
        assert!(rendered.samples.iter().any(|sample| sample.abs() > 0.01));
    }

    #[test]
    fn render_melody_mixes_loaded_audio_samples() {
        let project = MelodyProject {
            name: Some("Sample".to_string()),
            description: None,
            tempo: Some(120.0),
            sample_rate: Some(44100),
            layers: vec![MelodyLayer {
                id: 1,
                name: None,
                sound: None,
                muted: false,
                volume: 1.0,
                sound_sample: Some(AudioSample {
                    samples: vec![0.5; 22050],
                    sample_rate: 44100,
                }),
                sound_file_path: Some("kick.wav".to_string()),
                sound_label: Some("kick.wav".to_string()),
                melody_preset_id: Some("kick_pattern".to_string()),
                notes: vec![MelodyNote {
                    pitch: 60,
                    start: 0.0,
                    duration: 0.5,
                    velocity: Some(1.0),
                }],
            }],
        };
        let rendered = render_melody(project);
        assert!(rendered.samples.iter().any(|sample| sample.abs() > 0.1));
    }

    #[test]
    fn render_melody_extends_to_next_four_beat_grid_boundary() {
        let project = MelodyProject {
            name: Some("Tight End".to_string()),
            description: None,
            tempo: Some(60.0),
            sample_rate: Some(8000),
            layers: vec![MelodyLayer {
                id: 1,
                name: None,
                sound: None,
                muted: false,
                volume: 1.0,
                sound_sample: Some(AudioSample {
                    samples: vec![0.5; 8000],
                    sample_rate: 8000,
                }),
                sound_file_path: Some("tone.wav".to_string()),
                sound_label: Some("tone.wav".to_string()),
                melody_preset_id: None,
                notes: vec![MelodyNote {
                    pitch: 60,
                    start: 0.0,
                    duration: 1.0,
                    velocity: Some(1.0),
                }],
            }],
        };
        let rendered = render_melody(project);
        assert_eq!(rendered.samples.len(), 32000);
        assert!(rendered.samples[7999].abs() > 0.1);
        assert!(rendered.samples[8000..]
            .iter()
            .all(|sample| sample.abs() <= f32::EPSILON));
    }

    #[test]
    fn render_melody_keeps_long_synth_tails() {
        let mut sound = default_layer_sound(1);
        sound.sample_rate = 8000;
        sound.release = 0.4;
        sound.delay_time = 1.0;
        sound.delay_mix = 0.8;
        let rendered = render_melody(MelodyProject {
            name: Some("Long Tail".to_string()),
            description: None,
            tempo: Some(60.0),
            sample_rate: Some(8000),
            layers: vec![MelodyLayer {
                id: 1,
                name: None,
                sound: Some(sound),
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
                    velocity: Some(1.0),
                }],
            }],
        });
        assert!(rendered.samples.len() > (4.18 * 8000.0) as usize);
        assert!(rendered.samples.len() >= (5.2 * 8000.0) as usize);
    }

    #[test]
    fn render_melody_skips_layers_without_loaded_sounds() {
        let project = MelodyProject {
            name: Some("Silent".to_string()),
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
                    velocity: Some(1.0),
                }],
            }],
        };
        let rendered = render_melody(project);
        assert!(rendered
            .samples
            .iter()
            .all(|sample| sample.abs() <= f32::EPSILON));
    }

    #[test]
    fn normalization_preserves_large_layer_ids() {
        let project = MelodyProject {
            name: None,
            description: None,
            tempo: Some(120.0),
            sample_rate: Some(44100),
            layers: vec![MelodyLayer {
                id: 512,
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
                    velocity: None,
                }],
            }],
        };
        let normalized = project.normalized();
        assert_eq!(normalized.layers[0].id, 512);
        assert!(normalized.layers[0].sound.is_none());
    }

    #[test]
    fn render_melody_skips_muted_layers() {
        let project = MelodyProject {
            name: Some("Muted".to_string()),
            description: None,
            tempo: Some(120.0),
            sample_rate: Some(44100),
            layers: vec![MelodyLayer {
                id: 1,
                name: None,
                sound: Some(default_layer_sound(1)),
                muted: true,
                volume: 1.0,
                sound_sample: None,
                sound_file_path: None,
                sound_label: None,
                melody_preset_id: None,
                notes: vec![MelodyNote {
                    pitch: 60,
                    start: 0.0,
                    duration: 1.0,
                    velocity: Some(1.0),
                }],
            }],
        };
        let rendered = render_melody(project);
        assert!(rendered
            .samples
            .iter()
            .all(|sample| sample.abs() <= f32::EPSILON));
    }

    #[test]
    fn render_melody_applies_layer_volume() {
        let base_layer = MelodyLayer {
            id: 1,
            name: None,
            sound: None,
            muted: false,
            volume: 1.0,
            sound_sample: Some(AudioSample {
                samples: vec![0.5; 8000],
                sample_rate: 8000,
            }),
            sound_file_path: Some("tone.wav".to_string()),
            sound_label: Some("tone.wav".to_string()),
            melody_preset_id: None,
            notes: vec![MelodyNote {
                pitch: 60,
                start: 0.0,
                duration: 1.0,
                velocity: Some(1.0),
            }],
        };
        let full = render_melody(MelodyProject {
            name: None,
            description: None,
            tempo: Some(60.0),
            sample_rate: Some(8000),
            layers: vec![base_layer.clone()],
        });
        let mut quiet_layer = base_layer;
        quiet_layer.volume = 0.25;
        let quiet = render_melody(MelodyProject {
            name: None,
            description: None,
            tempo: Some(60.0),
            sample_rate: Some(8000),
            layers: vec![quiet_layer],
        });
        let full_peak = full
            .samples
            .iter()
            .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
        let quiet_peak = quiet
            .samples
            .iter()
            .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
        assert!(quiet_peak < full_peak * 0.35);
    }

    #[test]
    fn render_melody_preserves_stereo_pan_from_layer_sound() {
        let mut sound = default_layer_sound(1);
        sound.pan = -0.8;
        let rendered = render_melody(MelodyProject {
            name: Some("Stereo Layer".to_string()),
            description: None,
            tempo: Some(120.0),
            sample_rate: Some(44100),
            layers: vec![MelodyLayer {
                id: 1,
                name: None,
                sound: Some(sound),
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
                    velocity: Some(1.0),
                }],
            }],
        });
        assert_eq!(rendered.channels, 2);
        let left_peak = rendered
            .samples
            .chunks_exact(2)
            .map(|frame| frame[0].abs())
            .fold(0.0_f32, f32::max);
        let right_peak = rendered
            .samples
            .chunks_exact(2)
            .map(|frame| frame[1].abs())
            .fold(0.0_f32, f32::max);
        assert!(left_peak > right_peak * 1.5);
    }
}
