#!/usr/bin/env python3
"""Verify a 3GS registry signature without the Rust toolchain.

Usage:
  python3 scripts/verify_registry.py registry.json
  curl -s https://api.3gs.ai/registry > r.json && python3 scripts/verify_registry.py r.json

Checks the embedded Ed25519 signature against curator.pubkey. Pass --pubkey <z32>
to also require a specific curator key (recommended: the key you already trust).
Requires: pip install cryptography
"""
import json, sys
from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey

ZB32 = "ybndrfg8ejkmcpqxot1uwisza345h769"

def zbase32_decode(s):
    n = 0
    for c in s:
        n = n * 32 + ZB32.index(c)
    return (n >> (len(s) * 5 - 256)).to_bytes(32, "big")

args = sys.argv[1:]
expected = None
if "--pubkey" in args:
    i = args.index("--pubkey")
    expected = args[i + 1]
    del args[i : i + 2]
if len(args) != 1:
    sys.exit(__doc__)

doc = json.load(open(args[0], encoding="utf-8"))
if expected and doc["curator"]["pubkey"] != expected:
    sys.exit(f"FAIL: curator.pubkey is {doc['curator']['pubkey']}, expected {expected}")
if "signature" not in doc:
    sys.exit("FAIL: registry has no signature")
sig = doc.pop("signature")
if sig.get("alg") != "ed25519":
    sys.exit(f"FAIL: unsupported alg {sig.get('alg')!r}")
message = b"3gs-registry-v1\n" + json.dumps(
    doc, sort_keys=True, separators=(",", ":"), ensure_ascii=False
).encode()
key = Ed25519PublicKey.from_public_bytes(zbase32_decode(doc["curator"]["pubkey"]))
try:
    key.verify(bytes.fromhex(sig["sig"]), message)
except InvalidSignature:
    sys.exit("FAIL: signature does not verify against curator.pubkey")
print("OK: signed by", doc["curator"]["pubkey"])
