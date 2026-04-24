#[derive(Debug, Clone)]
pub struct PlateDetectTask {
    pub peak:      Option<f64>,
    pub post_peak: Option<f64>,
    pub trigger:   f64,
}

impl PlateDetectTask {
    pub fn new(trigger: f64) -> Self {
        Self {
            trigger,
            peak: None,
            post_peak: None,
        }
    }

    /// avg sampling

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
            self.peak      = Some(weight);
            self.post_peak = None;
            return None;
        }

        let Some(post_peak) = self.post_peak else {
            self.post_peak = Some(weight);
            return None;
        };

        // weight must go below trigger
        if weight > trigger {
            return None;
        }

        // Compute drop from peak
        let drop = current_peak - weight;

        // needs to drop by 50% of trigger
        if drop < trigger * 0.5 {
            return None;
        }

        let avg = (current_peak + post_peak) / 2.0;

        self.peak = None;

        return Some((current_peak, avg));
    }
}