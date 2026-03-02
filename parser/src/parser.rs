use crate::base_type::{base_type_size, lookup_base_type};
use crate::error::{FieldName, ParseError, Result};
use flatc_rs_schema::{self as schema, BaseType};

use tree_sitter::{Node, Parser as TsParser};

// ---------------------------------------------------------------------------
// Parser output: parsed schema + side-channel state
// ---------------------------------------------------------------------------

/// State accumulated during parsing that is not part of the Schema
/// itself but is needed by downstream passes (semantic analysis).
#[derive(Debug, Default, Clone)]
pub struct ParserState {
    pub root_type_name: Option<String>,
    /// The namespace that was active when `root_type` was declared.
    pub root_type_namespace: Option<String>,
    pub root_type_span: Option<schema::Span>,
    pub file_ident_span: Option<schema::Span>,
    pub declared_attributes: Vec<String>,
}

/// Result of parsing a single `.fbs` file.
pub struct ParseOutput {
    pub schema: schema::Schema,
    pub state: ParserState,
}

// ---------------------------------------------------------------------------
// Node helpers
// ---------------------------------------------------------------------------

/// Get a required child by tree-sitter field name.
fn required_child<'a>(node: &Node<'a>, field: FieldName) -> Result<Node<'a>> {
    node.child_by_field_name(field)
        .ok_or(ParseError::MissingField(field))
}

/// Iterator over all children matching a field name.
fn children_by_field<'a>(
    node: &Node<'a>,
    field: FieldName,
) -> impl Iterator<Item = Node<'a>> + 'a {
    let mut cursor = node.walk();
    let field_str = field.as_str();
    node.children_by_field_name(field_str, &mut cursor)
        .collect::<Vec<_>>()
        .into_iter()
}

// ---------------------------------------------------------------------------
// FbsParser
// ---------------------------------------------------------------------------

pub struct FbsParser<'src> {
    source: &'src str,
    file_name: Option<String>,
    ts_parser: TsParser,
    schema: schema::Schema,
    state: ParserState,
    namespace: Option<schema::Namespace>,
}

impl<'src> FbsParser<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            source,
            file_name: None,
            ts_parser: TsParser::new(),
            schema: schema::Schema::new(),
            state: ParserState::default(),
            namespace: None,
        }
    }

    pub fn with_file_name(mut self, file_name: String) -> Self {
        self.file_name = Some(file_name);
        self
    }

    /// Parse the source and return the schema + parser state.
    pub fn parse(mut self) -> Result<ParseOutput> {
        self.ts_parser
            .set_language(&flatc_rs_grammar::LANGUAGE.into())
            .map_err(|e| ParseError::TreeSitter(e.to_string()))?;

        let tree = self
            .ts_parser
            .parse(self.source, None)
            .ok_or_else(|| ParseError::TreeSitter("parse returned None".into()))?;

        let root = tree.root_node();
        if root.kind() != "source_file" {
            return Err(ParseError::InvalidGrammar);
        }

        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            self.visit_top_level(&child)?;
        }

        Ok(ParseOutput {
            schema: self.schema,
            state: self.state,
        })
    }

    // -----------------------------------------------------------------------
    // Text extraction helpers
    // -----------------------------------------------------------------------

    fn get_span(&self, node: &Node) -> schema::Span {
        let start = node.start_position();
        schema::Span::new(self.file_name.clone(), start.row as u32 + 1, start.column as u32 + 1)
    }

    fn text(&self, node: &Node) -> Result<&'src str> {
        node.utf8_text(self.source.as_bytes())
            .map_err(|_| ParseError::InvalidUtf8(node.start_byte()))
    }

    /// Extract a quoted string's inner content (strip surrounding quotes).
    fn string_content(&self, node: &Node) -> Result<String> {
        let raw = self.text(node)?;
        if raw.len() >= 2 {
            if raw.starts_with('"') && raw.ends_with('"') {
                return Ok(unescape_string(&raw[1..raw.len() - 1])?);
            }
            if raw.starts_with('\'') && raw.ends_with('\'') {
                return Ok(unescape_string(&raw[1..raw.len() - 1])?);
            }
        }
        Ok(raw.to_string())
    }

    /// Extract documentation lines from all `documentation` field children.
    fn documentation(&self, node: &Node) -> Result<Option<schema::Documentation>> {
        let lines: Vec<String> = children_by_field(node, FieldName::Documentation)
            .map(|doc_node| {
                let text = self.text(&doc_node)?;
                // Strip leading "///" prefix
                Ok(text.strip_prefix("///").unwrap_or(text).to_string())
            })
            .collect::<Result<Vec<_>>>()?;

        if lines.is_empty() {
            Ok(None)
        } else {
            let mut doc = schema::Documentation::new();
            doc.lines = lines;
            Ok(Some(doc))
        }
    }

    /// Parse metadata (key-value attributes) from a node's `metadata` child.
    fn attributes(&self, node: &Node) -> Result<Option<schema::Attributes>> {
        let metadata_node = match node.child_by_field_name(FieldName::Metadata) {
            Some(n) => n,
            None => return Ok(None),
        };

        let mut attrs = schema::Attributes::new();
        for fav_node in children_by_field(&metadata_node, FieldName::FieldAndValue) {
            let key_node = required_child(&fav_node, FieldName::FieldKey)?;
            let key = self.text(&key_node)?.to_string();

            let value = match fav_node.child_by_field_name(FieldName::FieldValue) {
                Some(val_node) => Some(self.text(&val_node)?.to_string()),
                None => None,
            };

            let mut entry = schema::KeyValue::new();
            entry.key = Some(key);
            entry.value = value;
            attrs.entries.push(entry);
        }

        Ok(Some(attrs))
    }

    fn set_namespace(&self, obj: &mut impl HasNamespace) {
        if let Some(ns) = &self.namespace {
            obj.set_namespace(ns.clone());
        }
    }

    // -----------------------------------------------------------------------
    // Top-level dispatch
    // -----------------------------------------------------------------------

    fn visit_top_level(&mut self, node: &Node) -> Result<()> {
        match node.kind() {
            "include" => self.visit_include(node),
            "native_include" => self.visit_native_include(node),
            "namespace_decl" => self.visit_namespace(node),
            "type_decl" => self.visit_type_decl(node),
            "enum_decl" => self.visit_enum_decl(node),
            "union_decl" => self.visit_union_decl(node),
            "root_decl" => self.visit_root_decl(node),
            "file_extension_decl" => self.visit_file_extension(node),
            "file_identifier_decl" => self.visit_file_identifier(node),
            "attribute_decl" => self.visit_attribute_decl(node),
            "rpc_decl" => self.visit_rpc_decl(node),
            "comment" | "object" => Ok(()), // skip top-level comments and object literals
            kind if kind.starts_with("ERROR") || kind == "error" => {
                Err(ParseError::UnknownNodeType(format!(
                    "syntax error at {}:{}",
                    node.start_position().row + 1,
                    node.start_position().column + 1
                )))
            }
            _other => Ok(()), // ignore unrecognized nodes gracefully
        }
    }

    // -----------------------------------------------------------------------
    // Include
    // -----------------------------------------------------------------------

    fn visit_include(&mut self, node: &Node) -> Result<()> {
        let name_node = required_child(node, FieldName::IncludeName)?;
        let filename = self.string_content(&name_node)?;
        let mut sf = schema::SchemaFile::new();
        sf.filename = Some(filename);
        self.schema.fbs_files.push(sf);
        Ok(())
    }

    fn visit_native_include(&mut self, _node: &Node) -> Result<()> {
        // native_include is for C++ header files, not .fbs schemas.
        // We recognize the syntax but don't process it -- native includes
        // are only relevant to the C++ code generator.
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Namespace
    // -----------------------------------------------------------------------

    fn visit_namespace(&mut self, node: &Node) -> Result<()> {
        let ident_node = required_child(node, FieldName::NamespaceIdent)?;
        let ns_text = self.text(&ident_node)?.to_string();
        let mut ns = schema::Namespace::new();
        ns.namespace = Some(ns_text);
        self.namespace = Some(ns);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Type declaration (table / struct)
    // -----------------------------------------------------------------------

    fn visit_type_decl(&mut self, node: &Node) -> Result<()> {
        let mut obj = schema::Object::new();
        obj.span = Some(self.get_span(node));

        // Detect syntax errors inside the declaration body
        check_children_for_errors(node)?;

        if let Some(doc) = self.documentation(node)? {
            obj.documentation = Some(doc);
        }

        let kind_node = required_child(node, FieldName::TableOrStructDeclaration)?;
        let kind_text = self.text(&kind_node)?;
        obj.is_struct = Some(kind_text == "struct");

        let name_node = required_child(node, FieldName::TableOrStructName)?;
        obj.name = Some(self.text(&name_node)?.to_string());

        self.set_namespace(&mut obj);

        obj.attributes = self.attributes(node)?;

        for field_node in children_by_field(node, FieldName::FieldDeclaration) {
            obj.fields.push(self.parse_field(&field_node)?);
        }

        self.schema.objects.push(obj);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Field
    // -----------------------------------------------------------------------

    fn parse_field(&self, node: &Node) -> Result<schema::Field> {
        let mut field = schema::Field::new();
        field.span = Some(self.get_span(node));

        if let Some(doc) = self.documentation(node)? {
            field.documentation = Some(doc);
        }

        let key_node = required_child(node, FieldName::FieldKey)?;
        field.name = Some(self.text(&key_node)?.to_string());

        // Type
        field.type_ = Some(self.parse_field_type(node)?);

        // Attributes
        let attrs = self.attributes(node)?;
        if let Some(ref attrs) = attrs {
            self.apply_system_attributes(&mut field, attrs)?;
        }
        field.attributes = attrs;

        // Default value
        if let Some(value_node) = node.child_by_field_name(FieldName::FieldValue) {
            self.parse_default_value(&mut field, &value_node)?;
        }

        Ok(field)
    }

    fn apply_system_attributes(
        &self,
        field: &mut schema::Field,
        attrs: &schema::Attributes,
    ) -> Result<()> {
        for entry in &attrs.entries {
            let key = entry.key.as_deref().unwrap_or("");
            match key {
                "required" => field.is_required = Some(true),
                "optional" => field.is_optional = Some(true),
                "deprecated" => field.is_deprecated = Some(true),
                "key" => field.is_key = Some(true),
                "id" => {
                    let val_str = entry
                        .value
                        .as_deref()
                        .ok_or_else(|| ParseError::UnexpectedContent {
                            found: "id without value".into(),
                            context: "id attribute requires a numeric value".into(),
                        })?;
                    let id: u32 = val_str.parse().map_err(|e: std::num::ParseIntError| {
                        ParseError::InvalidInteger {
                            value: val_str.into(),
                            reason: e.to_string(),
                        }
                    })?;
                    field.id = Some(id);
                }
                _ => {} // user-defined attributes: kept in attributes list
            }
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Type parsing
    // -----------------------------------------------------------------------

    fn parse_field_type(&self, node: &Node) -> Result<schema::Type> {
        let type_node = required_child(node, FieldName::FieldType)?;
        self.parse_type_node(&type_node)
    }

    fn parse_type_node(&self, type_node: &Node) -> Result<schema::Type> {
        let mut fb_type = schema::Type::new();
        fb_type.span = Some(self.get_span(type_node));

        // Check for user-defined type (full_ident)
        if let Some(ident_node) = type_node.child_by_field_name(FieldName::FullIdent) {
            let type_name = self.text(&ident_node)?;
            // Store as TABLE placeholder; semantic analysis corrects this.
            fb_type.base_type = Some(BaseType::BASE_TYPE_TABLE);
            fb_type.base_size = Some(4); // offset size
            fb_type.unresolved_name = Some(type_name.to_string());
            return Ok(fb_type);
        }

        // Check for array/vector type: [element_type] or [element_type:fixed_len]
        if let Some(array_elem_node) = type_node.child_by_field_name(FieldName::ArrayType) {
            let element_type = self.parse_type_node(&array_elem_node)?;

            if let Some(fixed_len_node) =
                type_node.child_by_field_name(FieldName::ArrayTypeFixedLength)
            {
                // Fixed-length array: [type:N]
                let len_text = self.text(&fixed_len_node)?;
                let fixed_len = parse_int_literal(len_text)?;
                fb_type.base_type = Some(BaseType::BASE_TYPE_ARRAY);
                fb_type.fixed_length = Some(fixed_len as u32);
            } else {
                // Variable-length vector: [type]
                fb_type.base_type = Some(BaseType::BASE_TYPE_VECTOR);
            }

            fb_type.element_type = element_type.base_type;
            fb_type.element_size = element_type.base_size;
            fb_type.unresolved_name = element_type.unresolved_name;
            fb_type.base_size = Some(4); // uoffset_t
            return Ok(fb_type);
        }

        // Scalar / built-in type
        let type_text = self.text(type_node)?;
        let bt = lookup_base_type(type_text).ok_or_else(|| ParseError::UnknownBaseType(type_text.into()))?;
        fb_type.base_type = Some(bt);
        if let Some(size) = base_type_size(bt) {
            fb_type.base_size = Some(size);
        }

        Ok(fb_type)
    }

    // -----------------------------------------------------------------------
    // Default values
    // -----------------------------------------------------------------------

    fn parse_default_value(&self, field: &mut schema::Field, value_node: &Node) -> Result<()> {
        // value = single_value | object | array_value
        let sv_node = match value_node.child_by_field_name(FieldName::SingleValue) {
            Some(n) => n,
            None => {
                // Array or object default value.
                // FlatBuffers only supports empty default values (e.g. `= []`).
                // Non-empty defaults (e.g. `= [1, 2, 3]`) are not supported.
                let raw = self.text(value_node)?;
                let trimmed = raw.trim();
                // Check if it's an empty array (possibly with whitespace inside)
                let is_empty = trimmed == "[]"
                    || (trimmed.starts_with('[')
                        && trimmed.ends_with(']')
                        && trimmed[1..trimmed.len() - 1].trim().is_empty());
                if !is_empty {
                    return Err(ParseError::UnexpectedContent {
                        found: raw.to_string(),
                        context: format!(
                            "non-empty array/object default values are not supported for field '{}'",
                            field.name.as_deref().unwrap_or("<unknown>")
                        ),
                    });
                }
                // Empty array default -- store as empty string to indicate presence.
                field.default_string = Some(String::new());
                return Ok(());
            }
        };

        // single_value = scalar | string_constant | full_ident
        if let Some(scalar_node) = sv_node.child_by_field_name(FieldName::ScalarValue) {
            let text = self.text(&scalar_node)?;
            // Try integer parse first
            if let Ok(i) = parse_signed_int(text) {
                field.default_integer = Some(i);
                return Ok(());
            }
            // Try float
            if let Ok(f) = parse_float(text) {
                field.default_real = Some(f);
                return Ok(());
            }
            // Bool constants
            match text {
                "true" => {
                    field.default_integer = Some(1);
                    return Ok(());
                }
                "false" => {
                    field.default_integer = Some(0);
                    return Ok(());
                }
                _ => {}
            }
        }

        if let Some(string_node) = sv_node.child_by_field_name(FieldName::StringConstant) {
            field.default_string = Some(self.string_content(&string_node)?);
            return Ok(());
        }

        // full_ident = enum value reference like `Color.Blue` or `Blue`, or `null`
        if let Some(ident_node) = sv_node.child_by_field_name(FieldName::FullIdent) {
            let ident_text = self.text(&ident_node)?.to_string();
            if ident_text == "null" {
                // `= null` means the field is optional (no default value)
                field.is_optional = Some(true);
            } else {
                // Store as string default for Phase 2 resolution (e.g. enum values)
                field.default_string = Some(ident_text);
            }
        }

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Enum declaration
    // -----------------------------------------------------------------------

    fn visit_enum_decl(&mut self, node: &Node) -> Result<()> {
        check_children_for_errors(node)?;
        let mut enum_decl = schema::Enum::new();
        enum_decl.span = Some(self.get_span(node));

        if let Some(doc) = self.documentation(node)? {
            enum_decl.documentation = Some(doc);
        }

        self.set_namespace(&mut enum_decl);

        let name_node = required_child(node, FieldName::EnumName)?;
        enum_decl.name = Some(self.text(&name_node)?.to_string());

        // Underlying type (required for enums in grammar: `enum Foo : byte`)
        if let Some(type_node) = node.child_by_field_name(FieldName::EnumType) {
            let type_text = self.text(&type_node)?;
            let bt = lookup_base_type(type_text)
                .ok_or_else(|| ParseError::UnknownBaseType(type_text.into()))?;
            let mut underlying = schema::Type::new();
            underlying.base_type = Some(bt);
            enum_decl.underlying_type = Some(underlying);
        }

        enum_decl.attributes = self.attributes(node)?;

        // Enum values
        for val_node in children_by_field(node, FieldName::EnumValDecl) {
            let mut eval = schema::EnumVal::new();
            eval.span = Some(self.get_span(&val_node));

            if let Some(doc) = self.documentation(&val_node)? {
                eval.documentation = Some(doc);
            }

            let key_node = required_child(&val_node, FieldName::EnumKey)?;
            eval.name = Some(self.text(&key_node)?.to_string());

            if let Some(int_node) = val_node.child_by_field_name(FieldName::EnumIntConstant) {
                let text = self.text(&int_node)?;
                let value = parse_signed_int(text).map_err(|_| ParseError::InvalidInteger {
                    value: text.into(),
                    reason: "enum value must be an integer".into(),
                })?;
                eval.value = Some(value);
            }

            eval.attributes = self.attributes(&val_node)?;

            enum_decl.values.push(eval);
        }

        self.schema.enums.push(enum_decl);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Union declaration
    // -----------------------------------------------------------------------

    fn visit_union_decl(&mut self, node: &Node) -> Result<()> {
        // Note: skip check_children_for_errors here because the grammar
        // produces ERROR nodes for valid `union Name : type { ... }` syntax.
        let mut union_decl = schema::Enum::new();
        union_decl.span = Some(self.get_span(node));
        union_decl.is_union = Some(true);

        if let Some(doc) = self.documentation(node)? {
            union_decl.documentation = Some(doc);
        }

        self.set_namespace(&mut union_decl);

        let name_node = required_child(node, FieldName::UnionName)?;
        union_decl.name = Some(self.text(&name_node)?.to_string());

        // Underlying type for unions is always UType (uint8 discriminator)
        let mut utype = schema::Type::new();
        utype.base_type = Some(BaseType::BASE_TYPE_U_TYPE);
        union_decl.underlying_type = Some(utype);

        union_decl.attributes = self.attributes(node)?;

        // Add NONE sentinel at index 0
        let mut none_val = schema::EnumVal::new();
        none_val.name = Some("NONE".to_string());
        none_val.value = Some(0);
        union_decl.values.push(none_val);

        // Union fields: `union U { Name: Type, Name2, ... }`
        // Each field has a key (full_ident) and optional value (: type).
        // When value is omitted, the key name IS the type name.
        let fields: Vec<Node> =
            children_by_field(node, FieldName::UnionFieldDecl).collect();

        for (i, field_node) in fields.iter().enumerate() {
            let mut eval = schema::EnumVal::new();
            eval.span = Some(self.get_span(field_node));

            if let Some(doc) = self.documentation(field_node)? {
                eval.documentation = Some(doc);
            }

            let key_node = required_child(field_node, FieldName::UnionFieldKey)?;
            let key_text = self.text(&key_node)?;
            eval.name = Some(key_text.to_string());
            eval.value = Some((i + 1) as i64);

            // Parse the variant's type (optional - when absent, key name is the type)
            if let Some(type_node) = field_node.child_by_field_name(FieldName::UnionFieldValue) {
                let variant_type = self.parse_type_node(&type_node)?;
                eval.union_type = Some(variant_type);
            } else {
                // No explicit type: variant name is the type name
                let mut variant_type = schema::Type::new();
                variant_type.base_type = Some(BaseType::BASE_TYPE_TABLE);
                variant_type.base_size = Some(4);
                variant_type.unresolved_name = Some(key_text.to_string());
                eval.union_type = Some(variant_type);
            }

            // Parse optional attributes on the union field
            eval.attributes = self.attributes(field_node)?;

            union_decl.values.push(eval);
        }

        self.schema.enums.push(union_decl);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Root type, file metadata, attribute declarations
    // -----------------------------------------------------------------------

    fn visit_root_decl(&mut self, node: &Node) -> Result<()> {
        let ident_node = required_child(node, FieldName::RootTypeIdent)?;
        self.state.root_type_name = Some(self.text(&ident_node)?.to_string());
        self.state.root_type_span = Some(self.get_span(node));
        self.state.root_type_namespace = self
            .namespace
            .as_ref()
            .and_then(|ns| ns.namespace.clone());
        Ok(())
    }

    fn visit_file_extension(&mut self, node: &Node) -> Result<()> {
        let ext_node = required_child(node, FieldName::FileExtensionConstant)?;
        self.schema.file_ext = Some(self.string_content(&ext_node)?);
        Ok(())
    }

    fn visit_file_identifier(&mut self, node: &Node) -> Result<()> {
        let id_node = required_child(node, FieldName::FileIdentifierConstant)?;
        self.schema.file_ident = Some(self.string_content(&id_node)?);
        self.state.file_ident_span = Some(self.get_span(node));
        Ok(())
    }

    fn visit_attribute_decl(&mut self, node: &Node) -> Result<()> {
        let name_node = required_child(node, FieldName::AttributeName)?;
        let attr_name = self.text(&name_node)?.to_string();
        self.state.declared_attributes.push(attr_name);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // RPC service
    // -----------------------------------------------------------------------

    fn visit_rpc_decl(&mut self, node: &Node) -> Result<()> {
        let mut service = schema::Service::new();
        service.span = Some(self.get_span(node));

        if let Some(doc) = self.documentation(node)? {
            service.documentation = Some(doc);
        }

        self.set_namespace(&mut service);

        let name_node = required_child(node, FieldName::RpcName)?;
        service.name = Some(self.text(&name_node)?.to_string());

        for method_node in children_by_field(node, FieldName::RpcMethod) {
            service.calls.push(self.parse_rpc_method(&method_node)?);
        }

        self.schema.services.push(service);
        Ok(())
    }

    fn parse_rpc_method(&self, node: &Node) -> Result<schema::RpcCall> {
        let mut call = schema::RpcCall::new();
        call.span = Some(self.get_span(node));

        if let Some(doc) = self.documentation(node)? {
            call.documentation = Some(doc);
        }

        let name_node = required_child(node, FieldName::RpcMethodName)?;
        call.name = Some(self.text(&name_node)?.to_string());

        // Request and response are type name identifiers.
        // Store as stub Objects with just the name; semantic analysis resolves them.
        let param_node = required_child(node, FieldName::RpcParameter)?;
        let mut req = schema::Object::new();
        req.name = Some(self.text(&param_node)?.to_string());
        call.request = Some(req);

        let return_node = required_child(node, FieldName::RpcReturnType)?;
        let mut resp = schema::Object::new();
        resp.name = Some(self.text(&return_node)?.to_string());
        call.response = Some(resp);

        call.attributes = self.attributes(node)?;

        Ok(call)
    }
}

// ---------------------------------------------------------------------------
// Trait for setting namespace on different schema types
// ---------------------------------------------------------------------------

trait HasNamespace {
    fn set_namespace(&mut self, ns: schema::Namespace);
}

impl HasNamespace for schema::Object {
    fn set_namespace(&mut self, ns: schema::Namespace) {
        self.namespace = Some(ns);
    }
}

impl HasNamespace for schema::Enum {
    fn set_namespace(&mut self, ns: schema::Namespace) {
        self.namespace = Some(ns);
    }
}

impl HasNamespace for schema::Service {
    fn set_namespace(&mut self, ns: schema::Namespace) {
        self.namespace = Some(ns);
    }
}

// ---------------------------------------------------------------------------
// Tree-sitter error detection
// ---------------------------------------------------------------------------

/// Check immediate children of a declaration node for ERROR/MISSING nodes.
/// Returns an error for the first ERROR child found (skips MISSING nodes for
/// default_value fields, since `= []` generates a benign MISSING node).
fn check_children_for_errors(node: &Node) -> Result<()> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.is_error() {
            let start = child.start_position();
            return Err(ParseError::SyntaxError {
                line: start.row + 1,
                column: start.column + 1,
                context: "unexpected token".to_string(),
            });
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Free functions: numeric parsing, string unescaping
// ---------------------------------------------------------------------------

/// Parse an integer literal (decimal or hex), unsigned.
fn parse_int_literal(s: &str) -> Result<i64> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        i64::from_str_radix(hex, 16).map_err(|e| ParseError::InvalidInteger {
            value: s.into(),
            reason: e.to_string(),
        })
    } else {
        s.parse::<i64>().map_err(|e| ParseError::InvalidInteger {
            value: s.into(),
            reason: e.to_string(),
        })
    }
}

/// Parse a possibly-signed integer from tree-sitter text (which may include +/- prefix).
fn parse_signed_int(s: &str) -> std::result::Result<i64, ()> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        i64::from_str_radix(hex, 16).map_err(|_| ())
    } else if let Some(rest) = s.strip_prefix('-') {
        if let Some(hex) = rest.strip_prefix("0x").or_else(|| rest.strip_prefix("0X")) {
            i64::from_str_radix(hex, 16).map(|v| -v).map_err(|_| ())
        } else {
            rest.parse::<i64>().map(|v| -v).map_err(|_| ())
        }
    } else if let Some(rest) = s.strip_prefix('+') {
        rest.parse::<i64>().map_err(|_| ())
    } else {
        s.parse::<i64>().map_err(|_| ())
    }
}

/// Parse a float literal including special values (nan, inf, infinity).
fn parse_float(s: &str) -> std::result::Result<f64, ()> {
    let s = s.trim();
    match s {
        "nan" => Ok(f64::NAN),
        "inf" | "infinity" => Ok(f64::INFINITY),
        "-inf" | "-infinity" => Ok(f64::NEG_INFINITY),
        "+inf" | "+infinity" => Ok(f64::INFINITY),
        _ => s.parse::<f64>().map_err(|_| ()),
    }
}

/// Unescape a string literal's contents (between the quotes).
fn unescape_string(s: &str) -> Result<String> {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();

    while let Some(c) = chars.next() {
        if c != '\\' {
            result.push(c);
            continue;
        }

        match chars.next() {
            Some('n') => result.push('\n'),
            Some('t') => result.push('\t'),
            Some('r') => result.push('\r'),
            Some('\\') => result.push('\\'),
            Some('"') => result.push('"'),
            Some(d @ '0'..='7') => {
                // Octal escape: 1-3 octal digits
                let mut octal = String::new();
                octal.push(d);
                // Consume up to 2 more octal digits
                for _ in 0..2 {
                    match chars.clone().next() {
                        Some(c @ '0'..='7') => {
                            octal.push(c);
                            chars.next();
                        }
                        _ => break,
                    }
                }
                let code = u32::from_str_radix(&octal, 8)
                    .map_err(|_| ParseError::InvalidEscape(format!("\\{octal}")))?;
                let ch = char::from_u32(code)
                    .ok_or_else(|| ParseError::InvalidEscape(format!("\\{octal}")))?;
                result.push(ch);
            }
            Some('x') => {
                let hex: String = chars.by_ref().take(2).collect();
                let code = u32::from_str_radix(&hex, 16)
                    .map_err(|_| ParseError::InvalidEscape(format!("\\x{hex}")))?;
                let ch = char::from_u32(code)
                    .ok_or_else(|| ParseError::InvalidEscape(format!("\\x{hex}")))?;
                result.push(ch);
            }
            Some('u') => {
                let hex: String = chars.by_ref().take(4).collect();
                let code = u32::from_str_radix(&hex, 16)
                    .map_err(|_| ParseError::InvalidEscape(format!("\\u{hex}")))?;
                let ch = char::from_u32(code)
                    .ok_or_else(|| ParseError::InvalidEscape(format!("\\u{hex}")))?;
                result.push(ch);
            }
            Some('U') => {
                let hex: String = chars.by_ref().take(8).collect();
                let code = u32::from_str_radix(&hex, 16)
                    .map_err(|_| ParseError::InvalidEscape(format!("\\U{hex}")))?;
                let ch = char::from_u32(code)
                    .ok_or_else(|| ParseError::InvalidEscape(format!("\\U{hex}")))?;
                result.push(ch);
            }
            Some(other) => {
                return Err(ParseError::InvalidEscape(format!("\\{other}")));
            }
            None => {
                return Err(ParseError::InvalidEscape("trailing backslash".into()));
            }
        }
    }

    Ok(result)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_int_literal_decimal() {
        assert_eq!(parse_int_literal("42").unwrap(), 42);
        assert_eq!(parse_int_literal("0").unwrap(), 0);
        assert_eq!(parse_int_literal("123456").unwrap(), 123456);
    }

    #[test]
    fn test_parse_int_literal_hex() {
        assert_eq!(parse_int_literal("0xFF").unwrap(), 255);
        assert_eq!(parse_int_literal("0XFF").unwrap(), 255);
        assert_eq!(parse_int_literal("0xF").unwrap(), 15);
        assert_eq!(parse_int_literal("0x0").unwrap(), 0);
    }

    #[test]
    fn test_parse_signed_int() {
        assert_eq!(parse_signed_int("42"), Ok(42));
        assert_eq!(parse_signed_int("-1"), Ok(-1));
        assert_eq!(parse_signed_int("+5"), Ok(5));
        assert_eq!(parse_signed_int("-0xFF"), Ok(-255));
        assert!(parse_signed_int("abc").is_err());
    }

    #[test]
    fn test_parse_float() {
        assert_eq!(parse_float("3.14"), Ok(3.14));
        assert_eq!(parse_float("1e10"), Ok(1e10));
        assert!(parse_float("nan").unwrap().is_nan());
        assert_eq!(parse_float("inf"), Ok(f64::INFINITY));
        assert_eq!(parse_float("-inf"), Ok(f64::NEG_INFINITY));
        assert_eq!(parse_float("infinity"), Ok(f64::INFINITY));
        assert_eq!(parse_float("+inf"), Ok(f64::INFINITY));
    }

    #[test]
    fn test_unescape_string_basic() {
        assert_eq!(unescape_string("hello").unwrap(), "hello");
        assert_eq!(unescape_string("").unwrap(), "");
    }

    #[test]
    fn test_unescape_string_escapes() {
        assert_eq!(unescape_string("a\\nb").unwrap(), "a\nb");
        assert_eq!(unescape_string("a\\tb").unwrap(), "a\tb");
        assert_eq!(unescape_string("a\\rb").unwrap(), "a\rb");
        assert_eq!(unescape_string("a\\\\b").unwrap(), "a\\b");
        assert_eq!(unescape_string("a\\\"b").unwrap(), "a\"b");
        assert_eq!(unescape_string("a\\0b").unwrap(), "a\0b");
    }

    #[test]
    fn test_unescape_string_hex() {
        assert_eq!(unescape_string("\\x41").unwrap(), "A");
        assert_eq!(unescape_string("\\x61").unwrap(), "a");
    }

    #[test]
    fn test_unescape_string_unicode() {
        assert_eq!(unescape_string("\\u0041").unwrap(), "A");
        assert_eq!(unescape_string("\\U00000041").unwrap(), "A");
    }

    #[test]
    fn test_unescape_string_octal() {
        // \0 = null
        assert_eq!(unescape_string("\\0").unwrap(), "\0");
        // \101 = 'A' (65 in octal = 0o101)
        assert_eq!(unescape_string("\\101").unwrap(), "A");
        // \077 = '?' (63 in octal = 0o077)
        assert_eq!(unescape_string("\\077").unwrap(), "?");
        // \0 followed by non-octal
        assert_eq!(unescape_string("\\0x").unwrap(), "\0x");
    }

    #[test]
    fn test_unescape_string_invalid() {
        assert!(unescape_string("\\q").is_err());
    }
}
