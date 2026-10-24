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
| 48 | 8 | created | `u64` Unix seconds, a multiple of 86400 (rounded down to the UTC day) |
| 56 | 1 | entry count | 1..=128 |
| 57 | n | entries | see below |
| end − 64 | 64 | signature | Ed25519 (RFC 8032, pure) over the domain followed by bytes `0..end−64` |

Entry: `key_len: u8` (1..=64), `key` (ASCII), `weight_q: u16`
(0..=10000; weight = `weight_q / 10000`), `evidence: u16` (event count,
saturating at 65535). Keys are sorted strictly by their bytes, with no
duplicates.

* **Signing domain:** the 25 bytes `seraphina/lesson-pack/v1` followed by
  one zero byte. It is product-neutral on purpose, so any Seraphina
  program can sign and verify packs. A new layout gets a new version
  number and a new domain.
* **`MAX_PACK_BYTES` = 8,953** (57 + 128 × 69 + 64). The reader checks
  the size before parsing, refuses trailing bytes, and verifies with
  `verify_strict` semantics (it refuses small-order keys and
  non-canonical `S`).
* **There is no nonce.** The pair (origin key, sequence) is unique, and
  the signature is deterministic, so the same content always gives the
  same bytes.
* **Pack id:** SHA-256 of the whole pack, signature included.
* **Fingerprint:** the first 16 bytes of SHA-256(public key), shown as 8
  groups of 4 hex digits.
* **Key grammar (allow-list revision 1):**
  `^ai\.[a-z0-9_]+$` or `^cmd\.user\.[a-z]+\.[a-z0-9_]+$`, where the
  command and mode names must also be in the receiver's allow-list.
  Revision 1 holds no other families.
* **Re-sharing:** a relayed pack is forwarded unchanged, with its
  original origin key, sequence and signature. A receiver counts it by
  origin key, never by the relayer.

## Glyph PNG carrier

The pack rides in exactly one private ancillary chunk of type `seLp`
(ancillary, private, reserved bit clear, safe to copy). The chunk comes
after `IHDR` and before the first `IDAT`, and its data is the pack bytes
unchanged. The checks a reader makes are listed in PLAN.md (C7, "Glyph
PNG").

## LAN beacon and challenge (for C7.2, the release after v0.2.0)

* **Beacon:** 107 bytes, made of `SSLB`, the version `1`, the public key
  (32 bytes), the TCP port as a `u16`, the UTC day number as a `u32`, and
  then the signature (64 bytes). It is signed over the 24 bytes
  `seraphina/lan-beacon/v1` and one zero byte, followed by the first 43
  bytes.
* **Challenge:** a 32-byte random value from the side that accepted the
  connection. The claimed key signs it over the domain
  `seraphina/lan-challenge/v1` and one zero byte, followed by the
  challenge and both public keys (the challenger's first). Each side
  challenges the other.
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
| `bad_flipped_byte.bin` | 176 | `d926da4db202ee5bc617497a3350fa70c2d38139982ad685f917264953be5b6e` | signature |
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
