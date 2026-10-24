# Public OBS Rust Rewrite — Plan

Goal: Seraphina Studio, a desktop app for paint, composite, 3D,
animation, streaming and a materials lab, and the tested library
behind it.
**Work smarter, not harder.**

## Where things stand

* The library modules that exist are listed in README.md (the list is
  checked against `src/lib.rs` by a test). Each has unit tests.
* `bitstream` is a port of OBS `bitstream_reader` and keeps a C ABI.
* Recording and RTMP streaming run in the app through ffmpeg
  (`src/bin/ui/output.rs`), tested against a local ffmpeg RTMP listener
  and, on Linux CI, mediamtx. Their source is the canvas, the screen, or a
  webcam (`src/bin/ui/camera.rs`: V4L2 on Linux, Media Foundation on
  Windows; macOS AVFoundation is planned). Live output has an optional
  stream delay (`encoder_packet` + `output_delay_queue` between the
  encoder ffmpeg and a copy-only relay ffmpeg).
* `tools/obs_cpp_to_rust_parser.py` turns an OBS `.c` file into a Rust
  skeleton that documents the lessons in `tools/lessons.json`.

## Planned (not built)

The v1 cleanup removed modules that the app and examples never used
(the `every_library_module_is_reached_from_a_binary_or_example` test
keeps it that way). They come back only as real implementations, each
with a test. The tables in README.md ("Planned (not built)") hold the
target dates (America/Edmonton; targets, not promises):

* v1 cleanup and M4 part 2: Wed Oct 28, 2026 (target).
* First green CI run, run locally: Sun Nov 1, 2026 (target). The GitHub
  run waits for the billing lock to lift; it does not unlock by itself.
* v0.2.0 release: Fri Nov 6, 2026 (target, pending the owner's approval).

Everything below comes after v0.2.0 and has no date yet:

1. Seraphina.AGI as a separate local process (Grok is today's
   provider): proposed.
2. Remove `dist/` and root clutter: M5.
3. Physics foundation: M6.
4. OBS sources, scenes and the rest of Stream (E6): M7+.
5. The rest of the Photoshop / After Effects / Maya tools listed as
   `[ ]` in TOOLS.md: M7+.

## C4: Shared learning (spec; planned, not built)

The owner's words: "Build shared learning into v1." The learning rule:
"Live weights, that learn by not using full re writes, but instead store
learned or updated lessons not stored yet." The vision: copies of the
app share learned lessons over a mesh gossip network, using glyph data
transfer, where a glyph image carries the data.

This section is the spec. No code lands until it has been reviewed.
Everything here is planned; the "Later" items are not part of v1.

### What it builds on

The local learner (`src/bin/ui/prefs.rs`, `Prefs`, `MercyBank`) is the
only learner. At cfb1cad it keeps EMA weights (α = 0.382) and event
counts in `mercy_bank.json`. The M4b learner work in progress adds:

* a lesson unit `Lesson { v, key, weight, count }` with
  `LESSON_VERSION = 1`;
* key grammar (`lesson_key`): `a-z 0-9 _ . -`, dot-separated non-empty
  parts, at most 96 bytes;
* key families: `tool.<mode>.<tool>`, `cmd.<user|ai>.<mode>.<command>`,
  `param.<key>`, `ai.<command>`, `ai.acceptance` and
  `reply.helpfulness`;
* an append-only delta log (`mercy_bank.lessons.jsonl`), where each line
  holds only the lessons that are new or changed since the last save,
  and compaction into a new base at 256 KiB;
* the `LessonStore` trait as the seam for Seraphina.AGI.

Shared learning reuses that lesson unit and that log. It adds no second
learner and no second store.

### Which lessons are shared

Only lessons whose weight means something to another user are shared.
Each one is keyed by names from compiled allow-lists: `Mode::ALL`
(paint, composite, scene, animate, stream, materials) and the AppCommand
names the bus already exposes (the same list as `docs/AI_TOOLS.md`).

| Shared key (on the wire) | Weight sent | Imported as |
|---|---|---|
| `ai.<command>` | the sender's EMA: how often they keep Seraphina's `<command>` rather than undo it (0..=1) | `peer.ai.<command>` |
| `cmd.user.<mode>.<command>` | the command's share of the sender's own commands in that mode: `count / Σ count(cmd.user.<mode>.*)` (0..=1) | `peer.cmd.<mode>.<command>` |

These stay local and are never exported in v1:

* `param.*`: raw values in mixed units, such as canvas sizes, which can
  identify a user;
* `tool.*`: overlaps `cmd.user.*`;
* `cmd.ai.*`: Seraphina's own usage, so sharing it would feed back on
  itself;
* `ai.acceptance` and `reply.helpfulness`: whole-user aggregates.

Imported lessons go to their own `peer.` keys. They never change the
user's own weights or counts, so the user's own learning always wins,
and an import can be undone exactly. `peer.*` weights start at 0.5
(neutral). They change only by bounded additive deltas, never by EMA.

**Export is incremental, like the log.** A pack holds only the shared
lessons that are new, or whose quantized weight changed, since the last
pack this install exported. The bank is never sent whole.

### Lesson pack format (v1)

A lesson pack is a fixed binary layout. It is canonical, so one meaning
has exactly one encoding and the signature covers it byte for byte. It
is not JSON: field order, whitespace and float formatting would make
the signed bytes ambiguous. All integers are little-endian.

| Offset | Size | Field | Rule |
|---|---|---|---|
| 0 | 4 | magic | `SSLP` |
| 4 | 1 | format version | `1`; any other value is refused |
| 5 | 1 | flags | `0`; any other value is refused |
| 6 | 2 | allow-list revision | the sender's allow-list revision; shown in the preview only |
| 8 | 32 | sender public key | Ed25519 |
| 40 | 8 | sequence | starts at 1 and goes up by 1 per export from this install |
| 48 | 8 | created | Unix seconds, rounded down to the day (less fingerprinting) |
| 56 | 16 | nonce | random (`getrandom`) |
| 72 | 1 | entry count | 1..=128 |
| 73 | n | entries | see below |
| end − 64 | 64 | signature | Ed25519 over `b"seraphina-studio/lesson-pack/v1\0"` followed by bytes `0..end−64` |

Each entry is `key_len: u8` (1..=64), then `key` (ASCII), then
`weight_q: u16` (0..=10000, where weight = `weight_q / 10000`), then
`evidence: u16` (the sender's event count, saturating at 65535).
Entries are sorted strictly by key: an unsorted or duplicate key is
refused.

* **Maximum size: 8,969 bytes** (`MAX_PACK_BYTES` = 73 + 128 × 69 + 64).
  The size is checked before anything is parsed. Bytes after the
  signature are refused.
* **No code, no text.** A pack holds a magic number, integers, one key
  and 64 bytes of signature. Each key must match the wire grammar
  (`^(ai\.[a-z0-9_]+|cmd\.user\.[a-z]+\.[a-z0-9_]+)$`, at most 64
  bytes). It must also be in this build's allow-list, with a known
  mode and a known AppCommand. There is no field that can hold a script,
  a prompt, a tool call, a file path, a URL, a name or an address. One
  bad key refuses the whole pack. Packs are never partly applied.
* **Pack id**: SHA-256 of the whole pack, signature included. It is
  used for the seen-set, the ledger and quarantine names.
* **Versioning**: a new layout gets a new format version and a new
  signing domain string. v1 readers refuse every other version, and the
  preview says "made by a newer Seraphina Studio". A key added to the
  allow-list in a newer release is refused by older readers for the
  same reason. Nothing is dropped silently.

### Import: bounded deltas with clamping

The pack carries quantized weights, not raw deltas, because a sender's
raw delta means nothing against the receiver's different baseline. The
change the importer applies is a bounded delta, computed and clamped on
import. For each entry, with `p` the current `peer.*` weight (0.5 if it
is absent):

* Entries with `evidence < 3` are skipped, and the preview lists them as
  "too little evidence".
* `β = 0.382 × min(evidence, 32) / 32`
* `Δ = clamp(β × (w − p), −0.05, +0.05)`, so one pack moves a key by at
  most 0.05.
* Per-sender cap: the sum of one sender's applied Δ on one key stays
  within ±0.15, recorded in the ledger. One sender alone can never move
  a key more than 0.15 from where everyone else left it.
* `p' = clamp(p + Δ, 0, 1)`. The Δ that was actually applied, after
  clamping, is recorded.

An imported lesson can only change `peer.*` numbers. They reach
Seraphina in one line of the system-prompt summary, built only from
allow-listed command names and numbers, for example "other Studio users
often undo: set_layer_blend (peer lessons from 4 senders)". No text
from a pack ever reaches the model. A pack cannot add tools, run
commands or change settings.

### Glyph PNG: export and import

**Real for v1: a private PNG chunk.** The exported file is a real PNG,
a 512 × 512 "glyph card". It is drawn with the existing text rasteriser
and shows the sender's key fingerprint, the date, the number of lessons
and a summary in plain words. The pack itself is stored in one private
ancillary chunk of type `seLp`:

* `s`: ancillary;
* `e`: private;
* `L`: the reserved bit, which must be upper case;
* `p`: safe to copy.

Writing uses the `png` crate (0.18, already in `Cargo.lock` through
`image`). `Writer::write_chunk` writes the `seLp` chunk before the image
data.

Reading does not decode pixels. A small chunk walker checks:

* the PNG signature;
* each chunk's length and CRC (`crc32fast`, already in `Cargo.lock`);
* that there is exactly one `seLp` chunk, and that it comes before
  `IDAT`;
* that `seLp` is at most `MAX_PACK_BYTES` long.

It also refuses:

* a file over 1 MiB;
* a declared chunk length longer than the bytes that remain (no
  allocation from an untrusted length);
* a missing `IEND`.

Only then is the pack parsed and verified.

* **Round trip**: export, then import, gives byte-identical pack bytes.
  The card decodes as a normal image with the `image` crate.
* **Known limit**: sites that re-encode images (social networks, many
  chat apps) strip private chunks. v1 says this in the export dialog.
  Files sent as attachments, on USB or over the LAN keep the chunk.
* **Later: the visual glyph.** This means data encoded in the pixels,
  so it survives a re-encode or a screenshot. It needs a fully specified
  code: cell size, colours, error correction and a capacity of at least
  `MAX_PACK_BYTES`, which is beyond a QR code's 2,953 bytes. Until that
  spec exists and has tests, it is labelled later, and the card's
  pixels carry no data.

### Transport and discovery

**Real and testable in v1:**

1. **Manual file share.** Export through a Save dialog (`rfd`), and
   import through an Open dialog or by dropping a `.png` on the window.
   How the file travels (email, USB, a shared folder) is up to the user.
2. **LAN transport over real sockets.** This is a separate toggle, off
   by default. Nothing is bound while it is off.
   * Discovery: a UDP beacon on port 47310, sent to the subnet broadcast
     address every 10 s and on demand. It holds `SSLB`, the version,
     the public key, the TCP port and the day, signed with the install
     key. It is 107 bytes: magic 4, version 1, key 32, port 2, day as `u32`
     4, signature 64. There is no hostname or user name. A beacon with
     a bad signature is ignored and counted.
   * Offer: a TCP connection to port 47311. One frame holds a `u32`
     length (≤ `MAX_PACK_BYTES`) and the pack. There is a 5 s timeout,
     at most 8 connections at once, and at most 64 KiB read per
     connection.
   * Received packs go to an import inbox. They are never applied
     automatically.
   * Gossip in v1 is user-driven. "Share" sends a pack to the LAN peers
     the user picks. A pack the user imported can be passed on with
     "Share again", which is a new share and needs the same approval.
     The pack keeps its original signature, so every hop verifies the
     origin.
   * Windows asks for a firewall exception the first time the toggle
     is turned on. The setting says so beforehand.

**Later (not in v1):**

* automatic epidemic gossip (relaying without a click), hop limits and
  fan-out;
* an internet mesh, NAT traversal, relays or rendezvous servers, and a
  DHT;
* mDNS/DNS-SD discovery;
* transport over the Seraphina Glyph OS mesh.

### Consent

* **Off by default.** Settings → Learning has "Share lessons" (export
  and import) and, under it, "LAN sharing". Both are off. A test checks
  the defaults and that no socket is bound while they are off.
* **Each share is approved.** The export preview lists every lesson in
  plain words, with its value and evidence. The user can untick lines
  before choosing "Sign and save" or "Sign and send".
* **Each import is approved, with a preview of the change.** The
  preview shows:
  * the sender's fingerprint and nickname, and whether it is trusted,
    new or blocked;
  * every change as before → after, for example
    `peer.ai.set_layer_blend 0.50 → 0.45`;
  * skipped entries and the reason for each.

  Then the user picks Approve, Reject or Block sender.
* **Seraphina cannot share or import.** These actions are not
  AppCommands and are not in her tool list. A test checks that.

### Integrity, signing, replay and rate limits

* **Identity.** Each install has an Ed25519 keypair, generated on first
  use from `getrandom`. The fingerprint is the first 16 bytes of
  SHA-256(public key), shown as 8 groups of 4 hex digits. "New identity"
  makes a fresh key: old packs stay valid, and peers see a new sender.
* **Checks, in order.** Each check refuses on failure and never
  half-applies:
  1. size;
  2. PNG structure and CRC;
  3. layout;
  4. key grammar and allow-list;
  5. weight range (`weight_q` ≤ 10000);
  6. signature (`verify_strict`, which refuses weak keys and
     non-canonical signatures);
  7. replay;
  8. trust.
* **Trust.** The first pack from an unknown key needs an explicit
  "Trust this sender" in the preview (trust on first use, decided by
  the user). Blocked keys are refused before the preview opens.
* **Replay.** Each sender has a sliding window: a sequence at or below
  `max_seq − 64`, or one already seen, is refused. The pack-id seen-set
  is persisted in the ledger and survives restarts.
* **Rate limits.**
  * LAN offers: 1 per sender per 10 s, 16 packs in the inbox, 8
    connections, and 1 beacon per sender per 5 s. Anything over a limit
    is dropped and counted.
  * The UI shows "N offers dropped".
  * Imports: at most 20 per hour, even when approved, so a click-through
    flood can't move many keys quickly.
* **Quarantine.** A refused pack is copied to
  `lessons/quarantine/<pack-id>.png`, next to a `.txt` file that gives
  the reason. It is never applied.
* **Rollback through the delta log.** An approved import is written as
  one line in `mercy_bank.lessons.jsonl` that records the pack id, the
  sender and each key's applied Δ, and as one ledger line.
  * "Undo import" appends a line that subtracts that pack's Δ from each
    key and clamps to 0..=1, then marks the ledger line reverted.
  * "Block sender" offers to undo all of that sender's imports.
  * Compaction keeps the ledger, so an import can still be undone after
    the log has been folded into the base.
  * This needs one new log record kind in `prefs.rs`
    (`{ "import": { pack, sender, deltas } }` and
    `{ "revert": pack }`), versioned with `LESSON_VERSION`.

**Proposed crates.** These were checked on Sat 24 Oct 2026 by resolving
them into a copy of `Cargo.lock`, with no build:

| Crate | Version | MSRV | Why |
|---|---|---|---|
| `ed25519-dalek` | 2.2 (`default-features = false`, `std`, `zeroize`, `fast`) | 1.81 | keys, `verify_strict` |
| `sha2` | 0.10 | below 1.88 (edition 2018); already required by `ed25519-dalek` 2.2 | pack id, fingerprint |
| `getrandom` | 0.3 (already in `Cargo.lock`) | 1.63 | key seed, nonce |
| `png` | 0.18 (already in `Cargo.lock`) | 1.73 | write the `seLp` chunk |
| `crc32fast` | 1.5 (already in `Cargo.lock`) | 1.63 | verify chunk CRCs |

* All of them are within MSRV 1.88. `curve25519-dalek` 4.1.3 is 1.60,
  and `ed25519` and `signature` 2.2 are 1.60.
* `cargo audit` on the resolved lock: no vulnerabilities, and the same 5
  allowed warnings as today. The past advisories are patched in these
  versions:
  * RUSTSEC-2022-0093 (`ed25519-dalek` < 2);
  * RUSTSEC-2024-0344 (`curve25519-dalek` < 4.1.3);
  * RUSTSEC-2021-0100 (`sha2` < 0.9.8).
* The cost is 19 new crates in the lock.
* `ed25519-dalek` 3.0 (MSRV 1.85) is also out. 2.2 is proposed because
  it is the long-reviewed line.
* They are all optional and behind the `ui` feature, like the other app
  dependencies.

### Privacy and key storage

* Packs hold only allow-listed keys, quantized numbers, a pseudonymous
  public key and a day-rounded date. A test fills the learner with
  `param.*`, path-like and sentence-like inputs and checks that no
  export contains them.
* The public key links a user's packs to each other. "New identity"
  breaks that link, and the export dialog says so.
* **Secret key storage.** The 32-byte seed is stored as a new
  `secrets::Secret::LessonSigningKey`.
  * On Windows it goes in Credential Manager as
    `SeraphinaStudio/lesson_signing_key`. Elsewhere, or if Credential
    Manager refuses, it goes in the studio folder in a file that only
    the user can read (0600), the same as the xAI key.
  * It is registered with `crash::register_secret`, so it is never
    logged or put in a crash report.
* **Plain files** live under `~/.seraphina/studio/lessons/`:
  * `identity.pub`;
  * `peers.json` (trusted and blocked keys, nicknames);
  * `ledger.jsonl` (accepted and refused pack ids, sequences, applied
    deltas, reverts);
  * `quarantine/`.

  None of them is secret. `SERAPHINA_HOME` moves them, as it moves the
  bank.

### Adversarial tests (planned)

Every test uses real files and real sockets on 127.0.0.1. The two peers
are two real installs, each with its own temporary `SERAPHINA_HOME` and
keypair. One test runs the peer as a second process. There are no
simulated peers. Each test must fail if its check is removed.

| Test | Proves |
|---|---|
| `glyph_png_round_trips_a_pack` | export then import gives identical pack bytes; the card decodes as a PNG |
| `tampered_png_is_refused` | a flipped byte in `seLp` is refused by the CRC; with the CRC recomputed, it is refused by the signature; the pack is quarantined |
| `every_single_byte_flip_is_refused` | a real pack with each of its bytes flipped in turn is never accepted |
| `oversized_payload_is_refused` | an 8,970-byte chunk, a 2 MiB PNG and a 4 GiB declared length are refused with no large allocation |
| `truncated_chunk_is_refused` | a file cut inside `seLp`, inside `IDAT` and before `IEND` is refused |
| `two_lesson_chunks_are_refused` | two `seLp` chunks, or `seLp` after `IDAT`, are refused |
| `replayed_pack_is_refused` | importing the same file twice, including after a restart, is refused the second time |
| `old_sequence_is_refused` | a sequence outside the window is refused |
| `bad_signature_is_refused` | a body signed by another key, a zeroed signature and a non-canonical signature are refused |
| `unknown_key_needs_explicit_trust` | a valid pack from an unknown key changes nothing until it is trusted; a blocked key is refused |
| `out_of_range_weight_is_refused` | a correctly signed pack with `weight_q = 10001` is refused |
| `import_delta_is_clamped` | a pack at weight 1.0 with full evidence moves a key by 0.05 at most; repeated packs from one sender stop at 0.15 |
| `non_allow_listed_key_is_refused` | `param.canvas.width`, `ai.C:/users/x`, an unknown command, an unknown mode and an upper-case key each refuse the whole pack |
| `unsorted_or_duplicate_keys_are_refused` | canonical order is enforced |
| `lan_flood_is_rate_limited` | 1,000 offers from one sender over real TCP give at most 1 inbox entry per 10 s and 16 in total; the drops are counted; the listener keeps its connection cap |
| `beacon_with_bad_signature_is_ignored` | real UDP beacons with bad signatures never become peers |
| `lan_offer_between_two_processes` | a pack sent by a second process arrives in the inbox and is not applied until approved |
| `nothing_listens_while_sharing_is_off` | connecting to the LAN ports is refused with both toggles at their defaults |
| `revert_import_restores_weights` | "Undo import" restores every `peer.*` key, and still does after a restart and after compaction |
| `blocking_sender_reverts_its_imports` | blocking offers to undo the sender's imports, and the undo is exact |
| `export_contains_only_new_or_changed_lessons` | a second export holds only lessons that changed since the first |
| `export_never_holds_private_lessons` | `param.*`, `tool.*`, `cmd.ai.*`, aggregates and invalid keys are never exported |
| `seraphina_cannot_share_or_import` | the AI tool list has no share or import command |
| `peer_lessons_reach_the_prompt_only_as_names_and_numbers` | the summary line contains only allow-listed names and numbers |
| `signing_key_is_private` | Unix: the key file is 0600 and its value is scrubbed from crash reports. Windows: it is in Credential Manager (Windows-only, run on a Windows PC) |

### How Seraphina and Glyph OS plug in (later)

* **Seraphina.AGI provider (later).** It reads and teaches lessons
  through the `LessonStore` trait, under the same allow-list. Lessons it
  learns can be exported as packs signed by the studio's install key.
  It gets no extra rights: its imports go through the same preview.
* **Seraphina Glyph OS mesh (later).** The lesson pack is the payload
  unit. Glyph OS supplies the transport (mesh gossip, data moved as
  glyphs) and, once it is specified, the visual glyph encoding. The pack
  format, signature, replay window and clamping stay the same, so
  Studio and Glyph OS can trade packs. Glyph OS streaming and rendering
  Seraphina herself belong to Glyph OS, not to this spec.

### Open questions for the owner

1. **When:** "v1" could mean v0.2.0, with a target of
   Fri Nov 6, 2026 (target). This work (crypto, PNG, LAN, UI) is larger than the local
   learner, which stays on the M4b target of Wed Oct 28, 2026 (target).
   Ship shared learning in v0.2.0 and move the release date, or ship it
   in the first release after?
2. **What is shared:** keep/undo outcomes for Seraphina's commands, and
   which commands people use in each mode. Should brush and other
   settings (`param.*`) ever be shared? They are left out because they
   can identify a user.
3. **Peer lessons:** they are kept separate (`peer.*`), so the user's own
   habits always win. Or should they blend into the user's own weights?
4. **Trust:** the first import from a new sender needs "Trust this
   sender". Or should a one-off import be allowed without trusting the
   key?
5. **Gossip:** in v1 a pack is only passed on when the user clicks
   Share. Automatic relaying is later. Is that acceptable for v1?
6. **The glyph card:** what should it look like? And is a visual glyph
   that survives social-media re-encoding needed in v1? That would need
   its own spec first.

## Principles
- Foundation first
- Math is perfect (prove it with tests)
- Preserve the public C ABI where a module ports OBS C code
- Fix the bugs in the safe path
- Delete before you add: no placeholder modules
- Every README claim is backed by a test or a measurement

## How to contribute
Clone, run `cargo test --lib` and `cargo test --features ui --bin ui`,
open a PR.
