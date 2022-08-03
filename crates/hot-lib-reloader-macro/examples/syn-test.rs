fn main() {
    if true {
        let content = r#"
impl Foo {
  fn xxx(&self) {}
}
"#;
        let ast = syn::parse_file(content).unwrap();
        dbg!(ast);
    };
}
