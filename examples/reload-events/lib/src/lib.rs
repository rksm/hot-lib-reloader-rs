use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct State {
    pub counter: usize,
}

impl State {
    pub fn load(reader: impl std::io::Read) -> Self {
        serde_json::from_reader(reader).expect("deserialize")
    }

    pub fn save(&self, writer: impl std::io::Write) {
        serde_json::to_writer(writer, self).expect("serialize state");
    }
}

#[no_mangle]
pub fn do_stuff(state: &mut State) {
    state.counter += 1;
    println!("doing stuff in iteration {}", state.counter);
}
