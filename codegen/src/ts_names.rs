//! Resolution of TypeScript type references.
//!
//! Every place the TypeScript generator names *another* schema type goes
//! through [`sym`]. How a reference is spelled depends on the active scope:
//!
//! * no scope (direct use of the per-type generators): the bare type name;
//! * [`Scope::flat`] (single-file `--ts-flat-files` layout of nested
//!   `export namespace` blocks): the bare name for types in the current
//!   namespace or the root namespace, otherwise the fully qualified dotted
//!   name (`Game.Items.Item`);
//! * [`Scope::module`] (one ES module per type, upstream `flatc --ts`
//!   layout): the bare name, recorded as an import of the target type's
//!   module. A bare name that would clash with the file's own exports or with
//!   another imported type is aliased to `Namespace_Path_Name`, matching
//!   upstream flatc.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use flatc_rs_schema::resolved::ResolvedSchema;

use crate::namespace_tree::namespace_str;

/// A referenceable generated symbol.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sym {
    /// Reader class of the table/struct at `schema.objects[idx]`.
    Obj(usize),
    /// Object API class (`XxxT`) of the table/struct at `schema.objects[idx]`.
    ObjT(usize),
    /// Enum (or union discriminator) at `schema.enums[idx]`.
    Enum(usize),
    /// `unionToXxx` helper of the union at `schema.enums[idx]`.
    UnionTo(usize),
    /// `unionListToXxx` helper of the union at `schema.enums[idx]`.
    UnionListTo(usize),
}

/// The type a file (or flat namespace block) is generated for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Owner {
    Obj(usize),
    Enum(usize),
}

impl Sym {
    fn owner(self) -> Owner {
        match self {
            Sym::Obj(i) | Sym::ObjT(i) => Owner::Obj(i),
            Sym::Enum(i) | Sym::UnionTo(i) | Sym::UnionListTo(i) => Owner::Enum(i),
        }
    }

    fn derive(self, base: &str) -> String {
        match self {
            Sym::Obj(_) | Sym::Enum(_) => base.to_string(),
            Sym::ObjT(_) => format!("{base}T"),
            Sym::UnionTo(_) => format!("unionTo{base}"),
            Sym::UnionListTo(_) => format!("unionListTo{base}"),
        }
    }
}

/// Name and namespace of the type that owns a symbol.
pub fn owner_name_ns(schema: &ResolvedSchema, owner: Owner) -> (String, String) {
    match owner {
        Owner::Obj(i) => {
            let o = &schema.objects[i];
            (o.name.clone(), namespace_str(o.namespace.as_ref()))
        }
        Owner::Enum(i) => {
            let e = &schema.enums[i];
            (e.name.clone(), namespace_str(e.namespace.as_ref()))
        }
    }
}

/// Kebab-case ("dasher") conversion used for module file and directory
/// names, as upstream flatc: `MyGame` -> `my-game`, `C2sChat` -> `c2s-chat`.
pub fn to_dasher(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let mut out = String::with_capacity(name.len() + 4);
    for (i, &c) in chars.iter().enumerate() {
        if c == '_' {
            out.push('-');
            continue;
        }
        if c.is_ascii_uppercase() && i > 0 {
            let prev = chars[i - 1];
            let next_lower = chars.get(i + 1).is_some_and(|n| n.is_ascii_lowercase());
            if prev.is_ascii_lowercase()
                || prev.is_ascii_digit()
                || (prev.is_ascii_uppercase() && next_lower)
            {
                out.push('-');
            }
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

/// Directory components (dashed) of a dotted namespace.
pub fn ns_dirs(ns: &str) -> Vec<String> {
    if ns.is_empty() {
        Vec::new()
    } else {
        ns.split('.').map(to_dasher).collect()
    }
}

/// Module path (relative to the output root, without extension) of the file
/// that defines `owner`.
pub fn owner_module_path(schema: &ResolvedSchema, owner: Owner) -> Vec<String> {
    let (name, ns) = owner_name_ns(schema, owner);
    let mut parts = ns_dirs(&ns);
    parts.push(to_dasher(&name));
    parts
}

/// ES import specifier from the module at `from` to the module at `to`
/// (both root-relative path components without extension).
pub fn relative_specifier(from: &[String], to: &[String]) -> String {
    let from_dir = &from[..from.len().saturating_sub(1)];
    let to_dir = &to[..to.len().saturating_sub(1)];
    let common = from_dir
        .iter()
        .zip(to_dir.iter())
        .take_while(|(a, b)| a == b)
        .count();
    let mut parts: Vec<String> = Vec::new();
    for _ in common..from_dir.len() {
        parts.push("..".to_string());
    }
    parts.extend(to[common..].iter().cloned());
    let joined = parts.join("/");
    if joined.starts_with("..") {
        format!("{joined}.js")
    } else {
        format!("./{joined}.js")
    }
}

enum Mode {
    Flat {
        current_ns: String,
    },
    Module {
        self_owner: Owner,
        self_path: Vec<String>,
        /// Local base names in use (the file's own exports and imports).
        taken: BTreeSet<String>,
        /// Owner -> local base name.
        bases: HashMap<Owner, String>,
        /// Specifier -> (exported, local) pairs.
        imports: BTreeMap<String, BTreeSet<(String, String)>>,
    },
}

thread_local! {
    static SCOPE: RefCell<Option<Mode>> = const { RefCell::new(None) };
}

/// RAII guard for an active resolution scope.
pub struct Scope {
    _private: (),
}

impl Scope {
    /// Flat single-file layout; `current_ns` is the dotted namespace the
    /// code is being emitted into.
    pub fn flat(current_ns: &str) -> Scope {
        SCOPE.with(|s| {
            *s.borrow_mut() = Some(Mode::Flat {
                current_ns: current_ns.to_string(),
            })
        });
        Scope { _private: () }
    }

    /// One-module-per-type layout for the file that defines `owner`.
    /// `own_exports` are the names the file itself exports.
    pub fn module(schema: &ResolvedSchema, owner: Owner, own_exports: &[String]) -> Scope {
        let (name, _) = owner_name_ns(schema, owner);
        let mut taken: BTreeSet<String> = own_exports.iter().cloned().collect();
        taken.insert(name.clone());
        taken.insert("flatbuffers".to_string());
        let mut bases = HashMap::new();
        bases.insert(owner, name);
        SCOPE.with(|s| {
            *s.borrow_mut() = Some(Mode::Module {
                self_owner: owner,
                self_path: owner_module_path(schema, owner),
                taken,
                bases,
                imports: BTreeMap::new(),
            })
        });
        Scope { _private: () }
    }

    /// Import statements recorded so far (module scope only), sorted.
    pub fn import_lines(&self) -> Vec<String> {
        SCOPE.with(|s| match s.borrow().as_ref() {
            Some(Mode::Module { imports, .. }) => imports
                .iter()
                .map(|(spec, names)| {
                    let list: Vec<String> = names
                        .iter()
                        .map(|(exported, local)| {
                            if exported == local {
                                exported.clone()
                            } else {
                                format!("{exported} as {local}")
                            }
                        })
                        .collect();
                    format!("import {{ {} }} from '{spec}';", list.join(", "))
                })
                .collect(),
            _ => Vec::new(),
        })
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        SCOPE.with(|s| *s.borrow_mut() = None);
    }
}

/// Spell a reference to `sym` from the code currently being generated.
pub fn sym(schema: &ResolvedSchema, sym: Sym) -> String {
    let owner = sym.owner();
    let (name, ns) = owner_name_ns(schema, owner);
    SCOPE.with(|s| {
        let mut guard = s.borrow_mut();
        match guard.as_mut() {
            None => sym.derive(&name),
            Some(Mode::Flat { current_ns }) => {
                if ns.is_empty() || ns == *current_ns {
                    sym.derive(&name)
                } else {
                    format!("{ns}.{}", sym.derive(&name))
                }
            }
            Some(Mode::Module {
                self_owner,
                self_path,
                taken,
                bases,
                imports,
            }) => {
                if owner == *self_owner {
                    return sym.derive(&name);
                }
                let base = match bases.get(&owner) {
                    Some(b) => b.clone(),
                    None => {
                        let base = if taken.contains(&name) {
                            let mut alias = format!("{}_{name}", ns.replace('.', "_"));
                            while taken.contains(&alias) {
                                alias.insert(0, '_');
                            }
                            alias
                        } else {
                            name.clone()
                        };
                        taken.insert(base.clone());
                        bases.insert(owner, base.clone());
                        base
                    }
                };
                let local = sym.derive(&base);
                let spec = relative_specifier(self_path, &owner_module_path(schema, owner));
                imports
                    .entry(spec)
                    .or_default()
                    .insert((sym.derive(&name), local.clone()));
                local
            }
        }
    })
}

/// TypeScript identifier of a union/enum member. Union members named after a
/// type in another namespace (`B.Bar`) become `B_Bar`, as upstream flatc.
pub fn enum_member(name: &str) -> String {
    name.replace('.', "_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dasher_matches_upstream_file_names() {
        assert_eq!(to_dasher("MyGame"), "my-game");
        assert_eq!(to_dasher("C2sChat"), "c2s-chat");
        assert_eq!(to_dasher("S2cLoginOk"), "s2c-login-ok");
        assert_eq!(to_dasher("xlqy3"), "xlqy3");
        assert_eq!(to_dasher("HTTPServer"), "http-server");
        assert_eq!(to_dasher("snake_case"), "snake-case");
    }

    #[test]
    fn relative_specifier_walks_between_directories() {
        let p = |s: &str| s.split('/').map(String::from).collect::<Vec<_>>();
        assert_eq!(relative_specifier(&p("a/b/x"), &p("a/b/y")), "./y.js");
        assert_eq!(relative_specifier(&p("a/b/x"), &p("a/c/y")), "../c/y.js");
        assert_eq!(relative_specifier(&p("x"), &p("a/y")), "./a/y.js");
        assert_eq!(relative_specifier(&p("a/x"), &p("y")), "../y.js");
    }
}
