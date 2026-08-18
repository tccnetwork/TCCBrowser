# Tiêu chuẩn TCC — phiên bản 0.1

> **Trạng thái:** bản nháp làm việc. Rút ra từ bản cài đặt tham chiếu đang chạy,
> đúng luật 1 của [`spec/README.md`](../../README.md). Chưa đóng băng.
>
> ⚠️ **Đây là BẢN DỊCH.** Bản chuẩn là [bản tiếng Anh](../README.md); hai bản
> mâu thuẫn nhau thì bản tiếng Anh thắng. Luật 11 trong
> `tools/kiem-luat-phu-thuoc.sh` chặn hai bản trôi khỏi nhau.

## Tài liệu này dành cho ai

Người muốn **tự viết một bản cài đặt TCC** — dựng gói, ký gói, hoặc chạy gói —
bằng bất kỳ ngôn ngữ nào. Nếu đọc hết mà vẫn phải hỏi mới làm được gói hợp lệ
thì tài liệu này chưa đạt; đó là phép thử duy nhất của nó.

## Cái gì có tính quy phạm, và phiên bản này dựa vào đâu

Mọi thứ một bản cài đặt phải thoả để tuân thủ 0.1 đều nằm **trong thư mục này**,
kèm **PHẢI** của chính nó. Không gì ngoài thư mục này mang tính quy phạm.

Điều đó là cố ý, và hệ quả của nó đáng nói thẳng. Bốn chỗ trong các tài liệu ở
đây trỏ tới [`../VERSIONING.md`](../../VERSIONING.md) — để giải thích vì sao một
danh sách chỉ nở ra khi tăng phiên bản, và thế nào là thay đổi phá vỡ. Những
liên kết ấy mang tính **tham khảo**: chúng nói vì sao một luật lại như thế, hoặc
bảo người biên tập đặc tả phải làm gì tiếp. Không liên kết nào gắn một yêu cầu
vào phần văn bản được trỏ tới.

Lý do phải nói ra: `VERSIONING.md` nằm **ngoài** mọi thư mục có phiên bản, nên
khác thư mục này, nó không bất biến. Câu chữ của nó, và số mục của nó, đổi được
sau khi 0.1 đóng băng, mà không để lại vết đính chính nào. Nếu những liên kết ấy
mang tính quy phạm thì một yêu cầu của 0.1 có thể bị viết lại mà không cần tăng
phiên bản — đúng cái hỏng mà `VERSIONING.md` §1 sinh ra để chặn. Chúng không
mang tính quy phạm, nên chuyện đó không xảy ra được; người đọc lần theo một liên
kết rồi thấy câu chữ khác đi thì mất một lời giải thích, không mất một luật.

Một người cài đặt chưa từng mở những liên kết ấy vẫn dựng được gói hợp lệ. Ngày
nào điều đó không còn đúng, luật ấy phải chuyển **vào** thư mục này.

## Từ dùng

| Từ | Nghĩa |
|---|---|
| **PHẢI** | bắt buộc. Không làm là không tuân thủ. |
| **KHÔNG ĐƯỢC** | cấm. Làm là không tuân thủ. |
| **NÊN** | khuyến nghị mạnh; đi khác phải có lý do viết ra được. |
| **CÓ THỂ** | tuỳ chọn. |

## Đọc theo thứ tự này

| | |
|---|---|
| [01 — Gói](01-goi.md) | Bố cục trên đĩa, dạng chuẩn tắc, băm nội dung |
| [02 — Bản kê khai](02-ban-ke-khai.md) | Mọi trường và ràng buộc |
| [03 — Chữ ký](03-chu-ky.md) | Chữ ký lai, bố cục byte, **giao diện FIPS 204** |
| [04 — Quyền năng](04-quyen-nang.md) | Phạm vi và luật khớp |
| [05 — Giao diện](05-giao-dien.md) | Cây component khai báo |
| [06 — Mã lỗi](06-ma-loi.md) | Mã ổn định để so khớp |

## Ba câu quyết định cả kiến trúc

Đọc ba câu này trước, vì mọi thứ còn lại là hệ quả của chúng.

**1. Ứng dụng KHÔNG gửi kèm mã.** Điểm vào là một **cây component khai báo**, không
phải thẻ đánh dấu, không phải kịch bản. Ứng dụng nói *có gì trên màn hình*; bản
cài đặt quyết định *vẽ ra sao*. Xem [05](05-giao-dien.md).

**2. Quyền năng không tồn tại cho tới khi được cấp.** Ứng dụng không có quyền mặc
định nào. Mọi thứ chạm ra ngoài đều phải xin, và người dùng trả lời **từng mục**.
Xem [04](04-quyen-nang.md).

**3. Chữ ký chứng minh gói KHÔNG BỊ SỬA — nó KHÔNG chứng minh người ký là ai.**
Khoá công khai nằm ngay trong bản kê khai; gói **tự ký**. Bản cài đặt **KHÔNG
ĐƯỢC** hiện chữ "nhà phát hành đã xác minh". Phiên bản 0.1 chưa có sổ đăng ký khoá.

## Tuân thủ

Bản cài đặt tuân thủ **PHẢI** đạt 100% bộ vector ở `conformance/vectors/`. Thư
mục ấy nằm ngoài thư mục này, nên có đúng một ràng buộc giữ cho nó không xê dịch
được ý nghĩa của 0.1: **một vector chỉ được kiểm một yêu cầu đã nêu sẵn trong
thư mục này.** Thêm một vector như thế là đính chính, không cần tăng phiên bản,
vì bản cài đặt trượt nó thì vốn đã không tuân thủ — văn bản đã nói điều đó trước
khi có vector. Một vector kiểm thứ gì khác là một thay đổi của tiêu chuẩn và
phải tăng phiên bản, bất kể nó cho kết quả gì trên bản cài đặt gốc.

Các nhóm vector:

| Nhóm | Kiểm gì |
|---|---|
| `canonical` | Dạng chuẩn tắc + băm nội dung |
| `signature` | Chữ ký lai: sinh khoá · ký · kiểm |
| `acvp-mldsa65` | Nửa hậu lượng tử, mốc ngoài của NIST |
| `manifest` | Nhận/từ chối bản kê khai |
| `ui` | Nhận/từ chối cây giao diện |
| `capability` | Khớp phạm vi |

Vector là **dữ liệu JSON**, không phải mã — đọc được từ mọi ngôn ngữ. So khớp
bằng **mã lỗi ổn định** ([06](06-ma-loi.md)), không bằng thông báo lỗi.

## Điều phiên bản 0.1 CHƯA có

Ghi ra để không ai tưởng nhầm:

- **Sổ đăng ký khoá / danh tính.** Chữ ký chỉ chứng minh toàn vẹn.
- **Ngữ cảnh chữ ký** (`ctx` của FIPS 204) — luôn RỖNG, xem [03](03-chu-ky.md).
- **Ứng dụng chạy mã** (WASM). Chỉ có giao diện khai báo.
- **Nhiều màn hình / điều hướng.** Một gói, một điểm vào.
- **Cập nhật gói.** Không có cơ chế nâng cấp nào được định nghĩa.
- **Định dạng đóng gói.** Một gói là một *thư mục*; không có dạng nén nào và
  không có định dạng tệp `.tccapp`. Xem [01](01-goi.md).
