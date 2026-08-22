#!/usr/bin/env bash
# Đóng gói `TCCBrowser.app` cho macOS.
#
# ⚠️ KỊCH BẢN NÀY CHƯA CHẠY TRỌN ĐƯỢC, và nó dừng lại ở đúng chỗ thiếu.
#
# # Vì sao ví cần gói ứng dụng đã ký
#
# `AccessControlOptions::USER_PRESENCE` — thứ bắt hệ điều hành hỏi Touch ID cho
# TỪNG lần lấy khoá — đòi quyền `keychain-access-groups`. Bản `cargo run` không
# có nó, và `store` hỏng ngay:
#
#     A required entitlement isn't present.
#
# # Ba lần thử, và cái bẫy ở lần thứ ba (17/08/2026)
#
# | Thử | Kết quả |
# |---|---|
# | Không ký | hỏng NGAY, thông báo rõ ràng |
# | Ký ad-hoc (`-s -`) kèm entitlements | qua cửa quyền, rồi TREO |
# | Ký bằng chứng thư thật, nhóm truy cập đúng mã đội | vẫn TREO |
#
# Lần thứ ba mới là bài học. Tiến trình **không báo lỗi** — nó nạp lên rồi treo
# ở bộ nạp với 8 KiB bộ nhớ, trạng thái `UE`, và `kill -9` không gỡ được. Không
# một dòng nào ra `stderr`, kể cả dòng in trước lệnh chạm Keychain đầu tiên.
#
# Nguyên nhân: quyền khai một nhóm truy cập mà **không hồ sơ cấp phép nào cho
# phép**. macOS không từ chối bằng một lỗi; nó treo tiến trình trong lúc thẩm
# định chữ ký.
#
# # Vậy còn thiếu gì
#
# Một **hồ sơ cấp phép macOS** (`.provisionprofile`) gắn với một App ID có bật
# nhóm truy cập Keychain, nhúng vào gói tại `Contents/embedded.provisionprofile`.
# Tạo nó là việc trên tài khoản Apple Developer của tổ chức — không phải việc
# kịch bản này tự làm được, và không phải việc nên làm hộ ai.
set -uo pipefail
cd "$(dirname "$0")/.."

TEN="TCCBrowser"
GOI="target/release/bundle/$TEN.app"
HO_SO="${TCC_PROVISION_PROFILE:-}"
CHUNG_THU="${TCC_SIGN_IDENTITY:-}"

echo "── dựng bản phát hành ──"
# Cờ `wallet` kéo theo `os-keystore`, tức là ví CẤT được khoá.
#
# Trước 22/08/2026 chỗ này cố ý chỉ dựng `window`: không có hồ sơ cấp phép thì
# ví không cất được, và một bản dựng có mục ví mà mục ấy hỏng thì tệ hơn một bản
# dựng không có ví. Giờ đã có hồ sơ, nên dựng đủ.
#
# Vẫn giữ đường cũ: `TCC_KHONG_VI=1` để dựng bản không ví.
CO_VI="${TCC_KHONG_VI:+window}"
CO_VI="${CO_VI:-wallet}"
echo "── cờ: $CO_VI ──"
cargo build --release -p tcc-browser --features "$CO_VI" || exit 1

echo "── ráp gói ──"
rm -rf "$GOI"
mkdir -p "$GOI/Contents/MacOS" "$GOI/Contents/Resources"
cp target/release/tcc-browser "$GOI/Contents/MacOS/$TEN"
cat > "$GOI/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleExecutable</key><string>$TEN</string>
  <key>CFBundleIdentifier</key><string>com.tcc.browser</string>
  <key>CFBundleName</key><string>$TEN</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>0.1</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
  <!--
    ⚠️ KHÔNG khai NSCameraUsageDescription, NSMicrophoneUsageDescription,
    NSLocationWhenInUseUsageDescription hay bất kỳ NS*UsageDescription nào.

    Lý do là một hạn chế của thư viện, không phải sở thích: wry 0.52.1 viết
    CỨNG WKPermissionDecision::Grant cho yêu cầu micro/camera của trang web
    (wkwebview/class/wry_web_view_ui_delegate.rs dòng 74) và không cho ghi đè.
    Nên ở tầng 2, một trang gọi getUserMedia() được cấp mà KHÔNG ai hỏi người
    dùng — đâm thẳng vào quyết định kiến trúc số 2.

    ⚠️ KHÔNG dùng dấu huyền trong khối này: heredoc dựng Info.plist cố ý KHÔNG
    đóng nháy (nó cần khai triển $TEN), nên dấu huyền bị vỏ lệnh chạy như lệnh.
    Đã trả giá 22/08/2026 — kịch bản dựng ra một Info.plist méo và tự báo lỗi.

    Chắn duy nhất còn lại nằm ở tầng hệ điều hành: thiếu chuỗi mô tả mục đích
    thì macOS TỪ CHỐI ứng dụng chạm micro/camera, nên lời Grant của wry không
    có gì để cấp.

    Thêm một dòng NS*UsageDescription vào đây là gỡ chắn ấy đi. Có phép kiểm
    trong kịch bản này canh, và luật kiến trúc 20 canh ở CI.
  -->
</dict></plist>
PLIST

# Chắn duy nhất chống việc wry tự cấp micro/camera — xem chú thích trong Info.plist.
# Soi KHAI BÁO thật, không soi mọi chỗ nhắc tên — y hệt luật 20. Bản trước dùng
# `grep -q "UsageDescription"` và nó bắt nhầm ĐÚNG CÂU CHÚ THÍCH giải thích vì
# sao không được khai. Một phép canh tự tố cáo mình là một phép canh người ta
# học cách bỏ qua.
if grep -qE "<key>NS[A-Za-z]*UsageDescription</key>" "$GOI/Contents/Info.plist"; then
  echo "❌ Info.plist khai NS*UsageDescription — gỡ mất chắn micro/camera của tầng 2"
  exit 1
fi
echo "✅ Info.plist không khai quyền thiết bị nào"
echo "✅ gói ở $GOI"

if [ -z "$HO_SO" ] || [ -z "$CHUNG_THU" ]; then
  cat <<'HUONG_DAN'

⚠️  DỪNG TRƯỚC BƯỚC KÝ — thiếu hồ sơ cấp phép.

Gói đã ráp xong và chạy được, nhưng KHÔNG có ví: `USER_PRESENCE` cần quyền
`keychain-access-groups`, mà quyền ấy cần một hồ sơ cấp phép macOS.

ĐỪNG ký kèm entitlements khi chưa có hồ sơ. Tiến trình sẽ không báo lỗi — nó
TREO trong bộ nạp, không in ra dòng nào, và `kill -9` không gỡ được (đã trả giá
ngày 17/08/2026).

Cần làm, trên tài khoản Apple Developer của tổ chức:
  1. Tạo App ID `com.tcc.browser`, bật Keychain Sharing
  2. Tạo hồ sơ cấp phép macOS cho App ID ấy, tải về
  3. Chạy lại:
       TCC_PROVISION_PROFILE=<đường-dẫn.provisionprofile> \
       TCC_SIGN_IDENTITY="Developer ID Application: …" \
       tools/dong-goi-macos.sh

HUONG_DAN
  exit 0
fi

cp "$HO_SO" "$GOI/Contents/embedded.provisionprofile"

# ⚠️ Tiền tố đội lấy TỪ CHÍNH HỒ SƠ, không viết cứng và không dùng
# `$(AppIdentifierPrefix)`.
#
# `$(AppIdentifierPrefix)` là biến của Xcode. `codesign` **không khai triển nó**
# — nó nhúng đúng chuỗi ấy làm quyền, chuỗi ấy không khớp hồ sơ, và quyền không
# được cấp. Đã trả giá 22/08/2026.
# Qua TỆP TẠM chứ không qua ống: `plistlib.load` cần luồng tua lại được, và một
# ống thì không. Lỗi nó ném ra ("File or stream is not seekable") không nhắc gì
# tới plist, nên nó đọc như hồ sơ hỏng.
TAM=$(mktemp) && security cms -D -i "$HO_SO" > "$TAM" 2>/dev/null
DOI=$(python3 -c "import plistlib,sys; print(plistlib.load(open(sys.argv[1],'rb'))['TeamIdentifier'][0])" "$TAM")
rm -f "$TAM"
if [ -z "$DOI" ]; then
  echo "❌ không đọc được TeamIdentifier từ $HO_SO"
  exit 1
fi
echo "✅ đội trong hồ sơ: $DOI"

cat > target/release/bundle/tcc.entitlements <<XML
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>com.apple.application-identifier</key><string>$DOI.com.tcc.browser</string>
  <key>com.apple.developer.team-identifier</key><string>$DOI</string>
  <key>keychain-access-groups</key>
  <array><string>$DOI.com.tcc.browser</string></array>
</dict></plist>
XML
codesign -f -o runtime -s "$CHUNG_THU" \
  --entitlements target/release/bundle/tcc.entitlements "$GOI" || exit 1
codesign -dv --entitlements - "$GOI" 2>&1 | tail -6
echo "✅ đã ký"
