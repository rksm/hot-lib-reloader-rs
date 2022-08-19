use std::io::ErrorKind;
use std::path::Path;
use std::process::{Command, Stdio};

#[cfg(feature = "verbose")]
use log;

static CODESIGN_BIN: &str = "codesign";

pub(crate) struct CodeSigner {
    found: bool,
}

impl CodeSigner {
    pub(crate) fn new() -> Self {
        let found = !matches!(Command::new(CODESIGN_BIN)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn(), Err(err) if err.kind() == ErrorKind::NotFound);

        if !found {
            eprintln!("[hot-lib-reloader] The MacOS `{CODESIGN_BIN}` executable cannot be found. See https://github.com/rksm/hot-lib-reloader-rs/issues/15 for more information for why this is needed. To install the XCode command line tools use brew or see https://mac.install.guide/commandlinetools/ for more options");
        }

        Self { found }
    }

    pub(crate) fn codesign(&self, f: impl AsRef<Path>) {
        if !self.found {
            log::debug!("skipping codesigning");
            return;
        }

        let f = f.as_ref().to_string_lossy().to_string();
        let result = Command::new(CODESIGN_BIN)
            // "--sign -" means to use ad-hoc identity
            // --force replaces an existing signature
            .args(["--sign", "-", "-v", "--force", &f])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .and_then(|proc| proc.wait_with_output());
        match result {
            Ok(result) => {
                log::debug!("codesigning success");
                let stdout = String::from_utf8_lossy(&result.stdout);
                let stderr = String::from_utf8_lossy(&result.stderr);
                log::trace!("[codesign stdout] {}", stdout);
                log::trace!("[codesign stderr] {}", stderr);
            }
            Err(err) => {
                eprintln!("[hot-lib-reloader] codesigning of {f} failed: {err}");
            }
        }
    }
}
