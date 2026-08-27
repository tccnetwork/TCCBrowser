#!/usr/bin/env bash
# Áp MỘT đột biến, chạy MỘT phép thử, rồi hoàn nguyên — và tự chốt từng bước.
#
#   tools/thu-dot-bien.sh <tệp> <chuỗi cũ> <chuỗi mới> <tên phép thử> [cờ cargo…]
#
# ⚠️ VÌ SAO CÓ TỆP NÀY: 26–27/08/2026 tôi viết tay đoạn "áp đột biến rồi chạy
# phép thử" khoảng mười lần, và nó sai **năm** lần — mỗi lần một kiểu, cả năm
# đều cho ra một câu trả lời NGHE HỢP LÝ:
#
#   1. Chuỗi cần thay xuất hiện HAI lần (hàm song sinh) → `python` bỏ cuộc,
#      nhưng vỏ lệnh chạy tiếp và báo "VẪN XANH" cho một đột biến CHƯA HỀ được
#      áp. Ba lần liên tiếp báo sai, và bản vá thật ra giết được cả ba.
#   2. Bộ lọc `cargo test <tên>` khớp một phép thử KHÁC có sẵn cùng tiền tố;
#      tôi đọc "1 passed" và tưởng là của mình. Phép thử của tôi chưa hề được
#      biên dịch.
#   3. Đếm `test result: FAILED` mà không tách LỖI BIÊN DỊCH → một đột biến
#      không dựng được bị đọc thành "sống sót".
#   4. Ngược lại: đếm `^error` rồi đọc `error: test failed` thành "không biên
#      dịch được", trong khi đó chính là PHÉP THỬ ĐỎ.
#   5. Hai bản cho kết quả y hệt nhau, và tôi định kết luận "vá không ăn" —
#      trong khi sự thật là đột biến TƯƠNG ĐƯƠNG, không đổi đầu ra.
#
# Cả năm cùng một hình dạng, và nó là hình dạng của cả dự án này: **một phép đo
# không phân biệt được "thứ tôi sợ đã xảy ra" với "phép đo của tôi không chạm
# tới thứ ấy" thì không phải một phép đo.**
#
# Nên tệp này chốt BỐN thứ, và dừng hẳn ở bất kỳ chốt nào không qua:
#   • chuỗi cũ xuất hiện ĐÚNG MỘT lần  → nếu không: VÔ HIỆU, không phải "xanh"
#   • tên phép thử CÓ trong `--list`   → nếu không: nó chưa từng chạy
#   • bản GỐC phải XANH trước đã       → nếu không: phép thử hỏng sẵn
#   • sau khi áp: tách ba kết quả — không-biên-dịch / ĐỎ / VẪN XANH
set -u

if [ "$#" -lt 4 ]; then
  sed -n '3p' "$0" | sed 's/^# \{0,1\}//'
  exit 2
fi
TEP=$1; CU=$2; MOI=$3; LOC=$4; shift 4
CO=("$@")

[ -f "$TEP" ] || { echo "✗ không có tệp $TEP"; exit 2; }

GOC=$(mktemp); cp "$TEP" "$GOC"
trap 'cp "$GOC" "$TEP"; rm -f "$GOC"' EXIT

# ── Chốt 1: tên phép thử phải TỒN TẠI ──────────────────────────────────────
if ! cargo test "${CO[@]}" -- --list 2>/dev/null | grep -qa "$LOC"; then
  echo "⛔ VÔ HIỆU: không có phép thử nào tên chứa '$LOC'."
  echo "   Chạy nó cũng vô nghĩa — bộ lọc sẽ khớp 0 phép thử, hoặc tệ hơn là"
  echo "   khớp một phép thử KHÁC và bạn đọc kết quả của người ta."
  exit 2
fi

# ── Chốt 2: bản GỐC phải xanh ──────────────────────────────────────────────
if ! cargo test "${CO[@]}" -- "$LOC" 2>&1 | grep -qa "test result: ok"; then
  echo "⛔ VÔ HIỆU: bản GỐC đã không xanh. Đo đột biến trên một phép thử đang"
  echo "   hỏng thì mọi kết luận đều vô nghĩa."
  exit 2
fi

# ── Chốt 3: đột biến phải áp được, ĐÚNG MỘT chỗ ────────────────────────────
if ! python3 - "$TEP" "$CU" "$MOI" <<'PY'
import io, sys
p, cu, moi = sys.argv[1], sys.argv[2], sys.argv[3]
s = io.open(p, encoding="utf-8").read()
n = s.count(cu)
if n != 1:
    sys.exit(f"chuỗi cũ xuất hiện {n} lần, cần đúng 1")
io.open(p, "w", encoding="utf-8").write(s.replace(cu, moi, 1))
PY
then
  echo "⛔ VÔ HIỆU: không áp được đột biến — KHÔNG phải 'vẫn xanh'."
  echo "   Xuất hiện nhiều lần thường nghĩa là mã bị CHÉP hai chỗ (hàm song"
  echo "   sinh). Thêm ngữ cảnh cho chuỗi, và nhớ rằng bản sao kia cũng cần"
  echo "   phép thử riêng — chứng minh trên bản này KHÔNG lây sang bản kia."
  exit 2
fi

# ── Đo, và tách BA kết quả ─────────────────────────────────────────────────
ra=$(cargo test "${CO[@]}" -- "$LOC" 2>&1)
if printf '%s' "$ra" | grep -qaE '^error\[E|could not compile'; then
  echo "⚠️  KHÔNG BIÊN DỊCH ĐƯỢC — phép đo vô hiệu, không phải 'sống sót'"
  printf '%s\n' "$ra" | grep -aE '^error' | head -3
  exit 1
elif printf '%s' "$ra" | grep -qa 'test result: FAILED'; then
  echo "✓ ĐỎ — phép thử giết được đột biến này"
  exit 0
else
  echo "✗ VẪN XANH — phép thử KHÔNG giết được."
  echo "  Hai khả năng, và chúng KHÁC NHAU:"
  echo "   • phép thử yếu → viết thêm khẳng định;"
  echo "   • đột biến TƯƠNG ĐƯƠNG → mã đổi mà đầu ra không đổi; không phép thử"
  echo "     nào phân biệt được, và không nên cố. Chứng minh bằng cách đo một"
  echo "     đại lượng BÊN TRONG (đếm số lần nhánh ấy chạy), rồi ghi vào mã."
  exit 1
fi
