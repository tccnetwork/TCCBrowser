# Ví — thiết kế và mô hình đe doạ (Giai đoạn 3.1)

> Tài liệu này viết **TRƯỚC** khi có mã, và nó tồn tại để nói rõ thứ kho khoá
> của hệ điều hành **không** làm được. Một tính năng bảo mật bị hiểu quá lên là
> nguy hiểm hơn không có nó, vì người ta dựa vào thứ không có ở đó.
>
> Cổng chặn cứng, không đổi: **không giao dịch mainnet nào trước khi qua kiểm
> định an ninh độc lập.**

## 1. Sự thật phần cứng quyết định cả thiết kế

**Secure Enclave của Apple KHÔNG giữ được khoá của TCC.**

Nó chỉ hỗ trợ đường cong NIST P-256. Ed25519 không, ML-DSA-65 càng không. Không
có cách nào sinh hay giữ một khoá ký TCC bên trong phần cứng ấy, và không có
lệnh nào bảo nó ký hộ.

Hệ quả không tránh được:

> **Khoá bí mật PHẢI nằm trong bộ nhớ tiến trình vào lúc ký.**

Kho khoá bảo vệ khoá **lúc nằm yên**. Nó không bảo vệ khoá **lúc dùng**. Mọi
câu chữ trong giao diện phải phản ánh đúng điều đó — xem §5.

TPM trên Windows cũng thế: nó ký được RSA và ECDSA P-256/384, không ký được
Ed25519 hay ML-DSA. Bài toán giống hệt trên cả hai hệ.

## 2. Vậy kho khoá còn đáng dùng không

Còn, và đây là những gì nó thật sự mua được:

| Chặn được | Vì sao |
|---|---|
| Ứng dụng khác trên máy đọc trộm tệp khoá | Keychain có danh sách kiểm soát truy cập gắn theo ứng dụng, không phải quyền tệp |
| Khoá theo bản sao lưu ra khỏi máy | Đánh dấu `ThisDeviceOnly` là iCloud Keychain không đồng bộ nó |
| Đọc đĩa khi máy đang khoá | FileVault cộng Keychain ở trạng thái khoá |
| Ký lén khi người dùng không có mặt | Yêu cầu **xác thực người dùng cho từng lần ký** (Touch ID / mật khẩu) |

Mục cuối là mục đáng giá nhất, và là mục dễ bị bỏ quên nhất khi cài đặt.

## 3. Những gì nó KHÔNG chặn — nói thẳng

- **Mã độc chạy dưới chính tài khoản người dùng**, gắn trình gỡ lỗi vào tiến
  trình trình duyệt đúng lúc ký. Khoá đang ở RAM; nó đọc được.
- **Bản trình duyệt TCC đã bị sửa.** Nếu nhị phân của ta bị thay, kho khoá cấp
  khoá cho nó y như cấp cho bản thật — danh sách kiểm soát truy cập nhận diện
  ứng dụng, mà ứng dụng đã bị chiếm.
- **Quyền root.**
- **Người dùng bị lừa bấm đồng ý.** Kho khoá không đọc hộ nội dung giao dịch.

Ba mục đầu chỉ có phần cứng ký hộ mới chặn được, mà phần cứng ấy không hỗ trợ
thuật toán của ta (§1). Mục thứ tư thuộc về màn hình xác nhận giao dịch, tức là
mục 3.2 của kế hoạch, không phải mục này.

## 4. Bọc khoá bằng Secure Enclave — có đáng không

Có một đường vòng: sinh một khoá P-256 **trong** Secure Enclave, dùng nó để mã
hoá khoá TCC. Khoá TCC lúc nằm yên được bảo vệ bằng khoá phần cứng không trích
xuất được.

**Đáng làm hay không tuỳ vào thứ nó thêm được, và nó thêm ít hơn vẻ ngoài.**
Kẻ tấn công vẫn phải chạy được mã dưới tài khoản người dùng để gọi giải mã —
mà nếu chạy được thì §3 đã thua rồi. Cái nó thật sự thêm: khoá TCC đã mã hoá
mà rời khỏi máy (sao lưu, ổ cắm ngoài, ảnh đĩa) thì vô dụng ở máy khác.

Kết luận: **để sau**. `ThisDeviceOnly` mua được phần lớn lợi ích đó với một
phần nhỏ độ phức tạp, và độ phức tạp thêm vào lớp mật mã là thứ phải trả bằng
một vòng kiểm định nữa.

## 5. Luật cho giao diện

Cùng hạng với luật "không bao giờ hiện *đã xác minh nhà phát hành*":

> **KHÔNG được nói khoá "được phần cứng bảo vệ" hay "nằm trong Secure Enclave".**
> Nó không nằm ở đó, và không thể. Câu đúng là *"khoá được kho khoá của hệ điều
> hành giữ, và cần bạn xác nhận mỗi lần ký"*.

Sẽ có phép thử quét bảng dịch tìm những cụm bị cấm, đúng lối đã làm với
`khong_chuoi_nao_noi_da_xac_minh_nha_phat_hanh`.

## 6. Hình dạng mã

Theo đúng lối `tcc-net` đã đi, vì nó đã chứng minh được giá trị:

```
tcc-shell  ──tiêm──▶  trait Keystore
                          ▲
              ┌───────────┴───────────┐
        tcc-keystore              bản GIẢ trong phép thử
     (Keychain / DPAPI)        (trong bộ nhớ, không chạm hệ điều hành)
```

- **Crate riêng** `tcc-keystore`, và **chỉ `tcc-shell` được phụ thuộc nó** —
  luật kiến trúc mới, cùng hình dạng với luật 8. Đọc `Cargo.toml` là thấy ngay
  bộ nạp ứng dụng không với tới được khoá.
- **Cờ tính năng riêng**, để dựng được bản trình duyệt **không có ví**. Bản đó
  hữu ích khi soi bảo mật: chạy nó thì chắc chắn không byte khoá nào được đọc,
  dù mã có lỗi gì.
- **Bản giả trong phép thử** không chạm hệ điều hành, nên phần logic kiểm được
  trên cả ba hệ mà không cần Keychain.
- Khoá bí mật đi qua một kiểu **tự xoá khi rời phạm vi** (`zeroize`), và **không
  bao giờ** có `Debug` hay `Display` in ra nội dung.

## 7. Ba câu hỏi — ĐÃ CÓ LỜI ĐÁP (15/08/2026)

Đọc `/Volumes/DATA/TCC/web-login/wallet` (ví web đang chạy) và
`/Volumes/DATA/Claude/blockchain/tcc-chain/v4` (nút chuỗi, beta3).

### 7.1 Chuỗi nhận chữ ký dạng nào

**ML-DSA-65 THUẦN, KHÔNG lai.** `DECISIONS-IRREVERSIBLE.md` D1 chốt: crate
`ml-dsa`, mức `MlDsa65`, FIPS 204 **bản cuối**. Mã trong `dilithium3/src/lib.rs`
ghi rõ *"ML-DSA-65 deterministic sign (empty context, pure mode)"*, chữ ký 3309
byte.

**Vậy nửa hậu lượng tử của `tcc-crypto` khớp byte với chuỗi.** Cùng crate, cùng
FIPS 204 bản cuối, cùng biến thể pure với `ctx` RỖNG, cùng ký tất định. Chính là
quyết định tôi đã ghi vào `spec/0.1/03-signature.md` — và hai bên đi tới nó độc
lập, vì cùng đọc một tiêu chuẩn.

> **Nhưng đây là HAI bộ ký, không phải một.** Gói TCC ký **lai**
> (Ed25519+ML-DSA); giao dịch chuỗi ký **thuần** ML-DSA. Khoá ví và khoá ký gói
> **KHÔNG được là một khoá** — tách khoá theo mục đích là luật cơ bản, và ở đây
> nó còn tự nhiên vì hai định dạng vốn đã khác nhau.

Băm: chuỗi dùng **BLAKE3** (D2), địa chỉ = `BLAKE3(pubkey)` 32 byte raw (D3),
hiển thị dạng hex `0x` + 64 ký tự (D4). Cùng hàm băm với băm nội dung gói.

### 7.2 Ví web lưu khoá thế nào

`localStorage["tcc_wallets_v4"]`, mỗi ví gồm `{address, label, pubkeyHex,
encryptedPrivkey, salt, iv}`. Mã hoá: **PBKDF2-SHA256 100.000 vòng → AES-GCM-256**,
mở bằng **mã PIN**. Khoá bị xoá khỏi bộ nhớ ngay sau mỗi lần ký (`wipeBytes`).

Có BIP39 trong `vendor/`, nên nhiều khả năng có cụm từ khôi phục — **cần xác
nhận** trước khi hứa "mang ví sang được".

⚠️ **PBKDF2 100k với một mã PIN là yếu.** PIN 6 chữ số chỉ có một triệu khả
năng; ai lấy được `localStorage` (XSS, hoặc đọc đĩa) thì dò cạn được. Con số
100.000 vòng hợp lý cho **mật khẩu**, không hợp lý cho **PIN**. Trình duyệt TCC
nếu nhập khoá từ ví web thì phải **cất lại bằng kho khoá hệ điều hành**, chứ
không chép nguyên cách bảo vệ ấy sang.

### 7.3 Địa chỉ và ký giao dịch

`signing_message()` trong `dilithium3/src/lib.rs` là **BLAKE3** của:

```text
nonce(BE) ‖ from(32) ‖ to(32) ‖ amount(BE) ‖ gas_price(BE)
          ‖ gas_limit(BE) ‖ timestamp(LE) ‖ payload.hash()
```

Hai điều đáng nói, và cả hai đều thuộc về chuỗi chứ không thuộc trình duyệt:

**(a) `timestamp` dùng LITTLE-endian trong khi mọi trường khác BIG-endian.**
Chạy được vì hai bên cùng làm thế, nhưng đó đúng là loại chi tiết làm vỡ bản cài
đặt thứ hai — cùng lớp với bẫy giao diện FIPS 204 mà dự án này đã dẫm.

**(b) `chain_id` KHÔNG có trong thông điệp ký.** `DECISIONS-IRREVERSIBLE.md` D6
nói `chain_id` phải đi vào *"tx-signing domain + chống replay cross-chain"* và
đánh dấu **CẦN-VERIFY**. Mã hiện chưa có nó. Nên một giao dịch đã ký về nguyên
tắc **phát lại được sang mạng TCC khác** cùng địa chỉ và nonce. Tài liệu quyết
định đã biết; mã thì chưa theo kịp — cần chốt **trước genesis**, vì sau đó là
bất khả nghịch.

### 7.4 Và một điều về CHÍNH trình duyệt

Ví web gọi `tcc_buildUnsignedTransfer` rồi **ký thẳng `signing_message_hex` do
máy chủ RPC trả về**. Vì thông điệp ấy là một **băm 32 byte**, ví không kiểm
được nó ứng với giao dịch nào.

> Người dùng thấy *"gửi 5 TCC cho X"* trên màn hình, nhưng ký một chuỗi băm mà
> máy chủ đưa. Một RPC bị chiếm có thể trả về băm của một giao dịch khác hẳn.

Đây là **ký mù**, và nó cho trình duyệt TCC một lý do tồn tại rất cụ thể: máy
chủ có trả về cả `unsigned_tx_base64`, nên trình duyệt **giải mã được, tự tính
lại `signing_message`, so khớp, rồi mới ký** — và hiện ra các trường đã giải mã
chứ không hiện lại thứ người dùng vừa gõ.

Đó chính là mục 3.2 của kế hoạch, và giờ nó không còn là "màn xác nhận cho đẹp"
mà là **thứ vá một lỗ ký mù**.

## 8. Những gì CÒN phải hỏi người

Chuỗi TCC đã có ví web đang chạy (`network3.tcc-coin.com`), và luật của dự án
nói **không viết lại phần mật mã đã có**.

Nên trước khi viết một dòng nào chạm khoá thật, cần biết:

1. Chuỗi TCC nhận chữ ký **dạng nào** — Dilithium thuần, hay lai như định dạng
   gói? Nếu khác định dạng của `tcc-crypto` thì đây là **hai** bộ ký, không phải
   một, và tài liệu phải nói rõ.
2. Khoá ví hiện được sinh và lưu **thế nào** ở ví web? Người dùng có ví sẵn phải
   mang được sang, nếu không thì "ví" trong trình duyệt là một ví thứ hai, và
   đó là một quyết định sản phẩm chứ không phải chi tiết cài đặt.
3. Có sẵn tài liệu về định dạng địa chỉ và ký giao dịch không?

1. **Có cụm từ khôi phục BIP39 không**, và nếu có thì đường dẫn dẫn xuất là gì?
   Cần biết trước khi hứa với người dùng rằng ví mang sang được.
2. **`chain_id` sẽ chốt số nào**, và có vào thông điệp ký trước genesis không?
   Trình duyệt phải ký đúng thứ chuỗi kiểm.
3. **Trình duyệt nhập ví cũ hay tạo ví mới?** Nhập thì phải đọc được định dạng
   `tcc_wallets_v4` và giải mã bằng PIN — tức là trình duyệt phải cài đặt lại
   PBKDF2+AES-GCM chỉ để đọc một lần rồi cất lại bằng kho khoá hệ điều hành.
