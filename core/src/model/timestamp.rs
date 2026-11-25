// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

//! A UTC instant with second precision, formatted as RFC 3339.

use core::fmt;
use core::ops::Range;
use core::time::Duration;

/// Seconds since the Unix epoch, UTC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(i64);

/// A proleptic Gregorian date.
#[derive(PartialEq, Eq)]
struct CivilDate {
    pub year: i64,
    pub month: i64,
    pub day: i64,
}

impl Timestamp {
    /// From seconds since the epoch.
    pub const fn from_unix(seconds: i64) -> Self {
        Self(seconds)
    }

    /// Parses `YYYY-MM-DDTHH:MM:SSZ`: every field unsigned digits, and the date one its month has.
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
        let num = |range: Range<usize>| {
            let field = text.get(range)?;
            field
                .bytes()
                .all(|byte| byte.is_ascii_digit())
                .then(|| field.parse::<i64>().ok())
                .flatten()
        };
        let date = CivilDate {
            year: num(0..4)?,
            month: num(5..7)?,
            day: num(8..10)?,
        };
        let (hour, minute, second) = (num(11..13)?, num(14..16)?, num(17..19)?);
        let days = Self::days_from_civil(&date);
        if Self::civil_from_days(days) != date || hour > 23 || minute > 59 || second > 59 {
            return None;
        }
        Some(Self(days * 86_400 + hour * 3600 + minute * 60 + second))
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

    /// Seconds since the epoch.
    pub const fn unix(self) -> i64 {
        self.0
    }

    // Howard Hinnant's civil-date algorithms.
    fn civil_from_days(days: i64) -> CivilDate {
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = doy - (153 * mp + 2) / 5 + 1;
        let month = if mp < 10 { mp + 3 } else { mp - 9 };
        let year = yoe + era * 400 + i64::from(month <= 2);
        CivilDate { year, month, day }
    }

    fn days_from_civil(date: &CivilDate) -> i64 {
        let y = if date.month <= 2 {
            date.year - 1
        } else {
            date.year
        };
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let mp = (date.month + 9) % 12;
        let doy = (153 * mp + 2) / 5 + date.day - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let date = Self::civil_from_days(self.0.div_euclid(86_400));
        let secs = self.0.rem_euclid(86_400);
        write!(
            f,
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            date.year,
            date.month,
            date.day,
            secs / 3600,
            secs % 3600 / 60,
            secs % 60
        )
    }
}
