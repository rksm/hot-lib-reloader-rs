pub struct State {
    pub counter: usize,
}

#[unsafe(no_mangle)]
pub fn do_stuff(state: &mut State) {
    state.counter += 1;
    println!("doing stuff in iteration {}", state.counter);
}
