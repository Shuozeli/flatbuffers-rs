//! Struct memory layout computation.
//!
//! FlatBuffers structs are stored inline (no vtable, no indirection). Their
//! binary layout must match exactly between writer and reader, so the compiler
//! must compute deterministic field offsets, padding, and total sizes.
//!
//! ## Algorithm
//!
//! 1. **Topological sort**: Structs are sorted in dependency order (leaf-first)
//!    using a depth-first traversal. This ensures that when we compute the
//!    layout of struct A which contains struct B, B's size and alignment are
//!    already known. Circular dependencies are detected and rejected.
//!
//! 2. **Field layout**: For each struct (in topological order), iterate fields
//!    in declaration order:
//!    - Compute each field's (size, alignment) from its type
//!    - Align the current offset up to the field's alignment
//!    - Record the field's offset and advance by its size
//!    - Track the maximum alignment seen across all fields
//!
//! 3. **force_align**: If the struct has a `force_align` attribute with a value
//!    larger than the natural max alignment, use that instead. This forces the
//!    struct's alignment (and therefore its total size padding) to a larger
//!    boundary.
//!
//! 4. **Final padding**: The total struct size is aligned up to max_align, so
//!    that arrays of the struct maintain proper alignment.
//!
//! 5. **Padding fields**: For each field, compute the padding bytes between its
//!    end and the next field's start (or the struct's end for the last field).
//!    This padding is stored in `field.padding` for codegen to emit.

use flatc_rs_schema::{self as schema, BaseType};

use crate::error::{AnalyzeError, Result};

/// Compute byte_size, min_align, field offsets, and padding for all structs.
///
/// Structs must be processed in dependency order (leaf structs first) so that
/// nested struct sizes are available. Circular structs must have been rejected
/// before calling this function.
pub fn compute_struct_layouts(schema: &mut schema::Schema) -> Result<()> {
    let order = topological_sort_structs(schema)?;

    for obj_idx in order {
        let obj = &schema.objects[obj_idx];
        if !obj.is_struct {
            continue;
        }

        let mut offset: u32 = 0;
        let mut max_align: u32 = 1;

        // Collect field layout info (we can't mutate while borrowing schema)
        let mut field_layouts: Vec<(u32, u32)> = Vec::new(); // (offset, size)
        let obj_name = obj.name.as_deref().unwrap_or("<unnamed>");

        for field in &obj.fields {
            let field_type = match field.type_.as_ref() {
                Some(t) => t,
                None => continue,
            };
            let (field_size, field_align) = type_size_align(field_type, &schema.objects, obj_name)?;

            // Align offset
            let aligned_offset = align_up(offset, field_align);
            field_layouts.push((aligned_offset, field_size));

            offset = aligned_offset + field_size;
            max_align = max_align.max(field_align);
        }

        // Apply force_align: override alignment if the attribute specifies a
        // larger alignment than the natural one.
        if let Some(fa) = get_force_align(obj) {
            max_align = max_align.max(fa);
        }

        // Final padding to align total struct size
        let total_size = align_up(offset, max_align);

        // Now apply layouts to the mutable schema
        let obj = &mut schema.objects[obj_idx];
        for (i, field) in obj.fields.iter_mut().enumerate() {
            let (field_offset, field_size) = field_layouts[i];
            field.offset = Some(field_offset);

            // Compute padding: gap between end of this field and start of next,
            // or gap to end of struct for the last field.
            let next_start = if i + 1 < field_layouts.len() {
                field_layouts[i + 1].0
            } else {
                total_size
            };
            let padding = next_start - (field_offset + field_size);
            if padding > 0 {
                field.padding = Some(padding);
            }
        }

        obj.byte_size = Some(total_size as i32);
        obj.min_align = Some(max_align as i32);
    }

    Ok(())
}

/// Extract the `force_align` attribute value from a struct, if present.
fn get_force_align(obj: &schema::Object) -> Option<u32> {
    obj.attributes.as_ref().and_then(|attrs| {
        attrs.entries.iter().find_map(|e| {
            if e.key.as_deref() == Some("force_align") {
                e.value.as_deref().and_then(|v| v.parse::<u32>().ok())
            } else {
                None
            }
        })
    })
}

/// Validate and convert a type index to `usize`, returning an error if out of range.
fn checked_obj_index(idx: i32, objects: &[schema::Object], struct_name: &str) -> Result<usize> {
    if idx < 0 || (idx as usize) >= objects.len() {
        return Err(AnalyzeError::InvalidTypeIndex {
            struct_name: struct_name.to_string(),
            index: idx,
            max_index: objects.len(),
        });
    }
    Ok(idx as usize)
}

/// Get the size and alignment of a type (for struct layout purposes).
fn type_size_align(
    ty: &schema::Type,
    objects: &[schema::Object],
    struct_name: &str,
) -> Result<(u32, u32)> {
    let bt = ty.base_type.unwrap_or(BaseType::BASE_TYPE_NONE);

    match bt {
        BaseType::BASE_TYPE_BOOL | BaseType::BASE_TYPE_BYTE | BaseType::BASE_TYPE_U_BYTE => {
            Ok((1, 1))
        }
        BaseType::BASE_TYPE_SHORT | BaseType::BASE_TYPE_U_SHORT => Ok((2, 2)),
        BaseType::BASE_TYPE_INT | BaseType::BASE_TYPE_U_INT | BaseType::BASE_TYPE_FLOAT => {
            Ok((4, 4))
        }
        BaseType::BASE_TYPE_LONG | BaseType::BASE_TYPE_U_LONG | BaseType::BASE_TYPE_DOUBLE => {
            Ok((8, 8))
        }
        BaseType::BASE_TYPE_STRUCT => {
            // Use already-computed struct layout
            if let Some(idx) = ty.index {
                let i = checked_obj_index(idx, objects, struct_name)?;
                let inner = &objects[i];
                let size = inner.byte_size.unwrap_or(0) as u32;
                let align = inner.min_align.unwrap_or(1) as u32;
                Ok((size, align))
            } else {
                Ok((0, 1)) // should not happen after type resolution
            }
        }
        BaseType::BASE_TYPE_ARRAY => {
            let fixed_len = ty.fixed_length.unwrap_or(0);
            let et = ty.element_type.unwrap_or(BaseType::BASE_TYPE_NONE);
            if et == BaseType::BASE_TYPE_STRUCT {
                // Look up actual struct size and alignment
                if let Some(idx) = ty.index {
                    let i = checked_obj_index(idx, objects, struct_name)?;
                    let inner = &objects[i];
                    let elem_size = inner.byte_size.unwrap_or(0) as u32;
                    let elem_align = inner.min_align.unwrap_or(1) as u32;
                    Ok((elem_size * fixed_len, elem_align))
                } else {
                    Ok((0, 1))
                }
            } else {
                let elem_size = ty.element_size.unwrap_or(0);
                let elem_align = element_alignment(ty);
                Ok((elem_size * fixed_len, elem_align))
            }
        }
        // Enum types in structs use the underlying type's size
        _ => {
            // For enums resolved to their underlying type, base_size is set
            let size = ty.base_size.unwrap_or(4);
            Ok((size, size))
        }
    }
}

/// Get alignment for an array element type.
fn element_alignment(ty: &schema::Type) -> u32 {
    let et = ty.element_type.unwrap_or(BaseType::BASE_TYPE_NONE);

    match et {
        BaseType::BASE_TYPE_BOOL | BaseType::BASE_TYPE_BYTE | BaseType::BASE_TYPE_U_BYTE => 1,
        BaseType::BASE_TYPE_SHORT | BaseType::BASE_TYPE_U_SHORT => 2,
        BaseType::BASE_TYPE_INT | BaseType::BASE_TYPE_U_INT | BaseType::BASE_TYPE_FLOAT => 4,
        BaseType::BASE_TYPE_LONG | BaseType::BASE_TYPE_U_LONG | BaseType::BASE_TYPE_DOUBLE => 8,
        _ => ty.element_size.unwrap_or(4),
    }
}

/// Align `offset` up to the next multiple of `align`.
fn align_up(offset: u32, align: u32) -> u32 {
    (offset + align - 1) & !(align - 1)
}

/// Maximum nesting depth for struct dependencies.
/// Prevents stack overflow from deeply nested struct chains.
const MAX_STRUCT_DEPTH: usize = 256;

/// Topological sort of struct indices. Returns indices in dependency order
/// (leaf structs first). Detects circular dependencies.
fn topological_sort_structs(schema: &schema::Schema) -> Result<Vec<usize>> {
    let struct_count = schema.objects.len();
    // State: 0 = unvisited, 1 = in-progress, 2 = done
    let mut state = vec![0u8; struct_count];
    let mut order = Vec::with_capacity(struct_count);
    let mut path: Vec<String> = Vec::new();

    for i in 0..struct_count {
        if !schema.objects[i].is_struct {
            continue;
        }
        if state[i] == 0 {
            visit_struct(schema, i, &mut state, &mut order, &mut path, 0)?;
        }
    }

    Ok(order)
}

fn visit_struct(
    schema: &schema::Schema,
    idx: usize,
    state: &mut [u8],
    order: &mut Vec<usize>,
    path: &mut Vec<String>,
    depth: usize,
) -> Result<()> {
    if state[idx] == 2 {
        return Ok(());
    }

    // G3.7: Prevent stack overflow from deeply nested struct chains
    if depth > MAX_STRUCT_DEPTH {
        let type_name = schema.objects[idx]
            .name
            .as_deref()
            .unwrap_or("<unnamed>")
            .to_string();
        return Err(AnalyzeError::StructDepthLimitExceeded {
            depth,
            type_name,
        });
    }

    let obj = &schema.objects[idx];
    let name = obj.name.as_deref().unwrap_or("<unnamed>").to_string();

    if state[idx] == 1 {
        // Cycle detected
        path.push(name);
        return Err(AnalyzeError::CircularStruct(path.clone()));
    }

    state[idx] = 1;
    path.push(name.clone());

    // Visit struct dependencies (both direct struct fields and array-of-struct fields)
    for field in &obj.fields {
        if let Some(ty) = field.type_.as_ref() {
            let bt = ty.base_type.unwrap_or(BaseType::BASE_TYPE_NONE);

            if bt == BaseType::BASE_TYPE_STRUCT {
                if let Some(dep_idx) = ty.index {
                    let i = checked_obj_index(dep_idx, &schema.objects, &name)?;
                    visit_struct(schema, i, state, order, path, depth + 1)?;
                }
            } else if bt == BaseType::BASE_TYPE_ARRAY {
                let et = ty.element_type.unwrap_or(BaseType::BASE_TYPE_NONE);
                if et == BaseType::BASE_TYPE_STRUCT {
                    if let Some(dep_idx) = ty.index {
                        let i = checked_obj_index(dep_idx, &schema.objects, &name)?;
                        visit_struct(schema, i, state, order, path, depth + 1)?;
                    }
                }
            }
        }
    }

    path.pop();
    state[idx] = 2;
    order.push(idx);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_up() {
        assert_eq!(align_up(0, 4), 0);
        assert_eq!(align_up(1, 4), 4);
        assert_eq!(align_up(4, 4), 4);
        assert_eq!(align_up(5, 4), 8);
        assert_eq!(align_up(0, 1), 0);
        assert_eq!(align_up(3, 1), 3);
        assert_eq!(align_up(3, 8), 8);
    }

    #[test]
    fn test_type_size_align_scalars() {
        let objects = vec![];
        let mut ty = schema::Type::new();

        ty.base_type = Some(BaseType::BASE_TYPE_BYTE);
        assert_eq!(type_size_align(&ty, &objects, "Test").unwrap(), (1, 1));

        ty.base_type = Some(BaseType::BASE_TYPE_SHORT);
        assert_eq!(type_size_align(&ty, &objects, "Test").unwrap(), (2, 2));

        ty.base_type = Some(BaseType::BASE_TYPE_INT);
        assert_eq!(type_size_align(&ty, &objects, "Test").unwrap(), (4, 4));

        ty.base_type = Some(BaseType::BASE_TYPE_DOUBLE);
        assert_eq!(type_size_align(&ty, &objects, "Test").unwrap(), (8, 8));
    }

    #[test]
    fn test_checked_obj_index_negative() {
        let objects = vec![];
        let err = checked_obj_index(-1, &objects, "Bad").unwrap_err();
        assert!(err.to_string().contains("invalid type index -1"));
    }

    #[test]
    fn test_checked_obj_index_out_of_range() {
        let objects = vec![];
        let err = checked_obj_index(5, &objects, "Bad").unwrap_err();
        assert!(err.to_string().contains("invalid type index 5"));
    }
}
