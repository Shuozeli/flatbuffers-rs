use std::fs;
use std::process::Command;

fn without_code_whitespace(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut normalized = String::with_capacity(source.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index].is_ascii_whitespace() {
            index += 1;
            continue;
        }

        if bytes[index] == b'"' {
            copy_quoted(bytes, &mut index, &mut normalized, b'"');
            continue;
        }

        if bytes[index] == b'/' && bytes.get(index + 1) == Some(&b'/') {
            while index < bytes.len() && bytes[index] != b'\n' {
                normalized.push(bytes[index] as char);
                index += 1;
            }
            normalized.push('\n');
            continue;
        }

        if bytes[index] == b'/' && bytes.get(index + 1) == Some(&b'*') {
            while index < bytes.len() {
                normalized.push(bytes[index] as char);
                if bytes[index] == b'*' && bytes.get(index + 1) == Some(&b'/') {
                    normalized.push('/');
                    index += 2;
                    break;
                }
                index += 1;
            }
            continue;
        }

        normalized.push(bytes[index] as char);
        index += 1;
    }

    normalized
}

fn copy_quoted(bytes: &[u8], index: &mut usize, output: &mut String, quote: u8) {
    output.push(quote as char);
    *index += 1;
    while *index < bytes.len() {
        let byte = bytes[*index];
        output.push(byte as char);
        *index += 1;
        if byte == b'\\' && *index < bytes.len() {
            output.push(bytes[*index] as char);
            *index += 1;
        } else if byte == quote {
            break;
        }
    }
}

#[test]
fn default_rust_output_matches_official_flatc_except_whitespace() {
    // Arrange
    let schema = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/testdata/official_rust_parity/official_rust_parity.fbs"
    );
    let expected =
        include_str!("../testdata/official_rust_parity/official_rust_parity_generated.rs");
    let output = tempfile::tempdir().unwrap();

    // Act
    let result = Command::new(env!("CARGO_BIN_EXE_flatc"))
        .args(["-r", "-o", output.path().to_str().unwrap(), schema])
        .output()
        .unwrap();

    // Assert
    assert!(
        result.status.success(),
        "flatc-rs failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let actual =
        fs::read_to_string(output.path().join("official_rust_parity_generated.rs")).unwrap();
    assert_eq!(
        without_code_whitespace(&actual),
        without_code_whitespace(expected)
    );
}
