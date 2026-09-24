use serde::Deserialize;

use crate::wavelog::{WavelogClient, WavelogError};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Subdivision {
    pub state: String,
    pub name: String,
}

impl WavelogClient {
    pub fn subdivisions(&self, dxcc: u32) -> Result<Vec<Subdivision>, WavelogError> {
        let pairs = [
            ("topic", "subdivisions".to_string()),
            ("dxcc", dxcc.to_string()),
        ];
        let response: CatalogResponse<Vec<Subdivision>> = self.get("catalog", &pairs)?;
        Ok(response.data)
    }
}

#[derive(Debug, Deserialize)]
struct CatalogResponse<T> {
    data: T,
}
