//! Tree-sitter grammar for FlatBuffers schema files.
//!
//! This crate provides the compiled tree-sitter parser for FlatBuffers `.fbs` files.
//!
//! ```
//! let code = "table Monster { hp: int; }";
//! let mut parser = tree_sitter::Parser::new();
//! parser
//!     .set_language(&flatc_rs_grammar::LANGUAGE.into())
//!     .expect("Error loading FlatBuffers parser");
//! let tree = parser.parse(code, None).unwrap();
//! assert!(!tree.root_node().has_error());
//! ```

use tree_sitter_language::LanguageFn;

extern "C" {
    fn tree_sitter_flatbuffers() -> *const ();
}

/// The tree-sitter [`LanguageFn`] for the FlatBuffers grammar.
pub const LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_flatbuffers) };

/// The content of the `node-types.json` file for this grammar.
pub const NODE_TYPES: &str = include_str!("node-types.json");

#[cfg(test)]
mod tests {
    #[test]
    fn test_can_load_grammar() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&super::LANGUAGE.into())
            .expect("Error loading FlatBuffers parser");
    }
}
