#!/usr/bin/env bash
# Kiểm đột biến TỪNG HÒM, và GHI LẠI kết quả.
#
# Vì sao có tệp này: 26/08/2026 `SECURITY.md` §3.31 phải thú nhận rằng lượt quét
# đột biến đầu tiên của dự án **không có hồ sơ theo từng hòm**. Phần lớn phát
# hiện §3.18–§3.30 ra từ nó, mà không ai — kể cả người chạy — kiểm lại được nó
# đã phủ tới đâu. Một phép đo không có hồ sơ là một câu chuyện, không phải một
# phép đo.
#
# BỐN CÁI BẪY ĐÃ DẪM PHẢI, cả bốn đều khiến con số SAI mà trông vẫn hợp lý:
#
#  1. HẾT ĐĨA giữa chừng → mọi đột biến còn lại thành `TIMEOUT`, đọc y hệt "bộ
#     thử chẳng bắt được gì". Dòng `No space left on device` nằm ở CUỐI đầu ra,
#     sau hàng chục dòng TIMEOUT. Nên: chốt `df` TRƯỚC mỗi hòm, và dừng hẳn.
#  2. `--timeout-multiplier` NHÂN THỜI GIAN CỦA BỘ THỬ HÒM ẤY, không phải của
#     lượt chạy thật. Nhân thời gian `tcc-spec` (~2s) rồi áp cho lượt cả không
#     gian làm việc (~70s) là đặt hạn giờ ngắn hơn phép thử. Dùng SỐ TUYỆT ĐỐI.
#  3. KHÔNG BẬT CỜ → mã sau `#[cfg(feature)]` không hề được biên dịch. Lượt chạy
#     `tcc-chain` không cờ báo 45 kẻ sống sót; bật cờ lên, con số thật là 25.
#  4. Vài hòm KHÔNG quét tự động được, và phải NÓI RA chứ không lặng lẽ bỏ:
#     `tcc-keystore --features os-keystore` ghi vào Keychain THẬT rồi chờ một cú
#     bấm chuột không bao giờ tới (đã treo hơn bốn mươi phút); `tcc-shell` với
#     `wallet`/`window` MỞ CỬA SỔ. Bỏ im lặng thì bảng kết quả nói dối bằng cách
#     im lặng — đúng hạng lỗi tệ nhất trong một hồ sơ kiểm định.
set -u
cd "$(dirname "$0")/.."

DIA_TOI_THIEU=${DIA_TOI_THIEU:-8}          # GB. Dưới mức này thì DỪNG, không chạy.
HAN_GIO=${HAN_GIO:-300}                    # giây, TUYỆT ĐỐI cho mỗi đột biến
RA=${RA:-/tmp/dot-bien}

# Ép bộ cờ cho lượt chạy này, kể cả ép thành RỖNG. Có để quét được cấu hình
# TỐI THIỂU — xem bẫy 8 trong `docs/kiem-dot-bien.md`: bất biến sống nhờ một cờ
# TẮT thì lượt quét với bộ cờ tối đa mù về mặt cấu trúc với nó. `cap_duoc` bật
# `wallet` là mọi nhánh trả `true` nên đột biến "luôn true" TƯƠNG ĐƯƠNG; tắt
# `wallet` thì đúng đột biến ấy làm 5 phép thử đỏ.
#
# Dùng kèm `RA=` để không ghi đè hồ sơ của lượt cờ-đủ:
#   RA=/tmp/dot-bien-toi-thieu CO_RIENG= tools/kiem-dot-bien.sh tcc-shell
mkdir -p "$RA"

# ⚠️ MỘT LƯỢT MỘT LÚC. 26/08/2026 tôi tưởng lượt quét đã chết — vì đếm tiến
# trình bằng mẫu `cargo mutants` CÓ DẤU CÁCH trong khi tên thật là
# `cargo-mutants` CÓ GẠCH NỐI — nên chạy lại chồng lên. Hai lượt cùng quét một
# hòm, tranh một khoá, và ghi đè tệp đầu ra của nhau: bản ghi của cả hai hỏng.
#
# Phép đo lại không phân biệt được "không chạy" với "mẫu của tôi không khớp" —
# đúng hạng lỗi CLAUDE.md đã ghi ba lần. Nên chốt bằng thứ không phụ thuộc vào
# việc đoán đúng tên tiến trình: một thư mục khoá, tạo được thì chạy.
KHOA="$RA/.dang-chay"
if ! mkdir "$KHOA" 2>/dev/null; then
  echo "✗ ĐÃ CÓ một lượt quét đang chạy (khoá: $KHOA)."
  echo "  Chắc chắn không còn lượt nào thì xoá thư mục ấy rồi chạy lại."
  echo "  Đếm tiến trình: pgrep -fl cargo-mutants   (GẠCH NỐI, không phải dấu cách)"
  exit 1
fi
trap 'rmdir "$KHOA" 2>/dev/null' EXIT

dia_con() { df -g /Volumes/DATA | tail -1 | awk '{print $4}'; }

chot_dia() {
  local con; con=$(dia_con)
  if [ "$con" -lt "$DIA_TOI_THIEU" ]; then
    echo "✗ DỪNG: còn ${con}GB, dưới ngưỡng ${DIA_TOI_THIEU}GB."
    echo "  Chạy tiếp là lặp lại đúng lần hỏng 25/08/2026: hết đĩa giữa chừng,"
    echo "  mọi đột biến còn lại thành TIMEOUT, và bảng kết quả trông như một"
    echo "  bộ thử vô dụng trong khi bộ thử không hề có lỗi gì."
    return 1
  fi
  echo "  (đĩa còn ${con}GB)"
  return 0
}

# hòm : cờ : ghi chú
# Cờ rút từ `.github/workflows/ci.yml` — hòm nào CI chạy có cờ thì quét có cờ.
HOM=(
  "tcc-spec::"
  "tcc-crypto::"
  "tcc-manifest::"
  "tcc-capability::"
  "tcc-ui::"
  "tcc-runtime::"
  "tcc-net::"
  "tcc-chain:import-web-wallet:"
  "tcc-render-raster:window,accesskit:"
  # ⚠️ PHẢI đủ ba cờ. `wallet_flow` khai sau `all(window, import-web-wallet,
  # os-keystore)`; quét với MỘT cờ thì tệp ấy KHÔNG được biên dịch, đột biến
  # không có tác dụng gì, phép thử xanh, và công cụ ghi 18 dòng MISSED. Đúng
  # bẫy 3, và tôi dẫm phải sau khi tự viết nó ra.
  #
  # Lý do đặt cờ hẹp lúc đầu — "wallet/window mở cửa sổ" — cũng sai: chính
  # `kiem-theo-co.sh` chạy `cargo test -p tcc-shell --features wallet` mỗi lượt
  # và nó xanh trong vài giây. Phép thử không mở cửa sổ nào. Một giả định chưa
  # bao giờ kiểm, đứng suốt trong kịch bản dưới dạng chú thích nghe hợp lý.
  "tcc-shell:wallet:"
  # ⚠️ Lý do BỎ đã đổi, 27/08/2026 — lý do cũ SAI.
  #
  # Cũ: "ghi Keychain THẬT rồi chờ bấm tay". Đo lại thì không đúng nữa: phép thử
  # bật hộp thoại ĐÃ là `#[ignore]`, và `cargo test -p tcc-keystore --features
  # os-keystore` chạy 30 giây, 9 xanh + 1 bỏ qua, không hộp thoại nào. Cùng hạng
  # lỗi với chú thích "wallet/window mở cửa sổ" ở `tcc-shell`: một giả định
  # đúng-lúc-viết, sống sót vì nó nghe hợp lý và không ai đo lại.
  #
  # Lý do THẬT, nặng hơn: đột biến trên `delete` làm nó KHÔNG xoá gì, và phép
  # thử sẽ để lại mục rác trong Keychain THẬT của người chạy. Một lượt quét là
  # hàng trăm lần dựng-chạy; rác tích lại trên máy người khác là cái giá không
  # được phép trả lén.
  #
  # Mở khoá được bằng cách: cho `SERVICE` đọc từ biến môi trường khi kiểm thử,
  # quét dưới một tên dịch vụ RIÊNG, rồi dọn sạch tên ấy sau lượt chạy.
  # ⚠️ Quét được TỪ 27/08/2026, sau khi tách tên dịch vụ Keychain cho phép thử
  # (`crates/tcc-keystore/src/macos.rs`, hằng `SERVICE` theo `cfg(test)`).
  #
  # Rác vẫn sinh ra — đột biến trên `delete` làm nó không xoá gì — nhưng nay nó
  # nằm gọn dưới MỘT tên biết trước. DỌN SAU MỖI LƯỢT:
  #
  #   while security delete-generic-password -s com.tcc.browser.wallet.KIEM-THU \
  #     >/dev/null 2>&1; do :; done
  #
  # Phép thử bật hộp thoại Keychain là `#[ignore]`, nên bộ thử chạy 30 giây và
  # KHÔNG chờ cú bấm nào.
  "tcc-keystore:os-keystore:"
)

mot_hom() {
  local hom=$1 co=$2
  local nhan="$hom${co:+ [$co]}"
  echo "▶ $hom${co:+ --features $co}"
  chot_dia || return 1
  local lenh=(cargo mutants -p "$hom" --timeout "$HAN_GIO" --output "$RA/$hom")
  [ -n "$co" ] && lenh+=(--features "$co")
  "${lenh[@]}" > "$RA/$hom.txt" 2>&1
  local ma=$?
  # ⚠️ Đọc `No space left` TRƯỚC khi đọc con số. Hết đĩa thì con số vô nghĩa.
  if grep -aq "No space left on device" "$RA/$hom.txt"; then
    echo "   ✗ HẾT ĐĨA giữa chừng — con số của hòm này VÔ NGHĨA, đừng chép vào bảng"
    return 1
  fi
  local tong song timeout hong
  tong=$(grep -aoE "Found [0-9]+ mutants" "$RA/$hom.txt" | grep -oE "[0-9]+" | head -1)
  song=$(grep -ac "^MISSED" "$RA/$hom.txt")
  timeout=$(grep -ac "^TIMEOUT" "$RA/$hom.txt")
  hong=$(grep -ac "^UNVIABLE" "$RA/$hom.txt")
  printf '%s|%s|%s|%s|%s|%s|%s\n' \
    "$hom" "$co" "${tong:-?}" "$song" "$timeout" "$hong" "$ma" >> "$RA/bang.txt"
  echo "   $tong đột biến · $song SỐNG SÓT · $timeout hết giờ · $hong không dựng được (mã=$ma)"
}

# Tham số: tên hòm cần quét. Không có thì quét tất. Có để chạy thử harness trên
# một hòm nhỏ trước khi giao cho nó nhiều giờ — 26/08/2026 tôi định cắt tạm bằng
# `sed` và cái bị hỏng là bản cắt, không phải kịch bản.
if [ "$#" -gt 0 ]; then
  chon=()
  for muc in "${HOM[@]}"; do
    for t in "$@"; do
      [ "${muc%%:*}" = "$t" ] && chon+=("$muc")
    done
  done
  [ "${#chon[@]}" -eq 0 ] && { echo "✗ không hòm nào khớp: $*"; exit 1; }
  HOM=("${chon[@]}")
fi

# Chỉ xoá bảng khi quét LẠI TỪ ĐẦU. 26/08/2026 lượt quét chết ngang ở hòm thứ
# tám; chạy tiếp ba hòm còn lại mà xoá bảng là mất số đo của bảy hòm đã xong —
# một công cụ sinh ra để GIỮ hồ sơ mà lại tự xoá hồ sơ.
if [ "$#" -eq 0 ]; then : > "$RA/bang.txt"; else touch "$RA/bang.txt"; fi
echo "── kiểm đột biến từng hòm · hạn giờ ${HAN_GIO}s · ngưỡng đĩa ${DIA_TOI_THIEU}GB ──"
for muc in "${HOM[@]}"; do
  IFS=: read -r hom co ghi <<< "$muc"
  # `${CO_RIENG+x}` phân biệt "đặt thành rỗng" với "không đặt" — cấu hình tối
  # thiểu CHÍNH LÀ chuỗi rỗng, nên `${CO_RIENG:-$co}` sẽ nuốt mất nó.
  [ -n "${CO_RIENG+x}" ] && co=$CO_RIENG
  # Bỏ qua phải là một QUYẾT ĐỊNH ghi rõ ("BỎ:"), không phải hệ quả của việc cờ
  # rỗng — suy ra từ chỗ trống là cách bỏ sót im lặng lẻn vào bảng.
  if [ "${ghi%%:*}" = "BỎ" ]; then
    ghi=${ghi#BỎ:}
    echo "▶ $hom — ⊘ BỎ QUA, có lý do: $ghi"
    printf '%s||⊘|⊘|⊘|⊘|%s\n' "$hom" "$ghi" >> "$RA/bang.txt"
    continue
  fi
  mot_hom "$hom" "$co" || { echo "── dừng sớm ở $hom ──"; break; }
done
echo "── bảng thô: $RA/bang.txt ──"
cat "$RA/bang.txt"
