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
    next=$(grep -m1 -E '^- \[ \]' "$VIEC" | sed 's/^- \[ \] //')
  else
    con=0; next="(không có $VIEC)"
  fi

  # Việc nền còn chạy không. Mẫu phải khớp TÊN THẬT: `cargo-mutants` có GẠCH
  # NỐI. 26/08/2026 đếm bằng mẫu có dấu cách nên tưởng lượt quét đã chết rồi
  # chạy chồng lên, hỏng bản ghi của cả hai lượt.
  chay=$(pgrep -f 'cargo-mutants|cargo test|cargo build|kiem-' 2>/dev/null | wc -l | tr -d ' ')

  if [ "$con" -eq 0 ]; then
    echo "NHẮC: hết việc trong $VIEC · $chay tiến trình nền"
  else
    echo "NHẮC: còn $con việc · kế tiếp: $next · $chay tiến trình nền"
  fi
}

if [ "$lap" -eq 0 ]; then
  mot_dong
else
  while true; do
    mot_dong
    sleep "$lap"
  done
fi
