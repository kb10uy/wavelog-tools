use std::{
    fs::File,
    io::{BufWriter, Write, stdout},
    path::PathBuf,
};

use anyhow::{Context, Result, bail};
use clap::Args;
use tracing::info;

use crate::{cli::QsoQueryArgs, config::Config, wavelog::QsoQuery};

/// Exports QSOs from Wavelog as ADIF.
#[derive(Debug, Clone, Args)]
pub struct Arguments {
    /// Write ADIF to the file instead of stdout.
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    #[command(flatten)]
    pub query: QsoQueryArgs,
}

pub fn run(args: Arguments, config: &Config) -> Result<()> {
    let Some(client) = config.wavelog_client()? else {
        bail!(
            "Wavelog is not configured; add [wavelog] to {}",
            config.path().display()
        );
    };
    let query = QsoQuery::from(args.query);

    let mut writer: Box<dyn Write> = match &args.output {
        Some(path) => {
            Box::new(BufWriter::new(File::create(path).with_context(|| {
                format!("failed to create {}", path.display())
            })?))
        }
        None => Box::new(BufWriter::new(stdout().lock())),
    };

    let mut total = 0;
    for (i, page) in client.qso_adif_pages(&query).enumerate() {
        let page = page.context("failed to fetch QSOs from Wavelog")?;
        let text = if i == 0 {
            page.adif.as_str()
        } else {
            strip_header(&page.adif)
        };
        writer.write_all(text.as_bytes())?;
        total += page.exported;
        info!("fetched {} QSOs from Wavelog (page {i})", page.exported);
    }
    writer.flush()?;
    info!("exported {total} QSOs");

    Ok(())
}

/// Returns the records part of ADI text, removing the header up to `<EOH>`.
fn strip_header(adif: &str) -> &str {
    const EOH: &[u8] = b"<eoh>";
    match adif
        .as_bytes()
        .windows(EOH.len())
        .position(|w| w.eq_ignore_ascii_case(EOH))
    {
        Some(pos) => &adif[pos + EOH.len()..],
        None => adif,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_header() {
        let adif = "Wavelog ADIF export
<ADIF_VER:5>3.1.6
<EOH>

<CALL:6>JA1ZLO<EOR>
";
        assert_eq!(
            strip_header(adif),
            "

<CALL:6>JA1ZLO<EOR>
"
        );

        let adif = "header <eoh><CALL:6>JA1ZLO<eor>";
        assert_eq!(strip_header(adif), "<CALL:6>JA1ZLO<eor>");
    }

    #[test]
    fn keeps_headerless_text() {
        let adif = "<CALL:6>JA1ZLO<EOR>
";
        assert_eq!(strip_header(adif), adif);
    }
}
