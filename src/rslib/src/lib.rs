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
extern crate syn;
extern crate libc;
extern crate walkdir;

use std::io::Read;
use std::fs::File;
use std::path::Path;
use std::ptr;

use libc::size_t;

pub mod pytypes;

// re-export
pub use self::pytypes::pybool::PyBool;
pub use self::pytypes::pystring::PyString;
pub use self::pytypes::pylist::PyList;
pub use self::pytypes::pydict::PyDict;
pub use self::pytypes::pytuple::PyTuple;
pub use self::pytypes::PyArg;

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn parse_src(path: *mut PyString, krate_data: &mut KrateData) -> *mut PyString {
    use std::convert::AsRef;
    let path = unsafe { PyString::from_ptr_to_string(path) };
    let path: &Path = path.as_ref();
    let dir = if let Some(parent) = path.parent() {
        parent
    } else {
        // unlikely this happens, but just in case
        return PyString::from(format!("crate in root directory not allowed")).as_ptr();
    };
    for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| if let Some(ext) = e.path().extension() {
                        ext == "rs"
                    } else {
                        false
                    }) {
        if let Err(err) = parse_file(krate_data, entry.path()) {
            return err;
        }
    }
    return ptr::null_mut::<PyString>();
}

fn parse_file(krate_data: &mut KrateData, path: &Path) -> Result<(), *mut PyString> {
    let mut f = match File::open(path) {
        Ok(file) => file,
        Err(_) => {
            return Err(PyString::from(format!("path not found: {}", path.to_str().unwrap()))
                           .as_ptr())
        }
    };
    let mut src = String::new();
    if f.read_to_string(&mut src).is_err() {
        return Err(PyString::from(format!("failed to read the source file: {}",
                                          path.to_str().unwrap()))
                           .as_ptr());
    }
    match syn::parse_crate(&src) {
        Ok(krate) => {
            syn::visit::walk_crate(krate_data, &krate);
            krate_data.collect_values();
        }
        Err(err) => return Err(PyString::from(err).as_ptr()),
    };
    Ok(())
}

#[doc(hidden)]
#[derive(Debug)]
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
            prefixes: prefixes,
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
                args.iter()
                    .fold(&mut fndef, |mut acc, arg| {
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
                    syn::FunctionRetTy::Default => fndef.push_str("type(void)"),
                    syn::FunctionRetTy::Ty(ty) => {
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

    fn add_fn(&mut self, name: String, fn_decl: &syn::FnDecl) {
        for prefix in &self.prefixes {
            if name.starts_with(prefix) {
                let syn::FnDecl { inputs, output, .. } = fn_decl.clone();
                let mut args = vec![];
                for arg in inputs {
                    match arg {
                        syn::FnArg::Captured(_, ty) => args.push(ty),
                        syn::FnArg::Ignored(ty) => args.push(ty),
                        _ => continue,
                    }
                }
                self.functions
                    .push(FnDef {
                              name,
                              args: args,
                              output: output,
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

fn type_repr(ty: &syn::Ty, r: Option<&str>) -> Result<String, ()> {
    let mut repr = String::new();
    match *ty {
        syn::Ty::Path(_, ref path) => {
            let syn::Path { ref segments, .. } = *path;
            if let Some(ty) = segments.last() {
                if r.is_some() {
                    Ok(format!("type({} {})", r.unwrap(), ty.ident))
                } else {
                    Ok(format!("type({})", ty.ident))
                }
            } else {
                Err(())
            }
        }
        syn::Ty::Ptr(ref ty) => {
            let syn::MutTy {
                ref ty,
                ref mutability,
            } = **ty;
            let m = match *mutability {
                syn::Mutability::Immutable => "*const",
                syn::Mutability::Mutable => "*mut",
            };
            repr.push_str(&type_repr(&*ty, Some(m))?);
            Ok(repr)
        }
        syn::Ty::Rptr(_, ref ty) => {
            let syn::MutTy {
                ref ty,
                ref mutability,
            } = **ty;
            let m = match *mutability {
                syn::Mutability::Immutable => "&",
                syn::Mutability::Mutable => "&mut",
            };
            repr.push_str(&type_repr(&*ty, Some(m))?);
            Ok(repr)
        }
        _ => Err(()),
    }
}

impl syn::visit::Visitor for KrateData {
    fn visit_item(&mut self, item: &syn::Item) {
        match item.node {
            syn::ItemKind::Fn(ref fn_decl, ..) => {
                if let syn::Visibility::Public = item.vis {
                    let name = format!("{}", item.ident);
                    self.add_fn(name, &*fn_decl)
                }
            }
            syn::ItemKind::Mod(Some(ref items)) => {
                for item in items {
                    self.visit_item(item);
                }
            }
            _ => {
                /*
                ignored:
                ExternCrate(Option<Ident>),
                Use(Box<ViewPath>),
                Static(Box<Ty>, Mutability, Box<Expr>),
                Const(Box<Ty>, Box<Expr>),
                ForeignMod(ForeignMod),
                Ty(Box<Ty>, Generics),
                Enum(Vec<Variant>, Generics),
                Struct(VariantData, Generics),
                Union(VariantData, Generics),
                Trait(Unsafety, Generics, Vec<TyParamBound>, Vec<TraitItem>),
                DefaultImpl(Unsafety, Path),
                Impl(Unsafety, ImplPolarity, Generics, Option<Path>, Box<Ty>, Vec<ImplItem>),
                Mac(Mac),
                */
            }
        }
    }
}

#[derive(Debug)]
struct FnDef {
    name: String,
    output: syn::FunctionRetTy,
    args: Vec<syn::Ty>,
}

// C FFI for KrateData objects:
#[doc(hidden)]
#[no_mangle]
pub extern "C" fn krate_data_new(ptr: *mut PyList) -> *mut KrateData {
    let p = unsafe { PyList::from_ptr(ptr) };
    let p: Vec<String> = PyList::into(p);
    Box::into_raw(Box::new(KrateData::new(p)))
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn krate_data_free(ptr: *mut KrateData) {
    if ptr.is_null() {
        return;
    }
    unsafe { *(Box::from_raw(ptr)) };
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
        Some(val) => PyString::from(val).as_ptr(),
        None => PyString::from("NO_IDX_ERROR").as_ptr(),
    }
}
