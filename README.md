# obs-rust-bitstream

**Public foundation for the OBS Studio rewrite in Rust.**

Goal: beat the ~800 scripts already finished upstream by using ~10% of the API actions.  
**Work smarter, not harder.**

Reusable patterns + differential helper + AI scaffold for automated assistance.

## Foundation modules (live)

| Module | What it does | Notes |
|--------|----------------|-------|
| **bitstream** | Position trait (wrap vs no-wrap is type-level) | Differential on short reads |
| **dstr** | Safe dynamic string | Fixes infinite loop + overflow |
| **utf8** | Safe truncation | Never splits a multibyte character |
| **diff** | Reusable differential test helper | Wired into modules |
| **crc32** | Standard CRC32 | Classic test vector |
| **ai** | AI/SI integration scaffold | Simple config for Grok, Claude, or Seraphina |
| **array_serializer** | Vec-based output serializer + LE helpers | Differential on short writes |
| **text_lookup** | HashMap store with escape expansion | Differential on insert/get |
| **path_extension** | Safe extension extractor | None on null/empty |
| **bmem** | Alloc size check + alignment contract | 32-byte alignment |

All have unit tests. C ABI shims keep original quirks where required.

## Status

- [x] Ten foundation modules with tests
- [x] Differential helper wired into modules
- [x] AI integration scaffold (swap backends with one variable)
- [x] Parser tools wired
- [x] CI (cargo test on push/PR)
- [x] Public under SynerGro-AI
- [ ] Batch application to remaining util files
- [ ] Live HTTP client in AI scaffold
- [ ] Workspace layout

## Layout

```
src/
  lib.rs
  bitstream.rs
  dstr.rs
  utf8.rs
  diff.rs
  crc32.rs
  ai.rs
  array_serializer.rs
  text_lookup.rs
  path_extension.rs
  bmem.rs
tools/
  obs_cpp_to_rust_parser.py
.github/workflows/ci.yml
PLAN.md
```

## Usage

```bash
cargo test
```

## AI integration

Simple variable adjustment:

```rust
use obs_rust_bitstream::ai::{AiAssistant, AiConfig};

let ai = AiAssistant::new(AiConfig::default());          // Grok
// let ai = AiAssistant::new(AiConfig::claude());       // Claude
// let ai = AiAssistant::new(AiConfig::seraphina());    // Seraphina
let review = ai.review_skeleton("/* generated rust */");
```

## Foundational plan

1. Foundation patterns first — done
2. Parser wired — done
3. High-leverage modules + differential helper — done
4. AI scaffold for automated help — done
5. Batch-apply to the rest
6. Keep pushing — public, free, open

AI-assisted contributions and PRs welcome.

See also the upstream [obs-rust/obs-studio](https://github.com/obs-rust/obs-studio) effort.

## License

GPL-2.0-or-later (matching OBS Studio).
