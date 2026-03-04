use crate::base_type::{base_type_size, lookup_base_type};
use crate::error::{ParseError, Result};
use crate::tokenizer::{tokenize, Token, TokenKind};
use flatc_rs_schema::{self as schema, BaseType};

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
// FbsParser
// ---------------------------------------------------------------------------

pub struct FbsParser<'src> {
    source: &'src str,
    file_name: Option<String>,
    tokens: Vec<Token>,
    pos: usize,
    schema: schema::Schema,
    state: ParserState,
    namespace: Option<schema::Namespace>,
}

impl<'src> FbsParser<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            source,
            file_name: None,
            tokens: Vec::new(),
            pos: 0,
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
        self.tokens = tokenize(self.source)?;

        while !self.at_eof() {
            // Collect doc comments before the next declaration
            let doc_comments = self.collect_doc_comments();

            if self.at_eof() {
                break;
            }

            let span_token = if !doc_comments.is_empty() {
                // Span starts at first doc comment
                doc_comments[0].clone()
            } else {
                self.peek().clone()
            };

            match self.peek_text() {
                "include" => self.parse_include()?,
                "native_include" => self.parse_native_include()?,
                "namespace" => self.parse_namespace()?,
                "table" | "struct" => self.parse_type_decl(doc_comments, &span_token)?,
                "enum" => self.parse_enum_decl(doc_comments, &span_token)?,
                "union" => self.parse_union_decl(doc_comments, &span_token)?,
                "root_type" => self.parse_root_decl()?,
                "file_extension" => self.parse_file_extension()?,
                "file_identifier" => self.parse_file_identifier()?,
                "attribute" => self.parse_attribute_decl()?,
                "rpc_service" => self.parse_rpc_decl(doc_comments, &span_token)?,
                _ => {
                    // Unknown top-level token -- skip it gracefully
                    self.advance();
                }
            }
        }

        Ok(ParseOutput {
            schema: self.schema,
            state: self.state,
        })
    }

    // -----------------------------------------------------------------------
    // Token navigation
    // -----------------------------------------------------------------------

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn peek_text(&self) -> &str {
        &self.tokens[self.pos].text
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    fn at_eof(&self) -> bool {
        self.tokens[self.pos].kind == TokenKind::Eof
    }

    fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos];
        if tok.kind != TokenKind::Eof {
            self.pos += 1;
        }
        tok
    }

    fn expect_kind(&mut self, kind: TokenKind) -> Result<&Token> {
        let tok = &self.tokens[self.pos];
        if tok.kind != kind {
            return Err(ParseError::SyntaxError {
                line: tok.line as usize,
                column: tok.col as usize,
                context: format!("expected {:?}, found '{}'", kind, tok.text),
            });
        }
        self.pos += 1;
        Ok(&self.tokens[self.pos - 1])
    }

    fn expect_ident(&mut self, keyword: &str) -> Result<&Token> {
        let tok = &self.tokens[self.pos];
        if tok.kind != TokenKind::Ident || tok.text != keyword {
            return Err(ParseError::SyntaxError {
                line: tok.line as usize,
                column: tok.col as usize,
                context: format!("expected '{}', found '{}'", keyword, tok.text),
            });
        }
        self.pos += 1;
        Ok(&self.tokens[self.pos - 1])
    }

    /// Read the next token as any identifier (including keywords used as names).
    fn expect_ident_any(&mut self) -> Result<&Token> {
        let tok = &self.tokens[self.pos];
        if tok.kind != TokenKind::Ident {
            return Err(ParseError::SyntaxError {
                line: tok.line as usize,
                column: tok.col as usize,
                context: format!("expected identifier, found '{}'", tok.text),
            });
        }
        self.pos += 1;
        Ok(&self.tokens[self.pos - 1])
    }

    fn get_span(&self, token: &Token) -> schema::Span {
        schema::Span::new(self.file_name.clone(), token.line, token.col)
    }

    fn set_namespace(&self, obj: &mut impl HasNamespace) {
        if let Some(ns) = &self.namespace {
            obj.set_namespace(ns.clone());
        }
    }

    // -----------------------------------------------------------------------
    // Doc comment collection
    // -----------------------------------------------------------------------

    fn collect_doc_comments(&mut self) -> Vec<Token> {
        let mut docs = Vec::new();
        while self.tokens[self.pos].kind == TokenKind::DocComment {
            docs.push(self.tokens[self.pos].clone());
            self.pos += 1;
        }
        docs
    }

    fn make_documentation(doc_tokens: &[Token]) -> Option<schema::Documentation> {
        if doc_tokens.is_empty() {
            return None;
        }
        let mut doc = schema::Documentation::new();
        doc.lines = doc_tokens.iter().map(|t| t.text.clone()).collect();
        Some(doc)
    }

    // -----------------------------------------------------------------------
    // Include
    // -----------------------------------------------------------------------

    fn parse_include(&mut self) -> Result<()> {
        self.expect_ident("include")?;
        let name_tok = self.expect_kind(TokenKind::StringLit)?;
        let filename = unescape_string(&name_tok.text)?;
        self.expect_kind(TokenKind::Semicolon)?;
        let mut sf = schema::SchemaFile::new();
        sf.filename = Some(filename);
        self.schema.fbs_files.push(sf);
        Ok(())
    }

    fn parse_native_include(&mut self) -> Result<()> {
        self.expect_ident("native_include")?;
        self.expect_kind(TokenKind::StringLit)?;
        self.expect_kind(TokenKind::Semicolon)?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Namespace
    // -----------------------------------------------------------------------

    fn parse_namespace(&mut self) -> Result<()> {
        self.expect_ident("namespace")?;
        let ns_text = self.parse_full_ident()?;
        self.expect_kind(TokenKind::Semicolon)?;
        let mut ns = schema::Namespace::new();
        ns.namespace = Some(ns_text);
        self.namespace = Some(ns);
        Ok(())
    }

    /// Parse a dotted identifier like `Foo.Bar.Baz`.
    fn parse_full_ident(&mut self) -> Result<String> {
        let first = self.expect_ident_any()?;
        let mut result = first.text.clone();
        while *self.peek_kind() == TokenKind::Dot {
            self.advance(); // .
            let next = self.expect_ident_any()?;
            result.push('.');
            result.push_str(&next.text);
        }
        Ok(result)
    }

    // -----------------------------------------------------------------------
    // Type declaration (table / struct)
    // -----------------------------------------------------------------------

    fn parse_type_decl(&mut self, doc: Vec<Token>, span_token: &Token) -> Result<()> {
        let mut obj = schema::Object::new();
        obj.span = Some(self.get_span(span_token));
        obj.documentation = Self::make_documentation(&doc);

        let kind_tok = self.advance();
        obj.is_struct = Some(kind_tok.text == "struct");

        let name_tok = self.expect_ident_any()?;
        obj.name = Some(name_tok.text.clone());

        self.set_namespace(&mut obj);

        // Optional metadata
        obj.attributes = self.parse_optional_metadata()?;

        self.expect_kind(TokenKind::LBrace)?;

        while *self.peek_kind() != TokenKind::RBrace && !self.at_eof() {
            let field_docs = self.collect_doc_comments();
            if *self.peek_kind() == TokenKind::RBrace {
                break;
            }
            let field_span_token = if !field_docs.is_empty() {
                field_docs[0].clone()
            } else {
                self.peek().clone()
            };
            obj.fields
                .push(self.parse_field(field_docs, &field_span_token)?);
        }

        self.expect_kind(TokenKind::RBrace)?;

        self.schema.objects.push(obj);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Field
    // -----------------------------------------------------------------------

    fn parse_field(&mut self, doc: Vec<Token>, span_token: &Token) -> Result<schema::Field> {
        let mut field = schema::Field::new();
        field.span = Some(self.get_span(span_token));
        field.documentation = Self::make_documentation(&doc);

        let name_tok = self.expect_ident_any()?;
        field.name = Some(name_tok.text.clone());

        self.expect_kind(TokenKind::Colon)?;

        // Type
        field.type_ = Some(self.parse_type()?);

        // Optional default value
        if *self.peek_kind() == TokenKind::Eq {
            self.advance(); // =
            self.parse_default_value(&mut field)?;
        }

        // Optional metadata
        let attrs = self.parse_optional_metadata()?;
        if let Some(ref attrs) = attrs {
            self.apply_system_attributes(&mut field, attrs)?;
        }
        field.attributes = attrs;

        self.expect_kind(TokenKind::Semicolon)?;

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
                    let val_str =
                        entry
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

    fn parse_type(&mut self) -> Result<schema::Type> {
        let mut fb_type = schema::Type::new();
        let type_start = self.peek().clone();
        fb_type.span = Some(self.get_span(&type_start));

        // Check for vector/array: [type] or [type:length]
        if *self.peek_kind() == TokenKind::LBracket {
            self.advance(); // [
            let element_type = self.parse_type()?;

            if *self.peek_kind() == TokenKind::Colon {
                // Fixed-length array: [type:N]
                self.advance(); // :
                let len_tok = self.expect_kind(TokenKind::IntLit)?;
                let len_text = &len_tok.text;
                let fixed_len = parse_int_literal(len_text)?;
                if fixed_len < 0 || fixed_len > u32::MAX as i64 {
                    return Err(ParseError::InvalidInteger {
                        value: len_text.to_string(),
                        reason: format!(
                            "fixed array length must be 0..{}, got {fixed_len}",
                            u32::MAX
                        ),
                    });
                }
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

            self.expect_kind(TokenKind::RBracket)?;
            return Ok(fb_type);
        }

        // Scalar or user-defined type
        let tok = self.expect_ident_any()?;
        let type_text = tok.text.clone();

        // Check for dotted type name (e.g., Foo.Bar)
        if *self.peek_kind() == TokenKind::Dot {
            let mut full_name = type_text;
            while *self.peek_kind() == TokenKind::Dot {
                self.advance(); // .
                let next = self.expect_ident_any()?;
                full_name.push('.');
                full_name.push_str(&next.text);
            }
            fb_type.base_type = Some(BaseType::BASE_TYPE_TABLE);
            fb_type.base_size = Some(4);
            fb_type.unresolved_name = Some(full_name);
            return Ok(fb_type);
        }

        // Try scalar type
        if let Some(bt) = lookup_base_type(&type_text) {
            fb_type.base_type = Some(bt);
            if let Some(size) = base_type_size(bt) {
                fb_type.base_size = Some(size);
            }
        } else {
            // User-defined type (single identifier)
            fb_type.base_type = Some(BaseType::BASE_TYPE_TABLE);
            fb_type.base_size = Some(4);
            fb_type.unresolved_name = Some(type_text);
        }

        Ok(fb_type)
    }

    // -----------------------------------------------------------------------
    // Default values
    // -----------------------------------------------------------------------

    fn parse_default_value(&mut self, field: &mut schema::Field) -> Result<()> {
        // Check for empty array default `[]`
        if *self.peek_kind() == TokenKind::LBracket {
            self.advance(); // [
            if *self.peek_kind() == TokenKind::RBracket {
                self.advance(); // ]
                field.default_string = Some(String::new());
                return Ok(());
            }
            // Non-empty array default
            return Err(ParseError::UnexpectedContent {
                found: format!("[{}...", self.peek_text()),
                context: format!(
                    "non-empty array/object default values are not supported for field '{}'",
                    field.name.as_deref().unwrap_or("<unknown>")
                ),
            });
        }

        // Check for object default `{}`
        if *self.peek_kind() == TokenKind::LBrace {
            // Skip to matching `}`
            self.advance(); // {
            let mut depth = 1;
            while depth > 0 && !self.at_eof() {
                match self.peek_kind() {
                    TokenKind::LBrace => {
                        depth += 1;
                        self.advance();
                    }
                    TokenKind::RBrace => {
                        depth -= 1;
                        self.advance();
                    }
                    _ => {
                        self.advance();
                    }
                }
            }
            return Err(ParseError::UnexpectedContent {
                found: "{...}".to_string(),
                context: format!(
                    "non-empty array/object default values are not supported for field '{}'",
                    field.name.as_deref().unwrap_or("<unknown>")
                ),
            });
        }

        // Sign prefix
        let sign = match self.peek_kind() {
            TokenKind::Plus => {
                self.advance();
                "+"
            }
            TokenKind::Minus => {
                self.advance();
                "-"
            }
            _ => "",
        };

        // String default
        if *self.peek_kind() == TokenKind::StringLit {
            let tok = self.advance();
            field.default_string = Some(unescape_string(&tok.text)?);
            return Ok(());
        }

        // Integer literal
        if *self.peek_kind() == TokenKind::IntLit {
            let tok = self.advance();
            let text = format!("{}{}", sign, tok.text);
            if let Ok(i) = parse_signed_int(&text) {
                field.default_integer = Some(i);
                return Ok(());
            }
        }

        // Float literal
        if *self.peek_kind() == TokenKind::FloatLit {
            let tok = self.advance();
            let text = format!("{}{}", sign, tok.text);
            if let Ok(f) = parse_float(&text) {
                field.default_real = Some(f);
                return Ok(());
            }
        }

        // Identifier: true, false, nan, inf, infinity, enum value, null
        if *self.peek_kind() == TokenKind::Ident {
            let tok = self.advance();
            let text = format!("{}{}", sign, tok.text);

            // Bool constants
            if text == "true" {
                field.default_integer = Some(1);
                return Ok(());
            }
            if text == "false" {
                field.default_integer = Some(0);
                return Ok(());
            }

            // Float special values
            if let Ok(f) = parse_float(&text) {
                field.default_real = Some(f);
                return Ok(());
            }

            // null
            if text == "null" {
                field.is_optional = Some(true);
                return Ok(());
            }

            // Enum value reference (possibly dotted)
            let mut ident = text;
            while *self.peek_kind() == TokenKind::Dot {
                self.advance(); // .
                let next = self.expect_ident_any()?;
                ident.push('.');
                ident.push_str(&next.text);
            }
            field.default_string = Some(ident);
            return Ok(());
        }

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Enum declaration
    // -----------------------------------------------------------------------

    fn parse_enum_decl(&mut self, doc: Vec<Token>, span_token: &Token) -> Result<()> {
        let mut enum_decl = schema::Enum::new();
        enum_decl.span = Some(self.get_span(span_token));
        enum_decl.documentation = Self::make_documentation(&doc);

        self.expect_ident("enum")?;

        let name_tok = self.expect_ident_any()?;
        enum_decl.name = Some(name_tok.text.clone());

        self.set_namespace(&mut enum_decl);

        // Underlying type: `enum Foo : byte`
        self.expect_kind(TokenKind::Colon)?;
        let type_tok = self.expect_ident_any()?;
        let bt = lookup_base_type(&type_tok.text)
            .ok_or_else(|| ParseError::UnknownBaseType(type_tok.text.clone()))?;
        let mut underlying = schema::Type::new();
        underlying.base_type = Some(bt);
        enum_decl.underlying_type = Some(underlying);

        // Optional metadata
        enum_decl.attributes = self.parse_optional_metadata()?;

        self.expect_kind(TokenKind::LBrace)?;

        // Enum values: comma-separated
        let mut first = true;
        while *self.peek_kind() != TokenKind::RBrace && !self.at_eof() {
            if !first {
                self.expect_kind(TokenKind::Comma)?;
                // Allow trailing comma
                if *self.peek_kind() == TokenKind::RBrace {
                    break;
                }
            }
            first = false;

            // Doc comments for value
            let val_docs = self.collect_doc_comments();
            if *self.peek_kind() == TokenKind::RBrace {
                break;
            }
            let val_span_token = if !val_docs.is_empty() {
                val_docs[0].clone()
            } else {
                self.peek().clone()
            };

            let mut eval = schema::EnumVal::new();
            eval.span = Some(self.get_span(&val_span_token));
            eval.documentation = Self::make_documentation(&val_docs);

            let key_tok = self.expect_ident_any()?;
            eval.name = Some(key_tok.text.clone());

            // Optional value: `= N`
            if *self.peek_kind() == TokenKind::Eq {
                self.advance(); // =
                                // Handle sign
                let sign = match self.peek_kind() {
                    TokenKind::Minus => {
                        self.advance();
                        "-"
                    }
                    TokenKind::Plus => {
                        self.advance();
                        ""
                    }
                    _ => "",
                };
                let int_tok = self.expect_kind(TokenKind::IntLit)?;
                let text = format!("{}{}", sign, int_tok.text);
                let value = parse_signed_int(&text).map_err(|_| ParseError::InvalidInteger {
                    value: text.clone(),
                    reason: "enum value must be an integer".into(),
                })?;
                eval.value = Some(value);
            }

            // Optional metadata on enum value
            eval.attributes = self.parse_optional_metadata()?;

            enum_decl.values.push(eval);
        }

        self.expect_kind(TokenKind::RBrace)?;

        self.schema.enums.push(enum_decl);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Union declaration
    // -----------------------------------------------------------------------

    fn parse_union_decl(&mut self, doc: Vec<Token>, span_token: &Token) -> Result<()> {
        let mut union_decl = schema::Enum::new();
        union_decl.span = Some(self.get_span(span_token));
        union_decl.is_union = Some(true);
        union_decl.documentation = Self::make_documentation(&doc);

        self.expect_ident("union")?;

        let name_tok = self.expect_ident_any()?;
        union_decl.name = Some(name_tok.text.clone());

        self.set_namespace(&mut union_decl);

        // Optional explicit underlying type: `union Foo : uint { ... }`
        if *self.peek_kind() == TokenKind::Colon {
            self.advance(); // :
            let type_tok = self.expect_ident_any()?;
            let bt = lookup_base_type(&type_tok.text)
                .ok_or_else(|| ParseError::UnknownBaseType(type_tok.text.clone()))?;
            let mut underlying = schema::Type::new();
            underlying.base_type = Some(bt);
            union_decl.underlying_type = Some(underlying);
        } else {
            // Default: UType (uint8 discriminator)
            let mut utype = schema::Type::new();
            utype.base_type = Some(BaseType::BASE_TYPE_U_TYPE);
            union_decl.underlying_type = Some(utype);
        }

        // Optional metadata
        union_decl.attributes = self.parse_optional_metadata()?;

        self.expect_kind(TokenKind::LBrace)?;

        // Add NONE sentinel at index 0
        let mut none_val = schema::EnumVal::new();
        none_val.name = Some("NONE".to_string());
        none_val.value = Some(0);
        union_decl.values.push(none_val);

        // Union fields
        let mut index = 1i64;
        let mut first = true;
        while *self.peek_kind() != TokenKind::RBrace && !self.at_eof() {
            if !first {
                self.expect_kind(TokenKind::Comma)?;
                if *self.peek_kind() == TokenKind::RBrace {
                    break;
                }
            }
            first = false;

            // Doc comments for variant
            let var_docs = self.collect_doc_comments();
            if *self.peek_kind() == TokenKind::RBrace {
                break;
            }
            let var_span_token = if !var_docs.is_empty() {
                var_docs[0].clone()
            } else {
                self.peek().clone()
            };

            let mut eval = schema::EnumVal::new();
            eval.span = Some(self.get_span(&var_span_token));
            eval.documentation = Self::make_documentation(&var_docs);
            eval.value = Some(index);

            // Parse variant key (full_ident)
            let key_text = self.parse_full_ident()?;
            eval.name = Some(key_text.clone());

            // Optional explicit type: `: Type`
            if *self.peek_kind() == TokenKind::Colon {
                self.advance(); // :
                let variant_type = self.parse_type()?;
                eval.union_type = Some(variant_type);
            } else {
                // Positional: variant name IS the type name (no span)
                let mut variant_type = schema::Type::new();
                variant_type.base_type = Some(BaseType::BASE_TYPE_TABLE);
                variant_type.base_size = Some(4);
                variant_type.unresolved_name = Some(key_text);
                eval.union_type = Some(variant_type);
            }

            // Optional explicit discriminant value: `= N`
            if *self.peek_kind() == TokenKind::Eq {
                self.advance(); // =
                let sign = match self.peek_kind() {
                    TokenKind::Minus => {
                        self.advance();
                        "-"
                    }
                    TokenKind::Plus => {
                        self.advance();
                        ""
                    }
                    _ => "",
                };
                let int_tok = self.expect_kind(TokenKind::IntLit)?;
                let text = format!("{}{}", sign, int_tok.text);
                let value = parse_signed_int(&text).map_err(|_| ParseError::InvalidInteger {
                    value: text.clone(),
                    reason: "union discriminant must be an integer".into(),
                })?;
                eval.value = Some(value);
                index = value + 1; // Next auto-value follows this
            }

            // Optional metadata on union variant
            eval.attributes = self.parse_optional_metadata()?;

            union_decl.values.push(eval);
            index += 1;
        }

        self.expect_kind(TokenKind::RBrace)?;

        self.schema.enums.push(union_decl);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Root type, file metadata, attribute declarations
    // -----------------------------------------------------------------------

    fn parse_root_decl(&mut self) -> Result<()> {
        let span_token = self.peek().clone();
        self.expect_ident("root_type")?;
        let ident_tok = self.expect_ident_any()?;
        self.state.root_type_name = Some(ident_tok.text.clone());
        self.state.root_type_span = Some(self.get_span(&span_token));
        self.state.root_type_namespace =
            self.namespace.as_ref().and_then(|ns| ns.namespace.clone());
        self.expect_kind(TokenKind::Semicolon)?;
        Ok(())
    }

    fn parse_file_extension(&mut self) -> Result<()> {
        self.expect_ident("file_extension")?;
        let tok = self.expect_kind(TokenKind::StringLit)?;
        self.schema.file_ext = Some(unescape_string(&tok.text)?);
        self.expect_kind(TokenKind::Semicolon)?;
        Ok(())
    }

    fn parse_file_identifier(&mut self) -> Result<()> {
        let span_token = self.peek().clone();
        self.expect_ident("file_identifier")?;
        let tok = self.expect_kind(TokenKind::StringLit)?;
        self.schema.file_ident = Some(unescape_string(&tok.text)?);
        self.state.file_ident_span = Some(self.get_span(&span_token));
        self.expect_kind(TokenKind::Semicolon)?;
        Ok(())
    }

    fn parse_attribute_decl(&mut self) -> Result<()> {
        self.expect_ident("attribute")?;
        let attr_name = if *self.peek_kind() == TokenKind::StringLit {
            let tok = self.advance();
            tok.text.clone()
        } else {
            let tok = self.expect_ident_any()?;
            tok.text.clone()
        };
        self.expect_kind(TokenKind::Semicolon)?;
        self.state.declared_attributes.push(attr_name);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // RPC service
    // -----------------------------------------------------------------------

    fn parse_rpc_decl(&mut self, doc: Vec<Token>, span_token: &Token) -> Result<()> {
        let mut service = schema::Service::new();
        service.span = Some(self.get_span(span_token));
        service.documentation = Self::make_documentation(&doc);

        self.expect_ident("rpc_service")?;

        let name_tok = self.expect_ident_any()?;
        service.name = Some(name_tok.text.clone());

        self.set_namespace(&mut service);

        self.expect_kind(TokenKind::LBrace)?;

        while *self.peek_kind() != TokenKind::RBrace && !self.at_eof() {
            let method_docs = self.collect_doc_comments();
            if *self.peek_kind() == TokenKind::RBrace {
                break;
            }
            let method_span_token = if !method_docs.is_empty() {
                method_docs[0].clone()
            } else {
                self.peek().clone()
            };
            service
                .calls
                .push(self.parse_rpc_method(method_docs, &method_span_token)?);
        }

        self.expect_kind(TokenKind::RBrace)?;

        self.schema.services.push(service);
        Ok(())
    }

    fn parse_rpc_method(&mut self, doc: Vec<Token>, span_token: &Token) -> Result<schema::RpcCall> {
        let mut call = schema::RpcCall::new();
        call.span = Some(self.get_span(span_token));
        call.documentation = Self::make_documentation(&doc);

        let name_tok = self.expect_ident_any()?;
        call.name = Some(name_tok.text.clone());

        self.expect_kind(TokenKind::LParen)?;
        let param_tok = self.expect_ident_any()?;
        let mut req = schema::Object::new();
        req.name = Some(param_tok.text.clone());
        call.request = Some(req);
        self.expect_kind(TokenKind::RParen)?;

        self.expect_kind(TokenKind::Colon)?;

        let return_tok = self.expect_ident_any()?;
        let mut resp = schema::Object::new();
        resp.name = Some(return_tok.text.clone());
        call.response = Some(resp);

        call.attributes = self.parse_optional_metadata()?;

        self.expect_kind(TokenKind::Semicolon)?;

        Ok(call)
    }

    // -----------------------------------------------------------------------
    // Metadata (attributes in parentheses)
    // -----------------------------------------------------------------------

    fn parse_optional_metadata(&mut self) -> Result<Option<schema::Attributes>> {
        if *self.peek_kind() != TokenKind::LParen {
            return Ok(None);
        }
        self.advance(); // (

        let mut attrs = schema::Attributes::new();
        let mut first = true;
        while *self.peek_kind() != TokenKind::RParen && !self.at_eof() {
            if !first {
                self.expect_kind(TokenKind::Comma)?;
            }
            first = false;

            let key_tok = self.expect_ident_any()?;
            let key = key_tok.text.clone();

            let value = if *self.peek_kind() == TokenKind::Colon {
                self.advance(); // :
                Some(self.parse_single_value_text()?)
            } else {
                None
            };

            let mut entry = schema::KeyValue::new();
            entry.key = Some(key);
            entry.value = value;
            attrs.entries.push(entry);
        }

        self.expect_kind(TokenKind::RParen)?;

        Ok(Some(attrs))
    }

    /// Parse a single value and return its raw text representation.
    /// Used for metadata values where we keep the raw text (including quotes for strings).
    fn parse_single_value_text(&mut self) -> Result<String> {
        // Sign prefix
        let sign = match self.peek_kind() {
            TokenKind::Plus => {
                self.advance();
                "+"
            }
            TokenKind::Minus => {
                self.advance();
                "-"
            }
            _ => "",
        };

        match self.peek_kind() {
            TokenKind::IntLit | TokenKind::FloatLit => {
                let tok = self.advance();
                Ok(format!("{}{}", sign, tok.text))
            }
            TokenKind::StringLit => {
                let tok = self.advance();
                // Return with quotes for metadata values (matches tree-sitter behavior)
                Ok(format!("\"{}\"", tok.text))
            }
            TokenKind::Ident => {
                let tok = self.advance();
                let mut text = format!("{}{}", sign, tok.text);
                // Handle dotted identifiers
                while *self.peek_kind() == TokenKind::Dot {
                    self.advance(); // .
                    let next = self.expect_ident_any()?;
                    text.push('.');
                    text.push_str(&next.text);
                }
                Ok(text)
            }
            _ => {
                let tok = self.peek();
                Err(ParseError::SyntaxError {
                    line: tok.line as usize,
                    column: tok.col as usize,
                    context: format!("expected value, found '{}'", tok.text),
                })
            }
        }
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

/// Parse a possibly-signed integer from text (which may include +/- prefix).
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
        assert_eq!(unescape_string("\\0").unwrap(), "\0");
        assert_eq!(unescape_string("\\101").unwrap(), "A");
        assert_eq!(unescape_string("\\077").unwrap(), "?");
        assert_eq!(unescape_string("\\0x").unwrap(), "\0x");
    }

    #[test]
    fn test_unescape_string_invalid() {
        assert!(unescape_string("\\q").is_err());
    }
}
