fn main() {
    if true {
        let content = r#"
#[cfg(debug_assertions)]
fn func() {}
"#;
        let ast = syn::parse_file(content).unwrap();
        dbg!(ast);
    };
}
