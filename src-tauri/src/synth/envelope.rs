pub struct Envelope {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    state: EnvState,
    level: f32,
    release_start_level: f32,
    release_elapsed: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_stage_times_do_not_create_invalid_levels() {
        let mut envelope = Envelope::new();
        envelope.attack = 0.0;
        envelope.decay = 0.0;
        envelope.release = 0.0;
        envelope.sustain = 0.4;
        let first = envelope.next_sample(44100.0);
        let second = envelope.next_sample(44100.0);
        envelope.trigger_release();
        let released = envelope.next_sample(44100.0);
        assert!(first.is_finite());
        assert!(second.is_finite());
        assert!(released.is_finite());
        assert!(envelope.is_done());
    }

    #[test]
    fn release_reaches_done_state() {
        let mut envelope = Envelope::new();
        envelope.attack = 0.0;
        envelope.decay = 0.0;
        envelope.sustain = 1.0;
        envelope.release = 0.01;
        envelope.next_sample(1000.0);
        envelope.next_sample(1000.0);
        envelope.trigger_release();
        for _ in 0..20 {
            envelope.next_sample(1000.0);
        }
        assert!(envelope.is_done());
    }
}

#[derive(PartialEq)]
enum EnvState {
    Attack,
    Decay,
    Sustain,
    Release,
    Done,
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.5,
            release: 0.2,
            state: EnvState::Attack,
            level: 0.0,
            release_start_level: 0.0,
            release_elapsed: 0.0,
        }
    }

    pub fn trigger_release(&mut self) {
        if self.state != EnvState::Release && self.state != EnvState::Done {
            self.release_start_level = self.level;
            self.release_elapsed = 0.0;
            self.state = EnvState::Release;
        }
    }

    pub fn retrigger(&mut self) {
        self.state = EnvState::Attack;
        self.level = 0.0;
        self.release_start_level = 0.0;
        self.release_elapsed = 0.0;
    }

    pub fn is_done(&self) -> bool {
        self.state == EnvState::Done
    }

    pub fn next_sample(&mut self, sample_rate: f32) -> f32 {
        let sample_rate = sample_rate.max(1.0);
        match self.state {
            EnvState::Attack => {
                if self.attack <= 0.0 {
                    self.level = 1.0;
                    self.state = EnvState::Decay;
                } else {
                    let step = 1.0 / (self.attack * sample_rate);
                    self.level += step;
                    if self.level >= 1.0 {
                        self.level = 1.0;
                        self.state = EnvState::Decay;
                    }
                }
            }
            EnvState::Decay => {
                if self.decay <= 0.0 {
                    self.level = self.sustain;
                    self.state = EnvState::Sustain;
                } else {
                    let step = (1.0 - self.sustain) / (self.decay * sample_rate);
                    self.level -= step;
                    if self.level <= self.sustain {
                        self.level = self.sustain;
                        self.state = EnvState::Sustain;
                    }
                }
            }
            EnvState::Sustain => {
                self.level = self.sustain;
            }
            EnvState::Release => {
                if self.release <= 0.0 || self.release_start_level <= 0.0 {
                    self.level = 0.0;
                    self.state = EnvState::Done;
                } else {
                    self.release_elapsed += 1.0 / sample_rate;
                    let progress = (self.release_elapsed / self.release).clamp(0.0, 1.0);
                    self.level = self.release_start_level * (1.0 - progress);
                    if progress >= 1.0 {
                        self.level = 0.0;
                        self.state = EnvState::Done;
                    }
                }
            }
            EnvState::Done => {
                self.level = 0.0;
            }
        }
        self.level
    }
}
