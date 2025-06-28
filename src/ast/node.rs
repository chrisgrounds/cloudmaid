use crate::cloudformation::property::Property;
use crate::cloudformation::resource::{Name, Resource, ResourceType};

#[derive(Debug, PartialEq, Clone)]
pub struct Node {
  pub name: Name,
  pub typ: ResourceType,
  pub properties: Property,
}

impl std::fmt::Display for Node {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match &self.typ {
      ResourceType::Lambda => write!(f, "{}([{}])", &self.get_name(), &self.get_name()),
      ResourceType::Sqs => write!(f, "{}(({}))", &self.get_name(), &self.get_name()),
      ResourceType::ApiGateway => write!(f, "{}[[{}]]", &self.get_name(), &self.get_name()),
      ResourceType::EventSourceMapping => write!(f, "{}{{{}||}}", &self.get_name(), &self.get_name()),
      _ => write!(f, ""),
    }
  }
}

impl Node {
  pub fn from(resource: Resource) -> Self {
    Node {
      name: resource.name,
      typ: resource.typ,
      properties: resource.properties,
    }
  }

  pub fn get_name(&self) -> String {
    match &self.properties {
      Property::Lambda { function_name, .. } => function_name.to_string(),
      Property::Sqs { queue_name, .. } => queue_name.to_string(),
      _ => self.name.0.clone(),
    }
  }
}
