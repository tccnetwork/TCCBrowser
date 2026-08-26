#!/usr/bin/env bash
# Dựng bảng kết quả kiểm đột biến từ NGUỒN ĐÚNG.
#
# ⚠️ Vì sao tách khỏi `kiem-dot-bien.sh`: kịch bản kia đếm bằng cách `grep` dòng
# chảy trên MÀN HÌNH của `cargo-mutants`. Mà công cụ ấy chỉ in ra màn hình những
# kết quả ĐÁNG LO (`MISSED`, `TIMEOUT`, `UNVIABLE` — và không phải lúc nào cũng
# đủ); đột biến BỊ BẮT thì nó im lặng.
#
# Giá đã trả, 26/08/2026: hai lần trong một buổi. Lần một, tôi đếm dòng giữa
# chừng rồi đọc ra "138 sống trên 142 đã chạy" — một tỷ lệ chỉ có thể có nếu bộ
# thử hỏng; số thật là 313 bị bắt trên 476. Lần hai, bảng ghi "0 không dựng
# được" trong khi `unviable.txt` có 19 dòng.
#
# Nguồn đúng là bốn tệp trong `mutants.out/`. Chúng là thứ công cụ GHI RA, không
# phải thứ nó KỂ LẠI.
set -u
RA=${RA:-/tmp/dot-bien}

dem() { [ -f "$1" ] && wc -l < "$1" | tr -d ' ' || echo 0; }

printf '| Hòm | Đột biến | Bị bắt | Sống sót | Hết giờ | Không dựng được | Tỷ lệ bắt |\n'
printf '|---|---:|---:|---:|---:|---:|---:|\n'
for d in "$RA"/*/; do
  hom=$(basename "$d")
  m="$d/mutants.out"
  [ -d "$m" ] || continue
  bat=$(dem "$m/caught.txt")
  song=$(dem "$m/missed.txt")
  gio=$(dem "$m/timeout.txt")
  hong=$(dem "$m/unviable.txt")
  tong=$((bat + song + gio + hong))
  # Tỷ lệ bắt tính trên số đột biến DỰNG ĐƯỢC — cộng cả `unviable` vào mẫu số là
  # tự cho điểm về việc mã không biên dịch được.
  mau=$((bat + song + gio))
  if [ "$mau" -gt 0 ]; then ty=$((bat * 100 / mau)); else ty=0; fi
  printf '| `%s` | %d | %d | **%d** | %d | %d | %d%% |\n' \
    "$hom" "$tong" "$bat" "$song" "$gio" "$hong" "$ty"
done
