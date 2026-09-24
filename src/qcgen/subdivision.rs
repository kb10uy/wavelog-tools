use std::collections::HashMap;

use compact_str::{CompactString, ToCompactString};
use tracing::warn;

use crate::wavelog::WavelogClient;

#[derive(Debug)]
pub struct SubdivisionResolver<'a> {
    client: &'a WavelogClient,
    cache: HashMap<u32, HashMap<String, CompactString>>,
}

impl<'a> SubdivisionResolver<'a> {
    pub fn new(client: &'a WavelogClient) -> SubdivisionResolver<'a> {
        SubdivisionResolver {
            client,
            cache: HashMap::new(),
        }
    }

    pub fn resolve(&mut self, dxcc: u32, state: &str) -> Option<CompactString> {
        let subdivisions =
            self.cache
                .entry(dxcc)
                .or_insert_with(|| match self.client.subdivisions(dxcc) {
                    Ok(list) => list
                        .into_iter()
                        .map(|s| (s.state.to_ascii_uppercase(), s.name.to_compact_string()))
                        .collect(),
                    Err(e) => {
                        warn!("failed to fetch subdivisions of DXCC {dxcc}: {e}");
                        HashMap::new()
                    }
                });
        subdivisions.get(&state.to_ascii_uppercase()).cloned()
    }
}
