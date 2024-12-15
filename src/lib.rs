use chrono::{DateTime, Datelike, Duration, Utc};
use std::fmt;

/// Time iteration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeStep {
    /// Step by a given number of seconds
    Second(i64),
    /// Step by a given number of minutes
    Minute(i64),
    /// Step by a given number of hours
    Hour(i64),
    /// Step by a given number of days
    Day(i64),
}
impl From<TimeStep> for Duration {
    fn from(value: TimeStep) -> Self {
        match &value {
            TimeStep::Second(s) => Duration::seconds(*s),
            TimeStep::Minute(m) => Duration::minutes(*m),
            TimeStep::Hour(h) => Duration::hours(*h),
            TimeStep::Day(d) => Duration::days(*d),
        }
    }
}
impl TimeStep {
    /// Returns the total step size in seconds.
    pub fn as_total_seconds(&self) -> i64 {
        Duration::from(self.clone()).num_seconds()
    }
}

impl fmt::Display for TimeStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeStep::Second(s) => write!(f, "{} second(s)", s),
            TimeStep::Minute(m) => write!(f, "{} minute(s)", m),
            TimeStep::Hour(h) => write!(f, "{} hour(s)", h),
            TimeStep::Day(d) => write!(f, "{} day(s)", d),
        }
    }
}

/// Range of time to iterate over.
pub struct TimeRange {
    /// Start time
    pub start: DateTime<Utc>,
    /// End time
    pub end: DateTime<Utc>,
    /// Step to increment by each iteration
    pub step: TimeStep,
    /// Whether to skip weekends (Saturday and Sunday)
    pub skip_weekends: bool,
    /// Optional  filter function to skip certain times.
    pub filter: Option<Box<dyn Fn(DateTime<Utc>) -> bool + Send + Sync>>,
}

pub struct TimeRangeIter {
    current: DateTime<Utc>,
    end: DateTime<Utc>,
    step: Duration,
    skip_weekends: bool,
    filter: Option<Box<dyn Fn(DateTime<Utc>) -> bool + Send + Sync>>,
}

impl Iterator for TimeRangeIter {
    type Item = DateTime<Utc>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current >= self.end {
                return None;
            }

            let candidate = self.current;
            self.current = self.current + self.step;

            if self.skip_weekends {
                let mut day_candidate = candidate;
                while (day_candidate.weekday().number_from_monday() == 6
                    || day_candidate.weekday().number_from_monday() == 7)
                    && day_candidate < self.end
                {
                    day_candidate = day_candidate + self.step;
                }
                if day_candidate != candidate {
                    self.current = day_candidate + self.step;
                    if day_candidate < self.end {
                        if let Some(ref f) = self.filter {
                            if f(day_candidate) {
                                return Some(day_candidate);
                            } else {
                                continue;
                            }
                        } else {
                            return Some(day_candidate);
                        }
                    } else {
                        return None;
                    }
                } else {
                    if let Some(ref f) = self.filter {
                        if f(candidate) {
                            return Some(candidate);
                        } else {
                            continue;
                        }
                    } else {
                        return Some(candidate);
                    }
                }
            } else {
                if let Some(ref f) = self.filter {
                    if f(candidate) {
                        return Some(candidate);
                    } else {
                        continue;
                    }
                } else {
                    return Some(candidate);
                }
            }
        }
    }
}

impl IntoIterator for TimeRange {
    type Item = DateTime<Utc>;
    type IntoIter = TimeRangeIter;

    fn into_iter(self) -> Self::IntoIter {
        TimeRangeIter {
            current: self.start,
            end: self.end,
            step: self.step.into(),
            skip_weekends: self.skip_weekends,
            filter: self.filter,
        }
    }
}

/// A builder to create `TimeRange`.
pub struct TimeRangeBuilder {
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
    step: Option<TimeStep>,
    skip_weekends: bool,
    filter: Option<Box<dyn Fn(DateTime<Utc>) -> bool + Send + Sync>>,
}

impl TimeRangeBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            start: None,
            end: None,
            step: None,
            skip_weekends: false,
            filter: None,
        }
    }

    /// Set the start time.
    pub fn start(mut self, start: DateTime<Utc>) -> Self {
        self.start = Some(start);
        self
    }

    /// Set the end time.
    pub fn end(mut self, end: DateTime<Utc>) -> Self {
        self.end = Some(end);
        self
    }

    /// Set the step.
    pub fn step(mut self, step: TimeStep) -> Self {
        self.step = Some(step);
        self
    }

    /// Whether to skip weekends.
    pub fn skip_weekends(mut self, skip: bool) -> Self {
        self.skip_weekends = skip;
        self
    }

    /// Provide a custom filter function to skip certain times.
    /// For example, you can skip holidays or specific conditions.
    pub fn filter<F>(mut self, f: F) -> Self
    where
        F: Fn(DateTime<Utc>) -> bool + Send + Sync + 'static,
    {
        self.filter = Some(Box::new(f));
        self
    }

    /// Build the `TimeRange`.
    pub fn build(self) -> Result<TimeRange, &'static str> {
        let start = self.start.ok_or("start time is required")?;
        let end = self.end.ok_or("end time is required")?;
        let step = self.step.ok_or("step is required")?;

        if end <= start {
            return Err("end time must be after start time");
        }

        Ok(TimeRange {
            start,
            end,
            step,
            skip_weekends: self.skip_weekends,
            filter: self.filter,
        })
    }
}

pub trait ComputeTimeRange {
    /// Compute the total number of steps in the range, after applying weekend skipping and filter.
    fn total_steps(&self) -> usize;

    /// Compute the total duration in seconds of all yielded steps.
    fn total_duration_in_seconds(&self) -> i64;
}

impl ComputeTimeRange for TimeRange {
    fn total_steps(&self) -> usize {
        let mut count = 0usize;
        let mut current = self.start;
        let step_duration = Duration::from(self.step);

        while current < self.end {
            let mut candidate = current;

            if self.skip_weekends {
                while (candidate.weekday().number_from_monday() == 6
                    || candidate.weekday().number_from_monday() == 7)
                    && candidate < self.end
                {
                    candidate = candidate + step_duration;
                }
            }

            if candidate >= self.end {
                break;
            }

            if let Some(ref f) = self.filter {
                if !f(candidate) {
                    current = candidate + step_duration;
                    continue;
                }
            }

            count += 1;
            current = candidate + step_duration;
        }

        count
    }

    fn total_duration_in_seconds(&self) -> i64 {
        let steps = self.total_steps();
        steps as i64 * self.step.as_total_seconds()
    }
}
