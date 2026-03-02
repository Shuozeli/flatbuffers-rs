//! Random FlatBuffers schema generator.
//!
//! Produces valid `.fbs` text that is guaranteed to parse, analyze, and codegen
//! successfully. Uses a deterministic seed so the same seed always produces the
//! same schema -- failing seeds are trivially reproducible.
//!
//! Every random decision and probability is configurable via [`GenConfig`].

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashSet;
use std::fmt::Write as FmtWrite;

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Every random decision in the generator is controlled by this config.
/// All `prob_*` fields are probabilities in `[0.0, 1.0]`.
pub struct GenConfig {
    // -- Capacity limits --
    pub max_enums: usize,
    pub max_structs: usize,
    pub max_tables: usize,
    pub max_unions: usize,
    pub max_fields_per_type: usize,
    pub max_enum_values: usize,
    pub max_union_variants: usize,
    pub max_fixed_array_len: usize,

    // -- Feature toggles --
    pub use_namespace: bool,
    pub use_root_type: bool,
    pub use_file_ident: bool,

    // -- Namespace config --
    /// Maximum number of distinct namespace sections (1 = single namespace at top,
    /// 2+ = namespace switching mid-schema with cross-namespace references).
    pub max_namespaces: usize,
    /// Probability that cross-namespace references use fully-qualified names.
    /// When types are in different namespaces, fields must use qualified names
    /// like `Game.Items.TableAlpha` to reference types from another namespace.
    pub prob_multi_namespace: f64,

    // -- Probabilities --
    /// Probability that a namespace is emitted when `use_namespace` is true.
    pub prob_namespace: f64,
    /// Probability that a file_identifier is emitted when `use_file_ident` is true.
    pub prob_file_ident: f64,
    /// Probability that a struct field becomes a fixed-length array.
    pub prob_fixed_array: f64,
    /// Probability that a full table uses explicit field IDs.
    pub prob_field_ids: f64,
    /// Probability that an eligible ref-type field gets `(required)`.
    pub prob_required: f64,
    /// Probability that a field gets `(deprecated)` (when not already required).
    pub prob_deprecated: f64,
    /// Probability that an eligible scalar/string field gets `(key)`.
    pub prob_key: f64,
    /// Probability that a scalar/enum/string field gets a default value.
    pub prob_default_value: f64,

    // -- Table field type weights --
    /// Relative weight for picking a scalar field in a full table.
    pub weight_scalar: u32,
    /// Relative weight for picking a string field.
    pub weight_string: u32,
    /// Relative weight for picking an enum field (when enums exist).
    pub weight_enum: u32,
    /// Relative weight for picking a struct field (when structs exist).
    pub weight_struct: u32,
    /// Relative weight for picking a table-ref field (when tables exist).
    pub weight_table_ref: u32,
    /// Relative weight for picking a vector field.
    pub weight_vector: u32,
    /// Relative weight for picking a union field (when unions exist and unused).
    pub weight_union: u32,

    // -- Vector element type weights --
    /// Relative weight for picking a scalar element in a vector.
    pub weight_vec_scalar: u32,
    /// Relative weight for picking a string element in a vector.
    pub weight_vec_string: u32,
    /// Relative weight for picking a table element in a vector.
    pub weight_vec_table: u32,
    /// Relative weight for picking a struct element in a vector.
    pub weight_vec_struct: u32,
    /// Relative weight for picking an enum element in a vector.
    pub weight_vec_enum: u32,

    // -- Additional feature toggles and probabilities --
    /// Probability of emitting a file_extension declaration.
    pub prob_file_extension: f64,
    /// Probability of emitting doc comments on types/fields.
    pub prob_doc_comment: f64,
    /// Probability of emitting an rpc_service declaration.
    pub prob_rpc_service: f64,
    /// Max number of RPC methods per service.
    pub max_rpc_methods: usize,
    /// Probability that a struct gets `(force_align: N)`.
    pub prob_force_align: f64,
    /// Probability that an enum gets `(bit_flags)`.
    pub prob_bit_flags: f64,
    /// Probability that a scalar type uses an alias (int8 vs byte).
    pub prob_type_alias: f64,
    /// Probability that an optional scalar field gets `= null`.
    pub prob_null_default: f64,
    /// Probability that a float/double default uses nan/inf.
    pub prob_nan_inf_default: f64,
}

impl Default for GenConfig {
    fn default() -> Self {
        Self {
            max_enums: 4,
            max_structs: 3,
            max_tables: 5,
            max_unions: 2,
            max_fields_per_type: 6,
            max_enum_values: 8,
            max_union_variants: 4,
            max_fixed_array_len: 4,

            use_namespace: true,
            use_root_type: true,
            use_file_ident: true,

            max_namespaces: 3,
            prob_multi_namespace: 0.6,

            prob_namespace: 0.6,
            prob_file_ident: 0.3,
            prob_fixed_array: 0.2,
            prob_field_ids: 0.3,
            prob_required: 0.15,
            prob_deprecated: 0.1,
            prob_key: 0.15,
            prob_default_value: 0.35,

            weight_scalar: 20,
            weight_string: 15,
            weight_enum: 10,
            weight_struct: 10,
            weight_table_ref: 15,
            weight_vector: 20,
            weight_union: 10,

            weight_vec_scalar: 25,
            weight_vec_string: 15,
            weight_vec_table: 25,
            weight_vec_struct: 15,
            weight_vec_enum: 20,

            prob_file_extension: 0.15,
            prob_doc_comment: 0.2,
            prob_rpc_service: 0.2,
            max_rpc_methods: 3,
            prob_force_align: 0.15,
            prob_bit_flags: 0.15,
            prob_type_alias: 0.2,
            prob_null_default: 0.15,
            prob_nan_inf_default: 0.15,
        }
    }
}

// ---------------------------------------------------------------------------
// Scalar types
// ---------------------------------------------------------------------------

const SCALAR_INT_TYPES: &[&str] = &[
    "byte", "ubyte", "short", "ushort", "int", "uint", "long", "ulong",
];

const ALL_SCALAR_TYPES: &[&str] = &[
    "bool", "byte", "ubyte", "short", "ushort", "int", "uint", "long", "ulong",
    "float", "double",
];

/// Type alias pairs: (canonical, alias). Used to randomly substitute aliases.
const TYPE_ALIASES: &[(&str, &str)] = &[
    ("byte", "int8"),
    ("ubyte", "uint8"),
    ("short", "int16"),
    ("ushort", "uint16"),
    ("int", "int32"),
    ("uint", "uint32"),
    ("long", "int64"),
    ("ulong", "uint64"),
    ("float", "float32"),
    ("double", "float64"),
];

// ---------------------------------------------------------------------------
// Tracking types
// ---------------------------------------------------------------------------

struct EnumInfo {
    name: String,
    /// Fully-qualified name including namespace (e.g., "Game.Items.Color").
    qualified_name: String,
    #[allow(dead_code)]
    underlying: String,
    value_names: Vec<String>,
}

struct StructInfo {
    name: String,
    qualified_name: String,
}

struct TableInfo {
    name: String,
    qualified_name: String,
}

struct UnionInfo {
    name: String,
    qualified_name: String,
    #[allow(dead_code)]
    variant_tables: Vec<String>,
}

// ---------------------------------------------------------------------------
// Name generation helpers
// ---------------------------------------------------------------------------

const SUFFIXES: &[&str] = &[
    "Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta", "Eta", "Theta",
    "Iota", "Kappa", "Lambda", "Mu", "Nu", "Xi", "Omicron", "Pi", "Rho",
    "Sigma", "Tau", "Upsilon", "Phi", "Chi", "Psi", "Omega",
];

const NAMESPACE_PARTS: &[&str] = &[
    "Game", "Data", "Schema", "Proto", "Model", "Core", "Items", "Player",
    "World", "Net",
];

// ---------------------------------------------------------------------------
// SchemaBuilder
// ---------------------------------------------------------------------------

pub struct SchemaBuilder {
    rng: StdRng,
    config: GenConfig,
    used_names: HashSet<String>,
    enums: Vec<EnumInfo>,
    structs: Vec<StructInfo>,
    tables: Vec<TableInfo>,
    unions: Vec<UnionInfo>,
    output: String,
    name_counter: usize,
    /// The current active namespace (None if no namespace declared yet).
    current_namespace: Option<String>,
    /// All namespace strings used in this schema.
    namespaces: Vec<String>,
}

impl SchemaBuilder {
    /// Generate a random valid `.fbs` schema from a deterministic seed.
    /// The same `(seed, config)` always produces the same output.
    pub fn generate(seed: u64, config: GenConfig) -> String {
        let mut b = SchemaBuilder {
            rng: StdRng::seed_from_u64(seed),
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
        };

        // Decide if we use namespaces at all
        let use_ns = b.config.use_namespace && b.rng.gen_bool(b.config.prob_namespace);

        // Decide how many namespace sections we'll have
        let num_ns_sections = if use_ns {
            if b.config.max_namespaces > 1 && b.rng.gen_bool(b.config.prob_multi_namespace) {
                b.rng.gen_range(2..=b.config.max_namespaces)
            } else {
                1
            }
        } else {
            0
        };

        // Pre-generate all namespace names
        let ns_names: Vec<String> = (0..num_ns_sections)
            .map(|_| b.gen_namespace_name())
            .collect();
        b.namespaces = ns_names.clone();

        if num_ns_sections == 0 {
            // No namespaces: generate everything in one flat section
            b.gen_section_types();
        } else if num_ns_sections == 1 {
            // Single namespace at top
            let ns = &ns_names[0];
            writeln!(b.output, "namespace {ns};\n").unwrap();
            b.current_namespace = Some(ns.clone());
            b.gen_section_types();
        } else {
            // Multiple namespaces: distribute types across namespace sections.
            // Each section gets some types, and later sections can cross-reference
            // types from earlier sections using fully-qualified names.
            for (i, ns) in ns_names.iter().enumerate() {
                writeln!(b.output, "namespace {ns};\n").unwrap();
                b.current_namespace = Some(ns.clone());

                if i == 0 {
                    // First section: generate enums and structs (no deps on tables)
                    let num_enums = b.rng.gen_range(0..=b.config.max_enums);
                    for _ in 0..num_enums {
                        b.gen_and_emit_enum();
                    }
                    let num_structs = b.rng.gen_range(0..=b.config.max_structs);
                    for _ in 0..num_structs {
                        b.gen_and_emit_struct();
                    }
                    // Also emit some tables in the first section
                    let num_tables = b.rng.gen_range(1..=2.min(b.config.max_tables).max(1));
                    for _ in 0..num_tables {
                        b.gen_and_emit_full_table();
                    }
                } else if i < num_ns_sections - 1 {
                    // Middle sections: more tables (can reference earlier types)
                    let num_tables = b.rng.gen_range(1..=2.min(b.config.max_tables).max(1));
                    for _ in 0..num_tables {
                        b.gen_and_emit_full_table();
                    }
                } else {
                    // Last section: variant tables, unions, and remaining main tables
                    let num_unions = b.rng.gen_range(0..=b.config.max_unions);
                    let mut all_variant_groups: Vec<Vec<String>> = Vec::new();
                    for _ in 0..num_unions {
                        let num_variants =
                            b.rng.gen_range(2..=b.config.max_union_variants);
                        let mut variant_names = Vec::new();
                        for _ in 0..num_variants {
                            let name = b.gen_and_emit_variant_table();
                            variant_names.push(name);
                        }
                        all_variant_groups.push(variant_names);
                    }
                    for variant_names in all_variant_groups {
                        b.gen_and_emit_union(variant_names);
                    }
                    let num_main = b.rng.gen_range(1..=b.config.max_tables);
                    for _ in 0..num_main {
                        b.gen_and_emit_full_table();
                    }
                }
            }
        }

        // root_type -- pick a table in the current (last) namespace so we
        // can use the unqualified name. FlatBuffers root_type resolves names
        // in the current namespace scope.
        if b.config.use_root_type && !b.tables.is_empty() {
            // Prefer a table in the current namespace
            let candidates: Vec<usize> = b.tables.iter().enumerate()
                .filter(|(_, t)| {
                    match &b.current_namespace {
                        Some(ns) => t.qualified_name.starts_with(ns)
                            && t.qualified_name[ns.len()..].starts_with('.'),
                        None => !t.qualified_name.contains('.'),
                    }
                })
                .map(|(i, _)| i)
                .collect();
            let idx = if candidates.is_empty() {
                // Fallback: pick any table (use unqualified name -- only works
                // if the table is in the current namespace; otherwise emit the
                // short name which works if there's no ambiguity)
                b.rng.gen_range(0..b.tables.len())
            } else {
                candidates[b.rng.gen_range(0..candidates.len())]
            };
            let root = &b.tables[idx].name;
            writeln!(b.output, "root_type {root};").unwrap();
        }

        // file_identifier
        if b.config.use_file_ident && b.rng.gen_bool(b.config.prob_file_ident) {
            let ident: String = (0..4)
                .map(|_| b.rng.gen_range(b'A'..=b'Z') as char)
                .collect();
            writeln!(b.output, "file_identifier \"{ident}\";").unwrap();
        }

        // file_extension
        if b.rng.gen_bool(b.config.prob_file_extension) {
            let exts = ["bin", "dat", "fb", "fbs", "msg"];
            let ext = exts[b.rng.gen_range(0..exts.len())];
            writeln!(b.output, "file_extension \"{ext}\";").unwrap();
        }

        // rpc_service
        if !b.tables.is_empty() && b.rng.gen_bool(b.config.prob_rpc_service) {
            b.gen_and_emit_rpc_service();
        }

        b.output
    }

    /// Generate a flat section (no namespace switching): enums, structs,
    /// variant tables, unions, main tables.
    fn gen_section_types(&mut self) {
        // enums
        let num_enums = self.rng.gen_range(0..=self.config.max_enums);
        for _ in 0..num_enums {
            self.gen_and_emit_enum();
        }

        // structs
        let num_structs = self.rng.gen_range(0..=self.config.max_structs);
        for _ in 0..num_structs {
            self.gen_and_emit_struct();
        }

        // variant tables + unions
        let num_unions = self.rng.gen_range(0..=self.config.max_unions);
        let mut all_variant_groups: Vec<Vec<String>> = Vec::new();
        for _ in 0..num_unions {
            let num_variants = self.rng.gen_range(2..=self.config.max_union_variants);
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
        let num_main = self.rng.gen_range(1..=self.config.max_tables);
        for _ in 0..num_main {
            self.gen_and_emit_full_table();
        }
    }

    // -- Name generators --

    fn fresh_name(&mut self, prefix: &str) -> String {
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

    fn gen_namespace_name(&mut self) -> String {
        let depth = self.rng.gen_range(1..=2);
        let mut ns = String::new();
        let mut used_parts = HashSet::new();
        for i in 0..depth {
            if i > 0 {
                ns.push('.');
            }
            // Pick a part that hasn't been used at this level (avoid "Game.Game")
            let p = loop {
                let candidate = NAMESPACE_PARTS[self.rng.gen_range(0..NAMESPACE_PARTS.len())];
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
    fn qualified_name(&self, short_name: &str) -> String {
        match &self.current_namespace {
            Some(ns) => format!("{ns}.{short_name}"),
            None => short_name.to_string(),
        }
    }

    /// Return the name to use when referencing a type from the current namespace.
    /// If the type is in the same namespace (or there are no namespaces), use the
    /// short name. If the type is in a different namespace, use the qualified name.
    fn reference_name(&self, short_name: &str, qualified_name: &str) -> String {
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

    fn field_name(&self, idx: usize) -> String {
        if idx < 26 {
            format!("field_{}", (b'a' + idx as u8) as char)
        } else {
            format!("field_{idx}")
        }
    }

    // -- Enum generation --

    fn gen_and_emit_enum(&mut self) {
        let name = self.fresh_name("Enum");
        let underlying_idx = self.rng.gen_range(0..SCALAR_INT_TYPES.len());
        let underlying = SCALAR_INT_TYPES[underlying_idx].to_string();
        let underlying_display = self.maybe_alias_scalar(&underlying);

        let is_bit_flags = self.rng.gen_bool(self.config.prob_bit_flags);
        let num_values = if is_bit_flags {
            // bit_flags enums typically have fewer values to avoid overflow
            self.rng.gen_range(2..=6.min(self.config.max_enum_values))
        } else {
            self.rng.gen_range(2..=self.config.max_enum_values)
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
        if is_bit_flags {
            // bit_flags: values are bit positions (0, 1, 2, ...)
            let vals: Vec<String> = value_names
                .iter()
                .enumerate()
                .map(|(i, vn)| format!("{vn} = {i}"))
                .collect();
            writeln!(self.output, "enum {name} : {underlying_display}{meta} {{").unwrap();
            writeln!(self.output, "  {}", vals.join(",\n  ")).unwrap();
        } else {
            let vals: Vec<String> = value_names
                .iter()
                .enumerate()
                .map(|(i, vn)| format!("{vn} = {i}"))
                .collect();
            writeln!(self.output, "enum {name} : {underlying_display}{meta} {{").unwrap();
            writeln!(self.output, "  {}", vals.join(",\n  ")).unwrap();
        }
        writeln!(self.output, "}}\n").unwrap();

        let qualified_name = self.qualified_name(&name);
        self.enums.push(EnumInfo {
            name,
            qualified_name,
            underlying,
            value_names,
        });
    }

    // -- Struct generation --

    fn gen_and_emit_struct(&mut self) {
        let name = self.fresh_name("Struct");
        let num_fields = self.rng.gen_range(1..=self.config.max_fields_per_type);
        let struct_index = self.structs.len();

        let mut lines = Vec::new();
        for i in 0..num_fields {
            let fname = self.field_name(i);
            let ftype = self.random_struct_field_type(struct_index);
            lines.push(format!("  {fname}: {ftype};"));
        }

        self.maybe_emit_doc_comment(&format!("Struct {name}."));
        let force_align = if self.rng.gen_bool(self.config.prob_force_align) {
            let alignments = [2, 4, 8, 16];
            let align = alignments[self.rng.gen_range(0..alignments.len())];
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
    fn random_struct_field_type(&mut self, struct_index: usize) -> String {
        let mut pool: Vec<String> = ALL_SCALAR_TYPES.iter().map(|s| s.to_string()).collect();
        for e in &self.enums {
            pool.push(self.reference_name(&e.name, &e.qualified_name));
        }
        for s in &self.structs[..struct_index] {
            pool.push(self.reference_name(&s.name, &s.qualified_name));
        }

        let base = pool[self.rng.gen_range(0..pool.len())].clone();

        if self.config.max_fixed_array_len > 0
            && self.rng.gen_bool(self.config.prob_fixed_array)
        {
            let len = self.rng.gen_range(1..=self.config.max_fixed_array_len);
            format!("[{base}:{len}]")
        } else {
            base
        }
    }

    // -- Variant table (simple table for union members) --

    fn gen_and_emit_variant_table(&mut self) -> String {
        let name = self.fresh_name("Table");
        let num_fields = self.rng.gen_range(1..=3);

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
    fn random_simple_table_field_type(&mut self) -> String {
        let mut pool: Vec<String> = ALL_SCALAR_TYPES.iter().map(|s| s.to_string()).collect();
        pool.push("string".to_string());
        for e in &self.enums {
            pool.push(self.reference_name(&e.name, &e.qualified_name));
        }
        for s in &self.structs {
            pool.push(self.reference_name(&s.name, &s.qualified_name));
        }
        pool[self.rng.gen_range(0..pool.len())].clone()
    }

    // -- Union generation --

    fn gen_and_emit_union(&mut self, variant_tables: Vec<String>) {
        let name = self.fresh_name("Union");

        writeln!(self.output, "union {name} {{").unwrap();
        let variants: Vec<String> = variant_tables.iter().map(|v| format!("  {v}")).collect();
        writeln!(self.output, "{}", variants.join(",\n")).unwrap();
        writeln!(self.output, "}}\n").unwrap();

        let qualified_name = self.qualified_name(&name);
        self.unions.push(UnionInfo {
            name,
            qualified_name,
            variant_tables,
        });
    }

    // -- Full table generation (can reference everything) --

    fn gen_and_emit_full_table(&mut self) {
        let name = self.fresh_name("Table");
        let num_fields = self.rng.gen_range(1..=self.config.max_fields_per_type);

        let use_ids = self.rng.gen_bool(self.config.prob_field_ids);
        let mut next_id: u16 = 0;
        let mut has_key = false;

        let mut lines = Vec::new();
        let mut used_union = false;
        for i in 0..num_fields {
            let fname = self.field_name(i);
            if self.rng.gen_bool(self.config.prob_doc_comment) {
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

            if !is_union && is_ref_type && self.rng.gen_bool(self.config.prob_required) {
                attrs.push("required".to_string());
            }

            if attrs.is_empty() && self.rng.gen_bool(self.config.prob_deprecated) {
                attrs.push("deprecated".to_string());
            }

            let is_scalar = ALL_SCALAR_TYPES.contains(&ftype.as_str());
            if !has_key
                && !is_union
                && attrs.is_empty()
                && (is_scalar || ftype == "string")
                && self.rng.gen_bool(self.config.prob_key)
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
    fn random_full_table_field_type(&mut self, used_union: bool) -> (String, bool) {
        let c = &self.config;
        let w_enum = if self.enums.is_empty() { 0 } else { c.weight_enum };
        let w_struct = if self.structs.is_empty() { 0 } else { c.weight_struct };
        let w_table = if self.tables.is_empty() { 0 } else { c.weight_table_ref };
        let w_union = if used_union || self.unions.is_empty() {
            0
        } else {
            c.weight_union
        };
        let total = c.weight_scalar + c.weight_string + w_enum + w_struct + w_table
            + c.weight_vector + w_union;

        let roll = self.rng.gen_range(0..total);
        let mut cursor = 0;

        cursor += c.weight_scalar;
        if roll < cursor {
            let idx = self.rng.gen_range(0..ALL_SCALAR_TYPES.len());
            return (ALL_SCALAR_TYPES[idx].to_string(), false);
        }

        cursor += c.weight_string;
        if roll < cursor {
            return ("string".to_string(), false);
        }

        cursor += w_enum;
        if roll < cursor {
            let idx = self.rng.gen_range(0..self.enums.len());
            let name = self.enums[idx].name.clone();
            let qname = self.enums[idx].qualified_name.clone();
            return (self.reference_name(&name, &qname), false);
        }

        cursor += w_struct;
        if roll < cursor {
            let idx = self.rng.gen_range(0..self.structs.len());
            let name = self.structs[idx].name.clone();
            let qname = self.structs[idx].qualified_name.clone();
            return (self.reference_name(&name, &qname), false);
        }

        cursor += w_table;
        if roll < cursor {
            let idx = self.rng.gen_range(0..self.tables.len());
            let ref_name = self.table_ref_name(idx);
            return (ref_name, false);
        }

        cursor += c.weight_vector;
        if roll < cursor {
            return (self.random_vector_type(), false);
        }

        // union
        let idx = self.rng.gen_range(0..self.unions.len());
        let name = self.unions[idx].name.clone();
        let qname = self.unions[idx].qualified_name.clone();
        (self.reference_name(&name, &qname), true)
    }

    fn random_vector_type(&mut self) -> String {
        let c = &self.config;
        let w_enum = if self.enums.is_empty() { 0 } else { c.weight_vec_enum };
        let w_struct = if self.structs.is_empty() { 0 } else { c.weight_vec_struct };
        let w_table = if self.tables.is_empty() { 0 } else { c.weight_vec_table };
        let total = c.weight_vec_scalar + c.weight_vec_string + w_enum + w_struct + w_table;

        let roll = self.rng.gen_range(0..total);
        let mut cursor = 0;

        cursor += c.weight_vec_scalar;
        if roll < cursor {
            let idx = self.rng.gen_range(0..ALL_SCALAR_TYPES.len());
            return format!("[{}]", ALL_SCALAR_TYPES[idx]);
        }

        cursor += c.weight_vec_string;
        if roll < cursor {
            return "[string]".to_string();
        }

        cursor += w_enum;
        if roll < cursor {
            let idx = self.rng.gen_range(0..self.enums.len());
            let name = self.enums[idx].name.clone();
            let qname = self.enums[idx].qualified_name.clone();
            return format!("[{}]", self.reference_name(&name, &qname));
        }

        cursor += w_struct;
        if roll < cursor {
            let idx = self.rng.gen_range(0..self.structs.len());
            let name = self.structs[idx].name.clone();
            let qname = self.structs[idx].qualified_name.clone();
            return format!("[{}]", self.reference_name(&name, &qname));
        }

        // table
        let idx = self.rng.gen_range(0..self.tables.len());
        let ref_name = self.table_ref_name(idx);
        format!("[{ref_name}]")
    }

    fn maybe_default_value(&mut self, ftype: &str) -> String {
        if !self.rng.gen_bool(self.config.prob_default_value) {
            return String::new();
        }

        // Optional scalar: `= null`
        let is_scalar_type = ALL_SCALAR_TYPES.contains(&ftype)
            || TYPE_ALIASES.iter().any(|&(_, alias)| alias == ftype);
        if is_scalar_type && self.rng.gen_bool(self.config.prob_null_default) {
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
                if self.rng.gen_bool(0.5) {
                    " = true".to_string()
                } else {
                    " = false".to_string()
                }
            }
            "byte" | "int8" => format!(" = {}", self.rng.gen_range(-128i16..=127) as i8),
            "ubyte" | "uint8" => format!(" = {}", self.rng.gen_range(0u8..=255)),
            "short" | "int16" => format!(" = {}", self.rng.gen_range(-100i16..=100)),
            "ushort" | "uint16" => format!(" = {}", self.rng.gen_range(0u16..=200)),
            "int" | "int32" => format!(" = {}", self.rng.gen_range(-1000i32..=1000)),
            "uint" | "uint32" => format!(" = {}", self.rng.gen_range(0u32..=1000)),
            "long" | "int64" => format!(" = {}", self.rng.gen_range(-1000i64..=1000)),
            "ulong" | "uint64" => format!(" = {}", self.rng.gen_range(0u64..=1000)),
            "float" | "float32" => {
                if self.rng.gen_bool(self.config.prob_nan_inf_default) {
                    let special = ["nan", "inf", "-inf", "+inf", "infinity", "-infinity"];
                    format!(" = {}", special[self.rng.gen_range(0..special.len())])
                } else {
                    let vals = [0.0, 1.0, -1.0, 3.14, 100.0];
                    format!(" = {}", vals[self.rng.gen_range(0..vals.len())])
                }
            }
            "double" | "float64" => {
                if self.rng.gen_bool(self.config.prob_nan_inf_default) {
                    let special = ["nan", "inf", "-inf", "+inf", "infinity", "-infinity"];
                    format!(" = {}", special[self.rng.gen_range(0..special.len())])
                } else {
                    let vals = [0.0, 1.0, -1.0, 2.718, 100.0];
                    format!(" = {}", vals[self.rng.gen_range(0..vals.len())])
                }
            }
            "string" => {
                let strings = ["", "hello", "test", "foo"];
                format!(" = \"{}\"", strings[self.rng.gen_range(0..strings.len())])
            }
            _ => {
                if let Some(e) = self.enums.iter().find(|e| e.name == short_ftype) {
                    let vname =
                        e.value_names[self.rng.gen_range(0..e.value_names.len())].clone();
                    format!(" = {vname}")
                } else {
                    String::new()
                }
            }
        }
    }

    // -- RPC service generation --

    fn gen_and_emit_rpc_service(&mut self) {
        let name = self.fresh_name("Service");
        let num_methods = self.rng.gen_range(1..=self.config.max_rpc_methods);

        if self.rng.gen_bool(self.config.prob_doc_comment) {
            writeln!(self.output, "/// RPC service for {name}.").unwrap();
        }
        writeln!(self.output, "rpc_service {name} {{").unwrap();

        // Collect tables in the current namespace for method types
        let current_ns_tables: Vec<String> = self.tables.iter()
            .filter(|t| {
                match &self.current_namespace {
                    Some(ns) => t.qualified_name.starts_with(ns)
                        && t.qualified_name[ns.len()..].starts_with('.'),
                    None => !t.qualified_name.contains('.'),
                }
            })
            .map(|t| t.name.clone())
            .collect();

        let table_pool: &[String] = if current_ns_tables.is_empty() {
            // Fallback to all tables
            &self.tables.iter().map(|t| t.name.clone()).collect::<Vec<_>>()
        } else {
            &current_ns_tables
        };

        let method_names = ["Get", "Put", "List", "Create", "Delete", "Update", "Query"];
        for i in 0..num_methods {
            let method_name = if i < method_names.len() {
                method_names[i].to_string()
            } else {
                format!("Method{i}")
            };
            let req = &table_pool[self.rng.gen_range(0..table_pool.len())];
            let resp = &table_pool[self.rng.gen_range(0..table_pool.len())];

            let mut attrs = Vec::new();
            let streaming_modes = ["none", "server", "client", "bidi"];
            if self.rng.gen_bool(0.3) {
                let mode = streaming_modes[self.rng.gen_range(0..streaming_modes.len())];
                attrs.push(format!("streaming: \"{mode}\""));
            }
            if self.rng.gen_bool(0.2) {
                attrs.push("idempotent".to_string());
            }

            let attr_str = if attrs.is_empty() {
                String::new()
            } else {
                format!(" ({})", attrs.join(", "))
            };

            if self.rng.gen_bool(self.config.prob_doc_comment) {
                writeln!(self.output, "  /// {method_name} method.").unwrap();
            }
            writeln!(self.output, "  {method_name}({req}):{resp}{attr_str};").unwrap();
        }

        writeln!(self.output, "}}\n").unwrap();
    }

    // -- Doc comment helper --

    fn maybe_emit_doc_comment(&mut self, subject: &str) {
        if self.rng.gen_bool(self.config.prob_doc_comment) {
            writeln!(self.output, "/// {subject}").unwrap();
        }
    }

    // -- Type alias helper --

    fn maybe_alias_scalar(&mut self, scalar: &str) -> String {
        if self.rng.gen_bool(self.config.prob_type_alias) {
            for &(canonical, alias) in TYPE_ALIASES {
                if scalar == canonical {
                    return alias.to_string();
                }
            }
        }
        scalar.to_string()
    }

    // -- Reference helpers --

    /// Get the name to use when referencing a table by index from the current namespace.
    fn table_ref_name(&self, idx: usize) -> String {
        let t = &self.tables[idx];
        self.reference_name(&t.name, &t.qualified_name)
    }

    fn is_table_ref(&self, name: &str) -> bool {
        self.tables.iter().any(|t| t.name == name || t.qualified_name == name)
    }

    fn is_struct_ref(&self, name: &str) -> bool {
        self.structs.iter().any(|s| s.name == name || s.qualified_name == name)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Determinism --

    #[test]
    fn deterministic_output() {
        let a = SchemaBuilder::generate(42, GenConfig::default());
        let b = SchemaBuilder::generate(42, GenConfig::default());
        assert_eq!(a, b, "same seed must produce identical output");
    }

    #[test]
    fn different_seeds_differ() {
        let a = SchemaBuilder::generate(1, GenConfig::default());
        let b = SchemaBuilder::generate(2, GenConfig::default());
        assert_ne!(a, b, "different seeds should produce different output");
    }

    // -- Capacity limits --

    #[test]
    fn enums_only() {
        let config = GenConfig {
            max_enums: 3,
            max_structs: 0,
            max_tables: 1,
            max_unions: 0,
            use_namespace: false,
            use_file_ident: false,
            ..GenConfig::default()
        };
        let out = SchemaBuilder::generate(10, config);
        assert!(out.contains("table "));
        assert!(!out.contains("struct "));
        assert!(!out.contains("union "));
    }

    #[test]
    fn structs_only() {
        let config = GenConfig {
            max_enums: 0,
            max_structs: 3,
            max_tables: 1,
            max_unions: 0,
            use_namespace: false,
            use_file_ident: false,
            ..GenConfig::default()
        };
        let out = SchemaBuilder::generate(10, config);
        assert!(out.contains("table "));
        assert!(!out.contains("enum "));
        assert!(!out.contains("union "));
    }

    #[test]
    fn tables_only() {
        let config = GenConfig {
            max_enums: 0,
            max_structs: 0,
            max_tables: 5,
            max_unions: 0,
            use_namespace: false,
            use_file_ident: false,
            ..GenConfig::default()
        };
        for seed in 0..20 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            assert!(out.contains("table "));
            assert!(!out.contains("enum "));
            assert!(!out.contains("struct "));
            assert!(!out.contains("union "));
        }
    }

    #[test]
    fn unions_require_variant_tables() {
        let config = GenConfig {
            max_enums: 0,
            max_structs: 0,
            max_tables: 3,
            max_unions: 2,
            max_union_variants: 2,
            use_namespace: false,
            use_file_ident: false,
            ..GenConfig::default()
        };
        for seed in 0..20 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            // If a union exists, its variant tables must appear before it
            if let Some(union_pos) = out.find("union ") {
                let union_block = &out[union_pos..];
                let brace_end = union_block.find('}').unwrap();
                let body = &union_block[..brace_end];
                for line in body.lines().skip(1) {
                    let variant = line.trim().trim_end_matches(',');
                    if variant.is_empty() {
                        continue;
                    }
                    let table_decl = format!("table {variant} {{");
                    let table_pos = out.find(&table_decl)
                        .unwrap_or_else(|| panic!(
                            "seed {seed}: union variant '{variant}' has no table definition\n{out}"
                        ));
                    assert!(
                        table_pos < union_pos,
                        "seed {seed}: variant table '{variant}' must appear before its union\n{out}"
                    );
                }
            }
        }
    }

    #[test]
    fn minimal_config_works() {
        let config = GenConfig {
            max_enums: 0,
            max_structs: 0,
            max_tables: 1,
            max_unions: 0,
            max_fields_per_type: 1,
            max_enum_values: 2,
            max_union_variants: 2,
            max_fixed_array_len: 0,
            use_namespace: false,
            use_root_type: true,
            use_file_ident: false,
            ..GenConfig::default()
        };
        let out = SchemaBuilder::generate(0, config);
        assert!(out.contains("table "), "must contain at least one table");
    }

    // -- Namespace --

    #[test]
    fn namespace_always_emitted_at_prob_1() {
        let config = GenConfig {
            use_namespace: true,
            prob_namespace: 1.0,
            prob_multi_namespace: 0.0,
            max_tables: 1,
            max_enums: 0,
            max_structs: 0,
            max_unions: 0,
            use_file_ident: false,
            ..GenConfig::default()
        };
        for seed in 0..20 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            assert!(
                out.contains("namespace "),
                "seed {seed}: prob_namespace=1.0 should always emit namespace\n{out}"
            );
        }
    }

    #[test]
    fn namespace_never_emitted_at_prob_0() {
        let config = GenConfig {
            use_namespace: true,
            prob_namespace: 0.0,
            max_tables: 1,
            max_enums: 0,
            max_structs: 0,
            max_unions: 0,
            use_file_ident: false,
            ..GenConfig::default()
        };
        for seed in 0..20 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            assert!(
                !out.contains("namespace "),
                "seed {seed}: prob_namespace=0.0 should never emit namespace\n{out}"
            );
        }
    }

    #[test]
    fn namespace_disabled_never_emits() {
        let config = GenConfig {
            use_namespace: false,
            prob_namespace: 1.0,
            max_tables: 1,
            max_enums: 0,
            max_structs: 0,
            max_unions: 0,
            use_file_ident: false,
            ..GenConfig::default()
        };
        for seed in 0..10 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            assert!(!out.contains("namespace "), "seed {seed}");
        }
    }

    #[test]
    fn multi_namespace_emits_multiple_sections() {
        let config = GenConfig {
            use_namespace: true,
            prob_namespace: 1.0,
            prob_multi_namespace: 1.0,
            max_namespaces: 3,
            max_tables: 3,
            max_enums: 2,
            max_structs: 1,
            max_unions: 0,
            use_file_ident: false,
            ..GenConfig::default()
        };
        for seed in 0..20 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            let ns_count = out.matches("namespace ").count();
            assert!(
                ns_count >= 2,
                "seed {seed}: expected 2+ namespace sections, got {ns_count}\n{out}"
            );
        }
    }

    #[test]
    fn multi_namespace_single_when_prob_0() {
        let config = GenConfig {
            use_namespace: true,
            prob_namespace: 1.0,
            prob_multi_namespace: 0.0,
            max_namespaces: 3,
            max_tables: 1,
            max_enums: 0,
            max_structs: 0,
            max_unions: 0,
            use_file_ident: false,
            ..GenConfig::default()
        };
        for seed in 0..20 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            let ns_count = out.matches("namespace ").count();
            assert_eq!(
                ns_count, 1,
                "seed {seed}: prob_multi_namespace=0 should emit exactly 1 namespace\n{out}"
            );
        }
    }

    #[test]
    fn multi_namespace_has_cross_references() {
        // Force multi-namespace with table refs to exercise cross-ns references
        let config = GenConfig {
            use_namespace: true,
            prob_namespace: 1.0,
            prob_multi_namespace: 1.0,
            max_namespaces: 2,
            max_tables: 4,
            max_enums: 1,
            max_structs: 0,
            max_unions: 0,
            max_fields_per_type: 4,
            weight_scalar: 5,
            weight_string: 0,
            weight_enum: 5,
            weight_struct: 0,
            weight_table_ref: 40,
            weight_vector: 20,
            weight_union: 0,
            use_file_ident: false,
            prob_required: 0.0,
            prob_deprecated: 0.0,
            prob_key: 0.0,
            prob_default_value: 0.0,
            prob_field_ids: 0.0,
            ..GenConfig::default()
        };
        let mut found_cross_ref = false;
        for seed in 0..50 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            // A cross-namespace reference contains a dot in a field type
            for line in out.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("field_") && trimmed.contains(": ") {
                    let colon = trimmed.find(':').unwrap();
                    let semi = trimmed.find(';').unwrap();
                    let ftype = trimmed[colon + 1..semi].trim();
                    // Strip vector brackets
                    let inner = ftype.trim_start_matches('[').trim_end_matches(']');
                    if inner.contains('.') {
                        found_cross_ref = true;
                    }
                }
            }
        }
        assert!(
            found_cross_ref,
            "expected at least one cross-namespace reference across 50 seeds"
        );
    }

    // -- File identifier --

    #[test]
    fn file_ident_always_at_prob_1() {
        let config = GenConfig {
            use_file_ident: true,
            prob_file_ident: 1.0,
            max_tables: 1,
            max_enums: 0,
            max_structs: 0,
            max_unions: 0,
            use_namespace: false,
            ..GenConfig::default()
        };
        for seed in 0..20 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            assert!(
                out.contains("file_identifier "),
                "seed {seed}: prob_file_ident=1.0 should always emit\n{out}"
            );
        }
    }

    #[test]
    fn file_ident_never_at_prob_0() {
        let config = GenConfig {
            use_file_ident: true,
            prob_file_ident: 0.0,
            max_tables: 1,
            max_enums: 0,
            max_structs: 0,
            max_unions: 0,
            use_namespace: false,
            ..GenConfig::default()
        };
        for seed in 0..20 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            assert!(!out.contains("file_identifier "), "seed {seed}");
        }
    }

    #[test]
    fn file_ident_is_4_chars() {
        let config = GenConfig {
            use_file_ident: true,
            prob_file_ident: 1.0,
            max_tables: 1,
            max_enums: 0,
            max_structs: 0,
            max_unions: 0,
            use_namespace: false,
            ..GenConfig::default()
        };
        for seed in 0..20 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            let start = out.find("file_identifier \"").unwrap() + "file_identifier \"".len();
            let end = out[start..].find('"').unwrap();
            assert_eq!(end, 4, "seed {seed}: file_identifier must be 4 chars");
        }
    }

    // -- Field IDs --

    #[test]
    fn field_ids_always_at_prob_1() {
        let config = GenConfig {
            prob_field_ids: 1.0,
            max_tables: 3,
            max_enums: 0,
            max_structs: 0,
            max_unions: 0,
            use_namespace: false,
            use_file_ident: false,
            prob_required: 0.0,
            prob_deprecated: 0.0,
            prob_key: 0.0,
            prob_default_value: 0.0,
            prob_rpc_service: 0.0,
            ..GenConfig::default()
        };
        for seed in 0..20 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            for line in out.lines() {
                let trimmed = line.trim();
                if trimmed.ends_with(';') && trimmed.contains(':') && !trimmed.starts_with("//") {
                    assert!(
                        trimmed.contains("id: "),
                        "seed {seed}: field should have id when prob=1.0: {trimmed}\n{out}"
                    );
                }
            }
        }
    }

    #[test]
    fn field_ids_never_at_prob_0() {
        let config = GenConfig {
            prob_field_ids: 0.0,
            max_tables: 3,
            max_enums: 0,
            max_structs: 0,
            max_unions: 0,
            use_namespace: false,
            use_file_ident: false,
            ..GenConfig::default()
        };
        for seed in 0..20 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            assert!(!out.contains("id: "), "seed {seed}: no field IDs expected\n{out}");
        }
    }

    // -- Attributes: required, deprecated, key --

    #[test]
    fn required_never_at_prob_0() {
        let config = GenConfig {
            prob_required: 0.0,
            max_tables: 3,
            max_enums: 0,
            max_structs: 0,
            max_unions: 0,
            use_namespace: false,
            use_file_ident: false,
            ..GenConfig::default()
        };
        for seed in 0..30 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            assert!(!out.contains("required"), "seed {seed}\n{out}");
        }
    }

    #[test]
    fn deprecated_never_at_prob_0() {
        let config = GenConfig {
            prob_deprecated: 0.0,
            prob_required: 0.0,
            max_tables: 3,
            max_enums: 0,
            max_structs: 0,
            max_unions: 0,
            use_namespace: false,
            use_file_ident: false,
            ..GenConfig::default()
        };
        for seed in 0..30 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            assert!(!out.contains("deprecated"), "seed {seed}\n{out}");
        }
    }

    #[test]
    fn key_never_at_prob_0() {
        let config = GenConfig {
            prob_key: 0.0,
            max_tables: 3,
            max_enums: 0,
            max_structs: 0,
            max_unions: 0,
            use_namespace: false,
            use_file_ident: false,
            ..GenConfig::default()
        };
        for seed in 0..30 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            assert!(!out.contains("key"), "seed {seed}\n{out}");
        }
    }

    // -- Default values --

    #[test]
    fn defaults_never_at_prob_0() {
        let config = GenConfig {
            prob_default_value: 0.0,
            max_tables: 3,
            max_enums: 2,
            max_structs: 0,
            max_unions: 0,
            use_namespace: false,
            use_file_ident: false,
            prob_required: 0.0,
            prob_deprecated: 0.0,
            prob_key: 0.0,
            ..GenConfig::default()
        };
        for seed in 0..30 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            let mut in_table = false;
            for line in out.lines() {
                if line.starts_with("table ") {
                    in_table = true;
                } else if line == "}" {
                    in_table = false;
                } else if in_table && line.trim().ends_with(';') {
                    assert!(
                        !line.contains(" = "),
                        "seed {seed}: no defaults expected: {line}\n{out}"
                    );
                }
            }
        }
    }

    // -- Fixed arrays --

    #[test]
    fn fixed_arrays_never_at_prob_0() {
        let config = GenConfig {
            prob_fixed_array: 0.0,
            max_structs: 3,
            max_enums: 0,
            max_tables: 1,
            max_unions: 0,
            max_fields_per_type: 6,
            use_namespace: false,
            use_file_ident: false,
            ..GenConfig::default()
        };
        for seed in 0..30 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            let mut in_struct = false;
            for line in out.lines() {
                if line.starts_with("struct ") {
                    in_struct = true;
                } else if line == "}" {
                    in_struct = false;
                } else if in_struct && line.trim().ends_with(';') {
                    assert!(
                        !line.contains(":]"),
                        "seed {seed}: no fixed arrays expected: {line}\n{out}"
                    );
                }
            }
        }
    }

    #[test]
    fn fixed_arrays_disabled_when_max_len_0() {
        let config = GenConfig {
            prob_fixed_array: 1.0,
            max_fixed_array_len: 0,
            max_structs: 3,
            max_enums: 0,
            max_tables: 1,
            max_unions: 0,
            use_namespace: false,
            use_file_ident: false,
            ..GenConfig::default()
        };
        for seed in 0..20 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            let mut in_struct = false;
            for line in out.lines() {
                if line.starts_with("struct ") {
                    in_struct = true;
                } else if line == "}" {
                    in_struct = false;
                } else if in_struct && line.trim().ends_with(';') {
                    assert!(
                        !line.contains(":]"),
                        "seed {seed}: max_len=0 should disable arrays: {line}\n{out}"
                    );
                }
            }
        }
    }

    // -- Table field type weights --

    #[test]
    fn weight_only_scalars() {
        let config = GenConfig {
            weight_scalar: 100,
            weight_string: 0,
            weight_enum: 0,
            weight_struct: 0,
            weight_table_ref: 0,
            weight_vector: 0,
            weight_union: 0,
            max_enums: 2,
            max_structs: 2,
            max_tables: 3,
            max_unions: 0,
            max_fields_per_type: 4,
            use_namespace: false,
            use_file_ident: false,
            prob_required: 0.0,
            prob_deprecated: 0.0,
            prob_key: 0.0,
            prob_default_value: 0.0,
            prob_field_ids: 0.0,
            prob_type_alias: 0.0,
            ..GenConfig::default()
        };
        for seed in 0..20 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            let mut in_main_table = false;
            let mut table_count = 0;
            for line in out.lines() {
                if line.starts_with("table ") {
                    table_count += 1;
                    in_main_table = true;
                } else if line == "}" {
                    in_main_table = false;
                } else if in_main_table && line.trim().ends_with(';') {
                    let field_line = line.trim();
                    let colon = field_line.find(':').unwrap();
                    let semi = field_line.find(';').unwrap();
                    let ftype = field_line[colon + 1..semi].trim();
                    assert!(
                        ALL_SCALAR_TYPES.contains(&ftype),
                        "seed {seed}: expected scalar, got '{ftype}'\n{out}"
                    );
                }
            }
            assert!(table_count > 0, "seed {seed}: should have tables");
        }
    }

    #[test]
    fn weight_only_strings() {
        let config = GenConfig {
            weight_scalar: 0,
            weight_string: 100,
            weight_enum: 0,
            weight_struct: 0,
            weight_table_ref: 0,
            weight_vector: 0,
            weight_union: 0,
            max_enums: 0,
            max_structs: 0,
            max_tables: 2,
            max_unions: 0,
            max_fields_per_type: 3,
            use_namespace: false,
            use_file_ident: false,
            prob_required: 0.0,
            prob_deprecated: 0.0,
            prob_key: 0.0,
            prob_default_value: 0.0,
            prob_field_ids: 0.0,
            ..GenConfig::default()
        };
        for seed in 0..20 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            let mut in_table = false;
            for line in out.lines() {
                if line.starts_with("table ") {
                    in_table = true;
                } else if line == "}" {
                    in_table = false;
                } else if in_table && line.trim().ends_with(';') {
                    assert!(
                        line.contains(": string;"),
                        "seed {seed}: expected string field, got: {line}\n{out}"
                    );
                }
            }
        }
    }

    #[test]
    fn weight_only_vectors() {
        let config = GenConfig {
            weight_scalar: 0,
            weight_string: 0,
            weight_enum: 0,
            weight_struct: 0,
            weight_table_ref: 0,
            weight_vector: 100,
            weight_union: 0,
            max_enums: 0,
            max_structs: 0,
            max_tables: 2,
            max_unions: 0,
            max_fields_per_type: 3,
            use_namespace: false,
            use_file_ident: false,
            prob_required: 0.0,
            prob_deprecated: 0.0,
            prob_key: 0.0,
            prob_default_value: 0.0,
            prob_field_ids: 0.0,
            ..GenConfig::default()
        };
        for seed in 0..20 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            let mut in_table = false;
            for line in out.lines() {
                if line.starts_with("table ") {
                    in_table = true;
                } else if line == "}" {
                    in_table = false;
                } else if in_table && line.trim().ends_with(';') {
                    assert!(
                        line.contains('[') && line.contains(']'),
                        "seed {seed}: expected vector field, got: {line}\n{out}"
                    );
                }
            }
        }
    }

    // -- Root type --

    #[test]
    fn root_type_emitted_when_enabled() {
        let config = GenConfig {
            use_root_type: true,
            max_tables: 2,
            max_enums: 0,
            max_structs: 0,
            max_unions: 0,
            use_namespace: false,
            use_file_ident: false,
            ..GenConfig::default()
        };
        for seed in 0..20 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            assert!(out.contains("root_type "), "seed {seed}\n{out}");
        }
    }

    #[test]
    fn root_type_not_emitted_when_disabled() {
        let config = GenConfig {
            use_root_type: false,
            max_tables: 2,
            max_enums: 0,
            max_structs: 0,
            max_unions: 0,
            use_namespace: false,
            use_file_ident: false,
            ..GenConfig::default()
        };
        for seed in 0..20 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            assert!(!out.contains("root_type "), "seed {seed}");
        }
    }

    // -- Name uniqueness --

    #[test]
    fn no_duplicate_type_names() {
        let config = GenConfig {
            max_enums: 4,
            max_structs: 3,
            max_tables: 5,
            max_unions: 2,
            ..GenConfig::default()
        };
        for seed in 0..50 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            let mut names: Vec<String> = Vec::new();
            for line in out.lines() {
                for prefix in &["enum ", "struct ", "table ", "union "] {
                    if line.starts_with(prefix) {
                        let rest = &line[prefix.len()..];
                        let name = rest.split(|c: char| !c.is_alphanumeric())
                            .next()
                            .unwrap_or("");
                        assert!(
                            !names.contains(&name.to_string()),
                            "seed {seed}: duplicate type name '{name}'\n{out}"
                        );
                        names.push(name.to_string());
                    }
                }
            }
        }
    }

    // -- Struct field validity --

    #[test]
    fn struct_fields_are_valid_types() {
        let config = GenConfig {
            max_enums: 2,
            max_structs: 3,
            max_tables: 1,
            max_unions: 0,
            max_fields_per_type: 4,
            use_namespace: false,
            use_file_ident: false,
            ..GenConfig::default()
        };
        for seed in 0..30 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            let mut known_types: HashSet<String> =
                ALL_SCALAR_TYPES.iter().map(|s| s.to_string()).collect();
            let mut in_struct = false;
            for line in out.lines() {
                if line.starts_with("enum ") {
                    let name = line.split_whitespace().nth(1).unwrap();
                    known_types.insert(name.to_string());
                } else if line.starts_with("struct ") {
                    let name = line.split_whitespace().nth(1).unwrap();
                    in_struct = true;
                    known_types.insert(name.to_string());
                } else if line == "}" {
                    in_struct = false;
                } else if in_struct && line.trim().ends_with(';') {
                    let trimmed = line.trim();
                    let colon = trimmed.find(':').unwrap();
                    let semi = trimmed.find(';').unwrap();
                    let ftype = trimmed[colon + 1..semi].trim();
                    let base_type = if ftype.starts_with('[') && ftype.contains(':') {
                        let inner = &ftype[1..ftype.len() - 1];
                        inner.split(':').next().unwrap()
                    } else {
                        ftype
                    };
                    assert!(
                        known_types.contains(base_type),
                        "seed {seed}: struct field type '{base_type}' not in known types\n{out}"
                    );
                }
            }
        }
    }

    // -- Output non-empty --

    #[test]
    fn output_is_nonempty() {
        for seed in 0..20 {
            let out = SchemaBuilder::generate(seed, GenConfig::default());
            assert!(!out.is_empty(), "seed {seed} produced empty output");
        }
    }

    // -- Feature coverage: table refs and vec-of-tables --

    #[test]
    fn generates_table_references() {
        let config = GenConfig::default();
        let mut found = false;
        for seed in 0..100 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            for line in out.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("field_") && trimmed.contains(": ") {
                    let colon = trimmed.find(':').unwrap();
                    let semi = trimmed.find(';').unwrap_or(trimmed.len());
                    let ftype = trimmed[colon + 1..semi].trim()
                        .split_whitespace().next().unwrap_or("");
                    // Check for direct table ref (not vector, not union)
                    if ftype.contains("Table") && !ftype.starts_with('[') {
                        found = true;
                    }
                }
            }
        }
        assert!(found, "expected at least one table reference across 100 schemas");
    }

    #[test]
    fn generates_vec_of_tables() {
        let config = GenConfig::default();
        let mut found = false;
        for seed in 0..100 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            for line in out.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("field_") && trimmed.contains("[") {
                    let bracket_start = trimmed.find('[').unwrap();
                    let bracket_end = trimmed.find(']').unwrap();
                    let inner = &trimmed[bracket_start + 1..bracket_end];
                    if inner.contains("Table") {
                        found = true;
                    }
                }
            }
        }
        assert!(found, "expected at least one [Table] vector across 100 schemas");
    }

    #[test]
    fn generates_namespaces() {
        let config = GenConfig::default();
        let mut found = false;
        for seed in 0..100 {
            let out = SchemaBuilder::generate(seed, config.clone_with_seed_safe());
            if out.contains("namespace ") {
                found = true;
            }
        }
        assert!(found, "expected at least one schema with namespace across 100 schemas");
    }

    // -- Helper: clone config for use across seeds --

    impl GenConfig {
        fn clone_with_seed_safe(&self) -> GenConfig {
            GenConfig {
                max_enums: self.max_enums,
                max_structs: self.max_structs,
                max_tables: self.max_tables,
                max_unions: self.max_unions,
                max_fields_per_type: self.max_fields_per_type,
                max_enum_values: self.max_enum_values,
                max_union_variants: self.max_union_variants,
                max_fixed_array_len: self.max_fixed_array_len,
                use_namespace: self.use_namespace,
                use_root_type: self.use_root_type,
                use_file_ident: self.use_file_ident,
                max_namespaces: self.max_namespaces,
                prob_multi_namespace: self.prob_multi_namespace,
                prob_namespace: self.prob_namespace,
                prob_file_ident: self.prob_file_ident,
                prob_fixed_array: self.prob_fixed_array,
                prob_field_ids: self.prob_field_ids,
                prob_required: self.prob_required,
                prob_deprecated: self.prob_deprecated,
                prob_key: self.prob_key,
                prob_default_value: self.prob_default_value,
                weight_scalar: self.weight_scalar,
                weight_string: self.weight_string,
                weight_enum: self.weight_enum,
                weight_struct: self.weight_struct,
                weight_table_ref: self.weight_table_ref,
                weight_vector: self.weight_vector,
                weight_union: self.weight_union,
                weight_vec_scalar: self.weight_vec_scalar,
                weight_vec_string: self.weight_vec_string,
                weight_vec_table: self.weight_vec_table,
                weight_vec_struct: self.weight_vec_struct,
                weight_vec_enum: self.weight_vec_enum,

                prob_file_extension: self.prob_file_extension,
                prob_doc_comment: self.prob_doc_comment,
                prob_rpc_service: self.prob_rpc_service,
                max_rpc_methods: self.max_rpc_methods,
                prob_force_align: self.prob_force_align,
                prob_bit_flags: self.prob_bit_flags,
                prob_type_alias: self.prob_type_alias,
                prob_null_default: self.prob_null_default,
                prob_nan_inf_default: self.prob_nan_inf_default,
            }
        }
    }
}
