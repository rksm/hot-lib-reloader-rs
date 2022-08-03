#[no_mangle]
pub fn test2(arg: &str) {
    println!("testing");
}

#[no_mangle]
pub fn test3(arg: i32, arg2: i32) -> i32 {
    arg + arg2
}
