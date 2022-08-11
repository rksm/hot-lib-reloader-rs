fn main() {
    if true {
        let content = r#"
fn main(arg: Foo<'static, 'static>) {
}
"#;
        let ast = syn::parse_file(content).unwrap();
        dbg!(ast);
    };
}
