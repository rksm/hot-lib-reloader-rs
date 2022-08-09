mod common;

#[hot_lib_reloader::hot_module(dylib = "lib_for_testing")]
mod hot_lib {
    hot_functions_from_file!("./lib_for_testing/src/lib.rs");

    #[lib_change_subscription]
    pub fn subscribe() -> std::sync::mpsc::Receiver<hot_lib_reloader::ChangedEvent> {}
}

#[test]
fn test() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();

    let n = hot_lib::do_more_stuff(Box::new(hot_lib::do_stuff));
    assert_eq!(n, 5);

    common::modify_file_and_do(
        "tests/lib_for_testing/src/lib.rs",
        |content| {
            content.replace(
                "pub fn do_stuff() -> i32 { 3 }",
                "pub fn do_stuff() -> i32 { 5 }",
            )
        },
        || {
            let rx = hot_lib::subscribe();
            common::recompile("tests/lib_for_testing");
            rx.recv().expect("waiting for lib reload");

            let n = hot_lib::do_more_stuff(Box::new(hot_lib::do_stuff));
            assert_eq!(n, 7);
        },
    );
}
