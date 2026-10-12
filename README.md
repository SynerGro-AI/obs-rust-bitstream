# obs-rust-bitstream

Perfect, tested foundation port of the OBS Studio `bitstream_reader` (the famous `uint8_t` wrap-at-255 bug) for the [obs-rust](https://github.com/obs-rust/obs-studio) effort.

## The foundational "opcode": Position trait

Wrap vs no-wrap is a **type-level property**:

- `usize` → never wraps (safe API)
- `u8` → `wrapping_add(1)` (C ABI, exact original behavior)

One shared bit-reading implementation. Math is perfect. Tests prove both paths.

## Layout (ready for obs-rust tree)

```
src/
  lib.rs
  bitstream.rs          # safe core + Position + C ABI shims + unit tests
tools/
  obs_cpp_to_rust_parser.py   # tree-sitter + quirks parser that points at this pattern
  README_parser.md
```

## Usage

```bash
cargo test
```

All tests pass (basic bits, r8/r16, safe no-wrap past 255, ABI wrap + re-read, empty).

## Foundational plan

1. **Foundation first** — this crate (Position trait + perfect bitstream).
2. **Parser wired** — `tools/obs_cpp_to_rust_parser.py` recognizes the pattern and emits the Position approach.
3. **Test** — unit tests already green.
4. **Test suite** — expand differential tests against original C oracle.
5. **Optimize** — then re-test.
6. **Next files** — apply the same perfect pattern to more util modules (dstr, utf8, serializers, etc.).
7. **Repeat** — foundation → parse → test → optimize → move on.

Public, free, open. Contributions and AI-assisted PRs welcome (the whole point of the obs-rust fork).

## License

GPL-2.0-or-later (matching OBS Studio).
