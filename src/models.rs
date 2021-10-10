pub enum SourceType {
    Unknown,
    Url,
    YouTube,
}

impl From<i32> for SourceType {
    fn from(value: i32) -> Self {
        match value {
            1 => Self::Url,
            2 => Self::YouTube,
            _ => Self::Unknown,
        }
    }
}
