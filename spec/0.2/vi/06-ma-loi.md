# 06 — Mã lỗi

> **Bản nháp.** 0.2 chưa phát hành. Chưa bản cài đặt nào đáp ứng nó. Xem
> [README.md](README.md).

## Tệp này là gì, và không phải gì

Mỗi thư mục phiên bản phải **tự đứng được**: người đọc để cài đặt 0.2 không phải
mở 0.1 ra mới biết trả về mã nào. Tệp này chưa đạt mức ấy — nó chỉ nói phần 0.2
**thêm vào**. Chép lại trọn bảng là một phần của việc hoàn thành 0.2, cùng với
`01`–`04`.

Từ giờ tới lúc ấy, đọc [bảng của 0.1](../../0.1/vi/06-ma-loi.md) trước. Mọi luật
ở đó giữ nguyên: mã để so khớp chứ không phải để đọc; lỗi bọc lỗi trả **nguyên
nhân gốc**; vỡ nhiều luật cùng lúc thì phép kiểm **đầu tiên** trong danh sách
thắng.

## Mã 0.2 thêm

`VERSIONING.md` §3 cho phép một phiên bản phụ thêm mã. Nó không cho phiên bản nào
bỏ hay đổi tên mã, và 0.2 không bỏ, không đổi tên mã nào.

| Mã | Khi nào |
|---|---|
| `bad-layout` | Một khai báo bố cục không thể có tác dụng: `size`/`min`/`max` đặt lên nút gốc; `fill` hay một phân số trên trục mà bề của cha suy từ nội dung; `min` lớn hơn `max` trên cùng một trục. [§3, §4.3 của 05](05-giao-dien.md) |
| `bad-scroll` | Một khai báo cuộn không thể có tác dụng hoặc không kiểm được: vùng cuộn có bề theo trục cuộn suy từ nội dung; vùng cuộn lồng trong một vùng cuộn khác cùng trục. [§9 của 05](05-giao-dien.md) |

Cả hai là **từ chối một khai báo**, không phải vẽ hỏng. Bản cài đặt nhận một bố
cục rồi không dựng nổi thì đó là lỗi của nó, không phải một mã lỗi. Chính chỗ
phân biệt ấy là lý do không mã nào trong hai mã này xuất hiện ở phần yêu cầu đối
với bộ dựng của 05.

## Chúng nằm ở đâu trong thứ tự

Danh sách có thứ tự của 0.1 dừng ở lượt kiểm nội dung. Bố cục được kiểm **sau**
khi đã biết gói còn nguyên và cây giao diện đọc được — một cây đã hỏng ở
`bad-json` thì không có bố cục nào để kiểm.

| # | Phép kiểm | Mã |
|---|---|---|
| … | *phép kiểm 1–10 của 0.1, giữ nguyên* | *xem [0.1](../../0.1/vi/06-ma-loi.md)* |
| 11 | Tính xác định và kẹp biên | `bad-layout` |
| 12 | Cuộn | `bad-scroll` |

Lỗi hình dạng vẫn là `bad-json`, y như 0.1: khoá lạ, sai kiểu, một từ ngoài vốn
từ đóng. `bad-layout` dành cho khai báo **đúng dạng mà bất khả**, không bao giờ
dành cho khai báo sai dạng.
