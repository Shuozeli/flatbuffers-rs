//! Schema API tests -- verify that after analysis, the Schema object has correct
//! field-level properties, struct layout, enum values, attributes, documentation,
//! services, and type resolution.

use flatc_rs_compiler::compile_single;
use flatc_rs_schema::BaseType;
use flatc_rs_schema::resolved::ResolvedSchema;

fn analyze(src: &str) -> ResolvedSchema {
    compile_single(src).unwrap().schema
}

// ---------------------------------------------------------------------------
// Field properties: id, default values, deprecated, required, key, optional
// ---------------------------------------------------------------------------

#[test]
fn field_ids_auto_assigned() {
    let s = analyze("table T { a:int; b:string; c:float; }");
    let t = &s.objects[0];
    assert_eq!(t.fields[0].id, Some(0));
    assert_eq!(t.fields[1].id, Some(1));
    assert_eq!(t.fields[2].id, Some(2));
}

#[test]
fn field_ids_explicit() {
    let s = analyze("table T { a:int (id: 2); b:string (id: 0); c:float (id: 1); }");
    let t = &s.objects[0];
    // Fields are reordered by id after analysis
    let by_name = |name: &str| {
        t.fields
            .iter()
            .find(|f| f.name == name)
            .unwrap()
    };
    assert_eq!(by_name("a").id, Some(2));
    assert_eq!(by_name("b").id, Some(0));
    assert_eq!(by_name("c").id, Some(1));
}

#[test]
fn field_default_integer() {
    let s = analyze("table T { hp:int = 100; mana:short = -10; }");
    let t = &s.objects[0];
    assert_eq!(t.fields[0].default_integer, Some(100));
    assert_eq!(t.fields[1].default_integer, Some(-10));
}

#[test]
fn field_default_real() {
    let s = analyze("table T { speed:float = 1.5; ratio:double = 3.14; }");
    let t = &s.objects[0];
    assert_eq!(t.fields[0].default_real, Some(1.5));
    assert_eq!(t.fields[1].default_real, Some(3.14));
}

#[test]
fn field_default_bool() {
    let s = analyze("table T { active:bool = true; hidden:bool = false; }");
    let t = &s.objects[0];
    assert_eq!(t.fields[0].default_integer, Some(1));
    assert_eq!(t.fields[1].default_integer, Some(0));
}

#[test]
fn field_default_enum() {
    let s = analyze("enum Color:byte { Red, Green, Blue }\ntable T { c:Color = Green; }");
    let t = &s.objects[0];
    // Enum defaults are stored as the string name, resolved to integer by codegen
    assert!(
        t.fields[0].default_integer == Some(1)
            || t.fields[0].default_string.as_deref() == Some("Green")
    );
}

#[test]
fn field_no_default() {
    let s = analyze("table T { x:int; }");
    let t = &s.objects[0];
    assert_eq!(t.fields[0].default_integer, None);
    assert_eq!(t.fields[0].default_real, None);
}

#[test]
fn field_deprecated() {
    let s = analyze("table T { old:int (deprecated); current:int; }");
    let t = &s.objects[0];
    assert!(t.fields[0].is_deprecated);
    assert!(!t.fields[1].is_deprecated);
}

#[test]
fn field_required() {
    let s = analyze("table T { name:string (required); opt:string; }");
    let t = &s.objects[0];
    assert!(t.fields[0].is_required);
    assert!(!t.fields[1].is_required);
}

#[test]
fn field_key() {
    let s = analyze("table T { id:int (key); name:string; }");
    let t = &s.objects[0];
    assert!(t.fields[0].is_key);
    assert!(!t.fields[1].is_key);
}

#[test]
fn field_optional_scalar() {
    let s = analyze("table T { x:int = null; y:int; }");
    let t = &s.objects[0];
    assert!(t.fields[0].is_optional);
    assert!(!t.fields[1].is_optional);
}

// ---------------------------------------------------------------------------
// Type resolution
// ---------------------------------------------------------------------------

#[test]
fn type_scalar_int() {
    let s = analyze("table T { x:int; }");
    let ty = s.objects[0].fields[0].type_;
    assert_eq!(ty.base_type, BaseType::BASE_TYPE_INT);
    assert_eq!(ty.base_size, Some(4));
}

#[test]
fn type_string() {
    let s = analyze("table T { name:string; }");
    let ty = s.objects[0].fields[0].type_;
    assert_eq!(ty.base_type, BaseType::BASE_TYPE_STRING);
}

#[test]
fn type_vector_of_scalars() {
    let s = analyze("table T { data:[int]; }");
    let ty = s.objects[0].fields[0].type_;
    assert_eq!(ty.base_type, BaseType::BASE_TYPE_VECTOR);
    assert_eq!(ty.element_type, Some(BaseType::BASE_TYPE_INT));
}

#[test]
fn type_vector_of_strings() {
    let s = analyze("table T { names:[string]; }");
    let ty = s.objects[0].fields[0].type_;
    assert_eq!(ty.base_type, BaseType::BASE_TYPE_VECTOR);
    assert_eq!(ty.element_type, Some(BaseType::BASE_TYPE_STRING));
}

#[test]
fn type_vector_of_tables() {
    let s = analyze("table Item { v:int; }\ntable T { items:[Item]; }");
    let ty = s.objects[1].fields[0].type_;
    assert_eq!(ty.base_type, BaseType::BASE_TYPE_VECTOR);
    assert_eq!(ty.element_type, Some(BaseType::BASE_TYPE_TABLE));
    assert_eq!(ty.index, Some(0)); // Item is objects[0]
}

#[test]
fn type_table_reference() {
    let s = analyze("table Inner { x:int; }\ntable Outer { inner:Inner; }");
    let ty = s.objects[1].fields[0].type_;
    assert_eq!(ty.base_type, BaseType::BASE_TYPE_TABLE);
    assert_eq!(ty.index, Some(0));
}

#[test]
fn type_struct_reference() {
    let s = analyze("struct Vec2 { x:float; y:float; }\ntable T { pos:Vec2; }");
    let ty = s.objects[1].fields[0].type_;
    assert_eq!(ty.base_type, BaseType::BASE_TYPE_STRUCT);
    assert_eq!(ty.index, Some(0));
}

#[test]
fn type_fixed_array() {
    let s = analyze("struct S { data:[int:4]; }");
    let ty = s.objects[0].fields[0].type_;
    assert_eq!(ty.base_type, BaseType::BASE_TYPE_ARRAY);
    assert_eq!(ty.element_type, Some(BaseType::BASE_TYPE_INT));
    assert_eq!(ty.fixed_length, Some(4));
}

#[test]
fn type_enum_field_uses_underlying() {
    let s = analyze("enum Color:short { Red, Green, Blue }\ntable T { c:Color; }");
    let ty = s.objects[0].fields[0].type_;
    assert_eq!(ty.base_type, BaseType::BASE_TYPE_SHORT);
    assert_eq!(ty.index, Some(0)); // enum index
}

// ---------------------------------------------------------------------------
// Struct layout: byte_size, min_align, field offset, padding
// ---------------------------------------------------------------------------

#[test]
fn struct_layout_basic() {
    let s = analyze("struct Vec3 { x:float; y:float; z:float; }");
    let obj = &s.objects[0];
    assert!(obj.is_struct);
    assert_eq!(obj.byte_size, Some(12)); // 3 * 4
    assert_eq!(obj.min_align, Some(4));
    assert_eq!(obj.fields[0].offset, Some(0));
    assert_eq!(obj.fields[1].offset, Some(4));
    assert_eq!(obj.fields[2].offset, Some(8));
}

#[test]
fn struct_layout_alignment_padding() {
    // byte(1) + padding(3) + int(4) = 8 bytes, align 4
    let s = analyze("struct S { b:byte; i:int; }");
    let obj = &s.objects[0];
    assert_eq!(obj.byte_size, Some(8));
    assert_eq!(obj.min_align, Some(4));
    assert_eq!(obj.fields[0].offset, Some(0));
    assert_eq!(obj.fields[1].offset, Some(4));
}

#[test]
fn struct_layout_double_alignment() {
    // int(4) + padding(4) + double(8) = 16 bytes, align 8
    let s = analyze("struct S { i:int; d:double; }");
    let obj = &s.objects[0];
    assert_eq!(obj.byte_size, Some(16));
    assert_eq!(obj.min_align, Some(8));
    assert_eq!(obj.fields[0].offset, Some(0));
    assert_eq!(obj.fields[1].offset, Some(8));
}

#[test]
fn struct_layout_nested() {
    let s = analyze(
        "struct Vec2 { x:float; y:float; }\n\
         struct Rect { min:Vec2; max:Vec2; }",
    );
    let rect = &s.objects[1];
    assert_eq!(rect.byte_size, Some(16)); // 2 * Vec2(8)
    assert_eq!(rect.min_align, Some(4));
    assert_eq!(rect.fields[0].offset, Some(0));
    assert_eq!(rect.fields[1].offset, Some(8));
}

#[test]
fn struct_force_align() {
    let s = analyze("struct S (force_align: 16) { x:float; y:float; z:float; }");
    let obj = &s.objects[0];
    assert_eq!(obj.min_align, Some(16));
    // byte_size is padded to alignment: 12 -> 16
    assert_eq!(obj.byte_size, Some(16));
}

// ---------------------------------------------------------------------------
// Enum details: values, underlying type, auto-numbering
// ---------------------------------------------------------------------------

#[test]
fn enum_values_auto_numbered() {
    let s = analyze("enum Color:byte { Red, Green, Blue }");
    let e = &s.enums[0];
    assert_eq!(e.name.as_str(), "Color");
    assert!(!e.is_union);
    assert_eq!(e.values.len(), 3);
    assert_eq!(e.values[0].name.as_str(), "Red");
    assert_eq!(e.values[0].value, 0);
    assert_eq!(e.values[1].name.as_str(), "Green");
    assert_eq!(e.values[1].value, 1);
    assert_eq!(e.values[2].name.as_str(), "Blue");
    assert_eq!(e.values[2].value, 2);
}

#[test]
fn enum_values_explicit() {
    let s = analyze("enum Status:int { OK = 0, NotFound = 404, Error = 500 }");
    let e = &s.enums[0];
    assert_eq!(e.values[0].value, 0);
    assert_eq!(e.values[1].value, 404);
    assert_eq!(e.values[2].value, 500);
}

#[test]
fn enum_values_mixed_auto() {
    let s = analyze("enum E:byte { A = 5, B, C, D = 10, E }");
    let e = &s.enums[0];
    assert_eq!(e.values[0].value, 5);
    assert_eq!(e.values[1].value, 6);
    assert_eq!(e.values[2].value, 7);
    assert_eq!(e.values[3].value, 10);
    assert_eq!(e.values[4].value, 11);
}

#[test]
fn enum_underlying_type() {
    let s = analyze("enum E:ushort { A, B }");
    let e = &s.enums[0];
    let ut = e.underlying_type;
    assert_eq!(ut.base_type, BaseType::BASE_TYPE_U_SHORT);
}

#[test]
fn enum_negative_values() {
    let s = analyze("enum E:int { Neg = -10, Zero = 0, Pos = 10 }");
    let e = &s.enums[0];
    assert_eq!(e.values[0].value, -10);
    assert_eq!(e.values[1].value, 0);
    assert_eq!(e.values[2].value, 10);
}

#[test]
fn enum_bitflags() {
    let s = analyze("enum Flags:ubyte (bit_flags) { Read, Write, Execute }");
    let e = &s.enums[0];
    assert_eq!(e.values[0].value, 0); // bit position 0
    assert_eq!(e.values[1].value, 1); // bit position 1
    assert_eq!(e.values[2].value, 2); // bit position 2
                                            // Verify bit_flags attribute is present
    let attrs = e.attributes.as_ref().unwrap();
    assert!(attrs
        .entries
        .iter()
        .any(|kv| kv.key.as_deref() == Some("bit_flags")));
}

// ---------------------------------------------------------------------------
// Union details
// ---------------------------------------------------------------------------

#[test]
fn union_variants() {
    let s = analyze(
        "table A { x:int; }\n\
         table B { y:string; }\n\
         union U { A, B }\n\
         table Root { u:U; }",
    );
    let u = &s.enums[0];
    assert_eq!(u.name.as_str(), "U");
    assert!(u.is_union);

    // NONE variant is auto-inserted at index 0
    assert_eq!(u.values[0].name.as_str(), "NONE");
    assert_eq!(u.values[0].value, 0);

    // A variant
    assert_eq!(u.values[1].name.as_str(), "A");
    assert_eq!(u.values[1].value, 1);
    let a_type = u.values[1].union_type.as_ref().unwrap();
    assert_eq!(a_type.base_type, BaseType::BASE_TYPE_TABLE);
    assert_eq!(a_type.index, Some(0)); // objects[0] = A

    // B variant
    assert_eq!(u.values[2].name.as_str(), "B");
    assert_eq!(u.values[2].value, 2);
    let b_type = u.values[2].union_type.as_ref().unwrap();
    assert_eq!(b_type.base_type, BaseType::BASE_TYPE_TABLE);
    assert_eq!(b_type.index, Some(1)); // objects[1] = B
}

#[test]
fn union_companion_type_field() {
    let s = analyze(
        "table A { x:int; }\n\
         union U { A }\n\
         table Root { equipped:U; }",
    );
    let root = s
        .objects
        .iter()
        .find(|o| o.name == "Root")
        .unwrap();

    // Companion _type field is auto-inserted before the union field
    assert_eq!(root.fields.len(), 2);
    assert_eq!(root.fields[0].name.as_str(), "equipped_type");
    assert_eq!(
        root.fields[0].type_.base_type,
        BaseType::BASE_TYPE_U_TYPE
    );
    assert_eq!(root.fields[1].name.as_str(), "equipped");
    assert_eq!(
        root.fields[1].type_.base_type,
        BaseType::BASE_TYPE_UNION
    );
}

// ---------------------------------------------------------------------------
// Services and RPC
// ---------------------------------------------------------------------------

#[test]
fn service_rpc_methods() {
    let s = analyze(
        "table Req { data:string; }\n\
         table Resp { result:int; }\n\
         rpc_service Svc { DoThing(Req):Resp; }",
    );
    assert_eq!(s.services.len(), 1);
    let svc = &s.services[0];
    assert_eq!(svc.name.as_str(), "Svc");

    assert_eq!(svc.calls.len(), 1);
    let call = &svc.calls[0];
    assert_eq!(call.name.as_str(), "DoThing");
    assert_eq!(call.request_index, 0); // objects[0] = Req
    assert_eq!(call.response_index, 1); // objects[1] = Resp
}

#[test]
fn service_multiple_methods() {
    let s = analyze(
        "table Req { data:string; }\n\
         table Resp { result:int; }\n\
         rpc_service Svc {\n\
           Get(Req):Resp;\n\
           Set(Req):Resp;\n\
           Delete(Req):Resp;\n\
         }",
    );
    let svc = &s.services[0];
    assert_eq!(svc.calls.len(), 3);
    assert_eq!(svc.calls[0].name.as_str(), "Get");
    assert_eq!(svc.calls[1].name.as_str(), "Set");
    assert_eq!(svc.calls[2].name.as_str(), "Delete");
}

// ---------------------------------------------------------------------------
// Attributes
// ---------------------------------------------------------------------------

#[test]
fn table_custom_attribute() {
    let s = analyze("attribute \"priority\";\ntable T (priority: \"high\") { x:int; }");
    let attrs = s.objects[0].attributes.as_ref().unwrap();
    let kv = attrs
        .entries
        .iter()
        .find(|e| e.key.as_deref() == Some("priority"))
        .unwrap();
    // Attribute values preserve quotes from the .fbs source
    assert!(kv.value.as_deref() == Some("high") || kv.value.as_deref() == Some("\"high\""));
}

#[test]
fn field_attribute_deprecated_and_key() {
    let s = analyze("table T { id:int (key); old:string (deprecated); }");
    let t = &s.objects[0];

    // key
    assert!(t.fields[0].is_key);

    // deprecated
    assert!(t.fields[1].is_deprecated);
}

#[test]
fn nested_flatbuffer_attribute() {
    let s = analyze(
        "table Inner { x:int; }\n\
         table Outer { data:[ubyte] (nested_flatbuffer: \"Inner\"); }",
    );
    let outer = &s.objects[1];
    let attrs = outer.fields[0].attributes.as_ref().unwrap();
    let nf = attrs
        .entries
        .iter()
        .find(|e| e.key.as_deref() == Some("nested_flatbuffer"))
        .unwrap();
    // Attribute values preserve quotes from the .fbs source
    assert!(nf.value.as_deref() == Some("Inner") || nf.value.as_deref() == Some("\"Inner\""));
}

// ---------------------------------------------------------------------------
// Documentation
// ---------------------------------------------------------------------------

#[test]
fn table_documentation() {
    let s = analyze("/// A documented table.\n/// Second line.\ntable T { x:int; }");
    let doc = s.objects[0].documentation.as_ref().unwrap();
    assert_eq!(doc.lines.len(), 2);
    assert!(doc.lines[0].contains("A documented table."));
    assert!(doc.lines[1].contains("Second line."));
}

#[test]
fn field_documentation() {
    let s = analyze("table T {\n  /// The x coordinate.\n  x:int;\n}");
    let doc = s.objects[0].fields[0].documentation.as_ref().unwrap();
    assert!(doc.lines[0].contains("x coordinate"));
}

#[test]
fn enum_documentation() {
    let s = analyze("/// Color enum.\nenum Color:byte { Red, Green, Blue }");
    let doc = s.enums[0].documentation.as_ref().unwrap();
    assert!(doc.lines[0].contains("Color enum"));
}

#[test]
fn enum_value_documentation() {
    let s = analyze("enum Color:byte {\n  /// The color red.\n  Red,\n  Green\n}");
    let doc = s.enums[0].values[0].documentation.as_ref().unwrap();
    assert!(doc.lines[0].contains("color red"));
    assert!(s.enums[0].values[1].documentation.is_none());
}

// ---------------------------------------------------------------------------
// Namespace
// ---------------------------------------------------------------------------

#[test]
fn object_namespace() {
    let s = analyze("namespace Game.World;\ntable Monster { hp:int; }");
    let ns = s.objects[0].namespace.as_ref().unwrap();
    assert_eq!(ns.namespace.as_deref(), Some("Game.World"));
}

#[test]
fn enum_namespace() {
    let s = analyze("namespace Game;\nenum Color:byte { Red }");
    let ns = s.enums[0].namespace.as_ref().unwrap();
    assert_eq!(ns.namespace.as_deref(), Some("Game"));
}

#[test]
fn cross_namespace_type_resolution() {
    let s = analyze(
        "namespace A;\nstruct Vec2 { x:float; y:float; }\n\
         namespace B;\ntable T { pos:A.Vec2; }",
    );
    let t = s
        .objects
        .iter()
        .find(|o| o.name == "T")
        .unwrap();
    let ty = t.fields[0].type_;
    assert_eq!(ty.base_type, BaseType::BASE_TYPE_STRUCT);
    // index should point to Vec2
    let vec2_idx = s
        .objects
        .iter()
        .position(|o| o.name == "Vec2")
        .unwrap();
    assert_eq!(ty.index, Some(vec2_idx as i32));
}

#[test]
fn service_namespace() {
    let s = analyze(
        "namespace Api;\n\
         table Req { x:int; }\n\
         table Resp { y:int; }\n\
         rpc_service Handler { Do(Req):Resp; }",
    );
    let ns = s.services[0].namespace.as_ref().unwrap();
    assert_eq!(ns.namespace.as_deref(), Some("Api"));
}

// ---------------------------------------------------------------------------
// File-level metadata
// ---------------------------------------------------------------------------

#[test]
fn file_identifier() {
    let s = analyze("file_identifier \"TEST\";\ntable T { x:int; }\nroot_type T;");
    assert_eq!(s.file_ident.as_deref(), Some("TEST"));
}

#[test]
fn file_extension() {
    let s = analyze("file_extension \"dat\";\ntable T { x:int; }");
    assert_eq!(s.file_ext.as_deref(), Some("dat"));
}

#[test]
fn root_table_set() {
    let s = analyze("table A { x:int; }\ntable B { y:int; }\nroot_type B;");
    assert_eq!(s.objects[s.root_table_index.unwrap()].name.as_str(), "B");
}

#[test]
fn no_root_table() {
    let s = analyze("table T { x:int; }");
    assert!(s.root_table_index.is_none());
}

// ---------------------------------------------------------------------------
// Table vs struct distinction
// ---------------------------------------------------------------------------

#[test]
fn table_is_not_struct() {
    let s = analyze("table T { x:int; }");
    assert!(!s.objects[0].is_struct);
}

#[test]
fn struct_is_struct() {
    let s = analyze("struct S { x:int; }");
    assert!(s.objects[0].is_struct);
}

// ---------------------------------------------------------------------------
// Source location (Span)
// ---------------------------------------------------------------------------

#[test]
fn object_span() {
    let s = analyze("table Monster { hp:int; }");
    let span = s.objects[0].span.as_ref().unwrap();
    assert_eq!(span.line, 1);
    assert_eq!(span.col, 1);
}

#[test]
fn field_span() {
    let s = analyze("table T {\n  x:int;\n  y:string;\n}");
    let f0 = s.objects[0].fields[0].span.as_ref().unwrap();
    assert_eq!(f0.line, 2);
    let f1 = s.objects[0].fields[1].span.as_ref().unwrap();
    assert_eq!(f1.line, 3);
}

#[test]
fn enum_span() {
    let s = analyze("enum Color:byte { Red }");
    let span = s.enums[0].span.as_ref().unwrap();
    assert_eq!(span.line, 1);
}

#[test]
fn enum_value_span() {
    let s = analyze("enum Color:byte {\n  Red,\n  Green\n}");
    let v0 = s.enums[0].values[0].span.as_ref().unwrap();
    assert_eq!(v0.line, 2);
    let v1 = s.enums[0].values[1].span.as_ref().unwrap();
    assert_eq!(v1.line, 3);
}

#[test]
fn service_span() {
    let s = analyze(
        "table Req { x:int; }\ntable Resp { y:int; }\n\
         rpc_service Svc { Do(Req):Resp; }",
    );
    let span = s.services[0].span.as_ref().unwrap();
    assert_eq!(span.line, 3);
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn multiple_enums_independent() {
    let s = analyze(
        "enum A:byte { X, Y }\n\
         enum B:short { P = 100, Q = 200 }",
    );
    assert_eq!(s.enums.len(), 2);
    assert_eq!(s.enums[0].name.as_str(), "A");
    assert_eq!(s.enums[1].name.as_str(), "B");
    assert_eq!(
        s.enums[1].underlying_type.base_type,
        BaseType::BASE_TYPE_SHORT
    );
}

#[test]
fn table_with_all_scalar_types() {
    let s = analyze(
        "table T {\n\
           b:bool; i8:byte; u8:ubyte; i16:short; u16:ushort;\n\
           i32:int; u32:uint; i64:long; u64:ulong;\n\
           f32:float; f64:double;\n\
         }",
    );
    let t = &s.objects[0];
    assert_eq!(t.fields.len(), 11);
    let types: Vec<BaseType> = t
        .fields
        .iter()
        .map(|f| f.type_.base_type)
        .collect();
    assert_eq!(
        types,
        vec![
            BaseType::BASE_TYPE_BOOL,
            BaseType::BASE_TYPE_BYTE,
            BaseType::BASE_TYPE_U_BYTE,
            BaseType::BASE_TYPE_SHORT,
            BaseType::BASE_TYPE_U_SHORT,
            BaseType::BASE_TYPE_INT,
            BaseType::BASE_TYPE_U_INT,
            BaseType::BASE_TYPE_LONG,
            BaseType::BASE_TYPE_U_LONG,
            BaseType::BASE_TYPE_FLOAT,
            BaseType::BASE_TYPE_DOUBLE,
        ]
    );
}

#[test]
fn forward_type_reference() {
    let s = analyze("table A { b:B; }\ntable B { x:int; }");
    let ty = s.objects[0].fields[0].type_;
    assert_eq!(ty.base_type, BaseType::BASE_TYPE_TABLE);
    assert_eq!(ty.index, Some(1)); // B is objects[1]
}

#[test]
fn struct_with_enum_field() {
    let s = analyze("enum Color:byte { Red, Green, Blue }\nstruct S { c:Color; }");
    let ty = s.objects[0].fields[0].type_;
    assert_eq!(ty.base_type, BaseType::BASE_TYPE_BYTE);
    assert_eq!(ty.index, Some(0)); // Color is enums[0]
}

#[test]
fn vector_of_enums() {
    let s = analyze("enum Color:byte { Red }\ntable T { colors:[Color]; }");
    let ty = s.objects[0].fields[0].type_;
    assert_eq!(ty.base_type, BaseType::BASE_TYPE_VECTOR);
    assert_eq!(ty.element_type, Some(BaseType::BASE_TYPE_BYTE));
    assert_eq!(ty.index, Some(0)); // Color enum index
}

#[test]
fn vector_of_structs() {
    let s = analyze("struct Vec2 { x:float; y:float; }\ntable T { points:[Vec2]; }");
    let ty = s.objects[1].fields[0].type_;
    assert_eq!(ty.base_type, BaseType::BASE_TYPE_VECTOR);
    assert_eq!(ty.element_type, Some(BaseType::BASE_TYPE_STRUCT));
    assert_eq!(ty.index, Some(0)); // Vec2 is objects[0]
}

#[test]
fn empty_table() {
    let s = analyze("table Empty { }");
    assert_eq!(s.objects[0].fields.len(), 0);
    assert!(!s.objects[0].is_struct);
}

#[test]
fn private_attribute() {
    let s = analyze("table T (private) { x:int; }");
    let attrs = s.objects[0].attributes.as_ref().unwrap();
    assert!(attrs
        .entries
        .iter()
        .any(|kv| kv.key.as_deref() == Some("private")));
}
