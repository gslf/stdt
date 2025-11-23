use std::fmt;
use crate::date::date::Date;

/// A wrapper structure for ISO 8601 Date and Time handling.
/// 
/// This struct wraps a `Date` object and provides parsing logic for both
/// **Extended Format** (`YYYY-MM-DDTHH:MM:SS`) and **Basic Format** (`YYYYMMDDTHHMMSS`).
/// It also validates calendar semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Iso8601 {
    pub date: Date,
    pub offset_str: Option<&'static str>, 
}

/// A structure representing an ISO 8601 Duration.
///
/// ISO 8601 Durations use the format `P[n]Y[n]M[n]DT[n]H[n]M[n]S`.
/// For example: `P3Y6M4DT12H30M5S` represents a duration of 3 years,
/// 6 months, 4 days, 12 hours, 30 minutes, and 5 seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IsoDuration {
    pub years: u32,
    pub months: u32,
    pub days: u32,
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
}

impl Iso8601 {
    /// Parses an ISO 8601 string into an `Iso8601` struct.
    ///
    /// This method automatically detects and handles:
    /// * Extended format: `2023-11-23T14:30:00`
    /// * Basic format: `20231123T143000`
    ///
    /// # Arguments
    ///
    /// * `s` - A string slice representing the date time.
    ///
    /// # Errors
    ///
    /// Returns `Result::Err` if the format is unrecognizable, the string is malformed,
    /// or the date is semantically invalid (e.g., month > 12).
    ///
    /// # Examples
    ///
    /// ```
    /// use stdt::date::iso8601::Iso8601;
    /// // Extended
    /// let iso = Iso8601::parse("2023-11-23T14:30:00").unwrap();
    /// // Basic
    /// let basic = Iso8601::parse("20231123T143000").unwrap();
    /// assert_eq!(iso, basic);
    /// ```
    pub fn parse(s: &str) -> Result<Self, String> {
        if s.is_empty() { return Err("String is empty".into()); }

        // Split date and time by 'T'
        let parts: Vec<&str> = s.split('T').collect();
        if parts.len() != 2 {
            return Err("Missing 'T' separator or invalid format".into());
        }

        let date_part = parts[0];
        let time_part = parts[1].trim_end_matches('Z'); // Strip UTC 'Z' marker if present

        let (year, month, day) = Self::parse_date_part(date_part)?;
        let (hour, minute, second) = Self::parse_time_part(time_part)?;

        // Validate logical correctness
        if !Self::is_valid_calendar(year, month, day, hour, minute, second) {
            return Err("Semantically invalid date".into());
        }

        let date = Date {
            year, month, day, hour, minute, second
        };

        Ok(Iso8601 {
            date,
            offset_str: None,
        })
    }

    /// Internal helper to parse the date portion (YYYY-MM-DD or YYYYMMDD).
    fn parse_date_part(s: &str) -> Result<(i32, u8, u8), String> {
        let parse_num = |str_slice: &str| -> Result<u32, String> {
            str_slice.parse::<u32>().map_err(|_| format!("Invalid number: {}", str_slice))
        };

        if s.contains('-') {
            // Extended format: YYYY-MM-DD
            let parts: Vec<&str> = s.split('-').collect();
            if parts.len() != 3 { return Err("Invalid extended date format".into()); }
            Ok((
                parse_num(parts[0])? as i32,
                parse_num(parts[1])? as u8,
                parse_num(parts[2])? as u8
            ))
        } else {
            // Basic format: YYYYMMDD (length 8)
            if s.len() != 8 { return Err("Invalid basic date length".into()); }
            Ok((
                parse_num(&s[0..4])? as i32,
                parse_num(&s[4..6])? as u8,
                parse_num(&s[6..8])? as u8
            ))
        }
    }

    /// Internal helper to parse the time portion (HH:MM:SS or HHMMSS).
    fn parse_time_part(s: &str) -> Result<(u8, u8, u8), String> {
        let parse_num = |str_slice: &str| -> Result<u8, String> {
            str_slice.parse::<u8>().map_err(|_| format!("Invalid number: {}", str_slice))
        };

        // Check for Extended format (contains ':')
        if s.contains(':') {
            let parts: Vec<&str> = s.split(':').collect();
            if parts.len() < 2 { return Err("Invalid extended time format".into()); } // Allow HH:MM
            let h = parse_num(parts[0])?;
            let m = parse_num(parts[1])?;
            let s = if parts.len() > 2 { parse_num(parts[2])? } else { 0 };
            Ok((h, m, s))
        } else {
            // Basic format: HHMMSS (len 6) or HHMM (len 4)
            match s.len() {
                6 => Ok((parse_num(&s[0..2])?, parse_num(&s[2..4])?, parse_num(&s[4..6])?)),
                4 => Ok((parse_num(&s[0..2])?, parse_num(&s[2..4])?, 0)),
                _ => Err("Invalid basic time length".into())
            }
        }
    }

    /// Returns the ISO 8601 Extended string representation.
    pub fn to_iso8601(&self) -> String {
        format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}", 
            self.date.year, self.date.month, self.date.day, self.date.hour, self.date.minute, self.date.second)
    }

    /// Returns the ISO 8601 Basic string representation (compact).
    pub fn to_iso8601_basic(&self) -> String {
        format!("{:04}{:02}{:02}T{:02}{:02}{:02}", 
            self.date.year, self.date.month, self.date.day, self.date.hour, self.date.minute, self.date.second)
    }

    // Reuse validation logic
    fn is_valid_calendar(y: i32, m: u8, d: u8, h: u8, min: u8, s: u8) -> bool {
        if m < 1 || m > 12 || h > 23 || min > 59 || s > 60 { return false; }
        let days_in_month = match m {
            4 | 6 | 9 | 11 => 30,
            2 => if (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0) { 29 } else { 28 },
            _ => 31,
        };
        d >= 1 && d <= days_in_month
    }
}

impl IsoDuration {
    /// Parses an ISO 8601 Duration string (e.g., "P3Y6M4DT12H30M5S").
    ///
    /// # Arguments
    ///
    /// * `s` - The duration string starting with 'P'.
    ///
    /// # Errors
    ///
    /// Returns an error if the string does not start with 'P' or contains invalid segments.
    ///
    /// # Examples
    ///
    /// ```
    /// use stdt::date::iso8601::IsoDuration;
    /// let dur = IsoDuration::parse("P1Y2DT3H").unwrap();
    /// assert_eq!(dur.years, 1);
    /// assert_eq!(dur.days, 2);
    /// assert_eq!(dur.hours, 3);
    /// assert_eq!(dur.minutes, 0);
    /// ```
    pub fn parse(s: &str) -> Result<Self, String> {
        if !s.starts_with('P') {
            return Err("Duration string must start with 'P'".into());
        }

        let mut dur = IsoDuration::default();
        let mut num_buf = String::new();
        let mut is_time_part = false; // Toggles after 'T' is encountered

        // Iterate characters skipping the first 'P'
        for c in s.chars().skip(1) {
            match c {
                '0'..='9' => num_buf.push(c),
                'T' => {
                    is_time_part = true;
                    if !num_buf.is_empty() { return Err("Unexpected number before 'T'".into()); }
                },
                'Y' => {
                    if is_time_part { return Err("Years not allowed in time part".into()); }
                    dur.years = num_buf.parse().map_err(|_| "Invalid year")?;
                    num_buf.clear();
                },
                'M' => {
                    let val = num_buf.parse().map_err(|_| "Invalid number for M")?;
                    if is_time_part { dur.minutes = val; } else { dur.months = val; }
                    num_buf.clear();
                },
                'D' => {
                    if is_time_part { return Err("Days not allowed in time part".into()); }
                    dur.days = num_buf.parse().map_err(|_| "Invalid day")?;
                    num_buf.clear();
                },
                'H' => {
                    if !is_time_part { return Err("Hours must be after 'T'".into()); }
                    dur.hours = num_buf.parse().map_err(|_| "Invalid hour")?;
                    num_buf.clear();
                },
                'S' => {
                    if !is_time_part { return Err("Seconds must be after 'T'".into()); }
                    dur.seconds = num_buf.parse().map_err(|_| "Invalid second")?;
                    num_buf.clear();
                },
                _ => return Err(format!("Invalid character in duration: {}", c)),
            }
        }

        Ok(dur)
    }

    /// Formats the duration back to ISO 8601 string.
    pub fn to_string(&self) -> String {
        let mut s = String::from("P");
        if self.years > 0 { s.push_str(&format!("{}Y", self.years)); }
        if self.months > 0 { s.push_str(&format!("{}M", self.months)); }
        if self.days > 0 { s.push_str(&format!("{}D", self.days)); }

        if self.hours > 0 || self.minutes > 0 || self.seconds > 0 {
            s.push('T');
            if self.hours > 0 { s.push_str(&format!("{}H", self.hours)); }
            if self.minutes > 0 { s.push_str(&format!("{}M", self.minutes)); }
            if self.seconds > 0 { s.push_str(&format!("{}S", self.seconds)); }
        }
        
        // Edge case: empty duration P0D
        if s == "P" { return "P0D".to_string(); }
        s
    }
}

// Implement Display for easy printing
impl fmt::Display for Iso8601 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_iso8601())
    }
}

impl fmt::Display for IsoDuration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_extended_iso8601() {
        let iso = Iso8601::parse("2023-11-23T14:30:05").unwrap();
        assert_eq!(iso.date.year, 2023);
        assert_eq!(iso.date.month, 11);
        assert_eq!(iso.date.second, 5);
    }

    #[test]
    fn test_parse_basic_iso8601() {
        // Compact format: YYYYMMDDTHHMMSS
        let iso = Iso8601::parse("20231123T143005").unwrap();
        assert_eq!(iso.date.year, 2023);
        assert_eq!(iso.date.month, 11);
        assert_eq!(iso.date.day, 23);
        assert_eq!(iso.date.minute, 30);
    }

    #[test]
    fn test_iso_output_formats() {
        let iso = Iso8601::parse("2023-11-23T14:30:00").unwrap();
        assert_eq!(iso.to_iso8601(), "2023-11-23T14:30:00");
        assert_eq!(iso.to_iso8601_basic(), "20231123T143000");
    }

    #[test]
    fn test_duration_parsing_full() {
        let raw = "P3Y6M4DT12H30M5S";
        let dur = IsoDuration::parse(raw).expect("Valid duration");
        assert_eq!(dur.years, 3);
        assert_eq!(dur.months, 6);
        assert_eq!(dur.days, 4);
        assert_eq!(dur.hours, 12);
        assert_eq!(dur.minutes, 30);
        assert_eq!(dur.seconds, 5);
    }

    #[test]
    fn test_duration_parsing_partial() {
        // Just Year and Second
        let raw = "P1YT5S"; 
        let dur = IsoDuration::parse(raw).unwrap();
        assert_eq!(dur.years, 1);
        assert_eq!(dur.seconds, 5);
        assert_eq!(dur.months, 0); // Default
    }

    #[test]
    fn test_duration_ambiguous_m() {
        // Month (before T) vs Minute (after T)
        let dur = IsoDuration::parse("P1MT1M").unwrap();
        assert_eq!(dur.months, 1);
        assert_eq!(dur.minutes, 1);
    }

    #[test]
    fn test_duration_formatting() {
        let mut dur = IsoDuration::default();
        dur.years = 1;
        dur.hours = 2;
        assert_eq!(dur.to_string(), "P1YT2H");
    }
}
