//! Decode a FlatBuffers binary into a JSON value using a compiled Schema.
//!
//! Reads the binary buffer directly using vtable/offset metadata from the schema,
//! producing a `serde_json::Value` tree. No intermediate representation.

use flatc_rs_schema::buf_reader::BufReader;
use flatc_rs_schema::resolved::{ResolvedEnum, ResolvedField, ResolvedObject, ResolvedSchema, ResolvedType};
use flatc_rs_schema::BaseType;
use serde_json::{json, Map, Value};

use super::error::{is_scalar, scalar_byte_size, JsonError};

const MAX_DEPTH: usize = 64;

/// Options controlling JSON output format.
#[derive(Debug, Clone)]
pub struct JsonOptions {
    /// Use strict JSON: no trailing commas, quoted field names (always true for serde_json).
    pub strict_json: bool,
    /// Output fields even when they have default values.
    pub output_defaults: bool,
    /// Output enum values as string identifiers rather than integers.
    pub output_enum_identifiers: bool,
    /// Treat input binary as size-prefixed (4-byte length header before the FlatBuffer).
    pub size_prefixed: bool,
}

impl Default for JsonOptions {
    fn default() -> Self {
        Self {
            strict_json: false,
            output_defaults: false,
            output_enum_identifiers: true,
            size_prefixed: false,
        }
    }
}

/// Decode a FlatBuffers binary into a JSON value.
///
/// `buf` is the complete FlatBuffer binary (including root offset and optional file identifier).
/// `schema` is the compiled FlatBuffers schema.
/// `root_type` is the name of the root table type.
/// `opts` controls output formatting.
pub fn binary_to_json(
    buf: &[u8],
    schema: &ResolvedSchema,
    root_type: &str,
    opts: &JsonOptions,
) -> Result<Value, JsonError> {
    let reader = BufReader::new(buf);
    let mut decoder = Decoder::new(reader, schema, opts);
    decoder.decode(root_type)
}

// ---------------------------------------------------------------------------
// Decoder
// ---------------------------------------------------------------------------

struct Decoder<'a> {
    reader: BufReader<'a>,
    schema: &'a ResolvedSchema,
    opts: &'a JsonOptions,
    object_index: std::collections::HashMap<&'a str, usize>,
}

impl<'a> Decoder<'a> {
    fn new(reader: BufReader<'a>, schema: &'a ResolvedSchema, opts: &'a JsonOptions) -> Self {
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
            reader,
            schema,
            opts,
            object_index,
        }
    }

    fn decode(&mut self, root_type: &str) -> Result<Value, JsonError> {
        // Size-prefixed buffers have a 4-byte length prefix before the FlatBuffer
        let base = if self.opts.size_prefixed { 4 } else { 0 };

        if self.reader.len() < base + 4 {
            return Err(JsonError::OutOfBounds {
                offset: base,
                need: 4,
                buf_len: self.reader.len(),
            });
        }

        let root_offset = self.reader.read_u32_le(base)? as usize + base;
        let root_idx = self.find_object_index(root_type)?;
        self.decode_table(root_offset, root_idx, 0)
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
    // Table decoding
    // -------------------------------------------------------------------

    fn decode_table(
        &self,
        table_offset: usize,
        obj_idx: usize,
        depth: usize,
    ) -> Result<Value, JsonError> {
        if depth > MAX_DEPTH {
            return Err(JsonError::MaxDepthExceeded { max: MAX_DEPTH });
        }

        let obj = self.get_object(obj_idx)?;
        let fields: Vec<ResolvedField> = obj.fields.clone();

        // Read soffset to vtable
        let vtable_soffset = self.reader.read_i32_le(table_offset)?;
        let vtable_offset = (table_offset as i64 - vtable_soffset as i64) as usize;
        let vtable_size = self.reader.read_u16_le(vtable_offset)? as usize;

        // Two-pass: first collect union type discriminants, then decode all fields
        let mut union_type_values: std::collections::HashMap<String, u8> =
            std::collections::HashMap::new();

        // First pass: read _type fields (BASE_TYPE_U_TYPE)
        for field in &fields {
            let ty = &field.type_;
            let bt = ty.base_type;
            if bt != BaseType::BASE_TYPE_U_TYPE {
                continue;
            }

            let field_id = field.id.unwrap_or(0) as usize;
            let vtable_entry_offset = vtable_offset + 4 + field_id * 2;
            if vtable_entry_offset + 2 > vtable_offset + vtable_size {
                continue;
            }

            let field_offset_in_table = self.reader.read_u16_le(vtable_entry_offset)? as usize;
            if field_offset_in_table == 0 {
                continue;
            }

            let field_data_offset = table_offset + field_offset_in_table;
            let val = self.reader.read_u8(field_data_offset)?;
            let fname = &field.name;
            let union_name = fname.strip_suffix("_type").unwrap_or(fname).to_string();
            union_type_values.insert(union_name, val);
        }

        // Second pass: decode all fields
        let mut result = Map::new();

        for field in &fields {
            let ty = &field.type_;
            let bt = ty.base_type;

            let field_id = field.id.unwrap_or(0) as usize;
            let vtable_entry_offset = vtable_offset + 4 + field_id * 2;
            if vtable_entry_offset + 2 > vtable_offset + vtable_size {
                continue;
            }

            let field_offset_in_table = self.reader.read_u16_le(vtable_entry_offset)? as usize;
            if field_offset_in_table == 0 {
                // Field not present, skip (or output default if requested)
                if self.opts.output_defaults {
                    let fname = field.name.clone();
                    if let Some(default) = self.default_value(field, bt, ty) {
                        result.insert(fname, default);
                    }
                }
                continue;
            }

            let field_data_offset = table_offset + field_offset_in_table;
            let fname = field.name.clone();

            let value =
                self.decode_field(field_data_offset, bt, ty, &fname, depth, &union_type_values)?;

            if let Some(v) = value {
                result.insert(fname, v);
            }
        }

        Ok(Value::Object(result))
    }

    // -------------------------------------------------------------------
    // Field decoding
    // -------------------------------------------------------------------

    fn decode_field(
        &self,
        offset: usize,
        bt: BaseType,
        ty: &ResolvedType,
        fname: &str,
        depth: usize,
        union_type_values: &std::collections::HashMap<String, u8>,
    ) -> Result<Option<Value>, JsonError> {
        match bt {
            // Scalars
            BaseType::BASE_TYPE_BOOL => {
                let v = self.reader.read_u8(offset)?;
                Ok(Some(Value::Bool(v != 0)))
            }

            BaseType::BASE_TYPE_BYTE => {
                let v = self.reader.read_i8(offset)?;
                Ok(Some(self.maybe_enum_value(v as i64, ty)))
            }
            BaseType::BASE_TYPE_U_BYTE => {
                let v = self.reader.read_u8(offset)?;
                Ok(Some(self.maybe_enum_value(v as i64, ty)))
            }
            BaseType::BASE_TYPE_SHORT => {
                let v = self.reader.read_i16_le(offset)?;
                Ok(Some(self.maybe_enum_value(v as i64, ty)))
            }
            BaseType::BASE_TYPE_U_SHORT => {
                let v = self.reader.read_u16_le(offset)?;
                Ok(Some(self.maybe_enum_value(v as i64, ty)))
            }
            BaseType::BASE_TYPE_INT => {
                let v = self.reader.read_i32_le(offset)?;
                Ok(Some(self.maybe_enum_value(v as i64, ty)))
            }
            BaseType::BASE_TYPE_U_INT => {
                let v = self.reader.read_u32_le(offset)?;
                Ok(Some(self.maybe_enum_value(v as i64, ty)))
            }
            BaseType::BASE_TYPE_LONG => {
                let v = self.reader.read_i64_le(offset)?;
                Ok(Some(self.maybe_enum_value(v, ty)))
            }
            BaseType::BASE_TYPE_U_LONG => {
                let v = self.reader.read_u64_le(offset)?;
                Ok(Some(json!(v)))
            }
            BaseType::BASE_TYPE_FLOAT => {
                let v = self.reader.read_f32_le(offset)?;
                Ok(Some(float_to_json(v as f64)))
            }
            BaseType::BASE_TYPE_DOUBLE => {
                let v = self.reader.read_f64_le(offset)?;
                Ok(Some(float_to_json(v)))
            }

            // Union type discriminant -- output as enum name or integer
            BaseType::BASE_TYPE_U_TYPE => {
                let v = self.reader.read_u8(offset)?;
                let enum_idx = require_type_index(ty, fname)?;
                Ok(Some(self.enum_value_to_json(v as i64, enum_idx)))
            }

            // String
            BaseType::BASE_TYPE_STRING => {
                let uoffset = self.reader.read_u32_le(offset)? as usize;
                let str_start = offset + uoffset;
                let length = self.reader.read_u32_le(str_start)? as usize;
                let bytes = self.reader.read_bytes(str_start + 4, length)?;
                let text = String::from_utf8_lossy(bytes);
                Ok(Some(Value::String(text.into_owned())))
            }

            // Table (referenced by offset)
            BaseType::BASE_TYPE_TABLE => {
                let uoffset = self.reader.read_u32_le(offset)? as usize;
                let table_start = offset + uoffset;
                let obj_idx = require_type_index(ty, fname)?;
                let val = self.decode_table(table_start, obj_idx, depth + 1)?;
                Ok(Some(val))
            }

            // Struct (inline in the table)
            BaseType::BASE_TYPE_STRUCT => {
                let obj_idx = require_type_index(ty, fname)?;
                let val = self.decode_struct(offset, obj_idx, depth + 1)?;
                Ok(Some(val))
            }

            // Vector
            BaseType::BASE_TYPE_VECTOR => {
                let uoffset = self.reader.read_u32_le(offset)? as usize;
                let vec_start = offset + uoffset;
                let val = self.decode_vector(vec_start, ty, depth + 1)?;
                Ok(Some(val))
            }

            // Union data
            BaseType::BASE_TYPE_UNION => {
                if let Some(&discriminant) = union_type_values.get(fname) {
                    if discriminant == 0 {
                        return Ok(None);
                    }
                    let uoffset = self.reader.read_u32_le(offset)? as usize;
                    let data_start = offset + uoffset;
                    let val = self.decode_union_data(data_start, ty, discriminant, depth + 1)?;
                    Ok(Some(val))
                } else {
                    Ok(None)
                }
            }

            _ => Ok(None),
        }
    }

    // -------------------------------------------------------------------
    // Struct decoding (inline, fixed layout)
    // -------------------------------------------------------------------

    fn decode_struct(
        &self,
        offset: usize,
        obj_idx: usize,
        depth: usize,
    ) -> Result<Value, JsonError> {
        if depth > MAX_DEPTH {
            return Err(JsonError::MaxDepthExceeded { max: MAX_DEPTH });
        }

        let obj = self.get_object(obj_idx)?;
        let fields: Vec<ResolvedField> = obj.fields.clone();

        let mut result = Map::new();

        for field in &fields {
            let field_ty = &field.type_;
            let field_bt = field_ty.base_type;
            let field_offset = offset + field.offset.unwrap_or(0) as usize;
            let fname = field.name.clone();

            match field_bt {
                BaseType::BASE_TYPE_STRUCT => {
                    let inner_idx = require_type_index(field_ty, &fname)?;
                    let val = self.decode_struct(field_offset, inner_idx, depth + 1)?;
                    result.insert(fname, val);
                }
                BaseType::BASE_TYPE_ARRAY => {
                    let val = self.decode_fixed_array(field_offset, field_ty, depth + 1)?;
                    result.insert(fname, val);
                }
                BaseType::BASE_TYPE_BOOL => {
                    let v = self.reader.read_u8(field_offset)?;
                    result.insert(fname, Value::Bool(v != 0));
                }
                bt if is_scalar(bt) => {
                    let val = self.read_scalar_value(field_offset, bt, field_ty)?;
                    result.insert(fname, val);
                }
                _ => {}
            }
        }

        Ok(Value::Object(result))
    }

    // -------------------------------------------------------------------
    // Fixed-length array decoding (struct-only)
    // -------------------------------------------------------------------

    fn decode_fixed_array(
        &self,
        offset: usize,
        ty: &ResolvedType,
        depth: usize,
    ) -> Result<Value, JsonError> {
        let elem_bt = ty.element_type.unwrap_or(BaseType::BASE_TYPE_U_BYTE);
        let fixed_len = ty.fixed_length.unwrap_or(0) as usize;

        let mut arr = Vec::with_capacity(fixed_len);

        match elem_bt {
            bt if is_scalar(bt) => {
                let elem_size = scalar_byte_size(bt);
                for i in 0..fixed_len {
                    let elem_offset = offset + i * elem_size;
                    let val = self.read_scalar_value(elem_offset, bt, ty)?;
                    arr.push(val);
                }
            }
            BaseType::BASE_TYPE_STRUCT => {
                let inner_idx = require_type_index(ty, "fixed_array element")?;
                let obj = self.get_object(inner_idx)?;
                let struct_size = obj.byte_size.unwrap_or(0) as usize;
                for i in 0..fixed_len {
                    let elem_offset = offset + i * struct_size;
                    let val = self.decode_struct(elem_offset, inner_idx, depth + 1)?;
                    arr.push(val);
                }
            }
            _ => {}
        }

        Ok(Value::Array(arr))
    }

    // -------------------------------------------------------------------
    // Vector decoding
    // -------------------------------------------------------------------

    fn decode_vector(&self, vec_start: usize, ty: &ResolvedType, depth: usize) -> Result<Value, JsonError> {
        let count = self.reader.read_u32_le(vec_start)? as usize;
        let data_start = vec_start + 4;

        let elem_bt = ty.element_type.unwrap_or(BaseType::BASE_TYPE_U_BYTE);

        let mut arr = Vec::with_capacity(count);

        match elem_bt {
            bt if is_scalar(bt) => {
                let elem_size = scalar_byte_size(bt);
                for i in 0..count {
                    let elem_offset = data_start + i * elem_size;
                    let val = self.read_scalar_value(elem_offset, bt, ty)?;
                    arr.push(val);
                }
            }

            BaseType::BASE_TYPE_STRING => {
                for i in 0..count {
                    let elem_offset = data_start + i * 4;
                    let uoffset = self.reader.read_u32_le(elem_offset)? as usize;
                    let str_start = elem_offset + uoffset;
                    let length = self.reader.read_u32_le(str_start)? as usize;
                    let bytes = self.reader.read_bytes(str_start + 4, length)?;
                    let text = String::from_utf8_lossy(bytes);
                    arr.push(Value::String(text.into_owned()));
                }
            }

            BaseType::BASE_TYPE_TABLE => {
                let obj_idx = require_type_index(ty, "vector element")?;
                for i in 0..count {
                    let elem_offset = data_start + i * 4;
                    let uoffset = self.reader.read_u32_le(elem_offset)? as usize;
                    let table_start = elem_offset + uoffset;
                    let val = self.decode_table(table_start, obj_idx, depth + 1)?;
                    arr.push(val);
                }
            }

            BaseType::BASE_TYPE_STRUCT => {
                let obj_idx = require_type_index(ty, "vector element")?;
                let obj = self.get_object(obj_idx)?;
                let struct_size = obj.byte_size.unwrap_or(0) as usize;
                for i in 0..count {
                    let elem_offset = data_start + i * struct_size;
                    let val = self.decode_struct(elem_offset, obj_idx, depth + 1)?;
                    arr.push(val);
                }
            }

            _ => {}
        }

        Ok(Value::Array(arr))
    }

    // -------------------------------------------------------------------
    // Union data decoding
    // -------------------------------------------------------------------

    fn decode_union_data(
        &self,
        offset: usize,
        ty: &ResolvedType,
        discriminant: u8,
        depth: usize,
    ) -> Result<Value, JsonError> {
        if discriminant == 0 {
            return Ok(Value::Null);
        }

        let enum_idx = require_type_index(ty, "union")?;
        let enum_def = self.get_enum(enum_idx)?;

        let variant = enum_def
            .values
            .iter()
            .find(|v| v.value == discriminant as i64);

        if let Some(variant) = variant {
            if let Some(ref union_type) = variant.union_type {
                let variant_bt = union_type.base_type;
                let variant_idx = require_type_index(union_type, "union variant")?;

                return match variant_bt {
                    BaseType::BASE_TYPE_TABLE => self.decode_table(offset, variant_idx, depth + 1),
                    BaseType::BASE_TYPE_STRUCT => {
                        self.decode_struct(offset, variant_idx, depth + 1)
                    }
                    BaseType::BASE_TYPE_STRING => {
                        let length = self.reader.read_u32_le(offset)? as usize;
                        let bytes = self.reader.read_bytes(offset + 4, length)?;
                        let text = String::from_utf8_lossy(bytes);
                        Ok(Value::String(text.into_owned()))
                    }
                    _ => Ok(Value::Null),
                };
            }
        }

        Ok(Value::Null)
    }

    // -------------------------------------------------------------------
    // Scalar reading
    // -------------------------------------------------------------------

    fn read_scalar_value(
        &self,
        offset: usize,
        bt: BaseType,
        ty: &ResolvedType,
    ) -> Result<Value, JsonError> {
        match bt {
            BaseType::BASE_TYPE_BOOL => {
                let v = self.reader.read_u8(offset)?;
                Ok(Value::Bool(v != 0))
            }
            BaseType::BASE_TYPE_BYTE => {
                let v = self.reader.read_i8(offset)?;
                Ok(self.maybe_enum_value(v as i64, ty))
            }
            BaseType::BASE_TYPE_U_BYTE => {
                let v = self.reader.read_u8(offset)?;
                Ok(self.maybe_enum_value(v as i64, ty))
            }
            BaseType::BASE_TYPE_SHORT => {
                let v = self.reader.read_i16_le(offset)?;
                Ok(self.maybe_enum_value(v as i64, ty))
            }
            BaseType::BASE_TYPE_U_SHORT => {
                let v = self.reader.read_u16_le(offset)?;
                Ok(self.maybe_enum_value(v as i64, ty))
            }
            BaseType::BASE_TYPE_INT => {
                let v = self.reader.read_i32_le(offset)?;
                Ok(self.maybe_enum_value(v as i64, ty))
            }
            BaseType::BASE_TYPE_U_INT => {
                let v = self.reader.read_u32_le(offset)?;
                Ok(self.maybe_enum_value(v as i64, ty))
            }
            BaseType::BASE_TYPE_LONG => {
                let v = self.reader.read_i64_le(offset)?;
                Ok(self.maybe_enum_value(v, ty))
            }
            BaseType::BASE_TYPE_U_LONG => {
                let v = self.reader.read_u64_le(offset)?;
                Ok(json!(v))
            }
            BaseType::BASE_TYPE_FLOAT => {
                let v = self.reader.read_f32_le(offset)?;
                Ok(float_to_json(v as f64))
            }
            BaseType::BASE_TYPE_DOUBLE => {
                let v = self.reader.read_f64_le(offset)?;
                Ok(float_to_json(v))
            }
            BaseType::BASE_TYPE_U_TYPE => {
                let v = self.reader.read_u8(offset)?;
                Ok(json!(v))
            }
            _ => Ok(Value::Null),
        }
    }

    // -------------------------------------------------------------------
    // Enum helpers
    // -------------------------------------------------------------------

    /// Convert an integer value to JSON, resolving enum names if applicable.
    fn maybe_enum_value(&self, val: i64, ty: &ResolvedType) -> Value {
        if self.opts.output_enum_identifiers {
            if let Some(idx) = ty.index {
                if idx >= 0 && (idx as usize) < self.schema.enums.len() {
                    let e = &self.schema.enums[idx as usize];
                    if let Some(ev) = e.values.iter().find(|ev| ev.value == val) {
                        return Value::String(ev.name.clone());
                    }
                }
            }
        }
        json!(val)
    }

    /// Convert an enum discriminant to JSON (name or integer).
    fn enum_value_to_json(&self, val: i64, enum_idx: usize) -> Value {
        if self.opts.output_enum_identifiers {
            if let Some(e) = self.schema.enums.get(enum_idx) {
                if let Some(ev) = e.values.iter().find(|ev| ev.value == val) {
                    return Value::String(ev.name.clone());
                }
            }
        }
        json!(val)
    }

    /// Produce a default JSON value for a field type.
    fn default_value(&self, field: &ResolvedField, bt: BaseType, ty: &ResolvedType) -> Option<Value> {
        // Use the field's default_integer / default_real if available,
        // otherwise use the type's zero value.
        match bt {
            BaseType::BASE_TYPE_BOOL => {
                let v = field.default_integer.unwrap_or(0);
                Some(Value::Bool(v != 0))
            }
            BaseType::BASE_TYPE_BYTE
            | BaseType::BASE_TYPE_U_BYTE
            | BaseType::BASE_TYPE_SHORT
            | BaseType::BASE_TYPE_U_SHORT
            | BaseType::BASE_TYPE_INT
            | BaseType::BASE_TYPE_U_INT
            | BaseType::BASE_TYPE_LONG
            | BaseType::BASE_TYPE_U_LONG => {
                let v = field.default_integer.unwrap_or(0);
                Some(self.maybe_enum_value(v, ty))
            }
            BaseType::BASE_TYPE_FLOAT | BaseType::BASE_TYPE_DOUBLE => {
                let v = field.default_real.unwrap_or(0.0);
                Some(float_to_json(v))
            }
            BaseType::BASE_TYPE_U_TYPE => {
                let v = field.default_integer.unwrap_or(0);
                let enum_idx = ty
                    .index
                    .filter(|&i| i >= 0)
                    .map(|i| i as usize)?;
                Some(self.enum_value_to_json(v, enum_idx))
            }
            BaseType::BASE_TYPE_STRING => Some(Value::Null),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Free functions
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

/// Convert an f64 to a JSON value, preserving integer representation for whole numbers.
fn float_to_json(v: f64) -> Value {
    if v.is_nan() || v.is_infinite() {
        // serde_json doesn't support NaN/Infinity natively, use null
        Value::Null
    } else {
        json!(v)
    }
}
