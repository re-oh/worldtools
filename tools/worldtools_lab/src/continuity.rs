use serde::Serialize;
use worldtools_analysis::{LodAudit, SeamAudit};

#[derive(Debug, Serialize)]
pub struct ContinuityReport {
    pub applicable: bool,
    pub compared_pairs: usize,
    pub compared_samples: usize,
    pub maximum_absolute_error_m: f32,
    pub root_mean_square_error_m: f64,
}

#[derive(Default)]
pub struct ErrorAccumulator {
    pairs: usize,
    samples: usize,
    maximum: f32,
    squared_error_sum: f64,
}

impl ErrorAccumulator {
    pub fn add_seam(&mut self, audit: SeamAudit) {
        self.add(
            audit.compared_samples,
            audit.maximum_absolute_error_m,
            audit.root_mean_square_error_m,
        );
    }

    pub fn add_lod(&mut self, audit: LodAudit) {
        self.add(
            audit.compared_samples,
            audit.maximum_absolute_error_m,
            audit.root_mean_square_error_m,
        );
    }

    #[must_use]
    pub fn finish(self, applicable: bool) -> ContinuityReport {
        let root_mean_square_error_m = if self.samples == 0 {
            0.0
        } else {
            (self.squared_error_sum / count_as_f64(self.samples)).sqrt()
        };
        ContinuityReport {
            applicable,
            compared_pairs: self.pairs,
            compared_samples: self.samples,
            maximum_absolute_error_m: self.maximum,
            root_mean_square_error_m,
        }
    }

    fn add(&mut self, samples: usize, maximum: f32, root_mean_square: f64) {
        self.pairs += 1;
        self.samples += samples;
        self.maximum = self.maximum.max(maximum);
        self.squared_error_sum += root_mean_square.powi(2) * count_as_f64(samples);
    }
}

#[allow(clippy::cast_precision_loss)]
fn count_as_f64(count: usize) -> f64 {
    count as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accumulator_weights_rms_by_sample_count() {
        let mut accumulator = ErrorAccumulator::default();
        accumulator.add_seam(SeamAudit {
            compared_samples: 1,
            maximum_absolute_error_m: 2.0,
            root_mean_square_error_m: 2.0,
        });
        accumulator.add_seam(SeamAudit {
            compared_samples: 3,
            maximum_absolute_error_m: 1.0,
            root_mean_square_error_m: 1.0,
        });
        let report = accumulator.finish(true);
        assert!((report.root_mean_square_error_m - (7.0_f64 / 4.0).sqrt()).abs() < 1.0e-12);
        assert_eq!(report.compared_pairs, 2);
    }
}
