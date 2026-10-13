# Contributing to obs-rust-bitstream

This is a pure-Rust rewrite of OBS Studio. It is public, free, and open.
Anyone can help.

## How to start

1. Clone the repo.
2. Run `cargo test`.
3. Pick a module from the list below and open a pull request.

The full suite must stay green. Every new module needs a differential test.

## What is already done

The pure foundation is covered:

- util (bitstream, strings, crc, serializers, math)
- graphics math (vec, matrix, quat)
- media-io (frame rate, formats, scaler, mixer)
- core objects (source, scene, filter, encoder, output)
- live show (keyer, director, studio, tally)
- output (record, stream, virtual cam, replay, export)
- control (keybind, help, undo/redo, macro, schedule)
- filters (color, crop, scale, noise, gain, compressor, limiter, gate, eq, reverb)
- transitions (fade, swipe, slide, cut)
- sources (image, text, color, browser, media, window, display, game, audio, group)

## What is left

Pure pieces that still need a port:

- more filters (sharpen, blur, chroma key)
- more transitions (stinger, luma wipe)
- more sources (slideshow, playlist)

Thin wrappers (platform capture, codecs, outputs) come after the pure pieces.

## Rules

- No unwraps.
- Result only for errors.
- Differential test for every public method.
- One module per pull request.
- Commit message explains the ladder step.

## Questions

Open an issue. Or just start. The tests will tell you if it is right.
