use std::fmt::Write as FmtWrite;

use crate::builder::SchemaBuilder;
use crate::chooser::Chooser;
use crate::types::UnionInfo;

impl<C: Chooser> SchemaBuilder<C> {
    pub(crate) fn gen_and_emit_union(&mut self, variant_tables: Vec<String>) {
        let name = self.fresh_name("Union");

        writeln!(self.output, "union {name} {{").unwrap();
        let variants: Vec<String> = variant_tables.iter().map(|v| format!("  {v}")).collect();
        writeln!(self.output, "{}", variants.join(",\n")).unwrap();
        writeln!(self.output, "}}\n").unwrap();

        let qualified_name = self.qualified_name(&name);
        self.unions.push(UnionInfo {
            name,
            qualified_name,
        });
    }
}
