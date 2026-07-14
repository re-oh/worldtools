use serde::{Deserialize, Serialize};

const HISTOGRAM_BINS: usize = 64;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Quantiles {
    pub p01: f32,
    pub p05: f32,
    pub p25: f32,
    pub p50: f32,
    pub p75: f32,
    pub p95: f32,
    pub p99: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Distribution {
    pub sample_count: usize,
    pub finite_sample_count: usize,
    pub non_finite_sample_count: usize,
    pub minimum: f32,
    pub maximum: f32,
    pub mean: f64,
    pub standard_deviation: f64,
    pub quantiles: Quantiles,
    /// Shannon entropy of 64 equal-width bins spanning this sample's own
    /// minimum and maximum. It describes within-sample variety and must not be
    /// compared as if every report used common elevation bins.
    pub range_normalized_entropy_bits: f64,
}

impl Distribution {
    #[must_use]
    pub fn measure(values: &[f32]) -> Self {
        let mut sorted = values
            .iter()
            .copied()
            .filter(|value| value.is_finite())
            .collect::<Vec<_>>();
        sorted.sort_unstable_by(f32::total_cmp);
        let finite_sample_count = sorted.len();
        if finite_sample_count == 0 {
            return Self {
                sample_count: values.len(),
                finite_sample_count: 0,
                non_finite_sample_count: values.len(),
                minimum: 0.0,
                maximum: 0.0,
                mean: 0.0,
                standard_deviation: 0.0,
                quantiles: Quantiles::default(),
                range_normalized_entropy_bits: 0.0,
            };
        }

        let minimum = sorted[0];
        let maximum = sorted[finite_sample_count - 1];
        let count = count_as_f64(finite_sample_count);
        let mean = sorted.iter().map(|&value| f64::from(value)).sum::<f64>() / count;
        let variance = sorted
            .iter()
            .map(|&value| {
                let delta = f64::from(value) - mean;
                delta * delta
            })
            .sum::<f64>()
            / count;

        Self {
            sample_count: values.len(),
            finite_sample_count,
            non_finite_sample_count: values.len() - finite_sample_count,
            minimum,
            maximum,
            mean,
            standard_deviation: variance.sqrt(),
            quantiles: Quantiles {
                p01: percentile(&sorted, 0.01),
                p05: percentile(&sorted, 0.05),
                p25: percentile(&sorted, 0.25),
                p50: percentile(&sorted, 0.50),
                p75: percentile(&sorted, 0.75),
                p95: percentile(&sorted, 0.95),
                p99: percentile(&sorted, 0.99),
            },
            range_normalized_entropy_bits: entropy(&sorted, minimum, maximum),
        }
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
fn percentile(sorted: &[f32], quantile: f64) -> f32 {
    let position = quantile * (sorted.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    let fraction = (position - lower as f64) as f32;
    sorted[lower] + (sorted[upper] - sorted[lower]) * fraction
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
fn entropy(values: &[f32], minimum: f32, maximum: f32) -> f64 {
    let span = maximum - minimum;
    if span <= f32::EPSILON {
        return 0.0;
    }
    let mut histogram = [0_usize; HISTOGRAM_BINS];
    for &value in values {
        let normalized = ((value - minimum) / span).clamp(0.0, 1.0);
        let bin = ((normalized * HISTOGRAM_BINS as f32) as usize).min(HISTOGRAM_BINS - 1);
        histogram[bin] += 1;
    }
    let sample_count = count_as_f64(values.len());
    histogram
        .into_iter()
        .filter(|&count| count != 0)
        .map(|count| {
            let probability = count_as_f64(count) / sample_count;
            -probability * probability.log2()
        })
        .sum()
}

#[allow(clippy::cast_precision_loss)]
fn count_as_f64(count: usize) -> f64 {
    // In-memory slices cannot approach the 2^53 threshold where this loses an
    // integer unit on supported desktop targets.
    count as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_distribution_is_well_defined() {
        let report = Distribution::measure(&[3.0; 8]);
        assert_eq!(report.minimum.to_bits(), 3.0_f32.to_bits());
        assert_eq!(report.maximum.to_bits(), 3.0_f32.to_bits());
        assert_eq!(report.standard_deviation.to_bits(), 0.0_f64.to_bits());
        assert_eq!(
            report.range_normalized_entropy_bits.to_bits(),
            0.0_f64.to_bits()
        );
    }

    #[test]
    fn quantiles_are_named_and_interpolated() {
        let report = Distribution::measure(&[0.0, 10.0, 20.0, 30.0, 40.0]);
        assert!((report.quantiles.p50 - 20.0).abs() < f32::EPSILON);
        assert!((report.quantiles.p25 - 10.0).abs() < f32::EPSILON);
        assert!((report.quantiles.p75 - 30.0).abs() < f32::EPSILON);
    }

    #[test]
    fn non_finite_samples_are_counted_but_not_summarized() {
        let report = Distribution::measure(&[1.0, f32::NAN, 3.0, f32::INFINITY]);
        assert_eq!(report.sample_count, 4);
        assert_eq!(report.finite_sample_count, 2);
        assert_eq!(report.non_finite_sample_count, 2);
        assert!((report.mean - 2.0).abs() < f64::EPSILON);
    }
}
