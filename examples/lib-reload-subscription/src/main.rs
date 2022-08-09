use hot_lib_reloader::hot_module;
use std::error::Error;

#[hot_module(dylib = "lib")]
mod hot_lib {
    use hot_lib_reloader::ChangedEvent;
    use std::sync::mpsc;

    hot_functions_from_file!("../lib/src/lib.rs");

    #[lib_change_subscription]
    pub fn lib_reload_rx() -> mpsc::Receiver<ChangedEvent> {}
}

fn main() -> Result<(), Box<dyn Error>> {
    let rx = hot_lib::lib_reload_rx();
    loop {
        hot_lib::do_stuff();
        // waits for a lib reload:
        let _event = rx.recv()?;
    }
}
