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

[ "$loi" = 0 ] && echo "════ ĐẠT ════" || echo "════ HỎNG: $loi ════"
exit "$loi"
