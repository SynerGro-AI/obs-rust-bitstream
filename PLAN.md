# Public OBS Rust Rewrite — Plan

Goal: a safe, tested Rust rewrite of the parts of OBS that a creator
needs, plus the Seraphina Studio desktop app on top of it.
**Work smarter, not harder.**

## Where things stand

* The library modules that exist are listed in README.md (the list is
  checked against `src/lib.rs` by a test). Each has unit tests.
* `bitstream` is a port of OBS `bitstream_reader` and keeps a C ABI.
* `diff` is the shared differential-test helper.
* Recording and RTMP streaming run in the app through ffmpeg
  (`src/bin/ui/output.rs`), tested against a local ffmpeg RTMP listener
  and, on Linux CI, mediamtx.
* `tools/obs_cpp_to_rust_parser.py` turns an OBS `.c` file into a Rust
  skeleton that documents the lessons in `tools/lessons.json`.

## Planned (not built)

The v1 cleanup removed placeholder modules that no binary used. They
come back only as real implementations, each with a test:

1. OBS sources: browser source, game capture, window capture, media,
   image and text sources.
2. OBS core: source/scene/filter/output/encoder/service registries,
   scene collections, transitions, hotkeys, profiles, replay buffer,
   virtual camera.
3. `pipe` (frame pipe) and `source_profiler` (per-source timing).
4. The rest of the Photoshop / After Effects / Maya tools listed as
   `[ ]` in TOOLS.md.

## Principles
- Foundation first
- Math is perfect (prove it with tests)
- Preserve the public C ABI where a module ports OBS C code
- Fix the bugs in the safe path
- Delete before you add: no placeholder modules
- Every README claim is backed by a test or a measurement

## How to contribute
Clone, run `cargo test --lib`, open a PR. Point the parser at the next
`.c` file and reuse the patterns and the differential helper.
