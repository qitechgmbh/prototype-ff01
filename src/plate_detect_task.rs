#[derive(Debug, Clone)]
pub struct PlateDetectTask {
    pub peak:    Option<f64>,
    pub trigger: f64,
}

impl PlateDetectTask {
    pub fn new(trigger: f64) -> Self {
        Self {
            trigger,
            peak: None,
        }
    }

    pub fn check(&mut self, weight: f64) -> Option<(f64, f64)> {
        let trigger = self.trigger;

        let Some(current_peak) = self.peak else {
            if weight >= trigger {
                // initialize peak
                self.peak = Some(weight);
            }
            return None;
        };

        // Rising phase → update peak
        if weight > current_peak {
            self.peak = Some(weight);
            return None;
        }

        // Compute drop from peak
        let drop = current_peak - weight;

        // needs to drop to 1/3 of trigger
        if drop < trigger * 0.33 {
            return None;
        }

        return self.peak.map(|v| (v, drop));
    }
}