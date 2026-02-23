use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use super::{
    BrokerConnector, ConnectorCapabilities, DeletionStatusCheck, DeletionSubmission, FoundRecord,
    PersonQuery,
};

/// BeenVerified opt-out search connector (scan-only).
///
/// Uses the public opt-out search endpoints to find records.
/// Deletion is not yet supported — BeenVerified's opt-out submission
/// requires CAPTCHA interaction that cannot be automated here.
pub struct BeenVerifiedBroker {
    client: Client,
}

impl BeenVerifiedBroker {
    pub fn new() -> anyhow::Result<Self> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
        Ok(Self { client })
    }
}

// ---------------------------------------------------------------------------
// Response deserialization (lenient — all fields optional/defaulted)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
struct BvSearchResponse {
    #[serde(default)]
    records: Vec<BvRecord>,
    /// Some API revisions use "results" instead of "records".
    #[serde(default)]
    results: Vec<BvRecord>,
}

impl BvSearchResponse {
    fn into_records(self) -> Vec<BvRecord> {
        if !self.records.is_empty() {
            self.records
        } else {
            self.results
        }
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
struct BvRecord {
    #[serde(default)]
    first_name: Option<String>,
    #[serde(default)]
    last_name: Option<String>,
    #[serde(default)]
    age: Option<u32>,
    #[serde(default)]
    city: Option<String>,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    addresses: Vec<String>,
    #[serde(default)]
    relatives: Vec<String>,
    #[serde(default)]
    profile_url: Option<String>,
}

/// Convert a single `BvRecord` into one or more `FoundRecord` entries.
fn bv_record_to_found_records(rec: &BvRecord) -> Vec<FoundRecord> {
    let mut out = Vec::new();
    let profile = rec.profile_url.clone();

    // Always emit a name record if we have at least a first or last name.
    let name = match (&rec.first_name, &rec.last_name) {
        (Some(f), Some(l)) => Some(format!("{f} {l}")),
        (Some(f), None) => Some(f.clone()),
        (None, Some(l)) => Some(l.clone()),
        (None, None) => None,
    };

    if let Some(name) = name {
        out.push(FoundRecord {
            data_type: "name".into(),
            data_value: name,
            profile_url: profile.clone(),
            metadata: None,
        });
    }

    if let Some(age) = rec.age {
        out.push(FoundRecord {
            data_type: "age".into(),
            data_value: age.to_string(),
            profile_url: profile.clone(),
            metadata: None,
        });
    }

    // Emit individual address records.
    for addr in &rec.addresses {
        out.push(FoundRecord {
            data_type: "address".into(),
            data_value: addr.clone(),
            profile_url: profile.clone(),
            metadata: None,
        });
    }

    // Fall back to city/state if no structured addresses.
    if rec.addresses.is_empty()
        && let (Some(city), Some(state)) = (&rec.city, &rec.state)
    {
        out.push(FoundRecord {
            data_type: "address".into(),
            data_value: format!("{city}, {state}"),
            profile_url: profile.clone(),
            metadata: None,
        });
    }

    if !rec.relatives.is_empty() {
        out.push(FoundRecord {
            data_type: "relatives".into(),
            data_value: rec.relatives.join(", "),
            profile_url: profile.clone(),
            metadata: None,
        });
    }

    out
}

// ---------------------------------------------------------------------------
// BrokerConnector implementation
// ---------------------------------------------------------------------------

#[async_trait]
impl BrokerConnector for BeenVerifiedBroker {
    fn id(&self) -> &str {
        "beenverified"
    }

    fn name(&self) -> &str {
        "BeenVerified"
    }

    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            can_scan: true,
            can_delete: false,
            can_check_status: false,
        }
    }

    fn home_country(&self) -> Option<&str> {
        Some("US")
    }

    fn data_countries(&self) -> &[&str] {
        &["US"]
    }

    async fn scan(&self, query: &PersonQuery) -> anyhow::Result<Vec<FoundRecord>> {
        let state = query
            .state
            .as_deref()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                anyhow::anyhow!("BeenVerified requires a state abbreviation (e.g. --state NY)")
            })?;

        // Attempt 1: JSON API endpoint
        let resp = self
            .client
            .get("https://www.beenverified.com/svc/optout/search/optouts")
            .query(&[
                ("firstName", query.first_name.as_str()),
                ("lastName", query.last_name.as_str()),
                ("state", state),
            ])
            .header("Accept", "application/json")
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                let body = r.text().await.unwrap_or_default();
                if let Ok(parsed) = serde_json::from_str::<BvSearchResponse>(&body) {
                    let records: Vec<FoundRecord> = parsed
                        .into_records()
                        .iter()
                        .flat_map(bv_record_to_found_records)
                        .collect();
                    if !records.is_empty() {
                        return Ok(records);
                    }
                }
                tracing::debug!("JSON API returned no parseable records, trying HTML fallback");
            }
            Ok(r) => {
                tracing::debug!(
                    "JSON API returned status {}, trying HTML fallback",
                    r.status()
                );
            }
            Err(e) => {
                tracing::debug!("JSON API request failed: {e}, trying HTML fallback");
            }
        }

        // Attempt 2: HTML fallback (stub — logs a warning, returns empty)
        let html_resp = self
            .client
            .get("https://www.beenverified.com/app/optout/search")
            .query(&[
                ("firstName", query.first_name.as_str()),
                ("lastName", query.last_name.as_str()),
                ("state", state),
            ])
            .send()
            .await;

        match html_resp {
            Ok(r) if r.status().is_success() => {
                tracing::warn!(
                    "HTML fallback received a response but structured HTML parsing is not yet implemented. \
                     Install the `scraper` crate and add parsing logic to extract records from the HTML page."
                );
            }
            Ok(r) => {
                tracing::debug!("HTML fallback returned status {}", r.status());
            }
            Err(e) => {
                tracing::debug!("HTML fallback request failed: {e}");
            }
        }

        Ok(vec![])
    }

    async fn request_deletion(
        &self,
        _query: &PersonQuery,
        _records: &[FoundRecord],
    ) -> anyhow::Result<DeletionSubmission> {
        anyhow::bail!(
            "BeenVerified deletion is not yet supported — the opt-out form requires CAPTCHA \
             interaction that cannot be automated without browser automation"
        )
    }

    async fn check_deletion_status(
        &self,
        _external_ref: &str,
    ) -> anyhow::Result<DeletionStatusCheck> {
        anyhow::bail!(
            "BeenVerified deletion status checking is not yet supported — \
             deletion must be implemented first"
        )
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn full_record() -> BvRecord {
        BvRecord {
            first_name: Some("Jane".into()),
            last_name: Some("Smith".into()),
            age: Some(34),
            city: Some("Brooklyn".into()),
            state: Some("NY".into()),
            addresses: vec!["123 Main St, Brooklyn, NY 11201".into()],
            relatives: vec!["John Smith".into(), "Mary Smith".into()],
            profile_url: Some("https://www.beenverified.com/people/jane-smith/".into()),
        }
    }

    #[test]
    fn test_parse_full_record() {
        let found = bv_record_to_found_records(&full_record());
        assert_eq!(found.len(), 4); // name, age, address, relatives
        assert_eq!(found[0].data_type, "name");
        assert_eq!(found[0].data_value, "Jane Smith");
        assert_eq!(found[1].data_type, "age");
        assert_eq!(found[1].data_value, "34");
        assert_eq!(found[2].data_type, "address");
        assert!(found[2].data_value.contains("Brooklyn"));
        assert_eq!(found[3].data_type, "relatives");
        assert!(found[3].data_value.contains("John Smith"));
    }

    #[test]
    fn test_parse_minimal_record() {
        let rec = BvRecord {
            first_name: Some("Bob".into()),
            ..Default::default()
        };
        let found = bv_record_to_found_records(&rec);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].data_type, "name");
        assert_eq!(found[0].data_value, "Bob");
    }

    #[test]
    fn test_search_response_records_field() {
        let json = r#"{"records": [{"first_name": "Alice", "last_name": "Jones"}]}"#;
        let resp: BvSearchResponse = serde_json::from_str(json).unwrap();
        let recs = resp.into_records();
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].first_name.as_deref(), Some("Alice"));
    }

    #[test]
    fn test_search_response_results_field() {
        let json = r#"{"results": [{"first_name": "Bob", "last_name": "Lee"}]}"#;
        let resp: BvSearchResponse = serde_json::from_str(json).unwrap();
        let recs = resp.into_records();
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].first_name.as_deref(), Some("Bob"));
    }

    #[test]
    fn test_empty_and_unknown_fields() {
        let json = r#"{"records": [{"unknown_field": "ignored", "first_name": null}]}"#;
        let resp: BvSearchResponse = serde_json::from_str(json).unwrap();
        let recs = resp.into_records();
        assert_eq!(recs.len(), 1);
        let found = bv_record_to_found_records(&recs[0]);
        assert!(found.is_empty()); // no name, no age, no addresses, no relatives
    }

    #[test]
    fn test_city_state_fallback_address() {
        let rec = BvRecord {
            first_name: Some("Test".into()),
            city: Some("Austin".into()),
            state: Some("TX".into()),
            ..Default::default()
        };
        let found = bv_record_to_found_records(&rec);
        assert_eq!(found.len(), 2); // name + address
        assert_eq!(found[1].data_type, "address");
        assert_eq!(found[1].data_value, "Austin, TX");
    }
}
