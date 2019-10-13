//! Binding Rust with Python, both ways!
//!
//! This library will generate and handle type conversions between Python and
//! Rust. To use Python from Rust refer to the
//! [library wiki](https://github.com/iduartgomez/rustypy/wiki), more general examples
//! and information on how to use Rust in Python can also be found there.
//!
//! Checkout the [PyTypes](../rustypy/pytypes/index.html) module documentation for more information
//! on how to write foreign functions that are compliant with Python as well as using the custom
//! types that will ease type conversion.
#![crate_type = "cdylib"]

extern crate cpython;
extern crate libc;
extern crate syn;
extern crate walkdir;

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::ptr;

use libc::size_t;

mod macros;
pub mod pytypes;

// re-export
pub use self::pytypes::pybool::PyBool;
pub use self::pytypes::pydict::PyDict;
pub use self::pytypes::pylist::PyList;
pub use self::pytypes::pystring::PyString;
pub use self::pytypes::pytuple::PyTuple;
pub use self::pytypes::PyArg;

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn parse_src(
    path: *mut PyString,
    krate_data: &mut KrateData,
) -> *mut PyString {
    let path = PyString::from_ptr_to_string(path);
    let path: &Path = path.as_ref();
    let dir = if let Some(parent) = path.parent() {
        parent
    } else {
        // unlikely this happens, but just in case
        return PyString::from("crate in root directory not allowed".to_string()).into_raw();
    };
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            if let Some(ext) = e.path().extension() {
                ext == "rs"
            } else {
                false
            }
        })
    {
        if let Err(err) = parse_file(krate_data, entry.path()) {
            return err;
        }
    }
    ptr::null_mut::<PyString>()
}

fn parse_file(krate_data: &mut KrateData, path: &Path) -> Result<(), *mut PyString> {
    let mut f = match File::open(path) {
        Ok(file) => file,
        Err(_) => {
            return Err(
                PyString::from(format!("path not found: {}", path.to_str().unwrap())).into_raw(),
            )
        }
    };
    let mut src = String::new();
    if f.read_to_string(&mut src).is_err() {
        return Err(PyString::from(format!(
            "failed to read the source file: {}",
            path.to_str().unwrap()
        ))
        .into_raw());
    }
    match syn::parse_file(&src) {
        Ok(krate) => {
            syn::visit::visit_file(krate_data, &krate);
            krate_data.collect_values();
        }
        Err(err) => return Err(PyString::from(format!("{}", err)).into_raw()),
    };
    Ok(())
}

#[doc(hidden)]
pub struct KrateData {
    functions: Vec<FnDef>,
    collected: Vec<String>,
    prefixes: Vec<String>,
}

impl KrateData {
    fn new(prefixes: Vec<String>) -> KrateData {
        KrateData {
            functions: vec![],
            collected: vec![],
            prefixes,
        }
    }

    fn collect_values(&mut self) {
        let mut add = true;
        for v in self.functions.drain(..) {
            let FnDef {
                name: mut fndef,
                args,
                output,
            } = v;
            if !args.is_empty() {
                fndef.push_str("::");
                args.iter().fold(&mut fndef, |acc, arg| {
                    if let Ok(repr) = type_repr(arg, None) {
                        acc.push_str(&repr);
                        acc.push(';');
                    } else {
                        add = false;
                    }
                    acc
                });
            }
            if add {
                match output {
                    syn::ReturnType::Default => fndef.push_str("type(void)"),
                    syn::ReturnType::Type(_, ty) => {
                        if let Ok(ty) = type_repr(&ty, None) {
                            fndef.push_str(&ty)
                        } else {
                            continue;
                        }
                    }
                }
                self.collected.push(fndef);
            } else {
                add = true
            }
        }
    }

    fn add_fn(&mut self, name: String, fn_decl: &syn::ItemFn) {
        for prefix in &self.prefixes {
            if name.starts_with(prefix) {
                let syn::ItemFn { sig, .. } = fn_decl.clone();
                let mut args = vec![];
                for arg in sig.inputs {
                    match arg {
                        syn::FnArg::Typed(pat_ty) => args.push(*pat_ty.ty),
                        _ => continue,
                    }
                }
                self.functions.push(FnDef {
                    name,
                    args,
                    output: sig.output,
                });
                break;
            }
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

fn type_repr(ty: &syn::Type, r: Option<&str>) -> Result<String, ()> {
    let mut repr = String::new();
    match ty {
        syn::Type::Path(path) => {
            let syn::TypePath { path, .. } = path;
            if let Some(ty) = path.segments.last() {
                if let Some(r) = r {
                    Ok(format!("type({} {})", r, ty.ident))
                } else {
                    Ok(format!("type({})", ty.ident))
                }
            } else {
                Err(())
            }
        }
        syn::Type::Ptr(ty) => {
            let syn::TypePtr {
                elem, mutability, ..
            } = ty;
            let m = match mutability {
                Some(_) => "*mut",
                _ => "*const",
            };
            repr.push_str(&type_repr(&*elem, Some(m))?);
            Ok(repr)
        }
        syn::Type::Reference(ty) => {
            let syn::TypeReference {
                elem, mutability, ..
            } = ty;
            let m = match mutability {
                Some(_) => "&mut",
                _ => "&",
            };
            repr.push_str(&type_repr(&*elem, Some(m))?);
            Ok(repr)
        }
        _ => Err(()),
    }
}

impl<'ast> syn::visit::Visit<'ast> for KrateData {
    fn visit_item(&mut self, item: &syn::Item) {
        match item {
            syn::Item::Fn(fn_decl, ..) => {
                if let syn::Visibility::Public(_) = fn_decl.vis {
                    let name = format!("{}", fn_decl.sig.ident);
                    self.add_fn(name, &*fn_decl)
                }
            }
            syn::Item::Mod(mod_item) if mod_item.content.is_some() => {
                for item in &mod_item.content.as_ref().unwrap().1 {
                    self.visit_item(item);
                }
            }
            _ => {}
        }
    }
}

struct FnDef {
    name: String,
    output: syn::ReturnType,
    args: Vec<syn::Type>,
}

// C FFI for KrateData objects:
#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn krate_data_new(ptr: *mut PyList) -> *mut KrateData {
    let p = PyList::from_ptr(ptr);
    let p: Vec<String> = PyList::into(p);
    Box::into_raw(Box::new(KrateData::new(p)))
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn krate_data_free(ptr: *mut KrateData) {
    if ptr.is_null() {
        return;
    }
    Box::from_raw(ptr);
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn krate_data_len(krate: &KrateData) -> size_t {
    krate.collected.len()
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn krate_data_iter(krate: &KrateData, idx: size_t) -> *mut PyString {
    match krate.iter_krate(idx as usize) {
        Some(val) => PyString::from(val).into_raw(),
        None => PyString::from("NO_IDX_ERROR").into_raw(),
    }
}
