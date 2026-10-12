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
- Parser tools that point at the patterns.
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
