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

    let start = timestamps.first().expect("timestamps should not be empty");
    let end = timestamps.last().expect("timestamps should not be empty");
    let delta = end.duration_since(*start);

    // len - 1 is used so only one of start/end is counted
    let count = (timestamps.len() - 1) as f64;
    let bpm = count * 60_000_f64 / delta.as_millis() as f64;

    Ok(bpm)
}
