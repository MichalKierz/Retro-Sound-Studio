#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod audio;
mod app_state;
mod export;
mod presets;
mod synth;

use app_state::{configure_portable_webview_data, load_app_state, save_app_state};
use audio::{
    analyze_waveform, render_melody, render_sound, MelodyProject, RenderedAudio, WaveformEnvelope,
};
use export::{default_export_dir, export_melody, export_sound};
use presets::{
    default_preset_dir, list_melody_layer_presets, list_melody_presets, list_sound_presets,
    load_audio_sample_file, load_melody_layer_preset, load_melody_preset, load_melody_preset_file,
    load_sound_preset, load_sound_preset_file, save_melody_preset, save_melody_preset_file,
    save_sound_preset_file,
};
use rodio::{buffer::SamplesBuffer, OutputStream, OutputStreamHandle, Sink, Source};
use std::sync::Mutex;
use synth::SynthParams;
use tauri::State;

#[derive(Clone)]
struct AudioClip {
    samples: Vec<f32>,
    sample_rate: u32,
    channels: u16,
}

struct AudioState {
    sink: Option<Sink>,
    stream_handle: OutputStreamHandle,
    current_clip: Option<AudioClip>,
    loop_enabled: bool,
    volume: f32,
}

#[derive(serde::Serialize)]
struct PlaybackInfo {
    duration_seconds: f32,
    sample_rate: u32,
    loop_enabled: bool,
    paused: bool,
    start_seconds: f32,
}

#[tauri::command]
fn play_sound(
    params: SynthParams,
    state: State<'_, Mutex<AudioState>>,
) -> Result<PlaybackInfo, String> {
    let rendered = render_sound(params, 0.85);
    replace_playback(rendered, state)
}

#[tauri::command]
fn play_melody(
    project: MelodyProject,
    state: State<'_, Mutex<AudioState>>,
) -> Result<PlaybackInfo, String> {
    let rendered = render_melody(project);
    replace_playback(rendered, state)
}

#[tauri::command]
fn render_sound_waveform(
    params: SynthParams,
    resolution: Option<usize>,
) -> Result<WaveformEnvelope, String> {
    let rendered = render_sound(params, 0.85);
    let samples = rendered.mono_samples();
    Ok(analyze_waveform(
        &samples,
        rendered.sample_rate,
        resolution.unwrap_or(4096),
    ))
}

#[tauri::command]
fn stop_playback(state: State<'_, Mutex<AudioState>>) -> Result<(), String> {
    let mut audio_state = state.lock().map_err(|error| error.to_string())?;
    if let Some(sink) = audio_state.sink.take() {
        sink.stop();
    }
    audio_state.loop_enabled = false;
    Ok(())
}

#[tauri::command]
fn pause_playback(state: State<'_, Mutex<AudioState>>) -> Result<bool, String> {
    let audio_state = state.lock().map_err(|error| error.to_string())?;
    let paused = if let Some(sink) = &audio_state.sink {
        if sink.is_paused() {
            sink.play();
            false
        } else {
            sink.pause();
            true
        }
    } else {
        false
    };
    Ok(paused)
}

#[tauri::command]
fn restart_playback(state: State<'_, Mutex<AudioState>>) -> Result<PlaybackInfo, String> {
    let mut audio_state = state.lock().map_err(|error| error.to_string())?;
    play_current_clip_from(&mut audio_state, 0.0, false)
}

#[tauri::command]
fn seek_playback(
    position_seconds: f32,
    state: State<'_, Mutex<AudioState>>,
) -> Result<PlaybackInfo, String> {
    let mut audio_state = state.lock().map_err(|error| error.to_string())?;
    let clip = audio_state
        .current_clip
        .clone()
        .ok_or_else(|| "No playback to seek".to_string())?;
    let was_paused = audio_state
        .sink
        .as_ref()
        .map(|sink| sink.is_paused())
        .unwrap_or(false);
    let position_seconds = position_seconds.clamp(0.0, clip_duration_seconds(&clip));
    play_current_clip_from(&mut audio_state, position_seconds, was_paused)
}

#[tauri::command]
fn set_loop(
    loop_status: bool,
    position_seconds: Option<f32>,
    state: State<'_, Mutex<AudioState>>,
) -> Result<PlaybackInfo, String> {
    let mut audio_state = state.lock().map_err(|error| error.to_string())?;
    let was_paused = audio_state
        .sink
        .as_ref()
        .map(|sink| sink.is_paused())
        .unwrap_or(false);
    let was_looping = audio_state.loop_enabled;
    let should_rebuild =
        audio_state.current_clip.is_some() && (audio_state.sink.is_some() || loop_status);
    audio_state.loop_enabled = loop_status;
    if !loop_status && was_looping {
        if let Some(sink) = audio_state.sink.take() {
            sink.stop();
        }
        let duration_seconds = audio_state
            .current_clip
            .as_ref()
            .map(clip_duration_seconds)
            .unwrap_or(0.0);
        let sample_rate = audio_state
            .current_clip
            .as_ref()
            .map(|clip| clip.sample_rate)
            .unwrap_or(44100);
        return Ok(PlaybackInfo {
            duration_seconds,
            sample_rate,
            loop_enabled: false,
            paused: false,
            start_seconds: 0.0,
        });
    }
    if should_rebuild {
        return play_current_clip_from(
            &mut audio_state,
            position_seconds.unwrap_or(0.0),
            paused_after_loop_rebuild(loop_status, was_paused),
        );
    }
    let duration_seconds = audio_state
        .current_clip
        .as_ref()
        .map(clip_duration_seconds)
        .unwrap_or(0.0);
    let sample_rate = audio_state
        .current_clip
        .as_ref()
        .map(|clip| clip.sample_rate)
        .unwrap_or(44100);
    Ok(PlaybackInfo {
        duration_seconds,
        sample_rate,
        loop_enabled: loop_status,
        paused: audio_state
            .sink
            .as_ref()
            .map(|sink| sink.is_paused())
            .unwrap_or(false),
        start_seconds: 0.0,
    })
}

fn clip_duration_seconds(clip: &AudioClip) -> f32 {
    let channels = clip.channels.max(1) as usize;
    (clip.samples.len() / channels) as f32 / clip.sample_rate as f32
}

fn paused_after_loop_rebuild(loop_status: bool, was_paused: bool) -> bool {
    !loop_status && was_paused
}

#[tauri::command]
fn set_volume(volume: f32, state: State<'_, Mutex<AudioState>>) -> Result<(), String> {
    let mut audio_state = state.lock().map_err(|error| error.to_string())?;
    audio_state.volume = volume.clamp(0.0, 100.0) / 100.0;
    if let Some(sink) = &audio_state.sink {
        sink.set_volume(audio_state.volume);
    }
    Ok(())
}

fn replace_playback(
    rendered: RenderedAudio,
    state: State<'_, Mutex<AudioState>>,
) -> Result<PlaybackInfo, String> {
    let mut audio_state = state.lock().map_err(|error| error.to_string())?;
    let clip = AudioClip {
        samples: rendered.samples,
        sample_rate: rendered.sample_rate,
        channels: rendered.channels.max(1),
    };
    audio_state.loop_enabled = false;
    audio_state.current_clip = Some(clip);
    play_current_clip_from(&mut audio_state, 0.0, false)
}

fn play_current_clip_from(
    audio_state: &mut AudioState,
    position_seconds: f32,
    paused: bool,
) -> Result<PlaybackInfo, String> {
    let clip = audio_state
        .current_clip
        .clone()
        .ok_or_else(|| "No playback loaded".to_string())?;
    if clip.samples.is_empty() {
        return Err("Rendered audio is empty".to_string());
    }
    if let Some(sink) = audio_state.sink.take() {
        sink.stop();
    }
    let sink = Sink::try_new(&audio_state.stream_handle).map_err(|error| error.to_string())?;
    sink.set_volume(audio_state.volume);
    let frame_count = clip.samples.len() / clip.channels as usize;
    let duration_seconds = frame_count as f32 / clip.sample_rate as f32;
    let max_start = frame_count.saturating_sub(1);
    let position_seconds = if duration_seconds > 0.0 && audio_state.loop_enabled {
        position_seconds.rem_euclid(duration_seconds)
    } else {
        position_seconds.clamp(0.0, duration_seconds)
    };
    let start_frame = ((position_seconds * clip.sample_rate as f32) as usize).min(max_start);
    let start = start_frame * clip.channels as usize;
    let remaining = clip.samples[start..].to_vec();
    let source = SamplesBuffer::new(clip.channels, clip.sample_rate, remaining);
    if audio_state.loop_enabled {
        sink.append(source);
        sink.append(
            SamplesBuffer::new(clip.channels, clip.sample_rate, clip.samples.clone())
                .repeat_infinite(),
        );
    } else {
        sink.append(source);
    }
    if paused {
        sink.pause();
    } else {
        sink.play();
    }
    audio_state.sink = Some(sink);
    Ok(PlaybackInfo {
        duration_seconds,
        sample_rate: clip.sample_rate,
        loop_enabled: audio_state.loop_enabled,
        paused,
        start_seconds: position_seconds,
    })
}

fn main() {
    let _ = configure_portable_webview_data();
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    tauri::Builder::default()
        .manage(Mutex::new(AudioState {
            sink: None,
            stream_handle,
            current_clip: None,
            loop_enabled: false,
            volume: 0.05,
        }))
        .invoke_handler(tauri::generate_handler![
            play_sound,
            play_melody,
            render_sound_waveform,
            stop_playback,
            pause_playback,
            restart_playback,
            seek_playback,
            set_loop,
            set_volume,
            load_app_state,
            save_app_state,
            list_sound_presets,
            load_sound_preset,
            load_sound_preset_file,
            load_audio_sample_file,
            save_sound_preset_file,
            default_preset_dir,
            list_melody_presets,
            load_melody_preset,
            load_melody_preset_file,
            list_melody_layer_presets,
            load_melody_layer_preset,
            save_melody_preset,
            save_melody_preset_file,
            export_sound,
            export_melody,
            default_export_dir
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enabling_loop_rebuilds_as_playing() {
        assert!(!paused_after_loop_rebuild(true, true));
        assert!(!paused_after_loop_rebuild(true, false));
        assert!(paused_after_loop_rebuild(false, true));
        assert!(!paused_after_loop_rebuild(false, false));
    }
}
