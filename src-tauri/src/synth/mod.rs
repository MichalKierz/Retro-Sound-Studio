use crate::audio::RenderedAudio;
use rodio::Source;
use std::time::Duration;

pub mod effects;
pub mod envelope;
pub mod filter;
pub mod lfo;
pub mod oscillator;

use effects::Effects;
use envelope::Envelope;
use filter::SVFFilter;
use lfo::Lfo;
use oscillator::Oscillator;

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
pub struct SynthParams {
    pub waveform: String,
    pub sub_waveform: String,
    pub duty_mode: String,
    pub duty_sequence: String,
    pub duty_sequence_rate: f32,
    pub pulse_width: f32,
    pub noise_color: String,
    pub noise_mode: String,
    pub noise_period: u32,
    pub sub_level: f32,

    pub wavetable: String,

    pub filter_type: String,
    pub filter_cutoff: f32,
    pub filter_resonance: f32,
    pub filter_env_depth: f32,

    pub frequency: f32,
    pub sweep_amount: f32,
    pub sweep_time: f32,
    pub portamento: f32,
    pub pitch_env_amount: f32,
    pub pitch_env_decay: f32,
    pub pitch_env_curve: String,

    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,

    pub filter_attack: f32,
    pub filter_decay: f32,
    pub filter_sustain: f32,
    pub filter_release: f32,

    pub lfo1_waveform: String,
    pub lfo1_speed: f32,
    pub lfo1_depth: f32,
    pub lfo1_routing: String,

    pub lfo2_waveform: String,
    pub lfo2_speed: f32,
    pub lfo2_depth: f32,
    pub lfo2_routing: String,

    pub delay_time: f32,
    pub delay_feedback: f32,
    pub delay_mix: f32,

    pub bit_depth: u32,
    pub sample_rate_reduction: u32,
    pub distortion_type: String,
    pub distortion_drive: f32,

    pub arp_chord: String,
    pub arp_speed: f32,

    pub retrigger_rate: f32,
    pub gate_length: f32,

    pub pan: f32,
    pub auto_pan_speed: f32,
    pub auto_pan_depth: f32,

    pub sample_rate: u32,
}

impl Default for SynthParams {
    fn default() -> Self {
        Self {
            waveform: "square".to_string(),
            sub_waveform: "none".to_string(),
            duty_mode: "free".to_string(),
            duty_sequence: "none".to_string(),
            duty_sequence_rate: 0.0,
            pulse_width: 0.35,
            noise_color: "white".to_string(),
            noise_mode: "lfsr".to_string(),
            noise_period: 1,
            sub_level: 0.0,
            wavetable: "none".to_string(),
            filter_type: "none".to_string(),
            filter_cutoff: 20000.0,
            filter_resonance: 0.0,
            filter_env_depth: 0.0,
            frequency: 440.0,
            sweep_amount: 0.0,
            sweep_time: 0.0,
            portamento: 0.0,
            pitch_env_amount: 0.0,
            pitch_env_decay: 0.0,
            pitch_env_curve: "exponential".to_string(),
            attack: 0.01,
            decay: 0.12,
            sustain: 0.55,
            release: 0.18,
            filter_attack: 0.01,
            filter_decay: 0.12,
            filter_sustain: 0.5,
            filter_release: 0.18,
            lfo1_waveform: "sine".to_string(),
            lfo1_speed: 5.0,
            lfo1_depth: 0.0,
            lfo1_routing: "none".to_string(),
            lfo2_waveform: "sine".to_string(),
            lfo2_speed: 1.0,
            lfo2_depth: 0.0,
            lfo2_routing: "none".to_string(),
            delay_time: 0.0,
            delay_feedback: 0.0,
            delay_mix: 0.0,
            bit_depth: 16,
            sample_rate_reduction: 1,
            distortion_type: "none".to_string(),
            distortion_drive: 1.0,
            arp_chord: "none".to_string(),
            arp_speed: 0.0,
            retrigger_rate: 0.0,
            gate_length: 1.0,
            pan: 0.0,
            auto_pan_speed: 0.0,
            auto_pan_depth: 0.0,
            sample_rate: 44100,
        }
    }
}

impl SynthParams {
    pub fn normalized(mut self) -> Self {
        self.waveform = normalize_choice(
            &self.waveform,
            &[
                "square",
                "triangle",
                "sawtooth",
                "sine",
                "noise",
                "wavetable",
            ],
            "square",
        );
        self.sub_waveform = normalize_choice(
            &self.sub_waveform,
            &["none", "square", "triangle", "sawtooth", "sine"],
            "none",
        );
        self.duty_mode =
            normalize_choice(&self.duty_mode, &["free", "12_5", "25", "50", "75"], "free");
        self.duty_sequence = normalize_choice(
            &self.duty_sequence,
            &["none", "classic_steps", "pulse_train", "skewed_ladder"],
            "none",
        );
        self.noise_color =
            normalize_choice(&self.noise_color, &["white", "pink", "brown"], "white");
        self.noise_mode =
            normalize_choice(&self.noise_mode, &["lfsr", "periodic", "metallic"], "lfsr");
        self.wavetable = normalize_choice(
            &self.wavetable,
            &["none", "gb_organ", "gb_bell", "gb_saw", "gb_pulse"],
            "none",
        );
        if self.waveform == "wavetable" && self.wavetable == "none" {
            self.wavetable = "gb_organ".to_string();
        }
        self.filter_type = normalize_choice(
            &self.filter_type,
            &["none", "lowpass", "highpass", "bandpass"],
            "none",
        );
        self.lfo1_waveform = normalize_choice(
            &self.lfo1_waveform,
            &["sine", "square", "triangle", "sawtooth"],
            "sine",
        );
        self.lfo2_waveform = normalize_choice(
            &self.lfo2_waveform,
            &["sine", "square", "triangle", "sawtooth"],
            "sine",
        );
        self.lfo1_routing = normalize_choice(
            &self.lfo1_routing,
            &["none", "pitch", "cutoff", "pwm", "amp"],
            "none",
        );
        self.lfo2_routing = normalize_choice(
            &self.lfo2_routing,
            &["none", "pitch", "cutoff", "pwm", "amp"],
            "none",
        );
        self.arp_chord = normalize_choice(
            &self.arp_chord,
            &["none", "major", "minor", "octave", "fifth"],
            "none",
        );
        self.pitch_env_curve = normalize_choice(
            &self.pitch_env_curve,
            &["linear", "exponential"],
            "exponential",
        );
        self.distortion_type = normalize_choice(
            &self.distortion_type,
            &["none", "hard_clip", "soft_clip", "foldback"],
            "none",
        );
        self.pulse_width = finite_clamp(self.pulse_width, 0.1, 0.9, 0.35);
        self.pulse_width = duty_width(&self.duty_mode).unwrap_or(self.pulse_width);
        self.duty_sequence_rate = finite_clamp(self.duty_sequence_rate, 0.0, 60.0, 0.0);
        self.sub_level = finite_clamp(self.sub_level, 0.0, 1.0, 0.0);
        self.noise_period = self.noise_period.clamp(1, 128);
        self.filter_cutoff = finite_clamp(self.filter_cutoff, 20.0, 20000.0, 20000.0);
        self.filter_resonance = finite_clamp(self.filter_resonance, 0.0, 0.99, 0.0);
        self.filter_env_depth = finite_clamp(self.filter_env_depth, -1.0, 1.0, 0.0);
        self.frequency = finite_clamp(self.frequency, 20.0, 20000.0, 440.0);
        self.sweep_amount = finite_clamp(self.sweep_amount, -72.0, 72.0, 0.0);
        self.sweep_time = finite_clamp(self.sweep_time, 0.0, 8.0, 0.0);
        self.portamento = finite_clamp(self.portamento, 0.0, 4.0, 0.0);
        self.pitch_env_amount = finite_clamp(self.pitch_env_amount, -72.0, 72.0, 0.0);
        self.pitch_env_decay = finite_clamp(self.pitch_env_decay, 0.0, 4.0, 0.0);
        self.attack = finite_clamp(self.attack, 0.0, 8.0, 0.01);
        self.decay = finite_clamp(self.decay, 0.0, 8.0, 0.12);
        self.sustain = finite_clamp(self.sustain, 0.0, 1.0, 0.55);
        self.release = finite_clamp(self.release, 0.0, 8.0, 0.18);
        self.filter_attack = finite_clamp(self.filter_attack, 0.0, 8.0, 0.01);
        self.filter_decay = finite_clamp(self.filter_decay, 0.0, 8.0, 0.12);
        self.filter_sustain = finite_clamp(self.filter_sustain, 0.0, 1.0, 0.5);
        self.filter_release = finite_clamp(self.filter_release, 0.0, 8.0, 0.18);
        self.lfo1_speed = finite_clamp(self.lfo1_speed, 0.0, 80.0, 0.0);
        self.lfo1_depth = finite_clamp(self.lfo1_depth, 0.0, 24.0, 0.0);
        self.lfo2_speed = finite_clamp(self.lfo2_speed, 0.0, 80.0, 0.0);
        self.lfo2_depth = finite_clamp(self.lfo2_depth, 0.0, 24.0, 0.0);
        self.delay_time = finite_clamp(self.delay_time, 0.0, 2.0, 0.0);
        self.delay_feedback = finite_clamp(self.delay_feedback, 0.0, 0.95, 0.0);
        self.delay_mix = finite_clamp(self.delay_mix, 0.0, 1.0, 0.0);
        self.bit_depth = self.bit_depth.clamp(1, 16);
        self.sample_rate_reduction = self.sample_rate_reduction.clamp(1, 64);
        self.distortion_drive = finite_clamp(self.distortion_drive, 0.1, 12.0, 1.0);
        self.arp_speed = finite_clamp(self.arp_speed, 0.0, 60.0, 0.0);
        self.retrigger_rate = finite_clamp(self.retrigger_rate, 0.0, 80.0, 0.0);
        self.gate_length = finite_clamp(self.gate_length, 0.02, 1.0, 1.0);
        self.pan = finite_clamp(self.pan, -1.0, 1.0, 0.0);
        self.auto_pan_speed = finite_clamp(self.auto_pan_speed, 0.0, 40.0, 0.0);
        self.auto_pan_depth = finite_clamp(self.auto_pan_depth, 0.0, 1.0, 0.0);
        self.sample_rate = self.sample_rate.clamp(8000, 192000);
        self
    }

    pub fn with_frequency(mut self, frequency: f32) -> Self {
        self.frequency = frequency;
        self
    }
}

fn duty_width(mode: &str) -> Option<f32> {
    match mode {
        "12_5" => Some(0.125),
        "25" => Some(0.25),
        "50" => Some(0.5),
        "75" => Some(0.75),
        _ => None,
    }
}

fn finite_clamp(value: f32, min: f32, max: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.clamp(min, max)
    } else {
        fallback
    }
}

fn normalize_choice(value: &str, allowed: &[&str], fallback: &str) -> String {
    if allowed.iter().any(|allowed_value| *allowed_value == value) {
        value.to_string()
    } else {
        fallback.to_string()
    }
}

pub struct RetroSynth {
    params: SynthParams,
    osc: Oscillator,
    filter: SVFFilter,
    amp_env: Envelope,
    filter_env: Envelope,
    lfo1: Lfo,
    lfo2: Lfo,
    effects: Effects,

    current_freq: f32,
    target_freq: f32,
    base_freq: f32,
    sweep_timer: f32,

    arp_index: usize,
    arp_timer: f32,

    pub total_samples: usize,
    pub current_sample: usize,
    pub released: bool,
    pub release_start_sample: usize,
    hold_samples: usize,
}

impl RetroSynth {
    pub fn new_with_hold(params: SynthParams, hold_seconds: f32) -> Self {
        let params = params.normalized();
        let mut osc = Oscillator::new();
        osc.waveform = params.waveform.clone();
        osc.sub_waveform = params.sub_waveform.clone();
        osc.pulse_width = params.pulse_width;
        osc.noise_color = params.noise_color.clone();
        osc.noise_mode = params.noise_mode.clone();
        osc.noise_period = params.noise_period;
        osc.sub_level = params.sub_level;
        osc.wavetable = params.wavetable.clone();

        let mut filter = SVFFilter::new();
        filter.filter_type = params.filter_type.clone();
        filter.cutoff = params.filter_cutoff;
        filter.resonance = params.filter_resonance;

        let mut amp_env = Envelope::new();
        amp_env.attack = params.attack;
        amp_env.decay = params.decay;
        amp_env.sustain = params.sustain;
        amp_env.release = params.release;

        let mut filter_env = Envelope::new();
        filter_env.attack = params.filter_attack;
        filter_env.decay = params.filter_decay;
        filter_env.sustain = params.filter_sustain;
        filter_env.release = params.filter_release;

        let mut lfo1 = Lfo::new();
        lfo1.waveform = params.lfo1_waveform.clone();
        lfo1.speed = params.lfo1_speed;
        lfo1.depth = params.lfo1_depth;
        lfo1.destination = params.lfo1_routing.clone();

        let mut lfo2 = Lfo::new();
        lfo2.waveform = params.lfo2_waveform.clone();
        lfo2.speed = params.lfo2_speed;
        lfo2.depth = params.lfo2_depth;
        lfo2.destination = params.lfo2_routing.clone();

        let mut effects = Effects::new();
        effects.bit_depth = params.bit_depth;
        effects.sample_rate_reduction = params.sample_rate_reduction;
        effects.delay_time = params.delay_time;
        effects.delay_feedback = params.delay_feedback;
        effects.delay_mix = params.delay_mix;
        effects.distortion_type = params.distortion_type.clone();
        effects.distortion_drive = params.distortion_drive;

        let hold_seconds = finite_clamp(hold_seconds, 0.02, 32.0, 0.85);
        let tail = params.release + params.delay_time * params.delay_mix.max(0.0) + 0.08;
        let total_duration = hold_seconds + tail.max(0.08);
        let total_samples = (total_duration * params.sample_rate as f32) as usize;
        let hold_samples = (hold_seconds * params.sample_rate as f32) as usize;

        Self {
            current_freq: params.frequency,
            target_freq: params.frequency,
            base_freq: params.frequency,
            params,
            osc,
            filter,
            amp_env,
            filter_env,
            lfo1,
            lfo2,
            effects,
            sweep_timer: 0.0,
            arp_index: 0,
            arp_timer: 0.0,
            total_samples,
            current_sample: 0,
            released: false,
            release_start_sample: 0,
            hold_samples,
        }
    }

    pub fn release(&mut self) {
        if !self.released {
            self.released = true;
            self.amp_env.trigger_release();
            self.filter_env.trigger_release();
            self.release_start_sample = self.current_sample;
            let samples_after_release =
                (self.params.release * self.params.sample_rate as f32) as usize;
            self.total_samples = self.current_sample + samples_after_release;
        }
    }

    fn get_arp_multiplier(chord: &str, index: usize) -> f32 {
        let semitones = match chord {
            "major" => [0, 4, 7, 12],
            "minor" => [0, 3, 7, 12],
            "octave" => [0, 12, 0, 12],
            "fifth" => [0, 7, 0, 7],
            _ => [0, 0, 0, 0],
        };
        f32::powf(2.0, semitones[index % 4] as f32 / 12.0)
    }

    pub fn render(mut self) -> RenderedAudio {
        let sample_rate = self.params.sample_rate;
        let stereo = self.params.pan.abs() > f32::EPSILON
            || (self.params.auto_pan_depth > f32::EPSILON
                && self.params.auto_pan_speed > f32::EPSILON);
        if !stereo {
            return RenderedAudio {
                samples: self.map(|sample| sample.clamp(-1.0, 1.0)).collect(),
                sample_rate,
                channels: 1,
            };
        }
        let mut samples = Vec::with_capacity(self.total_samples * 2);
        let auto_pan_speed = self.params.auto_pan_speed;
        let auto_pan_depth = self.params.auto_pan_depth;
        let static_pan = self.params.pan;
        let mut index = 0usize;
        while let Some(sample) = self.next() {
            let seconds = index as f32 / sample_rate as f32;
            let auto_pan = if auto_pan_speed > 0.0 && auto_pan_depth > 0.0 {
                (seconds * auto_pan_speed * std::f32::consts::TAU).sin() * auto_pan_depth
            } else {
                0.0
            };
            let pan = (static_pan + auto_pan).clamp(-1.0, 1.0);
            let left = ((1.0 - pan) * 0.5).sqrt();
            let right = ((1.0 + pan) * 0.5).sqrt();
            samples.push((sample * left).clamp(-1.0, 1.0));
            samples.push((sample * right).clamp(-1.0, 1.0));
            index += 1;
        }
        RenderedAudio {
            samples,
            sample_rate,
            channels: 2,
        }
    }
}

impl Iterator for RetroSynth {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_sample >= self.total_samples || self.amp_env.is_done() {
            return None;
        }

        let sr = self.params.sample_rate as f32;

        let lfo1_val = self.lfo1.next_sample(sr);
        let lfo2_val = self.lfo2.next_sample(sr);
        self.osc.pulse_width = self.current_duty_width();

        if !self.released && self.current_sample >= self.hold_samples {
            self.release();
        }
        self.apply_retrigger(sr);

        let mut pitch_mod = 1.0;
        let mut cutoff_mod = 0.0;
        let mut amp_mod = 1.0;

        let process_lfo = |dest: &str,
                           val: f32,
                           p_mod: &mut f32,
                           c_mod: &mut f32,
                           a_mod: &mut f32,
                           osc: &mut Oscillator| {
            match dest {
                "pitch" => *p_mod *= f32::powf(2.0, val / 12.0),
                "cutoff" => *c_mod += val * 5000.0,
                "pwm" => osc.pulse_width = (osc.pulse_width + val * 0.4).clamp(0.1, 0.9),
                "amp" => *a_mod *= 1.0 - val.abs().min(1.0),
                _ => {}
            }
        };

        process_lfo(
            &self.lfo1.destination,
            lfo1_val,
            &mut pitch_mod,
            &mut cutoff_mod,
            &mut amp_mod,
            &mut self.osc,
        );
        process_lfo(
            &self.lfo2.destination,
            lfo2_val,
            &mut pitch_mod,
            &mut cutoff_mod,
            &mut amp_mod,
            &mut self.osc,
        );

        let mut pitch_semitones = 0.0;
        if self.params.sweep_time > 0.0 && self.sweep_timer <= self.params.sweep_time {
            let sweep_progress = self.sweep_timer / self.params.sweep_time;
            pitch_semitones += self.params.sweep_amount * sweep_progress;
            self.sweep_timer += 1.0 / sr;
        }

        let seconds = self.current_sample as f32 / sr;
        if self.params.pitch_env_decay > 0.0 && self.params.pitch_env_amount.abs() > f32::EPSILON {
            let progress = (seconds / self.params.pitch_env_decay).clamp(0.0, 1.0);
            let remaining = if self.params.pitch_env_curve == "linear" {
                1.0 - progress
            } else {
                (-5.0 * progress).exp()
            };
            pitch_semitones += self.params.pitch_env_amount * remaining;
        }
        self.base_freq = self.params.frequency * f32::powf(2.0, pitch_semitones / 12.0);

        if self.params.arp_speed > 0.0 {
            self.arp_timer += 1.0 / sr;
            if self.arp_timer >= (1.0 / self.params.arp_speed) {
                self.arp_timer = 0.0;
                self.arp_index += 1;
            }
            self.target_freq =
                self.base_freq * Self::get_arp_multiplier(&self.params.arp_chord, self.arp_index);
        } else {
            self.target_freq = self.base_freq;
        }

        if self.params.portamento > 0.0 {
            let port_step =
                (self.target_freq - self.current_freq) / (self.params.portamento * sr).max(1.0);
            self.current_freq += port_step;
        } else {
            self.current_freq = self.target_freq;
        }

        let freq = self.current_freq * pitch_mod;

        let env_val = self.amp_env.next_sample(sr);
        let retrigger_gate = self.retrigger_gate(sr);
        let filter_env_val = self.filter_env.next_sample(sr);

        cutoff_mod += filter_env_val * self.params.filter_env_depth * 10000.0;

        let osc_out = self.osc.next_sample(freq, sr);
        let filter_out = self.filter.process(osc_out, sr, cutoff_mod);
        let amp_out = filter_out * env_val * amp_mod * retrigger_gate;

        let final_out = self.effects.process(amp_out, sr);

        self.current_sample += 1;
        Some(final_out)
    }
}

impl RetroSynth {
    fn current_duty_width(&self) -> f32 {
        if self.params.duty_sequence == "none" || self.params.duty_sequence_rate <= 0.0 {
            return self.params.pulse_width;
        }
        let step = ((self.current_sample as f32 / self.params.sample_rate as f32)
            * self.params.duty_sequence_rate) as usize;
        let values: &[f32] = match self.params.duty_sequence.as_str() {
            "classic_steps" => &[0.125, 0.25, 0.5, 0.75],
            "pulse_train" => &[0.125, 0.125, 0.75, 0.5],
            "skewed_ladder" => &[0.75, 0.125, 0.625, 0.25, 0.5, 0.875],
            _ => &[self.params.pulse_width],
        };
        values[step % values.len()]
    }

    fn retrigger_gate(&self, sample_rate: f32) -> f32 {
        if self.params.retrigger_rate <= 0.0 {
            return 1.0;
        }
        let period = sample_rate / self.params.retrigger_rate;
        let phase = (self.current_sample as f32 % period) / period;
        if phase <= self.params.gate_length {
            1.0
        } else {
            0.0
        }
    }

    fn apply_retrigger(&mut self, sample_rate: f32) {
        if self.params.retrigger_rate <= 0.0 || self.current_sample == 0 {
            return;
        }
        let period = (sample_rate / self.params.retrigger_rate).max(1.0) as usize;
        if self.current_sample % period == 0 {
            self.amp_env.retrigger();
            self.filter_env.retrigger();
        }
    }
}

impl Source for RetroSynth {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.params.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_params() -> SynthParams {
        SynthParams {
            waveform: "square".to_string(),
            sub_waveform: "none".to_string(),
            duty_mode: "free".to_string(),
            duty_sequence: "none".to_string(),
            duty_sequence_rate: 0.0,
            pulse_width: 0.35,
            noise_color: "white".to_string(),
            noise_mode: "lfsr".to_string(),
            noise_period: 1,
            sub_level: 0.0,
            wavetable: "none".to_string(),
            filter_type: "none".to_string(),
            filter_cutoff: 20000.0,
            filter_resonance: 0.0,
            filter_env_depth: 0.0,
            frequency: 440.0,
            sweep_amount: 0.0,
            sweep_time: 0.0,
            portamento: 0.0,
            pitch_env_amount: 0.0,
            pitch_env_decay: 0.0,
            pitch_env_curve: "exponential".to_string(),
            attack: 0.1,
            decay: 0.1,
            sustain: 0.5,
            release: 0.2,
            filter_attack: 0.0,
            filter_decay: 0.0,
            filter_sustain: 1.0,
            filter_release: 0.0,
            lfo1_waveform: "sine".to_string(),
            lfo1_speed: 0.0,
            lfo1_depth: 0.0,
            lfo1_routing: "none".to_string(),
            lfo2_waveform: "sine".to_string(),
            lfo2_speed: 0.0,
            lfo2_depth: 0.0,
            lfo2_routing: "none".to_string(),
            delay_time: 0.0,
            delay_feedback: 0.0,
            delay_mix: 0.0,
            distortion_type: "none".to_string(),
            distortion_drive: 1.0,
            arp_chord: "none".to_string(),
            arp_speed: 0.0,
            retrigger_rate: 0.0,
            gate_length: 1.0,
            pan: 0.0,
            auto_pan_speed: 0.0,
            auto_pan_depth: 0.0,
            bit_depth: 16,
            sample_rate_reduction: 1,
            sample_rate: 44100,
        }
    }

    #[test]
    fn test_modular_synth_initialization() {
        let params = default_params();
        let synth = RetroSynth::new_with_hold(params.clone(), 0.2);
        assert_eq!(synth.sample_rate(), 44100);
    }

    #[test]
    fn arpeggiator_and_portamento_generate_finite_audio() {
        let mut params = default_params();
        params.arp_chord = "minor".to_string();
        params.arp_speed = 12.0;
        params.portamento = 0.15;
        params.lfo1_routing = "pitch".to_string();
        params.lfo1_depth = 0.5;
        let synth = RetroSynth::new_with_hold(params, 0.4);
        let samples: Vec<f32> = synth.take(4096).collect();
        assert!(samples.iter().all(|sample| sample.is_finite()));
        assert!(samples.iter().any(|sample| sample.abs() > 0.01));
    }

    #[test]
    fn sound_creator_controls_change_render_when_dependencies_are_active() {
        assert_control_changes_render(
            "waveform",
            |_| {},
            |params| params.waveform = "sine".to_string(),
        );
        assert_control_changes_render("pulse_width", |_| {}, |params| params.pulse_width = 0.18);
        assert_control_changes_render(
            "noise_color",
            |params| params.waveform = "noise".to_string(),
            |params| params.noise_color = "brown".to_string(),
        );
        assert_control_changes_render(
            "duty_mode",
            |_| {},
            |params| params.duty_mode = "12_5".to_string(),
        );
        assert_control_changes_render(
            "duty_sequence",
            |params| params.duty_sequence_rate = 12.0,
            |params| params.duty_sequence = "classic_steps".to_string(),
        );
        assert_control_changes_render(
            "duty_sequence_rate",
            |params| {
                params.duty_sequence = "skewed_ladder".to_string();
                params.duty_sequence_rate = 3.0;
            },
            |params| params.duty_sequence_rate = 17.0,
        );
        assert_control_changes_render(
            "noise_mode",
            |params| params.waveform = "noise".to_string(),
            |params| params.noise_mode = "metallic".to_string(),
        );
        assert_control_changes_render(
            "noise_period",
            |params| params.waveform = "noise".to_string(),
            |params| params.noise_period = 18,
        );
        assert_control_changes_render(
            "wavetable",
            |params| params.waveform = "wavetable".to_string(),
            |params| params.wavetable = "gb_bell".to_string(),
        );
        assert_control_changes_render(
            "sub_waveform",
            |params| params.sub_level = 0.7,
            |params| params.sub_waveform = "sine".to_string(),
        );
        assert_control_changes_render(
            "sub_level",
            |params| params.sub_waveform = "triangle".to_string(),
            |params| params.sub_level = 0.85,
        );
        assert_control_changes_render(
            "filter_type",
            |params| {
                params.filter_cutoff = 1200.0;
                params.filter_resonance = 0.4;
            },
            |params| params.filter_type = "lowpass".to_string(),
        );
        assert_control_changes_render(
            "filter_cutoff",
            |params| {
                params.filter_type = "lowpass".to_string();
                params.filter_cutoff = 500.0;
            },
            |params| params.filter_cutoff = 6000.0,
        );
        assert_control_changes_render(
            "filter_resonance",
            |params| {
                params.filter_type = "lowpass".to_string();
                params.filter_cutoff = 1400.0;
            },
            |params| params.filter_resonance = 0.92,
        );
        assert_control_changes_render(
            "filter_env_depth",
            |params| {
                params.filter_type = "lowpass".to_string();
                params.filter_cutoff = 900.0;
                params.filter_decay = 0.08;
                params.filter_sustain = 0.1;
            },
            |params| params.filter_env_depth = 0.8,
        );
        assert_control_changes_render("frequency", |_| {}, |params| params.frequency = 660.0);
        assert_control_changes_render(
            "sweep_amount",
            |params| params.sweep_time = 0.35,
            |params| params.sweep_amount = 18.0,
        );
        assert_control_changes_render(
            "sweep_time",
            |params| {
                params.sweep_amount = 24.0;
                params.sweep_time = 0.05;
            },
            |params| params.sweep_time = 0.7,
        );
        assert_control_changes_render(
            "pitch_env_amount",
            |params| params.pitch_env_decay = 0.35,
            |params| params.pitch_env_amount = 24.0,
        );
        assert_control_changes_render(
            "pitch_env_decay",
            |params| {
                params.pitch_env_amount = 24.0;
                params.pitch_env_decay = 0.05;
            },
            |params| params.pitch_env_decay = 0.8,
        );
        assert_control_changes_render(
            "pitch_env_curve",
            |params| {
                params.pitch_env_amount = 24.0;
                params.pitch_env_decay = 0.8;
            },
            |params| params.pitch_env_curve = "linear".to_string(),
        );
        assert_control_changes_render(
            "portamento",
            |params| {
                params.arp_chord = "minor".to_string();
                params.arp_speed = 10.0;
            },
            |params| params.portamento = 0.35,
        );
        assert_control_changes_render("attack", |_| {}, |params| params.attack = 0.35);
        assert_control_changes_render(
            "decay",
            |params| {
                params.decay = 0.04;
                params.sustain = 0.2;
            },
            |params| params.decay = 0.75,
        );
        assert_control_changes_render("sustain", |_| {}, |params| params.sustain = 0.9);
        assert_control_changes_render("release", |_| {}, |params| params.release = 0.8);
        assert_control_changes_render(
            "filter_attack",
            |params| {
                params.filter_type = "lowpass".to_string();
                params.filter_cutoff = 700.0;
                params.filter_env_depth = 0.8;
            },
            |params| params.filter_attack = 0.35,
        );
        assert_control_changes_render(
            "filter_decay",
            |params| {
                params.filter_type = "lowpass".to_string();
                params.filter_cutoff = 700.0;
                params.filter_env_depth = 0.8;
                params.filter_decay = 0.03;
                params.filter_sustain = 0.15;
            },
            |params| params.filter_decay = 0.7,
        );
        assert_control_changes_render(
            "filter_sustain",
            |params| {
                params.filter_type = "lowpass".to_string();
                params.filter_cutoff = 700.0;
                params.filter_env_depth = 0.8;
            },
            |params| params.filter_sustain = 0.92,
        );
        assert_control_changes_render(
            "filter_release",
            |params| {
                params.filter_type = "lowpass".to_string();
                params.filter_cutoff = 700.0;
                params.filter_env_depth = 0.8;
                params.filter_release = 0.03;
            },
            |params| params.filter_release = 0.8,
        );
        assert_control_changes_render(
            "lfo1_waveform",
            |params| {
                params.lfo1_routing = "pitch".to_string();
                params.lfo1_speed = 8.0;
                params.lfo1_depth = 4.0;
            },
            |params| params.lfo1_waveform = "square".to_string(),
        );
        assert_control_changes_render(
            "lfo1_speed",
            |params| {
                params.lfo1_routing = "pitch".to_string();
                params.lfo1_speed = 2.0;
                params.lfo1_depth = 4.0;
            },
            |params| params.lfo1_speed = 18.0,
        );
        assert_control_changes_render(
            "lfo1_depth",
            |params| {
                params.lfo1_routing = "pitch".to_string();
                params.lfo1_speed = 8.0;
            },
            |params| params.lfo1_depth = 5.0,
        );
        assert_control_changes_render(
            "lfo1_routing",
            |params| {
                params.lfo1_speed = 8.0;
                params.lfo1_depth = 5.0;
            },
            |params| params.lfo1_routing = "pitch".to_string(),
        );
        assert_control_changes_render(
            "lfo2_waveform",
            |params| {
                params.lfo2_routing = "amp".to_string();
                params.lfo2_speed = 7.0;
                params.lfo2_depth = 0.7;
            },
            |params| params.lfo2_waveform = "sawtooth".to_string(),
        );
        assert_control_changes_render(
            "lfo2_speed",
            |params| {
                params.lfo2_routing = "amp".to_string();
                params.lfo2_speed = 2.0;
                params.lfo2_depth = 0.7;
            },
            |params| params.lfo2_speed = 16.0,
        );
        assert_control_changes_render(
            "lfo2_depth",
            |params| {
                params.lfo2_routing = "amp".to_string();
                params.lfo2_speed = 7.0;
            },
            |params| params.lfo2_depth = 0.8,
        );
        assert_control_changes_render(
            "lfo2_routing",
            |params| {
                params.lfo2_speed = 7.0;
                params.lfo2_depth = 0.8;
            },
            |params| params.lfo2_routing = "amp".to_string(),
        );
        assert_control_changes_render(
            "delay_time",
            |params| {
                params.delay_time = 0.03;
                params.delay_feedback = 0.35;
                params.delay_mix = 0.55;
            },
            |params| params.delay_time = 0.14,
        );
        assert_control_changes_render(
            "delay_feedback",
            |params| {
                params.delay_time = 0.04;
                params.delay_mix = 0.65;
            },
            |params| params.delay_feedback = 0.8,
        );
        assert_control_changes_render(
            "delay_mix",
            |params| {
                params.delay_time = 0.04;
                params.delay_feedback = 0.5;
            },
            |params| params.delay_mix = 0.75,
        );
        assert_control_changes_render("bit_depth", |_| {}, |params| params.bit_depth = 4);
        assert_control_changes_render(
            "sample_rate_reduction",
            |_| {},
            |params| params.sample_rate_reduction = 9,
        );
        assert_control_changes_render(
            "distortion_type",
            |params| params.distortion_drive = 3.5,
            |params| params.distortion_type = "soft_clip".to_string(),
        );
        assert_control_changes_render(
            "distortion_drive",
            |params| params.distortion_type = "hard_clip".to_string(),
            |params| params.distortion_drive = 5.0,
        );
        assert_control_changes_render(
            "arp_chord",
            |params| params.arp_speed = 10.0,
            |params| params.arp_chord = "minor".to_string(),
        );
        assert_control_changes_render(
            "arp_speed",
            |params| {
                params.arp_chord = "minor".to_string();
                params.arp_speed = 2.0;
            },
            |params| params.arp_speed = 18.0,
        );
        assert_control_changes_render(
            "retrigger_rate",
            |_| {},
            |params| params.retrigger_rate = 16.0,
        );
        assert_control_changes_render(
            "gate_length",
            |params| params.retrigger_rate = 16.0,
            |params| params.gate_length = 0.18,
        );
        assert_control_changes_render("pan", |_| {}, |params| params.pan = -0.8);
        assert_control_changes_render(
            "auto_pan_speed",
            |params| params.auto_pan_depth = 0.9,
            |params| params.auto_pan_speed = 6.0,
        );
        assert_control_changes_render(
            "auto_pan_depth",
            |params| params.auto_pan_speed = 6.0,
            |params| params.auto_pan_depth = 0.9,
        );
    }

    #[test]
    fn parameter_normalization_replaces_invalid_values() {
        let mut params = default_params();
        params.frequency = f32::NAN;
        params.filter_cutoff = f32::INFINITY;
        params.bit_depth = 200;
        params.sample_rate = 1;
        let normalized = params.normalized();
        assert_eq!(normalized.frequency, 440.0);
        assert_eq!(normalized.filter_cutoff, 20000.0);
        assert_eq!(normalized.bit_depth, 16);
        assert_eq!(normalized.sample_rate, 8000);
    }

    #[test]
    fn deserializing_legacy_presets_uses_defaults() {
        let params: SynthParams =
            serde_json::from_str(r#"{"waveform":"noise","frequency":220}"#).unwrap();
        assert_eq!(params.waveform, "noise");
        assert_eq!(params.frequency, 220.0);
        assert_eq!(params.sample_rate, 44100);
        assert_eq!(params.filter_type, "none");
    }

    #[test]
    fn wavetable_none_normalizes_to_audible_table() {
        let mut params = default_params();
        params.waveform = "wavetable".to_string();
        params.wavetable = "none".to_string();
        let normalized = params.normalized();
        assert_eq!(normalized.wavetable, "gb_organ");
        let rendered = RetroSynth::new_with_hold(normalized, 0.2).render();
        assert!(rendered.samples.iter().any(|sample| sample.abs() > 0.01));
    }

    fn assert_control_changes_render<F, G>(name: &str, setup: F, change: G)
    where
        F: Fn(&mut SynthParams),
        G: Fn(&mut SynthParams),
    {
        let mut left = default_params();
        setup(&mut left);
        let mut right = left.clone();
        change(&mut right);
        assert!(
            renders_differ(left, right),
            "{name} did not change rendered audio"
        );
    }

    fn renders_differ(left: SynthParams, right: SynthParams) -> bool {
        let left = RetroSynth::new_with_hold(left, 0.45).render().samples;
        let right = RetroSynth::new_with_hold(right, 0.45).render().samples;
        if left.len().abs_diff(right.len()) > 32 {
            return true;
        }
        let compared = left.len().min(right.len()).max(1);
        let mean_delta = left
            .iter()
            .zip(right.iter())
            .map(|(left, right)| (left - right).abs())
            .sum::<f32>()
            / compared as f32;
        mean_delta > 0.0005
    }
}
