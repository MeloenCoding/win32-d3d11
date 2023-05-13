#[derive(Debug)]
pub struct GraphicsError {
    pub details: String,
    pub origin: super::CallLocation,
}

impl super::ErrorBase for GraphicsError {}

impl std::fmt::Display for GraphicsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for GraphicsError {
    fn description(&self) -> &str {
        &self.details
    }
}

pub struct HResultError {
    pub h_result: windows::core::HRESULT,
    pub loc: super::CallLocation
}

impl HResultError {
    pub fn new(loc: super::CallLocation, h_result: windows::core::HRESULT) -> HResultError {
        return HResultError { h_result, loc };
    }
}

impl std::fmt::Display for HResultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error code: {}\nError string: {}\nError desc: ", self.h_result.0, self.h_result.message(), )
    }
}

pub struct DeviceRemovedError {
    
}

