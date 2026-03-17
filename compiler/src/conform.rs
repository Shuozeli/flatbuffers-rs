//! Schema backwards-compatibility checking (--conform).
//!
//! Validates that a "current" schema is a backwards-compatible evolution of a
//! "base" schema. Used in CI/CD to prevent breaking changes.

use flatc_rs_schema::resolved::{ResolvedSchema, ResolvedObject, ResolvedEnum, ResolvedType};

/// A single conformance violation.
#[derive(Debug, Clone)]
pub struct ConformError {
    pub message: String,
}

impl std::fmt::Display for ConformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Check that `current` is a backwards-compatible evolution of `base`.
///
/// Returns Ok(()) if compatible, or a list of violations.
pub fn check_conform(
    current: &ResolvedSchema,
    base: &ResolvedSchema,
) -> Result<(), Vec<ConformError>> {
    let mut errors = Vec::new();

    // Check each object (table/struct) in the base schema
    for base_obj in &base.objects {
        let base_name = &base_obj.name;
        if base_name.is_empty() {
            continue;
        }

        let current_obj = current
            .objects
            .iter()
            .find(|o| o.name == *base_name);
        let current_obj = match current_obj {
            Some(o) => o,
            None => {
                errors.push(ConformError {
                    message: format!("type '{base_name}' was removed"),
                });
                continue;
            }
        };

        // Table/struct kind must not change
        if base_obj.is_struct != current_obj.is_struct {
            let base_kind = if base_obj.is_struct {
                "struct"
            } else {
                "table"
            };
            let curr_kind = if current_obj.is_struct {
                "struct"
            } else {
                "table"
            };
            errors.push(ConformError {
                message: format!("type '{base_name}' changed from {base_kind} to {curr_kind}"),
            });
            continue;
        }

        // Check fields
        check_object_fields(&mut errors, base_name, base_obj, current_obj);
    }

    // Check each enum in the base schema
    for base_enum in &base.enums {
        let base_name = &base_enum.name;
        if base_name.is_empty() {
            continue;
        }

        let current_enum = current
            .enums
            .iter()
            .find(|e| e.name == *base_name);
        let current_enum = match current_enum {
            Some(e) => e,
            None => {
                errors.push(ConformError {
                    message: format!("enum '{base_name}' was removed"),
                });
                continue;
            }
        };

        // Union/enum kind must not change
        if base_enum.is_union != current_enum.is_union {
            errors.push(ConformError {
                message: format!("'{base_name}' changed between enum and union"),
            });
            continue;
        }

        // Check enum values
        check_enum_values(&mut errors, base_name, base_enum, current_enum);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_object_fields(
    errors: &mut Vec<ConformError>,
    type_name: &str,
    base_obj: &ResolvedObject,
    current_obj: &ResolvedObject,
) {
    for base_field in &base_obj.fields {
        let fname = &base_field.name;
        if fname.is_empty() {
            continue;
        }

        let current_field = current_obj
            .fields
            .iter()
            .find(|f| f.name == *fname);
        let current_field = match current_field {
            Some(f) => f,
            None => {
                errors.push(ConformError {
                    message: format!("field '{fname}' was removed from '{type_name}'"),
                });
                continue;
            }
        };

        // Field ID must not change
        if base_field.id != current_field.id {
            errors.push(ConformError {
                message: format!(
                    "field '{type_name}.{fname}' changed id from {:?} to {:?}",
                    base_field.id, current_field.id
                ),
            });
        }

        // Field type must not change
        if !types_compatible(&base_field.type_, &current_field.type_) {
            errors.push(ConformError {
                message: format!("field '{type_name}.{fname}' changed type"),
            });
        }

        // Default values must not change
        if base_field.default_integer != current_field.default_integer {
            errors.push(ConformError {
                message: format!(
                    "field '{type_name}.{fname}' changed default integer from {:?} to {:?}",
                    base_field.default_integer, current_field.default_integer
                ),
            });
        }
        if base_field.default_real != current_field.default_real {
            errors.push(ConformError {
                message: format!(
                    "field '{type_name}.{fname}' changed default real from {:?} to {:?}",
                    base_field.default_real, current_field.default_real
                ),
            });
        }
    }
}

fn check_enum_values(
    errors: &mut Vec<ConformError>,
    enum_name: &str,
    base_enum: &ResolvedEnum,
    current_enum: &ResolvedEnum,
) {
    for base_val in &base_enum.values {
        let vname = &base_val.name;
        if vname.is_empty() {
            continue;
        }

        let current_val = current_enum
            .values
            .iter()
            .find(|v| v.name == *vname);
        let current_val = match current_val {
            Some(v) => v,
            None => {
                errors.push(ConformError {
                    message: format!("enum value '{enum_name}.{vname}' was removed"),
                });
                continue;
            }
        };

        // Value must not change
        if base_val.value != current_val.value {
            errors.push(ConformError {
                message: format!(
                    "enum value '{enum_name}.{vname}' changed from {} to {}",
                    base_val.value, current_val.value
                ),
            });
        }
    }
}

/// Check if two ResolvedType descriptors are compatible.
fn types_compatible(a: &ResolvedType, b: &ResolvedType) -> bool {
    a.base_type == b.base_type && a.element_type == b.element_type && a.index == b.index
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_single;

    fn compile(src: &str) -> ResolvedSchema {
        compile_single(src).unwrap().schema
    }

    #[test]
    fn compatible_evolution_adding_fields() {
        let base = compile("table Foo { a:int; } root_type Foo;");
        let current = compile("table Foo { a:int; b:string; } root_type Foo;");
        assert!(check_conform(&current, &base).is_ok());
    }

    #[test]
    fn compatible_evolution_adding_enum_values() {
        let base =
            compile("enum Color:byte { Red = 0, Green = 1 } table T { c:Color; } root_type T;");
        let current = compile(
            "enum Color:byte { Red = 0, Green = 1, Blue = 2 } table T { c:Color; } root_type T;",
        );
        assert!(check_conform(&current, &base).is_ok());
    }

    #[test]
    fn error_removed_field() {
        let base = compile("table Foo { a:int; b:string; } root_type Foo;");
        let current = compile("table Foo { a:int; } root_type Foo;");
        let errs = check_conform(&current, &base).unwrap_err();
        assert!(errs.iter().any(|e| e.message.contains("'b' was removed")));
    }

    #[test]
    fn error_removed_type() {
        let base = compile("table Foo { a:int; } table Bar { x:int; } root_type Foo;");
        let current = compile("table Foo { a:int; } root_type Foo;");
        let errs = check_conform(&current, &base).unwrap_err();
        assert!(errs.iter().any(|e| e.message.contains("'Bar' was removed")));
    }

    #[test]
    fn error_changed_field_type() {
        let base = compile("table Foo { a:int; } root_type Foo;");
        let current = compile("table Foo { a:string; } root_type Foo;");
        let errs = check_conform(&current, &base).unwrap_err();
        assert!(errs.iter().any(|e| e.message.contains("changed type")));
    }

    #[test]
    fn error_removed_enum_value() {
        let base = compile(
            "enum Color:byte { Red = 0, Green = 1, Blue = 2 } table T { c:Color; } root_type T;",
        );
        let current =
            compile("enum Color:byte { Red = 0, Green = 1 } table T { c:Color; } root_type T;");
        let errs = check_conform(&current, &base).unwrap_err();
        assert!(errs
            .iter()
            .any(|e| e.message.contains("Blue") && e.message.contains("was removed")));
    }

    #[test]
    fn error_changed_enum_value() {
        let base =
            compile("enum Color:byte { Red = 0, Green = 1 } table T { c:Color; } root_type T;");
        let current =
            compile("enum Color:byte { Red = 0, Green = 2 } table T { c:Color; } root_type T;");
        let errs = check_conform(&current, &base).unwrap_err();
        assert!(errs
            .iter()
            .any(|e| e.message.contains("Green") && e.message.contains("changed")));
    }

    #[test]
    fn error_struct_to_table() {
        let base = compile("struct Pos { x:float; y:float; } table T { p:Pos; } root_type T;");
        let current = compile("table Pos { x:float; y:float; } table T { p:Pos; } root_type T;");
        let errs = check_conform(&current, &base).unwrap_err();
        assert!(errs
            .iter()
            .any(|e| e.message.contains("changed from struct to table")));
    }
}
