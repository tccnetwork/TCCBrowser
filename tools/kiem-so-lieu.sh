#!/usr/bin/env bash
# Số phép thử và số vector ghi trong tài liệu phải khớp số THẬT.
#
# Vì sao không nằm trong `kiem-luat-phu-thuoc.sh`: kịch bản ấy chạy TRƯỚC bước
# dựng, nên nó không biết hai con số này. Đếm mò thì sai — bản thử đếm `cases`
# trong JSON ra 122 trong khi bộ kiểm định báo 138.
#
# Người soát độc lập bắt được đúng chỗ này ngày 16/08/2026 (phát hiện F3):
# SECURITY.md ghi 234 phép thử / 135 vector trong khi thật là 238 / 136, còn
# docs/AUDIT.md thì đúng. Đây là loại trôi mà chính dự án đã chứng minh nhiều
# lần: cái gì không có máy canh thì trôi.
set -uo pipefail
cd "$(dirname "$0")/.."
loi=0
bao() { echo "❌ $*"; loi=$((loi + 1)); }
dat() { echo "✅ $*"; }

that_test=$(cargo test --workspace 2>&1 \
  | grep -E '^test result: ok' | sed 's/test result: ok. //;s/ passed.*//' | paste -sd+ - | bc)
that_vec=$(cargo run -q -p tcc-conformance 2>&1 \
  | grep -E '^TỔNG' | awk '{print $2}')

# Chỉ soi con số đứng SAU chính cái lệnh sinh ra nó:
#
#     cargo test --workspace        # 290 tests
#     cargo run -p tcc-conformance  # 138 conformance vectors
#
# Cố ý hẹp. Bản đầu quét mọi cụm "N phép thử" trong tài liệu và nó đòi sửa
# "211 phép thử mù hoàn toàn" — một sự thật LỊCH SỬ, sửa đi là bóp méo hồ sơ.
# Con số duy nhất phải đúng-ngay-hôm-nay là con số người soát chạy lệnh để đối
# chiếu, và nó luôn nằm ngay sau lệnh ấy.
kiem() {  # $1 = lệnh, $2 = số thật, $3 = tên để in
  local lech=""
  for f in README.md SECURITY.md ARCHITECTURE.md CLAUDE.md docs/*.md; do
    [ -f "$f" ] || continue
    # `--features` bị loại: đó là một lệnh KHÁC, số của nó khác ba đơn vị.
    for n in $(grep -F "$1" "$f" | grep -v -- '--features' | grep -oE '# *[0-9]+' | grep -oE '[0-9]+' | sort -u); do
      [ "$n" = "$2" ] || lech="$lech $(basename "$f"):$n"
    done
  done
  if [ -n "$lech" ]; then
    bao "tài liệu ghi sai $3 (thật là $2):$lech"
  else
    dat "$2 $3, mọi tài liệu nhắc tới đều ghi đúng"
  fi
}

kiem 'cargo test --workspace ' "$that_test" "phép thử"
kiem 'cargo run -p tcc-conformance' "$that_vec" "vector"

# ── FORMAT.md: mỗi tệp vector một mục, và con số ở tiêu đề phải ĐÚNG ──────────
#
# `FORMAT.md` là thứ DUY NHẤT người cài đặt bên ngoài đọc để dùng được bộ vector.
# Số sai ở đó là số sai ngay cửa vào.
#
# Nó đã trôi, và trôi lâu: đo ngày 23/08/2026 thì **4 trong 7** con số sai
# (`signature` 15 vs 9, `acvp` 26 vs 25, `manifest` 31 vs 34, `ui` 17 vs 27), và
# hai tệp — `package.json`, `verify.json` — **không có mục nào cả**. Không phép
# canh nào kêu, vì phép canh ở trên cố ý chỉ soi con số đứng sau một LỆNH.
#
# Đếm mọi mảng ở mức ngoài cùng trừ `notes` (mảng ấy là văn xuôi). Một tệp có
# nhiều mảng ca — `package.json` có ba — thì tiêu đề ghi TỔNG.
lech_format=$(python3 - <<'PY2'
import json, re, pathlib
s = open("conformance/FORMAT.md").read()
ghi = {t: int(n) for t, n in re.findall(r"^## `([a-z0-9-]+\.json)` — (\d+) cases", s, re.M)}
ra = []
for p in sorted(pathlib.Path("conformance/vectors").glob("*.json")):
    d = json.load(open(p))
    that = sum(len(v) for k, v in d.items() if isinstance(v, list) and k != "notes")
    if p.name not in ghi:
        ra.append(f"{p.name}:KHÔNG-CÓ-MỤC")
    elif ghi[p.name] != that:
        ra.append(f"{p.name}:ghi-{ghi[p.name]}-thật-{that}")
for ten in ghi:
    if not (pathlib.Path("conformance/vectors") / ten).exists():
        ra.append(f"{ten}:MỤC-THỪA")
print(" ".join(ra))
PY2
)
if [ -n "$lech_format" ]; then
  bao "conformance/FORMAT.md lệch với vectors/:$lech_format"
else
  dat "FORMAT.md mô tả đủ mọi tệp vector, và mọi con số đều đúng"
fi

[ "$loi" = 0 ] && echo "════ ĐẠT ════" || echo "════ HỎNG: $loi ════"
exit "$loi"
