use std::f32::consts::PI;

pub struct Lfo {
    pub waveform: String,
    pub speed: f32,
    pub depth: f32,
    pub destination: String,
    phase: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_lfo_outputs_zero() {
        let mut lfo = Lfo::new();
        lfo.depth = 12.0;
        lfo.destination = "none".to_string();
        assert_eq!(lfo.next_sample(44100.0), 0.0);
    }

    #[test]
    fn lfo_waveforms_are_finite() {
        for waveform in ["sine", "square", "triangle", "sawtooth"] {
            let mut lfo = Lfo::new();
            lfo.waveform = waveform.to_string();
            lfo.destination = "pitch".to_string();
            lfo.depth = 3.0;
            for _ in 0..128 {
                assert!(lfo.next_sample(44100.0).is_finite());
            }
        }
    }
}

impl Lfo {
    pub fn new() -> Self {
        Self {
            waveform: "sine".to_string(),
            speed: 5.0,
            depth: 0.0,
            destination: "none".to_string(),
            phase: 0.0,
        }
    }

    pub fn next_sample(&mut self, sample_rate: f32) -> f32 {
        if self.depth == 0.0 || self.destination == "none" {
            return 0.0;
        }

        let phase_inc = self.speed / sample_rate;
        self.phase = (self.phase + phase_inc) % 1.0;

        let val = match self.waveform.as_str() {
            "square" => {
                if self.phase < 0.5 {
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
            _ => (self.phase * 2.0 * PI).sin(),
        };

        val * self.depth
    }
}
