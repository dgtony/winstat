/// Instant buffer statistics as a result of adding new value.
pub struct InstantStat {
    pub mean: f64,
    pub stddev: f64,
}

/// Special buffer for computing second-order statistics
/// efficiently in the sliding window.
#[derive(Default)]
pub struct StatWindow {
    values: Vec<f64>,
    idx: usize,
    count: usize,
    mean: f64,
    var_sum: f64,
}

impl StatWindow {
    /// Create empty buffer with fixed window size.
    pub fn new(window_size: usize) -> Option<Self> {
        // windows lesser than 2 elements are nonsense
        if window_size < 2 {
            return None;
        }

        let mut buf = Vec::with_capacity(window_size);
        Vec::resize_with(&mut buf, window_size, Default::default);

        Some(StatWindow {
            values: buf,
            ..Default::default()
        })
    }

    /// Add a new value to the buffer.
    /// Method returns instant statistics of the buffer.
    pub fn push(&mut self, value: f64) -> InstantStat {
        // store new value in buffer
        let ejected_value = self.values[self.idx];
        self.values[self.idx] = value;
        self.move_idx();

        // compute statistics
        let (new_mean, new_var_sum, stddev) = if self.count < self.values.len() {
            self.count += 1;
            growing_phase(self.mean, self.var_sum, value, self.count)
        } else {
            sliding_phase(self.mean, self.var_sum, value, ejected_value, self.count)
        };

        self.mean = new_mean;
        self.var_sum = new_var_sum;

        InstantStat {
            mean: new_mean,
            stddev,
        }
    }

    fn move_idx(&mut self) {
        let new_idx = (self.idx + 1) % self.values.len();
        self.idx = new_idx;
    }
}

// Welford's online algorithm for computing variance
// in the growing array with O(1) complexity.
#[inline]
fn growing_phase(mean: f64, var_sum: f64, new_element: f64, count: usize) -> (f64, f64, f64) {
    if count < 2 {
        return (new_element, 0_f64, 0_f64);
    }

    let new_mean = mean + (new_element - mean) / count as f64;
    let new_var_sum = var_sum + (new_element - mean) * (new_element - new_mean);
    let sample_variance = new_var_sum / (count - 1) as f64;

    (new_mean, new_var_sum, sample_variance.sqrt())
}

// Modified algorithm for efficient variance
// computation in the sliding window.
#[inline]
fn sliding_phase(
    mean: f64,
    var_sum: f64,
    new_element: f64,
    ejected_element: f64,
    count: usize,
) -> (f64, f64, f64) {
    let new_mean = mean + (new_element - ejected_element) / count as f64;
    let new_var_sum = var_sum
        + (new_element - ejected_element) * (new_element + ejected_element - mean - new_mean);
    let sample_variance = new_var_sum / (count - 1) as f64;

    (new_mean, new_var_sum, sample_variance.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    // straightforward approach of computing mean and standard deviation
    fn exact_stat(arr: &[f64]) -> (f64, f64) {
        if arr.len() == 1 {
            return (arr[0], 0_f64);
        }

        let mean = arr.iter().fold(0_f64, |acc, &v| acc + v) / arr.len() as f64;
        let var =
            arr.iter().fold(0_f64, |acc, &v| acc + (v - mean).powi(2)) / (arr.len() - 1) as f64;
        (mean, var.sqrt())
    }

    fn in_delta(v1: f64, v2: f64, delta: f64) -> bool {
        (v1 - v2).abs() < delta
    }

    #[test]
    fn bad_windows() {
        assert!(StatWindow::new(0).is_none());
        assert!(StatWindow::new(1).is_none());
    }

    #[test]
    fn single_element() {
        let mut sw = StatWindow::new(10).unwrap();

        let val = 12.34;
        let result = sw.push(val);

        assert_eq!(result.mean, val);
        assert_eq!(result.stddev, 0_f64);
    }

    #[test]
    fn just_growing() {
        let max_err = 1e-12;
        let values: [f64; 4] = [1.0, 2.0, 4.0, 7.0];

        // reference buffer
        let mut growing_arr = Vec::new();
        let mut sw = StatWindow::new(values.len()).unwrap();

        for &v in values.iter() {
            let InstantStat { mean, stddev } = sw.push(v);

            growing_arr.push(v);
            let (em, es) = exact_stat(&growing_arr);

            assert!(
                in_delta(em, mean, max_err),
                "mean => exact: {}, online: {}",
                em,
                mean
            );
            assert!(
                in_delta(es, stddev, max_err),
                "standard deviation => exact: {}, online: {}",
                es,
                stddev
            );
        }
    }

    #[test]
    fn grow_and_slide() {
        let max_err = 1e-12;
        let win_size = 4_usize;
        let values: [f64; 10] = [1.0, 2.0, 4.0, 4.0, 7.2, 12.5, 2.8, 3.1, 65.3, 98.01];

        // reference buffer
        let mut growing_arr = Vec::new();
        let mut sw = StatWindow::new(win_size).unwrap();

        // growth
        for (step, &v) in values.iter().take(win_size).enumerate() {
            let InstantStat { mean, stddev } = sw.push(v);

            growing_arr.push(v);
            let (em, es) = exact_stat(&growing_arr);

            assert!(
                in_delta(em, mean, max_err),
                "growth phase ({}), mean => exact: {}, online: {}",
                step,
                em,
                mean
            );
            assert!(
                in_delta(es, stddev, max_err),
                "growth phase ({}), standard deviation => exact: {}, online: {}",
                step,
                es,
                stddev
            );
        }

        // sliding
        for (step, &v) in values.iter().skip(win_size).enumerate() {
            let InstantStat { mean, stddev } = sw.push(v);

            growing_arr.push(v);
            let (em, es) = exact_stat(&growing_arr[step + 1..]);

            assert!(
                in_delta(em, mean, max_err),
                "sliding phase ({}), mean => exact: {}, online: {}",
                step,
                em,
                mean
            );
            assert!(
                in_delta(es, stddev, max_err),
                "sliding phase ({}), standard deviation => exact: {}, online: {}",
                step,
                es,
                stddev
            );
        }
    }
}
