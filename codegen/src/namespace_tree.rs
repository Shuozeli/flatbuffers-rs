//! Shared namespace tree for grouping FlatBuffers types by namespace hierarchy.
//!
//! Both the Rust and TypeScript code generators need to group types (enums,
//! structs, tables) into nested namespace blocks. This module provides the
//! shared data structures and tree-building logic.

use std::collections::BTreeMap;

use flatc_rs_schema::resolved::ResolvedSchema;
use flatc_rs_schema::Namespace;

use super::should_generate;

/// What kind of type an entry represents.
pub enum TypeEntry {
    Enum(usize),
    Struct(usize),
    Table(usize),
}

/// A tree node for grouping types by namespace hierarchy.
///
/// This prevents duplicate namespace blocks when multiple types share
/// a common prefix (e.g., `MyGame.Example` and `MyGame.Example2` both need
/// a single wrapping namespace block for `MyGame`).
pub struct NamespaceNode {
    pub children: BTreeMap<String, NamespaceNode>,
    pub types: Vec<TypeEntry>,
}

impl NamespaceNode {
    pub fn new() -> Self {
        Self {
            children: BTreeMap::new(),
            types: Vec::new(),
        }
    }

    /// Insert a type at the given namespace path (split into components).
    pub fn insert(&mut self, parts: &[&str], entry: TypeEntry) {
        if parts.is_empty() {
            self.types.push(entry);
        } else {
            self.children
                .entry(parts[0].to_string())
                .or_insert_with(NamespaceNode::new)
                .insert(&parts[1..], entry);
        }
    }
}

/// Extract namespace string from an optional Namespace message.
pub fn namespace_str(ns: Option<&Namespace>) -> String {
    ns.and_then(|n| n.namespace.as_deref())
        .unwrap_or("")
        .to_string()
}

/// Build a namespace tree from a schema, filtering by declaration file.
///
/// Enums, structs, and tables are inserted into the tree at their namespace
/// path. Types whose `declaration_file` doesn't match the filter are skipped.
pub fn build_namespace_tree(
    schema: &ResolvedSchema,
    filter: &Option<std::collections::HashSet<String>>,
) -> NamespaceNode {
    let mut root = NamespaceNode::new();

    for (i, enum_def) in schema.enums.iter().enumerate() {
        if !should_generate(enum_def.declaration_file.as_deref(), filter) {
            continue;
        }
        let ns = namespace_str(enum_def.namespace.as_ref());
        let parts: Vec<&str> = if ns.is_empty() {
            vec![]
        } else {
            ns.split('.').collect()
        };
        root.insert(&parts, TypeEntry::Enum(i));
    }

    for (i, obj) in schema.objects.iter().enumerate() {
        if !should_generate(obj.declaration_file.as_deref(), filter) {
            continue;
        }
        let ns = namespace_str(obj.namespace.as_ref());
        let parts: Vec<&str> = if ns.is_empty() {
            vec![]
        } else {
            ns.split('.').collect()
        };
        let entry = if obj.is_struct {
            TypeEntry::Struct(i)
        } else {
            TypeEntry::Table(i)
        };
        root.insert(&parts, entry);
    }

    root
}
