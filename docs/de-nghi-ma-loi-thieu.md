# Đề nghị: ba mã lỗi bản này bắn ra mà đặc tả 0.1 không định nghĩa

> Trạng thái: **CHƯA QUYẾT**. Viết 25/08/2026. Theo
> [`spec/GOVERNANCE.md`](../spec/GOVERNANCE.md) §4 — sửa một đặc tả đã công bố
> không phải việc của một lượt dọn mã, nên đây là đề nghị, không phải bản vá.
>
> Luật 10b trong `tools/kiem-luat-phu-thuoc.sh` in tên ba mã này ra mỗi lượt
> chạy, kể cả khi ĐẠT. Danh sách miễn trừ im lặng chỉ đổi "chưa ai biết" lấy
> "không ai còn nhìn".

## 1. Vấn đề, dưới dạng quan sát được

`06-error-codes.md` nói bản cài đặt "PHẢI dùng đúng các mã này". Bản này bắn ra
ba mã KHÔNG có trong bảng ấy:

| Mã | Bắn ra ở | Nghĩa |
|---|---|---|
| `symlink` | `tcc-runtime/src/package.rs` | Gói chứa liên kết mềm |
| `package-too-large` | `tcc-runtime/src/package.rs` | Tổng kích thước gói vượt trần |
| `bad-scroll` | `tcc-ui/src/lib.rs` | Cây yêu cầu cuộn ở chỗ không cuộn được |

Hậu quả đo được: một bản cài đặt thứ hai đọc đặc tả rồi chối cùng những gói ấy
sẽ chối bằng **mã khác** — `bad-json`, hoặc một mã tự đặt. Chính mục "ba mã đã
gỡ" của `06-error-codes.md` viết rằng đó là điều mã ổn định sinh ra để ngăn.

Hai mã đầu thuộc **tầng gói**, tức là quyết định một gói có được nạp hay không.
`bad-scroll` mới có từ 24/08/2026, cùng lúc từ vựng bố cục được siết.

## 2. Ai vỡ

Không bản nào vỡ vì *hành vi*: cả ba trường hợp đã bị chối từ trước và vẫn bị
chối. Cái đổi là **chuỗi mã** trở thành bắt buộc. Một bản cài đặt đang chối gói
có liên kết mềm bằng mã khác sẽ thành không tuân thủ kể từ khi bảng có thêm dòng
`symlink` — nên đây không phải erratum thuần tuý, dù nó chỉ ghi lại hành vi vốn
đã có.

Nếu chọn hướng ngược lại — đổi mã trong bản này thành mã đã có trong bảng — thì
người vỡ là bất kỳ ai đang bắt `symlink`/`package-too-large`/`bad-scroll`. Theo
hiểu biết hiện tại, chưa có ai.

## 3. Kiểm bằng gì

Luật 2 của `spec/README.md`: không điều khoản nào vào tiêu chuẩn mà không có ít
nhất một vector. Ba dòng mới cần ba vector:

- `package.json`: gói chứa một mục liên kết mềm → `symlink`
- `package.json`: gói vượt trần tổng kích thước → `package-too-large`
- `ui.json`: cây đặt `scroll` ở chỗ không cuộn được → `bad-scroll`

Số vector đi từ 153 lên 156, và `kiem-so-lieu.sh` sẽ đòi mọi tài liệu ghi lại
cho đúng.

## 4. Đã cân nhắc gì khác

- **Không làm gì.** Bác. Mã lỗi không định nghĩa đúng là thứ mục "ba mã đã gỡ"
  nêu ra làm ví dụ về cái không được để tồn tại.
- **Đổi mã trong bản cài đặt cho khớp bảng hiện có.** Được với `bad-scroll`
  (`bad-layout` đã có trong bảng và cùng họ). Không được với `symlink` và
  `package-too-large`: không mã nào đang có mô tả đúng chúng, và nhét chúng vào
  `bad-json` là nói dối về nguyên nhân.
- **Coi là erratum, sửa tại chỗ trong 0.1.** Đây là hướng đề nghị cho hai mã
  tầng gói: bảng thiếu chúng là một chỗ SÓT, hành vi thì có từ đầu. Nhưng §2 ở
  trên nói rõ nó vẫn siết thêm ràng buộc, nên quyết định là của người bảo trì,
  không phải của tôi.

## Tìm ra thế nào

`cargo-mutants`, 25/08/2026. Đột biến trên `CryptoError::ma` sống sót vì
`bad-key` không nằm trong vector nào — và nó không nằm trong vector nào vì đặc
tả bảo nó không thể xảy ra. Kéo sợi chỉ ấy ra thì thấy `bad-key` **bắn được
thật** (32 byte `0x7f` không phải điểm trên đường cong; `ed25519-dalek` 3.0 kiểm
ngay tại `from_bytes` chứ không kiểm lười như đặc tả giả định). Sửa chỗ ấy xong,
câu hỏi "còn mã nào bản này bắn ra mà đặc tả không biết?" chưa ai từng hỏi —
luật 10 chỉ đi một chiều. Luật 10b hỏi chiều kia, và trả về ba mã này.
