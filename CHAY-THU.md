# Chạy thử trình duyệt TCC

Bản dựng: `v2/target/release/tcc-browser` (và `target/release/bundle/TCCBrowser.app`).

## Mở gói ví dụ — đây là thứ đáng xem nhất

```bash
cd /Volumes/DATA/TCCBrowser/v2
./target/release/tcc-browser examples/hello-tcc
```

Cửa sổ mở ra sau khi **chữ ký được kiểm xong**. Nếu chữ ký hỏng thì không có
cửa sổ nào — không phải cửa sổ báo lỗi, mà là không dựng ra.

Trong đó có một ô để **gõ thử tiếng Việt** (Telex của macOS). Đó là cổng duy
nhất cần một con người: mọi cách giả lập đều bơm chuỗi đã hoàn chỉnh vào ô nhập,
tức là bỏ qua đúng cái cần đo.

## Xem quyền đã cấp

```bash
./target/release/tcc-browser quyen examples/hello-tcc
```

## Thử phá — năm ví dụ đối kháng

```bash
cargo run -p tcc-shell --features window --example kiem-khoi-tan-cong
cargo run -p tcc-shell --features window --example kiem-khoi-tan-cong chi-csp
cargo run -p tcc-shell --features window --example kiem-bam-nut ma      # nút ma
cargo run -p tcc-shell --features window --example kiem-bam-nut ct-ma   # công tắc ma
```

Chúng in ra **số đối chứng hai chiều**, không assert suông: số nút ta dựng, số
nút WebKit nhìn thấy, số thẻ kịch bản còn sống.

## Chống ký mù — không cần ví, không cần khoá

```bash
# lấy một giao dịch thật từ testnet
curl -s -X POST https://rpc2.tcc-coin.com -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"tcc_buildUnsignedTransfer",
       "params":["<ĐỊA-CHỈ>","<ĐỊA-CHỈ>","1000000000000000000","thu"]}'

# rồi đưa hai trường vào
cargo run -p tcc-shell --example kiem-chong-ky-mu -- <unsigned_tx_hex> <signing_message_hex>
```

## ⚠️ Ví CHƯA chạy được, và vì sao

`TCCBrowser.app` dựng xong nhưng **chưa ký** — nên không có ví.

`USER_PRESENCE` (bắt Touch ID cho từng lần lấy khoá) cần quyền
`keychain-access-groups`, mà quyền ấy cần một **hồ sơ cấp phép macOS**. Thiếu nó
thì ký kèm entitlements làm tiến trình **treo im lặng**, không báo lỗi — xem
`docs/vi-thiet-ke.md` §19.

Cần làm trên tài khoản Apple Developer:

1. Tạo App ID `com.tcc.browser`, bật Keychain Sharing
2. Tạo hồ sơ cấp phép macOS, tải về
3. `TCC_PROVISION_PROFILE=… TCC_SIGN_IDENTITY=… tools/dong-goi-macos.sh`

Cho tới lúc ấy, ví **cố ý không chạy** thay vì chạy với bảo vệ yếu hơn mức nó
hứa — `wallet_store.rs`.
