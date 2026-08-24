#!/usr/bin/env bash
# Chạy CHÍNH cái nhị phân sản phẩm, đi HẾT đường, không cần người bấm.
#
# ⚠️ Vì sao cần: ngày 24/08/2026 `tcc-browser examples/hello-tcc` **abort** ngay
# lúc chuyển từ hộp thoại hỏi quyền sang màn ứng dụng — trong khi 351 phép thử
# tự động đều XANH. Không phép thử nào chạy chính cái nhị phân ấy đi qua hai màn
# hình, nên cả bộ kiểm không thấy gì.
#
# Và lần "kiểm khói" trước đó của tôi báo ĐẠT: nó chạy 12 giây, thấy tiến trình
# còn sống, rồi kết luận — trong khi tiến trình mới đứng ở màn ĐẦU. Nên kịch bản
# này không hỏi "còn sống không"; nó hỏi ba câu kiểm được:
#
#   1. mã thoát 0
#   2. không có "panicked" hay "abort" trong đầu ra
#   3. đầu ra có dòng chứng minh đã đi tới màn THỨ HAI
#
# Câu 3 là câu quan trọng. Hai câu đầu vẫn ĐẠT nếu chương trình dừng ở màn một.
set -u
cd "$(dirname "$0")/.."

GIAY=${GIAY:-2}
LOG=$(mktemp)
trap 'rm -f "$LOG"' EXIT

echo "── kiểm khói: chạy trọn đường, ${GIAY}s mỗi màn ──"
cargo build -q -p tcc-browser --features wallet || exit 1

# `tu-choi` trên hộp thoại: TỪ CHỐI mọi quyền. Cố ý chọn phía an toàn — một kịch
# bản kiểm khói tự CẤP quyền là một kịch bản dạy người ta rằng cấp quyền là mặc
# định.
TCC_QUEN_HET=1 TCC_TU_DONG_BAM=tu-choi TCC_TU_DONG_DONG="$GIAY" \
  ./target/debug/tcc-browser examples/hello-tcc > "$LOG" 2>&1
ma=$?

loi=0
if [ "$ma" != 0 ]; then
  echo "❌ mã thoát $ma"; loi=1
fi
if grep -aqE "panicked|abort" "$LOG"; then
  echo "❌ có hoảng loạn trong đầu ra:"; grep -aE "panicked|abort" "$LOG" | head -3; loi=1
fi
# Ba dòng này chỉ in ra SAU khi quyền đã được cấp — tức là sau khi hộp thoại đã
# được trả lời và chuỗi đã sang màn hai.
if ! grep -aq "quyền mạng:" "$LOG"; then
  echo "❌ không tới được màn thứ hai — chương trình dừng ở hộp thoại"; loi=1
fi
if ! grep -aq "quyền mạng: không" "$LOG"; then
  echo "❌ bấm 'từ chối' mà quyền vẫn được cấp"; loi=1
fi

if [ "$loi" = 0 ]; then
  echo "✅ đi hết đường: hộp thoại → trả lời → màn ứng dụng, quyền bị từ chối"
else
  echo "── đầu ra đầy đủ ──"; cat "$LOG"
fi

# ── Hai lệnh cửa sổ CÒN LẠI ─────────────────────────────────────────────────
#
# Chúng mở cửa sổ riêng, và cho tới 24/08/2026 chưa lần nào có ai chạy chúng —
# cùng hạng với đường chính. Kiểm nhẹ hơn: chạy được, không hoảng loạn, thoát 0.
# Nhẹ hơn vì chúng chỉ có MỘT màn hình, nên không có "màn thứ hai" để hỏi tới.
for lenh in "quyen" "hop-thoai"; do
  ra=$(TCC_TU_DONG_DONG="$GIAY" ./target/debug/tcc-browser "$lenh" examples/hello-tcc 2>&1)
  ma=$?
  if [ "$ma" != 0 ]; then
    echo "❌ lệnh '$lenh' thoát $ma"; echo "$ra" | head -5; loi=1
  elif printf '%s' "$ra" | grep -qE "panicked|abort"; then
    echo "❌ lệnh '$lenh' hoảng loạn"; printf '%s' "$ra" | grep -E "panicked|abort" | head -2; loi=1
  else
    echo "✅ lệnh '$lenh' chạy và tự đóng"
  fi
done

exit "$loi"
