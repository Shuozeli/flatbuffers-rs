use std::collections::HashSet;
use std::fmt::Write as FmtWrite;

use crate::builder::SchemaBuilder;
use crate::chooser::Chooser;
use crate::types::{EnumInfo, SCALAR_INT_TYPES};

impl<C: Chooser> SchemaBuilder<C> {
    pub(crate) fn gen_and_emit_enum(&mut self) {
        let name = self.fresh_name("Enum");
        let underlying_idx = self.chooser.pick(0, SCALAR_INT_TYPES.len() - 1);
        let underlying = SCALAR_INT_TYPES[underlying_idx].to_string();
        let underlying_display = self.maybe_alias_scalar(&underlying);

        let is_bit_flags = self.chooser.flip(self.config.prob_bit_flags);
        let num_values = if is_bit_flags {
            // bit_flags enums typically have fewer values to avoid overflow
            self.chooser.pick(2, 6.min(self.config.max_enum_values))
        } else {
            self.chooser.pick(2, self.config.max_enum_values)
        };

        let mut value_names = Vec::new();
        let mut used_value_names: HashSet<String> = HashSet::new();

        for i in 0..num_values {
            let vname = loop {
                let candidate = format!("V{i}");
                if used_value_names.insert(candidate.clone()) {
                    break candidate;
                }
            };
            value_names.push(vname);
        }

        // Emit
        self.maybe_emit_doc_comment(&format!("Enum {name}."));
        let meta = if is_bit_flags { " (bit_flags)" } else { "" };
        let vals: Vec<String> = value_names
            .iter()
            .enumerate()
            .map(|(i, vn)| format!("{vn} = {i}"))
            .collect();
        writeln!(self.output, "enum {name} : {underlying_display}{meta} {{").unwrap();
        writeln!(self.output, "  {}", vals.join(",\n  ")).unwrap();
        writeln!(self.output, "}}\n").unwrap();

        let qualified_name = self.qualified_name(&name);
        self.enums.push(EnumInfo {
            name,
            qualified_name,
            underlying,
            value_names,
        });
    }
}
