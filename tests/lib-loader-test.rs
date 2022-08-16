mod common;

#[hot_lib_reloader::hot_module(dylib = "lib_for_testing", file_watch_debounce = 50)]
mod hot_lib {
    hot_functions_from_file!("tests/lib_for_testing/src/lib.rs");

    #[lib_change_subscription]
    pub fn subscribe() -> hot_lib_reloader::LibReloadObserver {}

    #[lib_version]
    pub fn version() -> usize {}

    #[lib_updated]
    pub fn was_updated() -> bool {}
}

#[test]
fn test() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();

    let n = hot_lib::do_more_stuff(Box::new(hot_lib::do_stuff));
    assert_eq!(n, 5);

    assert_eq!(hot_lib::version(), 0);
    assert!(!hot_lib::was_updated());

    // simulate a file edit
    common::modify_file_and_do(
        "tests/lib_for_testing/src/lib.rs",
        |content| {
            content.replace(
                "pub fn do_stuff() -> i32 { 3 }",
                "pub fn do_stuff() -> i32 { 5 }",
            )
        },
        || {
            let lib_observer = hot_lib::subscribe();

            // simulate recompile
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(100));
                common::recompile("tests/lib_for_testing");
            });

            // wait for reload to begin (but make sure still have the old version loaded)
            let update_blocker = lib_observer.wait_for_about_to_reload();
            let n = hot_lib::do_more_stuff(Box::new(hot_lib::do_stuff));
            assert_eq!(n, 5);
            assert_eq!(hot_lib::version(), 0);
            assert!(!hot_lib::was_updated());

            // drop the blocker to allow update
            drop(update_blocker);

            // wait for reload to be completed
            lib_observer.wait_for_reload();

            // make rue lib is new
            let n = hot_lib::do_more_stuff(Box::new(hot_lib::do_stuff));
            assert_eq!(n, 7);
            assert_eq!(hot_lib::version(), 1);
            assert!(hot_lib::was_updated());
            assert!(!hot_lib::was_updated());
        },
    );
}
