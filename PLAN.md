# Public OBS Rust Rewrite — Foundational Plan

Goal: beat the ~800 scripts already finished upstream by using ~10% of the API actions.
**Work smarter, not harder.**

## Current foundation (high-leverage patterns)
- **bitstream** — Position trait (wrap vs no-wrap is type-level).
- **dstr** — safe path fixes infinite-loop on empty find + overflow.
- **utf8** — safe truncation that never splits a multibyte character.
- **diff** — reusable differential test helper (one harness for many modules).
- **crc32** — standard CRC32 with the classic test vector.
- Parser tools that point at the patterns.
- CI that runs cargo test on every push.

## Smart multiplier
One perfect pattern applies to many files.
The differential helper can be reused for every new module.
Parser + quirks table turns one action into coverage for dozens of modules.

## Next steps (in order)
1. Wire the differential helper into bitstream / dstr / utf8 tests.
2. Batch-apply the patterns to the remaining util files (path_extension, serializers, bmem, etc.).
3. Workspace layout so new crates drop in with the same structure.
4. Keep the public repo growing with high-leverage commits.

## Principles
- Foundation first
- Math is perfect (prove it with tests)
- Preserve public C ABI
- Fix the bugs in the safe path
- Work smarter: reusable patterns > one-off ports
- AI-assisted contributions welcome

## How to contribute
Clone, cargo test, open a PR. Point the parser at the next .c file; reuse the patterns and the differential helper.
