use serde_json::from_str;
use std::fs;

use cf_to_mermaid::ast::ast::AST;
use cf_to_mermaid::cloudformation::template::Template;

fn main() {
  println!("Hello, world!");
  let file_path = "sample-stack.template.json";
  let output_file_path = "output.md";

  match fs::read_to_string(file_path) {
    Ok(contents) => {
      println!("File contents:\n{}", contents);

      let cloudformation_template: Template = from_str(&contents.to_string()).unwrap();
      let ast = AST::from(cloudformation_template);
      let mermaid_representation = ast.to_mermaid();

      if fs::metadata(output_file_path).is_ok() {
        match fs::remove_file(output_file_path) {
          Ok(_) => println!("Deleted existing {}", output_file_path),
          Err(e) => println!("Error deleting file: {}", e),
        }
      }

      match fs::write(output_file_path, mermaid_representation) {
        Ok(_) => println!("Mermaid representation written to {}", output_file_path),
        Err(e) => println!("Error writing to file: {}", e),
      }
    }
    Err(e) => {
      println!("Error reading file: {}", e);
    }
  }
}
