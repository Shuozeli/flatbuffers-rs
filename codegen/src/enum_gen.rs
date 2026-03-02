use flatc_rs_schema::{self as schema, BaseType};

use super::code_writer::CodeWriter;
use super::type_map;
use super::CodeGenOptions;

/// Check if an enum has a specific attribute (e.g., "bit_flags").
fn has_attribute(enum_def: &schema::Enum, key: &str) -> bool {
    enum_def
        .attributes
        .as_ref()
        .is_some_and(|attrs| attrs.entries.iter().any(|e| e.key.as_deref() == Some(key)))
}

/// Generate Rust code for the enum at `schema.enums[index]`.
pub fn generate(w: &mut CodeWriter, schema: &schema::Schema, index: usize, opts: &CodeGenOptions) {
    let enum_def = &schema.enums[index];
    let is_bitflags = has_attribute(enum_def, "bit_flags");

    if is_bitflags {
        generate_bitflags(w, enum_def, opts);
    } else {
        generate_regular(w, enum_def, opts);
    }

    // Object API: generate union T enum for union types
    if opts.gen_object_api && enum_def.is_union == Some(true) {
        w.blank();
        gen_union_object_api(w, schema, index, opts);
    }
}

/// Generate a bitflags enum using the `bitflags!` macro.
fn generate_bitflags(w: &mut CodeWriter, enum_def: &schema::Enum, opts: &CodeGenOptions) {
    let name = enum_def.name.as_deref().unwrap_or("");
    let underlying_bt = type_map::get_base_type(enum_def.underlying_type.as_ref());
    let rust_type = type_map::scalar_rust_type(underlying_bt);
    let mod_name = format!("bitflags_{}", type_map::to_snake_case(name));

    // Wrap in a private module to avoid name conflicts with the bitflags! macro
    w.line("#[allow(non_upper_case_globals)]");
    w.block(&format!("mod {mod_name}"), |w| {
        // bitflags! macro invocation
        w.block("::flatbuffers::bitflags::bitflags!", |w| {
            // Documentation
            if let Some(doc) = &enum_def.documentation {
                for line in &doc.lines {
                    w.line(&format!("/// {line}"));
                }
            }

            w.line("#[derive(Default, Debug, Clone, Copy, PartialEq)]");
            w.block(&format!("pub struct {name}: {rust_type}"), |w| {
                for val in &enum_def.values {
                    let vname = val.name.as_deref().unwrap_or("");
                    let bit_pos = val.value.unwrap_or(0);
                    let bit_val: u64 = 1u64 << bit_pos;
                    // Documentation for individual values
                    if let Some(doc) = &val.documentation {
                        for line in &doc.lines {
                            w.line(&format!("/// {line}"));
                        }
                    }
                    w.line(&format!("const {vname} = {bit_val};"));
                }
            });
        });
    });
    w.line(&format!("pub use self::{mod_name}::{name};"));

    if opts.rust_serialize {
        w.blank();
        w.block(&format!("impl ::serde::Serialize for {name}"), |w| {
            w.block(
                "fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>\nwhere S: ::serde::Serializer",
                |w| {
                    w.line(&format!("serializer.serialize_{rust_type}(self.bits())"));
                },
            );
        });

        w.blank();
        w.block(&format!("impl<'de> ::serde::Deserialize<'de> for {name}"), |w| {
            w.block(
                "fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>\nwhere D: ::serde::Deserializer<'de>",
                |w| {
                    w.line(&format!("let b = {rust_type}::deserialize(deserializer)?;"));
                    w.line("Ok(Self::from_bits_retain(b))");
                },
            );
        });
    }

    w.blank();

    // Follow impl - uses from_bits_retain instead of Self()
    w.block(
        &format!("impl<'a> ::flatbuffers::Follow<'a> for {name}"),
        |w| {
            w.line("type Inner = Self;");
            w.line("#[inline]");
            w.block(
                "unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner",
                |w| {
                    w.line(&format!(
                        "let b = unsafe {{ ::flatbuffers::read_scalar_at::<{rust_type}>(buf, loc) }};"
                    ));
                    w.line("Self::from_bits_retain(b)");
                },
            );
        },
    );
    w.blank();

    // Push impl - uses .bits() instead of .0, 4-space indentation like C++
    w.line(&format!("impl ::flatbuffers::Push for {name} {{"));
    w.line(&format!("    type Output = {name};"));
    w.line("    #[inline]");
    w.line("    unsafe fn push(&self, dst: &mut [u8], _written_len: usize) {");
    w.line(&format!(
        "        unsafe {{ ::flatbuffers::emplace_scalar::<{rust_type}>(dst, self.bits()) }};"
    ));
    w.line("    }");
    w.line("}");
    w.blank();

    // EndianScalar impl - uses .bits() and from_bits_retain
    w.block(
        &format!("impl ::flatbuffers::EndianScalar for {name}"),
        |w| {
            w.line(&format!("type Scalar = {rust_type};"));
            w.line("#[inline]");
            w.block(&format!("fn to_little_endian(self) -> {rust_type}"), |w| {
                w.line("self.bits().to_le()");
            });
            w.line("#[inline]");
            w.line("#[allow(clippy::wrong_self_convention)]");
            w.block(
                &format!("fn from_little_endian(v: {rust_type}) -> Self"),
                |w| {
                    w.line(&format!("let b = {rust_type}::from_le(v);"));
                    w.line("Self::from_bits_retain(b)");
                },
            );
        },
    );
    w.blank();

    // Verifiable impl - multi-line run_verifier signature like C++
    w.line(&format!("impl<'a> ::flatbuffers::Verifiable for {name} {{"));
    w.line("  #[inline]");
    w.line("  fn run_verifier(");
    w.line("    v: &mut ::flatbuffers::Verifier, pos: usize");
    w.line("  ) -> Result<(), ::flatbuffers::InvalidFlatbuffer> {");
    w.line(&format!("    {rust_type}::run_verifier(v, pos)"));
    w.line("  }");
    w.line("}");
    w.blank();

    // SimpleToVerifyInSlice marker
    w.line(&format!(
        "impl ::flatbuffers::SimpleToVerifyInSlice for {name} {{}}"
    ));
}

/// Generate a regular (non-bitflags) enum.
fn generate_regular(w: &mut CodeWriter, enum_def: &schema::Enum, opts: &CodeGenOptions) {
    let name = enum_def.name.as_deref().unwrap_or("");
    let is_union = enum_def.is_union == Some(true);

    let underlying_bt = type_map::get_base_type(enum_def.underlying_type.as_ref());
    let rust_type = type_map::scalar_rust_type(underlying_bt);

    // Deprecated global constants (non-union enums only, matching C++ flatc)
    if !is_union && !enum_def.values.is_empty() {
        let min_val = enum_def
            .values
            .iter()
            .map(|v| v.value.unwrap_or(0))
            .min()
            .unwrap_or(0);
        let max_val = enum_def
            .values
            .iter()
            .map(|v| v.value.unwrap_or(0))
            .max()
            .unwrap_or(0);
        let upper_name = name.to_uppercase();
        let depr = "#[deprecated(since = \"2.0.0\", note = \"Use associated constants instead. This will no longer be generated in 2021.\")]";
        w.line(depr);
        w.line(&format!(
            "pub const ENUM_MIN_{upper_name}: {rust_type} = {min_val};"
        ));
        w.line(depr);
        w.line(&format!(
            "pub const ENUM_MAX_{upper_name}: {rust_type} = {max_val};"
        ));
        w.line(depr);
        w.line("#[allow(non_camel_case_types)]");
        let n = enum_def.values.len();
        w.line(&format!(
            "pub const ENUM_VALUES_{upper_name}: [{name}; {n}] = ["
        ));
        w.indent();
        for val in &enum_def.values {
            let vname = val.name.as_deref().unwrap_or("");
            let sanitized = type_map::sanitize_union_const_name(vname);
            let esc = type_map::escape_keyword(&sanitized);
            w.line(&format!("{name}::{esc},"));
        }
        w.dedent();
        w.line("];");
        w.blank();
    }

    // Struct definition
    w.line("#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]");
    w.line("#[repr(transparent)]");
    w.line(&format!("pub struct {name}(pub {rust_type});"));

    // Impl block with constants
    w.line("#[allow(non_upper_case_globals)]");
    w.block(&format!("impl {name}"), |w| {
        // Variant constants
        for val in &enum_def.values {
            let vname = val.name.as_deref().unwrap_or("");
            // Sanitize FQN: "MyGame.Example2.Monster" -> "MyGame_Example2_Monster"
            let sanitized = type_map::sanitize_union_const_name(vname);
            let escaped = type_map::escape_keyword(&sanitized);
            let vval = val.value.unwrap_or(0);
            w.line(&format!("pub const {escaped}: Self = Self({vval});"));
        }

        if !enum_def.values.is_empty() {
            w.blank();
            // ENUM_MIN / ENUM_MAX
            let min_val = enum_def
                .values
                .iter()
                .map(|v| v.value.unwrap_or(0))
                .min()
                .unwrap_or(0);
            let max_val = enum_def
                .values
                .iter()
                .map(|v| v.value.unwrap_or(0))
                .max()
                .unwrap_or(0);
            w.line(&format!("pub const ENUM_MIN: {rust_type} = {min_val};"));
            w.line(&format!("pub const ENUM_MAX: {rust_type} = {max_val};"));

            // ENUM_VALUES
            let vals: Vec<String> = enum_def
                .values
                .iter()
                .map(|v| {
                    let sanitized =
                        type_map::sanitize_union_const_name(v.name.as_deref().unwrap_or(""));
                    let esc = type_map::escape_keyword(&sanitized);
                    format!("Self::{esc}")
                })
                .collect();
            w.line("pub const ENUM_VALUES: &'static [Self] = &[");
            w.indent();
            for val in &vals {
                w.line(&format!("{val},"));
            }
            w.dedent();
            w.line("];");
        }

        // variant_name()
        w.line("/// Returns the variant's name or \"\" if unknown.");
        w.block("pub fn variant_name(self) -> Option<&'static str>", |w| {
            w.block("match self", |w| {
                for val in &enum_def.values {
                    let vname = val.name.as_deref().unwrap_or("");
                    let sanitized = type_map::sanitize_union_const_name(vname);
                    let escaped = type_map::escape_keyword(&sanitized);
                    // Return sanitized name as string
                    w.line(&format!("Self::{escaped} => Some(\"{sanitized}\"),"));
                }
                w.line("_ => None,");
            });
        });
    });

    // Debug impl
    w.block(&format!("impl ::core::fmt::Debug for {name}"), |w| {
        w.block(
            "fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result",
            |w| {
                w.line("if let Some(name) = self.variant_name() {");
                w.indent();
                w.line("f.write_str(name)");
                w.dedent();
                w.line("} else {");
                w.indent();
                w.line("f.write_fmt(format_args!(\"<UNKNOWN {:?}>\", self.0))");
                w.dedent();
                w.line("}");
            },
        );
    });

    if opts.rust_serialize {
        w.blank();
        w.block(&format!("impl ::serde::Serialize for {name}"), |w| {
            w.block(
                "fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>\nwhere S: ::serde::Serializer",
                |w| {
                    w.line("if let Some(name) = self.variant_name() {");
                    w.indent();
                    w.line(&format!("serializer.serialize_unit_variant(\"{name}\", self.0 as u32, name)"));
                    w.dedent();
                    w.line("} else {");
                    w.indent();
                    w.line(&format!("serializer.serialize_{rust_type}(self.0)"));
                    w.dedent();
                    w.line("}");
                },
            );
        });

        w.blank();
        w.block(&format!("impl<'de> ::serde::Deserialize<'de> for {name}"), |w| {
            w.block(
                "fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>\nwhere D: ::serde::Deserializer<'de>",
                |w| {
                    w.line("struct EnumVisitor;");
                    w.block("impl<'de> ::serde::de::Visitor<'de> for EnumVisitor", |w| {
                        w.line(&format!("type Value = {name};"));
                        w.block("fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result", |w| {
                            w.line(&format!("formatter.write_str(\"a {name} variant\")"));
                        });
                        w.block("fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>\nwhere E: ::serde::de::Error", |w| {
                            w.line(&format!("for item in {name}::ENUM_VALUES {{"));
                            w.indent();
                            w.line("if let Some(name) = item.variant_name() {");
                            w.indent();
                            w.line("if name == value { return Ok(*item); }");
                            w.dedent();
                            w.line("}");
                            w.dedent();
                            w.line("}");
                            w.line("Err(E::custom(format!(\"unknown variant: {}\", value)))");
                        });
                        w.block(&format!("fn visit_{rust_type}<E>(self, value: {rust_type}) -> Result<Self::Value, E>\nwhere E: ::serde::de::Error"), |w| {
                            w.line(&format!("Ok({name}(value))"));
                        });
                        w.block("fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>\nwhere E: ::serde::de::Error", |w| {
                            w.line(&format!("Ok({name}(value as {rust_type}))"));
                        });
                        w.block("fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>\nwhere E: ::serde::de::Error", |w| {
                            w.line(&format!("Ok({name}(value as {rust_type}))"));
                        });
                    });
                    w.line("deserializer.deserialize_any(EnumVisitor)");
                },
            );
        });
    }

    // Follow impl
    w.block(
        &format!("impl<'a> ::flatbuffers::Follow<'a> for {name}"),
        |w| {
            w.line("type Inner = Self;");
            w.line("#[inline]");
            w.block(
                "unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner",
                |w| {
                    w.line(&format!(
                        "let b = unsafe {{ ::flatbuffers::read_scalar_at::<{rust_type}>(buf, loc) }};"
                    ));
                    w.line("Self(b)");
                },
            );
        },
    );

    w.blank();
    // Push impl - C++ uses 4-space indentation inside Push body
    w.line(&format!("impl ::flatbuffers::Push for {name} {{"));
    w.line(&format!("    type Output = {name};"));
    w.line("    #[inline]");
    w.line("    unsafe fn push(&self, dst: &mut [u8], _written_len: usize) {");
    w.line(&format!(
        "        unsafe {{ ::flatbuffers::emplace_scalar::<{rust_type}>(dst, self.0) }};"
    ));
    w.line("    }");
    w.line("}");
    w.blank();

    // EndianScalar impl
    w.block(
        &format!("impl ::flatbuffers::EndianScalar for {name}"),
        |w| {
            w.line(&format!("type Scalar = {rust_type};"));
            w.line("#[inline]");
            w.block(&format!("fn to_little_endian(self) -> {rust_type}"), |w| {
                w.line("self.0.to_le()");
            });
            w.line("#[inline]");
            w.line("#[allow(clippy::wrong_self_convention)]");
            w.block(
                &format!("fn from_little_endian(v: {rust_type}) -> Self"),
                |w| {
                    w.line(&format!("let b = {rust_type}::from_le(v);"));
                    w.line("Self(b)");
                },
            );
        },
    );
    w.blank();

    // Verifiable impl - multi-line run_verifier signature like C++
    w.line(&format!("impl<'a> ::flatbuffers::Verifiable for {name} {{"));
    w.line("  #[inline]");
    w.line("  fn run_verifier(");
    w.line("    v: &mut ::flatbuffers::Verifier, pos: usize");
    w.line("  ) -> Result<(), ::flatbuffers::InvalidFlatbuffer> {");
    w.line(&format!("    {rust_type}::run_verifier(v, pos)"));
    w.line("  }");
    w.line("}");
    w.blank();

    // SimpleToVerifyInSlice marker
    w.line(&format!(
        "impl ::flatbuffers::SimpleToVerifyInSlice for {name} {{}}"
    ));

    // For union types: also generate marker struct for table offset
    if is_union {
        w.blank();
        w.line(&format!("pub struct {name}UnionTableOffset {{}}"));
    }
}

/// Generate the Object API union T enum for a union type.
fn gen_union_object_api(
    w: &mut CodeWriter,
    schema: &schema::Schema,
    index: usize,
    opts: &CodeGenOptions,
) {
    let enum_def = &schema.enums[index];
    let name = enum_def.name.as_deref().unwrap_or("");
    let t_name = format!("{name}T");
    let current_ns = enum_def
        .namespace
        .as_ref()
        .and_then(|n| n.namespace.as_deref())
        .unwrap_or("");

    // Enum definition
    w.line("#[non_exhaustive]");
    let mut derives = vec!["Debug", "Clone", "PartialEq"];
    if opts.rust_serialize {
        derives.push("::serde::Serialize");
        derives.push("::serde::Deserialize");
    }
    w.line(&format!("#[derive({})]", derives.join(", ")));
    w.block(&format!("pub enum {t_name}"), |w| {
        w.line("NONE,");
        for val in &enum_def.values {
            let vname = val.name.as_deref().unwrap_or("");
            if vname == "NONE" {
                continue;
            }
            // T enum variants use PascalCase: "MyGame.Example2.Monster" -> "MyGameExample2Monster"
            let t_variant = type_map::escape_keyword(&type_map::fqn_to_pascal(vname));
            let variant_bt = type_map::get_base_type(val.union_type.as_ref());
            if variant_bt == BaseType::BASE_TYPE_TABLE {
                let table_idx = val.union_type.as_ref().and_then(|t| t.index).unwrap_or(0) as usize;
                let table_name = type_map::resolve_object_name(schema, current_ns, table_idx);
                w.line(&format!("{t_variant}(alloc::boxed::Box<{table_name}T>),"));
            }
        }
    });
    w.blank();

    // Default impl
    w.block(&format!("impl Default for {t_name}"), |w| {
        w.block("fn default() -> Self", |w| {
            w.line("Self::NONE");
        });
    });
    w.blank();

    // Type discriminator method
    let snake = type_map::to_snake_case(name);
    w.block(&format!("impl {t_name}"), |w| {
        w.block(&format!("pub fn {snake}_type(&self) -> {name}"), |w| {
            w.block("match self", |w| {
                w.line(&format!("Self::NONE => {name}::NONE,"));
                for val in &enum_def.values {
                    let vname = val.name.as_deref().unwrap_or("");
                    if vname == "NONE" {
                        continue;
                    }
                    // T variant (PascalCase) maps to regular enum constant (underscores)
                    let t_variant = type_map::escape_keyword(&type_map::fqn_to_pascal(vname));
                    let const_name = type_map::escape_keyword(&type_map::sanitize_union_const_name(vname));
                    w.line(&format!("Self::{t_variant}(_) => {name}::{const_name},"));
                }
            });
        });
        w.blank();

        // Pack method
        w.block(
            "pub fn pack<'b, A: ::flatbuffers::Allocator + 'b>(\n    &self,\n    fbb: &mut ::flatbuffers::FlatBufferBuilder<'b, A>,\n  ) -> Option<::flatbuffers::WIPOffset<::flatbuffers::UnionWIPOffset>>",
            |w| {
                w.block("match self", |w| {
                    w.line("Self::NONE => None,");
                    for val in &enum_def.values {
                        let vname = val.name.as_deref().unwrap_or("");
                        if vname == "NONE" {
                            continue;
                        }
                        let t_variant = type_map::escape_keyword(&type_map::fqn_to_pascal(vname));
                        w.line(&format!(
                            "Self::{t_variant}(v) => Some(v.pack(fbb).as_union_value()),"
                        ));
                    }
                });
            },
        );
    });
}
