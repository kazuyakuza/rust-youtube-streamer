//! Histogram aggregation pins: count/min/max/mean, the fixed bucket ladder,
//! the unbounded final bucket, and total saturation.

use super::histogram::*;

fn record_all(histogram: &mut DurationHistogram, samples: &[u64]) {
    for sample in samples {
        histogram.record(*sample);
    }
}

#[test]
fn records_count_min_max_mean() {
    let mut histogram = DurationHistogram::new();
    record_all(&mut histogram, &[10, 20, 30]);
    let summary = histogram.snapshot();
    assert_eq!(summary.count, 3);
    assert_eq!(summary.min_ns, Some(10));
    assert_eq!(summary.max_ns, 30);
    assert_eq!(summary.mean_ns, 20);
}

#[test]
fn buckets_fit_fixed_ladder() {
    let mut histogram = DurationHistogram::new();
    record_all(&mut histogram, &[0, 999_999, 1_000_000]);
    let summary = histogram.snapshot();
    assert_eq!(summary.buckets[0].2, 2, "[0,1ms) holds 0 and 999_999");
    assert_eq!(summary.buckets[1].2, 1, "[1ms,5ms) holds exactly 1ms");
    assert_eq!(summary.buckets.len(), HISTOGRAM_BUCKET_COUNT);
    assert_eq!(summary.buckets[0].0, 0);
    assert_eq!(summary.buckets[0].1, Some(HISTOGRAM_BOUNDARIES_NS[0]));
}

#[test]
fn over_5s_lands_in_last_bucket() {
    let mut histogram = DurationHistogram::new();
    histogram.record(5_000_000_001);
    let summary = histogram.snapshot();
    let last = summary.buckets[HISTOGRAM_BUCKET_COUNT - 1];
    assert_eq!(last.2, 1);
    assert_eq!(last.1, None, "the final bucket is unbounded");
    assert_eq!(last.0, HISTOGRAM_BOUNDARIES_NS[7]);
}

#[test]
fn total_saturates_without_panic() {
    let mut histogram = DurationHistogram::new();
    record_all(&mut histogram, &[u64::MAX, 1]);
    let summary = histogram.snapshot();
    assert_eq!(summary.count, 2);
    assert_eq!(summary.min_ns, Some(1));
    assert_eq!(summary.max_ns, u64::MAX);
    assert_eq!(summary.mean_ns, u64::MAX / 2);
}
