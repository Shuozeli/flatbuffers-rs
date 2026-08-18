//! FlatBuffers JSON text parsing and formatting.

use serde_json::Value;

/// Parse strict JSON or the relaxed JSON syntax accepted by `flatc`.
pub fn parse_json_text(source: &str, strict: bool) -> Result<Value, String> {
    if strict {
        return serde_json::from_str(source).map_err(|error| error.to_string());
    }

    let normalized = normalize_relaxed_json(source)?;
    serde_json::from_str(&normalized).map_err(|error| error.to_string())
}

/// Format a JSON value using strict JSON or the default `flatc` field style.
pub fn format_json_text(value: &Value, strict: bool) -> Result<String, String> {
    if strict {
        return serde_json::to_string_pretty(value).map_err(|error| error.to_string());
    }

    let mut output = String::new();
    write_relaxed_value(value, 0, &mut output)?;
    Ok(output)
}

fn normalize_relaxed_json(source: &str) -> Result<String, String> {
    let bytes = source.as_bytes();
    let mut output = String::with_capacity(source.len());
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'"' => copy_string(source, bytes, &mut index, &mut output)?,
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index += 2;
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index += 2;
                let mut terminated = false;
                while index + 1 < bytes.len() {
                    if bytes[index] == b'*' && bytes[index + 1] == b'/' {
                        index += 2;
                        terminated = true;
                        break;
                    }
                    index += 1;
                }
                if !terminated {
                    return Err("unterminated block comment in JSON input".to_string());
                }
            }
            b',' if next_significant(bytes, index + 1)
                .is_some_and(|next| bytes[next] == b'}' || bytes[next] == b']') =>
            {
                index += 1;
            }
            b'-' | b'+' if starts_special_float(bytes, index + 1) => {
                let start = index;
                index += 1;
                while index < bytes.len() && is_identifier_continue(bytes[index]) {
                    index += 1;
                }
                push_quoted(&source[start..index], &mut output)?;
            }
            b'0'..=b'9' | b'-' | b'+' => {
                let start = index;
                index += 1;
                while index < bytes.len() && !is_value_delimiter(bytes[index]) {
                    index += 1;
                }
                let token = &source[start..index];
                if let Some(decimal) = normalize_hex_integer(token)? {
                    output.push_str(&decimal);
                } else if let Some(unsigned) = token.strip_prefix('+') {
                    output.push_str(unsigned);
                } else {
                    output.push_str(token);
                }
            }
            byte if is_identifier_start(byte) => {
                let start = index;
                index += 1;
                while index < bytes.len() && is_identifier_continue(bytes[index]) {
                    index += 1;
                }
                let token = &source[start..index];
                if matches!(token, "true" | "false" | "null") {
                    output.push_str(token);
                } else {
                    push_quoted(token, &mut output)?;
                }
            }
            byte => {
                output.push(byte as char);
                index += 1;
            }
        }
    }

    Ok(output)
}

fn copy_string(
    source: &str,
    bytes: &[u8],
    index: &mut usize,
    output: &mut String,
) -> Result<(), String> {
    let start = *index;
    *index += 1;
    while *index < bytes.len() {
        let byte = bytes[*index];
        *index += 1;
        if byte == b'\\' && *index < bytes.len() {
            *index += 1;
        } else if byte == b'"' {
            output.push_str(&source[start..*index]);
            return Ok(());
        }
    }
    Err("unterminated string in JSON input".to_string())
}

fn next_significant(bytes: &[u8], mut index: usize) -> Option<usize> {
    loop {
        while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
            index += 1;
        }
        if bytes.get(index) == Some(&b'/') && bytes.get(index + 1) == Some(&b'/') {
            index += 2;
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
            continue;
        }
        if bytes.get(index) == Some(&b'/') && bytes.get(index + 1) == Some(&b'*') {
            index += 2;
            while index + 1 < bytes.len() && !(bytes[index] == b'*' && bytes[index + 1] == b'/') {
                index += 1;
            }
            index = (index + 2).min(bytes.len());
            continue;
        }
        return (index < bytes.len()).then_some(index);
    }
}

fn starts_special_float(bytes: &[u8], index: usize) -> bool {
    bytes
        .get(index..)
        .is_some_and(|tail| tail.starts_with(b"inf") || tail.starts_with(b"nan"))
}

fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn is_identifier_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.')
}

fn is_value_delimiter(byte: u8) -> bool {
    byte.is_ascii_whitespace() || matches!(byte, b',' | b']' | b'}' | b':')
}

fn normalize_hex_integer(token: &str) -> Result<Option<String>, String> {
    let (negative, digits) = if let Some(value) = token.strip_prefix("-0x") {
        (true, value)
    } else if let Some(value) = token.strip_prefix("0x") {
        (false, value)
    } else if let Some(value) = token.strip_prefix("+0x") {
        (false, value)
    } else {
        return Ok(None);
    };
    let magnitude = u128::from_str_radix(digits, 16)
        .map_err(|_| format!("invalid hexadecimal integer '{token}'"))?;
    Ok(Some(if negative {
        format!("-{magnitude}")
    } else {
        magnitude.to_string()
    }))
}

fn push_quoted(value: &str, output: &mut String) -> Result<(), String> {
    output.push_str(&serde_json::to_string(value).map_err(|error| error.to_string())?);
    Ok(())
}

fn write_relaxed_value(value: &Value, indent: usize, output: &mut String) -> Result<(), String> {
    match value {
        Value::Object(object) => {
            output.push('{');
            if !object.is_empty() {
                output.push('\n');
                for (position, (key, value)) in object.iter().enumerate() {
                    write_indent(indent + 2, output);
                    if is_bare_field_name(key) {
                        output.push_str(key);
                    } else {
                        push_quoted(key, output)?;
                    }
                    output.push_str(": ");
                    write_relaxed_value(value, indent + 2, output)?;
                    if position + 1 < object.len() {
                        output.push(',');
                    }
                    output.push('\n');
                }
                write_indent(indent, output);
            }
            output.push('}');
        }
        Value::Array(values) => {
            output.push('[');
            if !values.is_empty() {
                output.push('\n');
                for (position, value) in values.iter().enumerate() {
                    write_indent(indent + 2, output);
                    write_relaxed_value(value, indent + 2, output)?;
                    if position + 1 < values.len() {
                        output.push(',');
                    }
                    output.push('\n');
                }
                write_indent(indent, output);
            }
            output.push(']');
        }
        _ => output.push_str(&serde_json::to_string(value).map_err(|error| error.to_string())?),
    }
    Ok(())
}

fn write_indent(indent: usize, output: &mut String) {
    output.extend(std::iter::repeat_n(' ', indent));
}

fn is_bare_field_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes.next().is_some_and(is_identifier_start) && bytes.all(is_identifier_continue)
}

#[cfg(test)]
mod tests {
    use super::{format_json_text, parse_json_text};
    use serde_json::json;

    #[test]
    fn relaxed_parser_accepts_flatc_text_syntax() {
        // Arrange
        let source = r#"{
          // line comment
          name: "英雄",
          color: Green,
          values: [1e3, 0x10,],
          /* block comment */
        }"#;

        // Act
        let parsed = parse_json_text(source, false).unwrap();

        // Assert
        assert_eq!(
            parsed,
            json!({"name": "英雄", "color": "Green", "values": [1000.0, 16]})
        );
    }

    #[test]
    fn strict_parser_rejects_relaxed_syntax() {
        // Arrange
        let source = "{ name: \"hero\", }";

        // Act
        let parsed = parse_json_text(source, true);

        // Assert
        assert!(parsed.is_err());
    }

    #[test]
    fn formatter_switches_field_name_style() {
        // Arrange
        let value = json!({"name": "hero", "nested": {"value": 1}});

        // Act
        let relaxed = format_json_text(&value, false).unwrap();
        let strict = format_json_text(&value, true).unwrap();

        // Assert
        assert!(relaxed.contains("name: \"hero\""));
        assert!(!relaxed.contains("\"name\":"));
        assert!(strict.contains("\"name\": \"hero\""));
    }
}
