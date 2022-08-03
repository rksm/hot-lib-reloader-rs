fn main() {
    if true {
        let content = r#"
fn xxx(loader: Res<BevyLibLoader>) {}
"#;
        let ast = syn::parse_file(content).unwrap();
        dbg!(ast);
    };
}
