#!/usr/bin/env bash
# Ký lại ví dụ sau khi sửa. Chạy từ thư mục v2/.
#
# ⚠️ KHOÁ NÀY AI CŨNG CÓ. Xem examples/README.md trước khi dùng lại nó cho việc gì.
set -eu
cd "$(dirname "$0")/.."

KHOA="examples/khoa-vi-du-AI-CUNG-CO.hex"
[ -f "$KHOA" ] || {
  echo "thiếu $KHOA — sinh khoá demo mới:"
  echo "  cargo run -p tcc-cli -- key --ra $KHOA"
  exit 1
}
for goi in examples/hello-tcc examples/vi-du-vi; do
  cargo run -q -p tcc-cli -- sign "$goi" --khoa "$KHOA"
  cargo run -q -p tcc-cli -- verify "$goi"
done
