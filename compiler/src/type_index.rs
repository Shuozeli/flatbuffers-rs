use std::collections::HashMap;

use flatc_rs_schema as schema;

use crate::error::{AnalyzeError, Result};

/// Reference to a user-defined type in the schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeRef {
    /// Index into `schema.objects`.
    Object(usize),
    /// Index into `schema.enums`.
    Enum(usize),
}

/// Symbol table mapping fully-qualified type names to schema indices.
#[derive(Debug)]
pub struct TypeIndex {
    /// Maps fully-qualified name (e.g. "MyGame.Sample.Monster") -> TypeRef.
    /// This is the authoritative mapping; duplicate FQNs are an error.
    fqn_types: HashMap<String, TypeRef>,
    /// Maps short name (e.g. "Monster") -> TypeRef.
    /// Short name collisions across namespaces are allowed; they become
    /// ambiguous and require FQN or namespace-qualified resolution.
    short_types: HashMap<String, Option<TypeRef>>,
}

impl TypeIndex {
    /// Build the type index from a parsed schema.
    ///
    /// Each object and enum is registered under its fully-qualified name
    /// (namespace.name). Short names are also indexed but marked ambiguous
    /// if multiple types share the same short name.
    pub fn build(schema: &schema::Schema) -> Result<Self> {
        let mut fqn_types = HashMap::new();
        let mut short_types: HashMap<String, Option<TypeRef>> = HashMap::new();

        for (i, obj) in schema.objects.iter().enumerate() {
            let name = obj.name.as_deref().unwrap_or("");
            let fqn = fully_qualified_name(obj.namespace.as_ref(), name);
            if fqn_types.contains_key(&fqn) {
                return Err(AnalyzeError::DuplicateName {
                    name: fqn,
                    span: obj.span.clone(),
                });
            }
            let type_ref = TypeRef::Object(i);
            fqn_types.insert(fqn, type_ref);
            // Register short name; mark ambiguous on collision
            short_types
                .entry(name.to_string())
                .and_modify(|existing| {
                    // A different type already registered this short name -> ambiguous
                    if *existing != Some(type_ref) {
                        *existing = None;
                    }
                })
                .or_insert(Some(type_ref));
        }

        for (i, enum_decl) in schema.enums.iter().enumerate() {
            let name = enum_decl.name.as_deref().unwrap_or("");
            let fqn = fully_qualified_name(enum_decl.namespace.as_ref(), name);
            if fqn_types.contains_key(&fqn) {
                return Err(AnalyzeError::DuplicateName {
                    name: fqn,
                    span: enum_decl.span.clone(),
                });
            }
            let type_ref = TypeRef::Enum(i);
            fqn_types.insert(fqn, type_ref);
            short_types
                .entry(name.to_string())
                .and_modify(|existing| {
                    if *existing != Some(type_ref) {
                        *existing = None;
                    }
                })
                .or_insert(Some(type_ref));
        }

        Ok(Self {
            fqn_types,
            short_types,
        })
    }

    /// Resolve a type name, trying in order:
    /// 1. Exact FQN match
    /// 2. Prepend `current_namespace` (e.g. "Monster" -> "MyGame.Sample.Monster")
    /// 3. Short name match (only if unambiguous)
    pub fn resolve(&self, name: &str, current_namespace: Option<&str>) -> Option<TypeRef> {
        // Try exact FQN match first
        if let Some(&r) = self.fqn_types.get(name) {
            return Some(r);
        }
        // Try prepending the current namespace
        if let Some(ns) = current_namespace {
            if !ns.is_empty() {
                let qualified = format!("{ns}.{name}");
                if let Some(&r) = self.fqn_types.get(&qualified) {
                    return Some(r);
                }
            }
        }
        // Try unambiguous short name
        if let Some(Some(r)) = self.short_types.get(name) {
            return Some(*r);
        }
        None
    }
}

/// Build a fully-qualified name from an optional namespace and a short name.
fn fully_qualified_name(ns: Option<&schema::Namespace>, name: &str) -> String {
    match ns.and_then(|n| n.namespace.as_deref()) {
        Some(ns) if !ns.is_empty() => format!("{ns}.{name}"),
        _ => name.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn make_namespace(ns: &str) -> Option<schema::Namespace> {
        let mut n = schema::Namespace::new();
        n.namespace = Some(ns.to_string());
        Some(n)
    }

    #[test]
    fn test_build_and_resolve_simple() {
        let mut schema = schema::Schema::new();

        let mut obj = schema::Object::new();
        obj.name = Some("Monster".into());
        schema.objects.push(obj);

        let mut enum_decl = schema::Enum::new();
        enum_decl.name = Some("Color".into());
        schema.enums.push(enum_decl);

        let index = TypeIndex::build(&schema).unwrap();
        assert_eq!(index.resolve("Monster", None), Some(TypeRef::Object(0)));
        assert_eq!(index.resolve("Color", None), Some(TypeRef::Enum(0)));
        assert_eq!(index.resolve("Unknown", None), None);
    }

    #[test]
    fn test_resolve_with_namespace() {
        let mut schema = schema::Schema::new();

        let mut obj = schema::Object::new();
        obj.name = Some("Monster".into());
        obj.namespace = make_namespace("MyGame.Sample");
        schema.objects.push(obj);

        let index = TypeIndex::build(&schema).unwrap();

        // FQN lookup
        assert_eq!(
            index.resolve("MyGame.Sample.Monster", None),
            Some(TypeRef::Object(0))
        );
        // Short name lookup
        assert_eq!(index.resolve("Monster", None), Some(TypeRef::Object(0)));
        // Namespace-qualified lookup
        assert_eq!(
            index.resolve("Monster", Some("MyGame.Sample")),
            Some(TypeRef::Object(0))
        );
    }

    #[test]
    fn test_duplicate_name_error() {
        let mut schema = schema::Schema::new();

        let mut obj1 = schema::Object::new();
        obj1.name = Some("Monster".into());
        schema.objects.push(obj1);

        let mut obj2 = schema::Object::new();
        obj2.name = Some("Monster".into());
        schema.objects.push(obj2);

        let result = TypeIndex::build(&schema);
        match result {
            Err(AnalyzeError::DuplicateName { name, .. }) => assert_eq!(name, "Monster"),
            _ => panic!("Expected DuplicateName error, got {:?}", result),
        }
    }
}
