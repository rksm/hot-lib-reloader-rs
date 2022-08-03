#[derive(Default, Debug)]
pub struct State {
    pub called: usize,
}

#[no_mangle]
pub fn step(state: &mut State) {
    println!("step called");

    state.called += 1;
}
