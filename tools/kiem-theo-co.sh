#!/usr/bin/env bash
# Chạy MỌI lệnh `cargo test`/`cargo clippy` theo cờ mà CI chạy — rút thẳng từ
# `.github/workflows/ci.yml`, không chép tay.
#
# Vì sao không chép tay: CLAUDE.md từng liệt kê bốn dòng, chép bằng tay. Tới
# 25/08/2026 danh sách ấy đã trôi ba đường:
#   • gọi `--test hai-bo-dung` — bộ thử xoá cùng WebView, lệnh chết ngay;
#   • ghi `-p tcc-shell --features window` HAI LẦN, và ghi thiếu `wallet`;
#   • bỏ hẳn `accesskit`, `import-web-wallet`, `os-keystore` mà CI vẫn chạy.
# Một danh sách chép tay thì trôi. Rút từ nguồn thì không trôi được.
#
# Lọc theo hệ điều hành: chỉ bỏ những bước CI ghi rõ `if: runner.os == 'X'` với
# X khác máy đang chạy. Bước không ghi gì thì chạy ở mọi nơi, nên chạy ở đây.
#
# Việc `linux-render`/`windows-render` là việc CỦA MỘT HỆ, nhưng lệnh của chúng
# vẫn chạy được ở đây, nên cứ chạy: nhiều hơn CI thì không sao, ÍT hơn mới đau.
set -u
cd "$(dirname "$0")/.."

case "$(uname -s)" in
  Darwin) TOI=macOS ;;
  Linux)  TOI=Linux ;;
  *)      TOI=Windows ;;
esac

lenh=$(python3 - "$TOI" <<'PY'
import re, sys
toi = sys.argv[1]
# Cắt tệp thành từng BƯỚC (`- name:`/`- run:`), giữ điều kiện `if:` của bước.
kho = open(".github/workflows/ci.yml").read().splitlines()
buoc, hien, dk = [], [], None
for d in kho:
    if re.match(r"\s*- (name|run|uses):", d):
        if hien:
            buoc.append((dk, hien))
        hien, dk = [d], None
    if hien is not None:
        hien.append(d)
        m = re.match(r"\s*if:\s*(.+)", d)
        if m:
            dk = m.group(1).strip()
if hien:
    buoc.append((dk, hien))

ra = []
for dk, than in buoc:
    if dk:
        m = re.search(r"runner\.os\s*(==|!=)\s*'(\w+)'", dk)
        if m and ((m.group(1) == "==") != (m.group(2) == toi)):
            continue
    for d in than:
        d = re.sub(r"^-\s*", "", d.strip())
        # Dạng gọn `- run: cargo …` phải cắt cả `run:`. Bản đầu không cắt, nên
        # im lặng bỏ sót cả nhóm `window` — bốn lệnh — mà vẫn báo ĐẠT.
        d = re.sub(r"^run:\s*", "", d).strip()
        # CHỈ lấy lệnh theo cờ; lệnh không cờ đã có cổng workspace lo.
        if re.match(r"^cargo (test|clippy|check) ", d) and "--features" in d:
            ra.append(d)
# Giữ thứ tự CI, bỏ trùng.
print("\n".join(dict.fromkeys(ra)))
PY
)

if [ -z "$lenh" ]; then
  echo "❌ không rút được lệnh nào từ ci.yml — bộ rút hỏng, KHÔNG phải CI sạch"
  exit 1
fi

so=$(printf '%s\n' "$lenh" | wc -l | tr -d ' ')

# `--dem`: chỉ in số lệnh rút được. `kiem-so-lieu.sh` gọi để giữ con số ghi
# trong CLAUDE.md khỏi trôi — chính hạng lỗi cổng này sinh ra để chặn.
if [ "${1:-}" = "--dem" ]; then echo "$so"; exit 0; fi
echo "── $so lệnh theo cờ, rút từ ci.yml (hệ: $TOI) ──"
hong=0
while IFS= read -r l; do
  printf '▶ %s\n' "$l"
  if ! eval "$l" >/tmp/kiem-theo-co.txt 2>&1; then
    echo "   ✗ HỎNG"
    grep -a "^error" -A 6 /tmp/kiem-theo-co.txt | head -12
    hong=$((hong + 1))
  else
    r=$(grep -ac "test result: FAILED" /tmp/kiem-theo-co.txt || true)
    if [ "$r" != "0" ]; then echo "   ✗ có phép thử đỏ"; hong=$((hong + 1)); else echo "   ✓"; fi
  fi
done <<< "$lenh"

if [ "$hong" -ne 0 ]; then
  echo "════ HỎNG: $hong lệnh ════"; exit 1
fi
echo "════ ĐẠT: $so lệnh theo cờ ════"
