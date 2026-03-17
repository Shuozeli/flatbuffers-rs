use crate::region::{AnnotatedRegion, RegionType};

/// Format annotated regions into the C++ flatc `.afb` text format.
///
/// Output matches:
/// ```text
/// +0x0000 | 44 00 00 00             | UOffset32  | 0x00000044 (68) Loc: 0x0044   | offset to root table `Foo`
/// ```
pub fn format_afb(
    binary: &[u8],
    regions: &[AnnotatedRegion],
    schema_file: &str,
    binary_file: &str,
) -> String {
    let mut out = String::new();

    // Header comment
    out.push_str("// Annotated Flatbuffer Binary\n");
    out.push_str("//\n");
    out.push_str(&format!("// Schema file: {schema_file}\n"));
    out.push_str(&format!("// Binary file: {binary_file}\n"));
    out.push('\n');

    // Sort regions by byte offset for linear output
    let mut sorted: Vec<usize> = (0..regions.len()).collect();
    sorted.sort_by_key(|&i| (regions[i].byte_range.start, regions[i].byte_range.end));

    // Group output by section type
    let mut current_section: Option<String> = None;

    for &idx in &sorted {
        let region = &regions[idx];

        // Skip container regions (VTable, StructInline) -- their children carry the data
        if matches!(
            region.region_type,
            RegionType::VTable { .. } | RegionType::StructInline { .. }
        ) {
            // Emit section header for vtable/struct
            let section_name = match &region.region_type {
                RegionType::VTable { type_name } => format!("vtable ({type_name})"),
                RegionType::StructInline { type_name } => format!("struct ({type_name})"),
                _ => unreachable!(),
            };
            if current_section.as_deref() != Some(&section_name) {
                out.push('\n');
                out.push_str(&format!("{section_name}:\n"));
                current_section = Some(section_name);
            }
            continue;
        }

        // Determine section header based on region type
        let section = section_for_region(region);
        if current_section.as_deref() != Some(&section) {
            out.push('\n');
            out.push_str(&format!("{section}:\n"));
            current_section = Some(section);
        }

        // Format the line
        let line = format_region_line(binary, region);
        out.push_str(&format!("  {line}\n"));
    }

    out
}

fn section_for_region(region: &AnnotatedRegion) -> String {
    match &region.region_type {
        RegionType::RootOffset | RegionType::FileIdentifier => "header".to_string(),
        RegionType::VTableSize | RegionType::VTableTableSize | RegionType::VTableEntry { .. } => {
            // Use field_path to find the type name
            if let Some(type_name) = region.field_path.first() {
                format!("vtable ({type_name})")
            } else {
                "vtable".to_string()
            }
        }
        RegionType::TableSOffset { type_name } => {
            if region.depth == 0 {
                format!("root_table ({type_name})")
            } else {
                format!("table ({type_name})")
            }
        }
        RegionType::ScalarField { .. }
        | RegionType::UnionTypeField { .. }
        | RegionType::UnionDataOffset { .. }
        | RegionType::StringOffset { .. }
        | RegionType::VectorOffset { .. } => {
            if let Some(type_name) = region.field_path.first() {
                format!("table ({type_name})")
            } else {
                "table".to_string()
            }
        }
        RegionType::StructField { .. } => {
            if let Some(type_name) = region.field_path.first() {
                format!("table ({type_name})")
            } else {
                "struct".to_string()
            }
        }
        RegionType::StringLength | RegionType::StringData { .. } | RegionType::StringTerminator => {
            let field = region
                .field_path
                .iter()
                .skip(1)
                .cloned()
                .collect::<Vec<_>>()
                .join(".");
            let type_name = region.field_path.first().cloned().unwrap_or_default();
            if field.is_empty() {
                format!("string ({type_name})")
            } else {
                format!("string ({type_name}.{field})")
            }
        }
        RegionType::VectorLength | RegionType::VectorElement { .. } => {
            let field = region
                .field_path
                .iter()
                .skip(1)
                .cloned()
                .collect::<Vec<_>>()
                .join(".");
            let type_name = region.field_path.first().cloned().unwrap_or_default();
            if field.is_empty() {
                format!("vector ({type_name})")
            } else {
                format!("vector ({type_name}.{field})")
            }
        }
        RegionType::Padding => "padding".to_string(),
        _ => "unknown".to_string(),
    }
}

fn format_region_line(binary: &[u8], region: &AnnotatedRegion) -> String {
    let start = region.byte_range.start;
    let end = region.byte_range.end;
    let size = end - start;

    // Hex bytes column (up to 16 bytes shown)
    let hex_bytes: String = if size <= 16 {
        binary[start..end]
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        let shown: String = binary[start..start + 16]
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect::<Vec<_>>()
            .join(" ");
        format!("{shown} ...")
    };

    // Type column
    let type_str = type_string_for_region(region, size);

    // Value column
    let value_str = value_string_for_region(binary, region, start, size);

    // Description column
    let desc = description_for_region(region);

    format!("+0x{start:04X} | {hex_bytes:<24}| {type_str:<11}| {value_str:<30}| {desc}",)
}

fn type_string_for_region(region: &AnnotatedRegion, size: usize) -> String {
    use flatc_rs_schema::BaseType;
    match &region.region_type {
        RegionType::RootOffset => "UOffset32".to_string(),
        RegionType::FileIdentifier => "char[4]".to_string(),
        RegionType::VTableSize | RegionType::VTableTableSize => "uint16_t".to_string(),
        RegionType::VTableEntry { .. } => "VOffset16".to_string(),
        RegionType::TableSOffset { .. } => "SOffset32".to_string(),
        RegionType::ScalarField { base_type, .. } | RegionType::StructField { base_type, .. } => {
            match base_type {
                BaseType::BASE_TYPE_BOOL => "uint8_t".to_string(),
                BaseType::BASE_TYPE_BYTE => "int8_t".to_string(),
                BaseType::BASE_TYPE_U_BYTE => "uint8_t".to_string(),
                BaseType::BASE_TYPE_SHORT => "int16_t".to_string(),
                BaseType::BASE_TYPE_U_SHORT => "uint16_t".to_string(),
                BaseType::BASE_TYPE_INT => "int32_t".to_string(),
                BaseType::BASE_TYPE_U_INT => "uint32_t".to_string(),
                BaseType::BASE_TYPE_LONG => "int64_t".to_string(),
                BaseType::BASE_TYPE_U_LONG => "uint64_t".to_string(),
                BaseType::BASE_TYPE_FLOAT => "float".to_string(),
                BaseType::BASE_TYPE_DOUBLE => "double".to_string(),
                _ => format!("{size}B"),
            }
        }
        RegionType::UnionTypeField { .. } => "UType8".to_string(),
        RegionType::StringOffset { .. }
        | RegionType::VectorOffset { .. }
        | RegionType::UnionDataOffset { .. } => "UOffset32".to_string(),
        RegionType::StringLength | RegionType::VectorLength => "uint32_t".to_string(),
        RegionType::StringData { .. } => format!("char[{size}]"),
        RegionType::StringTerminator => "char".to_string(),
        RegionType::VectorElement { .. } => format!("{size}B"),
        RegionType::Padding => format!("uint8_t[{size}]"),
        _ => format!("{size}B"),
    }
}

fn value_string_for_region(
    binary: &[u8],
    region: &AnnotatedRegion,
    start: usize,
    size: usize,
) -> String {
    match &region.region_type {
        RegionType::RootOffset => {
            if size == 4 && start + 4 <= binary.len() {
                let val = u32::from_le_bytes([
                    binary[start],
                    binary[start + 1],
                    binary[start + 2],
                    binary[start + 3],
                ]);
                let loc = start + val as usize;
                format!("0x{val:08X} ({val}) Loc: 0x{loc:04X}")
            } else {
                region.value_display.clone()
            }
        }
        RegionType::FileIdentifier => {
            let bytes = &binary[start..start + size.min(4)];
            String::from_utf8_lossy(bytes).into_owned()
        }
        RegionType::TableSOffset { .. } => {
            if size == 4 && start + 4 <= binary.len() {
                let val = i32::from_le_bytes([
                    binary[start],
                    binary[start + 1],
                    binary[start + 2],
                    binary[start + 3],
                ]);
                let loc = (start as i64 - val as i64) as usize;
                format!("0x{:08X} ({val}) Loc: 0x{loc:04X}", val as u32)
            } else {
                region.value_display.clone()
            }
        }
        RegionType::VTableSize | RegionType::VTableTableSize => {
            if size == 2 && start + 2 <= binary.len() {
                let val = u16::from_le_bytes([binary[start], binary[start + 1]]);
                format!("0x{val:04X} ({val})")
            } else {
                region.value_display.clone()
            }
        }
        RegionType::VTableEntry { .. } => {
            if size == 2 && start + 2 <= binary.len() {
                let val = u16::from_le_bytes([binary[start], binary[start + 1]]);
                format!("0x{val:04X} ({val})")
            } else {
                region.value_display.clone()
            }
        }
        RegionType::StringOffset { .. }
        | RegionType::VectorOffset { .. }
        | RegionType::UnionDataOffset { .. } => {
            if size == 4 && start + 4 <= binary.len() {
                let val = u32::from_le_bytes([
                    binary[start],
                    binary[start + 1],
                    binary[start + 2],
                    binary[start + 3],
                ]);
                let loc = start + val as usize;
                format!("0x{val:08X} ({val}) Loc: 0x{loc:04X}")
            } else {
                region.value_display.clone()
            }
        }
        RegionType::Padding => {
            let bytes = &binary[start..start + size.min(16)];
            let s: String = bytes
                .iter()
                .map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' })
                .collect();
            s
        }
        RegionType::StringData { .. } => {
            let bytes = &binary[start..start + size.min(40)];
            let s = String::from_utf8_lossy(bytes);
            if size > 40 {
                format!("{s}...")
            } else {
                s.to_string()
            }
        }
        _ => region.value_display.clone(),
    }
}

fn description_for_region(region: &AnnotatedRegion) -> String {
    match &region.region_type {
        RegionType::RootOffset => {
            let root_type = region.field_path.first().cloned().unwrap_or_default();
            format!("offset to root table `{root_type}`")
        }
        RegionType::FileIdentifier => "File Identifier".to_string(),
        RegionType::VTableSize => "size of this vtable".to_string(),
        RegionType::VTableTableSize => "size of referring table".to_string(),
        RegionType::VTableEntry {
            field_name,
            field_id,
        } => {
            format!("offset to field `{field_name}` (id: {field_id})")
        }
        RegionType::TableSOffset { type_name } => {
            format!("offset to vtable for `{type_name}`")
        }
        RegionType::ScalarField {
            field_name,
            base_type,
        } => {
            format!("table field `{field_name}` ({base_type:?})")
        }
        RegionType::StructField {
            field_name: _,
            base_type,
        } => {
            let path = region.field_path.join(".");
            format!("struct field `{path}` ({base_type:?})")
        }
        RegionType::UnionTypeField { field_name } => {
            format!("table field `{field_name}` (UType)")
        }
        RegionType::StringOffset { field_name } => {
            format!("offset to field `{field_name}` (string)")
        }
        RegionType::VectorOffset { field_name } => {
            format!("offset to field `{field_name}` (vector)")
        }
        RegionType::UnionDataOffset { field_name } => {
            format!("offset to field `{field_name}` (union)")
        }
        RegionType::StringLength => "length of string".to_string(),
        RegionType::StringData { field_name } => {
            format!("string data `{field_name}`")
        }
        RegionType::StringTerminator => "null terminator".to_string(),
        RegionType::VectorLength => "length of vector".to_string(),
        RegionType::VectorElement { index } => format!("element [{index}]"),
        RegionType::Padding => "padding".to_string(),
        _ => region.label.clone(),
    }
}
