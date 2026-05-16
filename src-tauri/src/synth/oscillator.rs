use std::f32::consts::PI;

pub struct Oscillator {
    pub waveform: String,
    pub sub_waveform: String,
    pub pulse_width: f32,
    pub noise_color: String,
    pub noise_mode: String,
    pub noise_period: u32,
    pub sub_level: f32,
    pub wavetable: String,
    phase: f32,
    sub_phase: f32,
    lfsr: u16,
    last_noise_val: f32,
    brown_noise: f32,
    held_noise: f32,
    noise_counter: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_waveforms_generate_finite_samples() {
        for waveform in ["square", "triangle", "sawtooth", "sine", "noise"] {
            let mut oscillator = Oscillator::new();
            oscillator.waveform = waveform.to_string();
            for _ in 0..128 {
                let sample = oscillator.next_sample(440.0, 44100.0);
                assert!(sample.is_finite());
                assert!((-1.5..=1.5).contains(&sample));
            }
        }
    }

    #[test]
    fn sub_oscillator_mix_stays_bounded() {
        let mut oscillator = Oscillator::new();
        oscillator.waveform = "square".to_string();
        oscillator.sub_waveform = "triangle".to_string();
        oscillator.sub_level = 0.75;
        for _ in 0..128 {
            let sample = oscillator.next_sample(110.0, 44100.0);
            assert!(sample.is_finite());
            assert!((-1.1..=1.1).contains(&sample));
        }
    }

    #[test]
    fn retro_noise_modes_use_distinct_clock_tables() {
        let mut lfsr = Oscillator::new();
        lfsr.noise_mode = "lfsr".to_string();
        lfsr.noise_period = 64;
        let mut periodic = Oscillator::new();
        periodic.noise_mode = "periodic".to_string();
        periodic.noise_period = 64;
        let mut metallic = Oscillator::new();
        metallic.noise_mode = "metallic".to_string();
        metallic.noise_period = 64;

        assert_eq!(lfsr.effective_noise_period(), 64);
        assert!(periodic.effective_noise_period() > lfsr.effective_noise_period());
        assert_ne!(
            metallic.effective_noise_period(),
            periodic.effective_noise_period()
        );
    }
}

impl Oscillator {
    pub fn new() -> Self {
        Self {
            waveform: "square".to_string(),
            sub_waveform: "none".to_string(),
            pulse_width: 0.5,
            noise_color: "white".to_string(),
            noise_mode: "lfsr".to_string(),
            noise_period: 1,
            sub_level: 0.0,
            wavetable: "none".to_string(),
            phase: 0.0,
            sub_phase: 0.0,
            lfsr: 0xACE1,
            last_noise_val: 0.0,
            brown_noise: 0.0,
            held_noise: 0.0,
            noise_counter: 0,
        }
    }

    pub fn next_sample(&mut self, frequency: f32, sample_rate: f32) -> f32 {
        let phase_inc = frequency / sample_rate;
        self.phase = (self.phase + phase_inc) % 1.0;

        let sub_freq = frequency / 2.0;
        let sub_phase_inc = sub_freq / sample_rate;
        self.sub_phase = (self.sub_phase + sub_phase_inc) % 1.0;

        let main_out = match self.waveform.as_str() {
            "square" => {
                if self.phase < self.pulse_width {
                    1.0
                } else {
                    -1.0
                }
            }
            "triangle" => {
                if self.phase < 0.5 {
                    4.0 * self.phase - 1.0
                } else {
                    3.0 - 4.0 * self.phase
                }
            }
            "sawtooth" => 2.0 * self.phase - 1.0,
            "sine" => (self.phase * 2.0 * PI).sin(),
            "noise" => self.next_noise(),
            "wavetable" => self.wavetable_sample(self.phase),
            _ => 0.0,
        };

        let sub_out = match self.sub_waveform.as_str() {
            "square" => {
                if self.sub_phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            "triangle" => {
                if self.sub_phase < 0.5 {
                    4.0 * self.sub_phase - 1.0
                } else {
                    3.0 - 4.0 * self.sub_phase
                }
            }
            "sawtooth" => 2.0 * self.sub_phase - 1.0,
            "sine" => (self.sub_phase * 2.0 * PI).sin(),
            _ => 0.0,
        };

        (main_out * (1.0 - self.sub_level)) + (sub_out * self.sub_level)
    }

    fn next_noise(&mut self) -> f32 {
        let period = self.effective_noise_period();
        if self.noise_counter == 0 || self.noise_counter >= period {
            self.noise_counter = 0;
            self.held_noise = self.compute_noise();
        }
        self.noise_counter += 1;
        self.held_noise
    }

    fn compute_noise(&mut self) -> f32 {
        let bit = match self.noise_mode.as_str() {
            "periodic" => ((self.lfsr >> 0) ^ (self.lfsr >> 6)) & 1,
            "metallic" => {
                ((self.lfsr >> 0) ^ (self.lfsr >> 1) ^ (self.lfsr >> 5) ^ (self.lfsr >> 14)) & 1
            }
            _ => ((self.lfsr >> 0) ^ (self.lfsr >> 2) ^ (self.lfsr >> 3) ^ (self.lfsr >> 5)) & 1,
        };
        self.lfsr = (self.lfsr >> 1) | (bit << 15);
        if self.noise_mode == "periodic" {
            self.lfsr &= 0x007f;
            if self.lfsr == 0 {
                self.lfsr = 0x005d;
            }
        }
        let white = ((self.lfsr as f32) / 65535.0) * 2.0 - 1.0;
        match self.noise_color.as_str() {
            "pink" => {
                let pink = (self.last_noise_val + white) * 0.5;
                self.last_noise_val = pink;
                pink
            }
            "brown" => {
                self.brown_noise = (self.brown_noise + (0.02 * white)) / 1.02;
                self.brown_noise * 3.5
            }
            _ => white,
        }
    }

    fn effective_noise_period(&self) -> u32 {
        const NES_PERIODS: [u32; 16] = [
            4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
        ];
        const GB_PERIODS: [u32; 8] = [2, 4, 8, 16, 32, 64, 128, 256];
        match self.noise_mode.as_str() {
            "periodic" => {
                let index = (((self.noise_period.saturating_sub(1)) as usize * NES_PERIODS.len())
                    / 128)
                    .min(NES_PERIODS.len() - 1);
                NES_PERIODS[index]
            }
            "metallic" => {
                let index = (((self.noise_period.saturating_sub(1)) as usize * GB_PERIODS.len())
                    / 128)
                    .min(GB_PERIODS.len() - 1);
                GB_PERIODS[index]
            }
            _ => self.noise_period.max(1),
        }
    }

    fn wavetable_sample(&self, phase: f32) -> f32 {
        let table: &[f32] = match self.wavetable.as_str() {
            "gb_organ" => &[
                0.0, 0.45, 0.8, 0.95, 0.7, 0.35, 0.1, -0.05, -0.1, -0.35, -0.7, -0.95, -0.8, -0.45,
                0.0, 0.2,
            ],
            "gb_bell" => &[
                0.0, 0.9, 0.3, 0.75, -0.15, 0.4, -0.6, 0.15, -0.9, -0.2, -0.5, 0.05, -0.25, 0.35,
                -0.1, 0.0,
            ],
            "gb_saw" => &[
                -1.0, -0.87, -0.73, -0.6, -0.47, -0.33, -0.2, -0.07, 0.07, 0.2, 0.33, 0.47, 0.6,
                0.73, 0.87, 1.0,
            ],
            "gb_pulse" => &[
                1.0, 1.0, 1.0, 1.0, 0.65, 0.2, -0.2, -0.65, -1.0, -1.0, -1.0, -1.0, -0.65, -0.2,
                0.2, 0.65,
            ],
            _ => &[
                0.0, 0.45, 0.8, 0.95, 0.7, 0.35, 0.1, -0.05, -0.1, -0.35, -0.7, -0.95, -0.8, -0.45,
                0.0, 0.2,
            ],
        };
        let position = phase * table.len() as f32;
        let left = position.floor() as usize % table.len();
        let right = (left + 1) % table.len();
        let mix = position.fract();
        table[left] * (1.0 - mix) + table[right] * mix
    }
}
