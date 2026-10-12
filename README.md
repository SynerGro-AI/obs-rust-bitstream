# obs-rust-bitstream

**Public foundation for the OBS Studio rewrite in Rust.**

Perfect, tested port of the OBS Studio `bitstream_reader` (the famous `uint8_t` wrap-at-255 bug). This is the first module of a public, free, open Rust rewrite of OBS internals, following the obs-rust approach: keep the public C ABI, rewrite internals in safe Rust, fix the bugs.

## The foundational "opcode": Position trait

Wrap vs no-wrap is a **type-level property**:

- `usize` → never wraps (safe API)
- `u8` → `wrapping_add(1)` (C ABI, exact original behavior)

One shared bit-reading implementation. Math is perfect. Tests prove both paths.

## Status

- [x] Foundation: perfect bitstream with Position trait + C ABI shims + unit tests
- [x] Parser tools wired (tree-sitter + quirks)
- [x] Public repo under SynerGro-AI
- [ ] Differential test suite against original C oracle
- [ ] Next modules: dstr, utf8, serializers, etc.
- [ ] Full obs-util port
- [ ] Integration into a larger obs-rust workspace

## Layout

```
src/
  lib.rs
  bitstream.rs          # safe core + Position + C ABI + tests
tools/
  obs_cpp_to_rust_parser.py
  README_parser.md
.github/workflows/
  ci.yml                # cargo test on push/PR
```

## Usage

```bash
cargo test
```

All tests pass (basic bits, r8/r16, safe no-wrap past 255, ABI wrap + re-read, empty).

## Foundational plan (executing)

1. **Foundation first** — this crate.
2. **Parser wired** — done.
3. **Test** — unit tests green.
4. **Test suite** — expand differential tests against C oracle.
5. **Optimize** → re-test.
6. **Next files** — apply the same perfect pattern to more util modules.
7. **Repeat** — foundation → parse → test → optimize → move on.

This is the public starting point for the OBS Rust rewrite. AI-assisted contributions and PRs welcome.

See also the upstream [obs-rust/obs-studio](https://github.com/obs-rust/obs-studio) effort.

## License

GPL-2.0-or-later (matching OBS Studio).
