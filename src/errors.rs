use std::{
    error::Error,
    fmt::{Display, Formatter},
};

/// Custom error type for GPU information retrieval errors.
#[derive(Debug)]
pub struct GpuInfoError(pub String);

impl Display for GpuInfoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for GpuInfoError {}
