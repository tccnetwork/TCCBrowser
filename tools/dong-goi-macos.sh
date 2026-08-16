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
cargo build --release -p tcc-browser --features window || exit 1

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
</dict></plist>
PLIST
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
cat > target/release/bundle/tcc.entitlements <<'XML'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>keychain-access-groups</key>
  <array><string>$(AppIdentifierPrefix)com.tcc.browser</string></array>
</dict></plist>
XML
codesign -f -o runtime -s "$CHUNG_THU" \
  --entitlements target/release/bundle/tcc.entitlements "$GOI" || exit 1
codesign -dv --entitlements - "$GOI" 2>&1 | tail -6
echo "✅ đã ký"
