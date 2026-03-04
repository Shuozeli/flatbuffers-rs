use crate::error::{ParseError, Result};

/// Token kind produced by the FBS tokenizer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    /// Identifier (includes keywords like `table`, `enum`, `true`, `false`, `nan`, `inf`, etc.)
    Ident,
    /// Integer literal (decimal or hex, no sign).
    IntLit,
    /// Float literal (no sign, no inf/nan -- those are Ident).
    FloatLit,
    /// String literal (quotes stripped, escape sequences preserved as-is).
    StringLit,
    /// Doc comment line (`///` prefix stripped).
    DocComment,

    // Symbols
    LBrace,    // {
    RBrace,    // }
    LParen,    // (
    RParen,    // )
    LBracket,  // [
    RBracket,  // ]
    Colon,     // :
    Semicolon, // ;
    Comma,     // ,
    Eq,        // =
    Dot,       // .
    Plus,      // +
    Minus,     // -

    Eof,
}

/// A token with its text and source location.
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    /// The raw text content of the token.
    /// For StringLit: content between quotes (escape sequences preserved).
    /// For DocComment: text after `///` prefix.
    /// For Ident: the identifier text.
    /// For IntLit/FloatLit: the numeric text.
    /// For symbols: the symbol character.
    /// For Eof: empty string.
    pub text: String,
    /// 1-based line number.
    pub line: u32,
    /// 1-based column number.
    pub col: u32,
}

/// Tokenize FBS source into a vector of tokens.
pub fn tokenize(source: &str) -> Result<Vec<Token>> {
    let mut tokenizer = Tokenizer::new(source);
    tokenizer.tokenize()
}

struct Tokenizer<'src> {
    source: &'src [u8],
    pos: usize,
    line: u32,
    col: u32,
}

impl<'src> Tokenizer<'src> {
    fn new(source: &'src str) -> Self {
        Self {
            source: source.as_bytes(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            if self.pos >= self.source.len() {
                tokens.push(Token {
                    kind: TokenKind::Eof,
                    text: String::new(),
                    line: self.line,
                    col: self.col,
                });
                break;
            }
            tokens.push(self.next_token()?);
        }
        Ok(tokens)
    }

    fn peek_at(&self, offset: usize) -> Option<u8> {
        self.source.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> u8 {
        let ch = self.source[self.pos];
        self.pos += 1;
        if ch == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        ch
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip whitespace
            while self.pos < self.source.len() && self.source[self.pos].is_ascii_whitespace() {
                self.advance();
            }

            if self.pos >= self.source.len() {
                break;
            }

            // Check for comments
            if self.source[self.pos] == b'/' {
                if self.peek_at(1) == Some(b'/') {
                    // Check for doc comment (///) vs regular comment (//)
                    if self.peek_at(2) == Some(b'/') {
                        // Doc comment -- don't skip, let next_token handle it
                        break;
                    }
                    // Regular line comment -- skip to end of line
                    while self.pos < self.source.len() && self.source[self.pos] != b'\n' {
                        self.advance();
                    }
                    continue;
                } else if self.peek_at(1) == Some(b'*') {
                    // Block comment -- skip to */
                    self.advance(); // /
                    self.advance(); // *
                    loop {
                        if self.pos >= self.source.len() {
                            break;
                        }
                        if self.source[self.pos] == b'*' && self.peek_at(1) == Some(b'/') {
                            self.advance(); // *
                            self.advance(); // /
                            break;
                        }
                        self.advance();
                    }
                    continue;
                }
            }

            break;
        }
    }

    fn next_token(&mut self) -> Result<Token> {
        let line = self.line;
        let col = self.col;
        let ch = self.source[self.pos];

        // Doc comment: ///
        if ch == b'/' && self.peek_at(1) == Some(b'/') && self.peek_at(2) == Some(b'/') {
            self.advance(); // /
            self.advance(); // /
            self.advance(); // /
            let start = self.pos;
            while self.pos < self.source.len() && self.source[self.pos] != b'\n' {
                self.advance();
            }
            let text = std::str::from_utf8(&self.source[start..self.pos])
                .map_err(|_| ParseError::InvalidUtf8(start))?;
            return Ok(Token {
                kind: TokenKind::DocComment,
                text: text.to_string(),
                line,
                col,
            });
        }

        // Symbols
        match ch {
            b'{' => {
                self.advance();
                return Ok(Token {
                    kind: TokenKind::LBrace,
                    text: "{".into(),
                    line,
                    col,
                });
            }
            b'}' => {
                self.advance();
                return Ok(Token {
                    kind: TokenKind::RBrace,
                    text: "}".into(),
                    line,
                    col,
                });
            }
            b'(' => {
                self.advance();
                return Ok(Token {
                    kind: TokenKind::LParen,
                    text: "(".into(),
                    line,
                    col,
                });
            }
            b')' => {
                self.advance();
                return Ok(Token {
                    kind: TokenKind::RParen,
                    text: ")".into(),
                    line,
                    col,
                });
            }
            b'[' => {
                self.advance();
                return Ok(Token {
                    kind: TokenKind::LBracket,
                    text: "[".into(),
                    line,
                    col,
                });
            }
            b']' => {
                self.advance();
                return Ok(Token {
                    kind: TokenKind::RBracket,
                    text: "]".into(),
                    line,
                    col,
                });
            }
            b':' => {
                self.advance();
                return Ok(Token {
                    kind: TokenKind::Colon,
                    text: ":".into(),
                    line,
                    col,
                });
            }
            b';' => {
                self.advance();
                return Ok(Token {
                    kind: TokenKind::Semicolon,
                    text: ";".into(),
                    line,
                    col,
                });
            }
            b',' => {
                self.advance();
                return Ok(Token {
                    kind: TokenKind::Comma,
                    text: ",".into(),
                    line,
                    col,
                });
            }
            b'=' => {
                self.advance();
                return Ok(Token {
                    kind: TokenKind::Eq,
                    text: "=".into(),
                    line,
                    col,
                });
            }
            b'+' => {
                self.advance();
                return Ok(Token {
                    kind: TokenKind::Plus,
                    text: "+".into(),
                    line,
                    col,
                });
            }
            b'-' => {
                self.advance();
                return Ok(Token {
                    kind: TokenKind::Minus,
                    text: "-".into(),
                    line,
                    col,
                });
            }
            _ => {}
        }

        // String literal
        if ch == b'"' || ch == b'\'' {
            return self.read_string(line, col);
        }

        // Numeric literal (starts with digit or dot-followed-by-digit)
        if ch.is_ascii_digit() {
            return self.read_number(line, col);
        }
        if ch == b'.' {
            if self.peek_at(1).is_some_and(|c| c.is_ascii_digit()) {
                return self.read_number(line, col);
            }
            // Plain dot
            self.advance();
            return Ok(Token {
                kind: TokenKind::Dot,
                text: ".".into(),
                line,
                col,
            });
        }

        // Identifier
        if ch.is_ascii_alphabetic() || ch == b'_' {
            return self.read_ident(line, col);
        }

        Err(ParseError::SyntaxError {
            line: line as usize,
            column: col as usize,
            context: format!("unexpected character '{}'", ch as char),
        })
    }

    fn read_ident(&mut self, line: u32, col: u32) -> Result<Token> {
        let start = self.pos;
        while self.pos < self.source.len()
            && (self.source[self.pos].is_ascii_alphanumeric() || self.source[self.pos] == b'_')
        {
            self.advance();
        }
        let text = std::str::from_utf8(&self.source[start..self.pos])
            .map_err(|_| ParseError::InvalidUtf8(start))?;
        Ok(Token {
            kind: TokenKind::Ident,
            text: text.to_string(),
            line,
            col,
        })
    }

    fn read_number(&mut self, line: u32, col: u32) -> Result<Token> {
        let start = self.pos;
        let mut is_float = false;

        // Check for hex prefix
        if self.source[self.pos] == b'0' && self.peek_at(1).is_some_and(|c| c == b'x' || c == b'X')
        {
            self.advance(); // 0
            self.advance(); // x/X
            while self.pos < self.source.len() && self.source[self.pos].is_ascii_hexdigit() {
                self.advance();
            }
            let text = std::str::from_utf8(&self.source[start..self.pos])
                .map_err(|_| ParseError::InvalidUtf8(start))?;
            return Ok(Token {
                kind: TokenKind::IntLit,
                text: text.to_string(),
                line,
                col,
            });
        }

        // Decimal digits before dot
        while self.pos < self.source.len() && self.source[self.pos].is_ascii_digit() {
            self.advance();
        }

        // Check for dot (float)
        if self.pos < self.source.len()
            && self.source[self.pos] == b'.'
            && self
                .peek_at(1)
                .is_some_and(|c| c.is_ascii_digit() || c == b'e' || c == b'E')
        {
            is_float = true;
            self.advance(); // .
            while self.pos < self.source.len() && self.source[self.pos].is_ascii_digit() {
                self.advance();
            }
        } else if self.pos < self.source.len() && self.source[self.pos] == b'.' && start == self.pos
        // started with dot (e.g., .5)
        {
            // This case is handled by the initial dot check, but just in case
            is_float = true;
            self.advance(); // .
            while self.pos < self.source.len() && self.source[self.pos].is_ascii_digit() {
                self.advance();
            }
        } else if self.pos < self.source.len() && self.source[self.pos] == b'.' {
            // Dot followed by non-digit and non-exponent -- could be a plain dot after a number
            // e.g., `3.` or `0.` -- treat as float if we already consumed digits
            if self.pos > start {
                // We have digits before the dot -- check if this is a float like `3.0` or just `3.`
                // Look ahead: if dot is followed by nothing numeric/exponent, don't consume it
                // Actually FBS grammar: `decimals "." [decimals] [exponent]` -- so `3.` is valid float
                is_float = true;
                self.advance(); // .
                while self.pos < self.source.len() && self.source[self.pos].is_ascii_digit() {
                    self.advance();
                }
            }
        }

        // Check for exponent (float)
        if self.pos < self.source.len()
            && (self.source[self.pos] == b'e' || self.source[self.pos] == b'E')
        {
            is_float = true;
            self.advance(); // e/E
            if self.pos < self.source.len()
                && (self.source[self.pos] == b'+' || self.source[self.pos] == b'-')
            {
                self.advance(); // +/-
            }
            while self.pos < self.source.len() && self.source[self.pos].is_ascii_digit() {
                self.advance();
            }
        }

        let text = std::str::from_utf8(&self.source[start..self.pos])
            .map_err(|_| ParseError::InvalidUtf8(start))?;
        Ok(Token {
            kind: if is_float {
                TokenKind::FloatLit
            } else {
                TokenKind::IntLit
            },
            text: text.to_string(),
            line,
            col,
        })
    }

    fn read_string(&mut self, line: u32, col: u32) -> Result<Token> {
        let quote = self.advance(); // opening quote
        let mut content = String::new();
        let is_single_quote = quote == b'\'';

        loop {
            if self.pos >= self.source.len() {
                return Err(ParseError::SyntaxError {
                    line: line as usize,
                    column: col as usize,
                    context: "unterminated string literal".into(),
                });
            }
            let ch = self.source[self.pos];
            if ch == quote {
                self.advance(); // closing quote
                break;
            }
            if ch == b'\\' && !is_single_quote {
                // Escape sequence -- preserve as-is in the token text
                content.push(self.advance() as char); // backslash
                if self.pos < self.source.len() {
                    content.push(self.advance() as char); // escaped char
                                                          // For multi-char escapes (hex, unicode, octal), consume the rest
                    let escaped = content.as_bytes()[content.len() - 1];
                    match escaped {
                        b'x' => {
                            // \xHH
                            for _ in 0..2 {
                                if self.pos < self.source.len()
                                    && self.source[self.pos].is_ascii_hexdigit()
                                {
                                    content.push(self.advance() as char);
                                }
                            }
                        }
                        b'u' => {
                            // \uHHHH
                            for _ in 0..4 {
                                if self.pos < self.source.len()
                                    && self.source[self.pos].is_ascii_hexdigit()
                                {
                                    content.push(self.advance() as char);
                                }
                            }
                        }
                        b'U' => {
                            // \UHHHHHHHH
                            for _ in 0..8 {
                                if self.pos < self.source.len()
                                    && self.source[self.pos].is_ascii_hexdigit()
                                {
                                    content.push(self.advance() as char);
                                }
                            }
                        }
                        b'0'..=b'7' => {
                            // Octal: up to 2 more digits
                            for _ in 0..2 {
                                if self.pos < self.source.len()
                                    && self.source[self.pos] >= b'0'
                                    && self.source[self.pos] <= b'7'
                                {
                                    content.push(self.advance() as char);
                                } else {
                                    break;
                                }
                            }
                        }
                        _ => {
                            // Simple escape like \n, \t, \\, \", etc.
                        }
                    }
                }
            } else if ch == b'\n' {
                // Newlines inside strings -- advance tracking but include in content
                content.push(self.advance() as char);
            } else {
                content.push(self.advance() as char);
            }
        }

        Ok(Token {
            kind: TokenKind::StringLit,
            text: content,
            line,
            col,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let tokens = tokenize("table Foo { x: int; }").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Ident);
        assert_eq!(tokens[0].text, "table");
        assert_eq!(tokens[1].kind, TokenKind::Ident);
        assert_eq!(tokens[1].text, "Foo");
        assert_eq!(tokens[2].kind, TokenKind::LBrace);
        assert_eq!(tokens[3].kind, TokenKind::Ident);
        assert_eq!(tokens[3].text, "x");
        assert_eq!(tokens[4].kind, TokenKind::Colon);
        assert_eq!(tokens[5].kind, TokenKind::Ident);
        assert_eq!(tokens[5].text, "int");
        assert_eq!(tokens[6].kind, TokenKind::Semicolon);
        assert_eq!(tokens[7].kind, TokenKind::RBrace);
        assert_eq!(tokens[8].kind, TokenKind::Eof);
    }

    #[test]
    fn test_doc_comment() {
        let tokens = tokenize("/// hello world\ntable T {}").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::DocComment);
        assert_eq!(tokens[0].text, " hello world");
        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[0].col, 1);
        assert_eq!(tokens[1].kind, TokenKind::Ident);
        assert_eq!(tokens[1].text, "table");
        assert_eq!(tokens[1].line, 2);
    }

    #[test]
    fn test_skip_regular_comment() {
        let tokens = tokenize("// comment\ntable T {}").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Ident);
        assert_eq!(tokens[0].text, "table");
    }

    #[test]
    fn test_skip_block_comment() {
        let tokens = tokenize("/* block */table T {}").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Ident);
        assert_eq!(tokens[0].text, "table");
    }

    #[test]
    fn test_numbers() {
        let tokens = tokenize("42 0xFF 3.14 1e10 .5").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::IntLit);
        assert_eq!(tokens[0].text, "42");
        assert_eq!(tokens[1].kind, TokenKind::IntLit);
        assert_eq!(tokens[1].text, "0xFF");
        assert_eq!(tokens[2].kind, TokenKind::FloatLit);
        assert_eq!(tokens[2].text, "3.14");
        assert_eq!(tokens[3].kind, TokenKind::FloatLit);
        assert_eq!(tokens[3].text, "1e10");
        assert_eq!(tokens[4].kind, TokenKind::FloatLit);
        assert_eq!(tokens[4].text, ".5");
    }

    #[test]
    fn test_string_literal() {
        let tokens = tokenize(r#""hello \"world\"""#).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::StringLit);
        assert_eq!(tokens[0].text, r#"hello \"world\""#);
    }

    #[test]
    fn test_single_quoted_string() {
        let tokens = tokenize("'hello'").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::StringLit);
        assert_eq!(tokens[0].text, "hello");
    }

    #[test]
    fn test_positions() {
        let tokens = tokenize("table Foo {\n  x: int;\n}").unwrap();
        // table at (1,1)
        assert_eq!((tokens[0].line, tokens[0].col), (1, 1));
        // Foo at (1,7)
        assert_eq!((tokens[1].line, tokens[1].col), (1, 7));
        // { at (1,11)
        assert_eq!((tokens[2].line, tokens[2].col), (1, 11));
        // x at (2,3)
        assert_eq!((tokens[3].line, tokens[3].col), (2, 3));
        // : at (2,4)
        assert_eq!((tokens[4].line, tokens[4].col), (2, 4));
        // int at (2,6)
        assert_eq!((tokens[5].line, tokens[5].col), (2, 6));
    }

    #[test]
    fn test_dot_vs_float() {
        let tokens = tokenize("Foo.Bar .5 3.14").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Ident);
        assert_eq!(tokens[0].text, "Foo");
        assert_eq!(tokens[1].kind, TokenKind::Dot);
        assert_eq!(tokens[2].kind, TokenKind::Ident);
        assert_eq!(tokens[2].text, "Bar");
        assert_eq!(tokens[3].kind, TokenKind::FloatLit);
        assert_eq!(tokens[3].text, ".5");
        assert_eq!(tokens[4].kind, TokenKind::FloatLit);
        assert_eq!(tokens[4].text, "3.14");
    }
}
