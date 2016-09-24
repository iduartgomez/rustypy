#![crate_type = "cdylib"]

extern crate cpython;
extern crate syntex_syntax as syntax;
extern crate syntex_errors;
extern crate libc;

use std::collections::HashMap;
use std::ffi::CStr;
use std::path::Path;

use libc::{c_char, c_uint, size_t};

use syntex_errors::DiagnosticBuilder;
use syntax::ast;
use syntax::codemap;
use syntax::parse::{self, ParseSess};
use syntax::parse::token::InternedString;
use syntax::visit::{FnKind, Visitor};

pub mod pytypes;

// re-export
pub use self::pytypes::{PyTuple, PyString, PyBool, PyArg};

#[no_mangle]
pub extern "C" fn parse_src(mod_: *const c_char, krate_data: &mut KrateData) -> c_uint {
    let path = unsafe {
        assert!(!mod_.is_null());
        CStr::from_ptr(mod_)
    };
    // parse and walk
    let parse_session = ParseSess::new();
    let krate = match parse(path.to_str().unwrap(), &parse_session) {
        Ok(krate) => krate,
        Err(None) => return 1 as c_uint, // return value to Python to raise error from there
        Err(Some(_)) => panic!(),
    };
    // prepare data for Python
    krate_data.visit_mod(&krate.module, krate.span, 0);
    krate_data.collect_values();
    return 0 as c_uint;
}

fn parse<'a, T: ?Sized + AsRef<Path>>(path: &T,
                                      parse_session: &'a ParseSess)
                                      -> Result<ast::Crate, Option<DiagnosticBuilder<'a>>> {
    let path = path.as_ref();
    let cfgs = vec![];
    match parse::parse_crate_from_file(path, cfgs, parse_session) {
        Ok(_) if parse_session.span_diagnostic.has_errors() => Err(None),
        Ok(krate) => Ok(krate),
        Err(e) => Err(Some(e)),
    }
}

#[derive(Debug)]
struct FnDef {
    name: String,
    process: bool,
    args: Vec<String>,
}

impl FnDef {
    fn new(name: InternedString) -> FnDef {
        let n = unsafe { String::from_raw_parts(name.as_ptr() as *mut _, name.len(), name.len()) };
        FnDef {
            name: n,
            process: true,
            args: Vec::new(),
        }
    }
    fn add_type(&mut self, ty: String) {
        self.args.push(ty);
    }
}

#[derive(Debug)]
pub struct KrateData {
    functions: HashMap<codemap::Span, FnDef>,
    collected: Vec<String>,
}

impl KrateData {
    fn new() -> KrateData {
        KrateData {
            functions: HashMap::new(),
            collected: vec![],
        }
    }
    fn get_types(&mut self, span: codemap::Span, fndecl: &ast::FnDecl) {
        let work_fn = self.functions.get_mut(&span).unwrap();
        for arg in &fndecl.inputs {
            let type_str = format!("{:?}", &arg.ty);
            work_fn.add_type(type_str);
        }
        let return_type = match &fndecl.output {
            &ast::FunctionRetTy::Ty(ref s) => format!("{:?}", s),
            &ast::FunctionRetTy::Default(_) => String::from("type(void)"),
        };
        work_fn.add_type(return_type);
    }
    fn collect_values(&mut self) {
        for (_, v) in self.functions.drain() {
            let mut fndef = String::from(v.name.as_str());
            if v.args.len() > 0 {
                fndef.push_str("::");
                v.args.iter().fold(&mut fndef, |mut acc, x| {
                    acc.push_str(&x);
                    acc.push(';');
                    acc
                });
            }
            self.collected.push(fndef);
        }
    }
    fn iter_krate(&self, idx: usize) -> Option<&str> {
        if self.collected.len() >= (idx + 1) {
            Some(&self.collected[idx])
        } else {
            None
        }
    }
}

// C FFI for KrateData objects:
#[no_mangle]
pub extern "C" fn krate_data_new() -> *mut KrateData {
    Box::into_raw(Box::new(KrateData::new()))
}

#[no_mangle]
pub extern "C" fn krate_data_free(ptr: *mut KrateData) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        Box::from_raw(ptr);
    }
}

#[no_mangle]
pub extern "C" fn krate_data_len(ptr: *const KrateData) -> size_t {
    let krate = unsafe {
        assert!(!ptr.is_null());
        &*ptr
    };
    krate.collected.len()
}

#[no_mangle]
pub extern "C" fn krate_data_iter(ptr: *const KrateData, idx: size_t) -> PyString {
    let krate = unsafe {
        assert!(!ptr.is_null());
        &*ptr
    };
    match krate.iter_krate(idx as usize) {
        Some(v) => {
            PyString {
                ptr: v.as_ptr() as *const c_char,
                length: v.len()
            }
        }
        None => {
            PyString {
                ptr: "NO_IDX_ERROR".as_ptr() as *const c_char,
                length: "NO_IDX_ERROR".len()
            }
        }
    }
}

impl Visitor for KrateData {
    fn visit_fn(&mut self,
                fnkind: FnKind,
                fndecl: &ast::FnDecl,
                _: &ast::Block,
                span: codemap::Span,
                _: ast::NodeId) {
        let process;
        let vis_: Option<&ast::Visibility>;
        match fnkind {
            FnKind::Closure => {
                vis_ = None;
                process = false;
            }
            FnKind::ItemFn(_, _, _, _, _, vis) => {
                vis_ = Some(vis);
                process = true;
            }
            FnKind::Method(_, _, _) => {
                vis_ = None;
                process = false;
            }
        }
        if process && self.functions.contains_key(&span) == true {
            match vis_ {
                Some(&ast::Visibility::Public) => self.get_types(span, fndecl),
                _ => {
                    println!("warning: function `{}` must be public",
                             self.functions.get(&span).unwrap().name);
                    self.functions.remove(&span);
                }
            };
        }
    }
    fn visit_name(&mut self, span: codemap::Span, name: ast::Name) {
        if name.as_str().contains("python_bind_") {
            self.functions.insert(span, FnDef::new(name.as_str()));
        }
    }
}
