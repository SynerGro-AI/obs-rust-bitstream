# OBS C/C++ → Rust Parser (Learned Lessons)

The parser loads `lessons.json` and emits skeletons that already
document the known bugs and point at the safe foundation modules.

## Usage

```bash
python3 tools/obs_cpp_to_rust_parser.py path/to/obs/file.c [out.rs]
```

## Adding a lesson

Edit `tools/lessons.json` or call the `note_lesson` helper from Python.
Each lesson records:

- match terms
- the original issue
- the fix applied in the safe path
- the module that demonstrates it
- a short hint

This keeps future generations smarter and reduces manual rework.

## AI assistance

For anything the lessons do not cover, feed the generated skeleton
to the AI scaffold in `src/ai.rs` (Grok / Claude / Seraphina).
Simple config swap is already there.

Glyph planner is not required for this step; the structured lessons
are the high-leverage “opcode” for the parser.
