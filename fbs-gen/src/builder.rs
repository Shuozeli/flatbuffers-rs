use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::HashSet;
use std::fmt::Write as FmtWrite;

use crate::chooser::{Chooser, RngChooser};
use crate::config::GenConfig;
use crate::types::{EnumInfo, StructInfo, TableInfo, UnionInfo};

pub struct SchemaBuilder<C: Chooser> {
    pub(crate) chooser: C,
    pub(crate) config: GenConfig,
    pub(crate) used_names: HashSet<String>,
    pub(crate) enums: Vec<EnumInfo>,
    pub(crate) structs: Vec<StructInfo>,
    pub(crate) tables: Vec<TableInfo>,
    pub(crate) unions: Vec<UnionInfo>,
    pub(crate) output: String,
    pub(crate) name_counter: usize,
    /// The current active namespace (None if no namespace declared yet).
    pub(crate) current_namespace: Option<String>,
    /// All namespace strings used in this schema.
    pub(crate) namespaces: Vec<String>,
}

/// Convenience wrapper that preserves the original API.
impl SchemaBuilder<RngChooser> {
    /// Generate a random valid `.fbs` schema from a deterministic seed.
    /// The same `(seed, config)` always produces the same output.
    pub fn generate(seed: u64, config: GenConfig) -> String {
        let chooser = RngChooser(StdRng::seed_from_u64(seed));
        SchemaBuilder::with_chooser(chooser, config).build()
    }
}

impl<C: Chooser> SchemaBuilder<C> {
    pub fn with_chooser(chooser: C, config: GenConfig) -> Self {
        SchemaBuilder {
            chooser,
            config,
            used_names: HashSet::new(),
            enums: Vec::new(),
            structs: Vec::new(),
            tables: Vec::new(),
            unions: Vec::new(),
            output: String::with_capacity(2048),
            name_counter: 0,
            current_namespace: None,
            namespaces: Vec::new(),
        }
    }

    pub fn build(mut self) -> String {
        // Decide if we use namespaces at all
        let use_ns = self.config.use_namespace && self.chooser.flip(self.config.prob_namespace);

        // Decide how many namespace sections we'll have
        let num_ns_sections = if use_ns {
            if self.config.max_namespaces > 1 && self.chooser.flip(self.config.prob_multi_namespace)
            {
                self.chooser.pick(2, self.config.max_namespaces)
            } else {
                1
            }
        } else {
            0
        };

        // Pre-generate all namespace names
        let ns_names: Vec<String> = (0..num_ns_sections)
            .map(|_| self.gen_namespace_name())
            .collect();
        self.namespaces = ns_names.clone();

        if num_ns_sections == 0 {
            // No namespaces: generate everything in one flat section
            self.gen_section_types();
        } else if num_ns_sections == 1 {
            // Single namespace at top
            let ns = &ns_names[0];
            writeln!(self.output, "namespace {ns};\n").unwrap();
            self.current_namespace = Some(ns.clone());
            self.gen_section_types();
        } else {
            // Multiple namespaces: distribute types across namespace sections.
            for (i, ns) in ns_names.iter().enumerate() {
                writeln!(self.output, "namespace {ns};\n").unwrap();
                self.current_namespace = Some(ns.clone());

                if i == 0 {
                    // First section: generate enums and structs (no deps on tables)
                    let num_enums = self.chooser.pick(0, self.config.max_enums);
                    for _ in 0..num_enums {
                        self.gen_and_emit_enum();
                    }
                    let num_structs = self.chooser.pick(0, self.config.max_structs);
                    for _ in 0..num_structs {
                        self.gen_and_emit_struct();
                    }
                    // Also emit some tables in the first section
                    let num_tables = self.chooser.pick(1, 2.min(self.config.max_tables).max(1));
                    for _ in 0..num_tables {
                        self.gen_and_emit_full_table();
                    }
                } else if i < num_ns_sections - 1 {
                    // Middle sections: more tables (can reference earlier types)
                    let num_tables = self.chooser.pick(1, 2.min(self.config.max_tables).max(1));
                    for _ in 0..num_tables {
                        self.gen_and_emit_full_table();
                    }
                } else {
                    // Last section: variant tables, unions, and remaining main tables
                    let num_unions = self.chooser.pick(0, self.config.max_unions);
                    let mut all_variant_groups: Vec<Vec<String>> = Vec::new();
                    for _ in 0..num_unions {
                        let num_variants = self.chooser.pick(2, self.config.max_union_variants);
                        let mut variant_names = Vec::new();
                        for _ in 0..num_variants {
                            let name = self.gen_and_emit_variant_table();
                            variant_names.push(name);
                        }
                        all_variant_groups.push(variant_names);
                    }
                    for variant_names in all_variant_groups {
                        self.gen_and_emit_union(variant_names);
                    }
                    let num_main = self.chooser.pick(1, self.config.max_tables);
                    for _ in 0..num_main {
                        self.gen_and_emit_full_table();
                    }
                }
            }
        }

        // root_type, file_identifier, file_extension, rpc_service
        self.emit_metadata();

        self.output
    }

    /// Generate a flat section (no namespace switching): enums, structs,
    /// variant tables, unions, main tables.
    pub(crate) fn gen_section_types(&mut self) {
        // enums
        let num_enums = self.chooser.pick(0, self.config.max_enums);
        for _ in 0..num_enums {
            self.gen_and_emit_enum();
        }

        // structs
        let num_structs = self.chooser.pick(0, self.config.max_structs);
        for _ in 0..num_structs {
            self.gen_and_emit_struct();
        }

        // variant tables + unions
        let num_unions = self.chooser.pick(0, self.config.max_unions);
        let mut all_variant_groups: Vec<Vec<String>> = Vec::new();
        for _ in 0..num_unions {
            let num_variants = self.chooser.pick(2, self.config.max_union_variants);
            let mut variant_names = Vec::new();
            for _ in 0..num_variants {
                let name = self.gen_and_emit_variant_table();
                variant_names.push(name);
            }
            all_variant_groups.push(variant_names);
        }
        for variant_names in all_variant_groups {
            self.gen_and_emit_union(variant_names);
        }

        // main tables
        let num_main = self.chooser.pick(1, self.config.max_tables);
        for _ in 0..num_main {
            self.gen_and_emit_full_table();
        }
    }
}
