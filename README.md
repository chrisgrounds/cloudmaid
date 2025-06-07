# Cloudmaid

A strongly-typed AWS Cloudformation to Mermaid parser, built in Rust.

## Usage

```
cloudmaid --input-file <INPUT_FILE> --output-file <OUTPUT_FILE>
```

## Contributing

Contributions are welcome! Please feel free to open an issue or submit a pull request.

To build simply run `cargo build` and to run, `cargo run`.

### AST

The core type is the `AST` type which is an recursive data type representing the relationships between resources in a Cloudformation template.

```rust
pub struct AST(pub Node, pub Vec<AST>);
```

### Cloudformation

The core type is `Template` type which stores the results of parsing a Cloudformation template in a Vector of `Resource`s.

```rust
pub struct Template {
  pub resources: Vec<Resource>,
}
```
