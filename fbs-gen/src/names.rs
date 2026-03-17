use std::collections::HashSet;

use crate::builder::SchemaBuilder;
use crate::chooser::Chooser;

pub(crate) const SUFFIXES: &[&str] = &[
    "Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta", "Eta", "Theta", "Iota", "Kappa",
    "Lambda", "Mu", "Nu", "Xi", "Omicron", "Pi", "Rho", "Sigma", "Tau", "Upsilon", "Phi", "Chi",
    "Psi", "Omega",
];

pub(crate) const NAMESPACE_PARTS: &[&str] = &[
    "Game", "Data", "Schema", "Proto", "Model", "Core", "Items", "Player", "World", "Net",
];

impl<C: Chooser> SchemaBuilder<C> {
    pub(crate) fn fresh_name(&mut self, prefix: &str) -> String {
        loop {
            let idx = self.name_counter;
            self.name_counter += 1;
            let suffix = if idx < SUFFIXES.len() {
                SUFFIXES[idx].to_string()
            } else {
                format!("T{idx}")
            };
            let candidate = format!("{prefix}{suffix}");
            if self.used_names.insert(candidate.clone()) {
                return candidate;
            }
        }
    }

    pub(crate) fn gen_namespace_name(&mut self) -> String {
        let depth = self.chooser.pick(1, 2);
        let mut ns = String::new();
        let mut used_parts = HashSet::new();
        for i in 0..depth {
            if i > 0 {
                ns.push('.');
            }
            // Pick a part that hasn't been used at this level (avoid "Game.Game")
            let p = loop {
                let candidate = NAMESPACE_PARTS[self.chooser.pick(0, NAMESPACE_PARTS.len() - 1)];
                if used_parts.insert(candidate) {
                    break candidate;
                }
            };
            ns.push_str(p);
        }
        // Ensure unique namespace names
        if self.namespaces.contains(&ns) {
            // Append a unique suffix
            let suffix = self.namespaces.len();
            ns.push_str(&format!("{suffix}"));
        }
        ns
    }

    /// Build the qualified name for a type: `namespace.TypeName` or just `TypeName`.
    pub(crate) fn qualified_name(&self, short_name: &str) -> String {
        match &self.current_namespace {
            Some(ns) => format!("{ns}.{short_name}"),
            None => short_name.to_string(),
        }
    }

    /// Return the name to use when referencing a type from the current namespace.
    /// If the type is in the same namespace (or there are no namespaces), use the
    /// short name. If the type is in a different namespace, use the qualified name.
    pub(crate) fn reference_name(&self, short_name: &str, qualified_name: &str) -> String {
        // Extract the namespace from the qualified name
        if let Some(dot_pos) = qualified_name.rfind('.') {
            let type_ns = &qualified_name[..dot_pos];
            if let Some(cur_ns) = &self.current_namespace {
                if cur_ns == type_ns {
                    return short_name.to_string();
                }
            }
            // Different namespace: use fully-qualified name
            qualified_name.to_string()
        } else {
            // No namespace in qualified name
            short_name.to_string()
        }
    }

    pub(crate) fn field_name(&self, idx: usize) -> String {
        if idx < 26 {
            format!("field_{}", char::from(b'a' + idx as u8))
        } else {
            format!("field_{idx}")
        }
    }
}
