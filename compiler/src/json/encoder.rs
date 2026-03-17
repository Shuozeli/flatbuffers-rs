//! Encode a JSON value into a FlatBuffers binary using a compiled Schema.

use flatc_rs_schema::resolved::{
    ResolvedEnum, ResolvedField, ResolvedObject, ResolvedSchema, ResolvedType,
};
use flatc_rs_schema::BaseType;
use serde_json::Value;

use super::error::{is_scalar, json_type_name, scalar_byte_size, JsonError};

const MAX_DEPTH: usize = 64;

/// Options controlling JSON -> binary encoding.
#[derive(Debug, Clone, Default)]
pub struct EncoderOptions {
    /// When true, silently skip JSON fields not present in the schema.
    /// When false (default), error on unknown fields.
    pub unknown_json: bool,
    /// When true, emit fields even when they equal the default value.
    /// Our encoder always writes all fields present in JSON, so this is
    /// effectively always true. Accepted for C++ flatc CLI compatibility.
    pub force_defaults: bool,
}

/// Encode a JSON value into a FlatBuffers binary.
///
/// `json` must be a JSON object representing the root table.
/// `schema` is the compiled FlatBuffers schema.
/// `root_type` is the name of the root table type.
pub fn json_to_binary(
    json: &Value,
    schema: &ResolvedSchema,
    root_type: &str,
) -> Result<Vec<u8>, JsonError> {
    json_to_binary_with_opts(json, schema, root_type, &EncoderOptions::default())
}

/// Encode a JSON value into a FlatBuffers binary with options.
pub fn json_to_binary_with_opts(
    json: &Value,
    schema: &ResolvedSchema,
    root_type: &str,
    opts: &EncoderOptions,
) -> Result<Vec<u8>, JsonError> {
    let mut enc = Encoder::new(schema, opts);
    enc.encode(json, root_type)
}

// ---------------------------------------------------------------------------
// Encoder
// ---------------------------------------------------------------------------

struct Encoder<'a> {
    schema: &'a ResolvedSchema,
    opts: &'a EncoderOptions,
    buf: Vec<u8>,
    object_index: std::collections::HashMap<&'a str, usize>,
}

impl<'a> Encoder<'a> {
    fn new(schema: &'a ResolvedSchema, opts: &'a EncoderOptions) -> Self {
        let mut object_index = std::collections::HashMap::new();
        for (i, obj) in schema.objects.iter().enumerate() {
            let name = obj.name.as_str();
            object_index.entry(name).or_insert(i);
            if let Some(short) = name.rsplit('.').next() {
                if short != name {
                    object_index.entry(short).or_insert(i);
                }
            }
        }
        Self {
            schema,
            opts,
            buf: Vec::with_capacity(256),
            object_index,
        }
    }

    fn encode(&mut self, json: &Value, root_type: &str) -> Result<Vec<u8>, JsonError> {
        let root_idx = self.find_object_index(root_type)?;

        // Reserve space for root offset (4 bytes)
        self.write_u32_le(0); // placeholder

        // Optionally write file identifier
        if let Some(ref ident) = self.schema.file_ident {
            let bytes = ident.as_bytes();
            for i in 0..4 {
                self.buf.push(if i < bytes.len() { bytes[i] } else { 0 });
            }
        }

        // Encode the root table
        let root_offset = self.encode_table(json, root_idx, 0)?;

        // Patch root offset
        self.patch_u32_le(0, root_offset as u32);

        Ok(std::mem::take(&mut self.buf))
    }

    // -------------------------------------------------------------------
    // Schema lookup helpers
    // -------------------------------------------------------------------

    fn find_object_index(&self, name: &str) -> Result<usize, JsonError> {
        // If the name matches the schema's root_table, prefer using root_table_index
        if let Some(idx) = self.schema.root_table_index {
            if self.schema.objects[idx].name == name {
                return Ok(idx);
            }
        }

        self.object_index
            .get(name)
            .copied()
            .ok_or_else(|| JsonError::RootTypeNotFound {
                name: name.to_string(),
            })
    }

    fn get_object(&self, idx: usize) -> Result<&ResolvedObject, JsonError> {
        self.schema
            .objects
            .get(idx)
            .ok_or(JsonError::ObjectIndexOutOfRange {
                index: idx,
                count: self.schema.objects.len(),
            })
    }

    fn get_enum(&self, idx: usize) -> Result<&ResolvedEnum, JsonError> {
        self.schema
            .enums
            .get(idx)
            .ok_or(JsonError::EnumIndexOutOfRange {
                index: idx,
                count: self.schema.enums.len(),
            })
    }

    // -------------------------------------------------------------------
    // Buffer helpers
    // -------------------------------------------------------------------

    fn align(&mut self, alignment: usize) {
        while !self.buf.len().is_multiple_of(alignment) {
            self.buf.push(0);
        }
    }

    fn write_u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    fn write_u16_le(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    fn write_u32_le(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    fn write_i32_le(&mut self, v: i32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    fn write_bytes(&mut self, data: &[u8]) {
        self.buf.extend_from_slice(data);
    }

    fn patch_u32_le(&mut self, pos: usize, v: u32) {
        self.buf[pos..pos + 4].copy_from_slice(&v.to_le_bytes());
    }

    // -------------------------------------------------------------------
    // Table encoding
    // -------------------------------------------------------------------

    fn encode_table(
        &mut self,
        json: &Value,
        obj_idx: usize,
        depth: usize,
    ) -> Result<usize, JsonError> {
        if depth > MAX_DEPTH {
            return Err(JsonError::MaxDepthExceeded { max: MAX_DEPTH });
        }

        let obj = self.get_object(obj_idx)?.clone();
        let type_name = obj.name.clone();

        let json_obj = json.as_object().ok_or_else(|| JsonError::ExpectedObject {
            type_name: type_name.clone(),
            actual: json_type_name(json),
        })?;

        // Check for unknown JSON fields (unless --unknown-json is set)
        if !self.opts.unknown_json {
            let schema_field_names: std::collections::HashSet<&str> =
                obj.fields.iter().map(|f| f.name.as_str()).collect();
            // Also include companion _type fields for unions
            let union_type_names: Vec<String> = obj
                .fields
                .iter()
                .filter(|f| f.type_.base_type == BaseType::BASE_TYPE_UNION)
                .map(|f| format!("{}_type", f.name))
                .collect();
            for key in json_obj.keys() {
                if !schema_field_names.contains(key.as_str())
                    && !union_type_names.iter().any(|n| n == key)
                {
                    return Err(JsonError::UnknownField {
                        field_name: key.clone(),
                        type_name: type_name.clone(),
                    });
                }
            }
        }

        // Sort fields by id for vtable construction
        let mut fields: Vec<ResolvedField> = obj.fields.clone();
        fields.sort_by_key(|f| f.id.unwrap_or(0));

        let max_field_id = fields
            .iter()
            .map(|f| f.id.unwrap_or(0) as usize)
            .max()
            .unwrap_or(0);
        let num_vtable_entries = max_field_id + 1;

        // Determine which fields are present in JSON and compute table layout
        struct FieldSlot {
            field: ResolvedField,
            field_id: usize,
            present: bool,
            size: usize,
            alignment: usize,
        }

        let mut slots: Vec<FieldSlot> = Vec::new();
        for field in &fields {
            let field_id = field.id.unwrap_or(0) as usize;
            let fname = &field.name;
            let ty = &field.type_;
            let bt = ty.base_type;

            let present = json_obj.contains_key(fname);
            let (size, alignment) = field_inline_size(bt, ty, self.schema);

            slots.push(FieldSlot {
                field: field.clone(),
                field_id,
                present,
                size,
                alignment,
            });
        }

        // Compute table data layout: soffset (4 bytes) + field data
        let mut table_data_size: usize = 4;
        let mut field_offsets_in_table: Vec<u16> = vec![0; num_vtable_entries];

        for slot in &slots {
            if !slot.present {
                continue;
            }
            if slot.alignment > 0 {
                while !table_data_size.is_multiple_of(slot.alignment) {
                    table_data_size += 1;
                }
            }
            field_offsets_in_table[slot.field_id] = table_data_size as u16;
            table_data_size += slot.size;
        }

        // Align table_data_size to 4
        while !table_data_size.is_multiple_of(4) {
            table_data_size += 1;
        }

        // Write vtable
        let vtable_size: u16 = (4 + num_vtable_entries * 2) as u16;
        self.align(2);
        let vtable_pos = self.buf.len();
        self.write_u16_le(vtable_size);
        self.write_u16_le(table_data_size as u16);
        for offset in field_offsets_in_table.iter().take(num_vtable_entries) {
            self.write_u16_le(*offset);
        }

        // Align to 4 before table data
        self.align(4);
        let table_pos = self.buf.len();

        // Write soffset (table_pos - vtable_pos, as i32)
        let soffset = (table_pos as i32) - (vtable_pos as i32);
        self.write_i32_le(soffset);

        // Write inline field data (with placeholders for offset types)
        let mut deferred: Vec<(usize, ResolvedField)> = Vec::new();

        // Pre-allocate table data area
        let table_data_start = table_pos;
        let remaining = table_data_size - 4;
        self.buf.resize(self.buf.len() + remaining, 0);

        // Write inline field values
        for slot in &slots {
            if !slot.present {
                continue;
            }

            let fname = &slot.field.name;
            let json_val = &json_obj[fname];
            let ty = &slot.field.type_;
            let bt = ty.base_type;
            let field_pos = table_data_start + field_offsets_in_table[slot.field_id] as usize;

            match bt {
                BaseType::BASE_TYPE_BOOL
                | BaseType::BASE_TYPE_BYTE
                | BaseType::BASE_TYPE_U_BYTE
                | BaseType::BASE_TYPE_SHORT
                | BaseType::BASE_TYPE_U_SHORT
                | BaseType::BASE_TYPE_INT
                | BaseType::BASE_TYPE_U_INT
                | BaseType::BASE_TYPE_LONG
                | BaseType::BASE_TYPE_U_LONG
                | BaseType::BASE_TYPE_FLOAT
                | BaseType::BASE_TYPE_DOUBLE => {
                    let bytes = self.encode_scalar_value(json_val, bt, ty, fname)?;
                    self.buf[field_pos..field_pos + bytes.len()].copy_from_slice(&bytes);
                }

                BaseType::BASE_TYPE_U_TYPE => {
                    let enum_idx = require_type_index(ty, fname)?;
                    let val = self.resolve_enum_value(json_val, enum_idx, fname)?;
                    self.buf[field_pos] = val as u8;
                }

                BaseType::BASE_TYPE_STRUCT => {
                    let inner_idx = require_type_index(ty, fname)?;
                    let bytes = self.encode_struct_inline(json_val, inner_idx, fname, depth + 1)?;
                    self.buf[field_pos..field_pos + bytes.len()].copy_from_slice(&bytes);
                }

                BaseType::BASE_TYPE_STRING
                | BaseType::BASE_TYPE_TABLE
                | BaseType::BASE_TYPE_VECTOR
                | BaseType::BASE_TYPE_UNION => {
                    deferred.push((slot.field_id, slot.field.clone()));
                }

                _ => {}
            }
        }

        // Serialize deferred children and patch uoffsets
        for (field_id, field) in &deferred {
            let fname = &field.name;
            let json_val = &json_obj[fname];
            let ty = &field.type_;
            let bt = ty.base_type;
            let field_pos = table_data_start + field_offsets_in_table[*field_id] as usize;

            match bt {
                BaseType::BASE_TYPE_STRING => {
                    let s = json_val.as_str().ok_or_else(|| JsonError::ExpectedString {
                        field_name: fname.clone(),
                        actual: json_type_name(json_val),
                    })?;
                    let target = self.encode_string(s);
                    let uoffset = (target - field_pos) as u32;
                    self.patch_u32_le(field_pos, uoffset);
                }

                BaseType::BASE_TYPE_TABLE => {
                    let inner_idx = require_type_index(ty, fname)?;
                    let target = self.encode_table(json_val, inner_idx, depth + 1)?;
                    let uoffset = (target - field_pos) as u32;
                    self.patch_u32_le(field_pos, uoffset);
                }

                BaseType::BASE_TYPE_VECTOR => {
                    let target = self.encode_vector(json_val, ty, fname, depth + 1)?;
                    let uoffset = (target - field_pos) as u32;
                    self.patch_u32_le(field_pos, uoffset);
                }

                BaseType::BASE_TYPE_UNION => {
                    let enum_idx = require_type_index(ty, fname)?;
                    let type_field_name = format!("{fname}_type");
                    let disc_val = json_obj.get(&type_field_name).ok_or_else(|| {
                        JsonError::MissingUnionType {
                            field_name: fname.clone(),
                        }
                    })?;
                    let discriminant =
                        self.resolve_enum_value(disc_val, enum_idx, &type_field_name)?;

                    if discriminant != 0 {
                        let target = self.encode_union_data(
                            json_val,
                            enum_idx,
                            discriminant as u8,
                            fname,
                            depth + 1,
                        )?;
                        let uoffset = (target - field_pos) as u32;
                        self.patch_u32_le(field_pos, uoffset);
                    }
                }

                _ => {}
            }
        }

        Ok(table_pos)
    }

    // -------------------------------------------------------------------
    // Scalar encoding
    // -------------------------------------------------------------------

    fn encode_scalar_value(
        &self,
        json_val: &Value,
        bt: BaseType,
        ty: &ResolvedType,
        field_name: &str,
    ) -> Result<Vec<u8>, JsonError> {
        // Check if this is an enum type
        let enum_idx = ty.index;
        if let Some(idx) = enum_idx {
            if (idx as usize) < self.schema.enums.len() {
                if let Value::String(_) = json_val {
                    let val = self.resolve_enum_value(json_val, idx as usize, field_name)?;
                    return self.integer_to_bytes(val, bt, field_name);
                }
            }
        }

        match bt {
            BaseType::BASE_TYPE_BOOL => {
                let v = match json_val {
                    Value::Bool(b) => *b,
                    Value::Number(n) => n.as_u64().unwrap_or(0) != 0,
                    _ => {
                        return Err(JsonError::ExpectedNumber {
                            field_name: field_name.to_string(),
                            base_type: format!("{bt:?}"),
                            actual: json_type_name(json_val),
                        });
                    }
                };
                Ok(vec![if v { 1 } else { 0 }])
            }

            BaseType::BASE_TYPE_BYTE => {
                let v = json_as_i64(json_val, field_name, bt)? as i8;
                Ok(v.to_le_bytes().to_vec())
            }
            BaseType::BASE_TYPE_U_BYTE => {
                let v = json_as_u64(json_val, field_name, bt)? as u8;
                Ok(v.to_le_bytes().to_vec())
            }
            BaseType::BASE_TYPE_SHORT => {
                let v = json_as_i64(json_val, field_name, bt)? as i16;
                Ok(v.to_le_bytes().to_vec())
            }
            BaseType::BASE_TYPE_U_SHORT => {
                let v = json_as_u64(json_val, field_name, bt)? as u16;
                Ok(v.to_le_bytes().to_vec())
            }
            BaseType::BASE_TYPE_INT => {
                let v = json_as_i64(json_val, field_name, bt)? as i32;
                Ok(v.to_le_bytes().to_vec())
            }
            BaseType::BASE_TYPE_U_INT => {
                let v = json_as_u64(json_val, field_name, bt)? as u32;
                Ok(v.to_le_bytes().to_vec())
            }
            BaseType::BASE_TYPE_LONG => {
                let v = json_as_i64(json_val, field_name, bt)?;
                Ok(v.to_le_bytes().to_vec())
            }
            BaseType::BASE_TYPE_U_LONG => {
                let v = json_as_u64(json_val, field_name, bt)?;
                Ok(v.to_le_bytes().to_vec())
            }
            BaseType::BASE_TYPE_FLOAT => {
                let v = json_as_f64(json_val, field_name, bt)? as f32;
                Ok(v.to_le_bytes().to_vec())
            }
            BaseType::BASE_TYPE_DOUBLE => {
                let v = json_as_f64(json_val, field_name, bt)?;
                Ok(v.to_le_bytes().to_vec())
            }

            _ => Ok(vec![0; scalar_byte_size(bt)]),
        }
    }

    fn integer_to_bytes(
        &self,
        val: i64,
        bt: BaseType,
        field_name: &str,
    ) -> Result<Vec<u8>, JsonError> {
        match bt {
            BaseType::BASE_TYPE_BOOL => Ok(vec![if val != 0 { 1 } else { 0 }]),
            BaseType::BASE_TYPE_BYTE => Ok((val as i8).to_le_bytes().to_vec()),
            BaseType::BASE_TYPE_U_BYTE => Ok((val as u8).to_le_bytes().to_vec()),
            BaseType::BASE_TYPE_SHORT => Ok((val as i16).to_le_bytes().to_vec()),
            BaseType::BASE_TYPE_U_SHORT => Ok((val as u16).to_le_bytes().to_vec()),
            BaseType::BASE_TYPE_INT => Ok((val as i32).to_le_bytes().to_vec()),
            BaseType::BASE_TYPE_U_INT => Ok((val as u32).to_le_bytes().to_vec()),
            BaseType::BASE_TYPE_LONG => Ok(val.to_le_bytes().to_vec()),
            BaseType::BASE_TYPE_U_LONG => Ok((val as u64).to_le_bytes().to_vec()),
            _ => Err(JsonError::NumberOutOfRange {
                field_name: field_name.to_string(),
                base_type: format!("{bt:?}"),
                value: val.to_string(),
            }),
        }
    }

    // -------------------------------------------------------------------
    // Enum resolution
    // -------------------------------------------------------------------

    fn resolve_enum_value(
        &self,
        json_val: &Value,
        enum_idx: usize,
        field_name: &str,
    ) -> Result<i64, JsonError> {
        let enum_def = self.get_enum(enum_idx)?;
        let enum_name = enum_def.name.clone();

        match json_val {
            Value::String(s) => {
                for ev in &enum_def.values {
                    if ev.name == *s {
                        return Ok(ev.value);
                    }
                }
                Err(JsonError::UnknownEnumValue {
                    enum_name,
                    value: s.clone(),
                })
            }
            Value::Number(n) => Ok(n.as_i64().unwrap_or(0)),
            _ => Err(JsonError::ExpectedNumber {
                field_name: field_name.to_string(),
                base_type: format!("enum {enum_name}"),
                actual: json_type_name(json_val),
            }),
        }
    }

    // -------------------------------------------------------------------
    // String encoding
    // -------------------------------------------------------------------

    fn encode_string(&mut self, s: &str) -> usize {
        self.align(4);
        let pos = self.buf.len();
        self.write_u32_le(s.len() as u32);
        self.write_bytes(s.as_bytes());
        self.write_u8(0); // null terminator
        self.align(4);
        pos
    }

    // -------------------------------------------------------------------
    // Struct encoding (inline, returns bytes)
    // -------------------------------------------------------------------

    fn encode_struct_inline(
        &self,
        json_val: &Value,
        obj_idx: usize,
        _field_name: &str,
        depth: usize,
    ) -> Result<Vec<u8>, JsonError> {
        if depth > MAX_DEPTH {
            return Err(JsonError::MaxDepthExceeded { max: MAX_DEPTH });
        }

        let obj = self.get_object(obj_idx)?;
        let type_name = obj.name.clone();
        let byte_size = obj.byte_size.unwrap_or(0) as usize;
        let fields = obj.fields.clone();

        let json_obj = json_val
            .as_object()
            .ok_or_else(|| JsonError::ExpectedObject {
                type_name: type_name.clone(),
                actual: json_type_name(json_val),
            })?;

        let mut data = vec![0u8; byte_size];

        for field in &fields {
            let fname = &field.name;
            let field_offset = field.offset.unwrap_or(0) as usize;
            let ty = &field.type_;
            let bt = ty.base_type;

            let json_field_val =
                json_obj
                    .get(fname.as_str())
                    .ok_or_else(|| JsonError::MissingStructField {
                        field_name: fname.clone(),
                    })?;

            match bt {
                BaseType::BASE_TYPE_STRUCT => {
                    let inner_idx = require_type_index(ty, fname)?;
                    let inner_bytes =
                        self.encode_struct_inline(json_field_val, inner_idx, fname, depth + 1)?;
                    let end = (field_offset + inner_bytes.len()).min(byte_size);
                    data[field_offset..end].copy_from_slice(&inner_bytes[..end - field_offset]);
                }
                _ => {
                    let bytes = self.encode_scalar_value(json_field_val, bt, ty, fname)?;
                    let end = (field_offset + bytes.len()).min(byte_size);
                    data[field_offset..end].copy_from_slice(&bytes[..end - field_offset]);
                }
            }
        }

        Ok(data)
    }

    // -------------------------------------------------------------------
    // Vector encoding
    // -------------------------------------------------------------------

    fn encode_vector(
        &mut self,
        json_val: &Value,
        ty: &ResolvedType,
        field_name: &str,
        depth: usize,
    ) -> Result<usize, JsonError> {
        let arr = json_val
            .as_array()
            .ok_or_else(|| JsonError::ExpectedArray {
                field_name: field_name.to_string(),
                actual: json_type_name(json_val),
            })?;

        let elem_bt = ty.element_type.unwrap_or(BaseType::BASE_TYPE_U_BYTE);

        match elem_bt {
            bt if is_scalar(bt) => {
                let elem_size = scalar_byte_size(bt);
                let alignment = elem_size.max(4);
                self.align(alignment);
                let pos = self.buf.len();
                self.write_u32_le(arr.len() as u32);
                for (i, elem) in arr.iter().enumerate() {
                    let elem_name = format!("{field_name}[{i}]");
                    let bytes = self.encode_scalar_value(elem, bt, ty, &elem_name)?;
                    self.write_bytes(&bytes);
                }
                self.align(4);
                Ok(pos)
            }

            BaseType::BASE_TYPE_STRING => {
                self.align(4);
                let pos = self.buf.len();
                self.write_u32_le(arr.len() as u32);

                let mut placeholders = Vec::new();
                for _ in arr.iter() {
                    placeholders.push(self.buf.len());
                    self.write_u32_le(0);
                }

                for (i, elem) in arr.iter().enumerate() {
                    let s = elem.as_str().ok_or_else(|| JsonError::ExpectedString {
                        field_name: format!("{field_name}[{i}]"),
                        actual: json_type_name(elem),
                    })?;
                    let str_pos = self.encode_string(s);
                    let uoffset = (str_pos - placeholders[i]) as u32;
                    self.patch_u32_le(placeholders[i], uoffset);
                }

                Ok(pos)
            }

            BaseType::BASE_TYPE_TABLE => {
                let inner_idx = require_type_index(ty, field_name)?;
                self.align(4);
                let pos = self.buf.len();
                self.write_u32_le(arr.len() as u32);

                let mut placeholders = Vec::new();
                for _ in arr.iter() {
                    placeholders.push(self.buf.len());
                    self.write_u32_le(0);
                }

                for (i, elem) in arr.iter().enumerate() {
                    let table_pos = self.encode_table(elem, inner_idx, depth + 1)?;
                    let uoffset = (table_pos - placeholders[i]) as u32;
                    self.patch_u32_le(placeholders[i], uoffset);
                }

                Ok(pos)
            }

            BaseType::BASE_TYPE_STRUCT => {
                let inner_idx = require_type_index(ty, field_name)?;
                let obj = self.get_object(inner_idx)?;
                let struct_align = obj.min_align.unwrap_or(1) as usize;
                let data_align = struct_align.max(4);
                while !(self.buf.len() + 4).is_multiple_of(data_align) {
                    self.buf.push(0);
                }
                let pos = self.buf.len();
                self.write_u32_le(arr.len() as u32);

                for (i, elem) in arr.iter().enumerate() {
                    let elem_name = format!("{field_name}[{i}]");
                    let bytes =
                        self.encode_struct_inline(elem, inner_idx, &elem_name, depth + 1)?;
                    self.write_bytes(&bytes);
                }
                self.align(4);

                Ok(pos)
            }

            _ => {
                // Unsupported element type -- write empty vector
                self.align(4);
                let pos = self.buf.len();
                self.write_u32_le(0);
                Ok(pos)
            }
        }
    }

    // -------------------------------------------------------------------
    // Union data encoding
    // -------------------------------------------------------------------

    fn encode_union_data(
        &mut self,
        json_val: &Value,
        enum_idx: usize,
        discriminant: u8,
        field_name: &str,
        depth: usize,
    ) -> Result<usize, JsonError> {
        let enum_def = self.get_enum(enum_idx)?;

        let variant = enum_def
            .values
            .iter()
            .find(|v| v.value == discriminant as i64);

        let variant = match variant {
            Some(v) => v.clone(),
            None => return Ok(self.buf.len()),
        };

        let union_type = match variant.union_type {
            Some(ref t) => *t,
            None => return Ok(self.buf.len()),
        };

        let variant_bt = union_type.base_type;

        match variant_bt {
            BaseType::BASE_TYPE_TABLE => {
                let inner_idx = require_type_index(&union_type, "union variant")?;
                self.encode_table(json_val, inner_idx, depth)
            }
            BaseType::BASE_TYPE_STRING => {
                let s = json_val.as_str().ok_or_else(|| JsonError::ExpectedString {
                    field_name: field_name.to_string(),
                    actual: json_type_name(json_val),
                })?;
                Ok(self.encode_string(s))
            }
            _ => Ok(self.buf.len()),
        }
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Extract a non-negative type index from a `ResolvedType`, or return a `JsonError`.
fn require_type_index(ty: &ResolvedType, context: &str) -> Result<usize, JsonError> {
    ty.index
        .filter(|&i| i >= 0)
        .map(|i| i as usize)
        .ok_or_else(|| JsonError::MissingTypeIndex {
            context: context.to_string(),
        })
}

fn json_as_i64(v: &Value, field_name: &str, bt: BaseType) -> Result<i64, JsonError> {
    match v {
        Value::Number(n) => n
            .as_i64()
            .or_else(|| n.as_f64().map(|f| f as i64))
            .ok_or_else(|| JsonError::NumberOutOfRange {
                field_name: field_name.to_string(),
                base_type: format!("{bt:?}"),
                value: n.to_string(),
            }),
        Value::Bool(b) => Ok(if *b { 1 } else { 0 }),
        _ => Err(JsonError::ExpectedNumber {
            field_name: field_name.to_string(),
            base_type: format!("{bt:?}"),
            actual: json_type_name(v),
        }),
    }
}

fn json_as_u64(v: &Value, field_name: &str, bt: BaseType) -> Result<u64, JsonError> {
    match v {
        Value::Number(n) => n
            .as_u64()
            .or_else(|| n.as_i64().map(|i| i as u64))
            .or_else(|| n.as_f64().map(|f| f as u64))
            .ok_or_else(|| JsonError::NumberOutOfRange {
                field_name: field_name.to_string(),
                base_type: format!("{bt:?}"),
                value: n.to_string(),
            }),
        Value::Bool(b) => Ok(if *b { 1 } else { 0 }),
        _ => Err(JsonError::ExpectedNumber {
            field_name: field_name.to_string(),
            base_type: format!("{bt:?}"),
            actual: json_type_name(v),
        }),
    }
}

fn json_as_f64(v: &Value, field_name: &str, bt: BaseType) -> Result<f64, JsonError> {
    match v {
        Value::Number(n) => n.as_f64().ok_or_else(|| JsonError::NumberOutOfRange {
            field_name: field_name.to_string(),
            base_type: format!("{bt:?}"),
            value: n.to_string(),
        }),
        Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
        _ => Err(JsonError::ExpectedNumber {
            field_name: field_name.to_string(),
            base_type: format!("{bt:?}"),
            actual: json_type_name(v),
        }),
    }
}

/// Compute the inline size and alignment of a field within table data.
fn field_inline_size(bt: BaseType, ty: &ResolvedType, schema: &ResolvedSchema) -> (usize, usize) {
    match bt {
        BaseType::BASE_TYPE_BOOL
        | BaseType::BASE_TYPE_BYTE
        | BaseType::BASE_TYPE_U_BYTE
        | BaseType::BASE_TYPE_U_TYPE => (1, 1),

        BaseType::BASE_TYPE_SHORT | BaseType::BASE_TYPE_U_SHORT => (2, 2),

        BaseType::BASE_TYPE_INT | BaseType::BASE_TYPE_U_INT | BaseType::BASE_TYPE_FLOAT => (4, 4),

        BaseType::BASE_TYPE_LONG | BaseType::BASE_TYPE_U_LONG | BaseType::BASE_TYPE_DOUBLE => {
            (8, 8)
        }

        BaseType::BASE_TYPE_STRING
        | BaseType::BASE_TYPE_TABLE
        | BaseType::BASE_TYPE_VECTOR
        | BaseType::BASE_TYPE_UNION => (4, 4),

        BaseType::BASE_TYPE_STRUCT => {
            let idx = ty.index.filter(|&i| i >= 0).map(|i| i as usize);
            match idx.and_then(|i| schema.objects.get(i)) {
                Some(obj) => {
                    let size = obj.byte_size.unwrap_or(0) as usize;
                    let align = obj.min_align.unwrap_or(1) as usize;
                    (size, align)
                }
                None => (0, 1),
            }
        }

        _ => (0, 1),
    }
}
