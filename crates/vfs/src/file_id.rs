use std::fmt::Display;

/// Handle to a file.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct FileId(pub u32);

/// safe because `FileId` is a newtype of `u32`
impl nohash_hasher::IsEnabled for FileId {}

impl From<u32> for FileId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl Display for FileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
