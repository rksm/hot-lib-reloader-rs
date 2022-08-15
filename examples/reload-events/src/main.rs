//! This example simulates how an act can run code in preparation before a
//! library is reloaded using a
//! [https://docs.rs/hot-lib-reloader/latest/hot_lib_reloader/struct.LibReloadObserver.html](LibReloadObserver).

use hot_lib_reloader::BlockReload;
use std::{error::Error, time::Duration};
use tokio::{spawn, sync::mpsc, task::spawn_blocking, time};

#[hot_lib_reloader::hot_module(dylib = "lib")]
mod hot_lib {
    pub use lib::State;
    hot_functions_from_file!("lib/src/lib.rs");

    // expose a type to subscribe to lib load events
    #[lib_change_subscription]
    pub fn subscribe() -> hot_lib_reloader::LibReloadObserver {}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // First we setup a channel that is used to pass about-to-reload tokens to
    // the main loop. `wait_for_about_to_reload` is a synchronous blocking
    // function so in order to use it in an async context we move it into a
    // separate task.
    let (tx, mut rx) = mpsc::channel(1);

    spawn(async move {
        loop {
            let block_reload = spawn_blocking(|| hot_lib::subscribe().wait_for_about_to_reload())
                .await
                .expect("get token");
            tx.send(block_reload).await.expect("send token");
        }
    });

    let mut state = hot_lib::State { counter: 0 };
    loop {
        tokio::select! {
            // This simulates the normal main loop behavior...
            _ = time::sleep(Duration::from_secs(1)) => {
                hot_lib::do_stuff(&mut state);
            }

            // when we receive a about-to-reload token then the reload is
            // blocked while the token is still in scope. This gives us the
            // control over how long the reload should wait.
            Some(block_reload_token) = rx.recv() => {
                do_reload(block_reload_token, &mut state).await;
            }
        }
    }
}

async fn do_reload(block_reload_token: BlockReload, state: &mut lib::State) {
    // Simulate heavy work. For example serialization etc.
    println!("About to reload lib but first do some long running operation...");
    let file = std::fs::File::create("state.json").expect("save file");
    state.save(file);
    time::sleep(Duration::from_secs(1)).await;

    // Now drop the token, allow the reload
    println!("...now we are ready for reloading...");
    drop(block_reload_token); // token drop causes reload to continue

    // Now we wait for the lib to be reloaded...
    spawn_blocking(|| hot_lib::subscribe().wait_for_reload())
        .await
        .expect("wait for reload");
    println!("...now we have the new library version loaded");

    // And here we know that the library is up-to-date. We can e.g.
    // deserialize state here.
    let file = std::fs::File::open("state.json").expect("open file");
    *state = lib::State::load(file);
}
