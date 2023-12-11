#[derive(Debug, Default, serde::Deserialize)]
pub struct Checkbox(String);

/// Extractor for a form checkbox (`<input type="checkbox" ... />`).
impl Checkbox {
    pub fn checked(&self) -> bool {
        match self.0.to_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => true,
            "0" | "false" | "no" | "off" => false,
            _ => false,
        }
    }
}
