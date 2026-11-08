//! When an answer of HTTP 429 asks to be asked again: `Retry-After`, and the time crates.io names
//! in cargo's error.
//!
//! RFC 9110 lets `Retry-After` be a number of seconds or an HTTP date. A sender must write a date as
//! an IMF-fixdate, `Sun, 06 Nov 1994 08:49:37 GMT`, and a recipient must read the two obsolete
//! forms as well: RFC 850's, `Sunday, 06-Nov-94 08:49:37 GMT`, and C's asctime, `Sun Nov  6
//! 08:49:37 1994`. crates.io names its time to try again as an IMF-fixdate, inside the message cargo
//! prints. RFC 850's year has two digits, and RFC 9110 has a recipient read one that looks more than
//! 50 years ahead as the most recent past year with those digits, so that form is resolved against
//! the clock's wall time when the wait is measured, and a date's own parse needs no clock.

#[cfg(test)]
use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// How long to wait after a 429 that names no time to try again, or names one this cannot read.
pub const UNNAMED_WAIT: Duration = Duration::from_secs(60);

const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const LONG_WEEKDAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// A day and a time of day, as a date names them, but for the year.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DayAndTime {
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
}

impl DayAndTime {
    /// Seconds since the Unix epoch, in `year`.
    fn seconds_in(self, year: i64) -> i64 {
        days_from_civil(year, self.month, self.day) * 86_400
            + i64::from(self.hour) * 3_600
            + i64::from(self.minute) * 60
            + i64::from(self.second)
    }
}

/// A moment an HTTP date names, to the second.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HttpDate {
    /// An IMF-fixdate or an asctime date, which name the year whole: seconds since the Unix epoch.
    Whole(i64),
    /// An RFC 850 date, which names the year by its last two digits, resolved when it is read.
    TwoDigitYear { year: u32, rest: DayAndTime },
}

impl HttpDate {
    /// Reads an HTTP date in any of its three forms, and `None` for anything else. The day's name
    /// must be a day's name, and is not checked against the date. An IMF-fixdate's day of the month
    /// may also be one digit, as the Python publisher's reading of crates.io's message took it.
    pub fn parse(text: &str) -> Option<Self> {
        Self::imf_fixdate(text)
            .or_else(|| Self::rfc_850(text))
            .or_else(|| Self::asctime(text))
    }

    /// `Sun, 06 Nov 1994 08:49:37 GMT`.
    fn imf_fixdate(text: &str) -> Option<Self> {
        let (weekday, rest) = text.split_once(", ")?;
        if !WEEKDAYS.contains(&weekday) {
            return None;
        }
        let mut words = rest.split(' ');
        let day = number(words.next()?, 1..=2)?;
        let month = month(words.next()?)?;
        let year = number(words.next()?, 4..=4)?;
        let rest = day_and_time(month, day, words.next()?, i64::from(year))?;
        if words.next() != Some("GMT") || words.next().is_some() {
            return None;
        }
        Some(Self::Whole(rest.seconds_in(i64::from(year))))
    }

    /// `Sunday, 06-Nov-94 08:49:37 GMT`.
    fn rfc_850(text: &str) -> Option<Self> {
        let (weekday, rest) = text.split_once(", ")?;
        if !LONG_WEEKDAYS.contains(&weekday) {
            return None;
        }
        let mut words = rest.split(' ');
        let mut date = words.next()?.split('-');
        let day = number(date.next()?, 2..=2)?;
        let month = month(date.next()?)?;
        let year = number(date.next()?, 2..=2)?;
        if date.next().is_some() {
            return None;
        }
        // A leap day is checked against a year with these last two digits; 00 is 2000's.
        let rest = day_and_time(month, day, words.next()?, 2000 + i64::from(year))?;
        if words.next() != Some("GMT") || words.next().is_some() {
            return None;
        }
        Some(Self::TwoDigitYear { year, rest })
    }

    /// `Sun Nov  6 08:49:37 1994`, its day of the month padded with a space or a zero.
    fn asctime(text: &str) -> Option<Self> {
        let (weekday, rest) = text.split_once(' ')?;
        if !WEEKDAYS.contains(&weekday) {
            return None;
        }
        let (month_name, rest) = rest.split_once(' ')?;
        let month = month(month_name)?;
        let rest = rest.strip_prefix(' ').unwrap_or(rest);
        let mut words = rest.split(' ');
        let day = number(words.next()?, 1..=2)?;
        let clock = words.next()?;
        let year = number(words.next()?, 4..=4)?;
        if words.next().is_some() {
            return None;
        }
        let rest = day_and_time(month, day, clock, i64::from(year))?;
        Some(Self::Whole(rest.seconds_in(i64::from(year))))
    }

    /// The moment the date names, read at the wall clock's `now`: an RFC 850 year in the century
    /// of `now`, unless that is more than 50 years ahead of it, when it is the century before.
    pub fn time(self, now: SystemTime) -> SystemTime {
        let seconds = match self {
            Self::Whole(seconds) => seconds,
            Self::TwoDigitYear { year, rest } => {
                let (this_year, _, _) = civil_from_days(seconds_of(now).div_euclid(86_400));
                let mut resolved = this_year - this_year.rem_euclid(100) + i64::from(year);
                if resolved > this_year + 50 {
                    resolved -= 100;
                }
                rest.seconds_in(resolved)
            }
        };
        let magnitude = Duration::from_secs(seconds.unsigned_abs());
        if seconds >= 0 {
            UNIX_EPOCH + magnitude
        } else {
            UNIX_EPOCH - magnitude
        }
    }
}

/// Seconds since the Unix epoch of `time`, to the second below it.
fn seconds_of(time: SystemTime) -> i64 {
    match time.duration_since(UNIX_EPOCH) {
        Ok(after) => i64::try_from(after.as_secs()).unwrap_or(i64::MAX),
        Err(before) => {
            let before = before.duration();
            let whole = i64::try_from(before.as_secs()).unwrap_or(i64::MAX);
            if before.subsec_nanos() == 0 {
                -whole
            } else {
                -whole - 1
            }
        }
    }
}

/// `time` as an IMF-fixdate, as crates.io writes one. The tests name their answers' times with it.
#[cfg(test)]
pub fn imf_fixdate(time: SystemTime) -> String {
    Fixdate(seconds_of(time)).to_string()
}

#[cfg(test)]
struct Fixdate(i64);

#[cfg(test)]
impl fmt::Display for Fixdate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let days = self.0.div_euclid(86_400);
        let within = self.0.rem_euclid(86_400);
        let (year, month, day) = civil_from_days(days);
        // 1970-01-01 was a Thursday.
        let weekday = match (days + 4).rem_euclid(7) {
            0 => "Sun",
            1 => "Mon",
            2 => "Tue",
            3 => "Wed",
            4 => "Thu",
            5 => "Fri",
            _ => "Sat",
        };
        let month = match month {
            1 => "Jan",
            2 => "Feb",
            3 => "Mar",
            4 => "Apr",
            5 => "May",
            6 => "Jun",
            7 => "Jul",
            8 => "Aug",
            9 => "Sep",
            10 => "Oct",
            11 => "Nov",
            _ => "Dec",
        };
        write!(
            formatter,
            "{weekday}, {day:02} {month} {year:04} {:02}:{:02}:{:02} GMT",
            within / 3_600,
            within % 3_600 / 60,
            within % 60
        )
    }
}

/// A run of ASCII digits whose count is in `digits`, read as a number.
fn number(text: &str, digits: std::ops::RangeInclusive<usize>) -> Option<u32> {
    if !digits.contains(&text.len()) || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

/// A month's three-letter name, as its number from 1.
fn month(name: &str) -> Option<u32> {
    let index = MONTHS.iter().position(|month| *month == name)?;
    u32::try_from(index).ok().map(|index| index + 1)
}

/// `HH:MM:SS` on `day` of `month`, checked against the days `year`'s month has.
fn day_and_time(month: u32, day: u32, clock: &str, year: i64) -> Option<DayAndTime> {
    let mut parts = clock.split(':');
    let hour = number(parts.next()?, 2..=2)?;
    let minute = number(parts.next()?, 2..=2)?;
    let second = number(parts.next()?, 2..=2)?;
    if parts.next().is_some() || day == 0 || day > days_in_month(year, month) {
        return None;
    }
    // A leap second, 60, is a second RFC 9110's grammar admits.
    if hour > 23 || minute > 59 || second > 60 {
        return None;
    }
    Some(DayAndTime {
        month,
        day,
        hour,
        minute,
        second,
    })
}

fn is_leap(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        2 if is_leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Days since 1970-01-01 of a date in the proleptic Gregorian calendar, by Howard Hinnant's
/// `days_from_civil`.
fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year.rem_euclid(400);
    let month_from_march = i64::from((month + 9) % 12);
    let day_of_year = (153 * month_from_march + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// The date `days` after 1970-01-01, by Howard Hinnant's `civil_from_days`.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let days = days + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_from_march = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_from_march + 2) / 5 + 1;
    let month = if month_from_march < 10 {
        month_from_march + 3
    } else {
        month_from_march - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

/// When an answer of HTTP 429 asks to be asked again.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetryAfter {
    /// A number of seconds from the answer.
    Seconds(u64),
    /// A moment.
    Date(HttpDate),
    /// No time named, or none this could read, which waits [`UNNAMED_WAIT`].
    Unnamed,
}

/// How long an answer asks the caller to wait, measured when it is read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Requested {
    For(Duration),
    /// The time it names has already passed.
    Passed,
}

impl RetryAfter {
    /// Reads a `Retry-After` header's value, where the answer carried one.
    pub fn from_header(value: Option<&str>) -> Self {
        let Some(value) = value.map(str::trim) else {
            return Self::Unnamed;
        };
        if !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()) {
            // A number too large to hold is a wait longer than any budget.
            return Self::Seconds(value.parse().unwrap_or(u64::MAX));
        }
        HttpDate::parse(value).map_or(Self::Unnamed, Self::Date)
    }

    /// The wait this asks for, measured against the wall clock's `now`.
    pub fn requested(self, now: SystemTime) -> Requested {
        match self {
            Self::Seconds(seconds) => Requested::For(Duration::from_secs(seconds)),
            Self::Date(date) => date
                .time(now)
                .duration_since(now)
                .map_or(Requested::Passed, Requested::For),
            Self::Unnamed => Requested::For(UNNAMED_WAIT),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2026-10-31T00:00:00Z, a Saturday.
    fn halloween() -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(1_793_404_800)
    }

    fn at(text: &str) -> Option<SystemTime> {
        HttpDate::parse(text).map(|date| date.time(halloween()))
    }

    #[test]
    fn an_imf_fixdate_reads_as_the_moment_it_names() {
        assert_eq!(at("Sat, 31 Oct 2026 00:00:00 GMT"), Some(halloween()));
        assert_eq!(
            at("Sun, 06 Nov 1994 08:49:37 GMT"),
            Some(UNIX_EPOCH + Duration::from_secs(784_111_777))
        );
        assert_eq!(
            at("Thu, 29 Feb 2024 23:59:59 GMT"),
            Some(UNIX_EPOCH + Duration::from_secs(1_709_251_199))
        );
        assert_eq!(
            HttpDate::parse("Mon, 1 Jan 2001 00:00:00 GMT"),
            HttpDate::parse("Mon, 01 Jan 2001 00:00:00 GMT")
        );
    }

    #[test]
    fn an_asctime_date_reads_as_the_moment_it_names() {
        assert_eq!(
            at("Sun Nov  6 08:49:37 1994"),
            Some(UNIX_EPOCH + Duration::from_secs(784_111_777))
        );
        assert_eq!(
            at("Sun Nov 06 08:49:37 1994"),
            at("Sun Nov  6 08:49:37 1994")
        );
        assert_eq!(
            at("Sat Oct 31 00:01:30 2026"),
            Some(halloween() + Duration::from_secs(90))
        );
    }

    /// RFC 9110: a two-digit year that appears more than 50 years ahead is the most recent past
    /// year with those digits.
    #[test]
    fn an_rfc_850_date_reads_with_its_century_resolved_against_now() {
        assert_eq!(
            at("Sunday, 06-Nov-94 08:49:37 GMT"),
            Some(UNIX_EPOCH + Duration::from_secs(784_111_777)),
            "2094 is 68 years ahead of 2026, so 94 is 1994"
        );
        assert_eq!(
            at("Saturday, 31-Oct-26 00:01:30 GMT"),
            Some(halloween() + Duration::from_secs(90))
        );
        assert_eq!(
            at("Wednesday, 01-Jan-76 00:00:00 GMT"),
            at("Wed, 01 Jan 2076 00:00:00 GMT"),
            "2076 is 50 years ahead, not more"
        );
        assert_eq!(
            at("Saturday, 01-Jan-77 00:00:00 GMT"),
            at("Sat, 01 Jan 1977 00:00:00 GMT"),
            "2077 is more than 50 years ahead, so 77 is 1977"
        );
        assert_eq!(
            RetryAfter::from_header(Some("Saturday, 31-Oct-26 00:01:30 GMT"))
                .requested(halloween()),
            Requested::For(Duration::from_secs(90))
        );
    }

    #[test]
    fn a_date_is_written_back_as_it_is_read() {
        for text in [
            "Sat, 31 Oct 2026 00:00:00 GMT",
            "Sun, 06 Nov 1994 08:49:37 GMT",
            "Thu, 29 Feb 2024 23:59:59 GMT",
            "Thu, 01 Jan 1970 00:00:00 GMT",
            "Wed, 31 Dec 1969 23:59:30 GMT",
        ] {
            let time = at(text).expect(text);
            assert_eq!(imf_fixdate(time), text);
        }
    }

    #[test]
    fn anything_but_an_http_date_is_not_a_date() {
        for text in [
            "",
            "soon",
            "Sun, 06 Nov 1994 08:49:37 UTC",
            "Sun, 06 Nov 1994 08:49:37 +0000",
            "Sun, 06 Nov 94 08:49:37 GMT",
            "Sun, 31 Nov 1994 08:49:37 GMT",
            "Sun, 29 Feb 2023 08:49:37 GMT",
            "Sun, 06 Nov 1994 24:00:00 GMT",
            "Sun, 06 Nov 1994 8:49:37 GMT",
            "Sun, 06 Nov 1994 08:49:37 GMT extra",
            "Dim, 06 Nov 1994 08:49:37 GMT",
            "Sun, 06-Nov-94 08:49:37 GMT",
            "Sunday, 06-Nov-1994 08:49:37 GMT",
            "Sunday, 6-Nov-94 08:49:37 GMT",
            "Sunday, 06-Nov-94 08:49:37",
            "Sun Nov  6 08:49:37 94",
            "Sun Nov  6 08:49:37 1994 GMT",
            "Sunday Nov  6 08:49:37 1994",
        ] {
            assert_eq!(HttpDate::parse(text), None, "{text:?}");
        }
    }

    #[test]
    fn a_number_of_seconds_is_waited_as_it_stands() {
        assert_eq!(
            RetryAfter::from_header(Some(" 120 ")).requested(halloween()),
            Requested::For(Duration::from_secs(120))
        );
        assert_eq!(
            RetryAfter::from_header(Some("99999999999999999999999")),
            RetryAfter::Seconds(u64::MAX)
        );
    }

    #[test]
    fn a_date_in_the_future_is_waited_until() {
        let answer = RetryAfter::from_header(Some("Sat, 31 Oct 2026 00:01:30 GMT"));
        assert_eq!(
            answer.requested(halloween()),
            Requested::For(Duration::from_secs(90))
        );
        assert_eq!(
            answer.requested(halloween() + Duration::from_millis(500)),
            Requested::For(Duration::from_millis(89_500))
        );
    }

    #[test]
    fn a_date_already_passed_asks_for_no_wait() {
        let answer = RetryAfter::from_header(Some("Fri, 30 Oct 2026 23:59:30 GMT"));
        assert_eq!(answer.requested(halloween()), Requested::Passed);
    }

    #[test]
    fn an_unreadable_or_missing_value_waits_the_unnamed_sixty_seconds() {
        for value in [None, Some(""), Some("soon"), Some("-5"), Some("1.5")] {
            let answer = RetryAfter::from_header(value);
            assert_eq!(answer, RetryAfter::Unnamed, "{value:?}");
            assert_eq!(
                answer.requested(halloween()),
                Requested::For(Duration::from_secs(60))
            );
        }
    }
}
