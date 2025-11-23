use lib::{Error, TopicConfig};
use rdkafka::{
    ClientConfig,
    admin::{AdminClient as RDAdminClient, AdminOptions, ResourceSpecifier},
    client::DefaultClientContext,
    config::FromClientConfigAndContext,
};

pub struct AdminClient {
    client: RDAdminClient<DefaultClientContext>,
    options: AdminOptions,
}

impl AdminClient {
    pub fn new(config: ClientConfig) -> Result<Self, Error> {
        let client = RDAdminClient::from_config_and_context(&config, DefaultClientContext)?;
        let options = AdminOptions::new();
        Ok(Self { client, options })
    }
    /// Loads the configuration details for the specified topic from the Kafka cluster.
    pub async fn topic_config(&self, topic: &str) -> Result<Option<TopicConfig>, Error> {
        let resource = ResourceSpecifier::Topic(topic);
        let result = self
            .client
            .describe_configs(&[resource], &self.options)
            .await?;

        if result.is_empty() {
            return Ok(None);
        }

        match result.first() {
            Some(Ok(c)) => Ok(Some(c.into())),
            Some(Err(e)) => Err(Error::Error(e.to_string())),
            None => Ok(None),
        }
    }
}
