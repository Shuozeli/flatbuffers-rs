pub(crate) const SCALAR_INT_TYPES: &[&str] = &[
    "byte", "ubyte", "short", "ushort", "int", "uint", "long", "ulong",
];

pub(crate) const ALL_SCALAR_TYPES: &[&str] = &[
    "bool", "byte", "ubyte", "short", "ushort", "int", "uint", "long", "ulong", "float", "double",
];

/// Type alias pairs: (canonical, alias). Used to randomly substitute aliases.
pub(crate) const TYPE_ALIASES: &[(&str, &str)] = &[
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

pub(crate) struct EnumInfo {
    pub name: String,
    /// Fully-qualified name including namespace (e.g., "Game.Items.Color").
    pub qualified_name: String,
    pub value_names: Vec<String>,
}

pub(crate) struct StructInfo {
    pub name: String,
    pub qualified_name: String,
}

pub(crate) struct TableInfo {
    pub name: String,
    pub qualified_name: String,
}

pub(crate) struct UnionInfo {
    pub name: String,
    pub qualified_name: String,
}
