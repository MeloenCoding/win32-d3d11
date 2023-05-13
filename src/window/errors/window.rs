#[derive(Debug)]
pub struct WindowError {
    pub details: String,
    pub origin: super::CallLocation,
}

impl super::ErrorBase for WindowError {}

impl std::fmt::Display for WindowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for WindowError {
    fn description(&self) -> &str {
        &self.details
    }
}