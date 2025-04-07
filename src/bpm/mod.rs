use itertools::Itertools;
use thiserror::Error;
use web_time::Instant;

#[derive(Error, Debug)]
pub enum BpmCalculationError {
    #[error("not enough data in input vector")]
    InsufficientData,
}

pub fn direct_count(timestamps: &[Instant]) -> Result<f64, BpmCalculationError> {
    if timestamps.len() < 2 {
        return Err(BpmCalculationError::InsufficientData);
    }

    let start = timestamps[0];
    let end = timestamps.last().unwrap();
    let delta = end.duration_since(start);
    // len - 1 is used so only one of start/end is counted
    let count = (timestamps.len() - 1) as f64;
    let bpm = count * 60_000_f64 / delta.as_millis() as f64;

    Ok(bpm)
}

pub fn simple_regression(timestamps: &[Instant]) -> Result<f64, BpmCalculationError> {
    // Slope of least squares regression line is equal to Cov(x, y) / Var(x)
    // https://seismo.berkeley.edu/~kirchner/eps_120/Toolkits/Toolkit_10.pdf
    if timestamps.len() < 2 {
        return Err(BpmCalculationError::InsufficientData);
    }

    let start = timestamps[0];
    let n = timestamps.len() as f64;

    let (sum_x, sum_x_squared, sum_xy) =
        timestamps
            .iter()
            .enumerate()
            .fold((0_u128, 0_u128, 0_u128), |(sx, sxx, sxy), (i, ts)| {
                let x = ts.duration_since(start).as_millis();
                (sx + x, sxx + x * x, sxy + (i as u128) * x)
            });

    let mean_x = sum_x as f64 / n;
    let mean_y = (n - 1_f64) / 2_f64;

    // beats per millisecond
    let slope =
        (sum_xy as f64 - n * mean_x * mean_y) / (sum_x_squared as f64 - n * mean_x * mean_x);

    Ok(slope * 60_000_f64)
}

pub fn thiel_sen(timestamps: &[Instant]) -> Result<f64, BpmCalculationError> {
    // The median of the slopes between every pair of points
    // Increased robustness, asymptotic efficiency (data required to converge)
    // https://en.wikipedia.org/wiki/Theil%E2%80%93Sen_estimator
    if timestamps.len() < 2 {
        return Err(BpmCalculationError::InsufficientData);
    }
    let start = timestamps[0];

    let mut slopes: Vec<_> = timestamps
        .iter()
        .map(|ts| ts.duration_since(start).as_millis())
        .enumerate()
        .tuple_combinations()
        // indices (number of beats) are the y-values
        .map(|((y1, x1), (y2, x2))| (y2 - y1) as f64 / (x2 - x1) as f64)
        .collect();
    let mid = slopes.len() / 2;
    let (_left, median, _right) = slopes.select_nth_unstable_by(mid, |a, b| a.total_cmp(b));

    Ok(*median * 60_000_f64)
}
