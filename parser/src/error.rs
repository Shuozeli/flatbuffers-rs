use std::fmt;

/// Tree-sitter field name used to locate child nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldName {
    ArrayType,
    ArrayTypeFixedLength,
    AttributeName,
    Documentation,
    EnumIntConstant,
    EnumKey,
    EnumName,
    EnumType,
    EnumValDecl,
    FieldAndValue,
    FieldDeclaration,
    FieldKey,
    FieldType,
    FieldValue,
    FieldWithoutType,
    FileExtensionConstant,
    FileIdentifierConstant,
    FullIdent,
    IncludeName,
    Metadata,
    NamespaceIdent,
    RootTypeIdent,
    RpcMethod,
    RpcMethodName,
    RpcName,
    RpcParameter,
    RpcReturnType,
    ScalarValue,
    SingleValue,
    StringConstant,
    TableOrStructDeclaration,
    TableOrStructName,
    UnionFieldDecl,
    UnionFieldKey,
    UnionFieldValue,
    UnionName,
}

impl FieldName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ArrayType => "array_type",
            Self::ArrayTypeFixedLength => "array_type_fixed_length",
            Self::AttributeName => "attribute_name",
            Self::Documentation => "documentation",
            Self::EnumIntConstant => "enum_int_constant",
            Self::EnumKey => "enum_key",
            Self::EnumName => "enum_name",
            Self::EnumType => "enum_type",
            Self::EnumValDecl => "enum_val_decl",
            Self::FieldAndValue => "field_and_value",
            Self::FieldDeclaration => "field_declaration",
            Self::FieldKey => "field_key",
            Self::FieldType => "field_type",
            Self::FieldValue => "field_value",
            Self::FieldWithoutType => "field_without_type",
            Self::FileExtensionConstant => "file_extension_constant",
            Self::FileIdentifierConstant => "file_identifier_constant",
            Self::FullIdent => "full_ident",
            Self::IncludeName => "include_name",
            Self::Metadata => "metadata",
            Self::NamespaceIdent => "namespace_ident",
            Self::RootTypeIdent => "root_type_ident",
            Self::RpcMethod => "rpc_method",
            Self::RpcMethodName => "rpc_method_name",
            Self::RpcName => "rpc_name",
            Self::RpcParameter => "rpc_parameter",
            Self::RpcReturnType => "rpc_return_type",
            Self::ScalarValue => "scalar_value",
            Self::SingleValue => "single_value",
            Self::StringConstant => "string_constant",
            Self::TableOrStructDeclaration => "table_or_struct_declaration",
            Self::TableOrStructName => "table_or_struct_name",
            Self::UnionFieldDecl => "union_field_decl",
            Self::UnionFieldKey => "union_field_key",
            Self::UnionFieldValue => "union_field_value",
            Self::UnionName => "union_name",
        }
    }
}

impl AsRef<[u8]> for FieldName {
    fn as_ref(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
}

impl fmt::Display for FieldName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("tree-sitter error: {0}")]
    TreeSitter(String),

    #[error("invalid grammar: root node is not source_file")]
    InvalidGrammar,

    #[error("unknown node type: {0}")]
    UnknownNodeType(String),

    #[error("expected field '{0}'")]
    MissingField(FieldName),

    #[error("invalid UTF-8 in source at byte offset {0}")]
    InvalidUtf8(usize),

    #[error("invalid string literal")]
    InvalidString,

    #[error("unrecognized base type: {0}")]
    UnknownBaseType(String),

    #[error("invalid integer literal: {value} ({reason})")]
    InvalidInteger { value: String, reason: String },

    #[error("invalid float literal: {value} ({reason})")]
    InvalidFloat { value: String, reason: String },

    #[error("unexpected content '{found}': {context}")]
    UnexpectedContent { found: String, context: String },

    #[error("invalid escape sequence: {0}")]
    InvalidEscape(String),

    #[error("syntax error at line {line}, column {column}: {context}")]
    SyntaxError {
        line: usize,
        column: usize,
        context: String,
    },
}

pub type Result<T> = std::result::Result<T, ParseError>;
