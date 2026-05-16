use retro_sound_studio::audio::{render_sound, RenderedAudio};
use retro_sound_studio::synth::SynthParams;
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> Result<(), String> {
    let root = workspace_root();
    let sound_dir = root.join("presets").join("sounds");
    let output_dir = root.join("presets").join("rendered-sounds");
    fs::create_dir_all(&output_dir).map_err(|error| error.to_string())?;
    let mut rendered_count = 0;
    for entry in fs::read_dir(&sound_dir).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let text = fs::read_to_string(&path).map_err(|error| error.to_string())?;
        let params: SynthParams = serde_json::from_str(&text).map_err(|error| error.to_string())?;
        let mut rendered = render_sound(params, 1.0);
        normalize_peak(&mut rendered.samples);
        let output = output_dir.join(format!(
            "{}.wav",
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("sound")
        ));
        write_wav(&output, rendered)?;
        rendered_count += 1;
    }
    println!(
        "Rendered {} sound assets to {}",
        rendered_count,
        output_dir.display()
    );
    Ok(())
}

fn workspace_root() -> PathBuf {
    let current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if current.join("presets").join("sounds").exists() {
        current
    } else {
        current.parent().unwrap_or(&current).to_path_buf()
    }
}

fn write_wav(path: &Path, rendered: RenderedAudio) -> Result<(), String> {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: rendered.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec).map_err(|error| error.to_string())?;
    if rendered.channels == 2 {
        for sample in rendered.samples {
            writer
                .write_sample(f32_to_i16(sample))
                .map_err(|error| error.to_string())?;
        }
    } else {
        for sample in rendered.samples {
            let sample = f32_to_i16(sample);
            writer
                .write_sample(sample)
                .map_err(|error| error.to_string())?;
            writer
                .write_sample(sample)
                .map_err(|error| error.to_string())?;
        }
    }
    writer.finalize().map_err(|error| error.to_string())
}

fn f32_to_i16(sample: f32) -> i16 {
    let sample = sample.clamp(-1.0, 1.0);
    if sample >= 0.0 {
        (sample * i16::MAX as f32).round() as i16
    } else {
        (sample * -(i16::MIN as f32)).round() as i16
    }
}

fn normalize_peak(samples: &mut [f32]) {
    let peak = samples
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));
    if peak > 0.0 {
        let gain = 0.95 / peak;
        for sample in samples {
            *sample = (*sample * gain).clamp(-1.0, 1.0);
        }
    }
}
