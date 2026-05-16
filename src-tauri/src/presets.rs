use crate::audio::{AudioSample, MelodyProject};
use crate::synth::SynthParams;
use rodio::{Decoder, Source};
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};

#[derive(serde::Serialize)]
pub struct PresetSummary {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[tauri::command]
pub fn list_sound_presets() -> Result<Vec<PresetSummary>, String> {
    list_presets("sounds")
}

#[tauri::command]
pub fn load_sound_preset(name: String) -> Result<SynthParams, String> {
    let path = preset_path("sounds", &name)?;
    load_sound_preset_from_path(path)
}

#[tauri::command]
pub fn load_sound_preset_file(path: String) -> Result<SynthParams, String> {
    let path = PathBuf::from(path);
    if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
        return Err("Sound preset file must be JSON".to_string());
    }
    load_sound_preset_from_path(path)
}

#[tauri::command]
pub fn load_audio_sample_file(path: String) -> Result<AudioSample, String> {
    let path = resolve_audio_sample_path(&path)?;
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .ok_or_else(|| "Audio sample file must have an extension".to_string())?;
    if !matches!(extension.as_str(), "wav" | "mp3" | "ogg" | "flac") {
        return Err("Audio sample file must be WAV, MP3, OGG or FLAC".to_string());
    }
    let file = fs::File::open(path).map_err(|error| error.to_string())?;
    let decoder = Decoder::new(BufReader::new(file)).map_err(|error| error.to_string())?;
    let sample_rate = decoder.sample_rate();
    let channels = decoder.channels().max(1) as usize;
    let raw: Vec<f32> = decoder.convert_samples::<f32>().collect();
    let samples = raw
        .chunks(channels)
        .map(|chunk| chunk.iter().sum::<f32>() / chunk.len().max(1) as f32)
        .collect();
    Ok(AudioSample {
        samples,
        sample_rate,
    }
    .normalized())
}

fn resolve_audio_sample_path(path: &str) -> Result<PathBuf, String> {
    let input = PathBuf::from(path);
    if input.is_absolute() && input.exists() {
        return Ok(input);
    }
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let candidates = [
        current_dir.join(&input),
        presets_root().join(&input),
        presets_root().join("rendered-sounds").join(&input),
        current_dir.join("exports").join("sounds").join(&input),
        current_dir
            .join("..")
            .join("exports")
            .join("sounds")
            .join(&input),
    ];
    candidates
        .into_iter()
        .find(|candidate| candidate.exists())
        .ok_or_else(|| format!("Audio sample file not found: {}", path))
}

#[tauri::command]
pub fn default_preset_dir(kind: String) -> Result<String, String> {
    let kind = match kind.as_str() {
        "sounds" => "sounds",
        "melody-workflows" => "melody-workflows",
        "melody-presets" => "melody-presets",
        "rendered-sounds" => "rendered-sounds",
        _ => return Err("Unknown preset directory".to_string()),
    };
    let dir = presets_root().join(kind);
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let dir = dir.canonicalize().unwrap_or(dir);
    Ok(dir.to_string_lossy().to_string())
}

fn load_sound_preset_from_path(path: PathBuf) -> Result<SynthParams, String> {
    let text = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let params: SynthParams = serde_json::from_str(&text).map_err(|error| error.to_string())?;
    Ok(params.normalized())
}

#[tauri::command]
pub fn save_sound_preset_file(path: String, params: SynthParams) -> Result<PresetSummary, String> {
    let path = ensure_json_path(PathBuf::from(path));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let params = params.normalized();
    let text = serde_json::to_string_pretty(&params).map_err(|error| error.to_string())?;
    fs::write(&path, text).map_err(|error| error.to_string())?;
    let id = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("custom_sound")
        .to_string();
    let (name, description) = read_preset_metadata(&path, "sounds")
        .unwrap_or_else(|| (title_from_id(&id), String::new()));
    Ok(PresetSummary {
        id,
        name,
        description,
    })
}

#[tauri::command]
pub fn list_melody_presets() -> Result<Vec<PresetSummary>, String> {
    list_presets("melody-workflows")
}

#[tauri::command]
pub fn load_melody_preset(name: String) -> Result<MelodyProject, String> {
    let path = preset_path("melody-workflows", &name)?;
    load_melody_project_from_path(path)
}

#[tauri::command]
pub fn load_melody_preset_file(path: String) -> Result<MelodyProject, String> {
    let path = PathBuf::from(path);
    if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
        return Err("Melody workflow file must be JSON".to_string());
    }
    load_melody_project_from_path(path)
}

#[tauri::command]
pub fn save_melody_preset_file(
    path: String,
    project: MelodyProject,
) -> Result<PresetSummary, String> {
    let path = ensure_json_path(PathBuf::from(path));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let mut project = project.normalized();
    let id = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("melody_workflow")
        .to_string();
    if project.name.as_deref().unwrap_or("").trim().is_empty()
        || project.name.as_deref() == Some("Current Melody")
    {
        project.name = Some(title_from_id(&id));
    }
    if project
        .description
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        project.description = Some(format!("{} melody workflow", title_from_id(&id)));
    }
    let text = serde_json::to_string_pretty(&project).map_err(|error| error.to_string())?;
    fs::write(&path, text).map_err(|error| error.to_string())?;
    Ok(PresetSummary {
        id,
        name: project
            .name
            .unwrap_or_else(|| "Custom Melody Workflow".to_string()),
        description: project.description.unwrap_or_default(),
    })
}

#[tauri::command]
pub fn list_melody_layer_presets() -> Result<Vec<PresetSummary>, String> {
    list_presets("melody-presets")
}

#[tauri::command]
pub fn load_melody_layer_preset(name: String) -> Result<MelodyProject, String> {
    let path = preset_path("melody-presets", &name)?;
    load_melody_project_from_path(path)
}

fn load_melody_project_from_path(path: PathBuf) -> Result<MelodyProject, String> {
    let text = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let project: MelodyProject = serde_json::from_str(&text).map_err(|error| error.to_string())?;
    Ok(project.normalized())
}

#[tauri::command]
pub fn save_melody_preset(
    filename: String,
    project: MelodyProject,
    description: String,
) -> Result<PresetSummary, String> {
    let mut project = project.normalized();
    let id = safe_id(&filename);
    if id.is_empty() {
        return Err("Preset filename cannot be empty".to_string());
    }
    let dir = presets_root().join("melody-workflows");
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    project.name = Some(project.name.unwrap_or_else(|| title_from_id(&id)));
    project.description = Some(if description.trim().is_empty() {
        "Custom melody preset".to_string()
    } else {
        description.trim().to_string()
    });
    let path = dir.join(format!("{}.json", id));
    let text = serde_json::to_string_pretty(&project).map_err(|error| error.to_string())?;
    fs::write(&path, text).map_err(|error| error.to_string())?;
    Ok(PresetSummary {
        id,
        name: project.name.unwrap_or_else(|| "Custom Melody".to_string()),
        description: project
            .description
            .unwrap_or_else(|| "Custom melody preset".to_string()),
    })
}

fn list_presets(kind: &str) -> Result<Vec<PresetSummary>, String> {
    let dir = presets_root().join(kind);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut presets = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("json") {
            let id = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or_default()
                .to_string();
            let (name, description) = read_preset_metadata(&path, kind)
                .unwrap_or_else(|| (title_from_id(&id), default_description(kind, &id)));
            presets.push(PresetSummary {
                id,
                name,
                description,
            });
        }
    }
    if kind == "sounds" {
        presets.sort_by_key(|preset| sound_sort_key(&preset.id));
    } else {
        presets.sort_by(|left, right| left.id.cmp(&right.id));
    }
    Ok(presets)
}

fn read_preset_metadata(path: &Path, kind: &str) -> Option<(String, String)> {
    let text = fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    let id = path.file_stem().and_then(|stem| stem.to_str())?;
    let fallback_name = if kind == "sounds" {
        derived_sound_name(id, &value).unwrap_or_else(|| title_from_id(id))
    } else {
        title_from_id(id)
    };
    let name = value
        .get("name")
        .and_then(|name| name.as_str())
        .unwrap_or(&fallback_name)
        .to_string();
    let description = value
        .get("description")
        .and_then(|description| description.as_str())
        .map(|description| description.to_string())
        .unwrap_or_else(|| default_description(kind, id));
    Some((name, description))
}

fn preset_path(kind: &str, name: &str) -> Result<PathBuf, String> {
    let id = safe_id(name);
    let path = presets_root().join(kind).join(format!("{}.json", id));
    if path.exists() {
        Ok(path)
    } else {
        Err(format!("Preset not found: {}", id))
    }
}

fn presets_root() -> PathBuf {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let direct = current_dir.join("presets");
    if direct.exists() {
        return direct;
    }
    let parent = current_dir.join("..").join("presets");
    if parent.exists() {
        return parent;
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let portable = dir.join("presets");
            if portable.exists() {
                return portable;
            }
            let bundled = dir.join("resources").join("presets");
            if bundled.exists() {
                return bundled;
            }
            let adjacent_resources = dir.join("..").join("resources").join("presets");
            if adjacent_resources.exists() {
                return adjacent_resources;
            }
        }
    }
    direct
}

fn ensure_json_path(path: PathBuf) -> PathBuf {
    if path.extension().and_then(|extension| extension.to_str()) == Some("json") {
        path
    } else {
        path.with_extension("json")
    }
}

fn safe_id(name: &str) -> String {
    name.chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || *character == '_' || *character == '-'
        })
        .collect()
}

fn title_from_id(id: &str) -> String {
    id.replace('_', " ")
        .split_whitespace()
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn default_description(kind: &str, id: &str) -> String {
    if kind == "sounds" {
        String::new()
    } else if kind == "melody-workflows" {
        format!("Bundled melody workflow {}", id)
    } else {
        format!("Bundled melody preset {}", id)
    }
}

fn derived_sound_name(id: &str, value: &serde_json::Value) -> Option<String> {
    if let Some((family, number)) = named_sound_file(id) {
        return Some(format!("{} {}", family, number));
    }
    if let Some(number) = preset_number(id) {
        let (family, local_number) = match number {
            1..=15 => ("Lead", number),
            16..=25 => ("Bass", number - 15),
            26..=35 => ("Pluck", number - 25),
            36..=45 => ("Arcade SFX", number - 35),
            46..=50 => ("Noise Drum", number - 45),
            _ => ("Sound", number),
        };
        return Some(format!("{} {}", family, local_number));
    }
    let params: SynthParams = serde_json::from_value::<SynthParams>(value.clone()).ok()?;
    let family = if params.waveform == "noise" {
        "Noise SFX"
    } else if params.frequency < 140.0 {
        "Bass"
    } else if params.sustain <= 0.05 && params.decay <= 0.25 {
        "Pluck"
    } else if params.sweep_amount.abs() > 8.0 {
        "Arcade SFX"
    } else {
        "Lead"
    };
    Some(family.to_string())
}

fn named_sound_file(id: &str) -> Option<(&'static str, u32)> {
    let (prefix, number) = id.rsplit_once('_')?;
    let number = number.parse::<u32>().ok()?;
    let family = match prefix {
        "lead" => "Lead",
        "bass" => "Bass",
        "pluck" => "Pluck",
        "arcade_sfx" => "Arcade SFX",
        "noise_drum" => "Noise Drum",
        _ => return None,
    };
    Some((family, number))
}

fn sound_sort_key(id: &str) -> (u8, u32, String) {
    if let Some((family, number)) = named_sound_file(id) {
        let group = match family {
            "Lead" => 0,
            "Bass" => 1,
            "Pluck" => 2,
            "Arcade SFX" => 3,
            "Noise Drum" => 4,
            _ => 9,
        };
        return (group, number, id.to_string());
    }
    if let Some(number) = preset_number(id) {
        return match number {
            1..=15 => (0, number, id.to_string()),
            16..=25 => (1, number - 15, id.to_string()),
            26..=35 => (2, number - 25, id.to_string()),
            36..=45 => (3, number - 35, id.to_string()),
            46..=50 => (4, number - 45, id.to_string()),
            _ => (9, number, id.to_string()),
        };
    }
    (9, u32::MAX, id.to_string())
}

fn preset_number(id: &str) -> Option<u32> {
    id.rsplit('_')
        .next()
        .and_then(|number| number.parse::<u32>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_id_keeps_expected_characters() {
        assert_eq!(safe_id("../preset_01.json"), "preset_01json");
    }

    #[test]
    fn title_from_id_is_readable() {
        assert_eq!(title_from_id("preset_01"), "Preset 01");
    }

    #[test]
    fn read_preset_metadata_uses_json_description() {
        let path = std::env::temp_dir().join("retro_sound_studio_metadata_test.json");
        fs::write(
            &path,
            r#"{"name":"Test Melody","description":"Readable preset description"}"#,
        )
        .unwrap();
        let metadata = read_preset_metadata(&path, "melody-workflows").unwrap();
        assert_eq!(metadata.0, "Test Melody");
        assert_eq!(metadata.1, "Readable preset description");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn sound_metadata_is_derived_from_synth_params() {
        let value: serde_json::Value = serde_json::from_str(
            r#"{
            "waveform":"square",
            "sub_waveform":"square",
            "pulse_width":0.5,
            "noise_color":"white",
            "sub_level":0.5,
            "filter_type":"lowpass",
            "filter_cutoff":900,
            "filter_resonance":0.2,
            "filter_env_depth":0.4,
            "frequency":80,
            "sweep_amount":0,
            "sweep_time":0,
            "portamento":0,
            "attack":0.01,
            "decay":0.2,
            "sustain":0.7,
            "release":0.2,
            "filter_attack":0.01,
            "filter_decay":0.2,
            "filter_sustain":0.5,
            "filter_release":0.2,
            "lfo1_waveform":"sine",
            "lfo1_speed":0,
            "lfo1_depth":0,
            "lfo1_routing":"none",
            "lfo2_waveform":"sine",
            "lfo2_speed":0,
            "lfo2_depth":0,
            "lfo2_routing":"none",
            "delay_time":0,
            "delay_feedback":0,
            "delay_mix":0,
            "bit_depth":16,
            "sample_rate_reduction":1,
            "arp_chord":"none",
            "arp_speed":0,
            "sample_rate":44100
        }"#,
        )
        .unwrap();
        assert_eq!(derived_sound_name("preset_16", &value).unwrap(), "Bass 1");
        assert_eq!(derived_sound_name("bass_11", &value).unwrap(), "Bass 11");
        assert_eq!(default_description("sounds", "preset_16"), "");
    }

    #[test]
    fn save_sound_preset_file_writes_json_file() {
        let path = std::env::temp_dir().join("retro_sound_studio_saved_sound_test");
        let saved =
            save_sound_preset_file(path.to_string_lossy().to_string(), SynthParams::default())
                .unwrap();
        let written_path = path.with_extension("json");
        assert!(written_path.exists());
        assert_eq!(saved.id, "retro_sound_studio_saved_sound_test");
        let params: SynthParams =
            serde_json::from_str(&fs::read_to_string(&written_path).unwrap()).unwrap();
        assert_eq!(params.sample_rate, 44100);
        let _ = fs::remove_file(written_path);
    }

    #[test]
    fn save_melody_preset_file_preserves_embedded_sound_metadata() {
        let path = std::env::temp_dir().join("retro_sound_studio_saved_workflow_test.json");
        let project = MelodyProject {
            name: Some("Saved Workflow".to_string()),
            description: None,
            tempo: Some(120.0),
            sample_rate: Some(44100),
            layers: vec![crate::audio::MelodyLayer {
                id: 1,
                name: Some("Lead".to_string()),
                sound: Some(SynthParams::default()),
                muted: false,
                volume: 1.0,
                sound_sample: None,
                sound_file_path: Some("".to_string()),
                sound_label: Some("Lead 1".to_string()),
                melody_preset_id: Some("lead_phrase".to_string()),
                notes: vec![crate::audio::MelodyNote {
                    pitch: 60,
                    start: 0.0,
                    duration: 1.0,
                    velocity: Some(0.8),
                }],
            }],
        };
        let saved = save_melody_preset_file(path.to_string_lossy().to_string(), project).unwrap();
        assert_eq!(saved.id, "retro_sound_studio_saved_workflow_test");
        let loaded = load_melody_preset_file(path.to_string_lossy().to_string()).unwrap();
        assert!(loaded.layers[0].sound.is_some());
        assert_eq!(loaded.layers[0].sound_label.as_deref(), Some("Lead 1"));
        assert_eq!(loaded.layers[0].sound_file_path, None);
        assert_eq!(
            loaded.layers[0].melody_preset_id.as_deref(),
            Some("lead_phrase")
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    fn relative_rendered_sound_samples_can_be_loaded() {
        if presets_root()
            .join("rendered-sounds")
            .join("lead_01.wav")
            .exists()
        {
            let sample = load_audio_sample_file("rendered-sounds/lead_01.wav".to_string()).unwrap();
            assert_eq!(sample.sample_rate, 44100);
            assert!(sample.samples.len() > 1000);
            assert!(sample.samples.iter().any(|sample| sample.abs() > 0.01));
        }
    }

    #[test]
    fn bundled_melody_workflows_are_clean_or_reference_rendered_sound_files() {
        let dir = presets_root().join("melody-workflows");
        let entries = fs::read_dir(dir).unwrap();
        let mut checked = 0;
        let melody_preset_ids = list_presets("melody-presets")
            .unwrap()
            .into_iter()
            .map(|preset| preset.id)
            .collect::<std::collections::HashSet<_>>();
        for entry in entries {
            let path = entry.unwrap().path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
                continue;
            }
            let text = fs::read_to_string(path).unwrap();
            let project: MelodyProject = serde_json::from_str(&text).unwrap();
            assert!(
                project.layers.len() <= 3,
                "workflow should stay compact and coherent"
            );
            for layer in project.layers {
                assert!(
                    layer.notes.len() <= 8,
                    "workflow layer has too many notes for a compact starter melody"
                );
                let sound_path = layer.sound_file_path.unwrap_or_default();
                assert!(
                    sound_path.starts_with("rendered-sounds/"),
                    "missing rendered sound path for layer {:?}",
                    layer.name
                );
                assert!(
                    sound_path.ends_with(".wav"),
                    "rendered sound path must point to a WAV file: {}",
                    sound_path
                );
                assert!(presets_root().join(&sound_path).exists());
                assert_eq!(
                    layer.sound_label.as_deref(),
                    sound_path.rsplit('/').next(),
                    "sound label must display the real WAV filename"
                );
                let melody_preset_id = layer.melody_preset_id.as_deref().unwrap_or_default();
                assert!(
                    melody_preset_ids.contains(melody_preset_id),
                    "workflow layer references missing melody preset: {}",
                    melody_preset_id
                );
                assert!(layer.sound.is_none());
            }
            checked += 1;
        }
        assert_eq!(checked, list_presets("melody-workflows").unwrap().len());
    }

    #[test]
    fn bundled_starter_pack_contains_json_and_rendered_audio() {
        let sounds = list_presets("sounds").unwrap();
        let melody_presets = list_presets("melody-presets").unwrap();
        let workflows = list_presets("melody-workflows").unwrap();
        assert!(sounds.len() >= 44);
        assert!(melody_presets.len() >= 48);
        assert!(workflows.len() >= 8);
        for preset in sounds {
            assert!(
                presets_root()
                    .join("rendered-sounds")
                    .join(format!("{}.wav", preset.id))
                    .exists(),
                "missing rendered WAV for sound preset {}",
                preset.id
            );
            assert!(!preset.name.trim().is_empty());
            assert!(!preset.description.trim().is_empty());
        }
    }

    #[test]
    fn bundled_melody_workflows_render_audible_audio_from_wavs() {
        let dir = presets_root().join("melody-workflows");
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
                continue;
            }
            let mut project = load_melody_project_from_path(path.clone()).unwrap();
            for layer in &mut project.layers {
                let sound_path = layer.sound_file_path.clone().unwrap();
                layer.sound_sample = Some(load_audio_sample_file(sound_path).unwrap());
            }
            let rendered = crate::audio::render_melody(project);
            assert!(
                rendered.samples.iter().any(|sample| sample.abs() > 0.02),
                "workflow rendered silence: {}",
                path.display()
            );
            assert!(
                rendered.samples.len() > rendered.sample_rate as usize,
                "workflow is too short: {}",
                path.display()
            );
        }
    }
}
