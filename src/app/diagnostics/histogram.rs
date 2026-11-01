//! Bounded duration aggregation: one fixed 9-bucket ladder per measured
//! duration plus count/min/max/mean. Recording is O(1) time and memory, and
//! no sample value is retained, so the aggregate can never grow.

use serde::Serialize;

pub(crate) const HISTOGRAM_BOUNDARIES_NS: [u64; 8] = [
    1_000_000,
    5_000_000,
    10_000_000,
    50_000_000,
    100_000_000,
    500_000_000,
    1_000_000_000,
    5_000_000_000,
];

pub(crate) const HISTOGRAM_BUCKET_COUNT: usize = 9;

/// `(lower_ns, upper_ns, count)`; a `None` upper bound is the final bucket.
pub(crate) type HistogramBuckets = [(u64, Option<u64>, u64); HISTOGRAM_BUCKET_COUNT];

/// The bounded snapshot of one histogram.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct HistogramSummary {
    pub(crate) count: u64,
    pub(crate) min_ns: Option<u64>,
    pub(crate) max_ns: u64,
    pub(crate) mean_ns: u64,
    pub(crate) buckets: HistogramBuckets,
}

/// Fixed-size duration histogram: counters only, no sample storage.
pub(crate) struct DurationHistogram {
    count: u64,
    min_ns: Option<u64>,
    max_ns: u64,
    total_ns: u64,
    buckets: [u64; HISTOGRAM_BUCKET_COUNT],
}

impl DurationHistogram {
    pub(crate) fn new() -> Self {
        Self {
            count: 0,
            min_ns: None,
            max_ns: 0,
            total_ns: 0,
            buckets: [0; HISTOGRAM_BUCKET_COUNT],
        }
    }

    /// Records one duration sample; saturating arithmetic keeps the
    /// aggregate total from wrapping on pathological inputs.
    pub(crate) fn record(&mut self, nanos: u64) {
        self.count = self.count.saturating_add(1);
        self.min_ns = Some(self.min_ns.map_or(nanos, |current| current.min(nanos)));
        self.max_ns = self.max_ns.max(nanos);
        self.total_ns = self.total_ns.saturating_add(nanos);
        let bucket = bucket_index(nanos);
        self.buckets[bucket] = self.buckets[bucket].saturating_add(1);
    }

    pub(crate) fn snapshot(&self) -> HistogramSummary {
        HistogramSummary {
            count: self.count,
            min_ns: self.min_ns,
            max_ns: self.max_ns,
            mean_ns: mean_nanos(self.total_ns, self.count),
            buckets: bucket_snapshot(&self.buckets),
        }
    }
}

fn mean_nanos(total_ns: u64, count: u64) -> u64 {
    if count == 0 {
        return 0;
    }
    total_ns / count
}

fn bucket_index(nanos: u64) -> usize {
    HISTOGRAM_BOUNDARIES_NS
        .iter()
        .position(|boundary| nanos < *boundary)
        .unwrap_or(HISTOGRAM_BUCKET_COUNT - 1)
}

fn bucket_snapshot(buckets: &[u64; HISTOGRAM_BUCKET_COUNT]) -> HistogramBuckets {
    let mut summary = [(0, None, 0); HISTOGRAM_BUCKET_COUNT];
    for (index, count) in buckets.iter().enumerate() {
        summary[index] = bucket_bounds(index).into_bucket(*count);
    }
    summary
}

/// The inclusive lower / exclusive upper bound pair for one bucket index.
struct BucketBounds {
    lower_ns: u64,
    upper_ns: Option<u64>,
}

impl BucketBounds {
    fn into_bucket(self, count: u64) -> (u64, Option<u64>, u64) {
        (self.lower_ns, self.upper_ns, count)
    }
}

fn bucket_bounds(index: usize) -> BucketBounds {
    let lower_ns = if index == 0 {
        0
    } else {
        HISTOGRAM_BOUNDARIES_NS[index - 1]
    };
    BucketBounds {
        lower_ns,
        upper_ns: HISTOGRAM_BOUNDARIES_NS.get(index).copied(),
    }
}
