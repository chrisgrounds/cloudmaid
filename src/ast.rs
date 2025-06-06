use crate::cloudformation::{ResourceContents, ResourceName, Template};

// data AST = AST (Node [AST])
// a -> b
// b -> c d
// c
// d
#[derive(Debug, PartialEq, Clone)]
pub struct AST(pub Node, pub Vec<AST>);

impl AST {
  pub fn to_mermaid(&self) -> String {
    let mut result = String::from("```mermaid\ngraph TD;\n");
    self.to_mermaid_helper(&mut result, "Root");
    result.push_str(&String::from("```").to_string());
    result
  }

  fn to_mermaid_helper(&self, result: &mut String, parent_name: &str) {
    let node_name = &self.0.name.0;
    if node_name != "Root" && parent_name != "Root" {
      result.push_str(&format!("{} --> {}\n", node_name, parent_name));
    }

    for child in &self.1 {
      child.to_mermaid_helper(result, node_name);
    }
  }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Node {
  pub name: ResourceName,
  pub type_: String,
}

impl From<Template> for AST {
  fn from(template: Template) -> Self {
    let mut ast_nodes = Vec::new();

    for (resource_name, resource_contents) in &template.resources {
      if should_include(resource_contents.type_.clone()) {
        let node = Node::from(resource_name.clone(), resource_contents.clone());
        let references = find_references(template.clone(), resource_name.clone());

        let child_asts: Vec<AST> = references
          .into_iter()
          .map(|(ref_name, ref_contents)| AST(Node::from(ref_name, ref_contents), vec![]))
          .filter(|a| should_include(a.0.type_.clone()))
          .collect();

        ast_nodes.push(AST(node, child_asts));
      }
    }

    AST(
      Node {
        name: ResourceName("Root".to_string()),
        type_: "Root".to_string(),
      },
      ast_nodes,
    )
  }
}

fn find_references(
  template: Template,
  resource_name: ResourceName,
) -> Vec<(ResourceName, ResourceContents)> {
  template
    .resources
    .into_iter()
    .filter(|(_, contents)| contents.properties.to_string().contains(&resource_name.0))
    .collect()
}

fn should_include(typ: String) -> bool {
  match typ.to_string() {
    val if val == *"AWS::Lambda::Function" => true,
    val if val == *"AWS::SQS::Queue" => true,
    val if val == *"AWS::ApiGateway::Method" => true,
    _ => false,
  }
}

impl Node {
  fn from(resource_name: ResourceName, contents: ResourceContents) -> Self {
    Node {
      name: resource_name,
      type_: contents.type_,
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
      name: ResourceName("name1".to_string()),
      type_: "Lambda".to_string(),
    };
    let ast = AST(node.clone(), vec![]);

    assert_eq!(ast, AST(node, vec![]));
  }

  #[test]
  fn test_ast_construction_with_child() {
    let parent_node = Node {
      name: ResourceName("name1".to_string()),
      type_: "Parent".to_string(),
    };
    let child_ast = AST(
      Node {
        name: ResourceName("name1".to_string()),
        type_: "Child".to_string(),
      },
      vec![],
    );
    let parent_ast = AST(parent_node.clone(), vec![child_ast.clone()]);

    assert_eq!(parent_ast, AST(parent_node, vec![child_ast]));
  }

  #[test]
  fn test_ast_construction_multiple_children() {
    let parent_node = Node {
      name: ResourceName("name1".to_string()),
      type_: "Parent".to_string(),
    };
    let child_ast1 = AST(
      Node {
        name: ResourceName("name1".to_string()),
        type_: "Child1".to_string(),
      },
      vec![],
    );
    let child_ast2 = AST(
      Node {
        name: ResourceName("name1".to_string()),
        type_: "Child2".to_string(),
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
      name: ResourceName("name1".to_string()),
      type_: "Parent".to_string(),
    };
    let child_ast1 = AST(
      Node {
        name: ResourceName("name1".to_string()),
        type_: "Child1".to_string(),
      },
      vec![AST(
        Node {
          name: ResourceName("name1".to_string()),
          type_: "NestedChild1".to_string(),
        },
        vec![],
      )],
    );
    let child_ast2 = AST(
      Node {
        name: ResourceName("name1".to_string()),
        type_: "Child2".to_string(),
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
        (
          ResourceName("mylambda".to_string()),
          ResourceContents {
            type_: "AWS::Lambda::Function".to_string(),
            properties: json!(""),
          },
        ),
        (
          ResourceName("mygateway".to_string()),
          ResourceContents {
            type_: "AWS::ApiGateway::Method".to_string(),
            properties: json!("mylambda"),
          },
        ),
      ],
    };

    let ast = AST::from(template);

    assert_eq!(ast, AST(Node { name: ResourceName("Root".to_string()), type_: "Root".to_string() }, vec![
      AST(Node { name: ResourceName("mygateway".to_string()), type_: "AWS::ApiGateway::Method".to_string() }, vec![
        AST(Node { name: ResourceName("mylambda".to_string()), type_: "AWS::Lambda::Function".to_string() }, vec![])
      ])
    ]));
  }
}
