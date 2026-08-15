#!/usr/bin/env python3
"""Dựng một gói TCC hợp lệ bằng PYTHON, rồi bắt bản Rust kiểm nó.

    python3 conformance/dung-goi-doc-lap.py

# Vì sao cần cái này, khi đã có 136 vector

Vector kiểm **bộ phân tích**: đưa dữ liệu vào, xem nhận hay từ chối. Chúng
không kiểm được chiều ngược lại — rằng một bên khác, viết bằng ngôn ngữ khác,
đọc đặc tả rồi **DỰNG** ra được thứ bản này chấp nhận.

Đó mới là câu hỏi của tính liên thông. Một bản cài đặt từ chối đúng mọi thứ
sai mà không tạo nổi một gói đúng thì vẫn không trao đổi gói được với ai.

Kịch bản này dựng cả gói từ con số không, chỉ dùng những gì `spec/0.1/` viết:

    dạng chuẩn tắc → băm nội dung → bản kê khai → ký lai → thư mục gói

rồi gọi `tcc verify`. Nếu bản Rust nhận, hai bên đồng ý về TOÀN BỘ định dạng,
không chỉ về số học của ML-DSA.

# Nó độc lập tới đâu, nói cho đúng

- **Ed25519**: viết thẳng theo RFC 8032 trong tệp này, không dùng thư viện.
- **ML-DSA-65**: `dilithium-py`, người khác viết, không chung dòng mã nào.
- **BLAKE3**: `blake3` từ PyPI — bản Rust dùng crate `blake3`, nên hai bên có
  thể chung một gốc thuật toán. Đây là chỗ YẾU nhất của phép thử; nhóm vector
  `canonical` neo nó bằng KAT công khai của chuỗi rỗng.
- **Dạng chuẩn tắc, bố cục byte, quy tắc bản kê khai**: viết lại từ đặc tả.

Tôi viết cả hai bên, nên đây KHÔNG phải bản cài đặt độc lập thật theo nghĩa
`spec/GOVERNANCE.md` §3 đòi. Nó bắt được sự bất đồng giữa hai lần đọc đặc tả;
nó không bắt được chỗ tôi hiểu sai đặc tả ở cả hai lần.

Cần: pip install dilithium-py blake3
"""

import hashlib
import json
import pathlib
import shutil
import subprocess
import sys
import tempfile

# ───────────────────────── Ed25519, RFC 8032 ─────────────────────────
# Viết thẳng ở đây thay vì gọi thư viện: nửa cổ điển là nửa dễ viết, và tự viết
# thì phép thử này thật sự có hai bản cài đặt chứ không phải hai lời gọi tới
# cùng một thư viện.

P = 2**255 - 19
L = 2**252 + 27742317777372353535851937790883648493
D = -121665 * pow(121666, P - 2, P) % P
I = pow(2, (P - 1) // 4, P)


def _inv(x):
    return pow(x, P - 2, P)


def _recover_x(y, sign):
    xx = (y * y - 1) * _inv(D * y * y + 1)
    x = pow(xx, (P + 3) // 8, P)
    if (x * x - xx) % P != 0:
        x = x * I % P
    if x % 2 != sign:
        x = P - x
    return x


BY = 4 * _inv(5) % P
BX = _recover_x(BY, 0)
B = (BX, BY, 1, BX * BY % P)


def _add(p, q):
    """Cộng điểm trên đường cong xoắn Edwards, toạ độ mở rộng.

    Chép đúng công thức của RFC 8032 §6. Bản đầu tôi tự sắp lại thứ tự toạ độ
    trả về và nó sai ngay từ khoá công khai — vector TEST 1 bắt được lập tức.
    Đó là lý do phép thử này neo vào RFC trước khi neo bất cứ thứ gì khác.
    """
    a, b, c, d = p
    e, f, g, h = q
    A = (b - a) * (f - e) % P
    Bb = (b + a) * (f + e) % P
    C = 2 * d * h * D % P
    Dd = 2 * c * g % P
    E, F, G, H = Bb - A, Dd - C, Dd + C, Bb + A
    return (E * F % P, G * H % P, F * G % P, E * H % P)


def _mul(s, p):
    q = (0, 1, 1, 0)
    while s > 0:
        if s & 1:
            q = _add(q, p)
        p = _add(p, p)
        s >>= 1
    return q


def _encode(p):
    x, y, z, _ = p
    zi = _inv(z)
    x, y = x * zi % P, y * zi % P
    return int.to_bytes(y | ((x & 1) << 255), 32, "little")


def _h(m):
    return hashlib.sha512(m).digest()


def _secret_scalar(seed):
    h = _h(seed)
    a = int.from_bytes(h[:32], "little")
    a &= (1 << 254) - 8
    a |= 1 << 254
    return a, h[32:]


def ed25519_public(seed: bytes) -> bytes:
    a, _ = _secret_scalar(seed)
    return _encode(_mul(a, B))


def ed25519_sign(seed: bytes, msg: bytes) -> bytes:
    a, prefix = _secret_scalar(seed)
    pub = _encode(_mul(a, B))
    r = int.from_bytes(_h(prefix + msg), "little") % L
    Rp = _encode(_mul(r, B))
    k = int.from_bytes(_h(Rp + pub + msg), "little") % L
    s = (r + k * a) % L
    return Rp + int.to_bytes(s, 32, "little")


# ───────────────────── Dạng chuẩn tắc, theo 01-package.md ─────────────────────


def canonical_bytes(files: dict[str, bytes]) -> bytes:
    """u64 độ dài đường dẫn (BE) ‖ đường dẫn ‖ u64 độ dài nội dung (BE) ‖ nội dung.

    Sắp theo thứ tự BYTE của đường dẫn, không theo thứ tự chèn.
    """
    ra = bytearray()
    for path in sorted(files, key=lambda p: p.encode()):
        p = path.encode()
        ra += len(p).to_bytes(8, "big") + p
        ra += len(files[path]).to_bytes(8, "big") + files[path]
    return bytes(ra)


def content_hash_hex(canon: bytes) -> str:
    """BLAKE3 ở chế độ XOF, lấy 48 byte đầu, hex chữ thường."""
    import blake3

    return blake3.blake3(canon).digest(length=48).hex()


def main() -> int:
    try:
        from dilithium_py.ml_dsa import ML_DSA_65
    except ImportError:
        print("✗ cần: pip install dilithium-py blake3")
        return 1

    print("Dựng một gói TCC bằng Python, rồi bắt bản Rust kiểm nó.\n")

    # 1 — khoá. Bí mật lai = [hạt giống Ed25519 32B][hạt giống ML-DSA 32B].
    seed_ed = bytes(range(32))
    seed_pq = bytes(range(128, 160))
    pub_ed = ed25519_public(seed_ed)
    pub_pq, sec_pq = ML_DSA_65.key_derive(seed_pq)
    publisher = (pub_ed + pub_pq).hex()
    print(f"  khoá công khai lai : {len(pub_ed) + len(pub_pq)} byte")
    assert len(pub_ed) + len(pub_pq) == 1984, "bố cục khoá công khai sai"

    # 2 — nội dung, và dạng chuẩn tắc của nó.
    ui = json.dumps(
        {"kind": "text", "content": "Chào từ Python"}, ensure_ascii=False
    ).encode()
    files = {"ui.json": ui}
    bam = content_hash_hex(canonical_bytes(files))
    print(f"  băm nội dung       : {bam[:16]}…")

    # 3 — bản kê khai. Chữ ký ký lên ĐÚNG chuỗi byte được ghi ra đĩa, nên phải
    #     tuần tự hoá MỘT LẦN rồi dùng lại chính chuỗi đó.
    manifest = {
        "spec_version": "0.1",
        "id": "com.tcc.python",
        "name": "Gói dựng bằng Python",
        "version": "1.0.0",
        "publisher": publisher,
        "scheme": "hybrid-ed25519-mldsa65-v1",
        "content_hash": bam,
        "entry": "ui.json",
        "capabilities": [],
    }
    manifest_bytes = json.dumps(manifest, ensure_ascii=False, indent=2).encode()

    # 4 — chữ ký lai. Giao diện NGOÀI của FIPS 204 với ctx RỖNG, đúng 03.
    sig_ed = ed25519_sign(seed_ed, manifest_bytes)
    sig_pq = ML_DSA_65.sign(sec_pq, manifest_bytes, ctx=b"", deterministic=True)
    signature = sig_ed + sig_pq
    print(f"  chữ ký lai         : {len(signature)} byte")
    assert len(signature) == 3373, "bố cục chữ ký sai"

    # 5 — ghi ra thư mục gói và gọi bản Rust.
    tmp = pathlib.Path(tempfile.mkdtemp(prefix="tcc-python-"))
    goi = tmp / "goi"
    (goi / "content").mkdir(parents=True)
    (goi / "manifest.json").write_bytes(manifest_bytes)
    (goi / "signature.hex").write_text(signature.hex())
    (goi / "content" / "ui.json").write_bytes(ui)

    print(f"\n  gọi: tcc verify {goi}\n")
    r = subprocess.run(
        ["cargo", "run", "-q", "-p", "tcc-cli", "--", "verify", str(goi)],
        cwd=pathlib.Path(__file__).resolve().parent.parent,
        capture_output=True,
        text=True,
    )
    print("\n".join("    " + x for x in (r.stdout + r.stderr).strip().split("\n")))
    shutil.rmtree(tmp, ignore_errors=True)

    if r.returncode == 0:
        print(
            "\n✓ ĐẠT — một gói dựng hoàn toàn bằng Python được bản Rust chấp nhận.\n"
            "  Hai bên đồng ý về dạng chuẩn tắc, băm nội dung, bố cục byte của\n"
            "  khoá và chữ ký, và giao diện FIPS 204 — không chỉ về số học ML-DSA."
        )
        return 0
    print("\n✗ TRƯỢT — hai bên bất đồng ở đâu đó trong định dạng gói.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
