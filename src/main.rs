use serde_json::from_str;
use std::fs;
use clap::Parser;

use cf_to_mermaid::ast::ast::AST;
use cf_to_mermaid::cloudformation::template::Template;
use cf_to_mermaid::cli::parse::Args;

fn main() {
  let args = Args::parse();

  match fs::read_to_string(args.input_file) {
    Ok(contents) => {
      let cloudformation_template: Template = from_str(&contents.to_string()).unwrap();
      let ast = AST::from(cloudformation_template);
      let mermaid = ast.to_mermaid();

      if fs::metadata(&args.output_file).is_ok() {
        match fs::remove_file(&args.output_file) {
          Ok(_) => println!("Deleted existing {}", &args.output_file),
          Err(e) => println!("Error deleting file: {}", e),
        }
      }

      match fs::write(&args.output_file, mermaid) {
        Ok(_) => println!("Mermaid written to {}", &args.output_file),
        Err(e) => println!("Error writing to file: {}", e),
      }
    }
    Err(e) => {
      println!("Error reading file: {}", e);
    }
  }
}
