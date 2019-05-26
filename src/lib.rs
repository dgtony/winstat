//! Computing second-order statistics online in sliding window.
//!
//! Package provides utility to efficiently generate instant basic statistics,
//! such as a mean and standard deviation for sliding window in the online manner.
//!
//! # Quick Start
//!
//! To start computing statistics you need at first to create a StatWindow
//! with a reasonable window size (greater than one).
//! After that you can add new values to the buffer one-by-one as they arrive
//! and get instant statistics for the given window in return.
//!
//! ```
//! use winstat::StatWindow;
//!
//! // static window for 5 elements
//! let window_size = 5;
//!
//! // create estimator
//! let mut sw = StatWindow::new(window_size);
//!
//! // ensure it was created
//! if sw.is_none() { println!("bad window size") }
//!
//! // add values on the fly
//! let values: [f64; 8] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
//! for &v in values.iter() {
//!     let (mean, stddev) = sw.push(v);
//!     println!("add {}, window stats => mean: {}, standard deviation: {}", v, mean, stddev);
//! }
//! ```
//!
//! Computation is pretty efficient in terms of both space and time complexity.
//! Adding a new element is *O(1)* operation, while memory usage is proportional
//! to the window size and there are no allocations taking place after buffer
//! was initially created.
//!
//! Under the hood statistics estimator operates in the two phases:
//! 1. growing buffer
//! 2. sliding window
//!
//! In the first phase window is not yet filled with elements and it grows
//! as new elements are added. Here we use
//! [Welford's online algorithm](https://en.wikipedia.org/wiki/Algorithms_for_calculating_variance)
//! to estimate statistics in the buffer.
//!
//! When window is full we need to rotate it on each element push, ejecting
//! oldest element from the buffer. In the second phase we use custom modified
//! online algorithm for statistics estimation, allowing us to eliminate
//! contribution of previously removed elements.
//!

mod estimator;

// re-export
pub use estimator::{InstantStat, StatWindow};

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::repeat;

    #[test]
    fn constant_series() {
        let mut sw = StatWindow::new(5).unwrap();

        let value: f64 = 5.2;
        let res: Vec<InstantStat> = repeat(value).take(10).map(|v| sw.push(v)).collect();

        for s in res.iter() {
            assert_eq!(s.mean, value);
            assert_eq!(s.stddev, 0.0);
        }
    }
}
