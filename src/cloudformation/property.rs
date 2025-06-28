use serde::Deserialize;

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(untagged)]
pub enum Property {
  Lambda {
    #[serde(rename = "FunctionName")]
    function_name: String,
    #[serde(rename = "Architectures")]
    architectures: Vec<String>,
  },
  Sqs {
    #[serde(rename = "QueueName")]
    queue_name: String,
  },
  ApiGateway {
    #[serde(rename = "HttpMethod")]
    http_method: String,
    #[serde(rename = "Integration")]
    integration: serde_json::Value,
  },
  EventSourceMapping {
    #[serde(rename = "EventSourceArn")]
    event_source_arn: serde_json::Value,
    #[serde(rename = "FunctionName")]
    function_name: serde_json::Value,
  },
  Other(serde_json::Value),
}