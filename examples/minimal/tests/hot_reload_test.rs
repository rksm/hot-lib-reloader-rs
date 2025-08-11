use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn parse_iteration_number(line: &str) -> Option<usize> {
    if let Some(pos) = line.rfind(" iteration ") {
        let num_str = &line[pos + " iteration ".len()..];
        num_str.parse().ok()
    } else {
        None
    }
}

const ORIGINAL_OUTPUT: &str = "doing stuff in iteration";
const MODIFIED_OUTPUT: &str = "doing more stuff in iteration";

#[test]
fn test_hot_reload_with_monotonic_counter() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .is_test(true)
        .init();

    let project_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // Build the library first
    let build_output = Command::new("cargo")
        .arg("build")
        .current_dir(project_dir.join("lib"))
        .output()
        .expect("Failed to build library");

    if !build_output.status.success() {
        panic!(
            "Failed to build library: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    // Build the main executable
    let build_output = Command::new("cargo")
        .arg("build")
        .current_dir(&project_dir)
        .output()
        .expect("Failed to build main executable");

    if !build_output.status.success() {
        panic!(
            "Failed to build main: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    // Start the main process
    let mut child = Command::new("cargo")
        .arg("run")
        .current_dir(&project_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start process");

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let reader = BufReader::new(stdout);

    let (tx, rx) = mpsc::channel();

    // Spawn thread to read output
    let reader_thread = thread::spawn(move || {
        let mut last_iteration = 0;
        let mut saw_original = false;
        let mut saw_modified = false;

        for line in reader.lines() {
            match line {
                Ok(line) => {
                    println!("[OUTPUT] {}", line);

                    if line.contains(ORIGINAL_OUTPUT) {
                        saw_original = true;
                        if let Some(num) = parse_iteration_number(&line) {
                            assert!(
                                num > last_iteration,
                                "Iteration {} not greater than {}",
                                num,
                                last_iteration
                            );
                            last_iteration = num;
                        }
                    } else if line.contains(MODIFIED_OUTPUT) {
                        saw_modified = true;
                        if let Some(num) = parse_iteration_number(&line) {
                            assert!(
                                num > last_iteration,
                                "Iteration {} not greater than {}",
                                num,
                                last_iteration
                            );
                            last_iteration = num;
                        }
                    }

                    let _ = tx.send((saw_original, saw_modified, last_iteration));
                }
                Err(e) => {
                    eprintln!("Error reading line: {}", e);
                    break;
                }
            }
        }
    });

    // Wait for initial output and collect a few iterations
    let mut saw_original = false;
    let mut iterations_before_modification = 0;

    for _ in 0..5 {
        thread::sleep(Duration::from_secs(1));

        // Drain all available messages
        while let Ok((original, _, last_iter)) = rx.try_recv() {
            if original {
                saw_original = true;
                iterations_before_modification = last_iter;
            }
        }
    }

    assert!(saw_original, "Did not see original output");
    assert!(
        iterations_before_modification >= 3,
        "Should have seen at least 3 iterations before modification, but got {}",
        iterations_before_modification
    );

    // Modify the library file
    let lib_file_path = project_dir.join("lib/src/lib.rs");
    let original_content = std::fs::read_to_string(&lib_file_path).expect("Failed to read lib.rs");

    let modified_content = original_content.replace(
        r#"println!("doing stuff in iteration {}", state.counter);"#,
        r#"println!("doing more stuff in iteration {}", state.counter);"#,
    );

    std::fs::write(&lib_file_path, &modified_content).expect("Failed to write modified lib.rs");

    // Rebuild the library
    thread::sleep(Duration::from_millis(500));
    let rebuild_output = Command::new("cargo")
        .arg("build")
        .current_dir(project_dir.join("lib"))
        .output()
        .expect("Failed to rebuild library");

    if !rebuild_output.status.success() {
        // Restore original file before panicking
        std::fs::write(&lib_file_path, &original_content).expect("Failed to restore lib.rs");
        panic!(
            "Failed to rebuild library: {}",
            String::from_utf8_lossy(&rebuild_output.stderr)
        );
    }

    // Wait for hot reload to detect change and reload
    thread::sleep(Duration::from_secs(2));

    // Check that we now see modified output with monotonic counters
    let mut saw_modified = false;
    let mut iterations_after_modification = 0;

    for _ in 0..5 {
        thread::sleep(Duration::from_secs(1));

        // Drain all available messages
        while let Ok((_, modified, last_iter)) = rx.try_recv() {
            if modified {
                saw_modified = true;
                // Verify counter is monotonically increasing from before modification
                assert!(
                    last_iter > iterations_before_modification,
                    "Counter should continue increasing after reload: {} should be > {}",
                    last_iter,
                    iterations_before_modification
                );
                iterations_after_modification = last_iter;
            }
        }
    }

    assert!(saw_modified, "Did not see modified output after reload");
    assert!(
        iterations_after_modification > iterations_before_modification,
        "Final iteration {} should be greater than initial {}",
        iterations_after_modification,
        iterations_before_modification
    );

    // Cleanup: restore original file
    std::fs::write(&lib_file_path, &original_content).expect("Failed to restore lib.rs");

    // Kill the child process
    child.kill().expect("Failed to kill child process");
    let _ = child.wait();

    // Wait for reader thread to finish
    let _ = reader_thread.join();
}
