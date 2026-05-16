use std::f32::consts::PI;

pub struct SVFFilter {
    pub filter_type: String,
    pub cutoff: f32,
    pub resonance: f32,
    ic1eq: f32,
    ic2eq: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_modes_stay_finite() {
        for filter_type in ["none", "lowpass", "highpass", "bandpass"] {
            let mut filter = SVFFilter::new();
            filter.filter_type = filter_type.to_string();
            filter.cutoff = 1200.0;
            filter.resonance = 0.8;
            for _ in 0..256 {
                let sample = filter.process(0.5, 44100.0, 200.0);
                assert!(sample.is_finite());
            }
        }
    }

    #[test]
    fn cutoff_modulation_is_clamped() {
        let mut filter = SVFFilter::new();
        filter.filter_type = "lowpass".to_string();
        filter.cutoff = 20.0;
        for modulation in [-100000.0, 100000.0] {
            let sample = filter.process(0.25, 44100.0, modulation);
            assert!(sample.is_finite());
        }
    }
}

impl SVFFilter {
    pub fn new() -> Self {
        Self {
            filter_type: "none".to_string(),
            cutoff: 20000.0,
            resonance: 0.0,
            ic1eq: 0.0,
            ic2eq: 0.0,
        }
    }

    pub fn process(&mut self, input: f32, sample_rate: f32, cutoff_mod: f32) -> f32 {
        if self.filter_type == "none" {
            return input;
        }

        let mut actual_cutoff = self.cutoff + cutoff_mod;
        if actual_cutoff < 20.0 {
            actual_cutoff = 20.0;
        }
        if actual_cutoff > sample_rate / 2.0 - 100.0 {
            actual_cutoff = sample_rate / 2.0 - 100.0;
        }

        let g = (PI * actual_cutoff / sample_rate).tan();
        let mut k = 2.0 - 2.0 * self.resonance;
        if k < 0.01 {
            k = 0.01;
        }

        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;

        let v3 = input - self.ic2eq;
        let v1 = a1 * self.ic1eq + a2 * v3;
        let v2 = self.ic2eq + a2 * self.ic1eq + a3 * v3;

        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;

        match self.filter_type.as_str() {
            "lowpass" => v2,
            "highpass" => input - k * v1 - v2,
            "bandpass" => v1,
            _ => input,
        }
    }
}
