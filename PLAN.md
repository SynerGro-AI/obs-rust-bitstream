# Public OBS Rust Rewrite — Foundational Plan

Goal: beat the ~800 scripts already finished upstream by using ~10% of the API actions.
**Work smarter, not harder.**

## Current foundation (high-leverage patterns)
- **bitstream** — Position trait (wrap vs no-wrap is type-level). Differential wired for short reads.
- **dstr** — safe path fixes infinite-loop on empty find + overflow. Differential wired.
- **utf8** — safe truncation that never splits a multibyte character. Differential wired.
- **diff** — reusable differential test helper (now used in modules).
- **crc32** — standard CRC32 with the classic test vector. Differential wired.
- **ai** — simple config scaffold to switch Grok / Claude / Seraphina for automated assistance.
- **array_serializer** — Vec-based output serializer + little-endian helpers. Differential wired.
- **text_lookup** — HashMap store with escape expansion. Differential wired.
- **path_extension** — safe extension extractor (None on null/empty). Differential wired.
- **bmem** — allocation size check + 32-byte alignment contract. Differential wired.
- **util_uint64** — exact (num * mul) / div via 128-bit intermediate. Differential wired.
- **util_uint128** — safe add and mul64_64 using native u128. Differential wired.
- **deque** — safe double-ended queue (circular buffer). Differential wired.
- **darray** — safe dynamic array (Vec wrapper). Differential wired.
- **config** — pure in-memory config store with defaults. Differential wired.
- **lexer** — pure base-token lexer. Differential wired.
- **serializer** — pure serializer trait + memory implementation. Differential wired.
- **profiler** — pure local profiler. Differential wired.
- **task** — pure task queue. Differential wired.
- **file_serializer** — pure file serializer. Differential wired.
- **buffered_serializer** — pure buffered serializer. Differential wired.
- **cf_lexer** — pure CF lexer. Differential wired.
- Parser tools that load `tools/lessons.json` and emit skeletons already annotated with the known fixes (Position trait, safe dstr, utf8 boundary, null-safety, zero-size alloc, etc.).
- CI that runs cargo test on every push.

## Smart multiplier
One perfect pattern applies to many files.
The differential helper is reused across modules.
AI config lets you swap backends with one variable change for automated help.
Parser + quirks table turns one action into coverage for dozens of modules.

## Next steps (in order)
1. Batch-apply the patterns to the remaining util files (path_extension, serializers, bmem, etc.).
2. Wire a real HTTP client into the AI scaffold (reqwest) for live automated review/generation.
3. Workspace layout so new crates drop in with the same structure.
4. Keep the public repo growing with high-leverage commits.

## Principles
- Foundation first
- Math is perfect (prove it with tests)
- Preserve public C ABI
- Fix the bugs in the safe path
- Work smarter: reusable patterns > one-off ports
- AI-assisted contributions welcome (Grok, Claude, or Seraphina)

## How to contribute
Clone, cargo test, open a PR. Point the parser at the next .c file; reuse the patterns, the differential helper, and the AI scaffold.
