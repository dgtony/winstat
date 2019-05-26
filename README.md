# winstat - statistics in window

Computing second-order statistics online in the sliding window.
Package provides utility to efficiently generate instant basic statistics,
such as a mean and standard deviation for sliding window in the online manner.


### Usage
To start computing statistics you need at first to create a StatWindow
with a reasonable window size (greater than one).
After that you can add new values to the buffer one-by-one as they arrive
and get instant statistics for the given window in return.

```rust
use winstat::StatWindow;

// static window for 5 elements
let window_size = 5;

// create estimator
let mut sw = StatWindow::new(window_size);

// ensure it was created
if sw.is_none() { println!("bad window size") }

// add values on the fly
let values: [f64; 8] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];

for &v in values.iter() {
    let (mean, stddev) = sw.push(v);
    println!("add {}, window stats => mean: {}, standard deviation: {}", v, mean, stddev);
}
```
