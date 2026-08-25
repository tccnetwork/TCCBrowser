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

# ── Gói xin quyền VÍ: màn quan trọng nhất, và chưa gì tự động đi qua ───────
#
# `hello-tcc` chỉ xin quyền MẠNG, nên tới 26/08/2026 hàng quyền ví — hàng duy
# nhất mang câu "việc này chuyển tiền" — chưa lần nào được một phép kiểm tự
# động dựng ra. Nó chỉ tồn tại trong phép thử đơn vị của bộ dựng.
#
# Kiểm ba điều, và điều thứ ba mới là điều đáng: dựng được, không hoảng loạn,
# và câu cảnh báo CÓ MẶT trong cây hộp thoại. Thiếu điều thứ ba thì đây chỉ là
# một phép kiểm "chạy được", mà chạy được không có nghĩa là nói đúng.
ra=$(TCC_TU_DONG_DONG="$GIAY" ./target/debug/tcc-browser hop-thoai examples/vi-du-vi 2>&1)
ma=$?
if [ "$ma" != 0 ]; then
  echo "❌ hộp thoại quyền ví thoát $ma"; echo "$ra" | head -5; loi=1
elif printf '%s' "$ra" | grep -qE "panicked|abort"; then
  echo "❌ hộp thoại quyền ví hoảng loạn"; loi=1
else
  echo "✅ hộp thoại gói xin quyền ví chạy và tự đóng"
fi

# Câu cảnh báo là chữ của KHUNG, không phải của gói — nên nó phải có mặt dù gói
# viết gì. Đọc từ bản dựng KHÔNG cửa sổ, chỗ in cây ra chữ.
cay=$(CARGO_TARGET_DIR=/tmp/kiem-khoi-cay cargo run -q -p tcc-browser -- \
  hop-thoai examples/vi-du-vi 2>&1 || true)
if printf '%s' "$cay" | grep -q "this moves money"; then
  echo "✅ hàng quyền ví mang câu 'this moves money'"
else
  echo "❌ hàng quyền ví KHÔNG mang câu cảnh báo — B45"; loi=1
fi

# ── Luồng VÍ: đường chưa ai chạy bao giờ ────────────────────────────────────
#
# `wallet_flow` cổng sang `open_sequence` ngày 23/08 và tới 24/08 vẫn chưa lần
# nào chạy với cửa sổ thật. Ở đây nó đi HAI màn: gõ cụm từ → bấm "tiếp tục" với
# ô rỗng → màn báo lỗi. Rồi hết danh sách bấm nên cửa sổ đóng.
#
# ⚠️ Tín hiệu "đã sang màn hai" ở đây là THỜI GIAN, không phải một dòng chữ:
# luồng này không in gì cho tới lúc kết thúc. Mỗi màn chờ $GIAY giây, nên hai màn
# phải mất hơn $GIAY giây. Dừng ở màn một thì nhanh hơn hẳn — và một phép kiểm
# chỉ xem mã thoát sẽ ĐẠT ở cả hai, vì huỷ ở màn nào cũng ra mã 1.
t0=$(python3 -c 'import time;print(time.time())')
ra=$(TCC_TU_DONG_BAM=cum-tu-tiep TCC_TU_DONG_DONG="$GIAY" \
  ./target/debug/tcc-browser vi cum-tu 2>&1)
ma=$?
giay=$(python3 -c "import time;print(f'{time.time()-$t0:.1f}')")
if printf '%s' "$ra" | grep -qE "panicked|abort"; then
  echo "❌ luồng ví hoảng loạn"; printf '%s' "$ra" | grep -E "panicked|abort" | head -2; loi=1
elif [ "$ma" != 1 ]; then
  # Đóng cửa sổ mà không nhập gì là HUỶ, và huỷ là mã 1. Mã 0 ở đây nghĩa là
  # một ví đã được khôi phục mà không ai gõ cụm từ nào.
  echo "❌ luồng ví thoát $ma, đáng lẽ 1 (huỷ)"; printf '%s' "$ra" | head -3; loi=1
elif python3 -c "import sys; sys.exit(0 if $giay > $GIAY * 1.4 else 1)"; then
  echo "✅ luồng ví đi qua hai màn (${giay}s) rồi huỷ"
else
  echo "❌ luồng ví chỉ ${giay}s — dừng ở màn đầu, không sang màn báo lỗi"; loi=1
fi

# ── Luồng NHẬP VÍ: ba màn, và tín hiệu là một CÂU, không phải thời gian ─────
#
# chọn ví → hỏi PIN → (PIN rỗng) → mở khoá thất bại.
#
# ⚠️ Câu này TỪNG viết: *"'sai PIN' chỉ in ra khi đã tới màn ba"*. SAI. Kiểm đột
# biến chỉ ra: bỏ hẳn màn báo lỗi đi thì luồng vẫn trả về đúng lỗi ấy, vì lỗi
# được đặt TRƯỚC khi màn hình được dựng. Kịch bản vẫn ĐẠT.
#
# Nên nói đúng thứ nó chứng minh: **PIN rỗng bị TỪ CHỐI**, và luồng chạy được
# qua hai lần bấm mà không hoảng loạn. Nó KHÔNG chứng minh màn báo lỗi hiện ra.
#
# Tín hiệu duy nhất phân biệt được là thời gian (đo 24/08: 8.7s có màn lỗi, 6.7s
# không) — và một ngưỡng thời gian hiệu chỉnh trên một máy là thứ sẽ chập chờn
# trên máy khác, rồi bị tắt đi. Chỗ hở này để nguyên và nói ra, hơn là lấp bằng
# một phép đo dễ lung lay.
#
# Địa chỉ lấy TỪ chính tệp dữ liệu, không chép cứng: chép cứng thì đổi tệp mẫu là
# kịch bản lặng lẽ bấm một mã không tồn tại, và "không có mã ấy" thì cửa sổ đóng
# — tức là ĐẠT vì lý do sai.
TEP_VI=crates/tcc-chain/data/vi-web-mau.json
DC=$(python3 -c "import json;print(list(json.load(open('$TEP_VI'))['wallets'])[0])")
ra=$(TCC_TU_DONG_BAM="nhap-$DC,mo-khoa-vi" TCC_TU_DONG_DONG="$GIAY" \
  ./target/debug/tcc-browser vi nhap "$TEP_VI" 2>&1)
ma=$?
if printf '%s' "$ra" | grep -qE "panicked|abort"; then
  echo "❌ luồng nhập ví hoảng loạn"; printf '%s' "$ra" | grep -E "panicked|abort" | head -2; loi=1
elif [ "$ma" = 0 ]; then
  # Mã 0 nghĩa là ví đã được nhập với PIN RỖNG. Đó là hỏng nặng nhất có thể ở
  # đường này.
  echo "❌ nhập ví THÀNH CÔNG với PIN rỗng"; loi=1
elif printf '%s' "$ra" | grep -q "sai PIN"; then
  echo "✅ luồng nhập ví: chọn ví → hỏi PIN → PIN rỗng bị từ chối"
else
  echo "❌ luồng nhập ví không từ chối PIN rỗng đúng cách:"; printf '%s' "$ra" | head -3; loi=1
fi

exit "$loi"
