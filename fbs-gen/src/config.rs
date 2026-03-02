/// Every random decision in the generator is controlled by this config.
/// All `prob_*` fields are probabilities in `[0.0, 1.0]`.
#[derive(Clone)]
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
