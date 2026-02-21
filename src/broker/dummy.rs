use async_trait::async_trait;

use super::{
    BrokerConnector, ConnectorCapabilities, DeletionStatusCheck, DeletionSubmission, FoundRecord,
    PersonQuery,
};

/// A dummy broker connector for testing and demonstration.
/// Returns fake data — useful for verifying the CLI pipeline works end-to-end.
pub struct DummyBroker;

#[async_trait]
impl BrokerConnector for DummyBroker {
    fn id(&self) -> &str {
        "dummy-broker"
    }

    fn name(&self) -> &str {
        "Dummy Broker"
    }

    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            can_scan: true,
            can_delete: true,
            can_check_status: true,
        }
    }

    async fn scan(&self, query: &PersonQuery) -> anyhow::Result<Vec<FoundRecord>> {
        let full_name = format!("{} {}", query.first_name, query.last_name);

        let mut records = vec![
            FoundRecord {
                data_type: "name".into(),
                data_value: full_name,
                profile_url: Some("https://dummy-broker.example.com/profile/12345".into()),
                metadata: None,
            },
            FoundRecord {
                data_type: "address".into(),
                data_value: format!(
                    "123 Main St, {}, {}",
                    query.city.as_deref().unwrap_or("Anytown"),
                    query.state.as_deref().unwrap_or("CA")
                ),
                profile_url: Some("https://dummy-broker.example.com/profile/12345".into()),
                metadata: None,
            },
        ];

        if let Some(email) = &query.email {
            records.push(FoundRecord {
                data_type: "email".into(),
                data_value: email.clone(),
                profile_url: None,
                metadata: None,
            });
        }

        if let Some(phone) = &query.phone {
            records.push(FoundRecord {
                data_type: "phone".into(),
                data_value: phone.clone(),
                profile_url: None,
                metadata: None,
            });
        }

        Ok(records)
    }

    async fn request_deletion(
        &self,
        _query: &PersonQuery,
        _records: &[FoundRecord],
    ) -> anyhow::Result<DeletionSubmission> {
        Ok(DeletionSubmission {
            external_ref: format!("DUMMY-{}", uuid::Uuid::new_v4()),
            message: Some("Deletion request submitted to Dummy Broker".into()),
        })
    }

    async fn check_deletion_status(
        &self,
        external_ref: &str,
    ) -> anyhow::Result<DeletionStatusCheck> {
        // Always returns in_progress for the dummy
        Ok(DeletionStatusCheck {
            status: "in_progress".into(),
            completed_at: None,
            message: Some(format!("Request {external_ref} is being processed")),
        })
    }
}
