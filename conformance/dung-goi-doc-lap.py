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

rồi gọi `tcc verify`. Và làm cả CHIỀU NGƯỢC: đọc gói do bản Rust ký, kiểm
bằng Python. Chiều ngược quan trọng ngang chiều xuôi — không có gì chứng minh
thứ `tcc sign` xuất ra là ĐỌC ĐƯỢC bởi ai khác, và một bản cài đặt sinh ra gói
mà chỉ chính nó đọc nổi vẫn qua được mọi phép thử của chính nó.

Nếu cả hai chiều đạt, hai bên đồng ý về TOÀN BỘ định dạng, không chỉ về số học
của ML-DSA.

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


def _decode(b: bytes):
    y = int.from_bytes(b, "little")
    sign = y >> 255
    y &= (1 << 255) - 1
    if y >= P:
        return None
    x = _recover_x(y, sign)
    if (x * x - (y * y - 1) * _inv(D * y * y + 1)) % P != 0:
        return None
    return (x, y, 1, x * y % P)


def _equal(p, q):
    if (p[0] * q[2] - q[0] * p[2]) % P != 0:
        return False
    return (p[1] * q[2] - q[1] * p[2]) % P == 0


def ed25519_verify(pub: bytes, msg: bytes, sig: bytes) -> bool:
    """Kiểm chữ ký Ed25519. Cần cho chiều NGƯỢC: Python đọc gói bản Rust dựng."""
    if len(sig) != 64 or len(pub) != 32:
        return False
    A = _decode(pub)
    Rp = _decode(sig[:32])
    if A is None or Rp is None:
        return False
    s = int.from_bytes(sig[32:], "little")
    if s >= L:
        return False
    k = int.from_bytes(_h(sig[:32] + pub + msg), "little") % L
    return _equal(_mul(s, B), _add(Rp, _mul(k, A)))


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


def kiem_goi_bang_python(goi: pathlib.Path) -> tuple[bool, str]:
    """Kiểm một gói do bản RUST dựng, hoàn toàn bằng Python.

    Chiều này quan trọng ngang chiều kia và chưa ai kiểm: không có gì chứng
    minh thứ `tcc sign` xuất ra là ĐỌC ĐƯỢC bởi ai khác. Một bản cài đặt sinh
    ra gói mà chỉ chính nó đọc nổi vẫn qua được mọi phép thử của chính nó.

    Theo đúng thứ tự của `01-package.md`: chữ ký TRƯỚC, rồi mới tin bản kê khai.
    """
    from dilithium_py.ml_dsa import ML_DSA_65

    ke_khai_byte = (goi / "manifest.json").read_bytes()
    chu_ky = bytes.fromhex((goi / "signature.hex").read_text().strip())
    if len(chu_ky) != 3373:
        return False, f"chữ ký {len(chu_ky)} byte, phải 3373"

    ke_khai = json.loads(ke_khai_byte)
    pub = bytes.fromhex(ke_khai["publisher"])
    if len(pub) != 1984:
        return False, f"khoá công khai {len(pub)} byte, phải 1984"

    # 1 — nửa cổ điển, ký lên BYTE THÔ của bản kê khai.
    if not ed25519_verify(pub[:32], ke_khai_byte, chu_ky[:64]):
        return False, "nửa Ed25519 không hợp lệ"

    # 2 — nửa hậu lượng tử. Giao diện NGOÀI, ctx RỖNG.
    if not ML_DSA_65.verify(pub[32:], ke_khai_byte, chu_ky[64:], ctx=b""):
        return False, "nửa ML-DSA-65 không hợp lệ"

    # 3 — CHỈ TỚI ĐÂY bản kê khai mới đáng tin. Giờ mới so nội dung với nó.
    tep: dict[str, bytes] = {}
    goc = goi / "content"
    for d in sorted(goc.rglob("*")):
        if d.is_file():
            tep[str(d.relative_to(goc)).replace("\\", "/")] = d.read_bytes()
    bam = content_hash_hex(canonical_bytes(tep))
    if bam != ke_khai["content_hash"]:
        return False, f"băm nội dung lệch: {bam[:16]}… ≠ {ke_khai['content_hash'][:16]}…"
    if ke_khai["entry"] not in tep:
        return False, f"điểm vào {ke_khai['entry']!r} không có trong gói"

    return True, f"{len(tep)} tệp, điểm vào {ke_khai['entry']}"


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

    if r.returncode != 0:
        print("\n✗ TRƯỢT — bản Rust từ chối gói do Python dựng.")
        return 1

    # ── Chiều ngược: Python đọc gói do bản RUST dựng ──
    print("\n── chiều ngược: Python kiểm gói do bản Rust ký ──\n")
    vi_du = pathlib.Path(__file__).resolve().parent.parent / "examples" / "hello-tcc"
    dat, ly_do = kiem_goi_bang_python(vi_du)
    print(f"    {'✓' if dat else '✗'} examples/hello-tcc — {ly_do}")
    if not dat:
        print("\n✗ TRƯỢT — Python không đọc nổi gói do bản Rust dựng.")
        return 1

    if True:
        print(
            "\n✓ ĐẠT CẢ HAI CHIỀU.\n"
            "  Python dựng → Rust nhận, và Rust ký → Python kiểm được.\n"
            "  Hai bên đồng ý về dạng chuẩn tắc, băm nội dung, bố cục byte của\n"
            "  khoá và chữ ký, và giao diện FIPS 204 — không chỉ số học ML-DSA."
        )
        return 0
    return 1


if __name__ == "__main__":
    sys.exit(main())
