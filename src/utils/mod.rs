pub mod cache;
pub mod colorspace;
pub mod history;
pub mod palette;
pub mod paths;
pub mod quantize;

use serde::Deserialize;

/// Newtype for palette size to prevent mixing up with array indices or counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct PaletteSize(usize);

impl PaletteSize {
    pub fn new(size: usize) -> Self {
        Self(size)
    }

    pub fn get(self) -> usize {
        self.0
    }
}
