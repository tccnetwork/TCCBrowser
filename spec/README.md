# Tiêu chuẩn TCC

Thư mục này là **sản phẩm thật** của dự án. Trình duyệt chỉ là bản cài đặt tham
chiếu của nó.

## Luật

1. **Rút ra từ mã đã chạy, không viết trước.** Tiêu chuẩn viết trước khi có mã
   phần lớn đều chết (XHTML 2.0, SOAP, họ WS-*). Tiêu chuẩn thành công thì được
   rút ra từ thứ đã chạy thật (HTML5, HTTP).
2. **Mỗi mục phải có ít nhất một phép kiểm** trong `tools/tcc-conformance`. Thêm
   điều vào đây mà không thêm phép kiểm là thêm một lời hứa không ai kiểm được.
3. **Phiên bản không bao giờ sửa tại chỗ.** `0.1/` đóng băng khi phát hành; thay
   đổi thì mở `0.2/`.

## Phép thử duy nhất chứng minh tiêu chuẩn viết đủ rõ

> Một người ngoài đọc `spec/0.1/` và tự làm được gói `.tccapp` hợp lệ mà **không
> cần hỏi ai**.

## Trạng thái

`0.1/` — **đã viết** (14/08/2026), rút ra từ bản cài đặt tham chiếu đang chạy.
Bản nháp làm việc, chưa đóng băng.

**Tiếng Anh là bản CHUẨN.** Tiếng Việt là bản dịch cho đội ngũ và cộng đồng TCC;
hai bản mâu thuẫn thì bản tiếng Anh thắng.

| | Bản chuẩn (en) | Bản dịch (vi) |
|---|---|---|
| Tổng quan | [README](0.1/README.md) | [README](0.1/vi/README.md) |
| Gói | [01](0.1/01-package.md) | [01](0.1/vi/01-goi.md) |
| Bản kê khai | [02](0.1/02-manifest.md) | [02](0.1/vi/02-ban-ke-khai.md) |
| Chữ ký | [03](0.1/03-signature.md) | [03](0.1/vi/03-chu-ky.md) |
| Quyền năng | [04](0.1/04-capabilities.md) | [04](0.1/vi/04-quyen-nang.md) |
| Giao diện | [05](0.1/05-interface.md) | [05](0.1/vi/05-giao-dien.md) |
| Mã lỗi | [06](0.1/06-error-codes.md) | [06](0.1/vi/06-ma-loi.md) |

Hai tài liệu áp cho **mọi phiên bản**, không nằm trong `0.1/`:

| | Bản chuẩn (en) | Bản dịch (vi) |
|---|---|---|
| Phiên bản & khai tử | [VERSIONING](VERSIONING.md) | [VERSIONING](vi/VERSIONING.md) |
| Quản trị | [GOVERNANCE](GOVERNANCE.md) | [GOVERNANCE](vi/GOVERNANCE.md) |

`GOVERNANCE.md` §1 nói thẳng thứ dễ nói tránh nhất: tiêu chuẩn này có **một tác
giả, một bản cài đặt, một bộ kiểm định — cùng một bên làm ra**. Nên "tuân thủ TCC
0.1" hôm nay chỉ có nghĩa là *đồng ý với một bản cài đặt*, không hơn.

**Hai luật CI giữ đặc tả khỏi trôi:**

| Luật | Kiểm gì | Vì sao |
|---|---|---|
| 10 | Mọi mã lỗi trong đặc tả **tồn tại trong mã** | Mã viết ra mà không có trong mã là lời hứa không ai giữ — người ngoài cài đặt theo nó sẽ không bao giờ khớp bộ kiểm định |
| 11 | Bản dịch **không trôi khỏi bản chuẩn** (số tệp, tập mã lỗi, và tài liệu chính sách phải có bản dịch) | Bản dịch lệch còn tệ hơn không có bản dịch: người đọc nó cài đặt theo một tiêu chuẩn khác mà không ai biết |
| 12 | Đặc tả **không có liên kết chết** | Người ngoài đọc đặc tả không có mã nguồn để đoán bù — một liên kết chết là một luật trỏ vào hư không |

## Còn thiếu để gọi là tiêu chuẩn quốc tế

- **Bản cài đặt thứ hai, độc lập.** Đây là thứ thiếu lớn nhất, và mọi thứ khác
  trong danh sách này đều nhỏ hơn nó.
- **Cổng ra Giai đoạn 2 chưa ai kiểm**: cần một người ngoài chưa đọc mã, đọc
  `spec/0.1/` rồi tự dựng một gói hợp lệ. Không tự kiểm được.
