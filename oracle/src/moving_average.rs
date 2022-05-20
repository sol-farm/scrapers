//! provides a set of traits, and functions that can be used to implement
//! a moving average over some arbitrary dataset. this is probably not a true moving average
//! but it has worked sufficiently for tulip v1 leverage farming

use anyhow::{anyhow, Result};
use average::{Estimate, Mean};
use chrono::prelude::*;
use chrono::DateTime;

pub trait MovingAverage {
    /// returns a new implementation of the MovingAverage calculator
    /// if no rates have been observed, provide an empty vec (vec![])
    /// as the input to `period_observed_rates`
    fn new(
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        period_observed_rates: Vec<f64>,
    ) -> Self;
    /// returns "ErrPeriodFinished" if attempting to observe a value
    /// which is outside the current period
    fn observe_value(&mut self, price: f64) -> Result<f64>;
    /// computes the current moving average based off the observed values
    /// updating the internal running average if the new result differs
    /// from the current stored result
    fn compute(&mut self) -> f64;
    /// returns the current running average
    fn moving_average(&self) -> f64;
    /// returns all the observed values
    fn observed_values(&self) -> Vec<f64>;
}

pub struct MovingAverageCalculator {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub period_running_average: f64,
    pub period_observed_values: Vec<f64>,
}

impl MovingAverage for MovingAverageCalculator {
    fn new(
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        period_observed_values: Vec<f64>,
    ) -> Self {
        MovingAverageCalculator {
            period_start,
            period_end,
            period_running_average: 0_f64,
            period_observed_values,
        }
    }
    fn observe_value(&mut self, value: f64) -> Result<f64> {
        let now = Utc::now().naive_utc();
        if now.gt(&self.period_end.naive_utc()) {
            return Err(anyhow!("ErrPeriodFinished"));
        }
        self.period_observed_values.push(value);
        // compute the new running average
        Ok(self.compute())
    }
    fn compute(&mut self) -> f64 {
        let estimator: Mean = (&self.period_observed_values).iter().collect();
        let average = estimator.estimate();
        if average != self.period_running_average {
            self.period_running_average = average;
        }
        average
    }
    fn moving_average(&self) -> f64 {
        self.period_running_average
    }
    fn observed_values(&self) -> Vec<f64> {
        self.period_observed_values.clone()
    }
}

#[cfg(test)]
mod test {
    use chrono::Duration;

    use super::*;
    #[test]
    fn test_moving_average_calculator() {
        let period_start = Utc::now();
        let period_end = period_start + Duration::seconds(10);
        let mut calculator = MovingAverageCalculator::new(period_start, period_end, vec![]);

        let average = calculator.observe_value(420_f64).unwrap();
        assert_eq!(average, 420_f64);

        let average = calculator.observe_value(69_f64).unwrap();
        assert_eq!(average, 244.5);

        let average = calculator.observe_value(420_f64).unwrap();
        assert_eq!(average, 303_f64);

        std::thread::sleep(std::time::Duration::from_secs(11));

        let average = calculator.observe_value(1337_f64);
        assert!(average.is_err());
        assert_eq!(
            average.err().unwrap().to_string(),
            "ErrPeriodFinished".to_string()
        );
    }
}
