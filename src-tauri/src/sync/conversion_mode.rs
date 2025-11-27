// In a new file: src/sync/conversion_mode.rs
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmailConversionMode {
    Markdown,
    Text,
}

impl std::str::FromStr for EmailConversionMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "markdown" => Ok(Self::Markdown),
            "text" => Ok(Self::Text),
            other => Err(format!("Unknown conversion mode: {}", other)),
        }
    }
}
