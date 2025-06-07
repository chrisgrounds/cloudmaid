use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

use crate::cloudformation::resource::{
  Name, Resource, ResourceContentsRaw, determine_resource_type, parse_properties,
};

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Template {
  #[serde(deserialize_with = "deserialize_resources")]
  pub resources: Vec<Resource>,
}

fn deserialize_resources<'de, D>(deserializer: D) -> Result<Vec<Resource>, D::Error>
where
  D: Deserializer<'de>,
{
  struct ResourcesVisitor;

  impl<'de> Visitor<'de> for ResourcesVisitor {
    type Value = Vec<Resource>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("a map of resources")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Vec<Resource>, M::Error>
    where
      M: MapAccess<'de>,
    {
      let mut resources = Vec::new();

      while let Some((key, raw_value)) = access.next_entry::<String, ResourceContentsRaw>()? {
        let typ = determine_resource_type(&raw_value.typ);

        let properties = parse_properties(typ.clone(), raw_value.properties)
          .map_err(|_| serde::de::Error::custom("Failed to parse properties"))?;

        resources.push(Resource {
          name: Name(key),
          typ,
          properties,
        });
      }
      Ok(resources)
    }
  }

  deserializer.deserialize_map(ResourcesVisitor)
}

#[cfg(test)]
mod test {
  use serde_json::json;

  use crate::cloudformation::property::Property;
  use crate::cloudformation::resource::{Name, ResourceType};

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

    let expected_resources = vec![Resource {
      name: Name("myresource1".to_string()),
      typ: ResourceType::Other,
      properties: Property::Other(json!({
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
      })),
    }];

    let template: Template = serde_json::from_str(json_data).unwrap();

    assert_eq!(template.resources, expected_resources);
  }

  #[test]
  fn test_deserialize_lambda_properties() {
    let json_data = r#"
      {
          "Resources": {
              "myLambdaFunction": {
                  "Type": "AWS::Lambda::Function",
                  "Properties": {
                      "Architectures": ["arm64"],
                      "Code": {
                          "S3Bucket": "cdk-hnb659fds-assets-202468521054-eu-west-2",
                          "S3Key": "d22a7d3156520fc8090a1ce8d253adcbcdaa31042f36fdb3d6df5b212cca6a24.zip"
                      },
                      "Environment": {
                          "Variables": {
                              "account": "202468521054",
                              "region": "eu-west-2",
                              "EVENT_BUS_NAME": {
                                  "Fn::ImportValue": "undefined-sample-eventbus-export-name"
                              }
                          }
                      },
                      "FunctionName": "undefined-sample-core-adoption-update"
                  }
              }
          }
      }
      "#;

    let expected_resources = vec![Resource {
      name: Name("myLambdaFunction".to_string()),
      typ: ResourceType::Lambda,
      properties: Property::Lambda {
        function_name: "undefined-sample-core-adoption-update".to_string(),
        architectures: vec!["arm64".to_string()],
      },
    }];

    let template: Template = serde_json::from_str(json_data).unwrap();

    assert_eq!(template.resources, expected_resources);
  }
}
