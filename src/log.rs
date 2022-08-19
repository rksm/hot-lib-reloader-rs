#[macro_export]
macro_rules! info { ($($arg:tt)+) => (println!("[INFO] {}", format!($($arg)+))) }
#[macro_export]
macro_rules! debug { ($($arg:tt)+) => (println!("[DEBUG] {}", format!($($arg)+))) }
#[macro_export]
macro_rules! trace { ($($arg:tt)+) => (println!("[TRACE] {}", format!($($arg)+))) }
#[macro_export]
macro_rules! warn_ { ($($arg:tt)+) => (println!("[WARN] {}", format!($($arg)+))) }
#[macro_export]
macro_rules! error { ($($arg:tt)+) => (println!("[ERROR] {}", format!($($arg)+))) }

pub use {debug, error, info, trace, warn_ as warn};
