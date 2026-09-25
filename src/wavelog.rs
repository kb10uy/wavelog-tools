mod catalog;
mod error;
mod qso;

use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, de::DeserializeOwned};
use ureq::Agent;
use url::Url;

pub use error::WavelogError;
pub use qso::{QslFilter, QsoQuery};

const API_ROOT: &str = "index.php/api/v2/";
const RESPONSE_BODY_LIMIT: u64 = 1024 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct WavelogClient {
    api_root: Url,
    token: SecretString,
    agent: Agent,
}

impl WavelogClient {
    pub fn new(base_url: &Url, token: SecretString) -> Result<WavelogClient, WavelogError> {
        let mut base_url = base_url.clone();
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        let api_root = base_url.join(API_ROOT)?;

        let agent = Agent::config_builder()
            .http_status_as_error(false)
            .build()
            .new_agent();

        Ok(WavelogClient {
            api_root,
            token,
            agent,
        })
    }

    fn get<T: DeserializeOwned>(
        &self,
        resource: &str,
        query: &[(&str, String)],
    ) -> Result<T, WavelogError> {
        let url = self.api_root.join(resource)?;
        let mut response = self
            .agent
            .get(url.as_str())
            .header(
                "Authorization",
                format!("Bearer {}", self.token.expose_secret()),
            )
            .query_pairs(query.iter().map(|(k, v)| (*k, v.as_str())))
            .call()?;

        let status = response.status();
        let body = response
            .body_mut()
            .with_config()
            .limit(RESPONSE_BODY_LIMIT)
            .read_to_string()?;

        if !status.is_success() {
            return Err(match serde_json::from_str::<ErrorEnvelope>(&body) {
                Ok(envelope) => WavelogError::Api {
                    status: status.as_u16(),
                    code: envelope.error.code,
                    message: envelope.error.message,
                },
                Err(_) => WavelogError::Status(status.as_u16()),
            });
        }

        Ok(serde_json::from_str(&body)?)
    }
}

#[derive(Debug, Deserialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}

#[derive(Debug, Deserialize)]
struct ErrorBody {
    code: String,
    message: String,
}
