use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Template {
  #[serde(deserialize_with = "deserialize_resources")]
  pub resources: Vec<(ResourceName, ResourceContents)>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ResourceName(pub String);

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct PropertyName(pub String);

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct ResourceContents {
  #[serde(rename = "Type")]
  pub type_: String,
  pub properties: serde_json::Value,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum PropertyValue {
  PropertyString(String),
  PropertyBool(bool),
  PropertyObject(String, Box<PropertyValue>),
  PropertyArray(Vec<PropertyValue>),
  PropertyArrayObject(Vec<(PropertyName, PropertyValue)>),
}

fn deserialize_resources<'de, D>(
  deserializer: D,
) -> Result<Vec<(ResourceName, ResourceContents)>, D::Error>
where
  D: Deserializer<'de>,
{
  struct ResourcesVisitor;

  impl<'de> Visitor<'de> for ResourcesVisitor {
    type Value = Vec<(ResourceName, ResourceContents)>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("a map of resources")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
      M: MapAccess<'de>,
    {
      let mut resources = Vec::new();
      while let Some((key, value)) = access.next_entry()? {
        resources.push((ResourceName(key), value));
      }
      Ok(resources)
    }
  }

  deserializer.deserialize_map(ResourcesVisitor)
}

#[cfg(test)]
mod test {
  use serde_json::json;

  use super::*;

  #[test]
  fn test_deserialize_template() {
    let json_data = r#"
    {
        "Resources": {
            "myresource1": {
                "Type": "AWS::IAM::Role",
                "Properties": {
                    "AssumeRolePolicyDocument": {
                        "Statement": [
                            {
                                "Action": "sts:AssumeRole",
                                "Effect": "Allow",
                                "Principal": {
                                    "Service": "lambda.amazonaws.com"
                                }
                            }
                        ],
                        "Version": "2012-10-17"
                    }
                }
            }
        }
    }
    "#;

    let expected_resources = vec![(
      ResourceName("myresource1".to_string()),
      ResourceContents {
        type_: "AWS::IAM::Role".to_string(),
        properties: json!({
          "AssumeRolePolicyDocument": {
            "Statement": [
              {
                "Action": "sts:AssumeRole",
                "Effect": "Allow",
                "Principal": {
                  "Service": "lambda.amazonaws.com"
                }
              }
            ],
            "Version": "2012-10-17"
          }
        }),
      },
    )];

    let template: Template = serde_json::from_str(json_data).unwrap();

    assert_eq!(template.resources, expected_resources);
  }
}
