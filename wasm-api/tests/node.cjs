const assert = require("node:assert/strict");
const wasm = require("../pkg/flatc_rs_wasm.js");

const singleSchema = "table Monster { hp:int; } root_type Monster;";
const request = {
  entryPath: "schemas/main.fbs",
  files: [
    {
      path: "schemas/main.fbs",
      source: 'include "model/monster.fbs"; root_type Monster;',
    },
    {
      path: "schemas/model/monster.fbs",
      source: 'include "types.fbs"; table Monster { pos:Vec3; }',
    },
    {
      path: "shared/types.fbs",
      source: "struct Vec3 { x:float; y:float; z:float; }",
    },
  ],
  includePaths: ["shared"],
};

function captureError(callback) {
  try {
    callback();
  } catch (error) {
    return error;
  }
  assert.fail("expected callback to throw");
}

assert.match(wasm.compile_fbs_to_rust(singleSchema, true), /Monster/);
assert.match(wasm.compile_fbs_to_ts(singleSchema, true), /Monster/);
assert.ok(wasm.compile_fbs_to_bfbs(singleSchema) instanceof Uint8Array);
assert.throws(
  () => wasm.annotate_flatbuffer(Uint8Array.of(0), singleSchema, "Monster"),
  Error,
);

assert.match(wasm.compile_fbs_files_to_rust(request, true), /Vec3/);
assert.match(wasm.compile_fbs_files_to_ts(request, true), /Vec3/);
assert.ok(wasm.compile_fbs_files_to_bfbs(request) instanceof Uint8Array);
const annotationError = captureError(() =>
  wasm.annotate_flatbuffer_files(Uint8Array.of(0), request, "Monster"),
);
assert.equal(annotationError.code, "annotation_error");

const malformedError = captureError(() =>
  wasm.compile_fbs_files_to_rust(
    {
      entryPath: "main.fbs",
      files: [{ path: "main.fbs", source: "table {" }],
    },
    false,
  ),
);
assert.equal(malformedError.code, "parse_error");

const missingError = captureError(() =>
  wasm.compile_fbs_files_to_bfbs({
    entryPath: "main.fbs",
    files: [
      {
        path: "main.fbs",
        source: 'include "missing.fbs"; table Main { value:int; }',
      },
    ],
  }),
);
assert.equal(missingError.code, "include_not_found");
assert.equal(missingError.details.include, "missing.fbs");

const cycleError = captureError(() =>
  wasm.compile_fbs_files_to_bfbs({
    entryPath: "a.fbs",
    files: [
      { path: "a.fbs", source: 'include "b.fbs"; table A { x:int; }' },
      { path: "b.fbs", source: 'include "a.fbs"; table B { x:int; }' },
    ],
  }),
);
assert.equal(cycleError.code, "include_cycle");

const traversalError = captureError(() =>
  wasm.compile_fbs_files_to_bfbs({
    entryPath: "schemas/main.fbs",
    files: [
      {
        path: "schemas/main.fbs",
        source: 'include "../secret.fbs"; table Main { x:int; }',
      },
      { path: "secret.fbs", source: "table Secret { value:string; }" },
    ],
  }),
);
assert.equal(traversalError.code, "path_traversal");

console.log("WASM Node integration tests passed");
