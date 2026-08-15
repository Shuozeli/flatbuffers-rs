# flatc-rs WASM API

This crate builds the FlatBuffers schema compiler for browsers and Node.js.
The generated package contains JavaScript glue, TypeScript declarations, and
`flatc_rs_wasm_bg.wasm`.

## Build

The `wasm-bindgen-cli` version must match the `wasm-bindgen` version in
`Cargo.lock`.

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.114 --locked
cargo build --release -p flatc-rs-wasm --target wasm32-unknown-unknown --locked
wasm-bindgen target/wasm32-unknown-unknown/release/flatc_rs_wasm.wasm \
  --out-dir wasm-api/pkg \
  --target nodejs
pnpm --dir wasm-api test
```

Use `--target web` instead of `nodejs` when generating a browser package.

## Multi-file API

The multi-file functions accept a virtual filesystem request:

```js
const request = {
  entryPath: "schemas/main.fbs",
  files: [
    { path: "schemas/main.fbs", source: 'include "types.fbs"; ...' },
    { path: "shared/types.fbs", source: "..." },
  ],
  includePaths: ["shared"],
};

const rust = wasm.compile_fbs_files_to_rust(request, true);
const typescript = wasm.compile_fbs_files_to_ts(request, true);
const bfbs = wasm.compile_fbs_files_to_bfbs(request);
```

All paths must be relative to the virtual root. The compiler rejects absolute
paths, traversal outside an include root, include cycles, excessive include
depth, and excessive file counts. Multi-file API failures throw an object with
stable `code`, `message`, and optional `details` properties. The original
single-source functions remain available for compatibility.
