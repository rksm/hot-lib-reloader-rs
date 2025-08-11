#[rustfmt::skip]
#[no_mangle]
pub fn do_stuff() -> i32 { 3 }

#[no_mangle]
pub fn do_more_stuff(callback: Box<dyn Fn() -> i32>) -> i32 {
    let n = callback();
    n + 2
}

#[rustfmt::skip]
#[unsafe(no_mangle)]
pub fn do_even_more_stuff() -> i32 { 3 }
