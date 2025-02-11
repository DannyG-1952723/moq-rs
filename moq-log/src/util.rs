pub const MAX_LOG_DATA_LEN: usize = 64;
pub const VERSION_STRING: &str = "moq-transfork-03";

pub type HexString = String;

pub fn bytes_to_hexstring(bytes: &[u8]) -> HexString {
    bytes.iter().map(|b| format!("{:02X}", b)).collect::<String>()
}
