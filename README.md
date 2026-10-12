# obs-rust-bitstream

**Public foundation for the OBS Studio rewrite in Rust.**

Goal: beat the ~800 scripts already finished upstream by using ~10% of the API actions.  
**Work smarter, not harder.**

Reusable patterns + a parser that points at them + a differential helper that can be reused everywhere.

## Foundation modules (live)

| Module | What it does | Notes |
|--------|----------------|-------|
| **bitstream** | Position trait (wrap vs no-wrap is type-level) | Fixes uint8_t wrap-at-255 |
| **dstr** | Safe dynamic string | Fixes infinite loop + overflow |
| **utf8** | Safe truncation | Never splits a multibyte character |
| **diff** | Reusable differential test helper | One harness for many modules |
| **crc32** | Standard CRC32 | Classic test vector included |

All have unit tests. C ABI shims keep original quirks where required.

## Status

- [x] Five foundation modules with tests
- [x] Parser tools wired
- [x] Reusable differential helper
- [x] CI (cargo test on push/PR)
- [x] Public under SynerGro-AI
- [ ] Wire differential into more modules
- [ ] Batch application to remaining util files
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
tools/
  obs_cpp_to_rust_parser.py
.github/workflows/ci.yml
PLAN.md
```

## Usage

```bash
cargo test
```

## Foundational plan

1. Foundation patterns first — done
2. Parser wired — done
3. High-leverage modules + reusable differential helper — done
4. Batch-apply to the rest
5. Keep pushing — public, free, open

AI-assisted contributions and PRs welcome.

See also the upstream [obs-rust/obs-studio](https://github.com/obs-rust/obs-studio) effort.

## License

GPL-2.0-or-later (matching OBS Studio).
