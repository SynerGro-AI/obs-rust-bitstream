#!/usr/bin/env python3
"""Make the known-answer test vectors for docs/LESSON_PACK.md.

Standalone: needs only Python 3 and the `cryptography` package (Ed25519
through OpenSSL). Ed25519 signatures are deterministic (RFC 8032), so
running this again gives byte-identical files. It first checks the
library against RFC 8032 section 7.1 TEST 1, then writes
tests/fixtures/lesson_pack/ and verifies every file it wrote.

    python3 tools/lesson_pack_vectors.py           # write and verify
    python3 tools/lesson_pack_vectors.py --check   # verify only
"""
import calendar, hashlib, json, struct, sys, zlib
from pathlib import Path
from cryptography.hazmat.primitives.asymmetric.ed25519 import (
    Ed25519PrivateKey, Ed25519PublicKey)
from cryptography.hazmat.primitives.serialization import Encoding, PublicFormat
from cryptography.exceptions import InvalidSignature

PACK_DOMAIN = b"seraphina/lesson-pack/v1\0"
BEACON_DOMAIN = b"seraphina/lan-beacon/v1\0"
CHALLENGE_DOMAIN = b"seraphina/lan-challenge/v1\0"
MAX_ENTRIES, MAX_KEY = 128, 64
MAX_PACK_BYTES = 57 + MAX_ENTRIES * (1 + MAX_KEY + 2 + 2) + 64  # 8953
OUT = Path(__file__).resolve().parent.parent / "tests/fixtures/lesson_pack"

def key(seed): return Ed25519PrivateKey.from_private_bytes(seed)
def pub(sk): return sk.public_key().public_bytes(Encoding.Raw, PublicFormat.Raw)

def rfc8032_self_check():
    sk = key(bytes.fromhex("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60"))
    assert pub(sk).hex() == "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a"
    assert sk.sign(b"").hex() == ("e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555"
                                  "fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b")

def body(pk, seq, created, entries, version=1, flags=0, allow_rev=1):
    b = b"SSLP" + struct.pack("<BBH", version, flags, allow_rev) + pk
    b += struct.pack("<QQB", seq, created - created % 86400, len(entries))
    for k, wq, ev in entries:
        kb = k.encode("ascii"); b += struct.pack("<B", len(kb)) + kb + struct.pack("<HH", wq, ev)
    return b

def pack(sk, seq, created, entries, domain=PACK_DOMAIN):
    b = body(pub(sk), seq, created, entries)
    return b + sk.sign(domain + b)

def verify(p):
    """The reader's checks that need no allow-list or state: size, layout,
    order, weight range, then the signature. Returns None or the reason."""
    if len(p) > MAX_PACK_BYTES or len(p) < 57 + 64: return "size"
    if p[:4] != b"SSLP" or p[4] != 1 or p[5] != 0: return "layout"
    n = p[56]
    if not 1 <= n <= MAX_ENTRIES: return "layout"
    i, prev = 57, None
    for _ in range(n):
        if i >= len(p) - 64: return "layout"
        kl = p[i]
        if not 1 <= kl <= MAX_KEY or i + 1 + kl + 4 > len(p) - 64: return "layout"
        k = p[i + 1:i + 1 + kl]
        if prev is not None and k <= prev: return "order"
        wq, _ev = struct.unpack_from("<HH", p, i + 1 + kl)
        if wq > 10000: return "weight"
        prev, i = k, i + 1 + kl + 4
    if i != len(p) - 64: return "layout"
    try: Ed25519PublicKey.from_public_bytes(p[8:40]).verify(p[-64:], PACK_DOMAIN + p[:-64])
    except InvalidSignature: return "signature"
    return None

def chunk(t, data): return struct.pack(">I", len(data)) + t + data + struct.pack(">I", zlib.crc32(t + data))

def glyph_png(p):
    """An 8x8 grey PNG with the pack in one seLp chunk right after IHDR."""
    ihdr = struct.pack(">IIBBBBB", 8, 8, 8, 0, 0, 0, 0)
    raw = b"".join(b"\0" + b"\x80" * 8 for _ in range(8))
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr) + chunk(b"seLp", p)
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))

def beacon(sk, port, day):
    b = b"SSLB" + bytes([1]) + pub(sk) + struct.pack("<HI", port, day)
    return b + sk.sign(BEACON_DOMAIN + b)

def build():
    rfc8032_self_check()
    a = key(bytes.fromhex("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60"))
    b = key(hashlib.sha256(b"seraphina lesson pack test key B").digest())
    t = calendar.timegm((2026, 10, 24, 0, 0, 0))
    files = {}
    files["a_seq1.bin"] = pack(a, 1, t, [("ai.set_layer_blend", 2500, 12),
                                         ("cmd.user.paint.brush_stroke", 6180, 340)])
    files["a_seq2.bin"] = pack(a, 2, t + 86400, [("ai.set_layer_blend", 2310, 15)])
    files["b_seq1.bin"] = pack(b, 1, t, [("ai.add_layer", 9000, 32)])
    good = files["a_seq1.bin"]
    flipped = bytearray(good); flipped[76] ^= 0x01  # low byte of the first weight_q
    files["bad_flipped_byte.bin"] = bytes(flipped)
    files["bad_wrong_domain.bin"] = pack(a, 1, t, [("ai.set_layer_blend", 2500, 12)],
                                         domain=b"seraphina-studio/lesson-pack/v1\0")
    files["bad_unsorted.bin"] = pack(a, 3, t, [("cmd.user.paint.brush_stroke", 6180, 340),
                                               ("ai.set_layer_blend", 2500, 12)])
    files["bad_weight_10001.bin"] = pack(a, 4, t, [("ai.set_layer_blend", 10001, 12)])
    files["glyph_a_seq1.png"] = glyph_png(good)
    files["beacon_a.bin"] = beacon(a, 47311, t // 86400)
    return a, b, files

def expectations(files):
    out = {}
    for name, data in sorted(files.items()):
        e = {"bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}
        if name.endswith(".bin") and not name.startswith("beacon"):
            e["verify"] = verify(data) or "ok"
        out[name] = e
    return out

def main():
    a, b, files = build()
    want = {"a_seq1.bin": "ok", "a_seq2.bin": "ok", "b_seq1.bin": "ok",
            "bad_flipped_byte.bin": "signature", "bad_wrong_domain.bin": "signature",
            "bad_unsorted.bin": "order", "bad_weight_10001.bin": "weight"}
    exp = expectations(files)
    for n, w in want.items(): assert exp[n]["verify"] == w, (n, exp[n])
    bc = files["beacon_a.bin"]; assert len(bc) == 107
    Ed25519PublicKey.from_public_bytes(bc[5:37]).verify(bc[-64:], BEACON_DOMAIN + bc[:-64])
    meta = {"generator": "tools/lesson_pack_vectors.py", "pack_domain": PACK_DOMAIN.decode().rstrip("\0") + "\\0",
            "beacon_domain": BEACON_DOMAIN.decode().rstrip("\0") + "\\0",
            "max_pack_bytes": MAX_PACK_BYTES,
            "key_a": {"seed": "RFC 8032 7.1 TEST 1 secret key", "public": pub(a).hex(),
                      "fingerprint": hashlib.sha256(pub(a)).hexdigest()[:32]},
            "key_b": {"seed": "sha256(b'seraphina lesson pack test key B')", "public": pub(b).hex(),
                      "fingerprint": hashlib.sha256(pub(b)).hexdigest()[:32]},
            "files": exp}
    if "--check" in sys.argv:
        old = json.loads((OUT / "vectors.json").read_text())
        assert old == meta, "vectors.json differs from a fresh computation"
        for n, d in files.items(): assert (OUT / n).read_bytes() == d, n
        print("all vectors match"); return
    OUT.mkdir(parents=True, exist_ok=True)
    for n, d in files.items(): (OUT / n).write_bytes(d)
    (OUT / "vectors.json").write_text(json.dumps(meta, indent=2) + "\n")
    print(json.dumps(meta, indent=2))

if __name__ == "__main__":
    main()
