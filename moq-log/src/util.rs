pub type HexString = String;

pub fn bytes_to_hexstring(bytes: &[u8]) -> HexString {
    bytes.iter().map(|b| format!("{:02X}", b)).collect::<String>()
}
