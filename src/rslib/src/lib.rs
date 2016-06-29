#![crate_type = "dylib"]
#![allow(dead_code, unused_variables)]

extern crate libc;
extern crate cpython;
extern crate syntex_syntax as syntax;

use std::collections::HashMap;
use syntax::ast;
use syntax::codemap::Span;
use syntax::visit::{FnKind, Visitor};
//use cpython::Python;

struct FnInfo<'a> {
    data: HashMap<&'a str, u32>,
}

impl<'a> FnInfo<'a> {
    fn new() -> FnInfo<'a> {
        FnInfo { data: HashMap::new() }
    }
}

impl<'v, 'b> Visitor<'v> for FnInfo<'b> {
    fn visit_fn(&mut self,
                fk: FnKind<'v>,
                fd: &'v ast::FnDecl,
                b: &'v ast::Block,
                s: Span,
                _: ast::NodeId) {

    }
}

#[no_mangle]
pub extern fn parse_src() {
	println!("hello from Rust!")
}
