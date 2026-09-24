use std::{fs::read_to_string, path::PathBuf};

use adif_reader::{LengthMode, document::AdifDocument, read_adi, read_adx};
use anyhow::{Context, Result};
use tracing::info;

use crate::wavelog::{QsoQuery, WavelogClient};

#[derive(Debug)]
pub enum AdifSource<'a> {
    File(PathBuf),
    Wavelog(&'a WavelogClient, QsoQuery),
}

impl AdifSource<'_> {
    pub fn read_documents(&self, length_mode: LengthMode) -> Result<Vec<AdifDocument>> {
        match self {
            AdifSource::File(path) => {
                let text = read_to_string(path)
                    .with_context(|| format!("failed to read {}", path.display()))?;
                let is_adx = path
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("adx"));
                let document = if is_adx {
                    read_adx(&text)
                } else {
                    read_adi(&text, length_mode)
                }
                .with_context(|| format!("failed to parse {}", path.display()))?;
                Ok(vec![document])
            }
            AdifSource::Wavelog(client, query) => {
                let mut documents = vec![];
                for (i, page) in client.qso_adif_pages(query).enumerate() {
                    let adif = page.context("failed to fetch QSOs from Wavelog")?;
                    let document = read_adi(&adif, length_mode)
                        .with_context(|| format!("failed to parse ADIF page {i}"))?;
                    info!(
                        "fetched {} QSOs from Wavelog (page {i})",
                        document.records().len()
                    );
                    documents.push(document);
                }
                Ok(documents)
            }
        }
    }
}
