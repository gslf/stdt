use std::fmt;
use crate::date::date::Date;

/// A wrapper structure for POSIX (Unix Timestamp) handling.
/// 
/// This struct wraps a `Date` object (business logic).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Posix {
    pub date: Date,
}

impl Posix {
    /// Manual parser for POSIX timestamp strings (e.g., "1700749800").
    ///
    /// # Arguments
    ///
    /// * `s` - A string slice that holds the numeric timestamp.
    ///
    /// # Errors
    ///
    /// Returns a `Result::Err` if the string contains non-numeric characters
    /// or represents a timestamp that cannot be represented (e.g., negative 
    /// timestamps are not supported by this implementation for simplicity).
    ///
    /// # Examples
    ///
    /// ```
    /// use stdt::date::posix::Posix;
    /// // 1700749800 is approx Nov 23 2023
    /// let posix = Posix::parse("1700749800").unwrap();
    /// assert_eq!(posix.date.year, 2023);
    /// ```
    pub fn parse(s: &str) -> Result<Self, String> {
        let timestamp = s.parse::<i64>()
            .map_err(|_| format!("Invalid timestamp format: {}", s))?;

        if timestamp < 0 {
            return Err("Negative timestamps (pre-1970) are not supported".into());
        }

        Self::from_timestamp(timestamp)
    }

    /// Constructs a Posix object from a raw integer.
    ///
    /// # Arguments
    ///
    /// * `ts` - Seconds since Jan 1 1970.
    pub fn from_timestamp(ts: i64) -> Result<Self, String> {
        // Constants
        const SECONDS_PER_MINUTE: i64 = 60;
        const SECONDS_PER_HOUR: i64 = 3600;
        const SECONDS_PER_DAY: i64 = 86400;

        // Calculate time of day
        let mut remaining = ts;
        let days_since_epoch = remaining / SECONDS_PER_DAY;
        remaining %= SECONDS_PER_DAY;

        let hour = (remaining / SECONDS_PER_HOUR) as u8;
        remaining %= SECONDS_PER_HOUR;
        let minute = (remaining / SECONDS_PER_MINUTE) as u8;
        let second = (remaining % SECONDS_PER_MINUTE) as u8;

        // Calculate Year and Day of Year
        let mut year = 1970;
        let mut days = days_since_epoch;

        loop {
            let days_in_year = if Self::is_leap_year(year) { 366 } else { 365 };
            if days < days_in_year {
                break;
            }
            days -= days_in_year;
            year += 1;
        }

        // Calculate Month and Day of Month
        // `days` is now the 0-indexed day of the current year
        let mut month = 1;
        let days_in_months = if Self::is_leap_year(year) {
            [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        } else {
            [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        };

        for &dim in days_in_months.iter() {
            if days < dim {
                break;
            }
            days -= dim;
            month += 1;
        }

        // `days` is 0-indexed, so add 1 for the calendar day
        let day = (days + 1) as u8;

        let date = Date {
            year, month, day, hour, minute, second
        };

        Ok(Posix {
            date,
        })
    }

    /// Returns a custom "Human Readable" string representation.
    ///
    /// Format: `YYYY-MM-DD HH:MM:SS UTC`
    ///
    /// # Examples
    ///
    /// ```
    /// use stdt::date::posix::Posix;
    /// let posix = Posix::parse("0").unwrap(); // Epoch
    /// assert_eq!(posix.to_human_string(), "1970-01-01 00:00:00 UTC");
    /// ```
    pub fn to_human_string(&self) -> String {
        format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC", 
            self.date.year, self.date.month, self.date.day, 
            self.date.hour, self.date.minute, self.date.second)
    }

    /// Returns the raw timestamp as a string.
    ///
    /// # Examples
    ///
    /// ```
    /// use stdt::date::posix::Posix;
    /// let posix = Posix::parse("1700749800").unwrap();
    /// assert_eq!(posix.to_string_timestamp(), "1700749800");
    /// ```
    pub fn to_string_timestamp(&self) -> String {
        let mut total_days: i64 = 0;
        
        // Add days for past years
        for y in 1970..self.date.year {
             total_days += if Self::is_leap_year(y) { 366 } else { 365 };
        }

        // Add days for past months in current year
        let days_in_months: [i64; 12] = if Self::is_leap_year(self.date.year) {
            [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        } else {
            [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        };

        for i in 0..(self.date.month - 1) as usize {
            total_days += days_in_months[i];
        }

        // Add days in current month (1-indexed -> 0-indexed)
        total_days += (self.date.day - 1) as i64;

        // Convert to seconds
        let timestamp = total_days * 86400 
            + (self.date.hour as i64) * 3600 
            + (self.date.minute as i64) * 60 
            + (self.date.second as i64);

        timestamp.to_string()
    }

    /// Manual formatting via pattern replacement.
    ///
    /// # Supported Tokens
    ///
    /// * `YYYY` = Year (2023)
    /// * `yy`   = Short Year (23)
    /// * `mm`   = Month (01-12)
    /// * `dd`   = Day (01-31)
    /// * `HH`   = Hour (00-23)
    /// * `MM`   = Minute (00-59)
    /// * `SS`   = Second (00-59)
    /// * `TS`   = Raw Timestamp
    ///
    /// # Examples
    ///
    /// ```
    /// use stdt::date::posix::Posix;
    /// let posix = Posix::parse("1700000000").unwrap();
    /// let formatted = posix.format("At TS, date was dd/mm/yy");
    /// assert_eq!(formatted, "At 1700000000, date was 14/11/23");
    /// ```
    pub fn format(&self, pattern: &str) -> String {
        pattern
            .replace("YYYY", &format!("{:04}", self.date.year))
            .replace("yy",   &format!("{:02}", self.date.year % 100))
            .replace("mm",   &format!("{:02}", self.date.month))
            .replace("dd",   &format!("{:02}", self.date.day))
            .replace("HH",   &format!("{:02}", self.date.hour))
            .replace("MM",   &format!("{:02}", self.date.minute))
            .replace("SS",   &format!("{:02}", self.date.second))
            .replace("TS",   &self.to_string_timestamp())
    }

    // --- Internal Helpers ---

    fn is_leap_year(y: i32) -> bool {
        (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
    }
}

impl fmt::Display for Posix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string_timestamp())
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_epoch() {
        let raw = "0";
        let posix = Posix::parse(raw).expect("Should parse epoch");
        
        assert_eq!(posix.date.year, 1970);
        assert_eq!(posix.date.month, 1);
        assert_eq!(posix.date.day, 1);
        assert_eq!(posix.date.hour, 0);
        assert_eq!(posix.date.minute, 0);
        assert_eq!(posix.date.second, 0);
    }

    #[test]
    fn test_parse_specific_date() {
        // 1699963200 = 2023-11-14 12:00:00 UTC
        let raw = "1699963200";
        let posix = Posix::parse(raw).expect("Should parse specific date");

        assert_eq!(posix.date.year, 2023);
        assert_eq!(posix.date.month, 11);
        assert_eq!(posix.date.day, 14);
        assert_eq!(posix.date.hour, 12);
        assert_eq!(posix.date.minute, 0);
        assert_eq!(posix.date.second, 0);
    }

    #[test]
    fn test_parse_leap_year() {
        // 2024 is a leap year. 
        // 1709208000 = Feb 29 2024, 12:00:00
        let raw = "1709208000";
        let posix = Posix::parse(raw).expect("Should parse leap day");
        
        assert_eq!(posix.date.year, 2024);
        assert_eq!(posix.date.month, 2);
        assert_eq!(posix.date.day, 29);
    }

    #[test]
    fn test_parse_invalid_string() {
        let raw = "not_a_number";
        let res = Posix::parse(raw);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Invalid timestamp format"));
    }

    #[test]
    fn test_parse_negative_timestamp() {
        let raw = "-100";
        let res = Posix::parse(raw);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Negative timestamps"));
    }

    #[test]
    fn test_custom_format() {
        // 1234567890 = 2009-02-13 23:31:30 UTC
        let posix = Posix::from_timestamp(1234567890).unwrap();

        let pattern = "TS -> YYYY/mm/dd";
        assert_eq!(posix.format(pattern), "1234567890 -> 2009/02/13");
    }

    #[test]
    fn test_display_trait() {
        let posix = Posix::from_timestamp(1000).unwrap();
        // Should display the raw integer
        assert_eq!(format!("{}", posix), "1000");
    }
}
