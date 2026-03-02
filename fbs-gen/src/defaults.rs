use std::fmt::Write as FmtWrite;

use crate::builder::SchemaBuilder;
use crate::chooser::Chooser;
use crate::types::{ALL_SCALAR_TYPES, TYPE_ALIASES};

impl<C: Chooser> SchemaBuilder<C> {
    pub(crate) fn maybe_default_value(&mut self, ftype: &str) -> String {
        if !self.chooser.flip(self.config.prob_default_value) {
            return String::new();
        }

        // Optional scalar: `= null`
        let is_scalar_type = ALL_SCALAR_TYPES.contains(&ftype)
            || TYPE_ALIASES.iter().any(|&(_, alias)| alias == ftype);
        if is_scalar_type && self.chooser.flip(self.config.prob_null_default) {
            return " = null".to_string();
        }

        // Extract the short name for enum lookup (strip namespace prefix)
        let short_ftype = if let Some(dot_pos) = ftype.rfind('.') {
            &ftype[dot_pos + 1..]
        } else {
            ftype
        };

        match short_ftype {
            "bool" => {
                if self.chooser.flip(0.5) {
                    " = true".to_string()
                } else {
                    " = false".to_string()
                }
            }
            "byte" | "int8" => {
                format!(" = {}", self.chooser.pick_i64(-128, 127) as i8)
            }
            "ubyte" | "uint8" => {
                format!(" = {}", self.chooser.pick(0, 255))
            }
            "short" | "int16" => {
                format!(" = {}", self.chooser.pick_i64(-100, 100) as i16)
            }
            "ushort" | "uint16" => {
                format!(" = {}", self.chooser.pick(0, 200))
            }
            "int" | "int32" => {
                format!(" = {}", self.chooser.pick_i64(-1000, 1000) as i32)
            }
            "uint" | "uint32" => {
                format!(" = {}", self.chooser.pick(0, 1000))
            }
            "long" | "int64" => {
                format!(" = {}", self.chooser.pick_i64(-1000, 1000))
            }
            "ulong" | "uint64" => {
                format!(" = {}", self.chooser.pick(0, 1000))
            }
            "float" | "float32" => {
                if self.chooser.flip(self.config.prob_nan_inf_default) {
                    let special = ["nan", "inf", "-inf", "+inf", "infinity", "-infinity"];
                    format!(" = {}", special[self.chooser.pick(0, special.len() - 1)])
                } else {
                    let vals = [0.0, 1.0, -1.0, 3.5, 100.0];
                    format!(" = {}", vals[self.chooser.pick(0, vals.len() - 1)])
                }
            }
            "double" | "float64" => {
                if self.chooser.flip(self.config.prob_nan_inf_default) {
                    let special = ["nan", "inf", "-inf", "+inf", "infinity", "-infinity"];
                    format!(" = {}", special[self.chooser.pick(0, special.len() - 1)])
                } else {
                    let vals = [0.0, 1.0, -1.0, 2.5, 100.0];
                    format!(" = {}", vals[self.chooser.pick(0, vals.len() - 1)])
                }
            }
            "string" => {
                let strings = ["", "hello", "test", "foo"];
                format!(
                    " = \"{}\"",
                    strings[self.chooser.pick(0, strings.len() - 1)]
                )
            }
            _ => {
                if let Some(e) = self.enums.iter().find(|e| e.name == short_ftype) {
                    let vname =
                        e.value_names[self.chooser.pick(0, e.value_names.len() - 1)].clone();
                    format!(" = {vname}")
                } else {
                    String::new()
                }
            }
        }
    }

    pub(crate) fn maybe_emit_doc_comment(&mut self, subject: &str) {
        if self.chooser.flip(self.config.prob_doc_comment) {
            writeln!(self.output, "/// {subject}").unwrap();
        }
    }

    pub(crate) fn maybe_alias_scalar(&mut self, scalar: &str) -> String {
        if self.chooser.flip(self.config.prob_type_alias) {
            for &(canonical, alias) in TYPE_ALIASES {
                if scalar == canonical {
                    return alias.to_string();
                }
            }
        }
        scalar.to_string()
    }

    /// Get the name to use when referencing a table by index from the current namespace.
    pub(crate) fn table_ref_name(&self, idx: usize) -> String {
        let t = &self.tables[idx];
        self.reference_name(&t.name, &t.qualified_name)
    }

    pub(crate) fn is_table_ref(&self, name: &str) -> bool {
        self.tables
            .iter()
            .any(|t| t.name == name || t.qualified_name == name)
    }

    pub(crate) fn is_struct_ref(&self, name: &str) -> bool {
        self.structs
            .iter()
            .any(|s| s.name == name || s.qualified_name == name)
    }
}
