use std::f64::consts::PI;
use rand::Rng;

pub struct DisplacementGenerator {
    amplitude: f64,
    frequency: f64,
    phase: f64,
    noise_level: f64,
    time: f64,
}

impl DisplacementGenerator {
    pub fn new(amplitude: f64, frequency: f64) -> Self {
        Self {
            amplitude,
            frequency,
            phase: 0.0,
            noise_level: 0.1,
            time: 0.0,
        }
    }

    pub fn set_noise_level(&mut self, level: f64) {
        self.noise_level = level.clamp(0.0, 1.0);
    }

    pub fn next_value(&mut self) -> f64 {
        let mut rng = rand::thread_rng();
        let base_value = self.amplitude * (2.0 * PI * self.frequency * self.time + self.phase).sin();
        let noise = (rng.gen::<f64>() * 2.0 - 1.0) * self.noise_level * self.amplitude;

        self.time += 0.1;
        if self.time >= 100.0 {
            self.time = 0.0;
            self.phase = rng.gen::<f64>() * 2.0 * PI;
        }

        base_value + noise
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_displacement_generator() {
        let mut gen = DisplacementGenerator::new(1.0, 0.5);
        let value = gen.next_value();
        assert!(value >= -1.5 && value <= 1.5);
    }
}