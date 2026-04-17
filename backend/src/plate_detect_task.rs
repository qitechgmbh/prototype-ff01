#[derive(Debug, Clone)]
pub struct PlateDetectTask {
    pub peak:    Option<f64>,
    pub trigger: f64,
    pub seen_rise: bool,
}

impl PlateDetectTask {
    pub fn new(trigger: f64) -> Self {
        Self {
            trigger,
            peak: None,
            seen_rise: false,
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
            self.seen_rise = true;
            self.peak = Some(weight);
            return None;
        }

        // must rise first
        if !self.seen_rise {
            return None;
        }

        // Compute drop from peak
        let drop = current_peak - weight;

        // needs to drop to 1/3 of trigger
        if drop < trigger * 0.33 {
            return None;
        }

        self.peak = None;
        self.seen_rise = false;

        return Some((current_peak, drop));
    }
}