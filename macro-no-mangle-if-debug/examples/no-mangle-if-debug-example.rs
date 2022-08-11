use no_mangle_if_debug::no_mangle_if_debug;

#[no_mangle_if_debug]
fn testing() {}

fn main() {
    testing();
}
