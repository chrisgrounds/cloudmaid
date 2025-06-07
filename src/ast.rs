use serde_json::json;

use crate::cloudformation::{Name, Property, Resource, ResourceType, Template};

// data AST = AST (Node [AST])
// a -> b
// b -> c d
// c
// d
#[derive(Debug, PartialEq, Clone)]
pub struct AST(pub Node, pub Vec<AST>);

impl AST {
  pub fn to_mermaid(&self) -> String {
    let mut result = String::from("```mermaid\nflowchart LR\n");
    self.to_mermaid_helper(&mut result, &Node { name: Name("".to_string()), typ: ResourceType::Other, properties: Property::Other(json!("")) });
    result.push_str(&String::from("```").to_string());
    result
  }

  fn to_mermaid_helper(&self, result: &mut String, parent_node: &Node) {
    let node = &self.0;

    match (node, parent_node) {
      (n, Node { name: Name(p), .. }) if p.is_empty() => result.push_str(&format!("{}\n", n)),
      (n, p) => result.push_str(&format!("{} --> {}\n", n, p)),
    }

    for child in &self.1 {
      child.to_mermaid_helper(result, node);
    }
  }
}

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
      _ => write!(f, ""),
    }
  }
}

impl Node {
  pub fn get_name(&self) -> String {
    match &self.properties {
      Property::Lambda { function_name, .. } => function_name.to_string(),
      Property::Sqs { queue_name, .. } => queue_name.to_string(),
      _ => self.name.0.clone(),
    }
  }
}

impl From<Template> for AST {
  fn from(template: Template) -> Self {
    let mut ast_nodes = Vec::new();

    for resource in &template.resources {
      if should_keep(resource.typ.clone()) {
        let node = Node::from(resource.clone());
        let references = find_references(template.clone(), resource.name.clone());

        let child_asts: Vec<AST> = references
          .into_iter()
          .map(|ref_resource| AST(Node::from(ref_resource.clone()), vec![]))
          .filter(|a| should_keep(a.0.typ.clone()))
          .collect();

        ast_nodes.push(AST(node, child_asts));
      }
    }

    AST(
      Node {
        name: Name("".to_string()),
        typ: ResourceType::Other,
        properties: Property::Other(json!("")),
      },
      ast_nodes,
    )
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
      _ => false, // TODO: Work out how to find references in lambda, sqs
    })
    .collect()
}

fn should_keep(typ: ResourceType) -> bool {
  match typ {
    ResourceType::Other => false,
    ResourceType::Lambda => true,
    ResourceType::Sqs => true,
    ResourceType::ApiGateway => true,
  }
}

impl Node {
  fn from(resource: Resource) -> Self {
    Node {
      name: resource.name,
      typ: resource.typ,
      properties: resource.properties,
    }
  }
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use super::*;

  #[test]
  fn test_ast_construction_single_node() {
    let node = Node {
      name: Name("name1".to_string()),
      typ: ResourceType::Lambda,
      properties: Property::Other(json!("")),
    };
    let ast = AST(node.clone(), vec![]);

    assert_eq!(ast, AST(node, vec![]));
  }

  #[test]
  fn test_ast_construction_with_child() {
    let parent_node = Node {
      name: Name("name1".to_string()),
      typ: ResourceType::Other,
      properties: Property::Other(json!("")),
    };
    let child_ast = AST(
      Node {
        name: Name("name1".to_string()),
        typ: ResourceType::Other,
        properties: Property::Other(json!("")),
      },
      vec![],
    );
    let parent_ast = AST(parent_node.clone(), vec![child_ast.clone()]);

    assert_eq!(parent_ast, AST(parent_node, vec![child_ast]));
  }

  #[test]
  fn test_ast_construction_multiple_children() {
    let parent_node = Node {
      name: Name("name1".to_string()),
      typ: ResourceType::Other,
      properties: Property::Other(json!("")),
    };
    let child_ast1 = AST(
      Node {
        name: Name("name1".to_string()),
        typ: ResourceType::Other,
        properties: Property::Other(json!("")),
      },
      vec![],
    );
    let child_ast2 = AST(
      Node {
        name: Name("name1".to_string()),
        typ: ResourceType::Other,
        properties: Property::Other(json!("")),
      },
      vec![],
    );
    let parent_ast = AST(
      parent_node.clone(),
      vec![child_ast1.clone(), child_ast2.clone()],
    );

    assert_eq!(parent_ast, AST(parent_node, vec![child_ast1, child_ast2]));
  }

  #[test]
  fn test_ast_construction_multiple_nested_children() {
    let parent_node = Node {
      name: Name("name1".to_string()),
      typ: ResourceType::Other,
      properties: Property::Other(json!("")),
    };
    let child_ast1 = AST(
      Node {
        name: Name("name1".to_string()),
        typ: ResourceType::Other,
        properties: Property::Other(json!("")),
      },
      vec![AST(
        Node {
          name: Name("name1".to_string()),
          typ: ResourceType::Other,
          properties: Property::Other(json!("")),
        },
        vec![],
      )],
    );
    let child_ast2 = AST(
      Node {
        name: Name("name1".to_string()),
        typ: ResourceType::Other,
        properties: Property::Other(json!("")),
      },
      vec![],
    );
    let parent_ast = AST(
      parent_node.clone(),
      vec![child_ast1.clone(), child_ast2.clone()],
    );

    assert_eq!(parent_ast, AST(parent_node, vec![child_ast1, child_ast2]));
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
          typ: ResourceType::Other,
          properties: Property::Other(json!("mylambda")),
        },
      ],
    };

    let ast = AST::from(template);

    assert_eq!(
      ast,
      AST(
        Node {
          name: Name("Root".to_string()),
          typ: ResourceType::Other,
          properties: Property::Other(json!(""))
        },
        vec![AST(
          Node {
            name: Name("mygateway".to_string()),
            typ: ResourceType::Other,
            properties: Property::Other(json!("mylambda"))
          },
          vec![AST(
            Node {
              name: Name("mylambda".to_string()),
              typ: ResourceType::Lambda,
              properties: Property::Lambda {
                function_name: "mylambda".to_string(),
                architectures: vec!["arm64".to_string()],
              }
            },
            vec![]
          )]
        )]
      )
    );
  }

  #[test]
  fn test_to_mermaid_with_lambda_node() {
    let lambda_node = Node {
      name: Name("reallylongname".to_string()),
      typ: ResourceType::Lambda,
      properties: Property::Lambda {
        function_name: "mylambda".to_string(),
        architectures: vec!["arm64".to_string()],
      },
    };
    let ast = AST(lambda_node, vec![]);

    let mermaid_output = ast.to_mermaid();
    let expected_output = "```mermaid\ngraph TD;\nmylambda\n```";

    assert_eq!(mermaid_output, expected_output);
  }
}
