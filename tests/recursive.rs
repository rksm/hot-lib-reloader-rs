mod common;

#[hot_lib_reloader::hot_module(dylib = "recursive_lib")]
mod hot_lib {
    hot_functions_from_file!("./recursive_lib/src/lib.rs");
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
            common::recompile("tests/recursive_lib");
            std::thread::sleep(std::time::Duration::from_secs(1));

            let n = hot_lib::do_more_stuff(Box::new(hot_lib::do_stuff));
            assert_eq!(n, 7);
        },
    );
}
