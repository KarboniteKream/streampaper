#[derive(Debug)]
pub enum SourceType {
    Url,
    YouTube,
    Stream,
    Unknown,
}

impl From<i32> for SourceType {
    fn from(value: i32) -> Self {
        match value {
            1 => Self::Url,
            2 => Self::YouTube,
            3 => Self::Stream,
            _ => Self::Unknown,
        }
    }
}
