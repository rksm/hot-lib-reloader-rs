pub fn recompile(dir: impl AsRef<std::path::Path>) {
    let cmd = std::process::Command::new("cargo")
        .arg("build")
        .current_dir(dir)
        .spawn()
        .and_then(|proc| proc.wait_with_output())
        .expect("cargo build failed");
    println!("[STDOUT] {}", String::from_utf8_lossy(&cmd.stdout));
    println!("[STDERR] {}", String::from_utf8_lossy(&cmd.stderr));
    println!("DONE");
}

pub fn modify_file_and_do(
    file: impl AsRef<std::path::Path>,
    modify_file_fn: impl FnOnce(&str) -> String,
    do_fn: impl FnOnce() + std::panic::UnwindSafe,
) {
    let file = file.as_ref().canonicalize().expect("cannot find lib file");

    let content = std::fs::read_to_string(&file).expect("cannot read file");
    let new_content = modify_file_fn(content.as_str());
    std::fs::write(&file, new_content).expect("cannot write lib file");

    let res = std::panic::catch_unwind(do_fn);

    std::fs::write(&file, content).expect("cannot restore file");

    res.expect("modify_file_and_do: do_fn panicked");
}
