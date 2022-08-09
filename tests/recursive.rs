mod common;

#[hot_lib_reloader::hot_module(dylib = "recursive_lib")]
mod hot_lib {
    hot_functions_from_file!("./recursive_lib/src/lib.rs");

    pub fn rx() -> std::sync::mpsc::Receiver<hot_lib_reloader::ChangedEvent> {
        __lib_loader_subscription()
    }
}

#[test]
fn test() {
    let n = hot_lib::do_more_stuff(Box::new(hot_lib::do_stuff));
    assert_eq!(n, 5);

    common::modify_file_and_do(
        "tests/recursive_lib/src/lib.rs",
        |content| {
            content.replace(
                "pub fn do_stuff() -> i32 { 3 }",
                "pub fn do_stuff() -> i32 { 5 }",
            )
        },
        || {
            let rx = hot_lib::rx();
            common::recompile("tests/recursive_lib");
            while let Ok(event) = rx.recv() {
                if matches!(event, hot_lib_reloader::ChangedEvent::LibReloaded) {
                    break;
                }
            }

            let n = hot_lib::do_more_stuff(Box::new(hot_lib::do_stuff));
            assert_eq!(n, 7);
        },
    );
}
