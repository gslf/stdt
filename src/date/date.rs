/// A lightweight date structure representing a specific moment in time.
/// 
/// This struct holds basic date and time components (year, month, day, hour, minute, second)
/// and an optional offset string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}
