use std::ffi::c_longlong;
use std::ops::{Add, Sub};
use std::time::Duration;

/// Timestamp of a frame, relative to the start of the stream.
#[repr(transparent)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct Timestamp(u64);

impl Timestamp {
    /// Timestamp equaling `0`.
    pub const ZERO: Self = Self(0);

    /// Creates a new timestamp from the given number of milliseconds.
    #[must_use]
    pub const fn from_millis(ts: u64) -> Self {
        Self(ts)
    }

    /// The time of this timestamp in milliseconds.
    #[must_use]
    pub const fn as_millis(self) -> u64 {
        self.0
    }

    pub(crate) fn as_native(self) -> c_longlong {
        self.0
            .try_into()
            .expect("Could not convert u64 timestamp into native timestamp")
    }
}

impl Sub for Timestamp {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        let delta_ms = self.0 - rhs.0;
        Duration::from_millis(delta_ms)
    }
}

impl Add<Duration> for Timestamp {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        let rhs_u64: u64 = rhs
            .as_millis()
            .try_into()
            .expect("Overflow when adding duration to timestamp");

        Self(self.0 + rhs_u64)
    }
}

#[cfg(test)]
mod test {
    use super::Timestamp;
    use std::time::Duration;

    #[test]
    fn timestamps_work() {
        let a = Timestamp::from_millis(0);
        let b = Timestamp::from_millis(100);
        let c = b + Duration::from_millis(100);

        assert_eq!((b - a).as_millis(), 100);
        assert_eq!(c.as_millis(), 200);
    }
}
