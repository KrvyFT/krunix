#![no_std]
#![no_main]

use user::{self as _, println};

#[unsafe(no_mangle)]
extern "C" fn main() -> i32 {
    println!("Hello World");
    0
}
