use crate::ast::node::Node;
use crate::cloudformation::property::Property;
use crate::cloudformation::resource::{Name, Resource, ResourceType};
use crate::cloudformation::template::Template;

#[derive(Debug, PartialEq, Clone)]
pub struct AST {
  pub edges: Vec<(Node, Node)>,
}

impl AST {
  pub fn to_mermaid(&self) -> String {
    let mut result = String::from("```mermaid\nflowchart LR\n");
    
    for (from, to) in &self.edges {
      result.push_str(&format!("{} --> {}\n", from, to));
    }
    
    result.push_str("```");
    result
  }
}

impl From<Template> for AST {
  fn from(template: Template) -> Self {
    let mut edges = Vec::new();

    for resource in &template.resources {
      if should_keep(resource.typ.clone()) {
        match resource.typ {
          ResourceType::EventSourceMapping => {
            if let Some((source_queue, target_lambda)) = extract_event_source_mapping_refs(resource, &template) {
              edges.push((source_queue, target_lambda));
            }
          },
          _ => {
            let referenced_node = Node::from(resource.clone());
            let references = find_references(template.clone(), resource.name.clone());

            for ref_resource in references {
              if should_keep(ref_resource.typ.clone()) {
                let referencing_node = Node::from(ref_resource);
                edges.push((referencing_node, referenced_node.clone()));
              }
            }
          }
        }
      }
    }

    AST { edges }
  }
}

fn find_references(template: Template, resource_name: Name) -> Vec<Resource> {
  template
    .resources
    .into_iter()
    .filter(|resource| match &resource.properties {
      Property::Other(properties) => properties.to_string().contains(&resource_name.0),
      Property::ApiGateway { integration, .. } => {
        integration.to_string().contains(&resource_name.0)
      }
      _ => false,
    })
    .collect()
}

fn extract_event_source_mapping_refs(resource: &Resource, template: &Template) -> Option<(Node, Node)> {
  if let Property::EventSourceMapping { event_source_arn, function_name } = &resource.properties {
    let queue_name = extract_ref_from_getatt(event_source_arn)?;
    
    let lambda_name = extract_ref_from_ref(function_name)?;
    
    let queue_resource = template.resources.iter().find(|r| r.name.0 == queue_name)?;
    let lambda_resource = template.resources.iter().find(|r| r.name.0 == lambda_name)?;
    
    Some((Node::from(queue_resource.clone()), Node::from(lambda_resource.clone())))
  } else {
    None
  }
}

fn extract_ref_from_getatt(value: &serde_json::Value) -> Option<String> {
  if let Some(get_att) = value.get("Fn::GetAtt") {
    if let Some(array) = get_att.as_array() {
      if let Some(resource_name) = array.get(0) {
        return resource_name.as_str().map(|s| s.to_string());
      }
    }
  }
  None
}

fn extract_ref_from_ref(value: &serde_json::Value) -> Option<String> {
  if let Some(ref_value) = value.get("Ref") {
    return ref_value.as_str().map(|s| s.to_string());
  }
  None
}

fn should_keep(typ: ResourceType) -> bool {
  match typ {
    ResourceType::Other => false,
    ResourceType::Lambda => true,
    ResourceType::Sqs => true,
    ResourceType::ApiGateway => true,
    ResourceType::EventSourceMapping => true,
  }
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use super::*;

  #[test]
  fn test_ast_construction_single_edge() {
    let node1 = Node {
      name: Name("name1".to_string()),
      typ: ResourceType::Sqs,
      properties: Property::Sqs {
        queue_name: "queue1".to_string(),
      },
    };
    let node2 = Node {
      name: Name("name2".to_string()),
      typ: ResourceType::Lambda,
      properties: Property::Lambda {
        function_name: "lambda1".to_string(),
        architectures: vec!["arm64".to_string()],
      },
    };
    let ast = AST { edges: vec![(node1.clone(), node2.clone())] };

    assert_eq!(ast, AST { edges: vec![(node1, node2)] });
  }

  #[test]
  fn test_ast_construction_multiple_edges() {
    let sqs_node = Node {
      name: Name("queue1".to_string()),
      typ: ResourceType::Sqs,
      properties: Property::Sqs {
        queue_name: "queue1".to_string(),
      },
    };
    let lambda_node1 = Node {
      name: Name("lambda1".to_string()),
      typ: ResourceType::Lambda,
      properties: Property::Lambda {
        function_name: "lambda1".to_string(),
        architectures: vec!["arm64".to_string()],
      },
    };
    let lambda_node2 = Node {
      name: Name("lambda2".to_string()),
      typ: ResourceType::Lambda,
      properties: Property::Lambda {
        function_name: "lambda2".to_string(),
        architectures: vec!["arm64".to_string()],
      },
    };
    
    let ast = AST { 
      edges: vec![
        (sqs_node.clone(), lambda_node1.clone()),
        (sqs_node.clone(), lambda_node2.clone())
      ] 
    };

    assert_eq!(ast, AST { 
      edges: vec![
        (sqs_node.clone(), lambda_node1),
        (sqs_node, lambda_node2)
      ] 
    });
  }

  #[test]
  fn test_ast_construction_chain_edges() {
    let api_node = Node {
      name: Name("api".to_string()),
      typ: ResourceType::ApiGateway,
      properties: Property::ApiGateway {
        http_method: "POST".to_string(),
        integration: serde_json::json!({}),
      },
    };
    let lambda_node = Node {
      name: Name("lambda".to_string()),
      typ: ResourceType::Lambda,
      properties: Property::Lambda {
        function_name: "lambda".to_string(),
        architectures: vec!["arm64".to_string()],
      },
    };
    let sqs_node = Node {
      name: Name("queue".to_string()),
      typ: ResourceType::Sqs,
      properties: Property::Sqs {
        queue_name: "queue".to_string(),
      },
    };
    
    let ast = AST { 
      edges: vec![
        (api_node.clone(), lambda_node.clone()),
        (lambda_node.clone(), sqs_node.clone())
      ] 
    };

    assert_eq!(ast, AST { 
      edges: vec![
        (api_node, lambda_node.clone()),
        (lambda_node, sqs_node)
      ] 
    });
  }


  #[test]
  fn test_ast_from_template() {
    let template = Template {
      resources: vec![
        Resource {
          name: Name("mylambda".to_string()),
          typ: ResourceType::Lambda,
          properties: Property::Lambda {
            function_name: "mylambda".to_string(),
            architectures: vec!["arm64".to_string()],
          },
        },
        Resource {
          name: Name("mygateway".to_string()),
          typ: ResourceType::ApiGateway,
          properties: Property::ApiGateway {
            http_method: "POST".to_string(),
            integration: json!("mylambda"),
          },
        },
      ],
    };

    let ast = AST::from(template);

    let expected_gateway_node = Node {
      name: Name("mygateway".to_string()),
      typ: ResourceType::ApiGateway,
      properties: Property::ApiGateway {
        http_method: "POST".to_string(),
        integration: json!("mylambda"),
      },
    };
    let expected_lambda_node = Node {
      name: Name("mylambda".to_string()),
      typ: ResourceType::Lambda,
      properties: Property::Lambda {
        function_name: "mylambda".to_string(),
        architectures: vec!["arm64".to_string()],
      },
    };

    assert_eq!(
      ast,
      AST {
        edges: vec![(expected_gateway_node, expected_lambda_node)]
      }
    );
  }

  #[test]
  fn test_to_mermaid_with_single_edge() {
    let sqs_node = Node {
      name: Name("myqueue".to_string()),
      typ: ResourceType::Sqs,
      properties: Property::Sqs {
        queue_name: "myqueue".to_string(),
      },
    };
    let lambda_node = Node {
      name: Name("mylambda".to_string()),
      typ: ResourceType::Lambda,
      properties: Property::Lambda {
        function_name: "mylambda".to_string(),
        architectures: vec!["arm64".to_string()],
      },
    };
    
    let ast = AST {
      edges: vec![(sqs_node, lambda_node)]
    };

    let mermaid_output = ast.to_mermaid();
    let expected_output = "```mermaid\nflowchart LR\nmyqueue((myqueue)) --> mylambda([mylambda])\n```";

    assert_eq!(mermaid_output, expected_output);
  }

  #[test]
  fn test_empty_template() {
    let template = Template {
      resources: vec![],
    };

    let ast = AST::from(template);

    assert_eq!(ast, AST { edges: vec![] });

    let mermaid_output = ast.to_mermaid();
    let expected_output = "```mermaid\nflowchart LR\n```";

    assert_eq!(mermaid_output, expected_output);
  }

  #[test]
  fn test_fan_in_pattern() {
    let template = Template {
      resources: vec![
        Resource {
          name: Name("mylambda".to_string()),
          typ: ResourceType::Lambda,
          properties: Property::Lambda {
            function_name: "mylambda".to_string(),
            architectures: vec!["arm64".to_string()],
          },
        },
        Resource {
          name: Name("myapi".to_string()),
          typ: ResourceType::ApiGateway,
          properties: Property::ApiGateway {
            http_method: "POST".to_string(),
            integration: json!("mylambda"),
          },
        },
        Resource {
          name: Name("myqueue".to_string()),
          typ: ResourceType::Sqs,
          properties: Property::Sqs {
            queue_name: "myqueue".to_string(),
          },
        },
      ],
    };

    let ast = AST::from(template);

    let expected_lambda_node = Node {
      name: Name("mylambda".to_string()),
      typ: ResourceType::Lambda,
      properties: Property::Lambda {
        function_name: "mylambda".to_string(),
        architectures: vec!["arm64".to_string()],
      },
    };
    let expected_api_node = Node {
      name: Name("myapi".to_string()),
      typ: ResourceType::ApiGateway,
      properties: Property::ApiGateway {
        http_method: "POST".to_string(),
        integration: json!("mylambda"),
      },
    };

    assert_eq!(
      ast,
      AST {
        edges: vec![(expected_api_node, expected_lambda_node)]
      }
    );

    let mermaid_output = ast.to_mermaid();
    let expected_output = "```mermaid\nflowchart LR\nmyapi[[myapi]] --> mylambda([mylambda])\n```";

    assert_eq!(mermaid_output, expected_output);
  }

  #[test]
  fn test_mixed_resource_types_filtering() {
    let template = Template {
      resources: vec![
        Resource {
          name: Name("mylambda".to_string()),
          typ: ResourceType::Lambda,
          properties: Property::Lambda {
            function_name: "mylambda".to_string(),
            architectures: vec!["arm64".to_string()],
          },
        },
        Resource {
          name: Name("myapi".to_string()),
          typ: ResourceType::ApiGateway,
          properties: Property::ApiGateway {
            http_method: "POST".to_string(),
            integration: json!("mylambda"),
          },
        },
        Resource {
          name: Name("unsupported".to_string()),
          typ: ResourceType::Other,
          properties: Property::Other(json!("some value")),
        },
      ],
    };

    let ast = AST::from(template);

    let expected_lambda_node = Node {
      name: Name("mylambda".to_string()),
      typ: ResourceType::Lambda,
      properties: Property::Lambda {
        function_name: "mylambda".to_string(),
        architectures: vec!["arm64".to_string()],
      },
    };
    let expected_api_node = Node {
      name: Name("myapi".to_string()),
      typ: ResourceType::ApiGateway,
      properties: Property::ApiGateway {
        http_method: "POST".to_string(),
        integration: json!("mylambda"),
      },
    };

    assert_eq!(
      ast,
      AST {
        edges: vec![(expected_api_node, expected_lambda_node)]
      }
    );

    let mermaid_output = ast.to_mermaid();
    let expected_output = "```mermaid\nflowchart LR\nmyapi[[myapi]] --> mylambda([mylambda])\n```";

    assert_eq!(mermaid_output, expected_output);
  }

  #[test]
  fn test_no_dependencies() {
    let template = Template {
      resources: vec![
        Resource {
          name: Name("lambda1".to_string()),
          typ: ResourceType::Lambda,
          properties: Property::Lambda {
            function_name: "lambda1".to_string(),
            architectures: vec!["arm64".to_string()],
          },
        },
        Resource {
          name: Name("lambda2".to_string()),
          typ: ResourceType::Lambda,
          properties: Property::Lambda {
            function_name: "lambda2".to_string(),
            architectures: vec!["arm64".to_string()],
          },
        },
        Resource {
          name: Name("queue1".to_string()),
          typ: ResourceType::Sqs,
          properties: Property::Sqs {
            queue_name: "queue1".to_string(),
          },
        },
      ],
    };

    let ast = AST::from(template);

    assert_eq!(ast, AST { edges: vec![] });

    let mermaid_output = ast.to_mermaid();
    let expected_output = "```mermaid\nflowchart LR\n```";

    assert_eq!(mermaid_output, expected_output);
  }

  #[test]
  fn test_event_source_mapping() {
    let template = Template {
      resources: vec![
        Resource {
          name: Name("MyQueue".to_string()),
          typ: ResourceType::Sqs,
          properties: Property::Sqs {
            queue_name: "MyQueue".to_string(),
          },
        },
        Resource {
          name: Name("MyLambda".to_string()),
          typ: ResourceType::Lambda,
          properties: Property::Lambda {
            function_name: "MyLambda".to_string(),
            architectures: vec!["arm64".to_string()],
          },
        },
        Resource {
          name: Name("MyEventSourceMapping".to_string()),
          typ: ResourceType::EventSourceMapping,
          properties: Property::EventSourceMapping {
            event_source_arn: json!({
              "Fn::GetAtt": ["MyQueue", "Arn"]
            }),
            function_name: json!({
              "Ref": "MyLambda"
            }),
          },
        },
      ],
    };

    let ast = AST::from(template);

    let expected_queue_node = Node {
      name: Name("MyQueue".to_string()),
      typ: ResourceType::Sqs,
      properties: Property::Sqs {
        queue_name: "MyQueue".to_string(),
      },
    };
    let expected_lambda_node = Node {
      name: Name("MyLambda".to_string()),
      typ: ResourceType::Lambda,
      properties: Property::Lambda {
        function_name: "MyLambda".to_string(),
        architectures: vec!["arm64".to_string()],
      },
    };

    // Should create SQS -> Lambda edge from EventSourceMapping
    assert_eq!(
      ast,
      AST {
        edges: vec![(expected_queue_node, expected_lambda_node)]
      }
    );

    let mermaid_output = ast.to_mermaid();
    let expected_output = "```mermaid\nflowchart LR\nMyQueue((MyQueue)) --> MyLambda([MyLambda])\n```";

    assert_eq!(mermaid_output, expected_output);
  }

  #[test]
  fn test_to_mermaid_with_multiple_edges() {
    let api_node = Node {
      name: Name("myapi".to_string()),
      typ: ResourceType::ApiGateway,
      properties: Property::ApiGateway {
        http_method: "POST".to_string(),
        integration: serde_json::json!({}),
      },
    };
    let lambda_node = Node {
      name: Name("mylambda".to_string()),
      typ: ResourceType::Lambda,
      properties: Property::Lambda {
        function_name: "mylambda".to_string(),
        architectures: vec!["arm64".to_string()],
      },
    };
    let sqs_node = Node {
      name: Name("myqueue".to_string()),
      typ: ResourceType::Sqs,
      properties: Property::Sqs {
        queue_name: "myqueue".to_string(),
      },
    };
    
    let ast = AST {
      edges: vec![
        (api_node, lambda_node.clone()),
        (lambda_node, sqs_node)
      ]
    };

    let mermaid_output = ast.to_mermaid();
    let expected_output = "```mermaid\nflowchart LR\nmyapi[[myapi]] --> mylambda([mylambda])\nmylambda([mylambda]) --> myqueue((myqueue))\n```";

    assert_eq!(mermaid_output, expected_output);
  }
}
