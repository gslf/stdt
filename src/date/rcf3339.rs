use std::fmt;
use crate::date::date::Date;


/// A wrapper structure for RFC3339 handling.
/// 
/// This struct wraps a `Date` object (business logic) and adds RFC3339 specific
/// context like the offset string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rfc3339 {
    pub date: Date,
    pub offset_str: Option<&'static str>, 
}

impl Rfc3339 {
    /// Manual parser for RFC3339 strings (e.g., "2023-11-23T14:30:00Z").
    ///
    /// # Arguments
    ///
    /// * `s` - A string slice that holds the date to parse.
    ///
    /// # Errors
    ///
    /// Returns a `Result::Err` if the string is too short, contains non-numeric characters
    /// in date fields, or represents an invalid calendar date (e.g., February 30th).
    ///
    /// # Examples
    ///
    /// ```
    /// use stdt::date::rcf3339::Rfc3339;
    /// let rfc = Rfc3339::parse("2023-11-23T14:30:00Z").unwrap();
    /// assert_eq!(rfc.date.year, 2023);
    /// assert_eq!(rfc.date.month, 11);
    /// ```
    pub fn parse(s: &str) -> Result<Self, String> {
        if s.len() < 19 { return Err("String too short".into()); }

        // Helper for parsing numeric slices
        let parse_part = |start, end| s[start..end].parse::<u32>()
            .map_err(|_| format!("Error parsing number between indices {} and {}", start, end));

        let year = parse_part(0, 4)? as i32;
        let month = parse_part(5, 7)? as u8;
        let day = parse_part(8, 10)? as u8;
        let hour = parse_part(11, 13)? as u8;
        let minute = parse_part(14, 16)? as u8;
        let second = parse_part(17, 19)? as u8;

        // Logical validation (Months, days, leap years)
        if !Self::is_valid_calendar(year, month, day, hour, minute, second) {
            return Err("Semantically invalid date".into());
        }

        let date = Date {
            year, month, day, hour, minute, second
        };

        Ok(Rfc3339 {
            date,
            offset_str: None, // Offset handling omitted for simplicity
        })
    }

    /// Returns a custom "Human Readable" string representation.
    ///
    /// Format: `DD/MM/YYYY - HH:MM`
    ///
    /// # Examples
    ///
    /// ```
    /// use stdt::date::rcf3339::Rfc3339;
    /// let rfc = Rfc3339::parse("2023-11-23T14:30:00Z").unwrap();
    /// assert_eq!(rfc.to_human_string(), "23/11/2023 - 14:30");
    /// ```
    pub fn to_human_string(&self) -> String {
        format!("{:02}/{:02}/{:04} - {:02}:{:02}", 
            self.date.day, self.date.month, self.date.year, self.date.hour, self.date.minute)
    }

    /// Reconstructs the RFC3339 string representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use stdt::date::rcf3339::Rfc3339;
    /// let rfc = Rfc3339::parse("2023-11-23T14:30:00Z").unwrap();
    /// assert_eq!(rfc.to_rfc3339(), "2023-11-23T14:30:00Z");
    /// ```
    pub fn to_rfc3339(&self) -> String {
        format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", 
            self.date.year, self.date.month, self.date.day, self.date.hour, self.date.minute, self.date.second)
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
    ///
    /// # Examples
    ///
    /// ```
    /// use stdt::date::rcf3339::Rfc3339;
    /// let rfc = Rfc3339::parse("2023-11-23T14:30:05Z").unwrap();
    /// let formatted = rfc.format("Today is dd/mm/yy at HH:MM");
    /// assert_eq!(formatted, "Today is 23/11/23 at 14:30");
    /// ```
    pub fn format(&self, pattern: &str) -> String {
        // Order is important: parse longer tokens first (YYYY before yy)
        pattern
            .replace("YYYY", &format!("{:04}", self.date.year))
            .replace("yy",   &format!("{:02}", self.date.year % 100))
            .replace("mm",   &format!("{:02}", self.date.month))
            .replace("dd",   &format!("{:02}", self.date.day))
            .replace("HH",   &format!("{:02}", self.date.hour))
            .replace("MM",   &format!("{:02}", self.date.minute))
            .replace("SS",   &format!("{:02}", self.date.second))
    }

    // --- Internal Validation Logic ---

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

// Implement Display for easy printing
impl fmt::Display for Rfc3339 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_rfc3339())
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_rfc3339() {
        let raw = "2023-11-23T14:30:05Z";
        let rfc = Rfc3339::parse(raw).expect("Should parse valid date");
        
        assert_eq!(rfc.date.year, 2023);
        assert_eq!(rfc.date.month, 11);
        assert_eq!(rfc.date.day, 23);
        assert_eq!(rfc.date.hour, 14);
        assert_eq!(rfc.date.minute, 30);
        assert_eq!(rfc.date.second, 5);
    }

    #[test]
    fn test_parse_invalid_string_length() {
        let raw = "2023-01-01"; // Too short
        let res = Rfc3339::parse(raw);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "String too short");
    }

    #[test]
    fn test_parse_invalid_numbers() {
        let raw = "2023-XX-01T10:00:00Z";
        let res = Rfc3339::parse(raw);
        assert!(res.is_err());
        // Verify it complains about parsing logic
        assert!(res.unwrap_err().contains("Error parsing number"));
    }

    #[test]
    fn test_calendar_validation_basic() {
        // 30th of February does not exist
        let raw = "2023-02-30T10:00:00Z";
        let res = Rfc3339::parse(raw);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "Semantically invalid date");
    }

    #[test]
    fn test_calendar_validation_time() {
        // Hour 25 is invalid
        let raw = "2023-10-10T25:00:00Z";
        let res = Rfc3339::parse(raw);
        assert!(res.is_err());
    }

    #[test]
    fn test_leap_year_logic() {
        // 2024 is a leap year, Feb 29 exists
        let valid_leap = "2024-02-29T12:00:00Z";
        assert!(Rfc3339::parse(valid_leap).is_ok());

        // 2023 is not a leap year, Feb 29 does not exist
        let invalid_leap = "2023-02-29T12:00:00Z";
        assert!(Rfc3339::parse(invalid_leap).is_err());
    }

    #[test]
    fn test_to_human_string() {
        let date_struct = Date {
            year: 2023, month: 5, day: 7,
            hour: 9, minute: 5, second: 0,
        };
        let rfc = Rfc3339 {
            date: date_struct,
            offset_str: None
        };
        // Expecting padding: 07/05/2023 - 09:05
        assert_eq!(rfc.to_human_string(), "07/05/2023 - 09:05");
    }

    #[test]
    fn test_custom_format() {
        let date_struct = Date {
            year: 2023, month: 12, day: 25,
            hour: 18, minute: 30, second: 45,
        };
        let rfc = Rfc3339 {
            date: date_struct,
            offset_str: None
        };

        let pattern = "Date: YYYY/mm/dd Time: HH:MM:SS";
        assert_eq!(rfc.format(pattern), "Date: 2023/12/25 Time: 18:30:45");

        let pattern_short = "yy-mm-dd";
        assert_eq!(rfc.format(pattern_short), "23-12-25");
    }

    #[test]
    fn test_display_trait() {
        let rfc = Rfc3339::parse("2023-11-23T14:30:00Z").unwrap();
        // format!("{}", date) uses Display trait
        assert_eq!(format!("{}", rfc), "2023-11-23T14:30:00Z");
    }
}
