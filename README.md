# obs-rust-bitstream

**Public foundation for the OBS Studio rewrite in Rust.**

Goal: beat the ~800 scripts already finished upstream by using ~10% of the API actions.  
**Work smarter, not harder.**

Reusable patterns (Position trait, safe truncation, bug-fix tests) + a parser that emits skeletons pointing at those patterns.

## Foundation modules (live)

| Module | What it does | Bug fixed in safe path |
|--------|----------------|------------------------|
| **bitstream** | Position trait makes wrap vs no-wrap a type-level property | uint8_t wraps at 255 and re-reads byte 0 |
| **dstr** | Safe dynamic string | Infinite loop on empty find; buffer overflow on insert |
| **utf8** | Safe truncation | Never splits a multibyte character at the 255-byte limit |

All have unit tests that prove the math/fixes. C ABI shims keep original quirks where required.

## Status

- [x] Foundation patterns (Position, safe truncation, bug fixes)
- [x] Three modules with tests
- [x] Parser tools wired
- [x] CI (cargo test on push/PR)
- [x] Public under SynerGro-AI
- [ ] Differential test harness (reusable across modules)
- [ ] Batch application to remaining util files
- [ ] Workspace layout for more crates

## Layout

```
src/
  lib.rs
  bitstream.rs
  dstr.rs
  utf8.rs
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
3. High-leverage modules with tests — in progress
4. Reusable differential suite
5. Batch-apply patterns to the rest
6. Keep pushing — public, free, open

AI-assisted contributions and PRs welcome.

See also the upstream [obs-rust/obs-studio](https://github.com/obs-rust/obs-studio) effort.

## License

GPL-2.0-or-later (matching OBS Studio).
