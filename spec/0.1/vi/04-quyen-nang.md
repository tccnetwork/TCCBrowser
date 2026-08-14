# 04 — Quyền năng

## Nguyên tắc

**Quyền năng không tồn tại cho tới khi được cấp.** Ứng dụng không có quyền mặc
định nào. Không có "quyền mặc định vô hại".

Bản cài đặt **NÊN** làm điều này đúng bằng **kiểu dữ liệu**, không bằng kỷ luật:
nếu không có quyền thì không có giá trị nào để cầm, và mã đọc mạng sẽ không biên
dịch nổi. "Quên một lần kiểm" khi đó là chuyện không xảy ra được.

## Khai trong bản kê khai

```json
{
  "name": "network",
  "scope": { "kind": "network", "hosts": ["shop.tcc-coin.com"] },
  "reason": "Tải danh sách sản phẩm"
}
```

| Trường | |
|---|---|
| `name` | PHẢI là một trong `network`, `storage`, `wallet`. Khác → `unknown-capability` |
| `scope` | Phạm vi, `kind` PHẢI khớp `name` |
| `reason` | Lý do bằng tiếng người, PHẢI không rỗng, chịu phép kiểm chuỗi ở [02](02-ban-ke-khai.md) |

**Mỗi quyền chỉ được khai MỘT lần.** Khai trùng → `duplicate-capability`.

⚠️ Đây không phải chuyện gọn gàng. Ứng dụng khai `network: [lành.com]` cho người
duyệt gói đọc, rồi khai thêm `network: [xấu.com]` ở dưới — bên cấp quyền lấy mục
sau đè mục trước, và cái được cấp là cái thứ hai. Bản cài đặt **PHẢI** chặn ở
**cả tầng kê khai lẫn tầng cấp quyền**; đừng để một tầng tin vào tầng kia.

## Ba loại phạm vi

### `network`

```json
{ "kind": "network", "hosts": ["shop.tcc-coin.com", "cdn.tcc-coin.com"] }
```

`hosts` **PHẢI** không rỗng — quyền mạng phải nêu đích danh máy chủ.

`hosts` **KHÔNG ĐƯỢC** chứa `*`. Ký tự đại diện biến một phạm vi hữu hạn thành
vô hạn, và người dùng không có cách nào đánh giá nó.

Mỗi tên máy chủ **PHẢI** là một tên miền hợp lệ:

- ASCII (dùng punycode cho tên miền quốc tế) — `non-ascii-host`
- 1–253 ký tự, mỗi đoạn 1–63 ký tự
- Chỉ **chữ cái, chữ số, `-`**; đoạn không bắt đầu/kết thúc bằng `-`
- CÓ THỂ có đúng một `.` ở cuối (dạng tuyệt đối)

⚠️ **Chỉ kiểm "là ASCII" là KHÔNG ĐỦ.** Chuỗi
`shop.tcc-coin.com:8080@evil.example` là ASCII, không rỗng, không có `*` — qua
hết. Nhưng dựng thành địa chỉ thì phần trước `@` là **userinfo**, còn máy chủ
thật là `evil.example`. Hộp thoại hỏi quyền hiện nguyên chuỗi đó và người đọc
lướt thấy "shop.tcc-coin.com".

**Luật rộng hơn:** chuỗi nào sắp đi vào một cú pháp khác (URL, đường dẫn, câu
lệnh) thì phải kiểm theo **cú pháp đích**, không phải theo "có ký tự lạ không".

### `storage`

```json
{ "kind": "storage", "quota_bytes": 1048576 }
```

### `wallet`

```json
{ "kind": "wallet", "may_request_signature": true }
```

`may_request_signature` là quyền **duy nhất chuyển được tiền của người dùng**.
Bản cài đặt **PHẢI** làm nó nổi bật hơn hẳn mọi quyền khác, và **PHẢI** nói rõ
bằng tiếng người rằng nó chuyển tiền — không phải một chữ "ví" chung chung.

## Luật khớp tên máy chủ

Khi ứng dụng gọi một máy chủ, bản cài đặt **PHẢI** khớp **CHÍNH XÁC**.

| Đã cấp | Gọi tới | |
|---|---|---|
| `shop.tcc-coin.com` | `shop.tcc-coin.com` | ✅ |
| `shop.tcc-coin.com` | `SHOP.TCC-COIN.COM` | ✅ tên miền không phân biệt hoa thường |
| `shop.tcc-coin.com` | `shop.tcc-coin.com.` | ✅ dấu chấm cuối là cùng máy chủ |
| `tcc-coin.com` | `shop.tcc-coin.com` | ❌ **tên miền con KHÔNG khớp** |
| `shop.tcc-coin.com` | `tcc-coin.com` | ❌ |
| `tcc-coin.com` | `evil-tcc-coin.com` | ❌ **khớp hậu tố là một lỗ** |
| `shop.tcc-coin.com` | `shop.tcc-coin.com.evil.example` | ❌ |

Chuẩn hoá trước khi so: bỏ dấu chấm cuối, đổi về chữ thường. Rồi so **bằng nhau**,
không phải "kết thúc bằng".

## Thu hồi

Thu hồi **PHẢI** có hiệu lực **tức thì**, kể cả với bản sao quyền năng mà ứng
dụng đang cầm trong tay. Bản cài đặt **KHÔNG ĐƯỢC** để một bản sao cũ tiếp tục
dùng được sau khi thu hồi.

## Hỏi người dùng

Bản cài đặt **PHẢI** hỏi **theo từng mục**, không hỏi một lần cho cả gói.

- Mặc định của mỗi mục **PHẢI** là **KHÔNG cấp**.
- Đồng ý mà không bật mục nào → **không quyền nào được cấp**.
- Bật một quyền **KHÔNG ĐƯỢC** kéo theo quyền khác.
- Đóng cửa sổ, hỏng hóc, hay bất kỳ đường nào không rõ ràng → **TỪ CHỐI**.

Hộp thoại **PHẢI** hiện: tên ứng dụng · **phạm vi cụ thể** (nêu đích danh máy
chủ, không nói "kết nối mạng") · **lý do nguyên văn của ứng dụng** · và câu cảnh
báo rằng chữ ký không chứng minh danh tính.

Hai nút quyết định **NÊN** ngang nhau về mặt thị giác. Làm nút đồng ý nổi hơn nút
từ chối là đẩy người dùng về một phía — và đẩy ở đúng chỗ nguy hiểm nhất.
