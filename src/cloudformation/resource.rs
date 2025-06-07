use serde::Deserialize;
use serde_json::from_value;
use crate::cloudformation::property::Property;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Resource {
  pub name: Name,
  pub typ: ResourceType,
  pub properties: Property,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub enum ResourceType {
  Lambda,
  Sqs,
  ApiGateway,
  Other,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Name(pub String);

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct ResourceContentsRaw {
  #[serde(rename = "Type")]
  pub typ: String,
  pub properties: serde_json::Value,
}

pub fn determine_resource_type(raw_type: &str) -> ResourceType {
  match raw_type {
    "AWS::Lambda::Function" => ResourceType::Lambda,
    "AWS::SQS::Queue" => ResourceType::Sqs,
    "AWS::ApiGateway::Method" => ResourceType::ApiGateway,
    _ => ResourceType::Other,
  }
}

pub fn parse_properties(
  rt: ResourceType,
  properties: serde_json::Value,
) -> Result<Property, serde_json::Error> {
  match rt {
    ResourceType::Other => Ok(Property::Other(properties)),
    _ => from_value(properties),
  }
}
