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

### 7.3 Địa chỉ và ký giao dịch — VÀ MỘT ĐÍNH CHÍNH

⚠️ **Bản đầu của mục này SAI, vì tôi đọc nhầm tệp.** Tôi dựa vào
`dilithium3/src/lib.rs` — đó là **SDK WASM cũ**. Nguồn sự thật của v4 là
`src/tx/signing.rs`, tệp tự ghi *"single source of truth"*.

Hai điều tôi đã báo là **không đúng với v4**:

| Tôi đã nói | Thật ra |
|---|---|
| `chain_id` KHÔNG có trong thông điệp ký | **CÓ.** `chain_id u64 LE`, ngay sau `version` |
| `timestamp` little-endian lệch pha với các trường big-endian | **Toàn bộ little-endian.** Không có gì lệch |

Và v4 còn có hai thứ tốt hơn tôi tưởng: một **bộ tách miền** `"tcc/v1/tx"` đứng
đầu thông điệp — thứ mà chữ ký GÓI của TCC hiện **chưa có**, đáng mượn — và
`expires_at` cũng nằm trong băm, nên máy chủ không kéo dài hạn một giao dịch đã
ký được.

Công thức đúng, **đã neo bằng một giao dịch thật lấy từ testnet chain 91338**:

```text
BLAKE3( "tcc/v1/tx" ‖ version(u32) ‖ chain_id(u64) ‖ from(32) ‖ to(32)
        ‖ nonce(u64) ‖ amount(u128) ‖ gas_price(u64) ‖ gas_limit(u64)
        ‖ timestamp(i64) ‖ expires_at(i64) ‖ BLAKE3(0x01 ‖ memo) )
```

Mọi số little-endian. `0x01` là `Payload::TAG_TRANSFER` — **khác** con số biến
thể trên dây (là `0`); hai không gian số riêng biệt, lẫn chúng là ra băm sai mà
vẫn "chạy".

Chuỗi còn có **v2** với bộ tách miền riêng `"tcc/v2/tx"`, bỏ `nonce` và
`gas_price`, thêm `recent_blockhash` và `priority_fee`. Trình duyệt hiện chỉ
hiểu v1; khi mạng chuyển sang v2 thì phải theo, và `version` trong gói tin là
thứ nói cho ta biết.

**Bài học, và nó thuộc về tôi.** Tôi đã báo cho bạn một "phát hiện bảo mật"
dựa trên một tệp không phải nguồn sự thật, và suýt để đội chuỗi đi sửa một thứ
không hỏng. Thứ cứu lại là đòi hỏi một **mốc ngoài** trước khi tin bố cục byte —
đúng nguyên tắc dự án này áp cho mọi thứ khác. Mẫu thật cho tôi biết mình sai
trong vòng một phút.

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


## 9. Đã dựng phần chống ký mù (15/08/2026)

`crates/tcc-chain` — **đọc, không ký**. Không khoá nào đi qua nó.

Nó giải mã `unsigned_tx_base64` (bincode 1.x: số nguyên little-endian cố định
độ dài, `Vec`/`String` có tiền tố `u64`, biến thể enum là `u32`), **tự tính lại**
`signing_message` theo đúng công thức của chuỗi, và **từ chối ký nếu lệch**.

Phép thử quan trọng nhất là `may_chu_doi_nguoi_nhan_thi_bi_bat`: máy chủ đưa băm
của giao dịch người dùng NGHĨ mình đang ký, còn gói tin lại là của kẻ gian. Ví
web hôm nay sẽ ký; phép kiểm này từ chối.

Và `moi_truong_deu_vao_thong_diep_ky` đòi rằng đổi **bất kỳ** trường nào cũng
đổi băm — một trường không vào băm là một trường máy chủ sửa được mà không ai
biết.

### ⚠️ Chưa có mốc ngoài — đây là điểm yếu lớn nhất của phần này

Bố cục byte trên đây **chép từ mã của chuỗi**, chưa đối chiếu với một giao dịch
THẬT. Nếu tôi đọc sai `bincode`, trình duyệt sẽ từ chối mọi giao dịch hợp lệ —
hỏng về phía an toàn, nhưng vẫn là hỏng, và chỉ lộ ra lúc có người dùng thật.

Cần **một mẫu thật** để neo, và có hai đường lấy:

1. Mở ví web, bật DevTools, chép lại một cặp `unsigned_tx_base64` +
   `signing_message_hex` từ lời gọi `tcc_buildUnsignedTransfer`. Không cần ký,
   không cần gửi gì.
2. Cho phép tôi gọi thẳng `tcc_buildUnsignedTransfer` trên RPC testnet. Đó là
   lời gọi CHỈ ĐỌC — nó dựng giao dịch chứ không phát tán — nhưng nó là một
   lượt gọi ra hạ tầng của bạn, nên tôi hỏi trước.

Có mẫu rồi thì nó thành vector kiểm định, và phần này mới đáng tin.


## 10. Keychain thật đã cài (15/08/2026)

`crates/tcc-keystore/src/macos.rs`, sau cờ `os-keystore`. Giá: **11 crate** thêm
vào (9 → 20). So với `wry` là 71, đây là rẻ.

Ba thứ được đặt, và mục đầu là mục cả kho khoá tồn tại vì nó:

| Đặt gì | Chặn được |
|---|---|
| `USER_PRESENCE` | Ký lén — hệ điều hành hỏi Touch ID hoặc mật khẩu **cho từng lần lấy khoá** |
| `set_access_synchronized(Some(false))` | Khoá theo iCloud sang máy khác |
| Tự chặn ghi đè | Keychain ghi đè **im lặng**; ghi đè khoá ví là mất tiền vĩnh viễn |

Dùng `USER_PRESENCE` chứ không `BIOMETRY_ANY`: máy không có cảm biến vân tay sẽ
không mở nổi ví, mà "không mở được ví" hỏng nặng hơn "phải gõ mật khẩu".

`contains` **không** đặt cờ ấy — bắt chạm Touch ID chỉ để biết "đã có ví chưa"
là cách nhanh nhất dạy người dùng chạm bừa mọi hộp thoại.

### ⚠️ Một hạn chế đã biết

**`Purpose` không tới được hộp thoại của hệ điều hành.** API này không cho
truyền chuỗi xuống, nên macOS hiện câu chung chung của riêng nó. Lý do vì thế
phải hiện ở **màn hình của ta**, ngay trước khi gọi `unlock` — đó là việc của
`transaction_screen`, và nó đã làm.

Tham số `Purpose` vẫn giữ, có chủ ý: nó ép bên gọi phải nghĩ ra lý do, và ngày
API cho phép truyền xuống thì không phải đổi chữ ký hàm.

### Lỗi được phân loại, không gộp

`NotFound` / `UserRefused` / `Os` là ba câu khác nhau với người dùng: "bạn chưa
có ví" và "bạn vừa bấm huỷ" không được hiện ra như nhau. Gộp cả ba thành một lỗi
là giao diện mất khả năng nói điều đúng.

### Còn chờ quyết định

Nhập ví cũ từ `tcc_wallets_v4` hay chỉ tạo ví mới — xem §8. Bản thân kho khoá
không phụ thuộc câu trả lời ấy, nên nó làm xong trước.


## 11. Sinh khoá ví — ĐÃ CHỐT: giống ví web từng byte (15/08/2026)

Câu hỏi §8.3 (*"nhập ví cũ hay tạo ví mới?"*) hoá ra đặt sai. Ví cũ và ví mới
**không phải hai loại ví**:

| | Ví web `tcc_wallets_v4` | Ví trong trình duyệt |
|---|---|---|
| Thuật toán | ML-DSA-65 | y hệt |
| Địa chỉ | `BLAKE3(pubkey)` | y hệt |
| Chuỗi | 91338 | y hệt |
| **Cất khoá** | `localStorage`, PIN → PBKDF2 100k → AES-GCM | **Keychain, `USER_PRESENCE`** |

Chỉ dòng cuối khác. Nên nguyên tắc là: **giống nhau ở chỗ sinh khoá, khác nhau ở
chỗ cất khoá.**

### Giống ở đâu

`crates/tcc-chain/src/wallet.rs` bám đúng chuỗi dẫn xuất của chuỗi TCC:

```text
chuỗi hạt giống (ASCII) → BLAKE3 derive_key("tcc_chain_2026_seed_v1", …)
                        → 32 byte → ML-DSA-65 KeyGen → pubkey 1952 byte
                        → BLAKE3(pubkey) = địa chỉ
```

**Neo bằng chương trình của đội chuỗi, không phải bằng mã của mình.**
`tcc-keygen address --seed hello` in ra `0x6c1be53f…`; phép thử
`dia_chi_khop_chuong_trinh_cua_doi_chuoi` đòi đúng con số ấy. Đây là bài học
§7.3: phép thử tự so mã của mình với chính nó thì không chứng minh được gì.

Đột biến để kiểm phép kiểm — đổi bối cảnh KDF một ký tự, và đổi
`BLAKE3(pubkey)` thành "32 byte đầu của pubkey". Cả hai đều đỏ, và cái thứ hai
**chỉ mỗi phép thử neo bắt được** — tám phép thử còn lại vẫn xanh. Neo ngoài là
thứ chịu lực ở đây.

### Khác ở đâu, và vì sao không chép sang

Ví web dùng PIN, sàn `MIN_PIN_LENGTH = 6`. PIN sáu chữ số là **một triệu** khả
năng; PBKDF2-SHA256 100k vòng ≈ 2×10⁵ phép nén mỗi lần đoán, một card đồ hoạ đời
mới chạy cỡ 10¹⁰ phép nén mỗi giây → **dò cạn trong khoảng vài chục giây**. Con
số ấy là tính, chưa đo — nhưng sai một bậc thì vẫn là *vài phút*.

Số vòng không cứu được: bí mật chỉ có một triệu khả năng thì làm chậm bao nhiêu
cũng vẫn đếm hết. Và **đó không phải lỗi của ví web** — trang web không có lựa
chọn nào khác. Trình duyệt thì có, và nếu chép cách cất khoá ấy sang thì trình
duyệt chỉ là **thêm một bản sao của cùng điểm yếu, nằm trên đĩa**.

### Hai đường dẫn xuất, và cái bẫy đã làm chuỗi đứng im

Cùng 32 byte có hai đường vào: **băm chuỗi** (`from_seed_phrase`, đường của ví
web) và **byte nguyên xi** (`from_raw_seed`, đường của `node.key_seed`). Đưa
cùng một giá trị vào hai đường ra hai ví khác nhau — đội chuỗi mất nhiều giờ vì
đúng chỗ này ngày 30/07/2026, nút ký phiếu bằng khoá không phải khoá đã đăng ký
và mạng dừng chốt khối **không in ra một dòng lỗi nào**.

Nên đây là **hai hàm tên khác nhau**, không phải một hàm với một cờ `bool`.
Chọn nhầm tên hàm thì đọc mã là thấy; chọn nhầm một `bool` thì không. Phép thử
`bam_chuoi_va_byte_nguyen_xi_ra_hai_vi_khac_nhau` tồn tại để ngày ai đó "dọn
dẹp" hai hàm thành một thì nó đỏ.

### ⚠️ Phải nói với đội chuỗi: đây KHÔNG phải BIP39 chuẩn

Ví web làm `mnemonicToEntropy → hex → generate_keypair`. BIP39 chuẩn là
`mnemonic → PBKDF2-2048 → seed 64 byte`. Ví web lấy **thẳng entropy** làm chuỗi
hạt giống, nên 24 từ ấy **không ví nào khác khôi phục được**, kể cả ví ghi "hỗ
trợ BIP39".

Không sai — nhưng phải ghi vào tài liệu là *"24 từ theo từ điển BIP39, dẫn xuất
riêng của TCC"*, đừng để người dùng tưởng mang đi đâu cũng được.

Và một chi tiết nhỏ mà mất ví: hex sinh bằng `toString(16)` là **chữ thường**.
Ai "chuẩn hoá" cụm từ khôi phục thành chữ hoa là ra ví khác. Ghim bằng phép thử
`hex_chu_hoa_ra_vi_khac`, không ghim bằng lời dặn.

### Còn lại

1. **BIP39 24 từ → entropy** trong Rust (cần từ điển 2048 từ + SHA-256).
2. **Nhập ví cũ**: đọc `tcc_wallets_v4`, giải mã PBKDF2+AES-GCM bằng PIN đúng
   một lần, rồi cất lại vào Keychain. Tốn công cài lại phần mật mã ấy chỉ để
   đọc một lần — nhưng không có nó thì người đang có ví bị bỏ lại.


## 12. Cụm từ khôi phục 24 chữ (15/08/2026)

`crates/tcc-chain/src/mnemonic.rs`. Đường đầy đủ đã nối xong:

```text
24 chữ → entropy 32 byte → hex CHỮ THƯỜNG → BLAKE3 KDF → ML-DSA-65 → địa chỉ
```

### Neo bằng hai chương trình không biết nhau

| Bước | Neo vào |
|---|---|
| chữ ↔ entropy | `@scure/bip39@1.3.0` — **thư viện đóng gói trong chính ví web** |
| entropy → địa chỉ | `tcc-keygen` — **chương trình của đội chuỗi** |

Phép thử `cum_tu_ra_dia_chi_khop_chuong_trinh_cua_doi_chuoi` đi hết cả hai
bước. Không chương trình nào trong hai cái ấy biết gì về mã ở đây.

### Từ điển được ghim bằng băm

`data/bip39-english.txt` lấy từ ví web, và băm SHA-256 của nó **khớp đúng
`english.txt` công bố trong kho `bitcoin/bips`**:

```
2f5eed53a4727b4bf8880d8f3f199efc90e58503646d9ff8eff3a2ed3b24dbda
```

Đáng làm phép kiểm ấy: một từ điển bị đổi vài từ là **cửa hậu trộm ví hoàn toàn
im lặng** — cụm từ vẫn "hợp lệ", chỉ ra một ví khác. Kết quả cũng là một tin
tốt cho ví web: bản đóng gói ở đó không bị sửa.

### ⚠️ Chỗ trình duyệt CỐ Ý khác ví web

Ví web, khi 24 chữ sai tổng kiểm, mời *"Try as raw seed?"* rồi coi cả cụm từ
như một chuỗi hạt giống thô — **nút đi tiếp vẫn bật**. Nó có chắn: màn xác nhận
hiện địa chỉ **và số dư** trước khi đi tiếp, nên gõ nhầm thì thấy số dư 0. Không
phải im lặng, nhưng yếu hơn hẳn tổng kiểm.

Trình duyệt **không làm đường lùi ấy**. Tổng kiểm sinh ra đúng để bắt một chữ
gõ nhầm; biến nó thành lời gợi ý là vứt đi thứ duy nhất phân biệt *"bạn gõ
nhầm"* với *"đây là ví khác"*. Hạt giống thô vẫn nhập được, nhưng phải là lựa
chọn người dùng **tự bấm**, không phải đề nghị hiện ra đúng lúc họ bối rối.

### Lỗi báo VỊ TRÍ, không báo chữ

`UnknownWord(24)` chứ không `UnknownWord("khongcotutrongtudien")`. Người dùng
cần biết gõ nhầm chữ thứ mấy; nhật ký không cần biết chữ ấy là gì. Một mẩu cụm
từ khôi phục lọt ra tệp nhật ký vẫn là một mẩu cụm từ khôi phục lọt ra ngoài.
Ghim bằng `chu_la_bao_vi_tri_chu_khong_bao_chu`.

### Không một `expect` nào trên đường sinh khoá

Bản đầu có bốn chỗ `expect` kiểu *"chỉ số luôn < 2048"*. Đều đúng — và đều là
một nhánh hoảng loạn không bao giờ chạy, nằm trong mã đụng tới khoá. Viết lại
để chúng không tồn tại: đếm chỉ số bằng `u32` ngay từ đầu, lấy byte thấp bằng
`to_le_bytes()[0]` thay vì đổi kiểu, và viết tay bảng hex bốn dòng thay cho
`write!`.

### Đột biến

| Đổi gì | Bắt được? |
|---|---|
| hex thành CHỮ HOA | ✅ — nhưng **chỉ mỗi phép thử neo địa chỉ** |
| bỏ kiểm tổng kiểm | ✅ |
| lệch một dòng trong từ điển | ✅ |

Lần nữa, phép thử neo ngoài là thứ chịu lực. Tám phép thử "hợp lý" quanh nó vẫn
xanh khi hex viết hoa.


## 13. Nhập ví cũ từ ví web (16/08/2026)

`crates/tcc-chain/src/import.rs`, sau cờ `import-web-wallet`. Giá: **14 crate**
thêm vào cả workspace (103 → 117). Đắt cho một việc người dùng làm đúng một lần
trong đời, nên nó nằm sau cờ chứ không nằm trong bản dựng của mọi người.

```text
PIN → PBKDF2-SHA256 100.000 vòng (muối 16 byte) → khoá AES-256
    → AES-GCM (IV 12 byte) → 32 byte hạt giống ML-DSA
```

Không vi phạm luật *"không viết lại phần mật mã đã có"*: `pbkdf2` và `aes-gcm`
là crate của RustCrypto, ta chỉ gọi. Thứ phải viết là **bộ đọc một định dạng**.

### Neo bằng bản ghi ví web THẬT

`data/vi-web-mau.json` dựng bằng đúng WebCrypto mà trang web gọi (`node:crypto`,
cùng tham số), không phải do tôi bịa ra từ việc đọc mã. Phép thử đi hết đường:
JSON → PIN → hạt giống → **địa chỉ khớp `tcc-keygen`**. Cụm từ khôi phục giải ra
cũng phải mở đúng ví ấy.

Muối và IV trong tệp mẫu **cố định** để bản dựng lặp lại được; bản ghi thật dùng
`crypto.getRandomValues`. Dùng lại muối/IV chỉ chấp nhận được trong một tệp mẫu
công khai không giữ tiền của ai.

### Giải mã được KHÔNG có nghĩa là đúng ví

Hạt giống lấy ra được dẫn xuất lại thành địa chỉ và **so với địa chỉ ghi trong
bản ghi**; lệch là từ chối. Cùng nguyên tắc với màn xác nhận giao dịch: kiểm
trước, tin sau.

### Kiểm đột biến tìm ra một lỗ thật

| Đổi gì | Bắt được? |
|---|---|
| bỏ phép kiểm địa chỉ | ✅ |
| lệch một vòng PBKDF2 (100.000 → 99.999) | ✅ (4 phép thử đỏ) |
| **bỏ phép kiểm độ dài khoá** | ❌ **KHÔNG** |

Cái thứ ba là lỗ thật. Ví sinh **trước** lần chuyển sang ML-DSA giữ khoá đã bung
4032 byte; không có phép kiểm ấy thì `copy_from_slice` **hoảng loạn** thay vì
báo lỗi sạch — và không phép thử nào chạm tới. Đã dựng bản ghi mẫu thứ hai
(`vi-web-mau-khoa-cu.json`, khoá 4032 byte, cùng WebCrypto) và phép thử mới giết
được đột biến ấy, đúng ở dòng `copy_from_slice`.

Câu lỗi cũng phải nói được **vì sao**: *"ví này cũ hơn bản ML-DSA"*, không phải
*"hỏng"*.

### ⚠️ Nhập KHÔNG làm bản cũ biến mất

Ví web vẫn giữ nguyên bản của nó trong `localStorage`, vẫn khoá bằng PIN, vẫn
yếu đúng như trước. Trình duyệt **không đụng vào dữ liệu ấy** — xoá hộ người
dùng thứ họ chưa bảo xoá là một cách hỏng riêng, và tệ hơn nếu bản nhập sang có
vấn đề.

Nên giao diện **phải nói ra** sau khi nhập xong: *"ví vẫn còn một bản ở trang
web, vẫn khoá bằng PIN"*. Người dùng tưởng mình đã dọn sạch trong khi bản yếu
vẫn nằm đó là tình huống xấu nhất — mất cảnh giác mà rủi ro không giảm. Đây là
việc của giao diện, **chưa làm**.

### Sai PIN và dữ liệu hỏng là MỘT lỗi

AES-GCM không phân biệt được: thẻ xác thực hỏng theo cùng một cách. Gộp làm một
là trung thực; tách ra là bịa ra thứ mình không biết.

### Mã sau cờ phải được CI biên dịch

`cargo test --workspace` trơn **không** biên dịch mã sau cờ. Đã thêm bước riêng
chạy clippy và phép thử với `--features import-web-wallet` trên cả ba nền. Mã
không được biên dịch trong CI là mã hỏng lúc nào không ai biết.


## 14. Nối vào khung trình duyệt (16/08/2026)

Ba mảnh của 3.1 giờ có chỗ đứng trong `tcc-shell`.

### `wallet_store.rs` — và chỗ TỪ CHỐI cất

`tcc-shell` khai `tcc-keystore` từ lâu nhưng **chưa hề dùng**. Giờ có `open()`,
và luật của nó là: **không có kho khoá hệ điều hành thì trả LỖI**. Không ghi ra
tệp, không mã hoá bằng mật khẩu tự nghĩ ra, không giữ trong bộ nhớ rồi hy vọng.

Ghi khoá ra tệp *"tạm thời cho chạy được"* là đúng thứ trình duyệt này sinh ra
để không làm. Ví web buộc phải làm thế vì trang web không có lựa chọn nào khác;
trình duyệt thì có, và bỏ lựa chọn ấy đi để đỡ vướng là vứt luôn lý do người
dùng đổi sang đây.

> Một tính năng **không chạy** thì người dùng thấy ngay và đi tìm cách khác. Một
> tính năng chạy mà bảo vệ kém hơn họ tưởng thì họ không thấy gì cả, cho tới lúc
> mất tiền.

`tcc_keystore::fake` có sẵn và tiện — chính vì tiện nên nguy hiểm. Tệp này không
nhắc tới nó ở bất kỳ nhánh nào, và phép thử
`khong_co_kho_khoa_thi_tu_choi_chu_khong_lui` đỏ ngày ai đó thêm đường lùi.

Tên mục dẫn từ **địa chỉ**, không từ nhãn người dùng đặt: nhãn đổi được, mà đổi
nhãn rồi mất khoá là cách hỏng không ai đoán ra.

### `import_screen.rs` — tồn tại vì một câu

Màn "xong" có mặt để nói: *"Trang web vẫn giữ một bản của ví này, vẫn khoá bằng
đúng mã PIN cũ."* Câu ấy mang `Emphasis::Warning`, **cùng mức với câu "việc này
chuyển tiền"** ở màn xác nhận giao dịch — và kèm một câu nữa nói người dùng
**làm gì** với bản ấy, chứ không chỉ báo là nó còn đó.

Hai đột biến, cả hai đỏ:

| Đổi gì | Bắt được? |
|---|---|
| hạ câu ấy xuống chữ thường | ✅ |
| bỏ hẳn câu ấy | ✅ |

Màn chọn ví **không cần PIN**: người dùng thấy mình có mấy ví, nhãn gì, địa chỉ
nào, rồi mới quyết định gõ PIN cho cái nào. Bắt gõ PIN trước khi biết mình đang
mở cái gì là dạy người ta gõ PIN vào mọi ô hỏi PIN.

Lỗi được **dịch**, không hiện nguyên văn. `match` toàn phần nên thêm nhánh
`ImportError` mà quên dịch thì không biên dịch được. Bốn nhánh "tệp không đọc
được" gộp làm một câu: người dùng không làm gì khác nhau được với chúng.

### Hai lint được gỡ, có ghi lý do

`label()` dài 147 dòng và có hai nhánh trùng nội dung (`"Cancel"`). Cả hai đều
**cố ý**: cắt bảng dịch nhỏ ra thì trình biên dịch không còn ép phủ hết mọi
nhánh — mà đó là cả lý do khoá là `enum`; và gộp hai nhánh trùng lại thì ngày
một trong hai đổi lời, chuỗi kia đổi theo mà không ai để ý.

### CI biên dịch cả hai cờ

Thêm bước chạy clippy + phép thử cho `tcc-shell` với `import-web-wallet` (cả ba
nền) và với `os-keystore` (chỉ macOS).


## 15. Đường ký thật — chống ký mù được cưỡng chế bằng KIỂU (16/08/2026)

`crates/tcc-chain/src/wallet.rs::sign_transaction` +
`crates/tcc-shell/src/signing_flow.rs`.

### Hàm ký nhận GIAO DỊCH, không nhận băm

Đây là chỗ §7.4 chuyển từ nhận định thành mã. Ví web nhận
`signing_message_hex` từ máy chủ rồi ký thẳng 32 byte ấy — nó **không có gì để
so**, vì băm là tất cả những gì nó có.

Ở đây **không có hàm nào ký một băm.** Muốn ký thì phải cầm được một `Transfer`
đã giải mã, và băm tính TỪ nó. Ai muốn thêm `sign_hash(&[u8; 32])` cho tiện:
đó chính là lỗ hổng, viết lại dưới dạng một hàm tiện dụng.

### `PendingTransaction` — người gác là trình biên dịch

Trường riêng tư, và **không có hàm dựng công khai nào ngoài `review`**. `review`
giải mã, tự tính lại, so, và chỉ sinh ra `PendingTransaction` khi khớp.

> Không viết ra được đoạn mã nào ký một giao dịch chưa qua kiểm — không phải vì
> có ai nhớ luật, mà vì **không có kiểu dữ liệu để cầm**.

Cùng cơ chế `tcc-capability` dùng để chặn một quyền chưa cấp. Có doctest
`compile_fail` chốt rằng dựng thẳng thì không biên dịch được.

`sign` **nuốt `self`**: một lần kiểm là một lần ký. Muốn ký lại thì phải kiểm
lại, và nếu gói tin đã đổi thì lần ấy trượt.

### Kiểm HAI lần, cố ý

`review` kiểm để không sinh ra `PendingTransaction`; `transaction_screen::build`
kiểm lại để không vẽ ra màn hình. Bỏ một trong hai vẫn còn một — đột biến chứng
minh: bỏ phép kiểm ở `review` thì 2 phép thử đỏ, bỏ **cả hai** thì 4 đỏ.

### Chữ ký neo vào một bản cài đặt khác

`chu_ky_khop_ban_cai_dat_python` so **từng byte** với `dilithium-py` ký cùng
thông điệp bằng cùng hạt giống. Chữ ký ML-DSA tất định nên so được.

Không phải "ký rồi tự kiểm lại bằng chính mình" — phép ấy xanh kể cả khi cả hai
chiều cùng sai. Và nếu `ml-dsa` đổi sang ký ngẫu nhiên hoặc lỡ thêm ngữ cảnh thì
phép thử đỏ ngay: đúng cái bẫy liên thông FIPS 204 đã ghi trong
`spec/0.1/03-signature.md`.

### Đây là câu trả lời cho một trong ba câu sản phẩm

Người soát độc lập (16/08/2026) xếp *"rủi ro sản phẩm lớn hơn rủi ro kỹ thuật"*
ở mức cao, vì ba câu trong `ke-hoach.md` vẫn treo — trong đó có *"lập trình viên
được gì mà web thường không cho"*.

Câu ấy giờ **có đáp án**, và nó nằm ở đây: trang web không đóng được khoảng cách
giữa "thứ hiện ra" và "thứ được ký", vì nó không có chỗ đứng nào an toàn hơn
chính máy chủ nó đang nói chuyện. Trình duyệt có.

Đáp án ấy hiện mới là mã có phép thử. **Chưa ai dùng nó** — nên nó trả lời được
câu hỏi, chứ chưa chứng minh được thị trường.


## 16. Gọi RPC thật, và mốc tôi tự dựng đã sập (16/08/2026)

Chạy phép kiểm chống ký mù trên một phản hồi RPC **thật** của testnet 91338,
ví `0x266346…9a71`. Kết quả lần đầu:

```
✗ TỪ CHỐI: còn 49 byte thừa sau khi giải mã xong
```

### Mẫu thử của tôi không phải mẫu của chuỗi

Mẫu cũ dài **148 byte** và dừng ngay sau memo. Máy chủ phát ra **209 byte**.
Chênh đúng 49 byte, và chúng là bốn trường ở đuôi cấu trúc `Transaction`:

| Trường | Byte |
|---|---|
| `signature` (độ dài u64, rỗng) | 8 |
| `public_key` (`Option`, nhãn) | 1 |
| `recent_blockhash` | 32 |
| `priority_fee` | 8 |

Vì sao mẫu cũ khớp mọi phép thử: **tôi tự ráp nó từ các trường**, không lấy
nguyên phản hồi máy chủ. Giá trị băm thì thật — nó khớp — nhưng cái vỏ quanh nó
là của tôi, nên bộ giải mã **chưa từng gặp thứ nó sẽ phải đọc**.

> Một mốc mình tự dựng thì không phải mốc.

Đây đúng là điều người soát độc lập nói bằng cách khác: 110/136 vector do chính
dự án sinh ra nên chúng chứng minh tính nhất quán, chưa chứng minh tính đúng.
Chỗ này là một ví dụ cụ thể, và nó chỉ lộ ra khi chạm vào thứ bên ngoài.

Mẫu bây giờ là **209 byte nguyên văn** máy chủ trả về.

### Và cái đuôi ấy hoá ra là chỗ nguy hiểm nhất

`recent_blockhash` và `priority_fee` là trường của **v2**, và chúng **KHÔNG nằm
trong thông điệp ký của v1**. Nên một máy chủ nhét giá trị vào đó rồi xưng v1
đưa ra một giao dịch có phần **không được chữ ký bảo vệ**: băm vẫn khớp, người
dùng vẫn thấy đúng số tiền, mà giao dịch mang thêm thứ họ không xác nhận.

Bộ giải mã giờ từ chối cả hai (`V2FieldInV1`), và từ chối cả gói tin xưng chưa
ký mà đã mang chữ ký (`AlreadySigned`). Phép thử
`kem_khoa_cong_khai_vao_duoi_khong_doi_bam` chốt rằng đuôi thật sự nằm ngoài
chữ ký — đó chính là lý do phải chặn bằng tay thay vì trông vào băm.

Bản đầu của phép thử ấy **so hai giá trị giống hệt nhau** rồi mang cái tên đó.
Nó xanh, và nó không kiểm gì cả. Viết lại: dựng một gói tin có kèm khoá công
khai 1952 byte ở đuôi, đòi nó giải ra cùng các trường và cùng băm.

### Ví dụ chạy được, không cần khoá

`crates/tcc-shell/examples/kiem-chong-ky-mu.rs` nhận `unsigned_tx_hex` và
`signing_message_hex` của một phản hồi bất kỳ. Toàn bộ phép kiểm chạy xong
**trước khi khoá được dùng tới** — đó là điểm của thiết kế, và nó cũng có nghĩa
là người soát chạy được phép kiểm này mà không cần ví nào.


## 17. Cả vòng, với chuỗi thật (16/08/2026)

`crates/tcc-shell/examples/gui-giao-dich.rs`, sau **cả hai** cờ
`network,os-keystore`.

```
✓ băm tự tính KHỚP băm máy chủ đưa

  người nhận: 0x266346046c9d284e8598a2ed52ac73e31b095da31d16cf1738c96ee3eb5e9a71
  số tiền   : 1 TCC
  phí tối đa: 0.00100000000002 TCC
  mạng      : 91338 (testnet)
  thứ tự    : 174

Việc này CHUYỂN TIỀN và không hoàn tác được.
Gõ đúng chữ `ky` để ký, bất cứ thứ gì khác là huỷ:
```

### Khoá không đi qua dòng lệnh, và không đi qua phiên làm việc

Không có tham số nào nhận hạt giống, cụm từ khôi phục hay mã PIN. Khoá lấy từ
kho khoá hệ điều hành, và hệ điều hành hỏi Touch ID ngay lúc lấy.

Đưa khoá qua tham số dòng lệnh là đưa nó vào lịch sử shell, vào `ps`, và vào
mọi bản ghi terminal — ba chỗ không ai nhớ dọn.

### Thứ tự không đảo được, và không phải vì tôi xếp thế

Bước "kiểm khớp" xong trước bước "ký" vì `sign` đòi một `PendingTransaction`,
và **chỉ `review` sinh ra được nó**. Bước ký nằm ở một hàm riêng nhận đúng kiểu
ấy — đọc chữ ký hàm là thấy ranh giới; đọc một `main` dài trăm dòng thì không.

### `POST` nằm ngoài trait mà ứng dụng thấy

`tcc_runtime::Network` vẫn chỉ có `get`. Một quyền mạng chỉ-`get` yếu hơn hẳn
quyền cho phép đẩy dữ liệu ra ngoài: ứng dụng lấy được thứ nó xin, nhưng không
dựng được một kênh gửi tuỳ ý.

Ví cần `POST`, nên `POST` nằm ở `tcc_net::rpc` — khung trình duyệt gọi được,
ứng dụng không có đường chạm tới. **Thêm `post` vào `Network` cho tiện là âm
thầm nới rộng nghĩa của mọi quyền mạng đã cấp từ trước.**

### Cổng chặn cứng nằm trong mã, không nằm trong lời dặn

Ví dụ này **từ chối chạy trên mọi mạng khác testnet 91338**. Không giao dịch
mainnet nào trước kiểm định an ninh độc lập.

### Còn thiếu đúng một bước

Chưa có giao dịch nào được ký và gửi thật — cần hạt giống nằm sẵn trong
Keychain, và việc cất nó vào là việc của người dùng trên máy của họ. Đường chạy
đã kiểm tới sát chỗ đó: dừng đúng ở `unlock`, với câu chỉ rõ phải cất khoá dưới
tên nào.


## 18. Chạy thật lần đầu, và hai thứ vỡ ra (17/08/2026)

### Không gõ seed thẳng vào cửa sổ được — và đó là cố ý

Đọc ngược nội dung ô nhập từ WebView về Rust **chưa mở trong đường chạy thật**.
`tcc-render-webview/src/window.rs` nói rõ: *"Đọc ngược là một đường dữ liệu mới,
và đường đó chỉ được mở trong một phép kiểm có người ngồi trước máy."*

Nên mở cửa sổ cho người dùng gõ seed vào lúc này là mở một cửa sổ **không có gì
nhận thứ họ gõ**. Đường nhập chạy được hôm nay là `examples/nhap-vi.rs`, nhận
một TỆP: JSON ví web (hỏi PIN, tiếng vọng tắt), 64 hex, hoặc 24 chữ.

Nhận tệp chứ không nhận tham số dòng lệnh: hạt giống trên dòng lệnh là hạt
giống trong lịch sử shell, trong `ps`, và trong mọi bản ghi terminal.

### `USER_PRESENCE` đòi binary được KÝ — bản `cargo run` thì không

```
✗ không cất được: A required entitlement isn't present.
```

`AccessControlOptions::USER_PRESENCE` cần quyền `keychain-access-groups`. Ký
ad-hoc kèm tệp entitlements thì qua được cửa ấy, nhưng rồi **treo** ở chỗ hệ
điều hành muốn hiện hộp thoại xác thực mà không có ngữ cảnh ứng dụng để hiện.

Kết luận thẳng: **kho khoá thật cần một gói ứng dụng đã ký**, không chạy được
từ `cargo run`. Đây là việc đóng gói, và nó chưa có trong kế hoạch.

### Và một LỖI trong mã của tôi, chỉ lộ khi chạy thật

Cất khoá bằng `security add-generic-password` rồi đọc lại:

| Gọi gì | Kết quả |
|---|---|
| `contains` (không điều kiện) | **true** |
| `unlock` (đặt `USER_PRESENCE`) | **"không có khoá nào tên…"** |

Đặt `USER_PRESENCE` lúc **đọc** là một **BỘ LỌC**, không phải một yêu cầu: truy
vấn ấy chỉ nhìn thấy mục đã được **CẤT** kèm bảo vệ đó.

Nên `unlock` đang nói *"không có ví"* trong khi sự thật là *"có ví, nhưng nó
không được bảo vệ như ta đòi"* — nói sai với người dùng về đúng thứ họ cần
biết, và theo hướng làm họ yên tâm.

Đã thêm `KeystoreError::UnprotectedKey`: hỏi lại bằng đường không điều kiện
trước khi kết luận "không có", và **từ chối dùng** khoá bảo vệ yếu hơn mức đã
hứa. Phép thử `khoa_khong_duoc_bao_ve_thi_noi_dung_chuyen_ay` cất một mục không
kèm bảo vệ rồi đòi đúng lỗi ấy.

> Ba lỗi hôm nay đều cùng một hình dạng: **mã đúng theo cách tôi đọc tài liệu,
> và sai theo cách hệ thống thật cư xử.** 49 byte thừa, entitlement, và bộ lọc
> này. Không phép thử nào tôi tự viết bắt được cả ba.


## 19b. GỠ ĐƯỢC — ví cất được khoá (22/08/2026)

Chạy trong gói đã ký, có hồ sơ cấp phép nhúng:

```
✓ CẤT ĐƯỢC vào Keychain với USER_PRESENCE
  contains = true
✓ xoá được — Keychain sạch lại
```

Đường đi, không cần ai bấm gì trên trang web của Apple:

1. Khoá **App Store Connect API** của đội `8E5HYH6F96` đã có sẵn trong dự án ERP
2. `com.tcc.browser` **đã tồn tại** trên tài khoản
3. Sinh **CSR tại chỗ** → tạo chứng thư `MAC_APP_DEVELOPMENT` qua API → khoá
   riêng nằm trên máy này ngay từ đầu, không phải xuất từ đâu cả
4. Đăng ký máy này làm thiết bị
5. Tạo hồ sơ `MAC_APP_DEVELOPMENT` buộc App ID + chứng thư + thiết bị

### Ba điều học được, cái thứ nhất là chẩn đoán lại §19

**Binary TRẦN không mang được hồ sơ cấp phép.** Hồ sơ phải nằm trong
`Contents/embedded.provisionprofile` của một **gói `.app`**. Ký một binary trần
kèm entitlements thì nó treo im lặng, `kill -9` không gỡ được — đúng chế độ hỏng
của §19, và tôi dựng lại được nó hôm nay bằng cách ký đúng binary ví dụ ấy. Bọc
vào `.app` cùng hồ sơ thì chạy ngay.

Nên §19 nói *"thiếu hồ sơ cấp phép"* là **đúng nhưng chưa đủ**: thiếu **một hồ
sơ mà bộ nạp TÌM RA ĐƯỢC**.

**Keychain Sharing KHÔNG phải quyền năng của App ID.** API trả về danh sách hợp
lệ và không có `KEYCHAIN_ACCESS_GROUPS` trong đó. Quyền `keychain-access-groups`
với tiền tố đội của chính mình được **bất kỳ hồ sơ nào** của App ID ấy cho phép.
Bớt hẳn một thao tác ghi mà kế hoạch tưởng là bắt buộc.

**`$(AppIdentifierPrefix)` là biến của XCODE.** `codesign` không khai triển nó —
nó nhúng đúng chuỗi ấy làm quyền, chuỗi ấy không khớp hồ sơ, quyền không được
cấp. `tools/dong-goi-macos.sh` giờ đọc tiền tố **từ chính hồ sơ**.

## 19. Ví bị chặn ở ĐÓNG GÓI, không phải ở mã (17/08/2026)

Ba lần thử để `USER_PRESENCE` chạy được:

| Thử | Kết quả |
|---|---|
| Không ký (`cargo run`) | hỏng NGAY: *"A required entitlement isn't present"* |
| Ký ad-hoc `-s -` kèm entitlements | qua cửa quyền, rồi **treo** |
| Ký bằng **chứng thư thật**, nhóm truy cập đúng mã đội `ZN2UMA7H7A` | vẫn **treo** |

### Cái bẫy nằm ở lần thứ ba

Tiến trình **không báo lỗi gì cả**. Nó nạp lên rồi treo trong bộ nạp: 8 KiB bộ
nhớ, trạng thái `UE`, `kill -9` không gỡ được. Không một dòng nào ra `stderr`,
**kể cả dòng in trước lệnh chạm Keychain đầu tiên** — nên nhìn từ ngoài thì
giống hệt "chương trình treo ở lệnh Keychain", mà thật ra nó chưa chạy dòng nào.

Tôi mất ba vòng đo mới thấy, vì `println!` bị đệm khi chuyển hướng ra tệp: bản
đầu tưởng là "treo ở `store`", đổi sang `eprintln!` mới thấy **không có gì in ra
cả**.

Nguyên nhân: quyền khai một nhóm truy cập Keychain mà **không hồ sơ cấp phép nào
cho phép**. macOS không từ chối bằng lỗi; nó treo tiến trình trong lúc thẩm định
chữ ký.

Và một chi tiết nhỏ tốn cả một vòng: `"Apple Development: … (V8KBARY5VT)"` —
chuỗi trong ngoặc **không phải mã đội**. Mã đội thật là `ZN2UMA7H7A`, chỉ thấy
được qua `codesign -dv`.

### Kết luận, và nó không phải việc của tôi

Cần một **hồ sơ cấp phép macOS** gắn với App ID `com.tcc.browser` có bật Keychain
Sharing, nhúng vào `Contents/embedded.provisionprofile`. Tạo nó là việc trên tài
khoản Apple Developer của tổ chức.

`tools/dong-goi-macos.sh` ráp gói xong và **dừng đúng trước bước ký**, in ra ba
việc cần làm. Nó cố ý không tự ký kèm entitlements khi thiếu hồ sơ — làm thế là
tạo ra một gói treo im lặng.

### Vì sao không hạ tiêu chuẩn cho xong

Cất khoá không kèm `USER_PRESENCE` thì mọi thứ chạy được ngay hôm nay — và
`unlock` của ta **từ chối** đúng trường hợp ấy (`UnprotectedKey`, §18). Giữ
nguyên sự từ chối đó:

> Ví chạy mà bảo vệ kém hơn mức nó hứa thì người dùng không thấy gì cả, cho tới
> lúc mất tiền. Ví **không chạy** thì họ thấy ngay.

Nên trạng thái đúng của dự án lúc này là: **phần ví xong về mã, bị chặn ở đóng
gói.** Đó là một câu khác hẳn "sắp xong".


## 20. Bộ dựng thứ hai (17/08/2026)

`crates/tcc-render-raster` — vẽ thẳng ra pixel, không một dòng HTML.

### Nó KHÔNG phải để đẹp hơn

`tcc-ui` có luật *"không được biết bộ dựng nào"*, và cho tới hôm nay luật ấy
**không ai kiểm được**: một trait chỉ có một bản cài đặt thì mọi giả định ngầm
đều nằm im, vì chưa có ai đá vào. Crate này là cú đá.

### Hai thứ nó cho không, mà WebView không cho

**Chạy được không cần màn hình.** `cargo test --workspace` không chạm WebView
được — trên macOS vòng lặp sự kiện phải ở luồng chính. Từ nay mọi màn hình của
khung được **kiểm trợ năng ở CI, trên cả ba nền, mỗi lần đẩy** — trước đó chỉ
kiểm được trong ví dụ có người bấm.

**Ảnh so được từng pixel**, nên một thay đổi bố cục ngoài ý muốn hiện ra thành
con số.

### Cây trợ năng dựng TỪ LƯỢT VẼ

Gọi `Node::accessibility_tree()` trong `published_accessibility` thì phép kiểm
ngang bằng **luôn xanh** — nó đang so một hàm với chính nó. Nên ở đây cây được
ghi lại **trong lúc vẽ**: vẽ sót một nút là đỏ.

Bản đầu bọc thêm một `Group` ở gốc và phép kiểm bắt ngay — đúng như nó phải thế.

### Phép kiểm đắt nhất: hai bộ dựng phải nói CÙNG một điều

`crates/tcc-shell/tests/hai-bo-dung.rs` chạy mỗi màn hình qua cả hai và đòi hai
cây trợ năng **bằng nhau**. Nếu hai bộ dựng nói hai điều khác nhau với trình đọc
màn hình thì ít nhất một cái đang nói dối, và cho tới hôm nay không có cách nào
biết.

### Và kiểm đột biến tìm ra một lỗ trong chính phép kiểm ấy

Bỏ cờ *"không hoàn tác"* của nút nguy hiểm → **ba phép thử màn hình thật vẫn
xanh**. Vì màn xác nhận giao dịch cố ý dùng hai nút cùng sắc thái, và màn quản
lý quyền rỗng thì không có nút nào.

> Một phép kiểm chéo chỉ chạy trên những màn hình đang có là một phép kiểm chéo
> che đúng chỗ chưa ai đi qua.

Đã thêm `phu_het_moi_loai_nut` — một cây phủ hết tám loại nút, kể cả loại màn
hình thật chưa dùng. Sau đó cả hai đột biến (cờ nguy hiểm, ảnh trang trí kèm
nhãn) đều đỏ.

### Ranh giới đã nói ra

Bộ dựng này xếp `Flow::Row` theo chiều dọc. Nói thẳng trong mã chứ không giả vờ:
bố cục hàng cần đo bề rộng từng phần tử, và crate này sinh ra để **kiểm** bố cục
dọc + trợ năng, không phải để thay WebView hôm nay.


## 21. Bố cục thật cho bộ dựng thứ hai — 4.2 (17/08/2026)

Bản 4.1 xếp mọi thứ thành một cột và ghi chú *"hàng ngang chưa làm"*. Giờ nó
làm rồi, và cách làm mới là chỗ đáng nói: **đo → đặt → vẽ**, ba lượt tách bạch.

Bản cũ gộp đo vào vẽ — và đó chính là lý do nó không xếp hàng ngang được: không
có kích thước thì không có gì để đặt cạnh nhau.

### Ba điều mới, ba đột biến, ba lần đỏ

| Điều | Đột biến để kiểm | Kết quả |
|---|---|---|
| Hàng ngang đặt cạnh nhau | đổi `dat_hang` → `dat_cot` | ✅ đỏ |
| Hàng hết chỗ thì xuống dòng | bỏ phép so bề rộng | ✅ đỏ |
| Chữ dài thì ngắt dòng | đặt bề rộng = 100 000 px | ✅ đỏ |

**Xuống dòng chứ không tràn ra ngoài.** Một nút bị đẩy khỏi mép là một nút người
dùng *không bấm được và không biết là có* — nên hai phép thử đều đòi thêm: không
một pixel nào chạm cột cuối cùng của ảnh.

### Một phép thử suýt vô nghĩa

Bản đầu của `chu_dai_thi_ngat_dong` viết `cao_dai > cao_ngan * 2`. Sai: lề trên
dưới cộng vào **cả hai** vế và làm phép nhân mất nghĩa — câu dài ngắt đúng hai
dòng mà phép thử vẫn đỏ. Sửa thành `cao_dai >= cao_ngan + 2 dòng`, và viết câu
thử dài hẳn ra ba dòng, vì hai dòng thì phép thử vẫn xanh **kể cả khi ngắt dòng
chỉ chạy đúng một nửa**.

### Bốn tham số hình học gom thành một kiểu

`dat_cot`/`dat_hang` từng nhận `(trai, tren, rong, khe)` rời. Bốn `f32` cạnh
nhau, đổi chỗ hai cái vẫn biên dịch, vẫn chạy, chỉ **vẽ sai** — và tôi suýt vấp
đúng thế lúc tách hàm. Giờ chúng là `struct Cho`, và trình biên dịch giữ hộ.

### Còn thiếu gì của 4.3

Căn lề, co giãn theo chỗ trống, hợp thành (chồng lớp, cắt xén). Hàng/cột và
xuống dòng chỉ là phần xương.


## 22. Cầu nối AccessKit — 4.4 (17/08/2026)

`crates/tcc-render-raster/src/accesskit_bridge.rs`, sau cờ `accesskit`.

### Chỗ dễ mất thông tin nhất

| Ta có | AccessKit có |
|---|---|
| `TextInput { secret: true }` | `Role::PasswordInput` ✅ |
| `Switch { on }` | `Role::Switch` + `toggled` ✅ |
| `Button { destructive: true }` | **chỉ `Role::Button`** ❌ |

Hai cái đầu dịch thẳng. Cái thứ ba **không có vai trò tương ứng**, và bỏ qua thì
người dùng trình đọc màn hình nghe *"Xoá dữ liệu, nút"* **y hệt** *"Huỷ, nút"* —
cùng một câu cho một việc xoá sạch và một việc không làm gì. Nên nó đi vào
`description`.

`PasswordInput` cũng không phải chuyện hình thức: `TextInput` làm trình đọc màn
hình **đọc to từng ký tự** mật khẩu. Đây đúng là bất biến B32, giờ áp cho nền
tảng thứ hai.

### Hai lỗi của tôi, và cả hai đều bị máy bắt

**Viết cứng câu tiếng Việt.** Bản đầu tôi để `CAU_MAT_MAT` là hằng số tiếng Việt
ngay trong crate. Sai hai lần: giao diện mặc định **tiếng Anh**, và WebView đã
nhận câu ấy làm **tham số** do `text.rs` dịch. Hai bộ dựng đọc hai câu khác nhau
cho cùng một nút là đúng thứ phép kiểm chéo sinh ra để chặn. Giờ nó là
`AccessText`, tiêm từ ngoài, mặc định tiếng Anh — y như `RendererText`.

**Đặt tên module bằng tiếng Việt.** `accesskit_cau` → **luật 13 bắt được**. Đó
là luật tôi tự viết vì quy ước này từng trôi suốt nhiều tháng khi chưa có máy
canh; hôm nay nó canh chính tôi.

### Đột biến

| Đổi gì | Bắt được? |
|---|---|
| ô mật khẩu → `TextInput` thường | ✅ |
| nút nguy hiểm không nói gì thêm | ✅ (2 phép thử + phép kiểm chéo) |
| nhóm mang nhãn | ✅ |

### Còn thiếu

Ánh xạ xong, nhưng **chưa nối adapter của nền tảng vào cửa sổ thật** — việc ấy
cần `accesskit_macos`/`accesskit_windows` và một cửa sổ, tức là cùng loại việc
đang bị chặn ở §19. Phần ánh xạ là phần có thể kiểm mà không cần cửa sổ, và nó
là phần chứa mọi quyết định dễ sai.


## 23. 4.3 — và một câu hỏi kế hoạch chưa hỏi (17/08/2026)

### Làm được ngay, và đã làm

**Căn giữa theo chiều dọc trong hàng.** Bản 4.2 đặt phần tử ngay lúc đo, nên mọi
thứ dính mép trên: một nhãn nhỏ cạnh một tiêu đề lớn trông như bị **treo lơ
lửng** — thứ người ta nhìn thấy ngay kể cả khi không biết gọi tên nó là gì.

Sửa được là nhờ đổi cách gom: gom hết một dòng rồi mới đặt, vì **chiều cao dòng
chỉ biết sau khi đã đo hết phần tử trên dòng ấy**. Đột biến bỏ căn giữa → đỏ.

### Và một lỗi tôi ĐI TÌM mà không có

Màn xác nhận giao dịch cố ý hiện địa chỉ **đủ 66 ký tự** — cắt ngắn là lỗ dò
trùng đầu-đuôi. Nhưng 66 ký tự không có dấu cách thì bộ ngắt dòng **theo từ**
không có chỗ nào để ngắt, và tôi chờ thấy nó tràn khỏi mép.

Không tràn: `cosmic-text` tự ngắt giữa từ khi một từ dài hơn cả dòng. Hai phép
thử vẫn giữ lại — địa chỉ trong nhóm lồng bốn tầng, và một từ 400 ký tự — vì
ngày đổi bộ chữ thì đây là chỗ vỡ đầu tiên.

### Câu hỏi kế hoạch chưa hỏi: "căn lề" có thuộc TIÊU CHUẨN không?

4.3 ghi *"bố cục, hợp thành"*, và cách hiểu tự nhiên là thêm thuộc tính `align`
vào `Group`. **Tôi cố ý không làm.**

Quyết định kiến trúc số 1 nói ứng dụng khai báo **ý định**, bộ dựng quyết định
**hình thức**. Mô hình nút hiện có `Flow` và `Gap` — không cỡ, không màu, không
căn lề — và điều đó không phải thiếu sót, đó là ranh giới.

`align: "center"` là **hình thức**. Thêm nó là:

- một thay đổi phá vỡ tiêu chuẩn (VERSIONING §2), kéo theo bản dịch, vector, cả
  hai bộ dựng;
- và một bước lùi khỏi luật *"ứng dụng không mô tả hình thức"*, sau đó `padding`,
  `width`, `color` sẽ tới, mỗi cái đều hợp lý một mình.

Nên **căn giữa là quyết định của bộ dựng**, không phải thuộc tính của nút. Kế
hoạch viết 4.3 trước khi mô hình khai báo ổn định, và chỗ này kế hoạch nên đổi
chứ không phải mô hình.

Ai thấy cần `align` thật thì phải trả lời trước: *ứng dụng nào cần nó, và vì sao
bộ dựng không tự quyết được?* Chưa có ứng dụng nào để hỏi câu ấy.


## 24. Bề rộng cũng là một cái hích (17/08/2026)

Phần co giãn của 4.3 hoá ra không phải chuyện bố cục, mà là chuyện **chống đẩy
người dùng**.

`SECURITY.md` và `transaction_screen` đã chốt: hai nút ở màn xác nhận giao dịch
mang **cùng sắc thái**, vì làm nút *"Ký"* nổi hơn là đẩy người dùng về một phía
đúng lúc nguy hiểm nhất. Có phép thử canh điều đó.

Nhưng **bề rộng cũng đẩy**, và không ai canh: *"Ký giao dịch này"* rộng gấp ba
*"Huỷ"*. Cùng màu, cùng viền, mà mắt vẫn bị kéo về nút to. Một cái hích bằng
hình học thay vì bằng màu.

### Sửa ở CẢ HAI bộ dựng, bằng hai đường khác nhau

| Bộ dựng | Cách |
|---|---|
| Pixel | Hàng toàn nút → kéo mọi nút về bề rộng lớn nhất, ngay trong bố cục |
| WebView | Đánh dấu `data-hang-nut` → CSS `flex:1 1 0` |

**Chỉ áp cho hàng TOÀN nút.** Một nút cạnh một nhãn thì kéo giãn ra là vô nghĩa,
và CSS không hỏi được *"hàng này có toàn nút không"* — nên bộ dựng đánh dấu, chứ
không viết CSS đoán mò.

Bốn đột biến, bốn lần đỏ: bỏ kéo bằng ở bộ dựng pixel, bỏ đánh dấu ở WebView, bỏ
luật CSS, và (đã kiểm trước đó) bỏ căn giữa dọc.

### Vì sao phải kiểm CẢ HAI

Bộ dựng người dùng thật nhìn thấy là **WebView**. Chữa mỗi bộ dựng pixel thì
phép thử xanh mà cái hích vẫn còn nguyên trên màn hình thật — đúng loại "xanh
nhưng không đúng" mà phép kiểm chéo sinh ra để chặn.


## 25. Đọc được chữ người dùng gõ — mắt xích cuối của màn nhập ví (17/08/2026)

Câu hỏi *"ví có tích hợp vào trình duyệt để cấu hình sau không"* vướng đúng một
chỗ: **vẽ ra ô PIN thì dễ, nhận lại thứ người ta gõ mới là đường dữ liệu mới**.
Trước hôm nay nó cố ý chưa mở.

### Mở, nhưng CHỈ cho màn hình của khung

Hai kịch bản nối sự kiện, không phải một kịch bản với một cờ:

| Kịch bản | Dùng ở đâu | Đọc ô nhập |
|---|---|---|
| `KICH_BAN_KHUNG` | hộp thoại quyền, màn nhập ví | **có** |
| `KICH_BAN_NOI_SU_KIEN` | màn hình ứng dụng | **không** |

Ứng dụng TCC **không mang mã**, nên không có ai bên trong nhận giá trị ấy cả.
Thu thập thứ không ai cần là mở rộng bề mặt mà không đổi lấy gì.

Tách hằng số chứ không đặt cờ `bool`: một cờ đặt sai vẫn biên dịch, còn gọi
nhầm hằng số thì đọc mã là thấy. Phép thử
`kich_ban_ung_dung_khong_doc_o_nhap` chốt rằng kịch bản của ứng dụng không chứa
chữ `.value`.

### `Debug` giấu giá trị, chỉ để lộ nhãn

`DialogAnswer` giờ mang mã PIN. Một `{:?}` trên nó rất dễ rơi vào nhật ký lỗi,
nên `Debug` được viết tay:

```
DialogAnswer { hanh_dong: "cho-phep", bat: [], o_nhap: ["Mã PIN: <9 ký tự>"] }
```

Giữ nhãn và độ dài — đủ để gỡ lỗi, không đủ để lộ.

### Có phép kiểm đi hết đường, qua WebKit thật

`kiem-bam-nut o-nhap`: đặt giá trị vào ô rồi **phát sự kiện `input` như bộ gõ
thật**, sau đó mới bấm. Đặt giá trị mà không phát sự kiện là bỏ qua đúng đoạn
mà trang thật chạy.

Chạy trong CI trên cả ba nền. Một đường dữ liệu mới mà không có phép kiểm đi
hết thì hỏng lúc nào không ai biết — ô nhập vẫn vẽ ra đẹp, chỉ là không ai nhận
thứ gõ vào.

`simulate_click` và `simulate_typing` dùng CHUNG một hàm chạy. Viết hai bản là
để chúng trôi dạt khỏi nhau, và lúc đó một bên xanh trong khi đường thật đã hỏng.

### Còn lại gì để ví vào được trình duyệt

1. Một cửa vào trong `tcc-browser` (lệnh con hoặc mục cài đặt)
2. Bật cờ `os-keystore` cho binary
3. **Hồ sơ cấp phép Apple** — §19, không phải việc của mã


## 26. Ví vào trong trình duyệt — và ba lỗi người dùng tìm ra (17/08/2026)

```bash
tcc-browser vi cum-tu                  # gõ 24 chữ / hạt giống
tcc-browser vi nhap <tệp-ví-web.json>  # nhập từ ví web, hỏi PIN
```

### Hai đường nhập, và người dùng hỏi đúng câu

*"Nhập PIN rồi thì sao, tôi tưởng phải nhập seed chứ?"* — câu ấy chỉ ra rằng tôi
mới làm giao diện cho **một** trong hai đường. `vi nhap` hỏi PIN vì hạt giống đã
nằm trong tệp; đường người ta hình dung khi nghe "nhập ví" là **gõ thẳng cụm
từ**, và nó chưa có. Giờ có.

### `tao` chỉ cho MỘT vòng lặp sự kiện mỗi tiến trình

Ba màn hình = ba `ask_dialog` = hoảng loạn ở `app_state.rs`, và nhìn từ ngoài
chỉ thấy **treo**. Đường hộp thoại hỏi quyền không lộ ra điều này vì nó chỉ mở
một cửa sổ mỗi lần chạy — lỗi nằm đó chờ tới đúng tính năng đầu tiên cần hai màn.

`dialog_sequence`: một cửa sổ, một `WebView`, mỗi màn là một lần `load_html`.
**Danh sách trắng đổi theo màn** — mã của màn trước không dùng lại được ở màn sau.

### Ba lỗi, không cái nào phép thử của tôi bắt được

| Người dùng thấy | Thật ra |
|---|---|
| *"trình duyệt không hiện gì"* | `main` chưa bao giờ gọi `run_app` |
| *"nhập sai thì tắt luôn"* | lỗi chỉ in ra **terminal**, cửa sổ đóng im lặng |
| *"bấm huỷ mất luôn 24 chữ"* | màn xác nhận dùng chung mã với nút huỷ |

Cả ba nằm ở **chỗ người dùng chạm vào**, không ở chỗ mã tính toán — nên không
phép thử đơn vị nào chạm tới. Hai thứ tôi bù vào:

**Kéo quyết định ra thành hàm thuần.** `phrase_step` kiểm được không cần cửa sổ.
Trước đó toàn bộ quyết định nằm trong một bao đóng chạy giữa vòng lặp sự kiện,
và một nhánh sai ở đó **trông y hệt "người dùng bấm huỷ"**.

**Ví dụ tự lái qua WebKit thật.** `kiem-cum-tu-sai` tự gõ sai, tự bấm, rồi hỏi:
màn lỗi có hiện lại không, hay cửa sổ đã đóng. Chạy trong CI.

### "Quay lại sửa" tách khỏi "Huỷ"

Màn xác nhận địa chỉ **sinh ra để bắt lỗi gõ**. Nút thứ hai của nó dùng chung mã
với "huỷ" thì thấy địa chỉ sai bấm là mất cả 24 chữ — và lần sau người ta đi dán
từ chỗ khác, mà **chỗ khác thường là một ô nhập trên web**. Hai mã riêng, để
danh sách trắng từng màn nói đúng thứ màn ấy cho phép.

### Câu lỗi của hệ điều hành được DỊCH

`A required entitlement isn't present` là câu nói với lập trình viên. Người dùng
đọc nó tưởng mình gõ sai cụm từ. Giờ màn hỏng nói: *"Bản dựng này chưa được ký…
Không phải bạn gõ sai"* — và giữ câu gốc ở dưới dạng chữ mờ, vì người soát cần nó.

Nhận dạng bằng chuỗi `entitlement` vì thư viện không cho mã riêng. Chuỗi đổi thì
phép nhận dạng lặng lẽ hỏng — nên câu gốc **vẫn phải hiện**, và có phép thử ghim
đúng chuỗi đang nhận.


## 27. Giao dịch THẬT đầu tiên qua đường chống ký mù (17/08/2026)

```
tx_hash 0xc06d6191c039ece24cc87ff8d4b4dae82257f657bbaf32c48e473c5c38017ade
nonce   174 → 175        ← bằng chứng nó đã vào khối
```

Testnet 91338, ví `0x266346…9a71`, 1 TCC. Rồi một giao dịch thứ hai
(`0x935cb477…`), nonce 175 → 176. Cả hai chốt sau khoảng 20 giây.

Đường đi trọn vẹn: gõ cụm từ → máy chủ dựng giao dịch chưa ký → **giải mã, tự
tính lại băm, so khớp** → hiện thứ ĐÃ GIẢI MÃ → người dùng bấm → ký → gửi.

Đây là câu trả lời cho *"lập trình viên được gì mà web thường không cho"*, lần
đầu ở dạng **một giao dịch có thật** chứ không phải một lập luận.

### Thử được mà KHÔNG hạ tiêu chuẩn nào

Cất khoá vẫn bị chặn ở đóng gói (§19). Nhưng **ký** thì không — nên
`examples/thu-vi-mot-phien.rs` giữ khoá **trong bộ nhớ đúng một phiên** và
không ghi ở đâu cả.

Không phải bản lùi: không có gì được cất bằng cách yếu hơn — **không có gì được
cất cả**. Màn hình nói thẳng điều đó **TRƯỚC khi người dùng gõ**, và có phép thử
chốt rằng câu ấy đứng trước ô nhập trong tài liệu. Người gõ cụm từ khôi phục vào
một cửa sổ mặc định tin rằng nó được lưu; nói sau khi họ gõ xong là đã để họ tin
nhầm một lượt.

### Và lỗi thứ ba cùng một hình dạng

| Lần | Người dùng thấy | Thật ra |
|---|---|---|
| 1 | trình duyệt không hiện gì | `main` chưa gọi `run_app` |
| 2 | nhập sai thì tắt luôn | lỗi chỉ in ra terminal |
| 3 | **bấm ký xong thì ẩn luôn** | **thành công cũng chỉ in ra terminal** |

Lần thứ ba đau nhất: tôi vừa sửa đúng chuyện này cho nhánh HỎNG và **quên nhánh
THÀNH CÔNG**. Người dùng bấm ký rồi thấy cửa sổ biến mất, không biết tiền đã đi
hay chưa — trạng thái tệ nhất một ví có thể để lại.

Màn "Đã gửi" hiện **mã giao dịch đủ** và một câu cố ý:

> *"Mạng đã nhận. Nó CHƯA vào khối — tra mã ở trên để biết khi nào lên."*

*"Đã gửi"* rất dễ đọc thành *"xong rồi"*. Mạng mới nhận, chưa ghi vào khối —
không nói ra là **hứa hộ chuỗi**.

### Bài học chung của ba lần

Cả ba đều là **kết quả không đi ra tới nơi người dùng nhìn**. Không phép thử đơn
vị nào chạm tới, vì chúng đều nằm ở chỗ mã giao tiếp với người chứ không ở chỗ
mã tính toán. Luật rút ra:

> Mỗi nhánh kết thúc của một luồng — xong, hỏng, huỷ — **phải có một màn hình**.
> Đóng cửa sổ không phải một câu trả lời.


## 28. Hợp thành: **không vẽ đè** — 4.3 xong (17/08/2026)

Với dự án này, câu hỏi của "hợp thành" không phải *"chồng lớp thế nào"* mà là:

> **Ứng dụng có vẽ đè lên thứ khác được không?**

Đè được thì che được câu *"việc này chuyển tiền"* — người dùng xác nhận một thứ
họ không đọc được. Trên web đó là đòn cũ mèm: `position:absolute`, `z-index`,
lề âm.

### Hai bộ dựng, hai cách chốt

**Bộ dựng pixel** khai ra nó đặt cái gì ở đâu (`placed_boxes`), và phép thử tự
tính xem có cặp nào chồng nhau — **80 cây sinh tự động**, kể cả cây lồng bốn
tầng trộn hàng/cột/khe ngẫu nhiên. Luôn phải là 0.

**WebView** không đếm được, nên chốt ở nguồn: định kiểu của ta **không được
chứa** `position:absolute`, `position:fixed`, `z-index`, lề âm, `transform`.
Ứng dụng không gửi CSS — nhưng nếu chính định kiểu của ta mở cửa thì một cây
khéo sắp vẫn che được.

Kèm một phép thử nữa: không `overflow:hidden`, không `text-overflow`, không
`nowrap` — **cắt im lặng giấu đi phần giao diện người dùng đáng ra phải thấy,
và phần bị giấu có thể là nút "Huỷ"**.

### Bản đầu của phép thử này VÔ NGHĨA, và đột biến chỉ ra

Tôi cho bộ dựng **tự đếm** số ô chồng nhau, rồi phép thử đọc con số ấy. Đột
biến đặt bộ đếm về 0 → phép thử xanh ngay, kể cả khi bố cục chồng thật.

> **Phép thử đang hỏi chính bị cáo.**

Sửa: bộ dựng chỉ khai hình học thô; phép tính "có chồng không" chuyển hẳn sang
phép thử. Làm hỏng bố cục thì không còn chỗ nào để giấu. Chạy lại đột biến —
đỏ.

Đây là lần thứ ba trong dự án cùng một hình dạng lỗi, và lần này tôi dẫm dù đã
viết nó vào `CLAUDE.md`: *"một phép thử chưa từng thấy đỏ không phải bằng
chứng"*.


## 29. Tầng 3 — lối thoát ra trình duyệt hệ thống (17/08/2026)

Người dùng hỏi: *"đã nhập URL và truy cập được website ngoài đời chưa?"* Chưa,
và câu trả lời đầy đủ đáng viết ra.

Dựng một ô nhập URL rồi gọi `load_url` vào WKWebView thì **làm trong một buổi**
— nhưng nó sẽ là một thứ *trông như* trình duyệt mà không phải. Mọi thứ dự án
này xây — cổng quyền năng, chống ký mù, không vẽ đè, ứng dụng không mang mã —
**không áp dụng được cho một trang web bất kỳ**, vì trang web mang mã của nó và
WebView chạy mã ấy.

Nên làm **tầng 3** trước: `crates/tcc-shell/src/external_link.rs`.

### Ba luật, mỗi luật chặn một đòn

| Luật | Chặn cái gì |
|---|---|
| Chỉ `http`/`https` | `file://` đọc trộm đĩa, `javascript:` chạy mã, lược đồ lạ mở ứng dụng khác |
| Không qua vỏ lệnh | Chèn lệnh — địa chỉ ở đây có thể đến từ một gói ứng dụng |
| Hiện **ĐỦ** địa chỉ trước khi mở | Phần bị cắt là chỗ kẻ gian đặt tên miền thật của chúng |

Ký tự điều khiển bị chặn **trước** khi xét lược đồ: `https://a.example\n; rm -rf /`
có lược đồ hoàn toàn hợp lệ.

Câu lỗi **không kể tên lược đồ nào bị chặn** — nói rõ *"`file://` bị chặn"* là
dạy người thử biết cái gì đã được nghĩ tới và cái gì chưa.

### Màn hình nói thẳng, không xin lỗi

> *"Ở đó không còn thứ gì của TCC che chắn: không cổng quyền năng, không chữ ký,
> không hỏi quyền."*

Kế hoạch viết *"không giấu, không xin lỗi"*, và đây là chỗ thực hiện câu ấy.
Hai nút cùng sắc thái — không đẩy người dùng ra ngoài, cũng không giữ họ lại
bằng cách làm nút kia mờ đi.

### Đột biến

| Đổi gì | Bắt được? |
|---|---|
| nhận mọi lược đồ | ✅ 3 phép thử đỏ |
| bỏ kiểm ký tự điều khiển | ✅ 2 đỏ |
| bỏ kiểm địa chỉ trước khi mở | ✅ 1 đỏ |

### Còn tầng 2 thì sao

Vẫn 0 dòng, và nó là phần **lớn nhất còn lại của cả dự án**. Tầng 3 không thay
thế nó — nó chỉ khiến việc chưa có tầng 2 **không phải một bế tắc**.


## 30. Danh tính — 3.3, và chỗ nó DỪNG LẠI (18/08/2026)

Quyết định kiến trúc số 3 nói thẳng: *"Chữ ký chứng minh gói KHÔNG BỊ SỬA — nó
KHÔNG chứng minh AI ký."* Nên 3.3 không phải chỗ thêm "đã xác minh nhà phát
hành"; nó là chỗ làm cho **thứ danh tính duy nhất ta thật sự có** trở nên đúng.

Thứ ấy là **tính liên tục của khoá**: lần đầu thấy khoá nào, và khoá có đổi
không. Đã có sẵn (`SignerStatus`), nhưng cách hiện nó thì sai.

### Vân tay cũ chỉ nhìn hai đầu khoá

```
10 ký tự đầu … 10 ký tự cuối     ← của khoá THÔ, không phải một băm
```

Chi phí không sai — muốn khớp cả hai đầu phải mò 80 bit. **Phạm vi** mới sai:
nó không phủ khúc giữa. Hai khoá trùng hai đầu và khác ruột hiện ra **y hệt
nhau**, và kẻ dựng ra cặp ấy chỉ cần đụng vào phần không ai nhìn.

Giờ là một băm phủ **toàn bộ** khoá, hiện **đủ 64 ký tự**, chia nhóm 8 cho dễ
đọc:

```text
BLAKE3("tcc/v1/publisher-fingerprint" ‖ publisher_hex) → 32 byte
```

Bối cảnh tách miền là bắt buộc: cùng BLAKE3 ấy còn sinh **băm nội dung gói** và
**địa chỉ ví**. Ba mục đích qua một hàm băm là chỗ dễ lẫn nhất — và không có
bối cảnh thì vân tay chính là **tiền tố** của băm nội dung, vì 32 byte đầu của
XOF-48 trùng BLAKE3-256 chuẩn.

### Vào TIÊU CHUẨN, vì người dùng so vân tay giữa các bản cài đặt

Hai trình duyệt hiện hai chuỗi khác nhau cho cùng một người ký là **biến phép
kiểm duy nhất người dùng làm được thành tiếng ồn**. Nên phép dẫn nằm ở
`spec/0.1/05-interface.md`, không nằm trong mã của riêng ta.

### Hai phép thử của tôi đều LỎNG, và đột biến chỉ ra cả hai

**Phép thử tách miền vô nghĩa.** Nó so `assert_ne!` giữa vân tay (32 byte) và
băm nội dung (48 byte) — hai chuỗi hex khác độ dài thì **không bao giờ bằng
nhau**, có bối cảnh hay không. Gỡ bối cảnh đi vẫn xanh. Phép so đúng: đòi vân
tay **không phải tiền tố** của băm nội dung.

**Phép thử "phủ toàn bộ khoá" chỉ chặn đúng một cách cắt.** Hai khoá thử chỉ
trùng 10 ký tự mỗi đầu, nên một đột biến *"lấy 32 ký tự mỗi đầu"* vẫn thấy chúng
khác nhau và vẫn xanh. Sửa: cho trùng **64 ký tự mỗi đầu**, chặn cả họ những
cách cắt thay vì một cách.

### Chứng thực thì CHƯA, và nói rõ vì sao

Chứng thực cần một bên thứ ba bảo lãnh cho một khoá — tức là **sổ khoá**, thứ
0.1 cố ý không có. Làm nửa vời ở đây là đúng thứ quyết định số 3 cấm: hiện một
dấu tích mà đằng sau nó không có ai chịu trách nhiệm.

Nó thuộc 0.2, và trước đó phải trả lời: **ai vận hành sổ khoá, và người dùng
dựa vào cái gì để tin bên ấy?**


## 31. "Bảo vệ nội dung" là một cái tên nói quá (18/08/2026)

Kế hoạch gọi 3.4 là *"bảo vệ nội dung TCC — quyền sở hữu chứng minh trên
chuỗi"*. Làm xong phần chứng minh thì thấy tên ấy **hứa nhiều hơn thứ làm
được**, và đây là chỗ ghi lại.

### Nội dung gói nằm trên đĩa ở dạng ĐỌC ĐƯỢC

Gói TCC là một **thư mục đã ký**, không phải một hộp khoá. Ai mở thư mục ra là
thấy. Nên một phép kiểm sở hữu chỉ chặn được **màn hình**, không chặn được
**tệp**.

> Chặn hiển thị mà không mã hoá nội dung là một cái khoá treo trên cánh cửa
> không có tường.

### Vì sao không mã hoá luôn cho xong

Vì không có chỗ nào cất khoá giải mã cho đúng người:

| Cất ở đâu | Hỏng thế nào |
|---|---|
| Trong gói | Ai cũng đọc — không mã hoá gì cả |
| Trên chuỗi | Chuỗi công khai; cùng chuyện |
| Người bán mã hoá cho người mua | Người bán vẫn giữ bản rõ, bán tiếp cho ai cũng được |
| **Máy chủ phát khoá sau khi kiểm sở hữu** | **Chạy được** — nhưng đó là một DỊCH VỤ, không phải tính chất của chuỗi |

**Bảo vệ nội dung không có máy chủ phát khoá là không làm được.** Ai bảo làm
được thì đang bán một cánh cửa không tường.

Nên mục 3.4 đổi tên trong kế hoạch thành **kiểm sở hữu trên chuỗi**, và phần
"bảo vệ" ghi rõ là chưa — chứ không để một cái tên đẹp che một tính chất không
có.

### Phần LÀM ĐƯỢC, và nó vẫn đáng làm

`crates/tcc-chain/src/ownership.rs` đọc phản hồi `tcc_getNftsByOwner` và trả
lời *"ví này có mã ấy không"*. Đủ cho **"chỉ chủ sở hữu mới thấy nút Mở"** —
đúng như kế hoạch tự viết: chặn sao chép tuỳ tiện, **không** chặn người có kỹ
thuật và có động lực.

### Và nó lặp lại đúng bài học chống ký mù

Máy chủ trả về cả `owner` lẫn `contract` trong phản hồi, nên **so lại được** —
và module này so lại. Không so thì một RPC bị chiếm chỉ cần trả danh sách của
một ví giàu nào đó là **mọi người đều "sở hữu"**.

> Máy chủ là bên ta đề phòng, không phải bên ta dựa vào.

Đột biến bỏ phép so lại `owner` → đỏ.

### Một chi tiết nhỏ mà mất quyền

`0x01` và `0x1` là cùng một mã. Không chuẩn hoá thì một bên viết cách này, một
bên viết cách kia, và **chủ sở hữu thật bị báo là không sở hữu**. Đột biến bỏ
phép bỏ số 0 ở đầu → đỏ.


## 32. Nền tảng web: ĐO, không liệt kê — 5.1 (18/08/2026)

Mục 5.1 là *"công bố chính xác những gì hỗ trợ"*. Một danh sách viết tay là một
**lời hứa**: đúng vào ngày viết, trôi ngay hôm sau, và không ai biết nó đã trôi.

Nên nền tảng là thứ **đo được** — `examples/do-nen-tang.rs` nạp tài liệu vào bộ
máy THẬT, hỏi nó có gì, in ra bảng. Chạy trong CI trên cả ba nền, vì ba nền là
**ba bộ máy khác nhau** (WKWebView, WebKitGTK, WebView2) và nền tảng công bố
được là **phần giao**, không phải phần hợp. Công bố phần hợp là hứa một thứ mà
một phần ba người dùng không có.

### Phép đo đầu tiên đã đổi cách nghĩ

macOS: **18/20**. Hai mục vắng — và lý do mới là phần đáng giá:

| Vắng | Vì sao |
|---|---|
| `crypto.subtle` | cần **ngữ cảnh an toàn** |
| `localStorage` | cần **nguồn gốc** thật |

Cả hai **không phải do bộ máy thiếu**. Chúng vắng vì tài liệu nạp qua
`with_html` chạy trong **nguồn gốc mờ** — không `https://`, không tên miền.

> **Nền tảng phụ thuộc vào CÁCH NẠP nội dung, không chỉ vào bộ máy.**

Hệ quả cho hai tầng thì ngược nhau:

- **Tầng 1** — tin tốt. `localStorage` và `crypto.subtle` **không tồn tại** để
  mà phải chặn: một lớp phòng thủ có sẵn, không phải viết dòng nào.
- **Tầng 2** — rào chắn. Trang web thật cần nguồn gốc thật, nên tầng 2 **không
  dùng chung cách nạp với tầng 1**. Nó cần giao thức riêng và mô hình nguồn gốc
  riêng — đó là thiết kế, không phải cấu hình.

### Ba mục cuối bảng KHÔNG phải thứ ta muốn có

`localStorage`, `Notification`, `navigator.geolocation` nằm trong bảng để biết
**phải tắt cái gì**. Một tính năng có mặt mà ta quên tắt là một tính năng người
dùng bị lộ.

### Và nhóm tiếng Việt đứng đầu bảng, có chủ ý

`normalize`, `Intl.Collator('vi')`, `Intl.Segmenter`, `font-variation-settings`
— chúng quyết định chữ hiện ra đúng hay sai, và là thứ ít ai kiểm.
