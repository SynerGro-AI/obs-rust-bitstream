#!/usr/bin/env python3
"""Make the known-answer test vectors for docs/LESSON_PACK.md.

Standalone: needs only Python 3 and the `cryptography` package (Ed25519
through OpenSSL). Ed25519 signatures are deterministic (RFC 8032), so
running this again gives byte-identical files. It first checks the
library against RFC 8032 section 7.1 TEST 1, then writes
tests/fixtures/lesson_pack/ and verifies every file it wrote.

OpenSSL's Ed25519 verify is not `verify_strict`, so the strict checks
(canonical S < L, canonical key and R encodings, no small-order key or
R) are done here explicitly with plain integer curve arithmetic. A
permissive RFC 8032 equation check is also computed, to prove that the
two strict-only bad fixtures really satisfy the plain equation and are
refused only by the strict checks.

--check also fails on any extra file in the fixture directory and on
any drift between the computed values and docs/LESSON_PACK.md (the
test-value table and the domain byte counts).

    python3 tools/lesson_pack_vectors.py           # write and verify
    python3 tools/lesson_pack_vectors.py --check   # verify only
"""
import calendar, hashlib, json, struct, sys, zlib
from pathlib import Path
from cryptography.hazmat.primitives.asymmetric.ed25519 import (
    Ed25519PrivateKey, Ed25519PublicKey)
from cryptography.hazmat.primitives.serialization import Encoding, PublicFormat
from cryptography.exceptions import InvalidSignature

PACK_DOMAIN = b"seraphina/lesson-pack/v1\0"         # 25 bytes with the NUL
BEACON_DOMAIN = b"seraphina/lan-beacon/v1\0"         # 24 bytes with the NUL
CHALLENGE_DOMAIN = b"seraphina/lan-challenge/v1\0"   # 27 bytes with the NUL
DOMAINS = {"pack_domain": PACK_DOMAIN, "beacon_domain": BEACON_DOMAIN,
           "challenge_domain": CHALLENGE_DOMAIN}
MAX_ENTRIES, MAX_KEY = 128, 64
MAX_PACK_BYTES = 57 + MAX_ENTRIES * (1 + MAX_KEY + 2 + 2) + 64  # 8953
ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "tests/fixtures/lesson_pack"
DOC = ROOT / "docs/LESSON_PACK.md"
DAY = 86400

# Ed25519 curve constants (RFC 8032 section 5.1).
P = 2**255 - 19
L = 2**252 + 27742317777372353535851937790883648493
D = -121665 * pow(121666, P - 2, P) % P
SQRT_M1 = pow(2, (P - 1) // 4, P)
IDENT = (0, 1, 1, 0)

def fail(msg):
    """Explicit failure that `python -O` cannot strip, unlike assert."""
    print(f"FAIL: {msg}", file=sys.stderr); sys.exit(1)

def pt_add(a, b):
    """Extended-coordinate point addition (RFC 8032 section 5.1.4)."""
    x1, y1, z1, t1 = a; x2, y2, z2, t2 = b
    A = (y1 - x1) * (y2 - x2) % P; B = (y1 + x1) * (y2 + x2) % P
    C = 2 * t1 * t2 * D % P; Dd = 2 * z1 * z2 % P
    E, F, G, H = B - A, Dd - C, Dd + C, B + A
    return (E * F % P, G * H % P, F * G % P, E * H % P)

def pt_mul(k, a):
    r = IDENT
    while k:
        if k & 1: r = pt_add(r, a)
        a, k = pt_add(a, a), k >> 1
    return r

def pt_eq(a, b):
    return (a[0] * b[2] - b[0] * a[2]) % P == 0 and (a[1] * b[2] - b[1] * a[2]) % P == 0

def pt_encode(a):
    zi = pow(a[2], P - 2, P); x, y = a[0] * zi % P, a[1] * zi % P
    return (y | (x & 1) << 255).to_bytes(32, "little")

def pt_decode(s):
    """Returns (point, reason). reason is None, or why the encoding is refused:
    'noncanonical' (y >= p, or x = 0 with the sign bit set) or 'not_on_curve'."""
    v = int.from_bytes(s, "little"); y, sign = v & (2**255 - 1), v >> 255
    if y >= P: return None, "noncanonical"
    x2 = (y * y - 1) * pow(D * y * y + 1, P - 2, P) % P
    x = pow(x2, (P + 3) // 8, P)
    if (x * x - x2) % P: x = x * SQRT_M1 % P
    if (x * x - x2) % P: return None, "not_on_curve"
    if x == 0 and sign: return None, "noncanonical"
    if x & 1 != sign: x = P - x
    return (x, y, 1, x * y % P), None

BASE = pt_decode((4 * pow(5, P - 2, P) % P).to_bytes(32, "little"))[0]

def small_order(pt): return pt_eq(pt_mul(8, pt), IDENT)

def strict_reason(pk, sig):
    """The checks `verify_strict` adds on top of the plain equation, done
    explicitly because OpenSSL does not do all of them. Returns None or
    the reason."""
    a, why = pt_decode(pk)
    if why: return f"{why}_key"
    if small_order(a): return "small_order_key"
    r, why = pt_decode(sig[:32])
    if why: return f"{why}_r"
    if small_order(r): return "small_order_r"
    if int.from_bytes(sig[32:], "little") >= L: return "noncanonical_s"
    return None

def lax_equation_holds(pk, sig, msg):
    """Permissive RFC 8032 check: [S]B == R + [k]A with no S < L and no
    small-order test. Used only to prove what the strict checks catch."""
    a, _ = pt_decode(pk); r, _ = pt_decode(sig[:32])
    if a is None or r is None: return False
    k = int.from_bytes(hashlib.sha512(sig[:32] + pk + msg).digest(), "little") % L
    return pt_eq(pt_mul(int.from_bytes(sig[32:], "little"), BASE), pt_add(r, pt_mul(k, a)))

def strict_self_check():
    """The curve code must agree with the library and know the 8 small-order points."""
    sk = key(bytes.fromhex("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60"))
    a = pt_mul(int.from_bytes(hashlib.sha512(bytes.fromhex(
        "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60")).digest()[:32], "little")
        & ~7 & (2**254 - 1) | 2**254, BASE)
    if pt_encode(a) != pub(sk): fail("curve self-check: [s]B does not match RFC 8032 TEST 1 key")
    if not lax_equation_holds(pub(sk), sk.sign(b""), b""):
        fail("curve self-check: lax equation rejects RFC 8032 TEST 1 signature")
    torsion = {pt_encode(pt_mul(i, pt_decode(bytes.fromhex(
        "c7176a703d4dd84fba3c0b760d10670f2a2053fa2c39ccc64ec7fd7792ac037a"))[0])) for i in range(8)}
    if len(torsion) != 8 or not all(small_order(pt_decode(t)[0]) for t in torsion):
        fail("curve self-check: the order-8 point does not generate 8 small-order points")
    if pt_decode((P + 1).to_bytes(32, "little"))[1] != "noncanonical":
        fail("curve self-check: y = p + 1 is not refused as non-canonical")

def key(seed): return Ed25519PrivateKey.from_private_bytes(seed)
def pub(sk): return sk.public_key().public_bytes(Encoding.Raw, PublicFormat.Raw)

def rfc8032_self_check():
    sk = key(bytes.fromhex("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60"))
    if pub(sk).hex() != "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a":
        fail("RFC 8032 7.1 TEST 1 public key mismatch")
    if sk.sign(b"").hex() != ("e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555"
                              "fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b"):
        fail("RFC 8032 7.1 TEST 1 signature mismatch")

def body(pk, seq, created, entries, version=1, flags=0, allow_rev=1, round_day=True):
    b = b"SSLP" + struct.pack("<BBH", version, flags, allow_rev) + pk
    b += struct.pack("<QQB", seq, created - created % DAY if round_day else created, len(entries))
    for k, wq, ev in entries:
        kb = k.encode("ascii"); b += struct.pack("<B", len(kb)) + kb + struct.pack("<HH", wq, ev)
    return b

def pack(sk, seq, created, entries, domain=PACK_DOMAIN, round_day=True):
    b = body(pub(sk), seq, created, entries, round_day=round_day)
    return b + sk.sign(domain + b)

def small_order_key_pack(seq, created, entries):
    """A pack whose origin key is the identity point (encoding 01 00..00, a
    small-order point). [k]A is then the identity for every message, so
    R = [S]B with any S < L satisfies the plain equation: a permissive
    verifier accepts this one signature for any content."""
    b = body((1).to_bytes(32, "little"), seq, created, entries)
    s = int.from_bytes(hashlib.sha512(b"seraphina small-order key vector").digest(), "little") % L
    return b + pt_encode(pt_mul(s, BASE)) + s.to_bytes(32, "little")

def noncanonical_s(p):
    """The same pack with S replaced by S + L: the plain equation still
    holds (S is only used mod L), but S >= L is not canonical."""
    s = int.from_bytes(p[-32:], "little") + L
    if s >= 2**256: fail("S + L does not fit in 32 bytes")
    return p[:-32] + s.to_bytes(32, "little")

def verify(p):
    """The reader's checks that need no allow-list or state: size, layout,
    created on a whole UTC day, order, weight range, the strict signature
    checks, then the signature. Returns None or the reason."""
    if len(p) > MAX_PACK_BYTES or len(p) < 57 + 64: return "size"
    if p[:4] != b"SSLP" or p[4] != 1 or p[5] != 0: return "layout"
    if struct.unpack_from("<Q", p, 48)[0] % DAY: return "created"
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
    why = strict_reason(p[8:40], p[-64:])
    if why: return why
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
    strict_self_check()
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
    files["bad_small_order_key.bin"] = small_order_key_pack(1, t, [("ai.set_layer_blend", 2500, 12)])
    files["bad_noncanonical_s.bin"] = noncanonical_s(files["a_seq2.bin"])
    files["bad_created_not_whole_day.bin"] = pack(a, 5, t + 3600, [("ai.set_layer_blend", 2500, 12)],
                                                  round_day=False)
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

def rwn_pack_bytes(profile):
    """Size of a complete allow-list revision 2 RWN pack (100 probe keys and
    one threshold key), as stated in docs/LESSON_PACK.md."""
    keys = [f"rwn.probe.{profile}.{n:03d}" for n in range(100)] + [f"rwn.{profile}.threshold"]
    return 57 + sum(1 + len(k) + 4 for k in keys) + 64

def doc_checks(meta):
    """docs/LESSON_PACK.md must state the same numbers this script computes."""
    if not DOC.is_file(): fail(f"{DOC}: missing")
    doc = DOC.read_text(encoding="utf-8")
    for name, dom in DOMAINS.items():
        phrase = f"`{dom[:-1].decode('ascii')}\\0`: {len(dom)} bytes including the trailing NUL"
        if phrase not in doc: fail(f"{DOC}: missing the exact domain count '{phrase}'")
    for prof in ("local", "local_lite"):
        phrase = f"{rwn_pack_bytes(prof):,} bytes for `{prof}`"
        if phrase not in doc: fail(f"{DOC}: missing the computed revision 2 size '{phrase}'")
    rows = {}
    for line in doc.splitlines():
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) == 4 and cells[0].startswith("`") and cells[2].startswith("`") and len(cells[2]) == 66:
            rows[cells[0].strip("`")] = (cells[1], cells[2].strip("`"), cells[3])
    if set(rows) != set(meta["files"]):
        fail(f"{DOC}: test-value table lists {sorted(rows)}, expected {sorted(meta['files'])}")
    for n, e in meta["files"].items():
        if rows[n][0] != str(e["bytes"]) or rows[n][1] != e["sha256"]:
            fail(f"{DOC}: row for {n} has {rows[n][0]} bytes / {rows[n][1]}, computed {e['bytes']} / {e['sha256']}")
        if "verify" in e and rows[n][2] != e["verify"]:
            fail(f"{DOC}: row for {n} says '{rows[n][2]}', computed '{e['verify']}'")

def main():
    a, b, files = build()
    want = {"a_seq1.bin": "ok", "a_seq2.bin": "ok", "b_seq1.bin": "ok",
            "bad_flipped_byte.bin": "signature", "bad_wrong_domain.bin": "signature",
            "bad_unsorted.bin": "order", "bad_weight_10001.bin": "weight",
            "bad_small_order_key.bin": "small_order_key", "bad_noncanonical_s.bin": "noncanonical_s",
            "bad_created_not_whole_day.bin": "created"}
    exp = expectations(files)
    for n, w in want.items():
        if exp[n]["verify"] != w: fail(f"{n}: verify gave {exp[n]['verify']}, expected {w}")
    packs = sorted(n for n in exp if "verify" in exp[n])
    if packs != sorted(want):
        fail(f"pack fixtures {packs} do not match the expected results {sorted(want)}")
    for n in ("bad_small_order_key.bin", "bad_noncanonical_s.bin"):
        p = files[n]
        if not lax_equation_holds(p[8:40], p[-64:], PACK_DOMAIN + p[:-64]):
            fail(f"{n}: the plain RFC 8032 equation does not hold, so it does not prove the strict check")
    for name, dom, n in (("pack_domain", PACK_DOMAIN, 25), ("beacon_domain", BEACON_DOMAIN, 24),
                         ("challenge_domain", CHALLENGE_DOMAIN, 27)):
        if len(dom) != n or dom[-1:] != b"\0" or b"\0" in dom[:-1]:
            fail(f"{name}: {len(dom)} bytes, expected {n} including exactly one trailing NUL")
    if rwn_pack_bytes("local") > MAX_PACK_BYTES or rwn_pack_bytes("local_lite") > MAX_PACK_BYTES:
        fail("a complete revision 2 RWN pack would exceed MAX_PACK_BYTES")
    bc = files["beacon_a.bin"]
    if len(bc) != 107: fail(f"beacon_a.bin: {len(bc)} bytes, expected 107")
    try: Ed25519PublicKey.from_public_bytes(bc[5:37]).verify(bc[-64:], BEACON_DOMAIN + bc[:-64])
    except InvalidSignature: fail("beacon_a.bin: signature does not verify")
    meta = {"generator": "tools/lesson_pack_vectors.py", "pack_domain": PACK_DOMAIN.decode().rstrip("\0") + "\\0",
            "beacon_domain": BEACON_DOMAIN.decode().rstrip("\0") + "\\0",
            "challenge_domain": CHALLENGE_DOMAIN.decode().rstrip("\0") + "\\0",
            "domain_bytes_with_nul": {k: len(v) for k, v in DOMAINS.items()},
            "max_pack_bytes": MAX_PACK_BYTES,
            "rwn_rev2_pack_bytes": {p: rwn_pack_bytes(p) for p in ("local", "local_lite")},
            "key_a": {"seed": "RFC 8032 7.1 TEST 1 secret key", "public": pub(a).hex(),
                      "fingerprint": hashlib.sha256(pub(a)).hexdigest()[:32]},
            "key_b": {"seed": "sha256(b'seraphina lesson pack test key B')", "public": pub(b).hex(),
                      "fingerprint": hashlib.sha256(pub(b)).hexdigest()[:32]},
            "files": exp}
    if "--check" in sys.argv:
        vj = OUT / "vectors.json"
        if not vj.is_file(): fail(f"{vj}: missing")
        if json.loads(vj.read_text(encoding="utf-8")) != meta: fail(f"{vj}: differs from a fresh computation")
        extra = sorted(f.name for f in OUT.iterdir() if f.name not in files and f.name != "vectors.json")
        if extra: fail(f"{OUT}: files not in the vector table: {', '.join(extra)}")
        for n, d in files.items():
            f = OUT / n
            if not f.is_file(): fail(f"{f}: missing")
            if f.read_bytes() != d: fail(f"{f}: differs from a fresh computation")
        doc_checks(meta)
        print("all vectors match"); return
    OUT.mkdir(parents=True, exist_ok=True)
    for n, d in files.items(): (OUT / n).write_bytes(d)
    (OUT / "vectors.json").write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps(meta, indent=2))

if __name__ == "__main__":
    main()
