# Public OBS Rust Rewrite — Foundational Plan

Goal: beat the ~800 scripts already finished upstream by using ~10% of the API actions.
**Work smarter, not harder.**

## Current foundation (high-leverage patterns)
- **bitstream** — Position trait (wrap vs no-wrap is type-level). One abstraction covers the ABI quirk forever.
- **dstr** — safe path fixes infinite-loop on empty find + overflow.
- **utf8** — safe truncation that never splits a multibyte character (fixes the 255-byte filename crash).
- Parser tools that recognize the patterns and emit skeletons pointing at them.
- CI that runs cargo test on every push.

## Smart multiplier
One perfect pattern (Position, safe truncation, bug-fix tests) applies to many files.
Parser + quirks table turns one action into coverage for dozens of modules.

## Next steps (in order)
1. Strengthen the parser to emit more complete, testable skeletons in one pass.
2. Differential test harness (vs C oracle) that can be reused across modules.
3. Batch-apply the patterns to the remaining util files (path_extension, serializers, crc32, bmem, etc.).
4. Workspace layout so new crates drop in with the same structure.
5. Keep the public repo growing with high-leverage commits.

## Principles
- Foundation first
- Math is perfect (prove it with tests)
- Preserve public C ABI
- Fix the bugs in the safe path
- Work smarter: reusable patterns > one-off ports
- AI-assisted contributions welcome

## How to contribute
Clone, cargo test, open a PR. Point the parser at the next .c file; improve the generated skeleton until the tests pass. The patterns are already here.
