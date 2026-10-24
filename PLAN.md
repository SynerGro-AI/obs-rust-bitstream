# Public OBS Rust Rewrite — Plan

Goal: Seraphina Studio, a desktop app for paint, composite, 3D,
animation, streaming and a materials lab, and the tested library
behind it.
**Work smarter, not harder.**

## Local compute first (design principle)

The owner's rule: "Use Python scripts whenever they are more efficient
than spending API credits, and let the CPU do the work instead of
burning credits. Prefer local compute over paid API calls wherever the
CPU can handle it faster or cheaper."

* Each spec has a "Paid calls still needed" list. For every paid call
  it says what the call is for and why the CPU can't do the job.
* Tests make no paid calls by default. Billed checks are `#[ignore]` and
  are run by hand, as `live_xai_*` and `live_imagine` already are.
* Offline tooling, such as fixtures, test values and doc checks, is a
  local Python script (for example `tools/lesson_pack_vectors.py`, which computes the shared-learning test values).

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

Shared learning by file (C7.1) is part of the v0.2.0 release; see
"C7: Shared learning" below.

Everything below comes after v0.2.0 and has no date yet:

1. Seraphina.AGI as a separate local process (Grok is today's
   provider): proposed.
2. Remove `dist/` and root clutter: M5.
3. Physics foundation: M6.
4. OBS sources, scenes and the rest of Stream (E6): M7+.
5. The rest of the Photoshop / After Effects / Maya tools listed as
   `[ ]` in TOOLS.md: M7+.
6. Shared learning over the LAN (C7.2): the release after v0.2.0.

## C7: Shared learning (spec; planned, not built)

The owner's words: "Build shared learning into v1." The learning rule:
"Live weights, that learn by not using full re writes, but instead store
learned or updated lessons not stored yet." The vision: copies of the
app share learned lessons over a mesh gossip network, using glyph data
transfer, where a glyph image carries the data.

This section is the spec. It was revised after the Auditor's review
(`spec-review-c4-bg.md`). It is called C7 because C4 already names the
M4b local learner ("One learning system shared by every mode (C4)").
Everything here is planned.

**Scope and dates (decided by the owner, Oct 24 2026: "Yes, go with all
four recommendations"):**

* **C7.1 is in v0.2.0:** file-based glyph-card packs only, meaning the
  pack format, signing, the glyph PNG, export and import by file or
  drag-and-drop, preview and consent, trust, replay protection, import
  limits, derived-state undo, and their tests. Target: v0.2.0,
  Fri Nov 6, 2026 (target, pending the owner's approval of the release).
* **C7.2 is LAN sharing, in the release after v0.2.0.** It covers the
  beacon, the TCP offers, the inbox and the firewall handling, with the
  hardening below. It has no date yet.
* **Hard dependency:** no C7 code lands until M4b is merged with its open
  review findings fixed. C7 builds on `Lesson`, `LessonStore` and the
  delta log, which exist only as M4b work in progress (they are not in
  `prefs.rs` at cfb1cad). If M4b slips past its
  Wed Oct 28, 2026 (target), the C7.1 date is re-planned with the owner
  rather than squeezed.

### What it builds on

The local learner (`src/bin/ui/prefs.rs`, `Prefs`, `MercyBank`) is the
only learner. At cfb1cad it keeps EMA weights (α = 0.382) and event
counts in `mercy_bank.json`. The M4b work in progress adds:

* a lesson unit `Lesson { v, key, weight, count }` with
  `LESSON_VERSION = 1`;
* key grammar (`lesson_key`): `a-z 0-9 _ . -`, dot-separated non-empty
  parts, at most 96 bytes;
* key families: `tool.<mode>.<tool>`, `cmd.<user|ai>.<mode>.<command>`,
  `param.<key>`, `ai.<command>`, `ai.acceptance` and
  `reply.helpfulness`;
* an append-only delta log (`mercy_bank.lessons.jsonl`) with compaction
  at 256 KiB;
* the `LessonStore` trait as the seam for Seraphina.AGI.

Shared learning reuses that lesson unit and that log. It adds no second
learner and no second store.

### Which lessons are shared

Only lessons whose weight means something to another user are shared,
keyed by names from compiled allow-lists (`Mode::ALL` and the AppCommand
names in `docs/AI_TOOLS.md`). This is allow-list revision 1.

| Shared key (on the wire) | Weight sent | Imported as |
|---|---|---|
| `ai.<command>` | the sender's EMA: how often they keep Seraphina's `<command>` rather than undo it (0..=1) | `peer.ai.<command>` |
| `cmd.user.<mode>.<command>` | the command's share of the sender's own commands in that mode: `count / Σ count(cmd.user.<mode>.*)` (0..=1) | `peer.cmd.<mode>.<command>` |

These stay local and are never exported: `param.*` (can identify a
user), `tool.*` (overlaps `cmd.user.*`), `cmd.ai.*` (Seraphina's own
usage, so sharing it would feed back on itself), and the aggregates
`ai.acceptance` and `reply.helpfulness`.

Imported lessons only ever affect `peer.*` values, never the user's own
weights or counts, so the user's own learning always wins. Export is
incremental: a pack holds only the shared lessons that are new, or whose
quantized weight changed, since this install's last pack.

### Lesson pack format (v1)

The byte layout, the signing domains, and the test keys and values are
published in [docs/LESSON_PACK.md](docs/LESSON_PACK.md), so that
Seraphina.AGI, RWN and Glyph OS can check their own readers byte for
byte. In short:

* **Header:** `SSLP`, the version `1`, the flags `0`, the allow-list
  revision `u16`, the origin public key (32 bytes), the sequence `u64`,
  and `created` as a `u64` rounded down to the UTC day.
* **Entries:** 1..=128 of them, each a `key_len` byte, the key,
  `weight_q` (0..=10000) and `evidence`. Keys are sorted strictly.
* **Signature:** Ed25519 over `seraphina/lesson-pack/v1\0` followed by
  the body.
* **Size:** `MAX_PACK_BYTES` = 8,953. There is no random nonce: the
  origin key and sequence make each pack unique, and identical content
  gives identical bytes.
* **No code, no text:** a pack holds a magic number, integers, keys from
  the allow-list grammar and a signature. It has no field that can hold
  a script, a prompt, a tool call, a path, a URL, a name or an address.
  One bad key refuses the whole pack, and packs are never partly
  applied.
* **Test values:** `tools/lesson_pack_vectors.py` (Python with the
  `cryptography` package, checked against RFC 8032 TEST 1) computed the
  files in `tests/fixtures/lesson_pack/`. Running it with `--check`
  recomputes them and compares byte for byte. The Rust reader's tests
  will load the same files.

### Import: derived state, bounded and exactly undoable

The pack carries quantized weights, not deltas, because a sender's
delta means nothing against the receiver's own baseline. The receiver
works out a bounded change and stores only that change. This is
receiver policy, not part of the wire format, so RWN can fuse the same
numbers in log-odds instead.

* **Ledger of changes:** each approved import appends one record
  `{ pack, origin, deltas: { key: Δ } }`. Nothing else is written.
* **Derived value:** `peer.k = clamp(0.5 + Σ Δ over live records, 0.2,
  0.8)`, computed when it is read. A revert appends `{ revert: pack }`,
  which removes that record from the sum, so the value afterwards is
  exactly what the remaining records give, even when a value sat at a
  limit. No subtract-and-clamp is ever stored. (A later import's Δ was
  worked out against the value at its own time and is not recomputed.)
* **Computing Δ for one entry**, with `p` the current derived value:
  * an entry with `evidence < 3` is skipped and listed in the preview;
  * `β = 0.382 × min(evidence, 32) / 32`;
  * `Δ = clamp(β × (w − p), −0.05, +0.05)`;
  * **per-origin cap:** Δ is cut so that one origin key's live Δ on one
    key sums to within ±0.15;
  * **per-key cap across all peers:** Δ is cut so that `0.5 + ΣΔ`
    stays within 0.2..=0.8 (`|peer.k − 0.5| ≤ 0.3`). New keys are free to
    make, so this cap, not the per-origin one, bounds what any one
    person can do.
* **To the prompt:** only allow-listed names and numbers, for example
  "other Studio users often undo: set_layer_blend (peer lessons from 2
  trusted senders)". The count includes only senders the user trusted.
  No text from a pack reaches the model, and a pack can't add tools,
  run commands or change settings.
* **Future convergence gate (Seraphina's mesh):** only peers whose
  fingerprint the user compared and marked verified count toward it.
  Senders trusted on first use only never count.

### Glyph PNG: export and import

**v1: a private PNG chunk.** The exported file is a real 512 × 512 PNG
"glyph card", drawn with the existing text rasteriser. It shows the
fingerprint, the date, the number of lessons and a plain-words summary.
The visual glyph is a label only in v1 (pending owner confirmation, see
the open questions): the data rides in one private ancillary `seLp`
chunk, written with the `png` crate's `Writer::write_chunk` after `IHDR`
and before the image data.

Reading does not decode pixels. The file is read through `take(1 MiB +
1)`, so a file over 1 MiB is refused from the stream without reading it
all. A small chunk walker then requires:

* the PNG signature, then `IHDR` as the first chunk;
* each chunk length at most 2^31 − 1 (the PNG limit) and no longer than
  the bytes that remain, so no buffer is allocated from an untrusted
  length;
* each chunk type made of ASCII letters only (`A–Z`, `a–z`);
* each CRC correct (`crc32fast`);
* exactly one `seLp`, placed before the first `IDAT`, and no longer than
  `MAX_PACK_BYTES`;
* an `IEND` chunk, with nothing after it.

Only then is the pack parsed and verified.

* **Round trip:** export, then import, gives byte-identical pack bytes.
  `tests/fixtures/lesson_pack/glyph_a_seq1.png` is a reference file.
* **Known limit:** sites that re-encode images strip private chunks.
  The export dialog says so. Files sent as attachments or on USB keep
  the chunk.
* **Later: a visual glyph** with the data in the pixels. It needs its own
  spec (cells, colours, error correction, a capacity of at least
  `MAX_PACK_BYTES`, and JPEG round-trip tests).

### Transport

**C7.1 (v0.2.0): manual file share.** Export through a Save dialog
(`rfd`), and import through an Open dialog or by dropping a `.png` on
the window. The app binds no sockets. Gossip is user-driven: a pack you
imported can be passed on with "Share again", which is a new,
user-approved share that writes the original pack unchanged, with the
original signature (re-sharing is pending owner confirmation).

**C7.2 (the release after v0.2.0): LAN, with the Auditor's hardening.**
It has its own toggle, off by default.

* **Discovery on demand only.** Beacons are sent only while the "Find
  peers" panel is open, at most every 10 s, and for at most 2 minutes.
  Nothing is sent continuously. The setting says that your public key
  is visible to everyone on the network.
* **Beacon:** 107 bytes, signed with its own domain
  `seraphina/lan-beacon/v1\0` (see `docs/LESSON_PACK.md`), sent to UDP
  47310. A beacon only says that a peer may be there. It is never proof
  of identity.
* **The app never replies to a UDP datagram**, so it can't be used for
  amplification.
* **Order of limits:** per source IP first (1 beacon per IP per 5 s, at
  most 64 tracked IPs), then a global Ed25519 verify budget of at most 50
  per second, and only then signature checks. Anything over a limit is
  dropped and counted.
* **Signed challenge, both ways:** a TCP connection to 47311 starts
  with a 32-byte random challenge from each side. Each claimed key must
  sign the other side's challenge over `seraphina/lan-challenge/v1\0`
  (with both public keys bound in). Only then is an offer accepted or a
  pack sent. A replayed beacon from another IP gets nothing.
* **Read limit:** one frame per connection, a `u32` length that must be
  ≤ `MAX_PACK_BYTES`, then the pack. The connection reads at most `4 +
  MAX_PACK_BYTES` bytes after the challenge exchange. It has a 5 s
  timeout, at most 8 connections at once, and 1 offer per verified
  sender per 10 s. The inbox holds at most 16 packs, which are never
  applied automatically.
* **Binding:** the beacon and the listener bind to the one private
  interface the user chose (RFC 1918 or link-local), never `0.0.0.0`.
  VPN, Hyper-V, WSL and other virtual adapters are not offered and get
  no broadcasts.
* **Network profile:** on a Windows "Public" network, LAN sharing
  refuses to start and says why.
* **Firewall detection:** if the panel has been open for 30 s with no
  peer and no inbound connection, it shows "No peers seen. Windows
  Firewall may be blocking Seraphina Studio". The Windows Defender
  prompt appears at the first non-loopback listen, and if it is
  dismissed, inbound traffic is blocked silently, so this message is
  the visible state. This behaviour can't be automated, so it is a
  manual Windows check, like `signing_key_is_private` on Windows.
  Neither can run in CI today: there is no Windows runner, and the
  organisation is billing-locked.
* **LAN packs skip the PNG checks.** Their checks start at the pack
  size.

**Later:** automatic gossip (after C7.2 ships and a convergence metric
is specified; pending owner confirmation), an internet mesh, NAT
traversal, relays, a DHT, mDNS, and transport over the Seraphina Glyph
OS mesh.

### Consent

* **Off by default.** Settings → Learning has "Share lessons". In C7.2 it
  gets a "LAN sharing" option under it. Both are off. A test checks the
  defaults.
* **Each share is approved.** The preview lists every lesson in plain
  words, with its value and evidence, and you can untick lines before
  "Sign and save".
* **Each import is approved, with a preview.** It shows:
  * the origin fingerprint (always shown in full), the nickname, and
    whether the sender is new, trusted, verified or blocked;
  * every change as before → after (`peer.ai.set_layer_blend 0.50 →
    0.45`), including any change that was cut by a cap;
  * skipped entries and the reason for each.

  Then you choose Approve, Reject or Block sender.
* **Seraphina can't share or import.** These actions are not
  AppCommands, and a test checks her tool list.

### Integrity, signing, replay and limits

* **Identity:** each install has an Ed25519 keypair, generated on first
  use from `getrandom`. "New identity" makes a fresh key. The export
  dialog says that a key links your packs to each other.
* **Trust (TOFU, pending owner confirmation):** the first pack from an
  unknown key needs "Trust this sender", with the full fingerprint on
  screen. An optional "Compare fingerprints" step marks a sender
  verified. Blocked keys are refused before the preview opens.
* **Replay protection, strictly increasing:** the ledger keeps the
  highest sequence accepted from each origin key. A pack with `seq <=
  max_seq` is refused ("older than a pack already imported from this
  sender"), even after a restart. There is no window. On the sending
  side, the next sequence number is written to the ledger and fsynced
  before the pack file is written, so a crash can never reuse a
  sequence.
* **Checks, in order (each refuses on failure, never half-applies):**
  1. file size through `take(1 MiB + 1)`, for files only;
  2. PNG structure and CRCs, for files only;
  3. pack size;
  4. layout and key order;
  5. key grammar and allow-list;
  6. weight range;
  7. signature (`verify_strict`);
  8. replay;
  9. trust.
* **Import limit:** at most 20 approved imports per hour, so clicking
  through a flood can't move many keys fast.
* **Refused packs:** the ledger records the pack id and the reason. A
  copy goes to `lessons/quarantine/`, capped at the newest 32 files and
  8 MiB in total, with the oldest deleted first, so attacker files
  can't fill the disk.
* **Undo:** "Undo import" appends `{ revert: pack }`. "Block sender"
  offers to revert every live record from that origin. Compaction keeps
  the ledger, so undo still works after the log is folded. Both record
  kinds are versioned with `LESSON_VERSION`.

**Proposed crates.** These were checked on Oct 24 2026 by resolving
them into a copy of `Cargo.lock`, with no build.

| Crate | Version | MSRV | Why |
|---|---|---|---|
| `ed25519-dalek` | 2.2 (`default-features = false`, `std`, `zeroize`, `fast`) | 1.81 | keys, `verify_strict` |
| `sha2` | 0.10 | already required by `ed25519-dalek` 2.2 | pack id, fingerprint |
| `getrandom` | 0.3 (already in `Cargo.lock`) | 1.63 | key seed; C7.2 challenges |
| `png` | 0.18 (already in `Cargo.lock`) | 1.73 | write the `seLp` chunk |
| `crc32fast` | 1.5 (already in `Cargo.lock`) | 1.63 | verify chunk CRCs |

* All of them fit within MSRV 1.88.
* `cargo audit` on the resolved lock found no vulnerabilities, and the
  same 5 allowed warnings as today. These past advisories are patched:
  * RUSTSEC-2022-0093;
  * RUSTSEC-2024-0344;
  * RUSTSEC-2021-0100.
* They add 19 crates to the lock, all optional behind the `ui` feature.
* `ed25519-dalek` 3.0 (MSRV 1.85) also exists. 2.2 is proposed because
  it is the long-reviewed line.

### Privacy and key storage

* Packs hold only allow-listed keys, quantized numbers, a pseudonymous
  public key and a date rounded to the day. A test fills the learner
  with `param.*`, path-like and sentence-like inputs and checks that no
  export contains them.
* **Secret key:** the 32-byte seed is stored as a new
  `secrets::Secret::LessonSigningKey`.
  * On Windows it goes in Credential Manager as
    `SeraphinaStudio/lesson_signing_key`. Elsewhere, or if Credential
    Manager refuses, it goes in a file only the user can read (0600).
  * It is registered with `crash::register_secret`.
* **Plain files** live under `~/.seraphina/studio/lessons/`:
  * `identity.pub`;
  * `peers.json`;
  * `ledger.jsonl` (import, revert and refusal records, the highest
    sequence per origin, and this install's own next sequence);
  * `quarantine/` (capped).

  `SERAPHINA_HOME` moves them.

### Local compute first: all local, no paid calls

Everything in C7.1 runs on your own CPU:

* **Packing:** building the canonical body and writing the PNG chunk.
* **Signing and verifying:** Ed25519 with `ed25519-dalek`, which takes
  microseconds per pack.
* **Checks:** the size, layout, order and weight checks and the PNG
  chunk walk.
* **Merging:** the Δ ledger, the caps, revert and compaction.

C7.1 opens no socket and makes no network request of any kind. C7.2's
LAN transport talks only to peers on your own network, so it uses no
paid service either.

**Tests:** the tests use fixed test keys, the files in
`tests/fixtures/lesson_pack/` and temporary folders. None needs a
network or an API key, and none is billed. The test values are computed
locally by `tools/lesson_pack_vectors.py` (Python `cryptography`); see
`docs/LESSON_PACK.md`.

**Paid calls still needed: none.** One thing touches a paid call
indirectly, and it adds no call of its own: the one-line peer-lesson
summary (at most 200 characters) is added to the system prompt of a
chat turn that you are already making. Writing that line takes no model;
it is a fixed template filled in from the ledger.

### Adversarial tests (planned)

Every test uses real files, and the C7.2 tests use real sockets on
127.0.0.1, with two real installs, each with its own temporary
`SERAPHINA_HOME`. Each test must fail if its check is removed.

**C7.1 (v0.2.0):**

| Test | Proves |
|---|---|
| `pack_matches_published_vectors` | the Rust writer, given the test keys from `docs/LESSON_PACK.md`, produces the fixture bytes exactly; the reader accepts the good files and refuses each `bad_*` file for its stated reason |
| `glyph_png_round_trips_a_pack` | export then import gives identical pack bytes; the card decodes as a PNG |
| `tampered_png_is_refused` | a flipped byte in `seLp` is refused by the CRC; with the CRC recomputed, it is refused by the signature; the refusal is recorded |
| `every_single_byte_flip_is_refused` | a real pack with each byte flipped in turn is never accepted |
| `oversized_payload_is_refused` | an 8,954-byte chunk, a 2 MiB PNG (refused after reading 1 MiB + 1 bytes) and a declared length of 2^31 are refused with no large allocation |
| `truncated_chunk_is_refused` | a file cut inside `seLp`, inside `IDAT` or before `IEND` is refused |
| `png_structure_rules_are_enforced` | `IHDR` not first, a non-letter chunk type byte, two `seLp` chunks, `seLp` after `IDAT`, and bytes after `IEND` are each refused |
| `replayed_pack_is_refused` | the same file imported twice, including after a restart, is refused the second time |
| `older_sequence_is_refused` | after `a_seq2`, `a_seq1` is refused |
| `sender_sequence_survives_a_crash` | the process is killed between persisting the counter and writing the file; the next export uses a fresh, higher sequence |
| `bad_signature_is_refused` | another key's signature, a zeroed signature, a non-canonical `S` and the old domain string are refused |
| `unknown_key_needs_explicit_trust` | a valid pack from an unknown key changes nothing until it is trusted; a blocked key is refused |
| `out_of_range_weight_is_refused` | `bad_weight_10001.bin` is refused |
| `import_delta_is_clamped` | one pack moves a key by 0.05 at most; one origin stops at 0.15 |
| `many_origins_hit_the_per_key_cap` | 20 fresh keys each importing weight 1.0 leave `peer.k` at 0.8, not higher |
| `revert_is_exact_at_the_bounds` | import A pushes a key to the 0.8 cap, import B follows, A is reverted: the value equals `clamp(0.5 + Δ_B)` exactly, also after a restart and after compaction |
| `blocking_sender_reverts_its_imports` | blocking removes exactly that origin's records |
| `prompt_counts_only_trusted_senders` | untrusted or refused senders never raise the sender count |
| `non_allow_listed_key_is_refused` | `param.canvas.width`, `ai.C:/users/x`, an unknown command, an unknown mode and an upper-case key each refuse the whole pack |
| `unsorted_or_duplicate_keys_are_refused` | `bad_unsorted.bin` and a duplicate key are refused |
| `quarantine_is_capped` | 40 refused 1 MiB files leave at most 32 files and 8 MiB in total |
| `export_contains_only_new_or_changed_lessons` | a second export holds only the changes |
| `export_never_holds_private_lessons` | `param.*`, `tool.*`, `cmd.ai.*`, aggregates and invalid keys are never exported |
| `reshare_keeps_the_original_signature` | "Share again" writes the original pack bytes |
| `seraphina_cannot_share_or_import` | the AI tool list has no share or import command |
| `peer_lessons_reach_the_prompt_only_as_names_and_numbers` | the summary holds only allow-listed names and numbers |
| `nothing_is_bound_in_c7_1` | no socket is open with sharing on or off |
| `shared_learning_code_never_reaches_the_network` | a source scan: the C7.1 modules don't use `net`, `ureq`, `llm` or `videogen`, so they make no network or paid call |
| `signing_key_is_private` | Unix: the key file is 0600 and scrubbed from crash reports. Windows: it is in Credential Manager (a manual check on a Windows PC) |

**C7.2 (the release after v0.2.0):** `beacon_with_bad_signature_is_ignored`,
`replayed_beacon_from_another_ip_gets_no_pack` (the challenge fails),
`udp_datagrams_are_never_answered`, `per_ip_limit_runs_before_verify`
(a flood of 10,000 forged beacons from one IP costs at most 1 verify per
5 s), `verify_budget_caps_cpu`, `oversized_length_prefix_is_refused`
(a length of `MAX_PACK_BYTES + 1` closes the connection having read 4
bytes), `lan_flood_is_rate_limited`, `beacons_stop_when_the_panel_closes`,
`listener_binds_only_the_chosen_interface`,
`nothing_listens_while_lan_is_off`, `lan_offer_between_two_processes`.
Two more are manual Windows checks: `public_profile_refuses_lan` and
`firewall_block_shows_the_warning`.

### How Seraphina, RWN and Glyph OS plug in (later)

* **Seraphina.AGI (later)** reads and teaches lessons through
  `LessonStore`, under the same allow-list. It gets no extra rights: its
  imports go through the same preview.
* **RWN:** the `{key, weight_q, evidence}` unit maps onto RWN's
  per-source reliability with an evidence count. A later allow-list
  revision may add an `rwn.source.<id>` family. Raw model weights do not
  fit this unit.
* **Glyph OS mesh (later):** the pack is the payload unit, and its
  domain is product-neutral, so Studio and Glyph OS trade the same
  bytes.

### Decisions and open questions

Decided by the owner (Oct 24 2026): C7.1 (file packs) ships in v0.2.0,
and C7.2 (LAN) comes in the next release.

These are proposed by the Auditor and written into this spec as the
design, **pending user confirmation**:

1. **Trust:** TOFU, with the fingerprint always shown and an optional
   "Compare fingerprints" step. Only verified peers count toward any
   convergence gate.
2. **Key families:** none beyond allow-list revision 1 in v1. An
   `rwn.source.<id>` family comes when Seraphina.AGI wires up RWN.
3. **Automatic gossip:** only after C7.2 ships and a convergence metric
   is specified, tied to Seraphina's mesh milestone.
4. **Visual glyph:** a label only in v1. The data rides in the PNG
   `seLp` chunk.
5. **Re-sharing:** allowed, as a new user-approved share that keeps the
   original signature. Receivers count by origin key.

Still open: what the glyph card should look like (layout and colours).

## Principles
- Foundation first
- Math is perfect (prove it with tests)
- Preserve the public C ABI where a module ports OBS C code
- Fix the bugs in the safe path
- Delete before you add: no placeholder modules
- Every README claim is backed by a test or a measurement
- Local compute first: the CPU and local Python scripts before paid API calls

## How to contribute
Clone, run `cargo test --lib` and `cargo test --features ui --bin ui`,
open a PR.
