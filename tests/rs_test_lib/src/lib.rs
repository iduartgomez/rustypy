#![allow(dead_code)]

extern crate libc;

use libc::{c_double};

/*fn python_bind_fail() {
    println!("Hello from Rust!");
}*/

pub fn python_bind_int(num: u32) -> u32 {
    let rnum = num + 1;
    rnum
}

pub fn python_bind_list1(_: Vec<(c_double, bool)>) -> Vec<String> {
    let returnval = vec![String::from("Rust")];
    returnval
}

pub fn some<'a>() -> &'a str {}
