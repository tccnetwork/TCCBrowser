#!/usr/bin/env bash
# Nhắc việc: in MỘT dòng nói việc kế tiếp và tình hình việc nền.
#
# Vì sao có tệp này: trợ lý chỉ hành động khi người dùng nhắn, HOẶC khi một việc
# nền báo về. Việc nền xong hết mà danh sách chưa hết thì không còn gì đánh thức
# nó — nhìn từ ngoài giống hệt "bỏ ngang". 27/08/2026 chủ dự án hỏi đúng câu ấy.
#
# Nên: chạy tệp này dưới một bộ theo dõi, mỗi dòng nó in ra là một lần đánh
# thức, và dòng ấy phải NÓI ĐƯỢC việc gì đang chờ — một nhịp "tick" rỗng chỉ tốn
# một lượt mà không đẩy việc đi đâu.
#
# ⚠️ CÔNG CỤ NÀY KHÔNG SỬA ĐƯỢC NGUYÊN NHÂN GỐC — đọc kỹ chỗ này.
#
# Nguyên nhân gốc: trợ lý không tự tạo được lượt làm việc mới. Thứ DUY NHẤT tạo
# ra lượt mới là một việc nền đang chạy báo về. Nên luật đúng là:
#
#     ĐỪNG kết thúc một lượt mà không có việc nền nào đang chạy,
#     trừ khi thật sự hết việc.
#
# 27/08/2026 chủ dự án chỉ ra rằng tôi KHÔNG giữ luật ấy: có lúc bộ nhắc báo
# "0 tiến trình nền" — tức tôi đứng im chờ đồng hồ 15 phút trong khi danh sách
# còn sáu việc. Bộ nhắc che mất khuyết điểm ấy: nó làm khoảng lặng ngắn lại nên
# trông như đã xong, trong khi nó chỉ hạ trần thời gian chết từ vô hạn xuống
# 900 giây. Nguyên nhân vẫn nguyên.
#
# Dòng "N tiến trình nền" in ra ở mỗi nhịp chính là để soi việc ấy: thấy 0 mà
# danh sách chưa hết thì đó là một lượt đã bị bỏ phí, không phải một lúc rảnh.
#
#   tools/nhac-viec.sh            # in một dòng rồi thoát
#   tools/nhac-viec.sh --lap 900  # in mỗi 900 giây, chạy mãi
#
# Danh sách việc: mỗi dòng một việc, dòng bắt đầu bằng `x ` là đã xong.
set -u
cd "$(dirname "$0")/.."

VIEC=${VIEC:-docs/viec-con-lai.md}
lap=0
[ "${1:-}" = "--lap" ] && lap=${2:-900}

mot_dong() {
  local con next chay
  if [ -f "$VIEC" ]; then
    # Bỏ tiêu đề, dòng trống, và việc đã đánh dấu `x `.
    con=$(grep -cE '^- \[ \]' "$VIEC" || true)
    next=$(grep -m1 -E '^- \[ \] (?!@)' "$VIEC" 2>/dev/null | sed 's/^- \[ \] //')
    [ -z "$next" ] && next=$(grep -m1 -E '^- \[ \]' "$VIEC" | grep -v '@NGƯỜI' | sed 's/^- \[ \] //')
  else
    con=0; next="(không có $VIEC)"
  fi

  # Tiến độ lượt quét đột biến, LÀM TRÒN tới trăm. Làm tròn có chủ đích: dòng
  # nhắc chỉ nên đổi khi có gì đáng đánh thức. Số chính xác thì đổi mỗi nhịp,
  # và một lần thức dậy để đọc "131 thay vì 130" là một lượt bỏ phí.
  local quet=""
  local m
  for m in /tmp/dot-bien*/*/mutants.out; do
    [ -d "$m" ] || continue
    local xong tong
    xong=$(( $(wc -l < "$m/caught.txt" 2>/dev/null || echo 0) \
           + $(wc -l < "$m/missed.txt" 2>/dev/null || echo 0) \
           + $(wc -l < "$m/unviable.txt" 2>/dev/null || echo 0) ))
    tong=$(grep -aoE "Found [0-9]+ mutants" "$(dirname "$m")".txt 2>/dev/null | grep -oE "[0-9]+" | head -1)
    [ -z "$tong" ] && continue
    [ "$xong" -ge "$tong" ] && continue
    quet=" · quét ~$(( xong / 100 * 100 ))/$tong"
  done

  # Việc nền còn chạy không. Mẫu phải khớp TÊN THẬT: `cargo-mutants` có GẠCH
  # NỐI. 26/08/2026 đếm bằng mẫu có dấu cách nên tưởng lượt quét đã chết rồi
  # chạy chồng lên, hỏng bản ghi của cả hai lượt.
  chay=$(pgrep -f 'cargo-mutants|cargo test|cargo build|kiem-' 2>/dev/null | wc -l | tr -d ' ')

  # Việc cần NGƯỜI thì trợ lý KHÔNG đẩy được milimet nào — nêu riêng, vì nêu
  # chung vào "còn N việc" là giấu mất chỗ đang thật sự tắc.
  local nguoi
  nguoi=$(grep -cE '^- \[ \] @NGƯỜI' "$VIEC" 2>/dev/null || echo 0)
  local may=$((con - nguoi))

  if [ "$con" -eq 0 ]; then
    echo "NHẮC: hết việc trong $VIEC · $chay tiến trình nền"
  elif [ "$may" -eq 0 ]; then
    echo "NHẮC: HẾT việc làm được bằng máy. Còn $nguoi việc CẦN NGƯỜI: $(grep -m1 -E '^- \[ \] @NGƯỜI' "$VIEC" | sed 's/^- \[ \] @NGƯỜI //')"
  else
    echo "NHẮC: còn $may việc máy + $nguoi việc CẦN NGƯỜI$quet · kế tiếp: $next · $chay tiến trình nền"
  fi
}

if [ "$lap" -eq 0 ]; then
  mot_dong
else
  # In khi dòng ĐỔI. Cứ 6 nhịp im thì in lại một lần, để "im lặng" không lẫn
  # với "đã chết" — đúng hạng lỗi mà tài liệu này đi phê bình: một phép đo
  # không phân biệt được hai thứ ấy thì không phải một phép đo.
  truoc=""
  im=0
  while true; do
    nay=$(mot_dong)
    if [ "$nay" != "$truoc" ] || [ "$im" -ge 6 ]; then
      printf '%s\n' "$nay"
      truoc=$nay
      im=0
    else
      im=$((im + 1))
    fi
    sleep "$lap"
  done
fi
