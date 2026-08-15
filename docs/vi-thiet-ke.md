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
