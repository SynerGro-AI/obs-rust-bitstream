# Lesson pack format, version 1 (planned, not built)

Status: specification only. No Rust reader or writer exists yet. It is
part of shared learning (C7 in PLAN.md). This file is the format
contract for every program that trades lesson packs: Seraphina Studio,
Seraphina.AGI, RWN and Glyph OS. It covers the bytes only. What a
receiver does with the numbers (Studio's bounded import rule, RWN's
log-odds fusion) is receiver policy and lives with each receiver.

The test values below are real. `tools/lesson_pack_vectors.py` computed
them with Python's `cryptography` package (Ed25519 through OpenSSL). That
script first checks the library against RFC 8032 section 7.1 TEST 1.
Ed25519 signatures are deterministic, so `python3
tools/lesson_pack_vectors.py --check` recomputes every file and compares
it byte for byte with `tests/fixtures/lesson_pack/`. Run it to check.
`--check` also fails on any file in that directory that is not in the
table below, and on any number in this file that differs from the
computed one: the test-value table, the domain byte counts and the
revision 2 pack sizes. Every failure exits 1 and names the file.

## Layout

All integers are little-endian. One meaning has exactly one encoding.

| Offset | Size | Field | Rule |
|---|---|---|---|
| 0 | 4 | magic | ASCII `SSLP` |
| 4 | 1 | format version | `1`; any other value is refused |
| 5 | 1 | flags | `0`; any other value is refused |
| 6 | 2 | allow-list revision | `u16`, the sender's key allow-list revision |
| 8 | 32 | origin public key | Ed25519 |
| 40 | 8 | sequence | `u64`, 1 for the origin key's first pack, then strictly increasing |
| 48 | 8 | created | `u64` Unix seconds, a multiple of 86400 (rounded down to the UTC day); any other value is refused (`created`) |
| 56 | 1 | entry count | 1..=128 |
| 57 | n | entries | see below |
| end − 64 | 64 | signature | Ed25519 (RFC 8032, pure) over the domain followed by bytes `0..end−64` |

Entry: `key_len: u8` (1..=64), `key` (ASCII), `weight_q: u16`
(0..=10000; weight = `weight_q / 10000`), `evidence: u16` (event count,
saturating at 65535). Keys are sorted strictly by their bytes, with no
duplicates.

* **Signing domain:** `seraphina/lesson-pack/v1\0`: 25 bytes including the trailing NUL
  (the 24 ASCII bytes `seraphina/lesson-pack/v1`, then one zero byte).
  It is product-neutral on purpose, so any Seraphina program can sign
  and verify packs. A new layout gets a new version number and a new
  domain.
* **`MAX_PACK_BYTES` = 8,953** (57 + 128 × 69 + 64). The reader checks
  the size before parsing and refuses trailing bytes.
* **Strict signature checks.** Every reader must refuse, before it
  accepts a signature, each of these. The reason names are the ones the
  vector script and the fixtures use.
  1. `noncanonical_key`: the origin key's `y` is ≥ p (2^255 − 19), or
     `x` is 0 with the sign bit set. Without this, one key would have
     more than one byte form and could dodge the replay ledger and the
     block list, which are keyed by the key bytes.
     `bad_noncanonical_key.bin` uses `y` = p + 1, a second byte form of
     the identity point. That key is also small order, so the fixture
     shows that check 1 runs before check 2. A forgeable non-canonical
     key that is not small order does not exist: `y` − p is 0 to 18, and
     a signature can be made without the private key only when `[k]A`
     vanishes.
  2. `small_order_key`: the origin key is one of the 8 small-order
     points (`[8]A` is the identity). With such a key one signature
     verifies for any content. `bad_small_order_key.bin` uses the
     identity point `01 00…00`.
  3. `noncanonical_r` / `small_order_r`: the same two checks on `R`,
     the first 32 signature bytes.
  4. `noncanonical_s`: `S`, the last 32 signature bytes as a
     little-endian integer, is ≥ L (2^252 +
     27742317777372353535851937790883648493).
     `bad_noncanonical_s.bin` is `a_seq2.bin` with `S` replaced by
     `S + L`.
  5. `signature`: the RFC 8032 equation `[S]B = R + [k]A`, with `R`
     compared as bytes.

  In Rust, `ed25519-dalek` 2.x `VerifyingKey::verify_strict` covers 2,
  the small-order part of 3, 4 and 5. The reader must still check 1
  explicitly (a range check on the key bytes before
  `VerifyingKey::from_bytes`), and `pack_matches_published_vectors`
  must show that the three strict-only fixtures are refused for their
  stated reason. A plain `verify` is not enough.

  Why the explicit checks: the plain equation **holds** for all three
  strict-only fixtures (the script computes it, reducing `y` mod p as a
  permissive decoder does, and fails if it ever does not), so only the
  strict checks refuse them. Python's `cryptography` 43.0.0 with OpenSSL
  3.5.7 (measured on Oct 24 2026) **accepts** `bad_small_order_key.bin`
  and `bad_noncanonical_key.bin`, and refuses `bad_noncanonical_s.bin`. So OpenSSL is not a strict verifier, and
  `tools/lesson_pack_vectors.py` makes every strict check itself with
  plain integer curve arithmetic before it calls OpenSSL.
* **There is no nonce.** The pair (origin key, sequence) is unique, and
  the signature is deterministic, so the same content always gives the
  same bytes.
* **Pack id:** SHA-256 of the whole pack, signature included.
* **Fingerprint:** the first 16 bytes of SHA-256(public key), shown as 8
  groups of 4 hex digits.
* **Key grammar:** set by the allow-list revisions below. A pack with
  any key outside the revisions the receiver knows is refused whole.
* **Re-sharing:** a relayed pack is forwarded unchanged, with its
  original origin key, sequence and signature. A receiver counts it by
  origin key, never by the relayer. Because the sequence belongs to the
  origin, a newer relayed pack makes an older direct pack a replay:
  once the relayed pack with sequence N+1 from key X has been imported,
  the direct pack N from X is refused as "older than a pack already
  imported from this sender". This is intended (one ledger per origin
  key), and the preview says so. Test:
  `relayed_newer_pack_makes_older_direct_pack_a_replay` (PLAN.md, C7).

## Allow-list revisions

This section is the only place where allow-list revisions are defined.
PLAN.md (C7) and Seraphina.AGI's RWN spec refer to it and do not
restate the families. A new revision is added here first. Revisions only
add families, so a receiver at revision N accepts packs at revisions
1..=N. A receiver refuses a pack whose header revision is higher than
its own, and refuses the whole pack if any key is outside its families.

Each family has an **import mode**:

* `blend`: the receiver's bounded import into `peer.*` (Studio's rule
  is in PLAN.md, C7).
* `observe`: the receiver stores the sender's latest value per key and
  changes no weight.

| Revision | Status | Family | Wire grammar | Mode | `weight_q` | `evidence` |
|---|---|---|---|---|---|---|
| 1 | v0.2.0 (C7.1) | AI command acceptance | `^ai\.[a-z0-9_]+$` | blend | the sender's keep rate for that command | event count |
| 1 | v0.2.0 (C7.1) | user command share | `^cmd\.user\.[a-z]+\.[a-z0-9_]+$` | blend | the command's share of the sender's commands in that mode | event count |
| 2 | planned (Seraphina.AGI RWN milestone, after v0.2.0) | RWN probe verdict | `^rwn\.probe\.(local\|local_lite)\.[0-9]{3}$` | observe | 10000 if the sender's active policy returns `ACCEPT` on probe item `nnn`, else 0 | the policy's `n_examples` (not an event count) |
| 2 | planned (Seraphina.AGI RWN milestone, after v0.2.0) | RWN threshold | `^rwn\.(local\|local_lite)\.threshold$` | observe | `round(accept_threshold × 10000)`, status display only | the policy's `n_examples` |

* **Revision 1:** the command and mode names must also be in the
  receiver's compiled allow-list (`Mode::ALL` and the AppCommand names
  in `docs/AI_TOOLS.md`). `ai.acceptance` is an aggregate and stays off
  the list.
* **Revision 2 (planned):** an RWN pack must be complete: all 101 keys
  of one profile (`nnn` = 000 to 099 plus the threshold). That makes
  2,545 bytes for `local` and 3,050 bytes for `local_lite`, both under
  `MAX_PACK_BYTES` (computed by the vector script). `evidence` means
  `n_examples` for these families, which is fine because `observe`
  never moves a weight. A revision 1 receiver, which is Studio in
  v0.2.0, refuses these packs whole on the unknown family. This is the
  intended separation. Revision 2 has no test vectors yet; they are
  added with the RWN milestone.
* **Not in revision 2:** Seraphina.AGI's proposed
  `rwn.<profile>.sha.<nn>` family (`nn` = 00 to 15, the policy body
  hash) is planned and is not part of revision 2. Revision 2 stays at
  101 keys per profile, with the 2,545 and 3,050 byte pack sizes above.
  Adding the family needs a new revision defined here,
  with its own sizes.
* **Reserved:** `rwn.source.<id>` (per-source reliability, `blend`) is
  reserved and in no revision. Fusing peer reliabilities into a fit
  needs a new RWN API.

## Glyph PNG carrier

The pack rides in exactly one private ancillary chunk of type `seLp`
(ancillary, private, reserved bit clear, safe to copy). The chunk comes
after `IHDR` and before the first `IDAT`, and its data is the pack bytes
unchanged. The checks a reader makes are listed in PLAN.md (C7, "Glyph
PNG").

## LAN beacon and challenge (for C7.2, the release after v0.2.0)

* **Beacon:** 107 bytes, made of `SSLB`, the version `1`, the public key
  (32 bytes), the TCP port as a `u16`, the UTC day number as a `u32`, and
  then the signature (64 bytes). It is signed over the domain
  `seraphina/lan-beacon/v1\0`: 24 bytes including the trailing NUL (23
  ASCII bytes, then one zero byte), followed by the first 43 bytes.
* **Challenge:** a 32-byte random value from the side that accepted the
  connection. The claimed key signs it over the domain
  `seraphina/lan-challenge/v1\0`: 27 bytes including the trailing NUL (26
  ASCII bytes, then one zero byte), followed by the challenge and both
  public keys (the challenger's first). Each side challenges the other.
  The same strict signature checks apply to beacons and challenges.
* There are no beacon-challenge test values yet. They are added with
  C7.2. `beacon_a.bin` below is a real signed beacon.

## Test keys

| Key | Seed | Public key | Fingerprint |
|---|---|---|---|
| A | RFC 8032 7.1 TEST 1 secret key `9d61b19d…1cae7f60` | `d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a` | `21fe31dfa154a261626bf854046fd227` |
| B | SHA-256 of the ASCII bytes `seraphina lesson pack test key B` | `bb38357aebd196813245bea26dbb716156166806ddc1bc6833a5923e8fcbf076` | `605224003b887110efb42f07932ea59b` |

These seeds are public test values. Never use them for a real install.

## Test values

`created` is 2026-10-24T00:00:00Z (1792800000). The files are in
`tests/fixtures/lesson_pack/`.

| File | Bytes | SHA-256 | Expected result |
|---|---|---|---|
| `a_seq1.bin` | 176 | `baadb2d68dfa5f7d9ba86aaeb61c3b010822c5bdcba830c5d3eea8dacf1a02eb` | ok |
| `a_seq2.bin` | 144 | `c1ae73f41713f8ae4729e8e90f8fce660c265cb73c9636d1224e3b90a84552da` | ok |
| `b_seq1.bin` | 138 | `2c9212c4008e17c1f171d9926d87d47ca435292d0cd5bd95a905ce3fab0187a3` | ok |
| `bad_created_not_whole_day.bin` | 144 | `e8b0cef4f566a4751ad975b041e16b773a43ac664a4ad1695a985a9415691e5d` | created |
| `bad_flipped_byte.bin` | 176 | `d926da4db202ee5bc617497a3350fa70c2d38139982ad685f917264953be5b6e` | signature |
| `bad_noncanonical_key.bin` | 144 | `0eda20d2a868532e0736bbafaedf5b55ab3ca4c2857fdd2638649f45b7c2ce1e` | noncanonical_key |
| `bad_noncanonical_s.bin` | 144 | `3efb992094229fa47e7acc4cff5c75b94572cfbb6a0f80f515973f24c9a8be3c` | noncanonical_s |
| `bad_small_order_key.bin` | 144 | `4f6aefdbe64ec721d93a7acbb62669d613585bfa36dd61819597181f599b449b` | small_order_key |
| `bad_unsorted.bin` | 176 | `de5b1bb9ec06a2e0e0f3d9ccfe86ae94db2178e49b420e5f520994c473654fc0` | order |
| `bad_weight_10001.bin` | 144 | `7fad56f5dba81ad98cc65d44093a1531b32107c8189f22903c69610fee49537c` | weight |
| `bad_wrong_domain.bin` | 144 | `ebe23a11a0103782d07987808e5508fcc371236ed0826787d98d44322ed04db4` | signature |
| `beacon_a.bin` | 107 | `21a2e35b6e9a2c29b69d3deeee4d8961d4c231bb9ebf0a002c3a43bbf5390cfe` | see below |
| `glyph_a_seq1.png` | 259 | `33903b814cb576934967ba4a84e7564134e3d8cb027b7d3a0585839db79bb590` | see below |

* `a_seq1.bin`: key A, sequence 1, with `ai.set_layer_blend` 2500/12
  and `cmd.user.paint.brush_stroke` 6180/340.
* `a_seq2.bin`: key A, sequence 2, created one day later, with
  `ai.set_layer_blend` 2310/15.
* `b_seq1.bin`: key B, sequence 1, with `ai.add_layer` 9000/32.
* `bad_flipped_byte.bin`: `a_seq1.bin` with bit 0 of byte 76 (the first
  `weight_q`) flipped, so the signature fails.
* `bad_wrong_domain.bin`: a pack signed over the old
  `seraphina-studio/lesson-pack/v1\0` domain, so the signature fails.
* `bad_unsorted.bin`: correctly signed but with its keys out of order,
  so it is refused before the signature check.
* `bad_weight_10001.bin`: correctly signed with `weight_q` = 10001, so it
  is refused.
* `bad_small_order_key.bin`: origin key = the identity point
  `0100…00` (small order), with `R = [S]B` for a fixed `S` (`S` =
  SHA-512 of `seraphina small-order key vector`, mod L). The plain
  equation holds for any content, and OpenSSL accepts it. Refused
  `small_order_key`.
* `bad_noncanonical_key.bin`: origin key bytes = p + 1 as 32
  little-endian bytes (`eeff…ff7f`), a non-canonical encoding of the
  identity point, with `R = [S]B` for a fixed `S` (`S` = SHA-512 of
  `seraphina noncanonical key vector`, mod L). A decoder that reduces
  `y` mod p sees the identity, so the plain equation holds for any
  content, and OpenSSL accepts it. Refused `noncanonical_key`.
* `bad_noncanonical_s.bin`: `a_seq2.bin` with `S` replaced by `S + L`.
  The plain equation still holds. Refused `noncanonical_s`.
* `bad_created_not_whole_day.bin`: key A, sequence 5, correctly signed,
  with `created` = 1792803600 (01:00 UTC, not a whole day). Refused
  `created`.
* `glyph_a_seq1.png`: an 8 × 8 grey PNG carrying `a_seq1.bin` in `seLp`
  right after `IHDR`.
* `beacon_a.bin`: key A's beacon for port 47311 on day 20750.

`a_seq1.bin` in hex:

```
53534c5001000100d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325
af021a68f707511a010000000000000000f5db6a00000000021261692e736574
5f6c617965725f626c656e64c4090c001b636d642e757365722e7061696e742e
62727573685f7374726f6b65241854014255251a970ad911c3154384d9c30b90
cac4d2bc33e3aed6155e37acc6393d355d5a8bb478178fec84d8ea883997ca8e
775fb2ea87459eec02d19d4a1f1dca00
```

`a_seq2.bin` in hex:

```
53534c5001000100d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325
af021a68f707511a02000000000000008046dd6a00000000011261692e736574
5f6c617965725f626c656e6406090f00c0ad9919d703c3e63bc4dbfd6de0aaa5
ac61570bafa5ff7c9f24f44ba867156357531f5cb1abeeecc32aa0f592355752
90d1c8cb275f7e41321be4bb04a52302
```

`b_seq1.bin` in hex:

```
53534c5001000100bb38357aebd196813245bea26dbb716156166806ddc1bc68
33a5923e8fcbf076010000000000000000f5db6a00000000010c61692e616464
5f6c61796572282320003e012e4dc40cca79c4db3ada947237b8b74e18bcddb2
7fe0eab638070df828e772437374cf3dd930b38c964d0b503bc408e9fc1ec177
20258071e4a9c0198002
```

`beacon_a.bin` in hex:

```
53534c4201d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a
68f707511acfb80e51000058bd56d8bf9b6eca3994a0ec1ce43e73cc019e885f
897e5f4c2b6204cfacf18db7293cc976ab93b095199ad7177219843cea30ba87
8a5ebfb1454b39eafa1305
```
