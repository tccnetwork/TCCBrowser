#!/usr/bin/env python3
"""Đối chiếu chữ ký với một bản cài đặt FIPS 204 ĐỘC LẬP.

    python3 conformance/doi-chieu-doc-lap.py [thư-mục-vector-ACVP]

# Vì sao cần cái này, khi đã có vector NIST

Vector ACVP neo được `keyGen` (25 ca) và `sigVer` (1 ca). Chiều **KÝ** thì
không: ACVP cho khoá bí mật ở dạng đã BUNG 4032 byte, còn thư viện `ml-dsa`
của dự án chỉ nạp được HẠT GIỐNG 32 byte. Không có đường ghép hai thứ đó.

Kịch bản này đi đường khác: lấy `dilithium-py` — một bản cài đặt thuần Python,
viết bởi người khác, không dùng chung một dòng mã nào với bản Rust — rồi ký
CÙNG thông điệp bằng CÙNG hạt giống và so từng byte.

# Hai bước, và bước một là bắt buộc

1. **Kiểm bản Python bằng chính vector NIST.** Không có bước này thì nó chỉ là
   một ý kiến thứ hai, và hai bản cùng sai theo một kiểu vẫn khớp nhau.
2. Ký song song rồi so byte.

Bước 2 mạnh hơn vài vector rời: nó khớp trên ĐÚNG cách dùng của dự án —
giao diện ngoài, context rỗng, ký tất định.

Cần: `pip install dilithium-py`
Vector ACVP: xem `conformance/vectors/acvp-mldsa65.json` để biết lấy ở đâu.
"""
import json
import pathlib
import sys

try:
    from dilithium_py.ml_dsa import ML_DSA_65
except ImportError:
    sys.exit("cần `pip install dilithium-py` — xem chú thích đầu tệp")

GOC = pathlib.Path(__file__).resolve().parent


def buoc_mot(thu_muc: pathlib.Path) -> bool:
    """Bản Python có thật sự đúng FIPS 204 không — hỏi NIST."""
    hoi = json.loads((thu_muc / "acvp-prompt.json").read_text())
    dap = json.loads((thu_muc / "acvp-ket.json").read_text())
    g = next(x for x in hoi["testGroups"] if x.get("parameterSet") == "ML-DSA-65")
    gk = next(x for x in dap["testGroups"] if x["tgId"] == g["tgId"])
    ket = {t["tcId"]: t for t in gk["tests"]}

    lech = [
        t["tcId"]
        for t in g["tests"]
        if ML_DSA_65.key_derive(bytes.fromhex(t["seed"]))[0].hex().upper()
        != ket[t["tcId"]]["pk"]
    ]
    n = len(g["tests"])
    if lech:
        print(f"  ✗ bản Python LỆCH NIST ở {len(lech)}/{n} ca — không dùng làm mốc được")
        return False
    print(f"  ✓ bản Python khớp NIST keyGen {n}/{n} — dùng làm mốc được")
    return True


def buoc_hai() -> bool:
    """Ký song song rồi so byte."""
    v = json.loads((GOC / "vectors" / "signature.json").read_text())
    hat_pq = bytes.fromhex(v["khoa"]["bi_mat_hex"])[32:]
    _, sk = ML_DSA_65.key_derive(hat_pq)
    pk, _ = ML_DSA_65.key_derive(hat_pq)

    # Khoá công khai trước: lệch ở đây thì so chữ ký là vô nghĩa.
    pq_cong_cua_ta = bytes.fromhex(v["khoa"]["cong_khai_hex"])[32:]
    if pk != pq_cong_cua_ta:
        print("  ✗ khoá công khai đã lệch")
        return False
    print("  ✓ khoá công khai khớp")

    tot = True
    for t in v["ky_hop_le"]:
        m = bytes.fromhex(t["thong_diep_hex"])
        cua_ta = bytes.fromhex(t["chu_ky_hex"])[64:]
        cua_py = ML_DSA_65.sign(sk, m, ctx=b"", deterministic=True)
        ok = cua_ta == cua_py
        tot &= ok
        print(f"  {'✓' if ok else '✗'} ký thông điệp {len(m):>4} byte")

    # Đòn phá nào chạm tới NỬA ML-DSA thì bản Python cũng phải từ chối.
    #
    # ⚠️ Chỉ những đòn CHẠM TỚI nửa đó. Bản đầu tôi đưa tất cả vào và kịch bản
    # đỏ ở đòn "lật một bit trong nửa Ed25519" — bản Python NHẬN, và nó ĐÚNG:
    # đòn đó không đụng nửa ML-DSA. Chữ ký lai vẫn hỏng vì nửa cổ điển hỏng.
    #
    # Đó chính là chữ ký lai làm đúng việc: phá một nửa không lan sang nửa kia.
    # Kịch bản kiểm sai, không phải mã sai.
    m = b"TCC conformance vector 0.1"
    goc_pq = None
    for t in v["ky_hop_le"]:
        if bytes.fromhex(t["thong_diep_hex"]) == m:
            goc_pq = bytes.fromhex(t["chu_ky_hex"])[64:]
            break

    bo_qua = 0
    for t in v["chu_ky_hong"]:
        ky = bytes.fromhex(t["chu_ky_hex"])
        pq = ky[64:] if len(ky) > 64 else b""
        if pq == goc_pq:
            bo_qua += 1
            continue  # đòn chỉ chạm nửa cổ điển — không phải việc của bản Python
        if ML_DSA_65.verify(pk, m, pq, ctx=b""):
            print(f"  ✗ bản Python NHẬN một chữ ký đã phá nửa ML-DSA: {t['ten']}")
            tot = False
    print(
        f"  ✓ mọi đòn chạm nửa ML-DSA đều bị bản Python từ chối "
        f"({bo_qua} đòn chỉ chạm nửa cổ điển, không xét ở đây)"
    )
    return tot


def main() -> int:
    thu_muc = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else GOC / "acvp"
    print("Bước 1 — bản Python có đúng FIPS 204 không?")
    if not (thu_muc / "acvp-prompt.json").exists():
        print(f"  … bỏ qua: không thấy vector ACVP ở {thu_muc}")
        print("  ⚠ CHƯA kiểm được bản Python. Kết quả bước 2 chỉ là 'hai bên khớp nhau',")
        print("    KHÔNG phải 'hai bên cùng đúng'.")
    elif not buoc_mot(thu_muc):
        return 1

    print()
    print("Bước 2 — hai bản cài đặt độc lập có ra cùng chữ ký không?")
    if not buoc_hai():
        return 1
    print()
    print("✓ ĐẠT — hai bản cài đặt độc lập thống nhất từng byte.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
