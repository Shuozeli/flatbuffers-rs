use std::fmt::Write as FmtWrite;

use crate::builder::SchemaBuilder;
use crate::chooser::Chooser;
use crate::types::{StructInfo, ALL_SCALAR_TYPES};

impl<C: Chooser> SchemaBuilder<C> {
    pub(crate) fn gen_and_emit_struct(&mut self) {
        let name = self.fresh_name("Struct");
        let num_fields = self.chooser.pick(1, self.config.max_fields_per_type);
        let struct_index = self.structs.len();

        let mut lines = Vec::new();
        for i in 0..num_fields {
            let fname = self.field_name(i);
            let ftype = self.random_struct_field_type(struct_index);
            lines.push(format!("  {fname}: {ftype};"));
        }

        self.maybe_emit_doc_comment(&format!("Struct {name}."));
        let force_align = if self.chooser.flip(self.config.prob_force_align) {
            let alignments = [2, 4, 8, 16];
            let align = alignments[self.chooser.pick(0, alignments.len() - 1)];
            format!(" (force_align: {align})")
        } else {
            String::new()
        };
        writeln!(self.output, "struct {name}{force_align} {{").unwrap();
        for line in &lines {
            writeln!(self.output, "{line}").unwrap();
        }
        writeln!(self.output, "}}\n").unwrap();

        let qualified_name = self.qualified_name(&name);
        self.structs.push(StructInfo {
            name,
            qualified_name,
        });
    }

    /// Pick a valid field type for a struct field.
    /// Structs can only contain: scalars, enums, earlier structs, fixed arrays.
    pub(crate) fn random_struct_field_type(&mut self, struct_index: usize) -> String {
        let mut pool: Vec<String> = ALL_SCALAR_TYPES.iter().map(|s| s.to_string()).collect();
        for e in &self.enums {
            pool.push(self.reference_name(&e.name, &e.qualified_name));
        }
        for s in &self.structs[..struct_index] {
            pool.push(self.reference_name(&s.name, &s.qualified_name));
        }

        let base = pool[self.chooser.pick(0, pool.len() - 1)].clone();

        if self.config.max_fixed_array_len > 0 && self.chooser.flip(self.config.prob_fixed_array) {
            let len = self.chooser.pick(1, self.config.max_fixed_array_len);
            format!("[{base}:{len}]")
        } else {
            base
        }
    }
}
