//! A UTC instant with second precision, formatted as RFC 3339.

use std::fmt;
use std::time::Duration;

/// Seconds since the Unix epoch, UTC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(i64);

impl Timestamp {
    /// From seconds since the epoch.
    pub const fn from_unix(seconds: i64) -> Self {
        Self(seconds)
    }

    /// Seconds since the epoch.
    pub const fn unix(self) -> i64 {
        self.0
    }

    /// This instant plus a duration (saturating).
    #[must_use]
    pub fn plus(self, duration: Duration) -> Self {
        let seconds = i64::try_from(duration.as_secs()).unwrap_or(i64::MAX);
        Self(self.0.saturating_add(seconds))
    }

    /// The time from `earlier` to this instant; zero if `earlier` is later.
    pub fn since(self, earlier: Self) -> Duration {
        Duration::from_secs(u64::try_from(self.0 - earlier.0).unwrap_or(0))
    }

    /// Parses `YYYY-MM-DDTHH:MM:SSZ`.
    pub fn parse(text: &str) -> Option<Self> {
        let b = text.as_bytes();
        if b.len() != 20
            || b[4] != b'-'
            || b[7] != b'-'
            || b[10] != b'T'
            || b[13] != b':'
            || b[16] != b':'
            || b[19] != b'Z'
        {
            return None;
        }
        let num = |range: std::ops::Range<usize>| text.get(range)?.parse::<i64>().ok();
        let (year, month, day) = (num(0..4)?, num(5..7)?, num(8..10)?);
        let (hour, minute, second) = (num(11..13)?, num(14..16)?, num(17..19)?);
        if !(1..=12).contains(&month)
            || !(1..=31).contains(&day)
            || hour > 23
            || minute > 59
            || second > 59
        {
            return None;
        }
        let days = days_from_civil(year, month, day);
        Some(Self(days * 86_400 + hour * 3600 + minute * 60 + second))
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let days = self.0.div_euclid(86_400);
        let secs = self.0.rem_euclid(86_400);
        let (year, month, day) = civil_from_days(days);
        write!(
            f,
            "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
            secs / 3600,
            secs % 3600 / 60,
            secs % 60
        )
    }
}

// Howard Hinnant's civil-date algorithms.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_known_instants() {
        assert_eq!(Timestamp::from_unix(0).to_string(), "1970-01-01T00:00:00Z");
        assert_eq!(
            Timestamp::from_unix(1_792_800_000).to_string(),
            "2026-10-24T00:00:00Z"
        );
        assert_eq!(
            Timestamp::from_unix(951_782_400).to_string(),
            "2000-02-29T00:00:00Z"
        );
    }

    #[test]
    fn parse_inverts_display() {
        for seconds in [0, 59, 951_782_400, 1_792_812_345, 4_102_444_799] {
            let t = Timestamp::from_unix(seconds);
            assert_eq!(Timestamp::parse(&t.to_string()), Some(t));
        }
    }

    #[test]
    fn parse_rejects_malformed_text() {
        for text in [
            "",
            "2026-10-24",
            "2026-13-01T00:00:00Z",
            "2026-10-24T24:00:00Z",
            "2026-10-24 00:00:00Z",
        ] {
            assert_eq!(Timestamp::parse(text), None, "{text}");
        }
    }

    #[test]
    fn arithmetic_saturates_and_clamps() {
        let t = Timestamp::from_unix(100);
        assert_eq!(t.plus(Duration::from_secs(5)).unix(), 105);
        assert_eq!(t.since(Timestamp::from_unix(40)), Duration::from_secs(60));
        assert_eq!(Timestamp::from_unix(40).since(t), Duration::ZERO);
    }
}
