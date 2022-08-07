#[derive(Default, Debug)]
pub struct State {
    call_count: usize,
}

#[no_mangle]
pub fn do_stuff(arg: &str) -> u32 {
    println!("doing stuff 123 {arg}");
    42
}

#[no_mangle]
pub fn do_stuff2<'a, 'b>(arg: &'a str, state: &'b mut State) {
    state.call_count += 1;
}
