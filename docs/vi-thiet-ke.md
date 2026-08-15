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

## 7. Một câu hỏi tôi KHÔNG tự trả lời được

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

Chưa có ba câu trả lời đó thì phần chạm khoá thật chưa nên viết. Phần **không**
chạm khoá thật — trait, bản giả, phép thử, luật kiến trúc — viết được ngay, và
đó là thứ đang làm.
