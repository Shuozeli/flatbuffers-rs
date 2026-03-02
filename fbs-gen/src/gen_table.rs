use std::fmt::Write as FmtWrite;

use crate::builder::SchemaBuilder;
use crate::chooser::Chooser;
use crate::types::{TableInfo, ALL_SCALAR_TYPES};

impl<C: Chooser> SchemaBuilder<C> {
    /// Generate a variant table (simple table for union members).
    pub(crate) fn gen_and_emit_variant_table(&mut self) -> String {
        let name = self.fresh_name("Table");
        let num_fields = self.chooser.pick(1, 3);

        let mut lines = Vec::new();
        for i in 0..num_fields {
            let fname = self.field_name(i);
            let ftype = self.random_simple_table_field_type();
            lines.push(format!("  {fname}: {ftype};"));
        }

        writeln!(self.output, "table {name} {{").unwrap();
        for line in &lines {
            writeln!(self.output, "{line}").unwrap();
        }
        writeln!(self.output, "}}\n").unwrap();

        let qualified_name = self.qualified_name(&name);
        self.tables.push(TableInfo {
            name: name.clone(),
            qualified_name,
        });
        name
    }

    /// Simple field types for variant tables (no unions, no table refs).
    pub(crate) fn random_simple_table_field_type(&mut self) -> String {
        let mut pool: Vec<String> = ALL_SCALAR_TYPES.iter().map(|s| s.to_string()).collect();
        pool.push("string".to_string());
        for e in &self.enums {
            pool.push(self.reference_name(&e.name, &e.qualified_name));
        }
        for s in &self.structs {
            pool.push(self.reference_name(&s.name, &s.qualified_name));
        }
        pool[self.chooser.pick(0, pool.len() - 1)].clone()
    }

    /// Generate a full table (can reference everything).
    pub(crate) fn gen_and_emit_full_table(&mut self) {
        let name = self.fresh_name("Table");
        let num_fields = self.chooser.pick(1, self.config.max_fields_per_type);

        let use_ids = self.chooser.flip(self.config.prob_field_ids);
        let mut next_id: u16 = 0;
        let mut has_key = false;

        let mut lines = Vec::new();
        let mut used_union = false;
        for i in 0..num_fields {
            let fname = self.field_name(i);
            if self.chooser.flip(self.config.prob_doc_comment) {
                lines.push(format!("  /// Field {fname}."));
            }

            let (ftype, is_union) = self.random_full_table_field_type(used_union);
            if is_union {
                used_union = true;
            }

            let mut attrs = Vec::new();
            let is_ref_type = ftype == "string"
                || ftype.starts_with('[')
                || self.is_table_ref(&ftype)
                || self.is_struct_ref(&ftype);

            if !is_union && is_ref_type && self.chooser.flip(self.config.prob_required) {
                attrs.push("required".to_string());
            }

            if attrs.is_empty() && self.chooser.flip(self.config.prob_deprecated) {
                attrs.push("deprecated".to_string());
            }

            let is_scalar = ALL_SCALAR_TYPES.contains(&ftype.as_str());
            if !has_key
                && !is_union
                && attrs.is_empty()
                && (is_scalar || ftype == "string")
                && self.chooser.flip(self.config.prob_key)
            {
                attrs.push("key".to_string());
                has_key = true;
            }

            if use_ids {
                if is_union {
                    attrs.push(format!("id: {}", next_id + 1));
                    next_id += 2;
                } else {
                    attrs.push(format!("id: {next_id}"));
                    next_id += 1;
                }
            }

            let default = if attrs.is_empty() && !is_union {
                self.maybe_default_value(&ftype)
            } else {
                String::new()
            };

            let attr_str = if attrs.is_empty() {
                String::new()
            } else {
                format!(" ({})", attrs.join(", "))
            };

            lines.push(format!("  {fname}: {ftype}{default}{attr_str};"));
        }

        self.maybe_emit_doc_comment(&format!("Table {name}."));
        writeln!(self.output, "table {name} {{").unwrap();
        for line in &lines {
            writeln!(self.output, "{line}").unwrap();
        }
        writeln!(self.output, "}}\n").unwrap();

        let qualified_name = self.qualified_name(&name);
        self.tables.push(TableInfo {
            name,
            qualified_name,
        });
    }

    /// Pick a field type for a full table. Returns (type_string, is_union).
    pub(crate) fn random_full_table_field_type(&mut self, used_union: bool) -> (String, bool) {
        let c = &self.config;
        let w_enum = if self.enums.is_empty() {
            0
        } else {
            c.weight_enum
        };
        let w_struct = if self.structs.is_empty() {
            0
        } else {
            c.weight_struct
        };
        let w_table = if self.tables.is_empty() {
            0
        } else {
            c.weight_table_ref
        };
        let w_union = if used_union || self.unions.is_empty() {
            0
        } else {
            c.weight_union
        };
        let weights = [
            c.weight_scalar,
            c.weight_string,
            w_enum,
            w_struct,
            w_table,
            c.weight_vector,
            w_union,
        ];

        let category = self.chooser.pick_weighted(&weights);
        match category {
            0 => {
                // scalar
                let idx = self.chooser.pick(0, ALL_SCALAR_TYPES.len() - 1);
                (ALL_SCALAR_TYPES[idx].to_string(), false)
            }
            1 => {
                // string
                ("string".to_string(), false)
            }
            2 => {
                // enum
                let idx = self.chooser.pick(0, self.enums.len() - 1);
                let name = self.enums[idx].name.clone();
                let qname = self.enums[idx].qualified_name.clone();
                (self.reference_name(&name, &qname), false)
            }
            3 => {
                // struct
                let idx = self.chooser.pick(0, self.structs.len() - 1);
                let name = self.structs[idx].name.clone();
                let qname = self.structs[idx].qualified_name.clone();
                (self.reference_name(&name, &qname), false)
            }
            4 => {
                // table ref
                let idx = self.chooser.pick(0, self.tables.len() - 1);
                let ref_name = self.table_ref_name(idx);
                (ref_name, false)
            }
            5 => {
                // vector
                (self.random_vector_type(), false)
            }
            6 => {
                // union
                let idx = self.chooser.pick(0, self.unions.len() - 1);
                let name = self.unions[idx].name.clone();
                let qname = self.unions[idx].qualified_name.clone();
                (self.reference_name(&name, &qname), true)
            }
            _ => unreachable!("pick_weighted returned invalid category"),
        }
    }

    pub(crate) fn random_vector_type(&mut self) -> String {
        let c = &self.config;
        let w_enum = if self.enums.is_empty() {
            0
        } else {
            c.weight_vec_enum
        };
        let w_struct = if self.structs.is_empty() {
            0
        } else {
            c.weight_vec_struct
        };
        let w_table = if self.tables.is_empty() {
            0
        } else {
            c.weight_vec_table
        };
        let weights = [
            c.weight_vec_scalar,
            c.weight_vec_string,
            w_enum,
            w_struct,
            w_table,
        ];

        let category = self.chooser.pick_weighted(&weights);
        match category {
            0 => {
                // scalar
                let idx = self.chooser.pick(0, ALL_SCALAR_TYPES.len() - 1);
                format!("[{}]", ALL_SCALAR_TYPES[idx])
            }
            1 => {
                // string
                "[string]".to_string()
            }
            2 => {
                // enum
                let idx = self.chooser.pick(0, self.enums.len() - 1);
                let name = self.enums[idx].name.clone();
                let qname = self.enums[idx].qualified_name.clone();
                format!("[{}]", self.reference_name(&name, &qname))
            }
            3 => {
                // struct
                let idx = self.chooser.pick(0, self.structs.len() - 1);
                let name = self.structs[idx].name.clone();
                let qname = self.structs[idx].qualified_name.clone();
                format!("[{}]", self.reference_name(&name, &qname))
            }
            4 => {
                // table
                let idx = self.chooser.pick(0, self.tables.len() - 1);
                let ref_name = self.table_ref_name(idx);
                format!("[{ref_name}]")
            }
            _ => unreachable!("pick_weighted returned invalid category"),
        }
    }
}
