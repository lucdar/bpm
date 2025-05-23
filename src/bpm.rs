use itertools::Itertools;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BpmCalculationError {
    #[error("not enough data in input vector")]
    InsufficientData,
}

pub fn direct_count(offsets: &[u64]) -> Result<f64, BpmCalculationError> {
    if offsets.len() < 2 {
        return Err(BpmCalculationError::InsufficientData);
    }

    let start = offsets[0];
    let end = offsets.last().unwrap();
    let delta = end - start;
    // len - 1 is used so only one of start/end is counted
    let count = (offsets.len() - 1) as f64;
    let bpm = count * 60_000_f64 / delta as f64;

    Ok(bpm)
}

pub fn simple_regression(offsets: &[u64]) -> Result<f64, BpmCalculationError> {
    // Slope of least squares regression line is equal to Cov(x, y) / Var(x)
    // https://seismo.berkeley.edu/~kirchner/eps_120/Toolkits/Toolkit_10.pdf
    if offsets.len() < 2 {
        return Err(BpmCalculationError::InsufficientData);
    }

    let (sum_x, sum_x_squared, sum_xy) = offsets
        .iter()
        .enumerate()
        .fold((0_u64, 0_u64, 0_u64), |(sx, sxx, sxy), (y, x)| {
            (sx + x, sxx + x * x, sxy + (y as u64) * x)
        });

    let n = offsets.len() as f64;
    let mean_x = sum_x as f64 / n;
    let mean_y = (n - 1_f64) / 2_f64;

    let slope = // beats per millisecond
        (sum_xy as f64 - n * mean_x * mean_y) /
        (sum_x_squared as f64 - n * mean_x * mean_x);

    Ok(slope * 60_000_f64)
}

pub fn thiel_sen(offsets: &[u64]) -> Result<f64, BpmCalculationError> {
    // The median of the slopes between every pair of points
    // Increased robustness, asymptotic efficiency (data required to converge)
    // https://en.wikipedia.org/wiki/Theil%E2%80%93Sen_estimator
    if offsets.len() < 2 {
        return Err(BpmCalculationError::InsufficientData);
    }

    let mut slopes: Vec<_> = offsets
        .iter()
        .enumerate()
        .tuple_combinations()
        // indices (number of beats) are the y-values
        .map(|((y1, x1), (y2, x2))| (y2 - y1) as f64 / (x2 - x1) as f64)
        .collect();
    let mid = slopes.len() / 2;
    let (_left, median, _right) = slopes.select_nth_unstable_by(mid, |a, b| a.total_cmp(b));

    Ok(*median * 60_000_f64)
}
