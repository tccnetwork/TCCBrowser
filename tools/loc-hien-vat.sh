#!/usr/bin/env bash
# Tách "kẻ sống sót" THẬT khỏi HIỆN VẬT của cấu hình.
#
#   tools/loc-hien-vat.sh <hòm> "<cờ>" <đường dẫn missed.txt>
#   tools/loc-hien-vat.sh tcc-shell "" /tmp/dot-bien-toi-thieu/tcc-shell/mutants.out/missed.txt
#
# ⚠️ VÌ SAO CẦN: `cargo-mutants` sinh đột biến từ TỆP NGUỒN, không phải từ mã đã
# biên dịch. Tệp nằm sau một `#[cfg(feature = …)]` đang TẮT thì đột biến ở đó
# không có tác dụng gì, phép thử xanh, và công cụ ghi `MISSED`.
#
# Giá đã trả, 26–27/08/2026, hai lần:
#   • `tcc-shell` quét với MỘT cờ: `wallet_flow` (đòi cả ba cờ) không được biên
#     dịch → 18 dòng MISSED không có thật. Tôi đã báo cáo một "lỗ" dựng trên đó
#     rồi phải rút lại.
#   • `tcc-shell` quét ở cấu hình TỐI THIỂU: **53 trên 62** kẻ sống là hiện vật
#     — `window_raster` 25, `wallet_flow` 18, `import_screen` 10. Tức 85% nhiễu.
#     Chỉ 9 kẻ là thật, và một trong chín là lỗ an ninh có thật.
#
# Cách đo: dựng vào một thư mục đích SẠCH rồi đọc tệp `.d` (dep-info) rustc sinh
# ra — chúng liệt kê đúng từng tệp `.rs` ĐÃ ĐƯỢC ĐỌC. Không đoán `cfg`, không
# đoán nền. Cùng kỹ thuật với `tools/dem-unsafe.sh`, và cùng lý do: mọi cách đo
# gián tiếp đều phồng.
set -u
cd "$(dirname "$0")/.."

if [ "$#" -lt 3 ]; then
  sed -n '3,6p' "$0" | sed 's/^# \{0,1\}//'
  exit 1
fi
HOM=$1; CO=$2; MISSED=$3

[ -f "$MISSED" ] || { echo "✗ không đọc được $MISSED"; exit 1; }

DICH=$(mktemp -d)
trap 'rm -rf "$DICH"' EXIT

# ⚠️ Phải là `cargo test --no-run`, KHÔNG phải `cargo build`. Trọng tài của
# `cargo-mutants` là lượt chạy PHÉP THỬ, và nó dựng nhiều hơn một bản dựng
# thường: 27/08/2026 đo được `window_raster.rs` KHÔNG có trong dep-info của
# `cargo build -p tcc-shell`, nhưng CÓ trong `cargo test --no-run -p tcc-shell`.
#
# Chênh ấy không nhỏ: tôi từng phân loại tay 25 kẻ sống ở `window_raster` là
# "hiện vật vì nằm sau cờ `window`" — suy từ dòng `#[cfg]` chứ không đo. Sai.
# Chúng được biên dịch thật, chỉ là không phép thử nào chạm tới. Đó là hạng
# KHÁC HẲN: không phải phép đo hỏng, mà là mã chưa ai thử.
lenh=(cargo test -q --no-run -p "$HOM" --target-dir "$DICH")
[ -n "$CO" ] && lenh+=(--features "$CO")
if ! "${lenh[@]}" 2>/dev/null; then
  echo "✗ không dựng được $HOM${CO:+ --features $CO} — chưa đo được tệp nào biên dịch"
  exit 1
fi

python3 - "$DICH" "$HOM" "$MISSED" <<'PY'
import pathlib, sys, collections

dich, hom, missed = pathlib.Path(sys.argv[1]), sys.argv[2], sys.argv[3]

# Mọi tệp .rs xuất hiện trong dep-info = mọi tệp THẬT SỰ được biên dịch.
da_dung = set()
for d in dich.rglob("*.d"):
    for t in d.read_text(errors="replace").replace(":", " ").split():
        if t.endswith(".rs"):
            da_dung.add(pathlib.Path(t).resolve())

if not da_dung:
    sys.exit("❌ tệp .d không liệt kê nguồn nào — bộ đọc hỏng, KHÔNG phải cây sạch")

that, hien_vat = [], collections.Counter()
for dong in pathlib.Path(missed).read_text(errors="replace").splitlines():
    if not dong.strip():
        continue
    tep = dong.split(":", 1)[0]
    if pathlib.Path(tep).resolve() in da_dung:
        that.append(dong)
    else:
        hien_vat[tep] += 1

tong = len(that) + sum(hien_vat.values())
print(f"── {hom}: {tong} kẻ sống sót ──")
if hien_vat:
    print(f"\n⊘ HIỆN VẬT — {sum(hien_vat.values())} kẻ, trong tệp KHÔNG được biên dịch ở cấu hình này:")
    for tep, n in hien_vat.most_common():
        print(f"   {n:3d}  {tep}")
    print("   Đây KHÔNG phải phép thử yếu. Đột biến ở đó không có tác dụng gì.")
print(f"\n✓ THẬT — {len(that)} kẻ, trong tệp đã biên dịch:")
for dong in that:
    print(f"   {dong}")
if tong:
    print(f"\nTỷ lệ nhiễu: {sum(hien_vat.values()) * 100 // tong}%")
PY
