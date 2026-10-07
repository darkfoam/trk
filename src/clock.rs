use chrono::{DateTime, FixedOffset, Local};

/// Source of "now". Passed explicitly so operations stay pure and tests can
/// pin time with `TRK_NOW`.
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<FixedOffset>;
}

pub struct RealClock;

impl Clock for RealClock {
    fn now(&self) -> DateTime<FixedOffset> {
        Local::now().fixed_offset()
    }
}

pub struct FixedClock {
    now: DateTime<FixedOffset>,
}

impl FixedClock {
    pub fn new(now: DateTime<FixedOffset>) -> Self {
        Self { now }
    }
}

impl Clock for FixedClock {
    fn now(&self) -> DateTime<FixedOffset> {
        self.now
    }
}

/// Real clock unless `TRK_NOW` names an RFC 3339 instant (test-only).
pub fn system_clock() -> Box<dyn Clock> {
    if let Ok(raw) = std::env::var("TRK_NOW")
        && let Ok(dt) = DateTime::parse_from_rfc3339(raw.trim())
    {
        return Box::new(FixedClock::new(dt));
    }
    Box::new(RealClock)
}
