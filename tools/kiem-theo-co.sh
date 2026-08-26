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
# Điều khiển việc: mỗi việc nền thành một NHÓM tiến trình riêng, để hết giờ thì
# giết được cả nhóm. Giết mỗi `cargo` thì nhị phân phép thử con vẫn treo lại.
set -m
cd "$(dirname "$0")/.."

# Hết giờ cho MỘT lệnh. Không có lệnh `timeout` trên macOS mặc định.
#
# ⚠️ Vì sao cần: 26/08/2026 cổng này treo hơn bốn mươi phút ở
# `cargo test -p tcc-keystore --features os-keystore`. Phép thử ấy ghi một mục
# THẬT vào Keychain rồi gọi `unlock`, và macOS bật hộp thoại xin quyền — cổng
# đứng chờ một cú bấm chuột không bao giờ tới. Một cổng treo vô hạn là một cổng
# người ta sẽ bỏ qua, và một cổng bị bỏ qua thì không phải cổng.
HAN_GIO=${HAN_GIO:-600}
# Mỗi lệnh giữ đầu ra RIÊNG. Bản đầu cho cả hai mươi lệnh ghi đè chung một tệp,
# nên khi lệnh thứ mười một đỏ thì đầu ra của nó đã bị chín lệnh sau xoá — cổng
# báo đỏ mà không nói được đỏ vì gì. Cổng như thế chỉ dạy người ta chạy lại cho
# tới lúc xanh, tức là dạy đúng thói quen nó sinh ra để chặn.
NOI_RA=${NOI_RA:-/tmp/kiem-theo-co}
rm -rf "$NOI_RA"; mkdir -p "$NOI_RA"
chay_han_gio() {
  eval "$1" >"$RA" 2>&1 &
  local pid=$!
  local n=0
  while kill -0 "$pid" 2>/dev/null; do
    if [ "$n" -ge "$HAN_GIO" ]; then
      kill -TERM -"$pid" 2>/dev/null || kill -TERM "$pid" 2>/dev/null
      sleep 2
      kill -KILL -"$pid" 2>/dev/null || kill -KILL "$pid" 2>/dev/null
      return 124
    fi
    sleep 1
    n=$((n + 1))
  done
  wait "$pid"
}

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
stt=0
while IFS= read -r l; do
  printf '▶ %s\n' "$l"
  stt=$((stt + 1))
  RA="$NOI_RA/$stt.txt"
  chay_han_gio "$l"
  ma=$?
  if [ "$ma" = 124 ]; then
    echo "   ✗ HẾT GIỜ sau ${HAN_GIO}s — KHÔNG phải phép thử đỏ"
    echo "     Lệnh chạm Keychain thật thì macOS có thể đang chờ một cú bấm."
    hong=$((hong + 1))
  elif [ "$ma" != 0 ]; then
    echo "   ✗ HỎNG (mã thoát $ma) — đầu ra đầy đủ: $RA"
    # In đuôi, KHÔNG lọc `^error`. 26/08/2026 một lệnh thoát khác 0 mà không hề
    # có dòng nào bắt đầu bằng `error`; bộ lọc ấy in ra đúng con số không.
    tail -20 "$RA" | sed 's/^/     /' 
    hong=$((hong + 1))
  else
    r=$(grep -ac "test result: FAILED" "$RA" || true)
    if [ "$r" != "0" ]; then
      echo "   ✗ có phép thử đỏ — đầu ra đầy đủ: $RA"
      grep -a "^---- \|^failures:" -A 6 "$RA" | head -20 | sed 's/^/     /'
      hong=$((hong + 1))
    else echo "   ✓"; fi
  fi
done <<< "$lenh"

if [ "$hong" -ne 0 ]; then
  echo "════ HỎNG: $hong lệnh ════"; exit 1
fi
echo "════ ĐẠT: $so lệnh theo cờ ════"
