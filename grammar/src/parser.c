#include "tree_sitter/parser.h"

#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic ignored "-Wmissing-field-initializers"
#endif

#ifdef _MSC_VER
#pragma optimize("", off)
#elif defined(__clang__)
#pragma clang optimize off
#elif defined(__GNUC__)
#pragma GCC optimize ("O0")
#endif

#define LANGUAGE_VERSION 14
#define STATE_COUNT 323
#define LARGE_STATE_COUNT 2
#define SYMBOL_COUNT 109
#define ALIAS_COUNT 0
#define TOKEN_COUNT 64
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 38
#define MAX_ALIAS_SEQUENCE_LENGTH 11
#define PRODUCTION_ID_COUNT 94

enum ts_symbol_identifiers {
  sym_documentation = 1,
  anon_sym_include = 2,
  anon_sym_SEMI = 3,
  anon_sym_native_include = 4,
  anon_sym_namespace = 5,
  anon_sym_attribute = 6,
  anon_sym_DQUOTE = 7,
  anon_sym_table = 8,
  anon_sym_struct = 9,
  anon_sym_LBRACE = 10,
  anon_sym_RBRACE = 11,
  anon_sym_enum = 12,
  anon_sym_COLON = 13,
  anon_sym_COMMA = 14,
  anon_sym_union = 15,
  anon_sym_root_type = 16,
  anon_sym_file_extension = 17,
  anon_sym_file_identifier = 18,
  anon_sym_LBRACK = 19,
  anon_sym_RBRACK = 20,
  anon_sym_EQ = 21,
  anon_sym_bool = 22,
  anon_sym_byte = 23,
  anon_sym_ubyte = 24,
  anon_sym_short = 25,
  anon_sym_ushort = 26,
  anon_sym_int = 27,
  anon_sym_uint = 28,
  anon_sym_float = 29,
  anon_sym_long = 30,
  anon_sym_ulong = 31,
  anon_sym_double = 32,
  anon_sym_int8 = 33,
  anon_sym_uint8 = 34,
  anon_sym_int16 = 35,
  anon_sym_uint16 = 36,
  anon_sym_int32 = 37,
  anon_sym_uint32 = 38,
  anon_sym_int64 = 39,
  anon_sym_uint64 = 40,
  anon_sym_float32 = 41,
  anon_sym_float64 = 42,
  anon_sym_string = 43,
  anon_sym_rpc_service = 44,
  anon_sym_LPAREN = 45,
  anon_sym_RPAREN = 46,
  sym_identifier = 47,
  anon_sym_DOT = 48,
  sym_plus_token = 49,
  sym_minus_token = 50,
  sym_true = 51,
  sym_false = 52,
  aux_sym_decimal_lit_token1 = 53,
  anon_sym_0 = 54,
  sym_hex_lit = 55,
  aux_sym_float_lit_token1 = 56,
  anon_sym_infinity = 57,
  anon_sym_inf = 58,
  sym_nan_token = 59,
  aux_sym_string_constant_token1 = 60,
  aux_sym_string_constant_token2 = 61,
  sym_escape_sequence = 62,
  sym_comment = 63,
  sym_source_file = 64,
  sym_include = 65,
  sym_native_include = 66,
  sym_namespace_decl = 67,
  sym_attribute_decl = 68,
  sym_type_decl = 69,
  sym_enum_decl = 70,
  sym_union_decl = 71,
  sym_object = 72,
  sym_root_decl = 73,
  sym_file_extension_decl = 74,
  sym_file_identifier_decl = 75,
  sym_value = 76,
  sym_enum_val_decl = 77,
  sym_field_decl = 78,
  sym_union_field_decl = 79,
  sym_type = 80,
  sym_rpc_decl = 81,
  sym_rpc_method = 82,
  sym_metadata = 83,
  sym_field_and_value = 84,
  sym_single_value = 85,
  sym_full_ident = 86,
  sym_scalar = 87,
  sym_bool_constant = 88,
  sym_int_constant = 89,
  sym_float_constant = 90,
  sym_plus_minus_constant = 91,
  sym_int_lit = 92,
  sym_decimal_lit = 93,
  sym_float_lit = 94,
  sym_inf_token = 95,
  sym_string_constant = 96,
  aux_sym_source_file_repeat1 = 97,
  aux_sym_source_file_repeat2 = 98,
  aux_sym_type_decl_repeat1 = 99,
  aux_sym_type_decl_repeat2 = 100,
  aux_sym_enum_decl_repeat1 = 101,
  aux_sym_union_decl_repeat1 = 102,
  aux_sym_object_repeat1 = 103,
  aux_sym_value_repeat1 = 104,
  aux_sym_rpc_decl_repeat1 = 105,
  aux_sym_metadata_repeat1 = 106,
  aux_sym_full_ident_repeat1 = 107,
  aux_sym_string_constant_repeat1 = 108,
};

static const char * const ts_symbol_names[] = {
  [ts_builtin_sym_end] = "end",
  [sym_documentation] = "documentation",
  [anon_sym_include] = "include",
  [anon_sym_SEMI] = ";",
  [anon_sym_native_include] = "native_include",
  [anon_sym_namespace] = "namespace",
  [anon_sym_attribute] = "attribute",
  [anon_sym_DQUOTE] = "\"",
  [anon_sym_table] = "table",
  [anon_sym_struct] = "struct",
  [anon_sym_LBRACE] = "{",
  [anon_sym_RBRACE] = "}",
  [anon_sym_enum] = "enum",
  [anon_sym_COLON] = ":",
  [anon_sym_COMMA] = ",",
  [anon_sym_union] = "union",
  [anon_sym_root_type] = "root_type",
  [anon_sym_file_extension] = "file_extension",
  [anon_sym_file_identifier] = "file_identifier",
  [anon_sym_LBRACK] = "[",
  [anon_sym_RBRACK] = "]",
  [anon_sym_EQ] = "=",
  [anon_sym_bool] = "bool",
  [anon_sym_byte] = "byte",
  [anon_sym_ubyte] = "ubyte",
  [anon_sym_short] = "short",
  [anon_sym_ushort] = "ushort",
  [anon_sym_int] = "int",
  [anon_sym_uint] = "uint",
  [anon_sym_float] = "float",
  [anon_sym_long] = "long",
  [anon_sym_ulong] = "ulong",
  [anon_sym_double] = "double",
  [anon_sym_int8] = "int8",
  [anon_sym_uint8] = "uint8",
  [anon_sym_int16] = "int16",
  [anon_sym_uint16] = "uint16",
  [anon_sym_int32] = "int32",
  [anon_sym_uint32] = "uint32",
  [anon_sym_int64] = "int64",
  [anon_sym_uint64] = "uint64",
  [anon_sym_float32] = "float32",
  [anon_sym_float64] = "float64",
  [anon_sym_string] = "string",
  [anon_sym_rpc_service] = "rpc_service",
  [anon_sym_LPAREN] = "(",
  [anon_sym_RPAREN] = ")",
  [sym_identifier] = "identifier",
  [anon_sym_DOT] = ".",
  [sym_plus_token] = "plus_token",
  [sym_minus_token] = "minus_token",
  [sym_true] = "true",
  [sym_false] = "false",
  [aux_sym_decimal_lit_token1] = "decimal_lit_token1",
  [anon_sym_0] = "0",
  [sym_hex_lit] = "hex_lit",
  [aux_sym_float_lit_token1] = "float_lit_token1",
  [anon_sym_infinity] = "infinity",
  [anon_sym_inf] = "inf",
  [sym_nan_token] = "nan_token",
  [aux_sym_string_constant_token1] = "string_constant_token1",
  [aux_sym_string_constant_token2] = "string_constant_token2",
  [sym_escape_sequence] = "escape_sequence",
  [sym_comment] = "comment",
  [sym_source_file] = "source_file",
  [sym_include] = "include",
  [sym_native_include] = "native_include",
  [sym_namespace_decl] = "namespace_decl",
  [sym_attribute_decl] = "attribute_decl",
  [sym_type_decl] = "type_decl",
  [sym_enum_decl] = "enum_decl",
  [sym_union_decl] = "union_decl",
  [sym_object] = "object",
  [sym_root_decl] = "root_decl",
  [sym_file_extension_decl] = "file_extension_decl",
  [sym_file_identifier_decl] = "file_identifier_decl",
  [sym_value] = "value",
  [sym_enum_val_decl] = "enum_val_decl",
  [sym_field_decl] = "field_decl",
  [sym_union_field_decl] = "union_field_decl",
  [sym_type] = "type",
  [sym_rpc_decl] = "rpc_decl",
  [sym_rpc_method] = "rpc_method",
  [sym_metadata] = "metadata",
  [sym_field_and_value] = "field_and_value",
  [sym_single_value] = "single_value",
  [sym_full_ident] = "full_ident",
  [sym_scalar] = "scalar",
  [sym_bool_constant] = "bool_constant",
  [sym_int_constant] = "int_constant",
  [sym_float_constant] = "float_constant",
  [sym_plus_minus_constant] = "plus_minus_constant",
  [sym_int_lit] = "int_lit",
  [sym_decimal_lit] = "decimal_lit",
  [sym_float_lit] = "float_lit",
  [sym_inf_token] = "inf_token",
  [sym_string_constant] = "string_constant",
  [aux_sym_source_file_repeat1] = "source_file_repeat1",
  [aux_sym_source_file_repeat2] = "source_file_repeat2",
  [aux_sym_type_decl_repeat1] = "type_decl_repeat1",
  [aux_sym_type_decl_repeat2] = "type_decl_repeat2",
  [aux_sym_enum_decl_repeat1] = "enum_decl_repeat1",
  [aux_sym_union_decl_repeat1] = "union_decl_repeat1",
  [aux_sym_object_repeat1] = "object_repeat1",
  [aux_sym_value_repeat1] = "value_repeat1",
  [aux_sym_rpc_decl_repeat1] = "rpc_decl_repeat1",
  [aux_sym_metadata_repeat1] = "metadata_repeat1",
  [aux_sym_full_ident_repeat1] = "full_ident_repeat1",
  [aux_sym_string_constant_repeat1] = "string_constant_repeat1",
};

static const TSSymbol ts_symbol_map[] = {
  [ts_builtin_sym_end] = ts_builtin_sym_end,
  [sym_documentation] = sym_documentation,
  [anon_sym_include] = anon_sym_include,
  [anon_sym_SEMI] = anon_sym_SEMI,
  [anon_sym_native_include] = anon_sym_native_include,
  [anon_sym_namespace] = anon_sym_namespace,
  [anon_sym_attribute] = anon_sym_attribute,
  [anon_sym_DQUOTE] = anon_sym_DQUOTE,
  [anon_sym_table] = anon_sym_table,
  [anon_sym_struct] = anon_sym_struct,
  [anon_sym_LBRACE] = anon_sym_LBRACE,
  [anon_sym_RBRACE] = anon_sym_RBRACE,
  [anon_sym_enum] = anon_sym_enum,
  [anon_sym_COLON] = anon_sym_COLON,
  [anon_sym_COMMA] = anon_sym_COMMA,
  [anon_sym_union] = anon_sym_union,
  [anon_sym_root_type] = anon_sym_root_type,
  [anon_sym_file_extension] = anon_sym_file_extension,
  [anon_sym_file_identifier] = anon_sym_file_identifier,
  [anon_sym_LBRACK] = anon_sym_LBRACK,
  [anon_sym_RBRACK] = anon_sym_RBRACK,
  [anon_sym_EQ] = anon_sym_EQ,
  [anon_sym_bool] = anon_sym_bool,
  [anon_sym_byte] = anon_sym_byte,
  [anon_sym_ubyte] = anon_sym_ubyte,
  [anon_sym_short] = anon_sym_short,
  [anon_sym_ushort] = anon_sym_ushort,
  [anon_sym_int] = anon_sym_int,
  [anon_sym_uint] = anon_sym_uint,
  [anon_sym_float] = anon_sym_float,
  [anon_sym_long] = anon_sym_long,
  [anon_sym_ulong] = anon_sym_ulong,
  [anon_sym_double] = anon_sym_double,
  [anon_sym_int8] = anon_sym_int8,
  [anon_sym_uint8] = anon_sym_uint8,
  [anon_sym_int16] = anon_sym_int16,
  [anon_sym_uint16] = anon_sym_uint16,
  [anon_sym_int32] = anon_sym_int32,
  [anon_sym_uint32] = anon_sym_uint32,
  [anon_sym_int64] = anon_sym_int64,
  [anon_sym_uint64] = anon_sym_uint64,
  [anon_sym_float32] = anon_sym_float32,
  [anon_sym_float64] = anon_sym_float64,
  [anon_sym_string] = anon_sym_string,
  [anon_sym_rpc_service] = anon_sym_rpc_service,
  [anon_sym_LPAREN] = anon_sym_LPAREN,
  [anon_sym_RPAREN] = anon_sym_RPAREN,
  [sym_identifier] = sym_identifier,
  [anon_sym_DOT] = anon_sym_DOT,
  [sym_plus_token] = sym_plus_token,
  [sym_minus_token] = sym_minus_token,
  [sym_true] = sym_true,
  [sym_false] = sym_false,
  [aux_sym_decimal_lit_token1] = aux_sym_decimal_lit_token1,
  [anon_sym_0] = anon_sym_0,
  [sym_hex_lit] = sym_hex_lit,
  [aux_sym_float_lit_token1] = aux_sym_float_lit_token1,
  [anon_sym_infinity] = anon_sym_infinity,
  [anon_sym_inf] = anon_sym_inf,
  [sym_nan_token] = sym_nan_token,
  [aux_sym_string_constant_token1] = aux_sym_string_constant_token1,
  [aux_sym_string_constant_token2] = aux_sym_string_constant_token2,
  [sym_escape_sequence] = sym_escape_sequence,
  [sym_comment] = sym_comment,
  [sym_source_file] = sym_source_file,
  [sym_include] = sym_include,
  [sym_native_include] = sym_native_include,
  [sym_namespace_decl] = sym_namespace_decl,
  [sym_attribute_decl] = sym_attribute_decl,
  [sym_type_decl] = sym_type_decl,
  [sym_enum_decl] = sym_enum_decl,
  [sym_union_decl] = sym_union_decl,
  [sym_object] = sym_object,
  [sym_root_decl] = sym_root_decl,
  [sym_file_extension_decl] = sym_file_extension_decl,
  [sym_file_identifier_decl] = sym_file_identifier_decl,
  [sym_value] = sym_value,
  [sym_enum_val_decl] = sym_enum_val_decl,
  [sym_field_decl] = sym_field_decl,
  [sym_union_field_decl] = sym_union_field_decl,
  [sym_type] = sym_type,
  [sym_rpc_decl] = sym_rpc_decl,
  [sym_rpc_method] = sym_rpc_method,
  [sym_metadata] = sym_metadata,
  [sym_field_and_value] = sym_field_and_value,
  [sym_single_value] = sym_single_value,
  [sym_full_ident] = sym_full_ident,
  [sym_scalar] = sym_scalar,
  [sym_bool_constant] = sym_bool_constant,
  [sym_int_constant] = sym_int_constant,
  [sym_float_constant] = sym_float_constant,
  [sym_plus_minus_constant] = sym_plus_minus_constant,
  [sym_int_lit] = sym_int_lit,
  [sym_decimal_lit] = sym_decimal_lit,
  [sym_float_lit] = sym_float_lit,
  [sym_inf_token] = sym_inf_token,
  [sym_string_constant] = sym_string_constant,
  [aux_sym_source_file_repeat1] = aux_sym_source_file_repeat1,
  [aux_sym_source_file_repeat2] = aux_sym_source_file_repeat2,
  [aux_sym_type_decl_repeat1] = aux_sym_type_decl_repeat1,
  [aux_sym_type_decl_repeat2] = aux_sym_type_decl_repeat2,
  [aux_sym_enum_decl_repeat1] = aux_sym_enum_decl_repeat1,
  [aux_sym_union_decl_repeat1] = aux_sym_union_decl_repeat1,
  [aux_sym_object_repeat1] = aux_sym_object_repeat1,
  [aux_sym_value_repeat1] = aux_sym_value_repeat1,
  [aux_sym_rpc_decl_repeat1] = aux_sym_rpc_decl_repeat1,
  [aux_sym_metadata_repeat1] = aux_sym_metadata_repeat1,
  [aux_sym_full_ident_repeat1] = aux_sym_full_ident_repeat1,
  [aux_sym_string_constant_repeat1] = aux_sym_string_constant_repeat1,
};

static const TSSymbolMetadata ts_symbol_metadata[] = {
  [ts_builtin_sym_end] = {
    .visible = false,
    .named = true,
  },
  [sym_documentation] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_include] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_SEMI] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_native_include] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_namespace] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_attribute] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DQUOTE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_table] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_struct] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LBRACE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RBRACE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_enum] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_COLON] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_COMMA] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_union] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_root_type] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_file_extension] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_file_identifier] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LBRACK] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RBRACK] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_bool] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_byte] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_ubyte] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_short] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_ushort] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_int] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_uint] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_float] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_long] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_ulong] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_double] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_int8] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_uint8] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_int16] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_uint16] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_int32] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_uint32] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_int64] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_uint64] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_float32] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_float64] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_string] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_rpc_service] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LPAREN] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RPAREN] = {
    .visible = true,
    .named = false,
  },
  [sym_identifier] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_DOT] = {
    .visible = true,
    .named = false,
  },
  [sym_plus_token] = {
    .visible = true,
    .named = true,
  },
  [sym_minus_token] = {
    .visible = true,
    .named = true,
  },
  [sym_true] = {
    .visible = true,
    .named = true,
  },
  [sym_false] = {
    .visible = true,
    .named = true,
  },
  [aux_sym_decimal_lit_token1] = {
    .visible = false,
    .named = false,
  },
  [anon_sym_0] = {
    .visible = true,
    .named = false,
  },
  [sym_hex_lit] = {
    .visible = true,
    .named = true,
  },
  [aux_sym_float_lit_token1] = {
    .visible = false,
    .named = false,
  },
  [anon_sym_infinity] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_inf] = {
    .visible = true,
    .named = false,
  },
  [sym_nan_token] = {
    .visible = true,
    .named = true,
  },
  [aux_sym_string_constant_token1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_string_constant_token2] = {
    .visible = false,
    .named = false,
  },
  [sym_escape_sequence] = {
    .visible = true,
    .named = true,
  },
  [sym_comment] = {
    .visible = true,
    .named = true,
  },
  [sym_source_file] = {
    .visible = true,
    .named = true,
  },
  [sym_include] = {
    .visible = true,
    .named = true,
  },
  [sym_native_include] = {
    .visible = true,
    .named = true,
  },
  [sym_namespace_decl] = {
    .visible = true,
    .named = true,
  },
  [sym_attribute_decl] = {
    .visible = true,
    .named = true,
  },
  [sym_type_decl] = {
    .visible = true,
    .named = true,
  },
  [sym_enum_decl] = {
    .visible = true,
    .named = true,
  },
  [sym_union_decl] = {
    .visible = true,
    .named = true,
  },
  [sym_object] = {
    .visible = true,
    .named = true,
  },
  [sym_root_decl] = {
    .visible = true,
    .named = true,
  },
  [sym_file_extension_decl] = {
    .visible = true,
    .named = true,
  },
  [sym_file_identifier_decl] = {
    .visible = true,
    .named = true,
  },
  [sym_value] = {
    .visible = true,
    .named = true,
  },
  [sym_enum_val_decl] = {
    .visible = true,
    .named = true,
  },
  [sym_field_decl] = {
    .visible = true,
    .named = true,
  },
  [sym_union_field_decl] = {
    .visible = true,
    .named = true,
  },
  [sym_type] = {
    .visible = true,
    .named = true,
  },
  [sym_rpc_decl] = {
    .visible = true,
    .named = true,
  },
  [sym_rpc_method] = {
    .visible = true,
    .named = true,
  },
  [sym_metadata] = {
    .visible = true,
    .named = true,
  },
  [sym_field_and_value] = {
    .visible = true,
    .named = true,
  },
  [sym_single_value] = {
    .visible = true,
    .named = true,
  },
  [sym_full_ident] = {
    .visible = true,
    .named = true,
  },
  [sym_scalar] = {
    .visible = true,
    .named = true,
  },
  [sym_bool_constant] = {
    .visible = true,
    .named = true,
  },
  [sym_int_constant] = {
    .visible = true,
    .named = true,
  },
  [sym_float_constant] = {
    .visible = true,
    .named = true,
  },
  [sym_plus_minus_constant] = {
    .visible = true,
    .named = true,
  },
  [sym_int_lit] = {
    .visible = true,
    .named = true,
  },
  [sym_decimal_lit] = {
    .visible = true,
    .named = true,
  },
  [sym_float_lit] = {
    .visible = true,
    .named = true,
  },
  [sym_inf_token] = {
    .visible = true,
    .named = true,
  },
  [sym_string_constant] = {
    .visible = true,
    .named = true,
  },
  [aux_sym_source_file_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_source_file_repeat2] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_type_decl_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_type_decl_repeat2] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_enum_decl_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_union_decl_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_object_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_value_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_rpc_decl_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_metadata_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_full_ident_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_string_constant_repeat1] = {
    .visible = false,
    .named = false,
  },
};

enum ts_field_identifiers {
  field_array_type = 1,
  field_array_type_fixed_length = 2,
  field_array_value = 3,
  field_array_value_item = 4,
  field_attribute_name = 5,
  field_documentation = 6,
  field_enum_int_constant = 7,
  field_enum_key = 8,
  field_enum_name = 9,
  field_enum_type = 10,
  field_enum_val_decl = 11,
  field_field_and_value = 12,
  field_field_declaration = 13,
  field_field_key = 14,
  field_field_type = 15,
  field_field_value = 16,
  field_file_extension_constant = 17,
  field_file_identifier_constant = 18,
  field_full_ident = 19,
  field_include_name = 20,
  field_metadata = 21,
  field_namespace_ident = 22,
  field_object_value = 23,
  field_root_type_ident = 24,
  field_rpc_method = 25,
  field_rpc_method_name = 26,
  field_rpc_name = 27,
  field_rpc_parameter = 28,
  field_rpc_return_type = 29,
  field_scalar_value = 30,
  field_single_value = 31,
  field_string_constant = 32,
  field_table_or_struct_declaration = 33,
  field_table_or_struct_name = 34,
  field_union_field_decl = 35,
  field_union_field_key = 36,
  field_union_field_value = 37,
  field_union_name = 38,
};

static const char * const ts_field_names[] = {
  [0] = NULL,
  [field_array_type] = "array_type",
  [field_array_type_fixed_length] = "array_type_fixed_length",
  [field_array_value] = "array_value",
  [field_array_value_item] = "array_value_item",
  [field_attribute_name] = "attribute_name",
  [field_documentation] = "documentation",
  [field_enum_int_constant] = "enum_int_constant",
  [field_enum_key] = "enum_key",
  [field_enum_name] = "enum_name",
  [field_enum_type] = "enum_type",
  [field_enum_val_decl] = "enum_val_decl",
  [field_field_and_value] = "field_and_value",
  [field_field_declaration] = "field_declaration",
  [field_field_key] = "field_key",
  [field_field_type] = "field_type",
  [field_field_value] = "field_value",
  [field_file_extension_constant] = "file_extension_constant",
  [field_file_identifier_constant] = "file_identifier_constant",
  [field_full_ident] = "full_ident",
  [field_include_name] = "include_name",
  [field_metadata] = "metadata",
  [field_namespace_ident] = "namespace_ident",
  [field_object_value] = "object_value",
  [field_root_type_ident] = "root_type_ident",
  [field_rpc_method] = "rpc_method",
  [field_rpc_method_name] = "rpc_method_name",
  [field_rpc_name] = "rpc_name",
  [field_rpc_parameter] = "rpc_parameter",
  [field_rpc_return_type] = "rpc_return_type",
  [field_scalar_value] = "scalar_value",
  [field_single_value] = "single_value",
  [field_string_constant] = "string_constant",
  [field_table_or_struct_declaration] = "table_or_struct_declaration",
  [field_table_or_struct_name] = "table_or_struct_name",
  [field_union_field_decl] = "union_field_decl",
  [field_union_field_key] = "union_field_key",
  [field_union_field_value] = "union_field_value",
  [field_union_name] = "union_name",
};

static const TSFieldMapSlice ts_field_map_slices[PRODUCTION_ID_COUNT] = {
  [1] = {.index = 0, .length = 1},
  [2] = {.index = 1, .length = 2},
  [3] = {.index = 3, .length = 1},
  [4] = {.index = 4, .length = 1},
  [5] = {.index = 5, .length = 1},
  [6] = {.index = 6, .length = 1},
  [7] = {.index = 7, .length = 1},
  [8] = {.index = 8, .length = 1},
  [9] = {.index = 9, .length = 1},
  [10] = {.index = 10, .length = 2},
  [11] = {.index = 12, .length = 1},
  [12] = {.index = 13, .length = 1},
  [13] = {.index = 14, .length = 2},
  [14] = {.index = 16, .length = 1},
  [15] = {.index = 17, .length = 1},
  [16] = {.index = 18, .length = 1},
  [17] = {.index = 19, .length = 1},
  [18] = {.index = 20, .length = 1},
  [19] = {.index = 21, .length = 3},
  [20] = {.index = 24, .length = 2},
  [21] = {.index = 26, .length = 1},
  [22] = {.index = 27, .length = 3},
  [23] = {.index = 30, .length = 2},
  [24] = {.index = 32, .length = 2},
  [25] = {.index = 34, .length = 2},
  [26] = {.index = 36, .length = 2},
  [27] = {.index = 38, .length = 2},
  [28] = {.index = 40, .length = 3},
  [29] = {.index = 43, .length = 2},
  [30] = {.index = 45, .length = 2},
  [31] = {.index = 47, .length = 2},
  [32] = {.index = 49, .length = 1},
  [33] = {.index = 50, .length = 1},
  [34] = {.index = 51, .length = 2},
  [35] = {.index = 53, .length = 2},
  [36] = {.index = 55, .length = 4},
  [37] = {.index = 59, .length = 1},
  [38] = {.index = 60, .length = 1},
  [39] = {.index = 61, .length = 1},
  [40] = {.index = 62, .length = 3},
  [41] = {.index = 65, .length = 2},
  [42] = {.index = 67, .length = 2},
  [43] = {.index = 69, .length = 3},
  [44] = {.index = 72, .length = 3},
  [45] = {.index = 75, .length = 4},
  [46] = {.index = 79, .length = 4},
  [47] = {.index = 83, .length = 3},
  [48] = {.index = 86, .length = 3},
  [49] = {.index = 89, .length = 3},
  [50] = {.index = 92, .length = 2},
  [51] = {.index = 94, .length = 2},
  [52] = {.index = 96, .length = 3},
  [53] = {.index = 99, .length = 2},
  [54] = {.index = 101, .length = 3},
  [55] = {.index = 104, .length = 3},
  [56] = {.index = 107, .length = 4},
  [57] = {.index = 111, .length = 5},
  [58] = {.index = 116, .length = 4},
  [59] = {.index = 120, .length = 4},
  [60] = {.index = 124, .length = 1},
  [61] = {.index = 125, .length = 1},
  [62] = {.index = 126, .length = 3},
  [63] = {.index = 129, .length = 3},
  [64] = {.index = 132, .length = 2},
  [65] = {.index = 134, .length = 2},
  [66] = {.index = 136, .length = 1},
  [67] = {.index = 137, .length = 4},
  [68] = {.index = 141, .length = 2},
  [69] = {.index = 143, .length = 3},
  [70] = {.index = 146, .length = 4},
  [71] = {.index = 150, .length = 4},
  [72] = {.index = 154, .length = 4},
  [73] = {.index = 158, .length = 5},
  [74] = {.index = 163, .length = 3},
  [75] = {.index = 166, .length = 4},
  [76] = {.index = 170, .length = 3},
  [77] = {.index = 173, .length = 3},
  [78] = {.index = 176, .length = 5},
  [79] = {.index = 181, .length = 5},
  [80] = {.index = 186, .length = 5},
  [81] = {.index = 191, .length = 3},
  [82] = {.index = 194, .length = 4},
  [83] = {.index = 198, .length = 4},
  [84] = {.index = 202, .length = 4},
  [85] = {.index = 206, .length = 3},
  [86] = {.index = 209, .length = 6},
  [87] = {.index = 215, .length = 1},
  [88] = {.index = 216, .length = 5},
  [89] = {.index = 221, .length = 2},
  [90] = {.index = 223, .length = 5},
  [91] = {.index = 228, .length = 4},
  [92] = {.index = 232, .length = 4},
  [93] = {.index = 236, .length = 5},
};

static const TSFieldMapEntry ts_field_map_entries[] = {
  [0] =
    {field_documentation, 0},
  [1] =
    {field_documentation, 0, .inherited = true},
    {field_documentation, 1, .inherited = true},
  [3] =
    {field_include_name, 1},
  [4] =
    {field_namespace_ident, 1},
  [5] =
    {field_attribute_name, 1},
  [6] =
    {field_documentation, 1, .inherited = true},
  [7] =
    {field_root_type_ident, 1},
  [8] =
    {field_file_extension_constant, 1},
  [9] =
    {field_file_identifier_constant, 1},
  [10] =
    {field_table_or_struct_declaration, 0},
    {field_table_or_struct_name, 1},
  [12] =
    {field_field_declaration, 0},
  [13] =
    {field_field_key, 0},
  [14] =
    {field_documentation, 1, .inherited = true},
    {field_documentation, 2, .inherited = true},
  [16] =
    {field_full_ident, 0},
  [17] =
    {field_union_name, 1},
  [18] =
    {field_union_field_key, 0},
  [19] =
    {field_rpc_name, 1},
  [20] =
    {field_attribute_name, 2},
  [21] =
    {field_field_declaration, 3, .inherited = true},
    {field_table_or_struct_declaration, 0},
    {field_table_or_struct_name, 1},
  [24] =
    {field_field_declaration, 0, .inherited = true},
    {field_field_declaration, 1, .inherited = true},
  [26] =
    {field_field_and_value, 1},
  [27] =
    {field_metadata, 2},
    {field_table_or_struct_declaration, 0},
    {field_table_or_struct_name, 1},
  [30] =
    {field_union_field_decl, 3},
    {field_union_name, 1},
  [32] =
    {field_metadata, 1},
    {field_union_field_key, 0},
  [34] =
    {field_documentation, 0, .inherited = true},
    {field_union_field_key, 1},
  [36] =
    {field_metadata, 2},
    {field_union_name, 1},
  [38] =
    {field_rpc_method, 3},
    {field_rpc_name, 1},
  [40] =
    {field_documentation, 0, .inherited = true},
    {field_table_or_struct_declaration, 1},
    {field_table_or_struct_name, 2},
  [43] =
    {field_documentation, 0, .inherited = true},
    {field_union_name, 2},
  [45] =
    {field_documentation, 0, .inherited = true},
    {field_rpc_name, 2},
  [47] =
    {field_field_key, 0},
    {field_field_value, 2},
  [49] =
    {field_scalar_value, 0},
  [50] =
    {field_string_constant, 0},
  [51] =
    {field_field_and_value, 1},
    {field_field_and_value, 2, .inherited = true},
  [53] =
    {field_field_and_value, 0, .inherited = true},
    {field_field_and_value, 1, .inherited = true},
  [55] =
    {field_field_declaration, 4, .inherited = true},
    {field_metadata, 2},
    {field_table_or_struct_declaration, 0},
    {field_table_or_struct_name, 1},
  [59] =
    {field_array_type, 1},
  [60] =
    {field_enum_key, 0},
  [61] =
    {field_union_field_decl, 1},
  [62] =
    {field_union_field_decl, 3},
    {field_union_field_decl, 4, .inherited = true},
    {field_union_name, 1},
  [65] =
    {field_union_field_decl, 0, .inherited = true},
    {field_union_field_decl, 1, .inherited = true},
  [67] =
    {field_union_field_key, 0},
    {field_union_field_value, 2},
  [69] =
    {field_documentation, 0, .inherited = true},
    {field_metadata, 2},
    {field_union_field_key, 1},
  [72] =
    {field_metadata, 2},
    {field_union_field_decl, 4},
    {field_union_name, 1},
  [75] =
    {field_documentation, 0, .inherited = true},
    {field_field_declaration, 4, .inherited = true},
    {field_table_or_struct_declaration, 1},
    {field_table_or_struct_name, 2},
  [79] =
    {field_documentation, 0, .inherited = true},
    {field_metadata, 3},
    {field_table_or_struct_declaration, 1},
    {field_table_or_struct_name, 2},
  [83] =
    {field_documentation, 0, .inherited = true},
    {field_union_field_decl, 4},
    {field_union_name, 2},
  [86] =
    {field_documentation, 0, .inherited = true},
    {field_metadata, 3},
    {field_union_name, 2},
  [89] =
    {field_documentation, 0, .inherited = true},
    {field_rpc_method, 4},
    {field_rpc_name, 2},
  [92] =
    {field_field_key, 0},
    {field_field_type, 2},
  [94] =
    {field_enum_key, 0},
    {field_metadata, 1},
  [96] =
    {field_enum_name, 1},
    {field_enum_type, 3},
    {field_enum_val_decl, 5},
  [99] =
    {field_documentation, 0, .inherited = true},
    {field_enum_key, 1},
  [101] =
    {field_metadata, 3},
    {field_union_field_key, 0},
    {field_union_field_value, 2},
  [104] =
    {field_documentation, 0, .inherited = true},
    {field_union_field_key, 1},
    {field_union_field_value, 3},
  [107] =
    {field_metadata, 2},
    {field_union_field_decl, 4},
    {field_union_field_decl, 5, .inherited = true},
    {field_union_name, 1},
  [111] =
    {field_documentation, 0, .inherited = true},
    {field_field_declaration, 5, .inherited = true},
    {field_metadata, 3},
    {field_table_or_struct_declaration, 1},
    {field_table_or_struct_name, 2},
  [116] =
    {field_documentation, 0, .inherited = true},
    {field_union_field_decl, 4},
    {field_union_field_decl, 5, .inherited = true},
    {field_union_name, 2},
  [120] =
    {field_documentation, 0, .inherited = true},
    {field_metadata, 3},
    {field_union_field_decl, 5},
    {field_union_name, 2},
  [124] =
    {field_object_value, 0},
  [125] =
    {field_single_value, 0},
  [126] =
    {field_field_key, 0},
    {field_field_type, 2},
    {field_metadata, 3},
  [129] =
    {field_documentation, 0, .inherited = true},
    {field_field_key, 1},
    {field_field_type, 3},
  [132] =
    {field_array_type, 1},
    {field_array_type_fixed_length, 3},
  [134] =
    {field_enum_int_constant, 2},
    {field_enum_key, 0},
  [136] =
    {field_enum_val_decl, 1},
  [137] =
    {field_enum_name, 1},
    {field_enum_type, 3},
    {field_enum_val_decl, 5},
    {field_enum_val_decl, 6, .inherited = true},
  [141] =
    {field_enum_val_decl, 0, .inherited = true},
    {field_enum_val_decl, 1, .inherited = true},
  [143] =
    {field_documentation, 0, .inherited = true},
    {field_enum_key, 1},
    {field_metadata, 2},
  [146] =
    {field_enum_name, 1},
    {field_enum_type, 3},
    {field_enum_val_decl, 6},
    {field_metadata, 4},
  [150] =
    {field_documentation, 0, .inherited = true},
    {field_metadata, 4},
    {field_union_field_key, 1},
    {field_union_field_value, 3},
  [154] =
    {field_documentation, 0, .inherited = true},
    {field_enum_name, 2},
    {field_enum_type, 4},
    {field_enum_val_decl, 6},
  [158] =
    {field_documentation, 0, .inherited = true},
    {field_metadata, 3},
    {field_union_field_decl, 5},
    {field_union_field_decl, 6, .inherited = true},
    {field_union_name, 2},
  [163] =
    {field_field_key, 0},
    {field_field_type, 2},
    {field_field_value, 4},
  [166] =
    {field_documentation, 0, .inherited = true},
    {field_field_key, 1},
    {field_field_type, 3},
    {field_metadata, 4},
  [170] =
    {field_enum_int_constant, 2},
    {field_enum_key, 0},
    {field_metadata, 3},
  [173] =
    {field_documentation, 0, .inherited = true},
    {field_enum_int_constant, 3},
    {field_enum_key, 1},
  [176] =
    {field_enum_name, 1},
    {field_enum_type, 3},
    {field_enum_val_decl, 6},
    {field_enum_val_decl, 7, .inherited = true},
    {field_metadata, 4},
  [181] =
    {field_documentation, 0, .inherited = true},
    {field_enum_name, 2},
    {field_enum_type, 4},
    {field_enum_val_decl, 6},
    {field_enum_val_decl, 7, .inherited = true},
  [186] =
    {field_documentation, 0, .inherited = true},
    {field_enum_name, 2},
    {field_enum_type, 4},
    {field_enum_val_decl, 7},
    {field_metadata, 5},
  [191] =
    {field_array_value, 0},
    {field_array_value, 2},
    {field_array_value_item, 1},
  [194] =
    {field_field_key, 0},
    {field_field_type, 2},
    {field_field_value, 4},
    {field_metadata, 5},
  [198] =
    {field_documentation, 0, .inherited = true},
    {field_field_key, 1},
    {field_field_type, 3},
    {field_field_value, 5},
  [202] =
    {field_documentation, 0, .inherited = true},
    {field_enum_int_constant, 3},
    {field_enum_key, 1},
    {field_metadata, 4},
  [206] =
    {field_rpc_method_name, 0},
    {field_rpc_parameter, 2},
    {field_rpc_return_type, 5},
  [209] =
    {field_documentation, 0, .inherited = true},
    {field_enum_name, 2},
    {field_enum_type, 4},
    {field_enum_val_decl, 7},
    {field_enum_val_decl, 8, .inherited = true},
    {field_metadata, 5},
  [215] =
    {field_array_value_item, 1},
  [216] =
    {field_array_value, 0},
    {field_array_value, 2},
    {field_array_value, 3},
    {field_array_value_item, 1},
    {field_array_value_item, 2, .inherited = true},
  [221] =
    {field_array_value_item, 0, .inherited = true},
    {field_array_value_item, 1, .inherited = true},
  [223] =
    {field_documentation, 0, .inherited = true},
    {field_field_key, 1},
    {field_field_type, 3},
    {field_field_value, 5},
    {field_metadata, 6},
  [228] =
    {field_metadata, 6},
    {field_rpc_method_name, 0},
    {field_rpc_parameter, 2},
    {field_rpc_return_type, 5},
  [232] =
    {field_documentation, 0, .inherited = true},
    {field_rpc_method_name, 1},
    {field_rpc_parameter, 3},
    {field_rpc_return_type, 6},
  [236] =
    {field_documentation, 0, .inherited = true},
    {field_metadata, 7},
    {field_rpc_method_name, 1},
    {field_rpc_parameter, 3},
    {field_rpc_return_type, 6},
};

static const TSSymbol ts_alias_sequences[PRODUCTION_ID_COUNT][MAX_ALIAS_SEQUENCE_LENGTH] = {
  [0] = {0},
};

static const uint16_t ts_non_terminal_alias_map[] = {
  0,
};

static const TSStateId ts_primary_state_ids[STATE_COUNT] = {
  [0] = 0,
  [1] = 1,
  [2] = 2,
  [3] = 3,
  [4] = 4,
  [5] = 5,
  [6] = 6,
  [7] = 7,
  [8] = 8,
  [9] = 9,
  [10] = 10,
  [11] = 11,
  [12] = 12,
  [13] = 13,
  [14] = 14,
  [15] = 15,
  [16] = 16,
  [17] = 17,
  [18] = 18,
  [19] = 19,
  [20] = 20,
  [21] = 21,
  [22] = 22,
  [23] = 23,
  [24] = 24,
  [25] = 25,
  [26] = 26,
  [27] = 27,
  [28] = 28,
  [29] = 29,
  [30] = 30,
  [31] = 31,
  [32] = 32,
  [33] = 33,
  [34] = 34,
  [35] = 35,
  [36] = 36,
  [37] = 37,
  [38] = 38,
  [39] = 39,
  [40] = 40,
  [41] = 41,
  [42] = 42,
  [43] = 43,
  [44] = 44,
  [45] = 45,
  [46] = 46,
  [47] = 47,
  [48] = 48,
  [49] = 49,
  [50] = 50,
  [51] = 51,
  [52] = 52,
  [53] = 53,
  [54] = 54,
  [55] = 55,
  [56] = 56,
  [57] = 57,
  [58] = 58,
  [59] = 59,
  [60] = 60,
  [61] = 61,
  [62] = 62,
  [63] = 63,
  [64] = 64,
  [65] = 65,
  [66] = 66,
  [67] = 67,
  [68] = 68,
  [69] = 69,
  [70] = 70,
  [71] = 71,
  [72] = 72,
  [73] = 73,
  [74] = 74,
  [75] = 75,
  [76] = 76,
  [77] = 77,
  [78] = 78,
  [79] = 79,
  [80] = 80,
  [81] = 81,
  [82] = 82,
  [83] = 83,
  [84] = 84,
  [85] = 85,
  [86] = 86,
  [87] = 87,
  [88] = 88,
  [89] = 89,
  [90] = 90,
  [91] = 91,
  [92] = 92,
  [93] = 93,
  [94] = 94,
  [95] = 95,
  [96] = 96,
  [97] = 97,
  [98] = 98,
  [99] = 99,
  [100] = 100,
  [101] = 101,
  [102] = 102,
  [103] = 103,
  [104] = 104,
  [105] = 105,
  [106] = 106,
  [107] = 107,
  [108] = 108,
  [109] = 109,
  [110] = 110,
  [111] = 111,
  [112] = 112,
  [113] = 113,
  [114] = 114,
  [115] = 115,
  [116] = 116,
  [117] = 117,
  [118] = 118,
  [119] = 119,
  [120] = 120,
  [121] = 121,
  [122] = 122,
  [123] = 123,
  [124] = 124,
  [125] = 125,
  [126] = 126,
  [127] = 127,
  [128] = 128,
  [129] = 129,
  [130] = 130,
  [131] = 131,
  [132] = 132,
  [133] = 133,
  [134] = 134,
  [135] = 135,
  [136] = 136,
  [137] = 137,
  [138] = 138,
  [139] = 139,
  [140] = 140,
  [141] = 141,
  [142] = 142,
  [143] = 143,
  [144] = 144,
  [145] = 145,
  [146] = 146,
  [147] = 147,
  [148] = 148,
  [149] = 149,
  [150] = 150,
  [151] = 151,
  [152] = 152,
  [153] = 153,
  [154] = 154,
  [155] = 155,
  [156] = 156,
  [157] = 157,
  [158] = 158,
  [159] = 159,
  [160] = 160,
  [161] = 132,
  [162] = 146,
  [163] = 163,
  [164] = 27,
  [165] = 165,
  [166] = 166,
  [167] = 167,
  [168] = 168,
  [169] = 169,
  [170] = 170,
  [171] = 171,
  [172] = 172,
  [173] = 173,
  [174] = 174,
  [175] = 175,
  [176] = 176,
  [177] = 177,
  [178] = 178,
  [179] = 179,
  [180] = 180,
  [181] = 181,
  [182] = 182,
  [183] = 183,
  [184] = 184,
  [185] = 185,
  [186] = 186,
  [187] = 21,
  [188] = 61,
  [189] = 77,
  [190] = 190,
  [191] = 191,
  [192] = 192,
  [193] = 193,
  [194] = 194,
  [195] = 195,
  [196] = 196,
  [197] = 197,
  [198] = 198,
  [199] = 199,
  [200] = 200,
  [201] = 201,
  [202] = 202,
  [203] = 203,
  [204] = 204,
  [205] = 205,
  [206] = 206,
  [207] = 207,
  [208] = 208,
  [209] = 209,
  [210] = 210,
  [211] = 211,
  [212] = 212,
  [213] = 191,
  [214] = 214,
  [215] = 215,
  [216] = 216,
  [217] = 217,
  [218] = 218,
  [219] = 219,
  [220] = 220,
  [221] = 221,
  [222] = 222,
  [223] = 223,
  [224] = 224,
  [225] = 225,
  [226] = 226,
  [227] = 227,
  [228] = 228,
  [229] = 229,
  [230] = 230,
  [231] = 231,
  [232] = 232,
  [233] = 233,
  [234] = 234,
  [235] = 235,
  [236] = 236,
  [237] = 237,
  [238] = 238,
  [239] = 239,
  [240] = 240,
  [241] = 241,
  [242] = 242,
  [243] = 243,
  [244] = 244,
  [245] = 245,
  [246] = 246,
  [247] = 88,
  [248] = 248,
  [249] = 249,
  [250] = 233,
  [251] = 251,
  [252] = 252,
  [253] = 253,
  [254] = 254,
  [255] = 255,
  [256] = 256,
  [257] = 257,
  [258] = 258,
  [259] = 259,
  [260] = 260,
  [261] = 261,
  [262] = 262,
  [263] = 263,
  [264] = 264,
  [265] = 265,
  [266] = 266,
  [267] = 267,
  [268] = 91,
  [269] = 269,
  [270] = 270,
  [271] = 271,
  [272] = 272,
  [273] = 273,
  [274] = 274,
  [275] = 275,
  [276] = 276,
  [277] = 277,
  [278] = 278,
  [279] = 279,
  [280] = 280,
  [281] = 281,
  [282] = 282,
  [283] = 283,
  [284] = 284,
  [285] = 285,
  [286] = 286,
  [287] = 287,
  [288] = 288,
  [289] = 289,
  [290] = 290,
  [291] = 291,
  [292] = 292,
  [293] = 293,
  [294] = 294,
  [295] = 295,
  [296] = 296,
  [297] = 297,
  [298] = 298,
  [299] = 299,
  [300] = 300,
  [301] = 301,
  [302] = 302,
  [303] = 303,
  [304] = 304,
  [305] = 305,
  [306] = 306,
  [307] = 307,
  [308] = 308,
  [309] = 309,
  [310] = 310,
  [311] = 311,
  [312] = 312,
  [313] = 313,
  [314] = 314,
  [315] = 315,
  [316] = 316,
  [317] = 317,
  [318] = 318,
  [319] = 319,
  [320] = 320,
  [321] = 321,
  [322] = 322,
};

static bool ts_lex(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (eof) ADVANCE(164);
      ADVANCE_MAP(
        '"', 171,
        '\'', 3,
        '(', 231,
        ')', 232,
        '+', 299,
        ',', 178,
        '-', 300,
        '.', 298,
        '/', 5,
        '0', 306,
        ':', 177,
        ';', 167,
        '=', 185,
        '[', 183,
        '\\', 19,
        ']', 184,
        'a', 129,
        'b', 101,
        'd', 99,
        'e', 91,
        'f', 24,
        'i', 87,
        'l', 102,
        'n', 25,
        'r', 106,
        's', 66,
        't', 26,
        'u', 29,
        '{', 174,
        '}', 175,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(162);
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(305);
      END_STATE();
    case 1:
      if (lookahead == '"') ADVANCE(171);
      if (lookahead == '/') ADVANCE(317);
      if (lookahead == '\\') ADVANCE(19);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(320);
      if (lookahead != 0) ADVANCE(321);
      END_STATE();
    case 2:
      if (lookahead == '"') ADVANCE(171);
      if (lookahead == '/') ADVANCE(6);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(2);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 3:
      if (lookahead == '\'') ADVANCE(322);
      if (lookahead != 0) ADVANCE(3);
      END_STATE();
    case 4:
      ADVANCE_MAP(
        '(', 231,
        ')', 232,
        ',', 178,
        '.', 297,
        '/', 6,
        ':', 177,
        ';', 167,
        '=', 185,
        '[', 183,
        ']', 184,
        'b', 275,
        'd', 271,
        'f', 261,
        'i', 268,
        'l', 276,
        's', 255,
        'u', 244,
        '{', 174,
        '}', 175,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(4);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 5:
      if (lookahead == '*') ADVANCE(8);
      if (lookahead == '/') ADVANCE(327);
      END_STATE();
    case 6:
      if (lookahead == '*') ADVANCE(8);
      if (lookahead == '/') ADVANCE(328);
      END_STATE();
    case 7:
      if (lookahead == '*') ADVANCE(7);
      if (lookahead == '/') ADVANCE(326);
      if (lookahead != 0) ADVANCE(8);
      END_STATE();
    case 8:
      if (lookahead == '*') ADVANCE(7);
      if (lookahead != 0) ADVANCE(8);
      END_STATE();
    case 9:
      if (lookahead == '.') ADVANCE(308);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(150);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(9);
      END_STATE();
    case 10:
      if (lookahead == '/') ADVANCE(5);
      if (lookahead == '}') ADVANCE(175);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(10);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 11:
      if (lookahead == '2') ADVANCE(216);
      END_STATE();
    case 12:
      if (lookahead == '2') ADVANCE(218);
      END_STATE();
    case 13:
      if (lookahead == '2') ADVANCE(224);
      END_STATE();
    case 14:
      if (lookahead == '4') ADVANCE(220);
      END_STATE();
    case 15:
      if (lookahead == '4') ADVANCE(222);
      END_STATE();
    case 16:
      if (lookahead == '4') ADVANCE(226);
      END_STATE();
    case 17:
      if (lookahead == '6') ADVANCE(212);
      END_STATE();
    case 18:
      if (lookahead == '6') ADVANCE(214);
      END_STATE();
    case 19:
      if (lookahead == 'U') ADVANCE(161);
      if (lookahead == 'u') ADVANCE(157);
      if (lookahead == 'x') ADVANCE(155);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(325);
      if (lookahead != 0) ADVANCE(323);
      END_STATE();
    case 20:
      if (lookahead == '_') ADVANCE(48);
      END_STATE();
    case 21:
      if (lookahead == '_') ADVANCE(72);
      END_STATE();
    case 22:
      if (lookahead == '_') ADVANCE(131);
      END_STATE();
    case 23:
      if (lookahead == '_') ADVANCE(121);
      END_STATE();
    case 24:
      if (lookahead == 'a') ADVANCE(78);
      if (lookahead == 'i') ADVANCE(80);
      if (lookahead == 'l') ADVANCE(100);
      END_STATE();
    case 25:
      if (lookahead == 'a') ADVANCE(86);
      END_STATE();
    case 26:
      if (lookahead == 'a') ADVANCE(31);
      if (lookahead == 'r') ADVANCE(140);
      END_STATE();
    case 27:
      if (lookahead == 'a') ADVANCE(36);
      END_STATE();
    case 28:
      if (lookahead == 'a') ADVANCE(125);
      END_STATE();
    case 29:
      if (lookahead == 'b') ADVANCE(149);
      if (lookahead == 'i') ADVANCE(97);
      if (lookahead == 'l') ADVANCE(107);
      if (lookahead == 'n') ADVANCE(76);
      if (lookahead == 's') ADVANCE(67);
      END_STATE();
    case 30:
      if (lookahead == 'b') ADVANCE(141);
      END_STATE();
    case 31:
      if (lookahead == 'b') ADVANCE(82);
      END_STATE();
    case 32:
      if (lookahead == 'b') ADVANCE(83);
      END_STATE();
    case 33:
      if (lookahead == 'c') ADVANCE(23);
      END_STATE();
    case 34:
      if (lookahead == 'c') ADVANCE(81);
      if (lookahead == 'f') ADVANCE(313);
      if (lookahead == 't') ADVANCE(196);
      END_STATE();
    case 35:
      if (lookahead == 'c') ADVANCE(127);
      END_STATE();
    case 36:
      if (lookahead == 'c') ADVANCE(51);
      END_STATE();
    case 37:
      if (lookahead == 'c') ADVANCE(53);
      END_STATE();
    case 38:
      if (lookahead == 'c') ADVANCE(84);
      END_STATE();
    case 39:
      if (lookahead == 'd') ADVANCE(49);
      END_STATE();
    case 40:
      if (lookahead == 'd') ADVANCE(61);
      END_STATE();
    case 41:
      if (lookahead == 'd') ADVANCE(54);
      END_STATE();
    case 42:
      if (lookahead == 'e') ADVANCE(188);
      END_STATE();
    case 43:
      if (lookahead == 'e') ADVANCE(301);
      END_STATE();
    case 44:
      if (lookahead == 'e') ADVANCE(303);
      END_STATE();
    case 45:
      if (lookahead == 'e') ADVANCE(172);
      END_STATE();
    case 46:
      if (lookahead == 'e') ADVANCE(190);
      END_STATE();
    case 47:
      if (lookahead == 'e') ADVANCE(206);
      END_STATE();
    case 48:
      if (lookahead == 'e') ADVANCE(146);
      if (lookahead == 'i') ADVANCE(40);
      END_STATE();
    case 49:
      if (lookahead == 'e') ADVANCE(166);
      END_STATE();
    case 50:
      if (lookahead == 'e') ADVANCE(170);
      END_STATE();
    case 51:
      if (lookahead == 'e') ADVANCE(169);
      END_STATE();
    case 52:
      if (lookahead == 'e') ADVANCE(180);
      END_STATE();
    case 53:
      if (lookahead == 'e') ADVANCE(230);
      END_STATE();
    case 54:
      if (lookahead == 'e') ADVANCE(168);
      END_STATE();
    case 55:
      if (lookahead == 'e') ADVANCE(119);
      END_STATE();
    case 56:
      if (lookahead == 'e') ADVANCE(20);
      END_STATE();
    case 57:
      if (lookahead == 'e') ADVANCE(115);
      END_STATE();
    case 58:
      if (lookahead == 'e') ADVANCE(21);
      END_STATE();
    case 59:
      if (lookahead == 'e') ADVANCE(114);
      END_STATE();
    case 60:
      if (lookahead == 'e') ADVANCE(93);
      END_STATE();
    case 61:
      if (lookahead == 'e') ADVANCE(98);
      END_STATE();
    case 62:
      if (lookahead == 'f') ADVANCE(74);
      END_STATE();
    case 63:
      if (lookahead == 'g') ADVANCE(202);
      END_STATE();
    case 64:
      if (lookahead == 'g') ADVANCE(204);
      END_STATE();
    case 65:
      if (lookahead == 'g') ADVANCE(228);
      END_STATE();
    case 66:
      if (lookahead == 'h') ADVANCE(104);
      if (lookahead == 't') ADVANCE(113);
      END_STATE();
    case 67:
      if (lookahead == 'h') ADVANCE(110);
      END_STATE();
    case 68:
      if (lookahead == 'i') ADVANCE(145);
      END_STATE();
    case 69:
      if (lookahead == 'i') ADVANCE(62);
      END_STATE();
    case 70:
      if (lookahead == 'i') ADVANCE(30);
      END_STATE();
    case 71:
      if (lookahead == 'i') ADVANCE(94);
      if (lookahead == 'u') ADVANCE(35);
      END_STATE();
    case 72:
      if (lookahead == 'i') ADVANCE(95);
      END_STATE();
    case 73:
      if (lookahead == 'i') ADVANCE(132);
      END_STATE();
    case 74:
      if (lookahead == 'i') ADVANCE(59);
      END_STATE();
    case 75:
      if (lookahead == 'i') ADVANCE(37);
      END_STATE();
    case 76:
      if (lookahead == 'i') ADVANCE(108);
      END_STATE();
    case 77:
      if (lookahead == 'i') ADVANCE(109);
      END_STATE();
    case 78:
      if (lookahead == 'l') ADVANCE(120);
      END_STATE();
    case 79:
      if (lookahead == 'l') ADVANCE(186);
      END_STATE();
    case 80:
      if (lookahead == 'l') ADVANCE(56);
      END_STATE();
    case 81:
      if (lookahead == 'l') ADVANCE(139);
      END_STATE();
    case 82:
      if (lookahead == 'l') ADVANCE(45);
      END_STATE();
    case 83:
      if (lookahead == 'l') ADVANCE(47);
      END_STATE();
    case 84:
      if (lookahead == 'l') ADVANCE(142);
      END_STATE();
    case 85:
      if (lookahead == 'm') ADVANCE(176);
      END_STATE();
    case 86:
      if (lookahead == 'm') ADVANCE(55);
      if (lookahead == 'n') ADVANCE(314);
      if (lookahead == 't') ADVANCE(68);
      END_STATE();
    case 87:
      if (lookahead == 'n') ADVANCE(34);
      END_STATE();
    case 88:
      if (lookahead == 'n') ADVANCE(63);
      END_STATE();
    case 89:
      if (lookahead == 'n') ADVANCE(179);
      END_STATE();
    case 90:
      if (lookahead == 'n') ADVANCE(181);
      END_STATE();
    case 91:
      if (lookahead == 'n') ADVANCE(138);
      END_STATE();
    case 92:
      if (lookahead == 'n') ADVANCE(64);
      END_STATE();
    case 93:
      if (lookahead == 'n') ADVANCE(122);
      END_STATE();
    case 94:
      if (lookahead == 'n') ADVANCE(65);
      END_STATE();
    case 95:
      if (lookahead == 'n') ADVANCE(38);
      END_STATE();
    case 96:
      if (lookahead == 'n') ADVANCE(73);
      END_STATE();
    case 97:
      if (lookahead == 'n') ADVANCE(124);
      END_STATE();
    case 98:
      if (lookahead == 'n') ADVANCE(134);
      END_STATE();
    case 99:
      if (lookahead == 'o') ADVANCE(143);
      END_STATE();
    case 100:
      if (lookahead == 'o') ADVANCE(28);
      END_STATE();
    case 101:
      if (lookahead == 'o') ADVANCE(103);
      if (lookahead == 'y') ADVANCE(123);
      END_STATE();
    case 102:
      if (lookahead == 'o') ADVANCE(88);
      END_STATE();
    case 103:
      if (lookahead == 'o') ADVANCE(79);
      END_STATE();
    case 104:
      if (lookahead == 'o') ADVANCE(117);
      END_STATE();
    case 105:
      if (lookahead == 'o') ADVANCE(133);
      END_STATE();
    case 106:
      if (lookahead == 'o') ADVANCE(105);
      if (lookahead == 'p') ADVANCE(33);
      END_STATE();
    case 107:
      if (lookahead == 'o') ADVANCE(92);
      END_STATE();
    case 108:
      if (lookahead == 'o') ADVANCE(89);
      END_STATE();
    case 109:
      if (lookahead == 'o') ADVANCE(90);
      END_STATE();
    case 110:
      if (lookahead == 'o') ADVANCE(118);
      END_STATE();
    case 111:
      if (lookahead == 'p') ADVANCE(27);
      END_STATE();
    case 112:
      if (lookahead == 'p') ADVANCE(52);
      END_STATE();
    case 113:
      if (lookahead == 'r') ADVANCE(71);
      END_STATE();
    case 114:
      if (lookahead == 'r') ADVANCE(182);
      END_STATE();
    case 115:
      if (lookahead == 'r') ADVANCE(144);
      END_STATE();
    case 116:
      if (lookahead == 'r') ADVANCE(70);
      END_STATE();
    case 117:
      if (lookahead == 'r') ADVANCE(126);
      END_STATE();
    case 118:
      if (lookahead == 'r') ADVANCE(128);
      END_STATE();
    case 119:
      if (lookahead == 's') ADVANCE(111);
      END_STATE();
    case 120:
      if (lookahead == 's') ADVANCE(44);
      END_STATE();
    case 121:
      if (lookahead == 's') ADVANCE(57);
      END_STATE();
    case 122:
      if (lookahead == 's') ADVANCE(77);
      END_STATE();
    case 123:
      if (lookahead == 't') ADVANCE(42);
      END_STATE();
    case 124:
      if (lookahead == 't') ADVANCE(198);
      END_STATE();
    case 125:
      if (lookahead == 't') ADVANCE(200);
      END_STATE();
    case 126:
      if (lookahead == 't') ADVANCE(192);
      END_STATE();
    case 127:
      if (lookahead == 't') ADVANCE(173);
      END_STATE();
    case 128:
      if (lookahead == 't') ADVANCE(194);
      END_STATE();
    case 129:
      if (lookahead == 't') ADVANCE(130);
      END_STATE();
    case 130:
      if (lookahead == 't') ADVANCE(116);
      END_STATE();
    case 131:
      if (lookahead == 't') ADVANCE(148);
      END_STATE();
    case 132:
      if (lookahead == 't') ADVANCE(147);
      END_STATE();
    case 133:
      if (lookahead == 't') ADVANCE(22);
      END_STATE();
    case 134:
      if (lookahead == 't') ADVANCE(69);
      END_STATE();
    case 135:
      if (lookahead == 't') ADVANCE(46);
      END_STATE();
    case 136:
      if (lookahead == 't') ADVANCE(50);
      END_STATE();
    case 137:
      if (lookahead == 't') ADVANCE(60);
      END_STATE();
    case 138:
      if (lookahead == 'u') ADVANCE(85);
      END_STATE();
    case 139:
      if (lookahead == 'u') ADVANCE(39);
      END_STATE();
    case 140:
      if (lookahead == 'u') ADVANCE(43);
      END_STATE();
    case 141:
      if (lookahead == 'u') ADVANCE(136);
      END_STATE();
    case 142:
      if (lookahead == 'u') ADVANCE(41);
      END_STATE();
    case 143:
      if (lookahead == 'u') ADVANCE(32);
      END_STATE();
    case 144:
      if (lookahead == 'v') ADVANCE(75);
      END_STATE();
    case 145:
      if (lookahead == 'v') ADVANCE(58);
      END_STATE();
    case 146:
      if (lookahead == 'x') ADVANCE(137);
      END_STATE();
    case 147:
      if (lookahead == 'y') ADVANCE(310);
      END_STATE();
    case 148:
      if (lookahead == 'y') ADVANCE(112);
      END_STATE();
    case 149:
      if (lookahead == 'y') ADVANCE(135);
      END_STATE();
    case 150:
      if (lookahead == '+' ||
          lookahead == '-') ADVANCE(152);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(309);
      END_STATE();
    case 151:
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(308);
      END_STATE();
    case 152:
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(309);
      END_STATE();
    case 153:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(323);
      END_STATE();
    case 154:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(307);
      END_STATE();
    case 155:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(153);
      END_STATE();
    case 156:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(155);
      END_STATE();
    case 157:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(156);
      END_STATE();
    case 158:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(157);
      END_STATE();
    case 159:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(158);
      END_STATE();
    case 160:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(159);
      END_STATE();
    case 161:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(160);
      END_STATE();
    case 162:
      if (eof) ADVANCE(164);
      ADVANCE_MAP(
        '"', 171,
        '\'', 3,
        '(', 231,
        ')', 232,
        '+', 299,
        ',', 178,
        '-', 300,
        '.', 298,
        '/', 5,
        '0', 306,
        ':', 177,
        ';', 167,
        '=', 185,
        '[', 183,
        ']', 184,
        'a', 129,
        'b', 101,
        'd', 99,
        'e', 91,
        'f', 24,
        'i', 87,
        'l', 102,
        'n', 25,
        'r', 106,
        's', 66,
        't', 26,
        'u', 29,
        '{', 174,
        '}', 175,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(162);
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(305);
      END_STATE();
    case 163:
      if (eof) ADVANCE(164);
      ADVANCE_MAP(
        '"', 171,
        '\'', 3,
        '(', 231,
        ')', 232,
        '+', 299,
        ',', 178,
        '-', 300,
        '.', 151,
        '/', 6,
        '0', 306,
        ':', 177,
        ';', 167,
        '=', 185,
        '[', 183,
        ']', 184,
        'f', 241,
        'i', 263,
        'n', 242,
        't', 279,
        '{', 174,
        '}', 175,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(163);
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(305);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 164:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 165:
      ACCEPT_TOKEN(sym_documentation);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(165);
      END_STATE();
    case 166:
      ACCEPT_TOKEN(anon_sym_include);
      END_STATE();
    case 167:
      ACCEPT_TOKEN(anon_sym_SEMI);
      END_STATE();
    case 168:
      ACCEPT_TOKEN(anon_sym_native_include);
      END_STATE();
    case 169:
      ACCEPT_TOKEN(anon_sym_namespace);
      END_STATE();
    case 170:
      ACCEPT_TOKEN(anon_sym_attribute);
      END_STATE();
    case 171:
      ACCEPT_TOKEN(anon_sym_DQUOTE);
      END_STATE();
    case 172:
      ACCEPT_TOKEN(anon_sym_table);
      END_STATE();
    case 173:
      ACCEPT_TOKEN(anon_sym_struct);
      END_STATE();
    case 174:
      ACCEPT_TOKEN(anon_sym_LBRACE);
      END_STATE();
    case 175:
      ACCEPT_TOKEN(anon_sym_RBRACE);
      END_STATE();
    case 176:
      ACCEPT_TOKEN(anon_sym_enum);
      END_STATE();
    case 177:
      ACCEPT_TOKEN(anon_sym_COLON);
      END_STATE();
    case 178:
      ACCEPT_TOKEN(anon_sym_COMMA);
      END_STATE();
    case 179:
      ACCEPT_TOKEN(anon_sym_union);
      END_STATE();
    case 180:
      ACCEPT_TOKEN(anon_sym_root_type);
      END_STATE();
    case 181:
      ACCEPT_TOKEN(anon_sym_file_extension);
      END_STATE();
    case 182:
      ACCEPT_TOKEN(anon_sym_file_identifier);
      END_STATE();
    case 183:
      ACCEPT_TOKEN(anon_sym_LBRACK);
      END_STATE();
    case 184:
      ACCEPT_TOKEN(anon_sym_RBRACK);
      END_STATE();
    case 185:
      ACCEPT_TOKEN(anon_sym_EQ);
      END_STATE();
    case 186:
      ACCEPT_TOKEN(anon_sym_bool);
      END_STATE();
    case 187:
      ACCEPT_TOKEN(anon_sym_bool);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 188:
      ACCEPT_TOKEN(anon_sym_byte);
      END_STATE();
    case 189:
      ACCEPT_TOKEN(anon_sym_byte);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 190:
      ACCEPT_TOKEN(anon_sym_ubyte);
      END_STATE();
    case 191:
      ACCEPT_TOKEN(anon_sym_ubyte);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 192:
      ACCEPT_TOKEN(anon_sym_short);
      END_STATE();
    case 193:
      ACCEPT_TOKEN(anon_sym_short);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 194:
      ACCEPT_TOKEN(anon_sym_ushort);
      END_STATE();
    case 195:
      ACCEPT_TOKEN(anon_sym_ushort);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 196:
      ACCEPT_TOKEN(anon_sym_int);
      if (lookahead == '1') ADVANCE(17);
      if (lookahead == '3') ADVANCE(11);
      if (lookahead == '6') ADVANCE(14);
      if (lookahead == '8') ADVANCE(208);
      END_STATE();
    case 197:
      ACCEPT_TOKEN(anon_sym_int);
      if (lookahead == '1') ADVANCE(239);
      if (lookahead == '3') ADVANCE(233);
      if (lookahead == '6') ADVANCE(236);
      if (lookahead == '8') ADVANCE(209);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 198:
      ACCEPT_TOKEN(anon_sym_uint);
      if (lookahead == '1') ADVANCE(18);
      if (lookahead == '3') ADVANCE(12);
      if (lookahead == '6') ADVANCE(15);
      if (lookahead == '8') ADVANCE(210);
      END_STATE();
    case 199:
      ACCEPT_TOKEN(anon_sym_uint);
      if (lookahead == '1') ADVANCE(240);
      if (lookahead == '3') ADVANCE(234);
      if (lookahead == '6') ADVANCE(237);
      if (lookahead == '8') ADVANCE(211);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 200:
      ACCEPT_TOKEN(anon_sym_float);
      if (lookahead == '3') ADVANCE(13);
      if (lookahead == '6') ADVANCE(16);
      END_STATE();
    case 201:
      ACCEPT_TOKEN(anon_sym_float);
      if (lookahead == '3') ADVANCE(235);
      if (lookahead == '6') ADVANCE(238);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 202:
      ACCEPT_TOKEN(anon_sym_long);
      END_STATE();
    case 203:
      ACCEPT_TOKEN(anon_sym_long);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 204:
      ACCEPT_TOKEN(anon_sym_ulong);
      END_STATE();
    case 205:
      ACCEPT_TOKEN(anon_sym_ulong);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 206:
      ACCEPT_TOKEN(anon_sym_double);
      END_STATE();
    case 207:
      ACCEPT_TOKEN(anon_sym_double);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 208:
      ACCEPT_TOKEN(anon_sym_int8);
      END_STATE();
    case 209:
      ACCEPT_TOKEN(anon_sym_int8);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 210:
      ACCEPT_TOKEN(anon_sym_uint8);
      END_STATE();
    case 211:
      ACCEPT_TOKEN(anon_sym_uint8);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 212:
      ACCEPT_TOKEN(anon_sym_int16);
      END_STATE();
    case 213:
      ACCEPT_TOKEN(anon_sym_int16);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 214:
      ACCEPT_TOKEN(anon_sym_uint16);
      END_STATE();
    case 215:
      ACCEPT_TOKEN(anon_sym_uint16);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 216:
      ACCEPT_TOKEN(anon_sym_int32);
      END_STATE();
    case 217:
      ACCEPT_TOKEN(anon_sym_int32);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 218:
      ACCEPT_TOKEN(anon_sym_uint32);
      END_STATE();
    case 219:
      ACCEPT_TOKEN(anon_sym_uint32);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 220:
      ACCEPT_TOKEN(anon_sym_int64);
      END_STATE();
    case 221:
      ACCEPT_TOKEN(anon_sym_int64);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 222:
      ACCEPT_TOKEN(anon_sym_uint64);
      END_STATE();
    case 223:
      ACCEPT_TOKEN(anon_sym_uint64);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 224:
      ACCEPT_TOKEN(anon_sym_float32);
      END_STATE();
    case 225:
      ACCEPT_TOKEN(anon_sym_float32);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 226:
      ACCEPT_TOKEN(anon_sym_float64);
      END_STATE();
    case 227:
      ACCEPT_TOKEN(anon_sym_float64);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 228:
      ACCEPT_TOKEN(anon_sym_string);
      END_STATE();
    case 229:
      ACCEPT_TOKEN(anon_sym_string);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 230:
      ACCEPT_TOKEN(anon_sym_rpc_service);
      END_STATE();
    case 231:
      ACCEPT_TOKEN(anon_sym_LPAREN);
      END_STATE();
    case 232:
      ACCEPT_TOKEN(anon_sym_RPAREN);
      END_STATE();
    case 233:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == '2') ADVANCE(217);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 234:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == '2') ADVANCE(219);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 235:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == '2') ADVANCE(225);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 236:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == '4') ADVANCE(221);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 237:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == '4') ADVANCE(223);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 238:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == '4') ADVANCE(227);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 239:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == '6') ADVANCE(213);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 240:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == '6') ADVANCE(215);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 241:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(259);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 242:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(264);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 243:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(287);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 244:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'b') ADVANCE(295);
      if (lookahead == 'i') ADVANCE(270);
      if (lookahead == 'l') ADVANCE(277);
      if (lookahead == 's') ADVANCE(256);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 245:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'b') ADVANCE(262);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 246:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(302);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 247:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(304);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 248:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(189);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 249:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(191);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 250:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(207);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 251:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'f') ADVANCE(312);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 252:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'g') ADVANCE(203);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 253:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'g') ADVANCE(205);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 254:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'g') ADVANCE(229);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 255:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'h') ADVANCE(273);
      if (lookahead == 't') ADVANCE(280);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 256:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'h') ADVANCE(278);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 257:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(284);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 258:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(269);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 259:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(283);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 260:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(187);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 261:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(272);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 262:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(250);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 263:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(251);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 264:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(315);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 265:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(257);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 266:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(252);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 267:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(253);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 268:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(285);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 269:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(254);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 270:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(286);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 271:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(293);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 272:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(243);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 273:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(281);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 274:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(260);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 275:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(274);
      if (lookahead == 'y') ADVANCE(290);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 276:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(266);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 277:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(267);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 278:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(282);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 279:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(292);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 280:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(258);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 281:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(288);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 282:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(289);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 283:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 's') ADVANCE(247);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 284:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(294);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 285:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(197);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 286:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(199);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 287:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(201);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 288:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(193);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 289:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(195);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 290:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(248);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 291:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(249);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 292:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'u') ADVANCE(246);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 293:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'u') ADVANCE(245);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 294:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'y') ADVANCE(311);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 295:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'y') ADVANCE(291);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 296:
      ACCEPT_TOKEN(sym_identifier);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 297:
      ACCEPT_TOKEN(anon_sym_DOT);
      END_STATE();
    case 298:
      ACCEPT_TOKEN(anon_sym_DOT);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(308);
      END_STATE();
    case 299:
      ACCEPT_TOKEN(sym_plus_token);
      END_STATE();
    case 300:
      ACCEPT_TOKEN(sym_minus_token);
      END_STATE();
    case 301:
      ACCEPT_TOKEN(sym_true);
      END_STATE();
    case 302:
      ACCEPT_TOKEN(sym_true);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 303:
      ACCEPT_TOKEN(sym_false);
      END_STATE();
    case 304:
      ACCEPT_TOKEN(sym_false);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 305:
      ACCEPT_TOKEN(aux_sym_decimal_lit_token1);
      if (lookahead == '.') ADVANCE(308);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(150);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(305);
      END_STATE();
    case 306:
      ACCEPT_TOKEN(anon_sym_0);
      if (lookahead == '.') ADVANCE(308);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(150);
      if (lookahead == 'X' ||
          lookahead == 'x') ADVANCE(154);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(9);
      END_STATE();
    case 307:
      ACCEPT_TOKEN(sym_hex_lit);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(307);
      END_STATE();
    case 308:
      ACCEPT_TOKEN(aux_sym_float_lit_token1);
      if (lookahead == 'E' ||
          lookahead == 'e') ADVANCE(150);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(308);
      END_STATE();
    case 309:
      ACCEPT_TOKEN(aux_sym_float_lit_token1);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(309);
      END_STATE();
    case 310:
      ACCEPT_TOKEN(anon_sym_infinity);
      END_STATE();
    case 311:
      ACCEPT_TOKEN(anon_sym_infinity);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 312:
      ACCEPT_TOKEN(anon_sym_inf);
      if (lookahead == 'i') ADVANCE(265);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 313:
      ACCEPT_TOKEN(anon_sym_inf);
      if (lookahead == 'i') ADVANCE(96);
      END_STATE();
    case 314:
      ACCEPT_TOKEN(sym_nan_token);
      END_STATE();
    case 315:
      ACCEPT_TOKEN(sym_nan_token);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(296);
      END_STATE();
    case 316:
      ACCEPT_TOKEN(aux_sym_string_constant_token1);
      if (lookahead == '\n') ADVANCE(321);
      if (lookahead == '"' ||
          lookahead == '\\') ADVANCE(328);
      if (lookahead != 0) ADVANCE(316);
      END_STATE();
    case 317:
      ACCEPT_TOKEN(aux_sym_string_constant_token1);
      if (lookahead == '*') ADVANCE(319);
      if (lookahead == '/') ADVANCE(316);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(321);
      END_STATE();
    case 318:
      ACCEPT_TOKEN(aux_sym_string_constant_token1);
      if (lookahead == '*') ADVANCE(318);
      if (lookahead == '/') ADVANCE(321);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(319);
      END_STATE();
    case 319:
      ACCEPT_TOKEN(aux_sym_string_constant_token1);
      if (lookahead == '*') ADVANCE(318);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(319);
      END_STATE();
    case 320:
      ACCEPT_TOKEN(aux_sym_string_constant_token1);
      if (lookahead == '/') ADVANCE(317);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(320);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(321);
      END_STATE();
    case 321:
      ACCEPT_TOKEN(aux_sym_string_constant_token1);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(321);
      END_STATE();
    case 322:
      ACCEPT_TOKEN(aux_sym_string_constant_token2);
      END_STATE();
    case 323:
      ACCEPT_TOKEN(sym_escape_sequence);
      END_STATE();
    case 324:
      ACCEPT_TOKEN(sym_escape_sequence);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(323);
      END_STATE();
    case 325:
      ACCEPT_TOKEN(sym_escape_sequence);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(324);
      END_STATE();
    case 326:
      ACCEPT_TOKEN(sym_comment);
      END_STATE();
    case 327:
      ACCEPT_TOKEN(sym_comment);
      if (lookahead == '/') ADVANCE(165);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(328);
      END_STATE();
    case 328:
      ACCEPT_TOKEN(sym_comment);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(328);
      END_STATE();
    default:
      return false;
  }
}

static const TSLexMode ts_lex_modes[STATE_COUNT] = {
  [0] = {.lex_state = 0},
  [1] = {.lex_state = 0},
  [2] = {.lex_state = 0},
  [3] = {.lex_state = 163},
  [4] = {.lex_state = 163},
  [5] = {.lex_state = 163},
  [6] = {.lex_state = 163},
  [7] = {.lex_state = 4},
  [8] = {.lex_state = 4},
  [9] = {.lex_state = 163},
  [10] = {.lex_state = 4},
  [11] = {.lex_state = 4},
  [12] = {.lex_state = 4},
  [13] = {.lex_state = 4},
  [14] = {.lex_state = 4},
  [15] = {.lex_state = 0},
  [16] = {.lex_state = 0},
  [17] = {.lex_state = 0},
  [18] = {.lex_state = 0},
  [19] = {.lex_state = 0},
  [20] = {.lex_state = 0},
  [21] = {.lex_state = 0},
  [22] = {.lex_state = 0},
  [23] = {.lex_state = 0},
  [24] = {.lex_state = 0},
  [25] = {.lex_state = 0},
  [26] = {.lex_state = 0},
  [27] = {.lex_state = 0},
  [28] = {.lex_state = 0},
  [29] = {.lex_state = 0},
  [30] = {.lex_state = 0},
  [31] = {.lex_state = 0},
  [32] = {.lex_state = 0},
  [33] = {.lex_state = 0},
  [34] = {.lex_state = 0},
  [35] = {.lex_state = 0},
  [36] = {.lex_state = 0},
  [37] = {.lex_state = 0},
  [38] = {.lex_state = 0},
  [39] = {.lex_state = 0},
  [40] = {.lex_state = 0},
  [41] = {.lex_state = 0},
  [42] = {.lex_state = 0},
  [43] = {.lex_state = 0},
  [44] = {.lex_state = 0},
  [45] = {.lex_state = 0},
  [46] = {.lex_state = 0},
  [47] = {.lex_state = 0},
  [48] = {.lex_state = 0},
  [49] = {.lex_state = 0},
  [50] = {.lex_state = 0},
  [51] = {.lex_state = 0},
  [52] = {.lex_state = 0},
  [53] = {.lex_state = 0},
  [54] = {.lex_state = 0},
  [55] = {.lex_state = 0},
  [56] = {.lex_state = 0},
  [57] = {.lex_state = 0},
  [58] = {.lex_state = 0},
  [59] = {.lex_state = 0},
  [60] = {.lex_state = 0},
  [61] = {.lex_state = 0},
  [62] = {.lex_state = 0},
  [63] = {.lex_state = 0},
  [64] = {.lex_state = 0},
  [65] = {.lex_state = 0},
  [66] = {.lex_state = 0},
  [67] = {.lex_state = 0},
  [68] = {.lex_state = 0},
  [69] = {.lex_state = 0},
  [70] = {.lex_state = 0},
  [71] = {.lex_state = 0},
  [72] = {.lex_state = 0},
  [73] = {.lex_state = 0},
  [74] = {.lex_state = 0},
  [75] = {.lex_state = 0},
  [76] = {.lex_state = 0},
  [77] = {.lex_state = 0},
  [78] = {.lex_state = 0},
  [79] = {.lex_state = 0},
  [80] = {.lex_state = 0},
  [81] = {.lex_state = 0},
  [82] = {.lex_state = 0},
  [83] = {.lex_state = 163},
  [84] = {.lex_state = 4},
  [85] = {.lex_state = 4},
  [86] = {.lex_state = 4},
  [87] = {.lex_state = 4},
  [88] = {.lex_state = 0},
  [89] = {.lex_state = 163},
  [90] = {.lex_state = 163},
  [91] = {.lex_state = 0},
  [92] = {.lex_state = 163},
  [93] = {.lex_state = 163},
  [94] = {.lex_state = 163},
  [95] = {.lex_state = 163},
  [96] = {.lex_state = 0},
  [97] = {.lex_state = 163},
  [98] = {.lex_state = 10},
  [99] = {.lex_state = 10},
  [100] = {.lex_state = 10},
  [101] = {.lex_state = 10},
  [102] = {.lex_state = 10},
  [103] = {.lex_state = 10},
  [104] = {.lex_state = 10},
  [105] = {.lex_state = 10},
  [106] = {.lex_state = 163},
  [107] = {.lex_state = 10},
  [108] = {.lex_state = 10},
  [109] = {.lex_state = 10},
  [110] = {.lex_state = 10},
  [111] = {.lex_state = 10},
  [112] = {.lex_state = 10},
  [113] = {.lex_state = 10},
  [114] = {.lex_state = 10},
  [115] = {.lex_state = 10},
  [116] = {.lex_state = 10},
  [117] = {.lex_state = 10},
  [118] = {.lex_state = 10},
  [119] = {.lex_state = 10},
  [120] = {.lex_state = 10},
  [121] = {.lex_state = 10},
  [122] = {.lex_state = 10},
  [123] = {.lex_state = 10},
  [124] = {.lex_state = 10},
  [125] = {.lex_state = 10},
  [126] = {.lex_state = 10},
  [127] = {.lex_state = 10},
  [128] = {.lex_state = 10},
  [129] = {.lex_state = 163},
  [130] = {.lex_state = 163},
  [131] = {.lex_state = 163},
  [132] = {.lex_state = 0},
  [133] = {.lex_state = 10},
  [134] = {.lex_state = 163},
  [135] = {.lex_state = 163},
  [136] = {.lex_state = 10},
  [137] = {.lex_state = 163},
  [138] = {.lex_state = 163},
  [139] = {.lex_state = 10},
  [140] = {.lex_state = 10},
  [141] = {.lex_state = 163},
  [142] = {.lex_state = 163},
  [143] = {.lex_state = 163},
  [144] = {.lex_state = 163},
  [145] = {.lex_state = 163},
  [146] = {.lex_state = 0},
  [147] = {.lex_state = 10},
  [148] = {.lex_state = 163},
  [149] = {.lex_state = 163},
  [150] = {.lex_state = 10},
  [151] = {.lex_state = 163},
  [152] = {.lex_state = 10},
  [153] = {.lex_state = 10},
  [154] = {.lex_state = 163},
  [155] = {.lex_state = 163},
  [156] = {.lex_state = 163},
  [157] = {.lex_state = 163},
  [158] = {.lex_state = 163},
  [159] = {.lex_state = 10},
  [160] = {.lex_state = 163},
  [161] = {.lex_state = 0},
  [162] = {.lex_state = 0},
  [163] = {.lex_state = 163},
  [164] = {.lex_state = 163},
  [165] = {.lex_state = 163},
  [166] = {.lex_state = 10},
  [167] = {.lex_state = 10},
  [168] = {.lex_state = 10},
  [169] = {.lex_state = 1},
  [170] = {.lex_state = 163},
  [171] = {.lex_state = 1},
  [172] = {.lex_state = 163},
  [173] = {.lex_state = 163},
  [174] = {.lex_state = 10},
  [175] = {.lex_state = 10},
  [176] = {.lex_state = 0},
  [177] = {.lex_state = 163},
  [178] = {.lex_state = 163},
  [179] = {.lex_state = 163},
  [180] = {.lex_state = 0},
  [181] = {.lex_state = 10},
  [182] = {.lex_state = 163},
  [183] = {.lex_state = 163},
  [184] = {.lex_state = 1},
  [185] = {.lex_state = 163},
  [186] = {.lex_state = 163},
  [187] = {.lex_state = 163},
  [188] = {.lex_state = 163},
  [189] = {.lex_state = 163},
  [190] = {.lex_state = 163},
  [191] = {.lex_state = 163},
  [192] = {.lex_state = 163},
  [193] = {.lex_state = 163},
  [194] = {.lex_state = 10},
  [195] = {.lex_state = 163},
  [196] = {.lex_state = 163},
  [197] = {.lex_state = 163},
  [198] = {.lex_state = 163},
  [199] = {.lex_state = 10},
  [200] = {.lex_state = 163},
  [201] = {.lex_state = 163},
  [202] = {.lex_state = 163},
  [203] = {.lex_state = 163},
  [204] = {.lex_state = 163},
  [205] = {.lex_state = 163},
  [206] = {.lex_state = 163},
  [207] = {.lex_state = 163},
  [208] = {.lex_state = 163},
  [209] = {.lex_state = 163},
  [210] = {.lex_state = 10},
  [211] = {.lex_state = 10},
  [212] = {.lex_state = 163},
  [213] = {.lex_state = 163},
  [214] = {.lex_state = 163},
  [215] = {.lex_state = 163},
  [216] = {.lex_state = 163},
  [217] = {.lex_state = 163},
  [218] = {.lex_state = 163},
  [219] = {.lex_state = 163},
  [220] = {.lex_state = 163},
  [221] = {.lex_state = 163},
  [222] = {.lex_state = 163},
  [223] = {.lex_state = 10},
  [224] = {.lex_state = 163},
  [225] = {.lex_state = 10},
  [226] = {.lex_state = 163},
  [227] = {.lex_state = 163},
  [228] = {.lex_state = 163},
  [229] = {.lex_state = 163},
  [230] = {.lex_state = 163},
  [231] = {.lex_state = 163},
  [232] = {.lex_state = 163},
  [233] = {.lex_state = 163},
  [234] = {.lex_state = 163},
  [235] = {.lex_state = 10},
  [236] = {.lex_state = 10},
  [237] = {.lex_state = 10},
  [238] = {.lex_state = 163},
  [239] = {.lex_state = 163},
  [240] = {.lex_state = 10},
  [241] = {.lex_state = 163},
  [242] = {.lex_state = 10},
  [243] = {.lex_state = 10},
  [244] = {.lex_state = 10},
  [245] = {.lex_state = 10},
  [246] = {.lex_state = 10},
  [247] = {.lex_state = 10},
  [248] = {.lex_state = 10},
  [249] = {.lex_state = 163},
  [250] = {.lex_state = 163},
  [251] = {.lex_state = 163},
  [252] = {.lex_state = 163},
  [253] = {.lex_state = 163},
  [254] = {.lex_state = 163},
  [255] = {.lex_state = 163},
  [256] = {.lex_state = 163},
  [257] = {.lex_state = 163},
  [258] = {.lex_state = 163},
  [259] = {.lex_state = 2},
  [260] = {.lex_state = 2},
  [261] = {.lex_state = 163},
  [262] = {.lex_state = 163},
  [263] = {.lex_state = 163},
  [264] = {.lex_state = 2},
  [265] = {.lex_state = 163},
  [266] = {.lex_state = 163},
  [267] = {.lex_state = 163},
  [268] = {.lex_state = 10},
  [269] = {.lex_state = 2},
  [270] = {.lex_state = 163},
  [271] = {.lex_state = 2},
  [272] = {.lex_state = 2},
  [273] = {.lex_state = 163},
  [274] = {.lex_state = 2},
  [275] = {.lex_state = 163},
  [276] = {.lex_state = 163},
  [277] = {.lex_state = 163},
  [278] = {.lex_state = 163},
  [279] = {.lex_state = 163},
  [280] = {.lex_state = 2},
  [281] = {.lex_state = 2},
  [282] = {.lex_state = 163},
  [283] = {.lex_state = 163},
  [284] = {.lex_state = 2},
  [285] = {.lex_state = 163},
  [286] = {.lex_state = 2},
  [287] = {.lex_state = 2},
  [288] = {.lex_state = 163},
  [289] = {.lex_state = 163},
  [290] = {.lex_state = 163},
  [291] = {.lex_state = 163},
  [292] = {.lex_state = 2},
  [293] = {.lex_state = 163},
  [294] = {.lex_state = 163},
  [295] = {.lex_state = 163},
  [296] = {.lex_state = 163},
  [297] = {.lex_state = 163},
  [298] = {.lex_state = 163},
  [299] = {.lex_state = 163},
  [300] = {.lex_state = 2},
  [301] = {.lex_state = 163},
  [302] = {.lex_state = 2},
  [303] = {.lex_state = 2},
  [304] = {.lex_state = 2},
  [305] = {.lex_state = 163},
  [306] = {.lex_state = 163},
  [307] = {.lex_state = 163},
  [308] = {.lex_state = 163},
  [309] = {.lex_state = 163},
  [310] = {.lex_state = 163},
  [311] = {.lex_state = 163},
  [312] = {.lex_state = 163},
  [313] = {.lex_state = 2},
  [314] = {.lex_state = 2},
  [315] = {.lex_state = 163},
  [316] = {.lex_state = 163},
  [317] = {.lex_state = 2},
  [318] = {.lex_state = 163},
  [319] = {.lex_state = 163},
  [320] = {.lex_state = 163},
  [321] = {.lex_state = 163},
  [322] = {.lex_state = 163},
};

static const uint16_t ts_parse_table[LARGE_STATE_COUNT][SYMBOL_COUNT] = {
  [0] = {
    [ts_builtin_sym_end] = ACTIONS(1),
    [sym_documentation] = ACTIONS(1),
    [anon_sym_include] = ACTIONS(1),
    [anon_sym_SEMI] = ACTIONS(1),
    [anon_sym_native_include] = ACTIONS(1),
    [anon_sym_namespace] = ACTIONS(1),
    [anon_sym_attribute] = ACTIONS(1),
    [anon_sym_DQUOTE] = ACTIONS(1),
    [anon_sym_table] = ACTIONS(1),
    [anon_sym_struct] = ACTIONS(1),
    [anon_sym_LBRACE] = ACTIONS(1),
    [anon_sym_RBRACE] = ACTIONS(1),
    [anon_sym_enum] = ACTIONS(1),
    [anon_sym_COLON] = ACTIONS(1),
    [anon_sym_COMMA] = ACTIONS(1),
    [anon_sym_union] = ACTIONS(1),
    [anon_sym_root_type] = ACTIONS(1),
    [anon_sym_file_extension] = ACTIONS(1),
    [anon_sym_file_identifier] = ACTIONS(1),
    [anon_sym_LBRACK] = ACTIONS(1),
    [anon_sym_RBRACK] = ACTIONS(1),
    [anon_sym_EQ] = ACTIONS(1),
    [anon_sym_bool] = ACTIONS(1),
    [anon_sym_byte] = ACTIONS(1),
    [anon_sym_ubyte] = ACTIONS(1),
    [anon_sym_short] = ACTIONS(1),
    [anon_sym_ushort] = ACTIONS(1),
    [anon_sym_int] = ACTIONS(1),
    [anon_sym_uint] = ACTIONS(1),
    [anon_sym_float] = ACTIONS(1),
    [anon_sym_long] = ACTIONS(1),
    [anon_sym_ulong] = ACTIONS(1),
    [anon_sym_double] = ACTIONS(1),
    [anon_sym_int8] = ACTIONS(1),
    [anon_sym_uint8] = ACTIONS(1),
    [anon_sym_int16] = ACTIONS(1),
    [anon_sym_uint16] = ACTIONS(1),
    [anon_sym_int32] = ACTIONS(1),
    [anon_sym_uint32] = ACTIONS(1),
    [anon_sym_int64] = ACTIONS(1),
    [anon_sym_uint64] = ACTIONS(1),
    [anon_sym_float32] = ACTIONS(1),
    [anon_sym_float64] = ACTIONS(1),
    [anon_sym_string] = ACTIONS(1),
    [anon_sym_rpc_service] = ACTIONS(1),
    [anon_sym_LPAREN] = ACTIONS(1),
    [anon_sym_RPAREN] = ACTIONS(1),
    [anon_sym_DOT] = ACTIONS(1),
    [sym_plus_token] = ACTIONS(1),
    [sym_minus_token] = ACTIONS(1),
    [sym_true] = ACTIONS(1),
    [sym_false] = ACTIONS(1),
    [aux_sym_decimal_lit_token1] = ACTIONS(1),
    [anon_sym_0] = ACTIONS(1),
    [sym_hex_lit] = ACTIONS(1),
    [aux_sym_float_lit_token1] = ACTIONS(1),
    [anon_sym_infinity] = ACTIONS(1),
    [anon_sym_inf] = ACTIONS(1),
    [sym_nan_token] = ACTIONS(1),
    [aux_sym_string_constant_token2] = ACTIONS(1),
    [sym_escape_sequence] = ACTIONS(1),
    [sym_comment] = ACTIONS(3),
  },
  [1] = {
    [sym_source_file] = STATE(310),
    [sym_include] = STATE(2),
    [sym_native_include] = STATE(2),
    [sym_namespace_decl] = STATE(15),
    [sym_attribute_decl] = STATE(15),
    [sym_type_decl] = STATE(15),
    [sym_enum_decl] = STATE(15),
    [sym_union_decl] = STATE(15),
    [sym_object] = STATE(15),
    [sym_root_decl] = STATE(15),
    [sym_file_extension_decl] = STATE(15),
    [sym_file_identifier_decl] = STATE(15),
    [sym_rpc_decl] = STATE(15),
    [aux_sym_source_file_repeat1] = STATE(2),
    [aux_sym_source_file_repeat2] = STATE(15),
    [aux_sym_type_decl_repeat1] = STATE(96),
    [ts_builtin_sym_end] = ACTIONS(5),
    [sym_documentation] = ACTIONS(7),
    [anon_sym_include] = ACTIONS(9),
    [anon_sym_native_include] = ACTIONS(11),
    [anon_sym_namespace] = ACTIONS(13),
    [anon_sym_attribute] = ACTIONS(15),
    [anon_sym_table] = ACTIONS(17),
    [anon_sym_struct] = ACTIONS(17),
    [anon_sym_LBRACE] = ACTIONS(19),
    [anon_sym_enum] = ACTIONS(21),
    [anon_sym_union] = ACTIONS(23),
    [anon_sym_root_type] = ACTIONS(25),
    [anon_sym_file_extension] = ACTIONS(27),
    [anon_sym_file_identifier] = ACTIONS(29),
    [anon_sym_rpc_service] = ACTIONS(31),
    [sym_comment] = ACTIONS(3),
  },
};

static const uint16_t ts_small_parse_table[] = {
  [0] = 18,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_documentation,
    ACTIONS(9), 1,
      anon_sym_include,
    ACTIONS(11), 1,
      anon_sym_native_include,
    ACTIONS(13), 1,
      anon_sym_namespace,
    ACTIONS(15), 1,
      anon_sym_attribute,
    ACTIONS(19), 1,
      anon_sym_LBRACE,
    ACTIONS(21), 1,
      anon_sym_enum,
    ACTIONS(23), 1,
      anon_sym_union,
    ACTIONS(25), 1,
      anon_sym_root_type,
    ACTIONS(27), 1,
      anon_sym_file_extension,
    ACTIONS(29), 1,
      anon_sym_file_identifier,
    ACTIONS(31), 1,
      anon_sym_rpc_service,
    ACTIONS(33), 1,
      ts_builtin_sym_end,
    STATE(96), 1,
      aux_sym_type_decl_repeat1,
    ACTIONS(17), 2,
      anon_sym_table,
      anon_sym_struct,
    STATE(18), 3,
      sym_include,
      sym_native_include,
      aux_sym_source_file_repeat1,
    STATE(16), 11,
      sym_namespace_decl,
      sym_attribute_decl,
      sym_type_decl,
      sym_enum_decl,
      sym_union_decl,
      sym_object,
      sym_root_decl,
      sym_file_extension_decl,
      sym_file_identifier_decl,
      sym_rpc_decl,
      aux_sym_source_file_repeat2,
  [68] = 25,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(37), 1,
      anon_sym_LBRACE,
    ACTIONS(39), 1,
      anon_sym_LBRACK,
    ACTIONS(41), 1,
      sym_identifier,
    ACTIONS(49), 1,
      sym_hex_lit,
    ACTIONS(51), 1,
      aux_sym_float_lit_token1,
    ACTIONS(55), 1,
      sym_nan_token,
    ACTIONS(57), 1,
      aux_sym_string_constant_token2,
    ACTIONS(59), 1,
      sym_comment,
    STATE(83), 1,
      sym_plus_minus_constant,
    STATE(130), 1,
      sym_decimal_lit,
    STATE(131), 1,
      sym_int_lit,
    STATE(143), 1,
      sym_string_constant,
    STATE(154), 1,
      sym_inf_token,
    STATE(156), 1,
      sym_full_ident,
    STATE(157), 1,
      sym_scalar,
    STATE(160), 1,
      sym_float_lit,
    STATE(177), 1,
      sym_object,
    STATE(178), 1,
      sym_single_value,
    STATE(266), 1,
      sym_value,
    ACTIONS(43), 2,
      sym_plus_token,
      sym_minus_token,
    ACTIONS(45), 2,
      sym_true,
      sym_false,
    ACTIONS(47), 2,
      aux_sym_decimal_lit_token1,
      anon_sym_0,
    ACTIONS(53), 2,
      anon_sym_infinity,
      anon_sym_inf,
    STATE(158), 3,
      sym_bool_constant,
      sym_int_constant,
      sym_float_constant,
  [150] = 25,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(37), 1,
      anon_sym_LBRACE,
    ACTIONS(39), 1,
      anon_sym_LBRACK,
    ACTIONS(41), 1,
      sym_identifier,
    ACTIONS(49), 1,
      sym_hex_lit,
    ACTIONS(51), 1,
      aux_sym_float_lit_token1,
    ACTIONS(55), 1,
      sym_nan_token,
    ACTIONS(57), 1,
      aux_sym_string_constant_token2,
    ACTIONS(59), 1,
      sym_comment,
    STATE(83), 1,
      sym_plus_minus_constant,
    STATE(130), 1,
      sym_decimal_lit,
    STATE(131), 1,
      sym_int_lit,
    STATE(143), 1,
      sym_string_constant,
    STATE(154), 1,
      sym_inf_token,
    STATE(156), 1,
      sym_full_ident,
    STATE(157), 1,
      sym_scalar,
    STATE(160), 1,
      sym_float_lit,
    STATE(177), 1,
      sym_object,
    STATE(178), 1,
      sym_single_value,
    STATE(208), 1,
      sym_value,
    ACTIONS(43), 2,
      sym_plus_token,
      sym_minus_token,
    ACTIONS(45), 2,
      sym_true,
      sym_false,
    ACTIONS(47), 2,
      aux_sym_decimal_lit_token1,
      anon_sym_0,
    ACTIONS(53), 2,
      anon_sym_infinity,
      anon_sym_inf,
    STATE(158), 3,
      sym_bool_constant,
      sym_int_constant,
      sym_float_constant,
  [232] = 25,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(37), 1,
      anon_sym_LBRACE,
    ACTIONS(39), 1,
      anon_sym_LBRACK,
    ACTIONS(41), 1,
      sym_identifier,
    ACTIONS(49), 1,
      sym_hex_lit,
    ACTIONS(51), 1,
      aux_sym_float_lit_token1,
    ACTIONS(55), 1,
      sym_nan_token,
    ACTIONS(57), 1,
      aux_sym_string_constant_token2,
    ACTIONS(59), 1,
      sym_comment,
    STATE(83), 1,
      sym_plus_minus_constant,
    STATE(130), 1,
      sym_decimal_lit,
    STATE(131), 1,
      sym_int_lit,
    STATE(143), 1,
      sym_string_constant,
    STATE(154), 1,
      sym_inf_token,
    STATE(156), 1,
      sym_full_ident,
    STATE(157), 1,
      sym_scalar,
    STATE(160), 1,
      sym_float_lit,
    STATE(177), 1,
      sym_object,
    STATE(178), 1,
      sym_single_value,
    STATE(222), 1,
      sym_value,
    ACTIONS(43), 2,
      sym_plus_token,
      sym_minus_token,
    ACTIONS(45), 2,
      sym_true,
      sym_false,
    ACTIONS(47), 2,
      aux_sym_decimal_lit_token1,
      anon_sym_0,
    ACTIONS(53), 2,
      anon_sym_infinity,
      anon_sym_inf,
    STATE(158), 3,
      sym_bool_constant,
      sym_int_constant,
      sym_float_constant,
  [314] = 25,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(37), 1,
      anon_sym_LBRACE,
    ACTIONS(39), 1,
      anon_sym_LBRACK,
    ACTIONS(41), 1,
      sym_identifier,
    ACTIONS(49), 1,
      sym_hex_lit,
    ACTIONS(51), 1,
      aux_sym_float_lit_token1,
    ACTIONS(55), 1,
      sym_nan_token,
    ACTIONS(57), 1,
      aux_sym_string_constant_token2,
    ACTIONS(59), 1,
      sym_comment,
    STATE(83), 1,
      sym_plus_minus_constant,
    STATE(130), 1,
      sym_decimal_lit,
    STATE(131), 1,
      sym_int_lit,
    STATE(143), 1,
      sym_string_constant,
    STATE(154), 1,
      sym_inf_token,
    STATE(156), 1,
      sym_full_ident,
    STATE(157), 1,
      sym_scalar,
    STATE(160), 1,
      sym_float_lit,
    STATE(177), 1,
      sym_object,
    STATE(178), 1,
      sym_single_value,
    STATE(224), 1,
      sym_value,
    ACTIONS(43), 2,
      sym_plus_token,
      sym_minus_token,
    ACTIONS(45), 2,
      sym_true,
      sym_false,
    ACTIONS(47), 2,
      aux_sym_decimal_lit_token1,
      anon_sym_0,
    ACTIONS(53), 2,
      anon_sym_infinity,
      anon_sym_inf,
    STATE(158), 3,
      sym_bool_constant,
      sym_int_constant,
      sym_float_constant,
  [396] = 6,
    ACTIONS(41), 1,
      sym_identifier,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(61), 1,
      anon_sym_LBRACK,
    STATE(95), 1,
      sym_full_ident,
    STATE(203), 1,
      sym_type,
    ACTIONS(63), 22,
      anon_sym_bool,
      anon_sym_byte,
      anon_sym_ubyte,
      anon_sym_short,
      anon_sym_ushort,
      anon_sym_int,
      anon_sym_uint,
      anon_sym_float,
      anon_sym_long,
      anon_sym_ulong,
      anon_sym_double,
      anon_sym_int8,
      anon_sym_uint8,
      anon_sym_int16,
      anon_sym_uint16,
      anon_sym_int32,
      anon_sym_uint32,
      anon_sym_int64,
      anon_sym_uint64,
      anon_sym_float32,
      anon_sym_float64,
      anon_sym_string,
  [436] = 6,
    ACTIONS(41), 1,
      sym_identifier,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(61), 1,
      anon_sym_LBRACK,
    STATE(95), 1,
      sym_full_ident,
    STATE(255), 1,
      sym_type,
    ACTIONS(63), 22,
      anon_sym_bool,
      anon_sym_byte,
      anon_sym_ubyte,
      anon_sym_short,
      anon_sym_ushort,
      anon_sym_int,
      anon_sym_uint,
      anon_sym_float,
      anon_sym_long,
      anon_sym_ulong,
      anon_sym_double,
      anon_sym_int8,
      anon_sym_uint8,
      anon_sym_int16,
      anon_sym_uint16,
      anon_sym_int32,
      anon_sym_uint32,
      anon_sym_int64,
      anon_sym_uint64,
      anon_sym_float32,
      anon_sym_float64,
      anon_sym_string,
  [476] = 21,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(41), 1,
      sym_identifier,
    ACTIONS(49), 1,
      sym_hex_lit,
    ACTIONS(51), 1,
      aux_sym_float_lit_token1,
    ACTIONS(55), 1,
      sym_nan_token,
    ACTIONS(57), 1,
      aux_sym_string_constant_token2,
    ACTIONS(59), 1,
      sym_comment,
    STATE(83), 1,
      sym_plus_minus_constant,
    STATE(130), 1,
      sym_decimal_lit,
    STATE(131), 1,
      sym_int_lit,
    STATE(143), 1,
      sym_string_constant,
    STATE(154), 1,
      sym_inf_token,
    STATE(156), 1,
      sym_full_ident,
    STATE(157), 1,
      sym_scalar,
    STATE(160), 1,
      sym_float_lit,
    STATE(263), 1,
      sym_single_value,
    ACTIONS(43), 2,
      sym_plus_token,
      sym_minus_token,
    ACTIONS(45), 2,
      sym_true,
      sym_false,
    ACTIONS(47), 2,
      aux_sym_decimal_lit_token1,
      anon_sym_0,
    ACTIONS(53), 2,
      anon_sym_infinity,
      anon_sym_inf,
    STATE(158), 3,
      sym_bool_constant,
      sym_int_constant,
      sym_float_constant,
  [546] = 6,
    ACTIONS(41), 1,
      sym_identifier,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(61), 1,
      anon_sym_LBRACK,
    STATE(95), 1,
      sym_full_ident,
    STATE(221), 1,
      sym_type,
    ACTIONS(63), 22,
      anon_sym_bool,
      anon_sym_byte,
      anon_sym_ubyte,
      anon_sym_short,
      anon_sym_ushort,
      anon_sym_int,
      anon_sym_uint,
      anon_sym_float,
      anon_sym_long,
      anon_sym_ulong,
      anon_sym_double,
      anon_sym_int8,
      anon_sym_uint8,
      anon_sym_int16,
      anon_sym_uint16,
      anon_sym_int32,
      anon_sym_uint32,
      anon_sym_int64,
      anon_sym_uint64,
      anon_sym_float32,
      anon_sym_float64,
      anon_sym_string,
  [586] = 6,
    ACTIONS(41), 1,
      sym_identifier,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(61), 1,
      anon_sym_LBRACK,
    STATE(95), 1,
      sym_full_ident,
    STATE(183), 1,
      sym_type,
    ACTIONS(63), 22,
      anon_sym_bool,
      anon_sym_byte,
      anon_sym_ubyte,
      anon_sym_short,
      anon_sym_ushort,
      anon_sym_int,
      anon_sym_uint,
      anon_sym_float,
      anon_sym_long,
      anon_sym_ulong,
      anon_sym_double,
      anon_sym_int8,
      anon_sym_uint8,
      anon_sym_int16,
      anon_sym_uint16,
      anon_sym_int32,
      anon_sym_uint32,
      anon_sym_int64,
      anon_sym_uint64,
      anon_sym_float32,
      anon_sym_float64,
      anon_sym_string,
  [626] = 6,
    ACTIONS(41), 1,
      sym_identifier,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(61), 1,
      anon_sym_LBRACK,
    STATE(95), 1,
      sym_full_ident,
    STATE(170), 1,
      sym_type,
    ACTIONS(63), 22,
      anon_sym_bool,
      anon_sym_byte,
      anon_sym_ubyte,
      anon_sym_short,
      anon_sym_ushort,
      anon_sym_int,
      anon_sym_uint,
      anon_sym_float,
      anon_sym_long,
      anon_sym_ulong,
      anon_sym_double,
      anon_sym_int8,
      anon_sym_uint8,
      anon_sym_int16,
      anon_sym_uint16,
      anon_sym_int32,
      anon_sym_uint32,
      anon_sym_int64,
      anon_sym_uint64,
      anon_sym_float32,
      anon_sym_float64,
      anon_sym_string,
  [666] = 6,
    ACTIONS(41), 1,
      sym_identifier,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(61), 1,
      anon_sym_LBRACK,
    STATE(95), 1,
      sym_full_ident,
    STATE(172), 1,
      sym_type,
    ACTIONS(63), 22,
      anon_sym_bool,
      anon_sym_byte,
      anon_sym_ubyte,
      anon_sym_short,
      anon_sym_ushort,
      anon_sym_int,
      anon_sym_uint,
      anon_sym_float,
      anon_sym_long,
      anon_sym_ulong,
      anon_sym_double,
      anon_sym_int8,
      anon_sym_uint8,
      anon_sym_int16,
      anon_sym_uint16,
      anon_sym_int32,
      anon_sym_uint32,
      anon_sym_int64,
      anon_sym_uint64,
      anon_sym_float32,
      anon_sym_float64,
      anon_sym_string,
  [706] = 6,
    ACTIONS(41), 1,
      sym_identifier,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(61), 1,
      anon_sym_LBRACK,
    STATE(95), 1,
      sym_full_ident,
    STATE(163), 1,
      sym_type,
    ACTIONS(63), 22,
      anon_sym_bool,
      anon_sym_byte,
      anon_sym_ubyte,
      anon_sym_short,
      anon_sym_ushort,
      anon_sym_int,
      anon_sym_uint,
      anon_sym_float,
      anon_sym_long,
      anon_sym_ulong,
      anon_sym_double,
      anon_sym_int8,
      anon_sym_uint8,
      anon_sym_int16,
      anon_sym_uint16,
      anon_sym_int32,
      anon_sym_uint32,
      anon_sym_int64,
      anon_sym_uint64,
      anon_sym_float32,
      anon_sym_float64,
      anon_sym_string,
  [746] = 15,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_documentation,
    ACTIONS(13), 1,
      anon_sym_namespace,
    ACTIONS(15), 1,
      anon_sym_attribute,
    ACTIONS(19), 1,
      anon_sym_LBRACE,
    ACTIONS(21), 1,
      anon_sym_enum,
    ACTIONS(23), 1,
      anon_sym_union,
    ACTIONS(25), 1,
      anon_sym_root_type,
    ACTIONS(27), 1,
      anon_sym_file_extension,
    ACTIONS(29), 1,
      anon_sym_file_identifier,
    ACTIONS(31), 1,
      anon_sym_rpc_service,
    ACTIONS(33), 1,
      ts_builtin_sym_end,
    STATE(96), 1,
      aux_sym_type_decl_repeat1,
    ACTIONS(17), 2,
      anon_sym_table,
      anon_sym_struct,
    STATE(17), 11,
      sym_namespace_decl,
      sym_attribute_decl,
      sym_type_decl,
      sym_enum_decl,
      sym_union_decl,
      sym_object,
      sym_root_decl,
      sym_file_extension_decl,
      sym_file_identifier_decl,
      sym_rpc_decl,
      aux_sym_source_file_repeat2,
  [803] = 15,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_documentation,
    ACTIONS(13), 1,
      anon_sym_namespace,
    ACTIONS(15), 1,
      anon_sym_attribute,
    ACTIONS(19), 1,
      anon_sym_LBRACE,
    ACTIONS(21), 1,
      anon_sym_enum,
    ACTIONS(23), 1,
      anon_sym_union,
    ACTIONS(25), 1,
      anon_sym_root_type,
    ACTIONS(27), 1,
      anon_sym_file_extension,
    ACTIONS(29), 1,
      anon_sym_file_identifier,
    ACTIONS(31), 1,
      anon_sym_rpc_service,
    ACTIONS(65), 1,
      ts_builtin_sym_end,
    STATE(96), 1,
      aux_sym_type_decl_repeat1,
    ACTIONS(17), 2,
      anon_sym_table,
      anon_sym_struct,
    STATE(17), 11,
      sym_namespace_decl,
      sym_attribute_decl,
      sym_type_decl,
      sym_enum_decl,
      sym_union_decl,
      sym_object,
      sym_root_decl,
      sym_file_extension_decl,
      sym_file_identifier_decl,
      sym_rpc_decl,
      aux_sym_source_file_repeat2,
  [860] = 15,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(67), 1,
      ts_builtin_sym_end,
    ACTIONS(69), 1,
      sym_documentation,
    ACTIONS(72), 1,
      anon_sym_namespace,
    ACTIONS(75), 1,
      anon_sym_attribute,
    ACTIONS(81), 1,
      anon_sym_LBRACE,
    ACTIONS(84), 1,
      anon_sym_enum,
    ACTIONS(87), 1,
      anon_sym_union,
    ACTIONS(90), 1,
      anon_sym_root_type,
    ACTIONS(93), 1,
      anon_sym_file_extension,
    ACTIONS(96), 1,
      anon_sym_file_identifier,
    ACTIONS(99), 1,
      anon_sym_rpc_service,
    STATE(96), 1,
      aux_sym_type_decl_repeat1,
    ACTIONS(78), 2,
      anon_sym_table,
      anon_sym_struct,
    STATE(17), 11,
      sym_namespace_decl,
      sym_attribute_decl,
      sym_type_decl,
      sym_enum_decl,
      sym_union_decl,
      sym_object,
      sym_root_decl,
      sym_file_extension_decl,
      sym_file_identifier_decl,
      sym_rpc_decl,
      aux_sym_source_file_repeat2,
  [917] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(104), 1,
      anon_sym_include,
    ACTIONS(107), 1,
      anon_sym_native_include,
    STATE(18), 3,
      sym_include,
      sym_native_include,
      aux_sym_source_file_repeat1,
    ACTIONS(102), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [947] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(110), 15,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_include,
      anon_sym_native_include,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [968] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(112), 15,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_include,
      anon_sym_native_include,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [989] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(114), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1008] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(116), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1027] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(118), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1046] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(120), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1065] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(122), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1084] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(124), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1103] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(126), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1122] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(128), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1141] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(130), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1160] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(132), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1179] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(134), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1198] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(136), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1217] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(138), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1236] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(140), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1255] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(142), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1274] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(144), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1293] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(144), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1312] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(146), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1331] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(148), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1350] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(150), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1369] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(152), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1388] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(154), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1407] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(156), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1426] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(158), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1445] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(160), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1464] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(162), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1483] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(160), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1502] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(164), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1521] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(166), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1540] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(168), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1559] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(170), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1578] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(172), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1597] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(174), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1616] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(176), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1635] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(178), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1654] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(180), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1673] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(180), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1692] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(182), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1711] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(184), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1730] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(186), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1749] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(188), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1768] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(190), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1787] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(192), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1806] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(194), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1825] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(196), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1844] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(198), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1863] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(200), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1882] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(202), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1901] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(204), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1920] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(206), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1939] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(208), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1958] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(210), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1977] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(212), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [1996] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(214), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [2015] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(216), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [2034] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(218), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [2053] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(188), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [2072] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(220), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [2091] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(222), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [2110] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(224), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [2129] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(226), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [2148] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(162), 13,
      ts_builtin_sym_end,
      sym_documentation,
      anon_sym_namespace,
      anon_sym_attribute,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_LBRACE,
      anon_sym_enum,
      anon_sym_union,
      anon_sym_root_type,
      anon_sym_file_extension,
      anon_sym_file_identifier,
      anon_sym_rpc_service,
  [2167] = 10,
    ACTIONS(49), 1,
      sym_hex_lit,
    ACTIONS(53), 1,
      anon_sym_inf,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(228), 1,
      anon_sym_infinity,
    STATE(106), 1,
      sym_int_lit,
    STATE(130), 1,
      sym_decimal_lit,
    STATE(142), 1,
      sym_float_lit,
    STATE(154), 1,
      sym_inf_token,
    ACTIONS(47), 2,
      aux_sym_decimal_lit_token1,
      anon_sym_0,
    ACTIONS(51), 2,
      aux_sym_float_lit_token1,
      sym_nan_token,
  [2200] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(232), 1,
      anon_sym_DOT,
    STATE(84), 1,
      aux_sym_full_ident_repeat1,
    ACTIONS(230), 9,
      anon_sym_SEMI,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_EQ,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
  [2221] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(237), 1,
      anon_sym_DOT,
    STATE(84), 1,
      aux_sym_full_ident_repeat1,
    ACTIONS(235), 9,
      anon_sym_SEMI,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_EQ,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
  [2242] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(237), 1,
      anon_sym_DOT,
    STATE(85), 1,
      aux_sym_full_ident_repeat1,
    ACTIONS(239), 9,
      anon_sym_SEMI,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_EQ,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
  [2263] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(230), 10,
      anon_sym_SEMI,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_EQ,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_DOT,
  [2279] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(241), 1,
      sym_documentation,
    STATE(88), 1,
      aux_sym_type_decl_repeat1,
    ACTIONS(244), 7,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_RBRACE,
      anon_sym_enum,
      anon_sym_COMMA,
      anon_sym_union,
      anon_sym_rpc_service,
  [2298] = 9,
    ACTIONS(47), 1,
      anon_sym_0,
    ACTIONS(49), 1,
      sym_hex_lit,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(246), 1,
      aux_sym_decimal_lit_token1,
    STATE(130), 1,
      sym_decimal_lit,
    STATE(131), 1,
      sym_int_lit,
    STATE(151), 1,
      sym_plus_minus_constant,
    STATE(182), 1,
      sym_int_constant,
    ACTIONS(43), 2,
      sym_plus_token,
      sym_minus_token,
  [2327] = 9,
    ACTIONS(47), 1,
      anon_sym_0,
    ACTIONS(49), 1,
      sym_hex_lit,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(246), 1,
      aux_sym_decimal_lit_token1,
    STATE(130), 1,
      sym_decimal_lit,
    STATE(131), 1,
      sym_int_lit,
    STATE(151), 1,
      sym_plus_minus_constant,
    STATE(179), 1,
      sym_int_constant,
    ACTIONS(43), 2,
      sym_plus_token,
      sym_minus_token,
  [2356] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(248), 8,
      sym_documentation,
      anon_sym_table,
      anon_sym_struct,
      anon_sym_RBRACE,
      anon_sym_enum,
      anon_sym_COMMA,
      anon_sym_union,
      anon_sym_rpc_service,
  [2370] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(250), 8,
      anon_sym_SEMI,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_EQ,
      anon_sym_LPAREN,
  [2384] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(252), 8,
      anon_sym_SEMI,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_EQ,
      anon_sym_LPAREN,
  [2398] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(254), 8,
      anon_sym_SEMI,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_EQ,
      anon_sym_LPAREN,
  [2412] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(256), 8,
      anon_sym_SEMI,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_COLON,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_EQ,
      anon_sym_LPAREN,
  [2426] = 8,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_documentation,
    ACTIONS(258), 1,
      anon_sym_table,
    ACTIONS(260), 1,
      anon_sym_struct,
    ACTIONS(262), 1,
      anon_sym_enum,
    ACTIONS(264), 1,
      anon_sym_union,
    ACTIONS(266), 1,
      anon_sym_rpc_service,
    STATE(88), 1,
      aux_sym_type_decl_repeat1,
  [2451] = 3,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(268), 3,
      aux_sym_decimal_lit_token1,
      anon_sym_0,
      anon_sym_inf,
    ACTIONS(270), 4,
      sym_hex_lit,
      aux_sym_float_lit_token1,
      anon_sym_infinity,
      sym_nan_token,
  [2466] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(274), 1,
      anon_sym_RBRACE,
    ACTIONS(276), 1,
      sym_identifier,
    STATE(248), 1,
      aux_sym_type_decl_repeat1,
    STATE(107), 2,
      sym_rpc_method,
      aux_sym_rpc_decl_repeat1,
  [2486] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(278), 1,
      anon_sym_RBRACE,
    ACTIONS(280), 1,
      sym_identifier,
    STATE(105), 1,
      aux_sym_type_decl_repeat2,
    STATE(240), 1,
      sym_field_decl,
    STATE(245), 1,
      aux_sym_type_decl_repeat1,
  [2508] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(282), 1,
      anon_sym_RBRACE,
    ACTIONS(284), 1,
      sym_identifier,
    STATE(144), 1,
      sym_full_ident,
    STATE(167), 1,
      aux_sym_type_decl_repeat1,
    STATE(226), 1,
      sym_union_field_decl,
  [2530] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(276), 1,
      sym_identifier,
    ACTIONS(286), 1,
      anon_sym_RBRACE,
    STATE(248), 1,
      aux_sym_type_decl_repeat1,
    STATE(128), 2,
      sym_rpc_method,
      aux_sym_rpc_decl_repeat1,
  [2550] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(280), 1,
      sym_identifier,
    ACTIONS(288), 1,
      anon_sym_RBRACE,
    STATE(105), 1,
      aux_sym_type_decl_repeat2,
    STATE(240), 1,
      sym_field_decl,
    STATE(245), 1,
      aux_sym_type_decl_repeat1,
  [2572] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(284), 1,
      sym_identifier,
    ACTIONS(290), 1,
      anon_sym_RBRACE,
    STATE(144), 1,
      sym_full_ident,
    STATE(167), 1,
      aux_sym_type_decl_repeat1,
    STATE(258), 1,
      sym_union_field_decl,
  [2594] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(284), 1,
      sym_identifier,
    ACTIONS(292), 1,
      anon_sym_RBRACE,
    STATE(144), 1,
      sym_full_ident,
    STATE(167), 1,
      aux_sym_type_decl_repeat1,
    STATE(212), 1,
      sym_union_field_decl,
  [2616] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(294), 1,
      sym_documentation,
    ACTIONS(297), 1,
      anon_sym_RBRACE,
    ACTIONS(299), 1,
      sym_identifier,
    STATE(105), 1,
      aux_sym_type_decl_repeat2,
    STATE(240), 1,
      sym_field_decl,
    STATE(245), 1,
      aux_sym_type_decl_repeat1,
  [2638] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(302), 6,
      anon_sym_SEMI,
      anon_sym_RBRACE,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
  [2650] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(276), 1,
      sym_identifier,
    ACTIONS(304), 1,
      anon_sym_RBRACE,
    STATE(248), 1,
      aux_sym_type_decl_repeat1,
    STATE(117), 2,
      sym_rpc_method,
      aux_sym_rpc_decl_repeat1,
  [2670] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(280), 1,
      sym_identifier,
    ACTIONS(306), 1,
      anon_sym_RBRACE,
    STATE(105), 1,
      aux_sym_type_decl_repeat2,
    STATE(240), 1,
      sym_field_decl,
    STATE(245), 1,
      aux_sym_type_decl_repeat1,
  [2692] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(280), 1,
      sym_identifier,
    ACTIONS(308), 1,
      anon_sym_RBRACE,
    STATE(105), 1,
      aux_sym_type_decl_repeat2,
    STATE(240), 1,
      sym_field_decl,
    STATE(245), 1,
      aux_sym_type_decl_repeat1,
  [2714] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(280), 1,
      sym_identifier,
    ACTIONS(310), 1,
      anon_sym_RBRACE,
    STATE(109), 1,
      aux_sym_type_decl_repeat2,
    STATE(240), 1,
      sym_field_decl,
    STATE(245), 1,
      aux_sym_type_decl_repeat1,
  [2736] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(284), 1,
      sym_identifier,
    ACTIONS(312), 1,
      anon_sym_RBRACE,
    STATE(144), 1,
      sym_full_ident,
    STATE(167), 1,
      aux_sym_type_decl_repeat1,
    STATE(258), 1,
      sym_union_field_decl,
  [2758] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(284), 1,
      sym_identifier,
    ACTIONS(314), 1,
      anon_sym_RBRACE,
    STATE(144), 1,
      sym_full_ident,
    STATE(167), 1,
      aux_sym_type_decl_repeat1,
    STATE(258), 1,
      sym_union_field_decl,
  [2780] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(284), 1,
      sym_identifier,
    ACTIONS(316), 1,
      anon_sym_RBRACE,
    STATE(144), 1,
      sym_full_ident,
    STATE(167), 1,
      aux_sym_type_decl_repeat1,
    STATE(258), 1,
      sym_union_field_decl,
  [2802] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(284), 1,
      sym_identifier,
    ACTIONS(318), 1,
      anon_sym_RBRACE,
    STATE(144), 1,
      sym_full_ident,
    STATE(167), 1,
      aux_sym_type_decl_repeat1,
    STATE(258), 1,
      sym_union_field_decl,
  [2824] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(284), 1,
      sym_identifier,
    ACTIONS(320), 1,
      anon_sym_RBRACE,
    STATE(144), 1,
      sym_full_ident,
    STATE(167), 1,
      aux_sym_type_decl_repeat1,
    STATE(258), 1,
      sym_union_field_decl,
  [2846] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(280), 1,
      sym_identifier,
    ACTIONS(322), 1,
      anon_sym_RBRACE,
    STATE(108), 1,
      aux_sym_type_decl_repeat2,
    STATE(240), 1,
      sym_field_decl,
    STATE(245), 1,
      aux_sym_type_decl_repeat1,
  [2868] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(324), 1,
      sym_documentation,
    ACTIONS(327), 1,
      anon_sym_RBRACE,
    ACTIONS(329), 1,
      sym_identifier,
    STATE(248), 1,
      aux_sym_type_decl_repeat1,
    STATE(117), 2,
      sym_rpc_method,
      aux_sym_rpc_decl_repeat1,
  [2888] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(284), 1,
      sym_identifier,
    ACTIONS(332), 1,
      anon_sym_RBRACE,
    STATE(144), 1,
      sym_full_ident,
    STATE(167), 1,
      aux_sym_type_decl_repeat1,
    STATE(214), 1,
      sym_union_field_decl,
  [2910] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(280), 1,
      sym_identifier,
    ACTIONS(334), 1,
      anon_sym_RBRACE,
    STATE(120), 1,
      aux_sym_type_decl_repeat2,
    STATE(240), 1,
      sym_field_decl,
    STATE(245), 1,
      aux_sym_type_decl_repeat1,
  [2932] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(280), 1,
      sym_identifier,
    ACTIONS(336), 1,
      anon_sym_RBRACE,
    STATE(105), 1,
      aux_sym_type_decl_repeat2,
    STATE(240), 1,
      sym_field_decl,
    STATE(245), 1,
      aux_sym_type_decl_repeat1,
  [2954] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(280), 1,
      sym_identifier,
    ACTIONS(338), 1,
      anon_sym_RBRACE,
    STATE(124), 1,
      aux_sym_type_decl_repeat2,
    STATE(240), 1,
      sym_field_decl,
    STATE(245), 1,
      aux_sym_type_decl_repeat1,
  [2976] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(280), 1,
      sym_identifier,
    ACTIONS(340), 1,
      anon_sym_RBRACE,
    STATE(99), 1,
      aux_sym_type_decl_repeat2,
    STATE(240), 1,
      sym_field_decl,
    STATE(245), 1,
      aux_sym_type_decl_repeat1,
  [2998] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(284), 1,
      sym_identifier,
    ACTIONS(342), 1,
      anon_sym_RBRACE,
    STATE(144), 1,
      sym_full_ident,
    STATE(167), 1,
      aux_sym_type_decl_repeat1,
    STATE(258), 1,
      sym_union_field_decl,
  [3020] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(280), 1,
      sym_identifier,
    ACTIONS(344), 1,
      anon_sym_RBRACE,
    STATE(105), 1,
      aux_sym_type_decl_repeat2,
    STATE(240), 1,
      sym_field_decl,
    STATE(245), 1,
      aux_sym_type_decl_repeat1,
  [3042] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(280), 1,
      sym_identifier,
    ACTIONS(346), 1,
      anon_sym_RBRACE,
    STATE(102), 1,
      aux_sym_type_decl_repeat2,
    STATE(240), 1,
      sym_field_decl,
    STATE(245), 1,
      aux_sym_type_decl_repeat1,
  [3064] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(284), 1,
      sym_identifier,
    ACTIONS(348), 1,
      anon_sym_RBRACE,
    STATE(144), 1,
      sym_full_ident,
    STATE(167), 1,
      aux_sym_type_decl_repeat1,
    STATE(193), 1,
      sym_union_field_decl,
  [3086] = 7,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(284), 1,
      sym_identifier,
    ACTIONS(350), 1,
      anon_sym_RBRACE,
    STATE(144), 1,
      sym_full_ident,
    STATE(167), 1,
      aux_sym_type_decl_repeat1,
    STATE(258), 1,
      sym_union_field_decl,
  [3108] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(276), 1,
      sym_identifier,
    ACTIONS(352), 1,
      anon_sym_RBRACE,
    STATE(248), 1,
      aux_sym_type_decl_repeat1,
    STATE(117), 2,
      sym_rpc_method,
      aux_sym_rpc_decl_repeat1,
  [3128] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(354), 6,
      anon_sym_SEMI,
      anon_sym_RBRACE,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
  [3140] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(356), 6,
      anon_sym_SEMI,
      anon_sym_RBRACE,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
  [3152] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(358), 6,
      anon_sym_SEMI,
      anon_sym_RBRACE,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
  [3164] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_documentation,
    ACTIONS(360), 1,
      anon_sym_RBRACE,
    ACTIONS(362), 1,
      anon_sym_COMMA,
    STATE(146), 1,
      aux_sym_type_decl_repeat1,
    STATE(233), 1,
      aux_sym_object_repeat1,
  [3183] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(364), 1,
      anon_sym_RBRACE,
    ACTIONS(366), 1,
      sym_identifier,
    STATE(199), 1,
      aux_sym_type_decl_repeat1,
    STATE(267), 1,
      sym_enum_val_decl,
  [3202] = 5,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(370), 1,
      anon_sym_COLON,
    ACTIONS(372), 1,
      anon_sym_LPAREN,
    STATE(257), 1,
      sym_metadata,
    ACTIONS(368), 2,
      anon_sym_RBRACE,
      anon_sym_COMMA,
  [3219] = 5,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(372), 1,
      anon_sym_LPAREN,
    ACTIONS(376), 1,
      anon_sym_EQ,
    STATE(270), 1,
      sym_metadata,
    ACTIONS(374), 2,
      anon_sym_RBRACE,
      anon_sym_COMMA,
  [3236] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(366), 1,
      sym_identifier,
    ACTIONS(378), 1,
      anon_sym_RBRACE,
    STATE(199), 1,
      aux_sym_type_decl_repeat1,
    STATE(267), 1,
      sym_enum_val_decl,
  [3255] = 5,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(372), 1,
      anon_sym_LPAREN,
    ACTIONS(382), 1,
      anon_sym_EQ,
    STATE(262), 1,
      sym_metadata,
    ACTIONS(380), 2,
      anon_sym_RBRACE,
      anon_sym_COMMA,
  [3272] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(384), 5,
      anon_sym_SEMI,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
  [3283] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(366), 1,
      sym_identifier,
    ACTIONS(386), 1,
      anon_sym_RBRACE,
    STATE(199), 1,
      aux_sym_type_decl_repeat1,
    STATE(267), 1,
      sym_enum_val_decl,
  [3302] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(284), 1,
      sym_identifier,
    STATE(144), 1,
      sym_full_ident,
    STATE(167), 1,
      aux_sym_type_decl_repeat1,
    STATE(258), 1,
      sym_union_field_decl,
  [3321] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(388), 5,
      anon_sym_SEMI,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
  [3332] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(390), 5,
      anon_sym_SEMI,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
  [3343] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(392), 5,
      anon_sym_SEMI,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
  [3354] = 5,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(372), 1,
      anon_sym_LPAREN,
    ACTIONS(396), 1,
      anon_sym_COLON,
    STATE(265), 1,
      sym_metadata,
    ACTIONS(394), 2,
      anon_sym_RBRACE,
      anon_sym_COMMA,
  [3371] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(398), 5,
      anon_sym_SEMI,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
  [3382] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_documentation,
    ACTIONS(362), 1,
      anon_sym_COMMA,
    ACTIONS(400), 1,
      anon_sym_RBRACE,
    STATE(88), 1,
      aux_sym_type_decl_repeat1,
    STATE(213), 1,
      aux_sym_object_repeat1,
  [3401] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(366), 1,
      sym_identifier,
    ACTIONS(402), 1,
      anon_sym_RBRACE,
    STATE(199), 1,
      aux_sym_type_decl_repeat1,
    STATE(267), 1,
      sym_enum_val_decl,
  [3420] = 6,
    ACTIONS(47), 1,
      anon_sym_0,
    ACTIONS(49), 1,
      sym_hex_lit,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(246), 1,
      aux_sym_decimal_lit_token1,
    STATE(130), 1,
      sym_decimal_lit,
    STATE(291), 1,
      sym_int_lit,
  [3439] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(404), 5,
      anon_sym_SEMI,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
  [3450] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(366), 1,
      sym_identifier,
    ACTIONS(406), 1,
      anon_sym_RBRACE,
    STATE(199), 1,
      aux_sym_type_decl_repeat1,
    STATE(267), 1,
      sym_enum_val_decl,
  [3469] = 6,
    ACTIONS(47), 1,
      anon_sym_0,
    ACTIONS(49), 1,
      sym_hex_lit,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(246), 1,
      aux_sym_decimal_lit_token1,
    STATE(106), 1,
      sym_int_lit,
    STATE(130), 1,
      sym_decimal_lit,
  [3488] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(366), 1,
      sym_identifier,
    ACTIONS(408), 1,
      anon_sym_RBRACE,
    STATE(199), 1,
      aux_sym_type_decl_repeat1,
    STATE(267), 1,
      sym_enum_val_decl,
  [3507] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(366), 1,
      sym_identifier,
    ACTIONS(410), 1,
      anon_sym_RBRACE,
    STATE(199), 1,
      aux_sym_type_decl_repeat1,
    STATE(267), 1,
      sym_enum_val_decl,
  [3526] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(412), 5,
      anon_sym_SEMI,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
  [3537] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(414), 5,
      anon_sym_SEMI,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
  [3548] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(416), 5,
      anon_sym_SEMI,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
  [3559] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(418), 5,
      anon_sym_SEMI,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
  [3570] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(420), 5,
      anon_sym_SEMI,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
  [3581] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(366), 1,
      sym_identifier,
    ACTIONS(422), 1,
      anon_sym_RBRACE,
    STATE(199), 1,
      aux_sym_type_decl_repeat1,
    STATE(267), 1,
      sym_enum_val_decl,
  [3600] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(424), 5,
      anon_sym_SEMI,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
  [3611] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_documentation,
    ACTIONS(362), 1,
      anon_sym_COMMA,
    ACTIONS(426), 1,
      anon_sym_RBRACE,
    STATE(162), 1,
      aux_sym_type_decl_repeat1,
    STATE(250), 1,
      aux_sym_object_repeat1,
  [3630] = 6,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_documentation,
    ACTIONS(362), 1,
      anon_sym_COMMA,
    ACTIONS(428), 1,
      anon_sym_RBRACE,
    STATE(88), 1,
      aux_sym_type_decl_repeat1,
    STATE(191), 1,
      aux_sym_object_repeat1,
  [3649] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(372), 1,
      anon_sym_LPAREN,
    STATE(252), 1,
      sym_metadata,
    ACTIONS(430), 2,
      anon_sym_RBRACE,
      anon_sym_COMMA,
  [3663] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(126), 4,
      anon_sym_SEMI,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_LPAREN,
  [3673] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(432), 4,
      anon_sym_SEMI,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_COMMA,
  [3683] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(366), 1,
      sym_identifier,
    STATE(198), 1,
      sym_enum_val_decl,
    STATE(199), 1,
      aux_sym_type_decl_repeat1,
  [3699] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(284), 1,
      sym_identifier,
    STATE(134), 1,
      sym_full_ident,
    STATE(247), 1,
      aux_sym_type_decl_repeat1,
  [3715] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(366), 1,
      sym_identifier,
    STATE(199), 1,
      aux_sym_type_decl_repeat1,
    STATE(202), 1,
      sym_enum_val_decl,
  [3731] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(434), 1,
      anon_sym_DQUOTE,
    ACTIONS(436), 1,
      aux_sym_string_constant_token1,
    ACTIONS(438), 1,
      sym_escape_sequence,
    STATE(184), 1,
      aux_sym_string_constant_repeat1,
  [3747] = 5,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(372), 1,
      anon_sym_LPAREN,
    ACTIONS(440), 1,
      anon_sym_SEMI,
    ACTIONS(442), 1,
      anon_sym_EQ,
    STATE(305), 1,
      sym_metadata,
  [3763] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(444), 1,
      anon_sym_DQUOTE,
    ACTIONS(446), 1,
      aux_sym_string_constant_token1,
    ACTIONS(448), 1,
      sym_escape_sequence,
    STATE(169), 1,
      aux_sym_string_constant_repeat1,
  [3779] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(372), 1,
      anon_sym_LPAREN,
    STATE(253), 1,
      sym_metadata,
    ACTIONS(450), 2,
      anon_sym_RBRACE,
      anon_sym_COMMA,
  [3793] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(452), 4,
      anon_sym_SEMI,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_COMMA,
  [3803] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(366), 1,
      sym_identifier,
    STATE(199), 1,
      aux_sym_type_decl_repeat1,
    STATE(251), 1,
      sym_enum_val_decl,
  [3819] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(366), 1,
      sym_identifier,
    STATE(190), 1,
      sym_enum_val_decl,
    STATE(199), 1,
      aux_sym_type_decl_repeat1,
  [3835] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_documentation,
    STATE(88), 1,
      aux_sym_type_decl_repeat1,
    ACTIONS(454), 2,
      anon_sym_RBRACE,
      anon_sym_COMMA,
  [3849] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(456), 4,
      anon_sym_SEMI,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_LPAREN,
  [3859] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(458), 4,
      anon_sym_SEMI,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_LPAREN,
  [3869] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(372), 1,
      anon_sym_LPAREN,
    STATE(256), 1,
      sym_metadata,
    ACTIONS(460), 2,
      anon_sym_RBRACE,
      anon_sym_COMMA,
  [3883] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(7), 1,
      sym_documentation,
    STATE(176), 1,
      aux_sym_type_decl_repeat1,
    ACTIONS(462), 2,
      anon_sym_RBRACE,
      anon_sym_COMMA,
  [3897] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(366), 1,
      sym_identifier,
    STATE(199), 1,
      aux_sym_type_decl_repeat1,
    STATE(267), 1,
      sym_enum_val_decl,
  [3913] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(372), 1,
      anon_sym_LPAREN,
    STATE(261), 1,
      sym_metadata,
    ACTIONS(464), 2,
      anon_sym_RBRACE,
      anon_sym_COMMA,
  [3927] = 5,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(372), 1,
      anon_sym_LPAREN,
    ACTIONS(466), 1,
      anon_sym_SEMI,
    ACTIONS(468), 1,
      anon_sym_EQ,
    STATE(276), 1,
      sym_metadata,
  [3943] = 5,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(470), 1,
      anon_sym_DQUOTE,
    ACTIONS(472), 1,
      aux_sym_string_constant_token1,
    ACTIONS(475), 1,
      sym_escape_sequence,
    STATE(184), 1,
      aux_sym_string_constant_repeat1,
  [3959] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(478), 4,
      anon_sym_SEMI,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_LPAREN,
  [3969] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(480), 4,
      anon_sym_SEMI,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_LPAREN,
  [3979] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(114), 4,
      anon_sym_SEMI,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_LPAREN,
  [3989] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(188), 4,
      anon_sym_SEMI,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_LPAREN,
  [3999] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(188), 4,
      anon_sym_SEMI,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_LPAREN,
  [4009] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(482), 1,
      anon_sym_RBRACE,
    ACTIONS(484), 1,
      anon_sym_COMMA,
    STATE(196), 1,
      aux_sym_enum_decl_repeat1,
  [4022] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(362), 1,
      anon_sym_COMMA,
    ACTIONS(486), 1,
      anon_sym_RBRACE,
    STATE(217), 1,
      aux_sym_object_repeat1,
  [4035] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(488), 1,
      anon_sym_RBRACE,
    ACTIONS(490), 1,
      anon_sym_COMMA,
    STATE(209), 1,
      aux_sym_union_decl_repeat1,
  [4048] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(492), 1,
      anon_sym_RBRACE,
    ACTIONS(494), 1,
      anon_sym_COMMA,
    STATE(205), 1,
      aux_sym_union_decl_repeat1,
  [4061] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(496), 3,
      sym_documentation,
      anon_sym_RBRACE,
      sym_identifier,
  [4070] = 4,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(57), 1,
      aux_sym_string_constant_token2,
    ACTIONS(59), 1,
      sym_comment,
    STATE(306), 1,
      sym_string_constant,
  [4083] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(498), 1,
      anon_sym_RBRACE,
    ACTIONS(500), 1,
      anon_sym_COMMA,
    STATE(215), 1,
      aux_sym_enum_decl_repeat1,
  [4096] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(502), 1,
      anon_sym_COMMA,
    ACTIONS(504), 1,
      anon_sym_RPAREN,
    STATE(201), 1,
      aux_sym_metadata_repeat1,
  [4109] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(506), 1,
      anon_sym_RBRACE,
    ACTIONS(508), 1,
      anon_sym_COMMA,
    STATE(218), 1,
      aux_sym_enum_decl_repeat1,
  [4122] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(510), 1,
      sym_identifier,
    STATE(247), 1,
      aux_sym_type_decl_repeat1,
  [4135] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(512), 1,
      anon_sym_COMMA,
    ACTIONS(515), 1,
      anon_sym_RPAREN,
    STATE(200), 1,
      aux_sym_metadata_repeat1,
  [4148] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(502), 1,
      anon_sym_COMMA,
    ACTIONS(517), 1,
      anon_sym_RPAREN,
    STATE(200), 1,
      aux_sym_metadata_repeat1,
  [4161] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(519), 1,
      anon_sym_RBRACE,
    ACTIONS(521), 1,
      anon_sym_COMMA,
    STATE(219), 1,
      aux_sym_enum_decl_repeat1,
  [4174] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(372), 1,
      anon_sym_LPAREN,
    ACTIONS(523), 1,
      anon_sym_LBRACE,
    STATE(279), 1,
      sym_metadata,
  [4187] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(525), 1,
      anon_sym_RBRACE,
    ACTIONS(527), 1,
      anon_sym_COMMA,
    STATE(209), 1,
      aux_sym_union_decl_repeat1,
  [4200] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(529), 1,
      anon_sym_RBRACE,
    ACTIONS(531), 1,
      anon_sym_COMMA,
    STATE(209), 1,
      aux_sym_union_decl_repeat1,
  [4213] = 4,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(57), 1,
      aux_sym_string_constant_token2,
    ACTIONS(59), 1,
      sym_comment,
    STATE(275), 1,
      sym_string_constant,
  [4226] = 4,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(57), 1,
      aux_sym_string_constant_token2,
    ACTIONS(59), 1,
      sym_comment,
    STATE(296), 1,
      sym_string_constant,
  [4239] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(372), 1,
      anon_sym_LPAREN,
    ACTIONS(533), 1,
      anon_sym_SEMI,
    STATE(320), 1,
      sym_metadata,
  [4252] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(535), 1,
      anon_sym_RBRACE,
    ACTIONS(537), 1,
      anon_sym_COMMA,
    STATE(209), 1,
      aux_sym_union_decl_repeat1,
  [4265] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(540), 3,
      sym_documentation,
      anon_sym_RBRACE,
      sym_identifier,
  [4274] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(542), 3,
      sym_documentation,
      anon_sym_RBRACE,
      sym_identifier,
  [4283] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(544), 1,
      anon_sym_RBRACE,
    ACTIONS(546), 1,
      anon_sym_COMMA,
    STATE(204), 1,
      aux_sym_union_decl_repeat1,
  [4296] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(362), 1,
      anon_sym_COMMA,
    ACTIONS(548), 1,
      anon_sym_RBRACE,
    STATE(217), 1,
      aux_sym_object_repeat1,
  [4309] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(550), 1,
      anon_sym_RBRACE,
    ACTIONS(552), 1,
      anon_sym_COMMA,
    STATE(231), 1,
      aux_sym_union_decl_repeat1,
  [4322] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(554), 1,
      anon_sym_RBRACE,
    ACTIONS(556), 1,
      anon_sym_COMMA,
    STATE(215), 1,
      aux_sym_enum_decl_repeat1,
  [4335] = 4,
    ACTIONS(35), 1,
      anon_sym_DQUOTE,
    ACTIONS(57), 1,
      aux_sym_string_constant_token2,
    ACTIONS(59), 1,
      sym_comment,
    STATE(321), 1,
      sym_string_constant,
  [4348] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(559), 1,
      anon_sym_RBRACE,
    ACTIONS(561), 1,
      anon_sym_COMMA,
    STATE(217), 1,
      aux_sym_object_repeat1,
  [4361] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(564), 1,
      anon_sym_RBRACE,
    ACTIONS(566), 1,
      anon_sym_COMMA,
    STATE(215), 1,
      aux_sym_enum_decl_repeat1,
  [4374] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(568), 1,
      anon_sym_RBRACE,
    ACTIONS(570), 1,
      anon_sym_COMMA,
    STATE(215), 1,
      aux_sym_enum_decl_repeat1,
  [4387] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(372), 1,
      anon_sym_LPAREN,
    ACTIONS(572), 1,
      anon_sym_LBRACE,
    STATE(285), 1,
      sym_metadata,
  [4400] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(372), 1,
      anon_sym_LPAREN,
    ACTIONS(574), 1,
      anon_sym_LBRACE,
    STATE(299), 1,
      sym_metadata,
  [4413] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(576), 1,
      anon_sym_COMMA,
    ACTIONS(578), 1,
      anon_sym_RBRACK,
    STATE(234), 1,
      aux_sym_value_repeat1,
  [4426] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(580), 3,
      sym_documentation,
      anon_sym_RBRACE,
      sym_identifier,
  [4435] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(372), 1,
      anon_sym_LPAREN,
    ACTIONS(582), 1,
      anon_sym_SEMI,
    STATE(293), 1,
      sym_metadata,
  [4448] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(584), 3,
      sym_documentation,
      anon_sym_RBRACE,
      sym_identifier,
  [4457] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(586), 1,
      anon_sym_RBRACE,
    ACTIONS(588), 1,
      anon_sym_COMMA,
    STATE(192), 1,
      aux_sym_union_decl_repeat1,
  [4470] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(372), 1,
      anon_sym_LPAREN,
    ACTIONS(590), 1,
      anon_sym_LBRACE,
    STATE(308), 1,
      sym_metadata,
  [4483] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(372), 1,
      anon_sym_LPAREN,
    ACTIONS(592), 1,
      anon_sym_LBRACE,
    STATE(307), 1,
      sym_metadata,
  [4496] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(372), 1,
      anon_sym_LPAREN,
    ACTIONS(594), 1,
      anon_sym_SEMI,
    STATE(297), 1,
      sym_metadata,
  [4509] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(372), 1,
      anon_sym_LPAREN,
    ACTIONS(596), 1,
      anon_sym_LBRACE,
    STATE(273), 1,
      sym_metadata,
  [4522] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(598), 1,
      anon_sym_RBRACE,
    ACTIONS(600), 1,
      anon_sym_COMMA,
    STATE(209), 1,
      aux_sym_union_decl_repeat1,
  [4535] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(602), 1,
      anon_sym_RBRACE,
    ACTIONS(604), 1,
      anon_sym_COMMA,
    STATE(215), 1,
      aux_sym_enum_decl_repeat1,
  [4548] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(362), 1,
      anon_sym_COMMA,
    ACTIONS(606), 1,
      anon_sym_RBRACE,
    STATE(217), 1,
      aux_sym_object_repeat1,
  [4561] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(576), 1,
      anon_sym_COMMA,
    ACTIONS(608), 1,
      anon_sym_RBRACK,
    STATE(241), 1,
      aux_sym_value_repeat1,
  [4574] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(610), 3,
      sym_documentation,
      anon_sym_RBRACE,
      sym_identifier,
  [4583] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(612), 3,
      sym_documentation,
      anon_sym_RBRACE,
      sym_identifier,
  [4592] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(614), 3,
      sym_documentation,
      anon_sym_RBRACE,
      sym_identifier,
  [4601] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(372), 1,
      anon_sym_LPAREN,
    ACTIONS(616), 1,
      anon_sym_SEMI,
    STATE(309), 1,
      sym_metadata,
  [4614] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(372), 1,
      anon_sym_LPAREN,
    ACTIONS(618), 1,
      anon_sym_LBRACE,
    STATE(282), 1,
      sym_metadata,
  [4627] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(620), 3,
      sym_documentation,
      anon_sym_RBRACE,
      sym_identifier,
  [4636] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(622), 1,
      anon_sym_COMMA,
    ACTIONS(625), 1,
      anon_sym_RBRACK,
    STATE(241), 1,
      aux_sym_value_repeat1,
  [4649] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(627), 3,
      sym_documentation,
      anon_sym_RBRACE,
      sym_identifier,
  [4658] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(629), 3,
      sym_documentation,
      anon_sym_RBRACE,
      sym_identifier,
  [4667] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(631), 3,
      sym_documentation,
      anon_sym_RBRACE,
      sym_identifier,
  [4676] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(633), 1,
      sym_identifier,
    STATE(247), 1,
      aux_sym_type_decl_repeat1,
  [4689] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(635), 3,
      sym_documentation,
      anon_sym_RBRACE,
      sym_identifier,
  [4698] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(244), 1,
      sym_identifier,
    ACTIONS(637), 1,
      sym_documentation,
    STATE(247), 1,
      aux_sym_type_decl_repeat1,
  [4711] = 4,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(272), 1,
      sym_documentation,
    ACTIONS(640), 1,
      sym_identifier,
    STATE(247), 1,
      aux_sym_type_decl_repeat1,
  [4724] = 3,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(642), 1,
      anon_sym_COLON,
    ACTIONS(644), 2,
      anon_sym_COMMA,
      anon_sym_RPAREN,
  [4735] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(362), 1,
      anon_sym_COMMA,
    ACTIONS(646), 1,
      anon_sym_RBRACE,
    STATE(217), 1,
      aux_sym_object_repeat1,
  [4748] = 4,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(648), 1,
      anon_sym_RBRACE,
    ACTIONS(650), 1,
      anon_sym_COMMA,
    STATE(232), 1,
      aux_sym_enum_decl_repeat1,
  [4761] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(652), 2,
      anon_sym_RBRACE,
      anon_sym_COMMA,
  [4769] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(654), 2,
      anon_sym_RBRACE,
      anon_sym_COMMA,
  [4777] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(656), 2,
      anon_sym_COMMA,
      anon_sym_RPAREN,
  [4785] = 3,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(658), 1,
      anon_sym_COLON,
    ACTIONS(660), 1,
      anon_sym_RBRACK,
  [4795] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(662), 2,
      anon_sym_RBRACE,
      anon_sym_COMMA,
  [4803] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(664), 2,
      anon_sym_RBRACE,
      anon_sym_COMMA,
  [4811] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(666), 2,
      anon_sym_RBRACE,
      anon_sym_COMMA,
  [4819] = 3,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(668), 1,
      sym_identifier,
    STATE(197), 1,
      sym_field_and_value,
  [4829] = 3,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(284), 1,
      sym_identifier,
    STATE(295), 1,
      sym_full_ident,
  [4839] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(670), 2,
      anon_sym_RBRACE,
      anon_sym_COMMA,
  [4847] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(672), 2,
      anon_sym_RBRACE,
      anon_sym_COMMA,
  [4855] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(674), 2,
      anon_sym_COMMA,
      anon_sym_RPAREN,
  [4863] = 3,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(676), 1,
      anon_sym_DQUOTE,
    ACTIONS(678), 1,
      sym_identifier,
  [4873] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(680), 2,
      anon_sym_RBRACE,
      anon_sym_COMMA,
  [4881] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(682), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
  [4889] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(684), 2,
      anon_sym_RBRACE,
      anon_sym_COMMA,
  [4897] = 2,
    ACTIONS(3), 1,
      sym_comment,
    ACTIONS(248), 2,
      sym_documentation,
      sym_identifier,
  [4905] = 3,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(668), 1,
      sym_identifier,
    STATE(254), 1,
      sym_field_and_value,
  [4915] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(686), 2,
      anon_sym_RBRACE,
      anon_sym_COMMA,
  [4923] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(688), 1,
      sym_identifier,
  [4930] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(690), 1,
      sym_identifier,
  [4937] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(692), 1,
      anon_sym_LBRACE,
  [4944] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(694), 1,
      sym_identifier,
  [4951] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(696), 1,
      anon_sym_SEMI,
  [4958] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(698), 1,
      anon_sym_SEMI,
  [4965] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(700), 1,
      anon_sym_COLON,
  [4972] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(702), 1,
      anon_sym_COLON,
  [4979] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(704), 1,
      anon_sym_LBRACE,
  [4986] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(706), 1,
      sym_identifier,
  [4993] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(708), 1,
      sym_identifier,
  [5000] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(710), 1,
      anon_sym_LBRACE,
  [5007] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(712), 1,
      anon_sym_LBRACE,
  [5014] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(714), 1,
      sym_identifier,
  [5021] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(716), 1,
      anon_sym_LBRACE,
  [5028] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(718), 1,
      sym_identifier,
  [5035] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(720), 1,
      sym_identifier,
  [5042] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(722), 1,
      anon_sym_COLON,
  [5049] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(724), 1,
      anon_sym_RPAREN,
  [5056] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(726), 1,
      anon_sym_SEMI,
  [5063] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(728), 1,
      anon_sym_RBRACK,
  [5070] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(730), 1,
      sym_identifier,
  [5077] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(732), 1,
      anon_sym_SEMI,
  [5084] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(734), 1,
      anon_sym_LBRACE,
  [5091] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(736), 1,
      anon_sym_SEMI,
  [5098] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(738), 1,
      anon_sym_SEMI,
  [5105] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(740), 1,
      anon_sym_SEMI,
  [5112] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(742), 1,
      anon_sym_COLON,
  [5119] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(744), 1,
      anon_sym_LBRACE,
  [5126] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(746), 1,
      sym_identifier,
  [5133] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(748), 1,
      anon_sym_COLON,
  [5140] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(750), 1,
      sym_identifier,
  [5147] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(752), 1,
      sym_identifier,
  [5154] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(754), 1,
      sym_identifier,
  [5161] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(756), 1,
      anon_sym_SEMI,
  [5168] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(758), 1,
      anon_sym_SEMI,
  [5175] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(760), 1,
      anon_sym_LBRACE,
  [5182] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(762), 1,
      anon_sym_LBRACE,
  [5189] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(764), 1,
      anon_sym_SEMI,
  [5196] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(766), 1,
      ts_builtin_sym_end,
  [5203] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(768), 1,
      anon_sym_COLON,
  [5210] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(770), 1,
      anon_sym_RPAREN,
  [5217] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(772), 1,
      sym_identifier,
  [5224] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(774), 1,
      sym_identifier,
  [5231] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(776), 1,
      anon_sym_LPAREN,
  [5238] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(778), 1,
      anon_sym_LPAREN,
  [5245] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(780), 1,
      sym_identifier,
  [5252] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(782), 1,
      anon_sym_SEMI,
  [5259] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(784), 1,
      anon_sym_DQUOTE,
  [5266] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(786), 1,
      anon_sym_SEMI,
  [5273] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(788), 1,
      anon_sym_SEMI,
  [5280] = 2,
    ACTIONS(59), 1,
      sym_comment,
    ACTIONS(790), 1,
      anon_sym_SEMI,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(2)] = 0,
  [SMALL_STATE(3)] = 68,
  [SMALL_STATE(4)] = 150,
  [SMALL_STATE(5)] = 232,
  [SMALL_STATE(6)] = 314,
  [SMALL_STATE(7)] = 396,
  [SMALL_STATE(8)] = 436,
  [SMALL_STATE(9)] = 476,
  [SMALL_STATE(10)] = 546,
  [SMALL_STATE(11)] = 586,
  [SMALL_STATE(12)] = 626,
  [SMALL_STATE(13)] = 666,
  [SMALL_STATE(14)] = 706,
  [SMALL_STATE(15)] = 746,
  [SMALL_STATE(16)] = 803,
  [SMALL_STATE(17)] = 860,
  [SMALL_STATE(18)] = 917,
  [SMALL_STATE(19)] = 947,
  [SMALL_STATE(20)] = 968,
  [SMALL_STATE(21)] = 989,
  [SMALL_STATE(22)] = 1008,
  [SMALL_STATE(23)] = 1027,
  [SMALL_STATE(24)] = 1046,
  [SMALL_STATE(25)] = 1065,
  [SMALL_STATE(26)] = 1084,
  [SMALL_STATE(27)] = 1103,
  [SMALL_STATE(28)] = 1122,
  [SMALL_STATE(29)] = 1141,
  [SMALL_STATE(30)] = 1160,
  [SMALL_STATE(31)] = 1179,
  [SMALL_STATE(32)] = 1198,
  [SMALL_STATE(33)] = 1217,
  [SMALL_STATE(34)] = 1236,
  [SMALL_STATE(35)] = 1255,
  [SMALL_STATE(36)] = 1274,
  [SMALL_STATE(37)] = 1293,
  [SMALL_STATE(38)] = 1312,
  [SMALL_STATE(39)] = 1331,
  [SMALL_STATE(40)] = 1350,
  [SMALL_STATE(41)] = 1369,
  [SMALL_STATE(42)] = 1388,
  [SMALL_STATE(43)] = 1407,
  [SMALL_STATE(44)] = 1426,
  [SMALL_STATE(45)] = 1445,
  [SMALL_STATE(46)] = 1464,
  [SMALL_STATE(47)] = 1483,
  [SMALL_STATE(48)] = 1502,
  [SMALL_STATE(49)] = 1521,
  [SMALL_STATE(50)] = 1540,
  [SMALL_STATE(51)] = 1559,
  [SMALL_STATE(52)] = 1578,
  [SMALL_STATE(53)] = 1597,
  [SMALL_STATE(54)] = 1616,
  [SMALL_STATE(55)] = 1635,
  [SMALL_STATE(56)] = 1654,
  [SMALL_STATE(57)] = 1673,
  [SMALL_STATE(58)] = 1692,
  [SMALL_STATE(59)] = 1711,
  [SMALL_STATE(60)] = 1730,
  [SMALL_STATE(61)] = 1749,
  [SMALL_STATE(62)] = 1768,
  [SMALL_STATE(63)] = 1787,
  [SMALL_STATE(64)] = 1806,
  [SMALL_STATE(65)] = 1825,
  [SMALL_STATE(66)] = 1844,
  [SMALL_STATE(67)] = 1863,
  [SMALL_STATE(68)] = 1882,
  [SMALL_STATE(69)] = 1901,
  [SMALL_STATE(70)] = 1920,
  [SMALL_STATE(71)] = 1939,
  [SMALL_STATE(72)] = 1958,
  [SMALL_STATE(73)] = 1977,
  [SMALL_STATE(74)] = 1996,
  [SMALL_STATE(75)] = 2015,
  [SMALL_STATE(76)] = 2034,
  [SMALL_STATE(77)] = 2053,
  [SMALL_STATE(78)] = 2072,
  [SMALL_STATE(79)] = 2091,
  [SMALL_STATE(80)] = 2110,
  [SMALL_STATE(81)] = 2129,
  [SMALL_STATE(82)] = 2148,
  [SMALL_STATE(83)] = 2167,
  [SMALL_STATE(84)] = 2200,
  [SMALL_STATE(85)] = 2221,
  [SMALL_STATE(86)] = 2242,
  [SMALL_STATE(87)] = 2263,
  [SMALL_STATE(88)] = 2279,
  [SMALL_STATE(89)] = 2298,
  [SMALL_STATE(90)] = 2327,
  [SMALL_STATE(91)] = 2356,
  [SMALL_STATE(92)] = 2370,
  [SMALL_STATE(93)] = 2384,
  [SMALL_STATE(94)] = 2398,
  [SMALL_STATE(95)] = 2412,
  [SMALL_STATE(96)] = 2426,
  [SMALL_STATE(97)] = 2451,
  [SMALL_STATE(98)] = 2466,
  [SMALL_STATE(99)] = 2486,
  [SMALL_STATE(100)] = 2508,
  [SMALL_STATE(101)] = 2530,
  [SMALL_STATE(102)] = 2550,
  [SMALL_STATE(103)] = 2572,
  [SMALL_STATE(104)] = 2594,
  [SMALL_STATE(105)] = 2616,
  [SMALL_STATE(106)] = 2638,
  [SMALL_STATE(107)] = 2650,
  [SMALL_STATE(108)] = 2670,
  [SMALL_STATE(109)] = 2692,
  [SMALL_STATE(110)] = 2714,
  [SMALL_STATE(111)] = 2736,
  [SMALL_STATE(112)] = 2758,
  [SMALL_STATE(113)] = 2780,
  [SMALL_STATE(114)] = 2802,
  [SMALL_STATE(115)] = 2824,
  [SMALL_STATE(116)] = 2846,
  [SMALL_STATE(117)] = 2868,
  [SMALL_STATE(118)] = 2888,
  [SMALL_STATE(119)] = 2910,
  [SMALL_STATE(120)] = 2932,
  [SMALL_STATE(121)] = 2954,
  [SMALL_STATE(122)] = 2976,
  [SMALL_STATE(123)] = 2998,
  [SMALL_STATE(124)] = 3020,
  [SMALL_STATE(125)] = 3042,
  [SMALL_STATE(126)] = 3064,
  [SMALL_STATE(127)] = 3086,
  [SMALL_STATE(128)] = 3108,
  [SMALL_STATE(129)] = 3128,
  [SMALL_STATE(130)] = 3140,
  [SMALL_STATE(131)] = 3152,
  [SMALL_STATE(132)] = 3164,
  [SMALL_STATE(133)] = 3183,
  [SMALL_STATE(134)] = 3202,
  [SMALL_STATE(135)] = 3219,
  [SMALL_STATE(136)] = 3236,
  [SMALL_STATE(137)] = 3255,
  [SMALL_STATE(138)] = 3272,
  [SMALL_STATE(139)] = 3283,
  [SMALL_STATE(140)] = 3302,
  [SMALL_STATE(141)] = 3321,
  [SMALL_STATE(142)] = 3332,
  [SMALL_STATE(143)] = 3343,
  [SMALL_STATE(144)] = 3354,
  [SMALL_STATE(145)] = 3371,
  [SMALL_STATE(146)] = 3382,
  [SMALL_STATE(147)] = 3401,
  [SMALL_STATE(148)] = 3420,
  [SMALL_STATE(149)] = 3439,
  [SMALL_STATE(150)] = 3450,
  [SMALL_STATE(151)] = 3469,
  [SMALL_STATE(152)] = 3488,
  [SMALL_STATE(153)] = 3507,
  [SMALL_STATE(154)] = 3526,
  [SMALL_STATE(155)] = 3537,
  [SMALL_STATE(156)] = 3548,
  [SMALL_STATE(157)] = 3559,
  [SMALL_STATE(158)] = 3570,
  [SMALL_STATE(159)] = 3581,
  [SMALL_STATE(160)] = 3600,
  [SMALL_STATE(161)] = 3611,
  [SMALL_STATE(162)] = 3630,
  [SMALL_STATE(163)] = 3649,
  [SMALL_STATE(164)] = 3663,
  [SMALL_STATE(165)] = 3673,
  [SMALL_STATE(166)] = 3683,
  [SMALL_STATE(167)] = 3699,
  [SMALL_STATE(168)] = 3715,
  [SMALL_STATE(169)] = 3731,
  [SMALL_STATE(170)] = 3747,
  [SMALL_STATE(171)] = 3763,
  [SMALL_STATE(172)] = 3779,
  [SMALL_STATE(173)] = 3793,
  [SMALL_STATE(174)] = 3803,
  [SMALL_STATE(175)] = 3819,
  [SMALL_STATE(176)] = 3835,
  [SMALL_STATE(177)] = 3849,
  [SMALL_STATE(178)] = 3859,
  [SMALL_STATE(179)] = 3869,
  [SMALL_STATE(180)] = 3883,
  [SMALL_STATE(181)] = 3897,
  [SMALL_STATE(182)] = 3913,
  [SMALL_STATE(183)] = 3927,
  [SMALL_STATE(184)] = 3943,
  [SMALL_STATE(185)] = 3959,
  [SMALL_STATE(186)] = 3969,
  [SMALL_STATE(187)] = 3979,
  [SMALL_STATE(188)] = 3989,
  [SMALL_STATE(189)] = 3999,
  [SMALL_STATE(190)] = 4009,
  [SMALL_STATE(191)] = 4022,
  [SMALL_STATE(192)] = 4035,
  [SMALL_STATE(193)] = 4048,
  [SMALL_STATE(194)] = 4061,
  [SMALL_STATE(195)] = 4070,
  [SMALL_STATE(196)] = 4083,
  [SMALL_STATE(197)] = 4096,
  [SMALL_STATE(198)] = 4109,
  [SMALL_STATE(199)] = 4122,
  [SMALL_STATE(200)] = 4135,
  [SMALL_STATE(201)] = 4148,
  [SMALL_STATE(202)] = 4161,
  [SMALL_STATE(203)] = 4174,
  [SMALL_STATE(204)] = 4187,
  [SMALL_STATE(205)] = 4200,
  [SMALL_STATE(206)] = 4213,
  [SMALL_STATE(207)] = 4226,
  [SMALL_STATE(208)] = 4239,
  [SMALL_STATE(209)] = 4252,
  [SMALL_STATE(210)] = 4265,
  [SMALL_STATE(211)] = 4274,
  [SMALL_STATE(212)] = 4283,
  [SMALL_STATE(213)] = 4296,
  [SMALL_STATE(214)] = 4309,
  [SMALL_STATE(215)] = 4322,
  [SMALL_STATE(216)] = 4335,
  [SMALL_STATE(217)] = 4348,
  [SMALL_STATE(218)] = 4361,
  [SMALL_STATE(219)] = 4374,
  [SMALL_STATE(220)] = 4387,
  [SMALL_STATE(221)] = 4400,
  [SMALL_STATE(222)] = 4413,
  [SMALL_STATE(223)] = 4426,
  [SMALL_STATE(224)] = 4435,
  [SMALL_STATE(225)] = 4448,
  [SMALL_STATE(226)] = 4457,
  [SMALL_STATE(227)] = 4470,
  [SMALL_STATE(228)] = 4483,
  [SMALL_STATE(229)] = 4496,
  [SMALL_STATE(230)] = 4509,
  [SMALL_STATE(231)] = 4522,
  [SMALL_STATE(232)] = 4535,
  [SMALL_STATE(233)] = 4548,
  [SMALL_STATE(234)] = 4561,
  [SMALL_STATE(235)] = 4574,
  [SMALL_STATE(236)] = 4583,
  [SMALL_STATE(237)] = 4592,
  [SMALL_STATE(238)] = 4601,
  [SMALL_STATE(239)] = 4614,
  [SMALL_STATE(240)] = 4627,
  [SMALL_STATE(241)] = 4636,
  [SMALL_STATE(242)] = 4649,
  [SMALL_STATE(243)] = 4658,
  [SMALL_STATE(244)] = 4667,
  [SMALL_STATE(245)] = 4676,
  [SMALL_STATE(246)] = 4689,
  [SMALL_STATE(247)] = 4698,
  [SMALL_STATE(248)] = 4711,
  [SMALL_STATE(249)] = 4724,
  [SMALL_STATE(250)] = 4735,
  [SMALL_STATE(251)] = 4748,
  [SMALL_STATE(252)] = 4761,
  [SMALL_STATE(253)] = 4769,
  [SMALL_STATE(254)] = 4777,
  [SMALL_STATE(255)] = 4785,
  [SMALL_STATE(256)] = 4795,
  [SMALL_STATE(257)] = 4803,
  [SMALL_STATE(258)] = 4811,
  [SMALL_STATE(259)] = 4819,
  [SMALL_STATE(260)] = 4829,
  [SMALL_STATE(261)] = 4839,
  [SMALL_STATE(262)] = 4847,
  [SMALL_STATE(263)] = 4855,
  [SMALL_STATE(264)] = 4863,
  [SMALL_STATE(265)] = 4873,
  [SMALL_STATE(266)] = 4881,
  [SMALL_STATE(267)] = 4889,
  [SMALL_STATE(268)] = 4897,
  [SMALL_STATE(269)] = 4905,
  [SMALL_STATE(270)] = 4915,
  [SMALL_STATE(271)] = 4923,
  [SMALL_STATE(272)] = 4930,
  [SMALL_STATE(273)] = 4937,
  [SMALL_STATE(274)] = 4944,
  [SMALL_STATE(275)] = 4951,
  [SMALL_STATE(276)] = 4958,
  [SMALL_STATE(277)] = 4965,
  [SMALL_STATE(278)] = 4972,
  [SMALL_STATE(279)] = 4979,
  [SMALL_STATE(280)] = 4986,
  [SMALL_STATE(281)] = 4993,
  [SMALL_STATE(282)] = 5000,
  [SMALL_STATE(283)] = 5007,
  [SMALL_STATE(284)] = 5014,
  [SMALL_STATE(285)] = 5021,
  [SMALL_STATE(286)] = 5028,
  [SMALL_STATE(287)] = 5035,
  [SMALL_STATE(288)] = 5042,
  [SMALL_STATE(289)] = 5049,
  [SMALL_STATE(290)] = 5056,
  [SMALL_STATE(291)] = 5063,
  [SMALL_STATE(292)] = 5070,
  [SMALL_STATE(293)] = 5077,
  [SMALL_STATE(294)] = 5084,
  [SMALL_STATE(295)] = 5091,
  [SMALL_STATE(296)] = 5098,
  [SMALL_STATE(297)] = 5105,
  [SMALL_STATE(298)] = 5112,
  [SMALL_STATE(299)] = 5119,
  [SMALL_STATE(300)] = 5126,
  [SMALL_STATE(301)] = 5133,
  [SMALL_STATE(302)] = 5140,
  [SMALL_STATE(303)] = 5147,
  [SMALL_STATE(304)] = 5154,
  [SMALL_STATE(305)] = 5161,
  [SMALL_STATE(306)] = 5168,
  [SMALL_STATE(307)] = 5175,
  [SMALL_STATE(308)] = 5182,
  [SMALL_STATE(309)] = 5189,
  [SMALL_STATE(310)] = 5196,
  [SMALL_STATE(311)] = 5203,
  [SMALL_STATE(312)] = 5210,
  [SMALL_STATE(313)] = 5217,
  [SMALL_STATE(314)] = 5224,
  [SMALL_STATE(315)] = 5231,
  [SMALL_STATE(316)] = 5238,
  [SMALL_STATE(317)] = 5245,
  [SMALL_STATE(318)] = 5252,
  [SMALL_STATE(319)] = 5259,
  [SMALL_STATE(320)] = 5266,
  [SMALL_STATE(321)] = 5273,
  [SMALL_STATE(322)] = 5280,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = false}}, SHIFT_EXTRA(),
  [5] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 0, 0, 0),
  [7] = {.entry = {.count = 1, .reusable = true}}, SHIFT(91),
  [9] = {.entry = {.count = 1, .reusable = true}}, SHIFT(216),
  [11] = {.entry = {.count = 1, .reusable = true}}, SHIFT(195),
  [13] = {.entry = {.count = 1, .reusable = true}}, SHIFT(260),
  [15] = {.entry = {.count = 1, .reusable = true}}, SHIFT(264),
  [17] = {.entry = {.count = 1, .reusable = true}}, SHIFT(302),
  [19] = {.entry = {.count = 1, .reusable = true}}, SHIFT(132),
  [21] = {.entry = {.count = 1, .reusable = true}}, SHIFT(300),
  [23] = {.entry = {.count = 1, .reusable = true}}, SHIFT(284),
  [25] = {.entry = {.count = 1, .reusable = true}}, SHIFT(286),
  [27] = {.entry = {.count = 1, .reusable = true}}, SHIFT(206),
  [29] = {.entry = {.count = 1, .reusable = true}}, SHIFT(207),
  [31] = {.entry = {.count = 1, .reusable = true}}, SHIFT(271),
  [33] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1, 0, 0),
  [35] = {.entry = {.count = 1, .reusable = true}}, SHIFT(171),
  [37] = {.entry = {.count = 1, .reusable = true}}, SHIFT(161),
  [39] = {.entry = {.count = 1, .reusable = true}}, SHIFT(5),
  [41] = {.entry = {.count = 1, .reusable = false}}, SHIFT(86),
  [43] = {.entry = {.count = 1, .reusable = true}}, SHIFT(97),
  [45] = {.entry = {.count = 1, .reusable = false}}, SHIFT(149),
  [47] = {.entry = {.count = 1, .reusable = false}}, SHIFT(129),
  [49] = {.entry = {.count = 1, .reusable = true}}, SHIFT(130),
  [51] = {.entry = {.count = 1, .reusable = true}}, SHIFT(154),
  [53] = {.entry = {.count = 1, .reusable = false}}, SHIFT(155),
  [55] = {.entry = {.count = 1, .reusable = false}}, SHIFT(154),
  [57] = {.entry = {.count = 1, .reusable = true}}, SHIFT(138),
  [59] = {.entry = {.count = 1, .reusable = true}}, SHIFT_EXTRA(),
  [61] = {.entry = {.count = 1, .reusable = true}}, SHIFT(8),
  [63] = {.entry = {.count = 1, .reusable = false}}, SHIFT(93),
  [65] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 2, 0, 0),
  [67] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat2, 2, 0, 0),
  [69] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat2, 2, 0, 0), SHIFT_REPEAT(91),
  [72] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat2, 2, 0, 0), SHIFT_REPEAT(260),
  [75] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat2, 2, 0, 0), SHIFT_REPEAT(264),
  [78] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat2, 2, 0, 0), SHIFT_REPEAT(302),
  [81] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat2, 2, 0, 0), SHIFT_REPEAT(132),
  [84] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat2, 2, 0, 0), SHIFT_REPEAT(300),
  [87] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat2, 2, 0, 0), SHIFT_REPEAT(284),
  [90] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat2, 2, 0, 0), SHIFT_REPEAT(286),
  [93] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat2, 2, 0, 0), SHIFT_REPEAT(206),
  [96] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat2, 2, 0, 0), SHIFT_REPEAT(207),
  [99] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat2, 2, 0, 0), SHIFT_REPEAT(271),
  [102] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0),
  [104] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(216),
  [107] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(195),
  [110] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_include, 3, 0, 3),
  [112] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_native_include, 3, 0, 3),
  [114] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_object, 2, 0, 0),
  [116] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_decl, 11, 0, 86),
  [118] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_root_decl, 3, 0, 7),
  [120] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_file_extension_decl, 3, 0, 8),
  [122] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_file_identifier_decl, 3, 0, 9),
  [124] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_type_decl, 4, 0, 10),
  [126] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_object, 4, 0, 13),
  [128] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_decl, 4, 0, 15),
  [130] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_rpc_decl, 4, 0, 17),
  [132] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_attribute_decl, 5, 0, 18),
  [134] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_type_decl, 5, 0, 19),
  [136] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_type_decl, 5, 0, 22),
  [138] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_decl, 5, 0, 23),
  [140] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_decl, 5, 0, 26),
  [142] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_rpc_decl, 5, 0, 27),
  [144] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_type_decl, 5, 0, 28),
  [146] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_decl, 5, 0, 29),
  [148] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_rpc_decl, 5, 0, 30),
  [150] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_type_decl, 6, 0, 36),
  [152] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_decl, 6, 0, 23),
  [154] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_decl, 6, 0, 40),
  [156] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_namespace_decl, 3, 0, 4),
  [158] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_decl, 6, 0, 44),
  [160] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_type_decl, 6, 0, 46),
  [162] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_type_decl, 6, 0, 45),
  [164] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_decl, 6, 0, 47),
  [166] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_decl, 6, 0, 48),
  [168] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_rpc_decl, 6, 0, 49),
  [170] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_attribute_decl, 3, 0, 5),
  [172] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_decl, 7, 0, 52),
  [174] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_decl, 7, 0, 40),
  [176] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_decl, 7, 0, 44),
  [178] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_decl, 7, 0, 56),
  [180] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_type_decl, 7, 0, 57),
  [182] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_decl, 7, 0, 47),
  [184] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_decl, 7, 0, 58),
  [186] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_decl, 7, 0, 59),
  [188] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_object, 3, 0, 6),
  [190] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_decl, 8, 0, 52),
  [192] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_decl, 8, 0, 67),
  [194] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_decl, 8, 0, 70),
  [196] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_decl, 8, 0, 56),
  [198] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_decl, 8, 0, 72),
  [200] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_decl, 8, 0, 58),
  [202] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_decl, 8, 0, 59),
  [204] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_decl, 8, 0, 73),
  [206] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_decl, 9, 0, 67),
  [208] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_decl, 9, 0, 70),
  [210] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_decl, 9, 0, 78),
  [212] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_decl, 9, 0, 72),
  [214] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_decl, 9, 0, 79),
  [216] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_decl, 9, 0, 80),
  [218] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_decl, 9, 0, 73),
  [220] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_decl, 10, 0, 78),
  [222] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_decl, 10, 0, 79),
  [224] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_decl, 10, 0, 80),
  [226] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_decl, 10, 0, 86),
  [228] = {.entry = {.count = 1, .reusable = true}}, SHIFT(155),
  [230] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_full_ident_repeat1, 2, 0, 0),
  [232] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_full_ident_repeat1, 2, 0, 0), SHIFT_REPEAT(292),
  [235] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_full_ident, 2, 0, 0),
  [237] = {.entry = {.count = 1, .reusable = true}}, SHIFT(292),
  [239] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_full_ident, 1, 0, 0),
  [241] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_type_decl_repeat1, 2, 0, 2), SHIFT_REPEAT(91),
  [244] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_type_decl_repeat1, 2, 0, 2),
  [246] = {.entry = {.count = 1, .reusable = true}}, SHIFT(129),
  [248] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_type_decl_repeat1, 1, 0, 1),
  [250] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_type, 5, 0, 64),
  [252] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_type, 1, 0, 0),
  [254] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_type, 3, 0, 37),
  [256] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_type, 1, 0, 14),
  [258] = {.entry = {.count = 1, .reusable = true}}, SHIFT(303),
  [260] = {.entry = {.count = 1, .reusable = true}}, SHIFT(281),
  [262] = {.entry = {.count = 1, .reusable = true}}, SHIFT(313),
  [264] = {.entry = {.count = 1, .reusable = true}}, SHIFT(317),
  [266] = {.entry = {.count = 1, .reusable = true}}, SHIFT(287),
  [268] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_plus_minus_constant, 1, 0, 0),
  [270] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_plus_minus_constant, 1, 0, 0),
  [272] = {.entry = {.count = 1, .reusable = true}}, SHIFT(268),
  [274] = {.entry = {.count = 1, .reusable = true}}, SHIFT(29),
  [276] = {.entry = {.count = 1, .reusable = true}}, SHIFT(315),
  [278] = {.entry = {.count = 1, .reusable = true}}, SHIFT(56),
  [280] = {.entry = {.count = 1, .reusable = true}}, SHIFT(301),
  [282] = {.entry = {.count = 1, .reusable = true}}, SHIFT(38),
  [284] = {.entry = {.count = 1, .reusable = true}}, SHIFT(86),
  [286] = {.entry = {.count = 1, .reusable = true}}, SHIFT(39),
  [288] = {.entry = {.count = 1, .reusable = true}}, SHIFT(57),
  [290] = {.entry = {.count = 1, .reusable = true}}, SHIFT(58),
  [292] = {.entry = {.count = 1, .reusable = true}}, SHIFT(28),
  [294] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_type_decl_repeat2, 2, 0, 20), SHIFT_REPEAT(268),
  [297] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_type_decl_repeat2, 2, 0, 20),
  [299] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_type_decl_repeat2, 2, 0, 20), SHIFT_REPEAT(301),
  [302] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_int_constant, 2, 0, 0),
  [304] = {.entry = {.count = 1, .reusable = true}}, SHIFT(35),
  [306] = {.entry = {.count = 1, .reusable = true}}, SHIFT(31),
  [308] = {.entry = {.count = 1, .reusable = true}}, SHIFT(40),
  [310] = {.entry = {.count = 1, .reusable = true}}, SHIFT(32),
  [312] = {.entry = {.count = 1, .reusable = true}}, SHIFT(65),
  [314] = {.entry = {.count = 1, .reusable = true}}, SHIFT(41),
  [316] = {.entry = {.count = 1, .reusable = true}}, SHIFT(53),
  [318] = {.entry = {.count = 1, .reusable = true}}, SHIFT(67),
  [320] = {.entry = {.count = 1, .reusable = true}}, SHIFT(68),
  [322] = {.entry = {.count = 1, .reusable = true}}, SHIFT(26),
  [324] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_rpc_decl_repeat1, 2, 0, 0), SHIFT_REPEAT(268),
  [327] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_rpc_decl_repeat1, 2, 0, 0),
  [329] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_rpc_decl_repeat1, 2, 0, 0), SHIFT_REPEAT(315),
  [332] = {.entry = {.count = 1, .reusable = true}}, SHIFT(34),
  [334] = {.entry = {.count = 1, .reusable = true}}, SHIFT(36),
  [336] = {.entry = {.count = 1, .reusable = true}}, SHIFT(82),
  [338] = {.entry = {.count = 1, .reusable = true}}, SHIFT(37),
  [340] = {.entry = {.count = 1, .reusable = true}}, SHIFT(45),
  [342] = {.entry = {.count = 1, .reusable = true}}, SHIFT(54),
  [344] = {.entry = {.count = 1, .reusable = true}}, SHIFT(46),
  [346] = {.entry = {.count = 1, .reusable = true}}, SHIFT(47),
  [348] = {.entry = {.count = 1, .reusable = true}}, SHIFT(49),
  [350] = {.entry = {.count = 1, .reusable = true}}, SHIFT(76),
  [352] = {.entry = {.count = 1, .reusable = true}}, SHIFT(50),
  [354] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_decimal_lit, 1, 0, 0),
  [356] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_int_lit, 1, 0, 0),
  [358] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_int_constant, 1, 0, 0),
  [360] = {.entry = {.count = 1, .reusable = true}}, SHIFT(21),
  [362] = {.entry = {.count = 1, .reusable = true}}, SHIFT(180),
  [364] = {.entry = {.count = 1, .reusable = true}}, SHIFT(70),
  [366] = {.entry = {.count = 1, .reusable = true}}, SHIFT(137),
  [368] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_field_decl, 2, 0, 25),
  [370] = {.entry = {.count = 1, .reusable = true}}, SHIFT(13),
  [372] = {.entry = {.count = 1, .reusable = true}}, SHIFT(259),
  [374] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_val_decl, 2, 0, 53),
  [376] = {.entry = {.count = 1, .reusable = true}}, SHIFT(89),
  [378] = {.entry = {.count = 1, .reusable = true}}, SHIFT(71),
  [380] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_val_decl, 1, 0, 38),
  [382] = {.entry = {.count = 1, .reusable = true}}, SHIFT(90),
  [384] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string_constant, 1, 0, 0),
  [386] = {.entry = {.count = 1, .reusable = true}}, SHIFT(73),
  [388] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string_constant, 3, 0, 0),
  [390] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_float_constant, 2, 0, 0),
  [392] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_single_value, 1, 0, 33),
  [394] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_field_decl, 1, 0, 16),
  [396] = {.entry = {.count = 1, .reusable = true}}, SHIFT(14),
  [398] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string_constant, 2, 0, 0),
  [400] = {.entry = {.count = 1, .reusable = true}}, SHIFT(61),
  [402] = {.entry = {.count = 1, .reusable = true}}, SHIFT(78),
  [404] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_bool_constant, 1, 0, 0),
  [406] = {.entry = {.count = 1, .reusable = true}}, SHIFT(79),
  [408] = {.entry = {.count = 1, .reusable = true}}, SHIFT(80),
  [410] = {.entry = {.count = 1, .reusable = true}}, SHIFT(62),
  [412] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_float_lit, 1, 0, 0),
  [414] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_inf_token, 1, 0, 0),
  [416] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_single_value, 1, 0, 14),
  [418] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_single_value, 1, 0, 32),
  [420] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_scalar, 1, 0, 0),
  [422] = {.entry = {.count = 1, .reusable = true}}, SHIFT(22),
  [424] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_float_constant, 1, 0, 0),
  [426] = {.entry = {.count = 1, .reusable = true}}, SHIFT(187),
  [428] = {.entry = {.count = 1, .reusable = true}}, SHIFT(188),
  [430] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_field_decl, 3, 0, 42),
  [432] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_metadata, 4, 0, 34),
  [434] = {.entry = {.count = 1, .reusable = false}}, SHIFT(141),
  [436] = {.entry = {.count = 1, .reusable = false}}, SHIFT(184),
  [438] = {.entry = {.count = 1, .reusable = true}}, SHIFT(184),
  [440] = {.entry = {.count = 1, .reusable = true}}, SHIFT(211),
  [442] = {.entry = {.count = 1, .reusable = true}}, SHIFT(6),
  [444] = {.entry = {.count = 1, .reusable = false}}, SHIFT(145),
  [446] = {.entry = {.count = 1, .reusable = false}}, SHIFT(169),
  [448] = {.entry = {.count = 1, .reusable = true}}, SHIFT(169),
  [450] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_field_decl, 4, 0, 55),
  [452] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_metadata, 3, 0, 21),
  [454] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_object_repeat1, 2, 0, 6),
  [456] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_value, 1, 0, 60),
  [458] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_value, 1, 0, 61),
  [460] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_val_decl, 3, 0, 65),
  [462] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_object_repeat1, 1, 0, 0),
  [464] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_val_decl, 4, 0, 77),
  [466] = {.entry = {.count = 1, .reusable = true}}, SHIFT(194),
  [468] = {.entry = {.count = 1, .reusable = true}}, SHIFT(4),
  [470] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_string_constant_repeat1, 2, 0, 0),
  [472] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_string_constant_repeat1, 2, 0, 0), SHIFT_REPEAT(184),
  [475] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_string_constant_repeat1, 2, 0, 0), SHIFT_REPEAT(184),
  [478] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_value, 3, 0, 81),
  [480] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_value, 4, 0, 88),
  [482] = {.entry = {.count = 1, .reusable = true}}, SHIFT(52),
  [484] = {.entry = {.count = 1, .reusable = true}}, SHIFT(153),
  [486] = {.entry = {.count = 1, .reusable = true}}, SHIFT(164),
  [488] = {.entry = {.count = 1, .reusable = true}}, SHIFT(59),
  [490] = {.entry = {.count = 1, .reusable = true}}, SHIFT(114),
  [492] = {.entry = {.count = 1, .reusable = true}}, SHIFT(60),
  [494] = {.entry = {.count = 1, .reusable = true}}, SHIFT(115),
  [496] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_field_decl, 4, 0, 50),
  [498] = {.entry = {.count = 1, .reusable = true}}, SHIFT(63),
  [500] = {.entry = {.count = 1, .reusable = true}}, SHIFT(133),
  [502] = {.entry = {.count = 1, .reusable = true}}, SHIFT(269),
  [504] = {.entry = {.count = 1, .reusable = true}}, SHIFT(173),
  [506] = {.entry = {.count = 1, .reusable = true}}, SHIFT(64),
  [508] = {.entry = {.count = 1, .reusable = true}}, SHIFT(136),
  [510] = {.entry = {.count = 1, .reusable = true}}, SHIFT(135),
  [512] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_metadata_repeat1, 2, 0, 35), SHIFT_REPEAT(269),
  [515] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_metadata_repeat1, 2, 0, 35),
  [517] = {.entry = {.count = 1, .reusable = true}}, SHIFT(165),
  [519] = {.entry = {.count = 1, .reusable = true}}, SHIFT(66),
  [521] = {.entry = {.count = 1, .reusable = true}}, SHIFT(139),
  [523] = {.entry = {.count = 1, .reusable = true}}, SHIFT(175),
  [525] = {.entry = {.count = 1, .reusable = true}}, SHIFT(42),
  [527] = {.entry = {.count = 1, .reusable = true}}, SHIFT(113),
  [529] = {.entry = {.count = 1, .reusable = true}}, SHIFT(69),
  [531] = {.entry = {.count = 1, .reusable = true}}, SHIFT(127),
  [533] = {.entry = {.count = 1, .reusable = true}}, SHIFT(223),
  [535] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_union_decl_repeat1, 2, 0, 41),
  [537] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_union_decl_repeat1, 2, 0, 41), SHIFT_REPEAT(140),
  [540] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_field_decl, 5, 0, 62),
  [542] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_field_decl, 5, 0, 63),
  [544] = {.entry = {.count = 1, .reusable = true}}, SHIFT(33),
  [546] = {.entry = {.count = 1, .reusable = true}}, SHIFT(112),
  [548] = {.entry = {.count = 1, .reusable = true}}, SHIFT(27),
  [550] = {.entry = {.count = 1, .reusable = true}}, SHIFT(44),
  [552] = {.entry = {.count = 1, .reusable = true}}, SHIFT(123),
  [554] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_enum_decl_repeat1, 2, 0, 68),
  [556] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_enum_decl_repeat1, 2, 0, 68), SHIFT_REPEAT(181),
  [559] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_object_repeat1, 2, 0, 2),
  [561] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_object_repeat1, 2, 0, 2), SHIFT_REPEAT(180),
  [564] = {.entry = {.count = 1, .reusable = true}}, SHIFT(72),
  [566] = {.entry = {.count = 1, .reusable = true}}, SHIFT(147),
  [568] = {.entry = {.count = 1, .reusable = true}}, SHIFT(74),
  [570] = {.entry = {.count = 1, .reusable = true}}, SHIFT(150),
  [572] = {.entry = {.count = 1, .reusable = true}}, SHIFT(116),
  [574] = {.entry = {.count = 1, .reusable = true}}, SHIFT(168),
  [576] = {.entry = {.count = 1, .reusable = true}}, SHIFT(3),
  [578] = {.entry = {.count = 1, .reusable = true}}, SHIFT(185),
  [580] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_field_decl, 6, 0, 74),
  [582] = {.entry = {.count = 1, .reusable = true}}, SHIFT(236),
  [584] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_field_decl, 6, 0, 75),
  [586] = {.entry = {.count = 1, .reusable = true}}, SHIFT(48),
  [588] = {.entry = {.count = 1, .reusable = true}}, SHIFT(103),
  [590] = {.entry = {.count = 1, .reusable = true}}, SHIFT(119),
  [592] = {.entry = {.count = 1, .reusable = true}}, SHIFT(121),
  [594] = {.entry = {.count = 1, .reusable = true}}, SHIFT(237),
  [596] = {.entry = {.count = 1, .reusable = true}}, SHIFT(100),
  [598] = {.entry = {.count = 1, .reusable = true}}, SHIFT(55),
  [600] = {.entry = {.count = 1, .reusable = true}}, SHIFT(111),
  [602] = {.entry = {.count = 1, .reusable = true}}, SHIFT(81),
  [604] = {.entry = {.count = 1, .reusable = true}}, SHIFT(159),
  [606] = {.entry = {.count = 1, .reusable = true}}, SHIFT(77),
  [608] = {.entry = {.count = 1, .reusable = true}}, SHIFT(186),
  [610] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_field_decl, 7, 0, 82),
  [612] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_field_decl, 7, 0, 83),
  [614] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_rpc_method, 7, 0, 85),
  [616] = {.entry = {.count = 1, .reusable = true}}, SHIFT(244),
  [618] = {.entry = {.count = 1, .reusable = true}}, SHIFT(104),
  [620] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_type_decl_repeat2, 1, 0, 11),
  [622] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_value_repeat1, 2, 0, 89), SHIFT_REPEAT(3),
  [625] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_value_repeat1, 2, 0, 89),
  [627] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_field_decl, 8, 0, 90),
  [629] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_rpc_method, 8, 0, 91),
  [631] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_rpc_method, 8, 0, 92),
  [633] = {.entry = {.count = 1, .reusable = true}}, SHIFT(298),
  [635] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_rpc_method, 9, 0, 93),
  [637] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_type_decl_repeat1, 2, 0, 2), SHIFT_REPEAT(268),
  [640] = {.entry = {.count = 1, .reusable = true}}, SHIFT(316),
  [642] = {.entry = {.count = 1, .reusable = true}}, SHIFT(9),
  [644] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_field_and_value, 1, 0, 12),
  [646] = {.entry = {.count = 1, .reusable = true}}, SHIFT(189),
  [648] = {.entry = {.count = 1, .reusable = true}}, SHIFT(75),
  [650] = {.entry = {.count = 1, .reusable = true}}, SHIFT(152),
  [652] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_field_decl, 4, 0, 54),
  [654] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_field_decl, 5, 0, 71),
  [656] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_metadata_repeat1, 2, 0, 21),
  [658] = {.entry = {.count = 1, .reusable = true}}, SHIFT(148),
  [660] = {.entry = {.count = 1, .reusable = true}}, SHIFT(94),
  [662] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_val_decl, 4, 0, 76),
  [664] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_field_decl, 3, 0, 43),
  [666] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_union_decl_repeat1, 2, 0, 39),
  [668] = {.entry = {.count = 1, .reusable = true}}, SHIFT(249),
  [670] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_val_decl, 5, 0, 84),
  [672] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_val_decl, 2, 0, 51),
  [674] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_field_and_value, 3, 0, 31),
  [676] = {.entry = {.count = 1, .reusable = true}}, SHIFT(272),
  [678] = {.entry = {.count = 1, .reusable = true}}, SHIFT(290),
  [680] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_union_field_decl, 2, 0, 24),
  [682] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_value_repeat1, 2, 0, 87),
  [684] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_enum_decl_repeat1, 2, 0, 66),
  [686] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_enum_val_decl, 3, 0, 69),
  [688] = {.entry = {.count = 1, .reusable = true}}, SHIFT(294),
  [690] = {.entry = {.count = 1, .reusable = true}}, SHIFT(319),
  [692] = {.entry = {.count = 1, .reusable = true}}, SHIFT(126),
  [694] = {.entry = {.count = 1, .reusable = true}}, SHIFT(229),
  [696] = {.entry = {.count = 1, .reusable = true}}, SHIFT(24),
  [698] = {.entry = {.count = 1, .reusable = true}}, SHIFT(210),
  [700] = {.entry = {.count = 1, .reusable = true}}, SHIFT(280),
  [702] = {.entry = {.count = 1, .reusable = true}}, SHIFT(10),
  [704] = {.entry = {.count = 1, .reusable = true}}, SHIFT(166),
  [706] = {.entry = {.count = 1, .reusable = true}}, SHIFT(238),
  [708] = {.entry = {.count = 1, .reusable = true}}, SHIFT(228),
  [710] = {.entry = {.count = 1, .reusable = true}}, SHIFT(118),
  [712] = {.entry = {.count = 1, .reusable = true}}, SHIFT(101),
  [714] = {.entry = {.count = 1, .reusable = true}}, SHIFT(239),
  [716] = {.entry = {.count = 1, .reusable = true}}, SHIFT(110),
  [718] = {.entry = {.count = 1, .reusable = true}}, SHIFT(318),
  [720] = {.entry = {.count = 1, .reusable = true}}, SHIFT(283),
  [722] = {.entry = {.count = 1, .reusable = true}}, SHIFT(7),
  [724] = {.entry = {.count = 1, .reusable = true}}, SHIFT(311),
  [726] = {.entry = {.count = 1, .reusable = true}}, SHIFT(51),
  [728] = {.entry = {.count = 1, .reusable = true}}, SHIFT(92),
  [730] = {.entry = {.count = 1, .reusable = true}}, SHIFT(87),
  [732] = {.entry = {.count = 1, .reusable = true}}, SHIFT(242),
  [734] = {.entry = {.count = 1, .reusable = true}}, SHIFT(98),
  [736] = {.entry = {.count = 1, .reusable = true}}, SHIFT(43),
  [738] = {.entry = {.count = 1, .reusable = true}}, SHIFT(25),
  [740] = {.entry = {.count = 1, .reusable = true}}, SHIFT(243),
  [742] = {.entry = {.count = 1, .reusable = true}}, SHIFT(12),
  [744] = {.entry = {.count = 1, .reusable = true}}, SHIFT(174),
  [746] = {.entry = {.count = 1, .reusable = true}}, SHIFT(288),
  [748] = {.entry = {.count = 1, .reusable = true}}, SHIFT(11),
  [750] = {.entry = {.count = 1, .reusable = true}}, SHIFT(220),
  [752] = {.entry = {.count = 1, .reusable = true}}, SHIFT(227),
  [754] = {.entry = {.count = 1, .reusable = true}}, SHIFT(312),
  [756] = {.entry = {.count = 1, .reusable = true}}, SHIFT(225),
  [758] = {.entry = {.count = 1, .reusable = true}}, SHIFT(20),
  [760] = {.entry = {.count = 1, .reusable = true}}, SHIFT(125),
  [762] = {.entry = {.count = 1, .reusable = true}}, SHIFT(122),
  [764] = {.entry = {.count = 1, .reusable = true}}, SHIFT(246),
  [766] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [768] = {.entry = {.count = 1, .reusable = true}}, SHIFT(274),
  [770] = {.entry = {.count = 1, .reusable = true}}, SHIFT(277),
  [772] = {.entry = {.count = 1, .reusable = true}}, SHIFT(278),
  [774] = {.entry = {.count = 1, .reusable = true}}, SHIFT(289),
  [776] = {.entry = {.count = 1, .reusable = true}}, SHIFT(314),
  [778] = {.entry = {.count = 1, .reusable = true}}, SHIFT(304),
  [780] = {.entry = {.count = 1, .reusable = true}}, SHIFT(230),
  [782] = {.entry = {.count = 1, .reusable = true}}, SHIFT(23),
  [784] = {.entry = {.count = 1, .reusable = true}}, SHIFT(322),
  [786] = {.entry = {.count = 1, .reusable = true}}, SHIFT(235),
  [788] = {.entry = {.count = 1, .reusable = true}}, SHIFT(19),
  [790] = {.entry = {.count = 1, .reusable = true}}, SHIFT(30),
};

#ifdef __cplusplus
extern "C" {
#endif
#ifdef TREE_SITTER_HIDE_SYMBOLS
#define TS_PUBLIC
#elif defined(_WIN32)
#define TS_PUBLIC __declspec(dllexport)
#else
#define TS_PUBLIC __attribute__((visibility("default")))
#endif

TS_PUBLIC const TSLanguage *tree_sitter_flatbuffers(void) {
  static const TSLanguage language = {
    .version = LANGUAGE_VERSION,
    .symbol_count = SYMBOL_COUNT,
    .alias_count = ALIAS_COUNT,
    .token_count = TOKEN_COUNT,
    .external_token_count = EXTERNAL_TOKEN_COUNT,
    .state_count = STATE_COUNT,
    .large_state_count = LARGE_STATE_COUNT,
    .production_id_count = PRODUCTION_ID_COUNT,
    .field_count = FIELD_COUNT,
    .max_alias_sequence_length = MAX_ALIAS_SEQUENCE_LENGTH,
    .parse_table = &ts_parse_table[0][0],
    .small_parse_table = ts_small_parse_table,
    .small_parse_table_map = ts_small_parse_table_map,
    .parse_actions = ts_parse_actions,
    .symbol_names = ts_symbol_names,
    .field_names = ts_field_names,
    .field_map_slices = ts_field_map_slices,
    .field_map_entries = ts_field_map_entries,
    .symbol_metadata = ts_symbol_metadata,
    .public_symbol_map = ts_symbol_map,
    .alias_map = ts_non_terminal_alias_map,
    .alias_sequences = &ts_alias_sequences[0][0],
    .lex_modes = ts_lex_modes,
    .lex_fn = ts_lex,
    .primary_state_ids = ts_primary_state_ids,
  };
  return &language;
}
#ifdef __cplusplus
}
#endif
