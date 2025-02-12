pub const MAX_LOG_DATA_LEN: usize = 64;
pub const VERSION_STRING: &str = "moq-transfork-03";

pub type PathId = String;
pub type GroupId = String;
pub type HexString = String;

pub fn bytes_to_hexstring(bytes: &[u8]) -> HexString {
    bytes.iter().map(|b| format!("{:02X}", b)).collect::<String>()
}

pub fn is_empty_or_none(path: &Option<PathId>) -> bool {
    match path {
        Some(p) => p.is_empty(),
        None => true,
    }
}
