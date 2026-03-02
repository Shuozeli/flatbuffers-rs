use std::fmt::Write as FmtWrite;

use crate::builder::SchemaBuilder;
use crate::chooser::Chooser;

impl<C: Chooser> SchemaBuilder<C> {
    /// Emit root_type, file_identifier, file_extension, and rpc_service.
    pub(crate) fn emit_metadata(&mut self) {
        // root_type -- pick a table in the current (last) namespace
        if self.config.use_root_type && !self.tables.is_empty() {
            let candidates: Vec<usize> = self
                .tables
                .iter()
                .enumerate()
                .filter(|(_, t)| match &self.current_namespace {
                    Some(ns) => {
                        t.qualified_name.starts_with(ns)
                            && t.qualified_name[ns.len()..].starts_with('.')
                    }
                    None => !t.qualified_name.contains('.'),
                })
                .map(|(i, _)| i)
                .collect();
            let idx = if candidates.is_empty() {
                self.chooser.pick(0, self.tables.len() - 1)
            } else {
                candidates[self.chooser.pick(0, candidates.len() - 1)]
            };
            let root = &self.tables[idx].name;
            writeln!(self.output, "root_type {root};").unwrap();
        }

        // file_identifier
        if self.config.use_file_ident && self.chooser.flip(self.config.prob_file_ident) {
            let ident: String = (0..4)
                .map(|_| self.chooser.pick(b'A' as usize, b'Z' as usize) as u8 as char)
                .collect();
            writeln!(self.output, "file_identifier \"{ident}\";").unwrap();
        }

        // file_extension
        if self.chooser.flip(self.config.prob_file_extension) {
            let exts = ["bin", "dat", "fb", "fbs", "msg"];
            let ext = exts[self.chooser.pick(0, exts.len() - 1)];
            writeln!(self.output, "file_extension \"{ext}\";").unwrap();
        }

        // rpc_service
        if !self.tables.is_empty() && self.chooser.flip(self.config.prob_rpc_service) {
            self.gen_and_emit_rpc_service();
        }
    }

    pub(crate) fn gen_and_emit_rpc_service(&mut self) {
        let name = self.fresh_name("Service");
        let num_methods = self.chooser.pick(1, self.config.max_rpc_methods);

        if self.chooser.flip(self.config.prob_doc_comment) {
            writeln!(self.output, "/// RPC service for {name}.").unwrap();
        }
        writeln!(self.output, "rpc_service {name} {{").unwrap();

        // Collect tables in the current namespace for method types
        let current_ns_tables: Vec<String> = self
            .tables
            .iter()
            .filter(|t| match &self.current_namespace {
                Some(ns) => {
                    t.qualified_name.starts_with(ns)
                        && t.qualified_name[ns.len()..].starts_with('.')
                }
                None => !t.qualified_name.contains('.'),
            })
            .map(|t| t.name.clone())
            .collect();

        let table_pool: Vec<String> = if current_ns_tables.is_empty() {
            self.tables.iter().map(|t| t.name.clone()).collect()
        } else {
            current_ns_tables
        };

        let method_names = ["Get", "Put", "List", "Create", "Delete", "Update", "Query"];
        for i in 0..num_methods {
            let method_name = if i < method_names.len() {
                method_names[i].to_string()
            } else {
                format!("Method{i}")
            };
            let req = &table_pool[self.chooser.pick(0, table_pool.len() - 1)];
            let resp = &table_pool[self.chooser.pick(0, table_pool.len() - 1)];

            let mut attrs = Vec::new();
            let streaming_modes = ["none", "server", "client", "bidi"];
            if self.chooser.flip(0.3) {
                let mode = streaming_modes[self.chooser.pick(0, streaming_modes.len() - 1)];
                attrs.push(format!("streaming: \"{mode}\""));
            }
            if self.chooser.flip(0.2) {
                attrs.push("idempotent".to_string());
            }

            let attr_str = if attrs.is_empty() {
                String::new()
            } else {
                format!(" ({})", attrs.join(", "))
            };

            if self.chooser.flip(self.config.prob_doc_comment) {
                writeln!(self.output, "  /// {method_name} method.").unwrap();
            }
            writeln!(self.output, "  {method_name}({req}):{resp}{attr_str};").unwrap();
        }

        writeln!(self.output, "}}\n").unwrap();
    }
}
