# Public OBS Rust Rewrite — Foundational Plan

This repo is the public starting point.

## Current foundation
- Perfect bitstream port with Position trait (wrap vs no-wrap is type-level)
- C ABI shims that preserve original quirks
- Unit tests that prove the math
- Parser tools that recognize the pattern
- CI that runs cargo test
- **dstr foundation module** (safe path fixes infinite-loop on empty find + overflow)

## Next steps (in order)

1. Differential test suite against the original C oracle (obs-c-oracle style)
2. Expand dstr with full C ABI shims and more tests
3. Port next util modules using the same perfect pattern:
   - utf8 (fix mid-character truncation at 255 bytes)
   - path_extension, serializers, crc32, etc.
4. Workspace layout for multiple crates (obs-util-rust, obs-graphics-rust, ...)
5. Integration notes for dropping into obs-rust/obs-studio
6. Optimize hot paths, re-test, fuzz
7. Keep pushing — public, free, open

## Principles
- Foundation first
- Math is perfect (prove it with tests)
- Preserve public C ABI
- Fix the bugs in the safe path
- AI-assisted contributions welcome

## How to contribute
Clone, cargo test, open a PR. Point the parser at the next .c file and improve the generated skeleton until the tests pass.
