pub struct Effects {
    pub bit_depth: u32,
    pub sample_rate_reduction: u32,
    pub delay_time: f32,
    pub delay_feedback: f32,
    pub delay_mix: f32,
    pub distortion_type: String,
    pub distortion_drive: f32,

    delay_buffer: Vec<f32>,
    delay_index: usize,
    sample_hold: f32,
    sample_counter: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bit_depth_and_rate_reduction_stay_finite() {
        let mut effects = Effects::new();
        effects.bit_depth = 4;
        effects.sample_rate_reduction = 4;
        for _ in 0..128 {
            let sample = effects.process(0.37, 44100.0);
            assert!(sample.is_finite());
        }
    }

    #[test]
    fn delay_feedback_stays_bounded_for_short_run() {
        let mut effects = Effects::new();
        effects.delay_time = 0.02;
        effects.delay_feedback = 0.5;
        effects.delay_mix = 0.5;
        for _ in 0..2048 {
            let sample = effects.process(0.2, 44100.0);
            assert!(sample.is_finite());
            assert!(sample.abs() < 4.0);
        }
    }
}

impl Effects {
    pub fn new() -> Self {
        Self {
            bit_depth: 16,
            sample_rate_reduction: 1,
            delay_time: 0.0,
            delay_feedback: 0.0,
            delay_mix: 0.0,
            distortion_type: "none".to_string(),
            distortion_drive: 1.0,

            delay_buffer: vec![0.0; 44100 * 2],
            delay_index: 0,
            sample_hold: 0.0,
            sample_counter: 0,
        }
    }

    pub fn process(&mut self, mut sample: f32, sample_rate: f32) -> f32 {
        sample = self.apply_distortion(sample);

        if self.bit_depth < 16 && self.bit_depth > 0 {
            let steps = f32::powf(2.0, self.bit_depth as f32) - 1.0;
            sample = (sample * steps).round() / steps;
        }

        if self.sample_rate_reduction > 1 {
            if self.sample_counter == 0 || self.sample_counter >= self.sample_rate_reduction {
                self.sample_counter = 0;
                self.sample_hold = sample;
            } else {
                sample = self.sample_hold;
            }
            self.sample_counter += 1;
        }

        if self.delay_time > 0.0 && self.delay_mix > 0.0 {
            let delay_samples = (self.delay_time * sample_rate) as usize;
            if delay_samples > 0 && delay_samples < self.delay_buffer.len() {
                let read_idx = if self.delay_index >= delay_samples {
                    self.delay_index - delay_samples
                } else {
                    self.delay_buffer.len() - (delay_samples - self.delay_index)
                };

                let delayed_sample = self.delay_buffer[read_idx];
                let out = (sample * (1.0 - self.delay_mix)) + (delayed_sample * self.delay_mix);

                self.delay_buffer[self.delay_index] = sample + delayed_sample * self.delay_feedback;
                self.delay_index = (self.delay_index + 1) % self.delay_buffer.len();

                return out;
            }
        }

        sample
    }

    fn apply_distortion(&self, sample: f32) -> f32 {
        let driven = sample * self.distortion_drive.max(0.1);
        match self.distortion_type.as_str() {
            "hard_clip" => driven.clamp(-1.0, 1.0),
            "soft_clip" => (driven * 1.4).tanh(),
            "foldback" => {
                let threshold = 0.75;
                if driven.abs() <= threshold {
                    driven
                } else {
                    let folded = ((driven.abs() - threshold) % (threshold * 4.0)) - threshold * 2.0;
                    folded.abs() - threshold
                }
            }
            _ => sample,
        }
    }
}
