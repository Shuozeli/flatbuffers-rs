//! DataBuilder: walks a compiled Schema and builds random `serde_json::Value`.

use flatc_rs_schema::{BaseType, Enum, Field, Object, Schema, Type};
use rand::rngs::StdRng;
use rand::Rng;
use serde_json::{Map, Value};

use crate::{DataGenConfig, DataGenError};

pub struct DataBuilder<'a> {
    schema: &'a Schema,
    rng: StdRng,
    config: DataGenConfig,
}

impl<'a> DataBuilder<'a> {
    pub fn new(schema: &'a Schema, rng: StdRng, config: DataGenConfig) -> Self {
        Self {
            schema,
            rng,
            config,
        }
    }

    /// Build the root table as a JSON value.
    pub fn build_root(&mut self, root_type: &str) -> Result<Value, DataGenError> {
        let idx = self.find_object_index(root_type)?;
        self.build_table(idx, 0)
    }

    // -------------------------------------------------------------------
    // Schema lookup helpers
    // -------------------------------------------------------------------

    fn find_object_index(&self, name: &str) -> Result<usize, DataGenError> {
        // Exact match first
        for (i, obj) in self.schema.objects.iter().enumerate() {
            if let Some(ref obj_name) = obj.name {
                if obj_name == name {
                    return Ok(i);
                }
            }
        }
        // Short name match (without namespace)
        for (i, obj) in self.schema.objects.iter().enumerate() {
            if let Some(ref obj_name) = obj.name {
                let short = obj_name.rsplit('.').next().unwrap_or(obj_name);
                if short == name {
                    return Ok(i);
                }
            }
        }
        Err(DataGenError::RootTypeNotFound {
            name: name.to_string(),
        })
    }

    fn get_object(&self, idx: usize) -> Result<&Object, DataGenError> {
        self.schema
            .objects
            .get(idx)
            .ok_or(DataGenError::ObjectIndexOutOfRange {
                index: idx,
                count: self.schema.objects.len(),
            })
    }

    fn get_enum(&self, idx: usize) -> Result<&Enum, DataGenError> {
        self.schema
            .enums
            .get(idx)
            .ok_or(DataGenError::EnumIndexOutOfRange {
                index: idx,
                count: self.schema.enums.len(),
            })
    }

    // -------------------------------------------------------------------
    // Table generation
    // -------------------------------------------------------------------

    fn build_table(&mut self, obj_idx: usize, depth: usize) -> Result<Value, DataGenError> {
        let obj = self.get_object(obj_idx)?.clone();
        let mut map = Map::new();

        // Sort fields by id for consistent output
        let mut fields: Vec<Field> = obj.fields.clone();
        fields.sort_by_key(|f| f.id.unwrap_or(0));

        // Collect union field names so we can skip companion U_TYPE fields
        // (they'll be emitted when we process the union field itself)
        let union_type_fields: Vec<String> = fields
            .iter()
            .filter(|f| {
                f.type_.as_ref().and_then(|t| t.base_type) == Some(BaseType::BASE_TYPE_UNION)
            })
            .filter_map(|f| f.name.as_ref().map(|n| format!("{n}_type")))
            .collect();

        for field in &fields {
            let fname = field.name.clone().unwrap_or_default();
            let ty = field.type_.as_ref();
            let bt = ty
                .and_then(|t| t.base_type)
                .unwrap_or(BaseType::BASE_TYPE_NONE);

            // Skip deprecated fields
            if field.is_deprecated == Some(true) {
                continue;
            }

            // Skip U_TYPE companion fields (handled by union generation)
            if bt == BaseType::BASE_TYPE_U_TYPE {
                continue;
            }

            // Skip companion _type fields for unions
            if union_type_fields.contains(&fname) {
                continue;
            }

            // Decide whether to include this field
            let required = field.is_required == Some(true);
            if !required {
                // At max depth, skip table/vector-of-table/union fields
                if depth >= self.config.max_depth {
                    match bt {
                        BaseType::BASE_TYPE_TABLE | BaseType::BASE_TYPE_UNION => continue,
                        BaseType::BASE_TYPE_VECTOR => {
                            let elem_bt = ty
                                .and_then(|t| t.element_type)
                                .unwrap_or(BaseType::BASE_TYPE_NONE);
                            if elem_bt == BaseType::BASE_TYPE_TABLE {
                                continue;
                            }
                        }
                        _ => {}
                    }
                }
                if !self.rng.gen_bool(self.config.prob_include_field) {
                    continue;
                }
            }

            match bt {
                BaseType::BASE_TYPE_UNION => {
                    // Generate union: pick a variant, emit _type and data
                    let enum_idx = ty.and_then(|t| t.index).unwrap_or(0) as usize;
                    if let Some((type_val, data_val)) = self.build_union(enum_idx, depth)? {
                        map.insert(format!("{fname}_type"), type_val);
                        map.insert(fname, data_val);
                    }
                }
                _ => {
                    if let Some(val) = self.build_field_value(ty, bt, depth)? {
                        map.insert(fname, val);
                    }
                }
            }
        }

        Ok(Value::Object(map))
    }

    // -------------------------------------------------------------------
    // Field value dispatch
    // -------------------------------------------------------------------

    fn build_field_value(
        &mut self,
        ty: Option<&Type>,
        bt: BaseType,
        depth: usize,
    ) -> Result<Option<Value>, DataGenError> {
        // Check if this is an enum-typed scalar
        let enum_idx = ty.and_then(|t| t.index);
        if let Some(idx) = enum_idx {
            let idx = idx as usize;
            if idx < self.schema.enums.len() {
                let e = &self.schema.enums[idx];
                if e.is_union != Some(true) && is_scalar(bt) {
                    return Ok(Some(self.build_enum_value(idx)?));
                }
            }
        }

        match bt {
            BaseType::BASE_TYPE_BOOL => Ok(Some(Value::Bool(self.rng.gen_bool(0.5)))),

            BaseType::BASE_TYPE_BYTE => Ok(Some(Value::Number(
                self.rng.gen_range(-128i64..=127).into(),
            ))),
            BaseType::BASE_TYPE_U_BYTE => {
                Ok(Some(Value::Number(self.rng.gen_range(0u64..=255).into())))
            }
            BaseType::BASE_TYPE_SHORT => Ok(Some(Value::Number(
                self.rng.gen_range(-1000i64..=1000).into(),
            ))),
            BaseType::BASE_TYPE_U_SHORT => {
                Ok(Some(Value::Number(self.rng.gen_range(0u64..=2000).into())))
            }
            BaseType::BASE_TYPE_INT => Ok(Some(Value::Number(
                self.rng.gen_range(-10000i64..=10000).into(),
            ))),
            BaseType::BASE_TYPE_U_INT => {
                Ok(Some(Value::Number(self.rng.gen_range(0u64..=20000).into())))
            }
            BaseType::BASE_TYPE_LONG => Ok(Some(Value::Number(
                self.rng.gen_range(-100000i64..=100000).into(),
            ))),
            BaseType::BASE_TYPE_U_LONG => Ok(Some(Value::Number(
                self.rng.gen_range(0u64..=200000).into(),
            ))),
            BaseType::BASE_TYPE_FLOAT | BaseType::BASE_TYPE_DOUBLE => {
                let v: f64 = self.rng.gen_range(-100.0..100.0);
                // Round to 2 decimal places to produce clean JSON
                let rounded = (v * 100.0).round() / 100.0;
                Ok(Some(Value::Number(
                    serde_json::Number::from_f64(rounded)
                        .unwrap_or_else(|| serde_json::Number::from(0)),
                )))
            }

            BaseType::BASE_TYPE_STRING => Ok(Some(Value::String(self.random_string()))),

            BaseType::BASE_TYPE_TABLE => {
                let inner_idx = ty.and_then(|t| t.index).unwrap_or(0) as usize;
                Ok(Some(self.build_table(inner_idx, depth + 1)?))
            }

            BaseType::BASE_TYPE_STRUCT => {
                let inner_idx = ty.and_then(|t| t.index).unwrap_or(0) as usize;
                Ok(Some(self.build_struct(inner_idx)?))
            }

            BaseType::BASE_TYPE_VECTOR => Ok(Some(self.build_vector(ty, depth)?)),

            // U_TYPE is handled by union logic in build_table
            BaseType::BASE_TYPE_U_TYPE => Ok(None),

            _ => Ok(None),
        }
    }

    // -------------------------------------------------------------------
    // Struct generation (all fields required)
    // -------------------------------------------------------------------

    fn build_struct(&mut self, obj_idx: usize) -> Result<Value, DataGenError> {
        let obj = self.get_object(obj_idx)?.clone();
        let mut map = Map::new();

        let mut fields = obj.fields.clone();
        fields.sort_by_key(|f| f.offset.unwrap_or(0));

        for field in &fields {
            let fname = field.name.clone().unwrap_or_default();
            let ty = field.type_.as_ref();
            let bt = ty
                .and_then(|t| t.base_type)
                .unwrap_or(BaseType::BASE_TYPE_NONE);

            // Structs can only contain scalars, enums, and nested structs
            match bt {
                BaseType::BASE_TYPE_STRUCT => {
                    let inner_idx = ty.and_then(|t| t.index).unwrap_or(0) as usize;
                    map.insert(fname, self.build_struct(inner_idx)?);
                }
                _ => {
                    if let Some(val) = self.build_field_value(ty, bt, 0)? {
                        map.insert(fname, val);
                    }
                }
            }
        }

        Ok(Value::Object(map))
    }

    // -------------------------------------------------------------------
    // Enum value generation
    // -------------------------------------------------------------------

    fn build_enum_value(&mut self, enum_idx: usize) -> Result<Value, DataGenError> {
        let e = self.get_enum(enum_idx)?.clone();
        let values: Vec<_> = e.values.iter().filter_map(|v| v.name.clone()).collect();

        if values.is_empty() {
            return Ok(Value::Number(0.into()));
        }

        let idx = self.rng.gen_range(0..values.len());
        Ok(Value::String(values[idx].clone()))
    }

    // -------------------------------------------------------------------
    // Union generation
    // -------------------------------------------------------------------

    fn build_union(
        &mut self,
        enum_idx: usize,
        depth: usize,
    ) -> Result<Option<(Value, Value)>, DataGenError> {
        let e = self.get_enum(enum_idx)?.clone();

        // Filter non-NONE variants
        let variants: Vec<_> = e
            .values
            .iter()
            .filter(|v| v.value != Some(0))
            .cloned()
            .collect();

        if variants.is_empty() {
            return Ok(None);
        }

        let chosen = &variants[self.rng.gen_range(0..variants.len())];
        let type_name = chosen.name.clone().unwrap_or_default();
        let type_val = Value::String(type_name);

        // Build the union data
        let union_type = match &chosen.union_type {
            Some(t) => t.clone(),
            None => return Ok(None),
        };

        let variant_bt = union_type.base_type.unwrap_or(BaseType::BASE_TYPE_NONE);

        let data_val = match variant_bt {
            BaseType::BASE_TYPE_TABLE => {
                let inner_idx = union_type.index.unwrap_or(0) as usize;
                self.build_table(inner_idx, depth + 1)?
            }
            BaseType::BASE_TYPE_STRING => Value::String(self.random_string()),
            _ => Value::Object(Map::new()),
        };

        Ok(Some((type_val, data_val)))
    }

    // -------------------------------------------------------------------
    // Vector generation
    // -------------------------------------------------------------------

    fn build_vector(&mut self, ty: Option<&Type>, depth: usize) -> Result<Value, DataGenError> {
        let elem_bt = ty
            .and_then(|t| t.element_type)
            .unwrap_or(BaseType::BASE_TYPE_U_BYTE);

        let len = self
            .rng
            .gen_range(self.config.min_vector_len..=self.config.max_vector_len);

        let mut arr = Vec::with_capacity(len);
        for _ in 0..len {
            match elem_bt {
                BaseType::BASE_TYPE_TABLE => {
                    let inner_idx = ty.and_then(|t| t.index).unwrap_or(0) as usize;
                    if depth < self.config.max_depth {
                        arr.push(self.build_table(inner_idx, depth + 1)?);
                    }
                }
                BaseType::BASE_TYPE_STRUCT => {
                    let inner_idx = ty.and_then(|t| t.index).unwrap_or(0) as usize;
                    arr.push(self.build_struct(inner_idx)?);
                }
                _ => {
                    // For vector elements, pass the type info to handle enum vectors
                    if let Some(val) = self.build_field_value(ty, elem_bt, depth)? {
                        arr.push(val);
                    }
                }
            }
        }

        Ok(Value::Array(arr))
    }

    // -------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------

    fn random_string(&mut self) -> String {
        let len = self.rng.gen_range(1..=self.config.max_string_len);
        (0..len)
            .map(|_| {
                let idx = self.rng.gen_range(0..36);
                if idx < 26 {
                    (b'a' + idx as u8) as char
                } else {
                    (b'0' + (idx - 26) as u8) as char
                }
            })
            .collect()
    }
}

fn is_scalar(bt: BaseType) -> bool {
    matches!(
        bt,
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
            | BaseType::BASE_TYPE_DOUBLE
    )
}
