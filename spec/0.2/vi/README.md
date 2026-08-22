# Tiêu chuẩn TCC — phiên bản 0.2

> ## ⛔ ĐÂY LÀ BẢN NHÁP. CHƯA BẢN CÀI ĐẶT NÀO THOẢ. KHÔNG DÙNG ĐỂ XÉT TUÂN THỦ.
>
> Đây là **ngược hẳn** trạng thái của 0.1, và chỗ khác nhau ấy chính là lý do
> tồn tại của cái nhãn này.
>
> [`spec/0.1/README.md`](../../0.1/vi/README.md) nói 0.1 được *rút ra từ bản cài
> đặt tham chiếu đang chạy*. Thư mục này thì **viết trước**, trước mọi dòng mã,
> mà luật 1 của [`spec/README.md`](../../README.md) nói đó là thứ giết chết các
> tiêu chuẩn. Vẫn viết, vì lựa chọn còn lại — một người cài đặt tự đoán ra bố cục
> từ một từ vựng rỗng — còn tệ hơn. Nhưng nó được dán nhãn trung thực:
>
> - **Chưa bản cài đặt nào, kể cả của chính dự án này, thoả bất kỳ điều khoản nào
>   ở đây.**
> - **Không điều nào ở đây được viện trong một tuyên bố tuân thủ.** Câu "tuân thủ
>   TCC 0.2" hôm nay vô nghĩa và sẽ còn vô nghĩa cho tới khi thư mục này được
>   viết lại *TỪ* mã đang chạy.
> - **Không điều nào ở đây được dùng để từ chối một gói.** Hai mã lỗi nó đề xuất
>   không tồn tại ở đâu cả.
> - Vector của nó **không được nối vào bộ chạy** và không được
>   `tools/kiem-so-lieu.sh` đếm. Không gì đạt chúng; không gì cuộn chúng.
>
> ⚠️ **Đây là BẢN DỊCH.** Bản chuẩn là [bản tiếng Anh](../README.md); hai bản mâu
> thuẫn nhau thì bản tiếng Anh thắng.

## Ở đây có gì

| | Bản chuẩn (en) | Bản dịch (vi) |
|---|---|---|
| Giao diện — bố cục | [05](../05-interface.md) | [05](05-giao-dien.md) |

**Chỉ có chừng ấy, và một tệp không làm nên một phiên bản.**

[`spec/0.1/README.md`](../../0.1/vi/README.md) hứa rằng mọi thứ một bản cài đặt
phải thoả đều nằm trong thư mục phiên bản của chính nó, và không gì ngoài thư mục
ấy mang tính quy phạm. Vậy một bản 0.2 thật phải **chép lại cả sáu tài liệu**,
không được trỏ ngược về 0.1 — vì 0.1 chưa đóng băng, nên một yêu cầu của 0.2 tựa
vào câu chữ của 0.1 là một yêu cầu sửa được mà không cần tăng phiên bản, đúng cái
hỏng mà [`VERSIONING.md`](../../VERSIONING.md) §1 sinh ra để chặn.

Thư mục này chưa làm gì trong số đó. Mọi liên kết nó trỏ vào `0.1/` đều mang tính
**tham khảo**: nó ghi lại điều bản nháp này giả định là vẫn còn đúng. Trước khi
phát hành được, `01`–`04` và `06` phải được chuyển sang, và phải có một
`06-ma-loi.md` của riêng nó.

## Vì sao là bố cục, và vì sao là bây giờ

0.1 cho ứng dụng hai từ về bố cục, `flow` và `gap`. Mọi màn hình không phải một
danh sách dọc đơn thuần đều không dựng được: một thanh tiêu đề đứng yên trong khi
danh sách cuộn bên dưới, một cột bên cạnh phần nội dung, một hàng nút đẩy về
cuối. Đó là lỗ hổng bản nháp này lấp, và là lỗ hổng duy nhất.

Mô hình là **Flexbox**, đúng phần con mà crate `taffy` cài đặt, vì `taffy` là thứ
bản cài đặt sẽ dùng. Mô tả một mô hình khác với mô hình bản cài đặt tham chiếu
đang chạy thì bảo đảm hai bên sẽ trôi khỏi nhau, mà luật 11 của 0.1 tồn tại chính
vì trôi là thứ dự án này liên tục tìm thấy.

Hai hành vi của Flexbox bị bỏ đi chứ không mô tả — co nhỏ, và trọng số nở — và
[05](05-giao-dien.md) nói rõ ở từng chỗ là bỏ cái gì và vì sao.

## Viết ra rồi mới lộ ba điều

Ba điều không nhìn thấy được trước khi các điều khoản tồn tại:

1. **"Không có pixel" ép ra toàn bộ từ vựng.** 0.1 cấm ứng dụng khai hình thức,
   vì lý do bảo mật: ứng dụng đặt được kích thước theo đơn vị thiết bị thì vẽ
   được thứ không phân biệt nổi với chính giao diện của trình duyệt. Điều đó loại
   mọi đơn vị độ dài, nên kích thước buộc phải thành một **tập từ đóng**. Kết quả
   là chín từ và không con số nào — chặt hơn mọi hệ bố cục đang dùng, và chính
   luật bảo mật của 0.1 làm nên chuyện đó chứ không phải một sở thích.
2. **Phần trăm và kích thước theo nội dung tạo thành một vòng**, và phải có thứ
   gì đó cấm nó. Một phân số làm con phụ thuộc cha; `content` làm cha phụ thuộc
   con. CSS gỡ vòng bằng cách lặng lẽ vứt phần trăm đi. TCC không thể: §3 của
   [05](05-giao-dien.md) từ chối thay vì vứt, và chính luật ấy chặn bố cục lại
   trong một lượt xuống và một lượt lên, nên không cần thêm giới hạn cây nào.
3. **Hình dạng vector kiểm định không diễn đạt nổi một yêu cầu bố cục.** Mọi
   vector sẵn có chỉ nói *nhận* hoặc *từ chối*. "Hai con này bằng nhau" không
   phải cái nào trong hai, nên §12 của [05](05-giao-dien.md) đưa ra loại trường
   hợp thứ hai, khẳng định **quan hệ giữa các hộp** — không bao giờ khẳng định
   hình học tuyệt đối, vì mỗi bản cài đặt tự chọn độ lớn khoảng cách của mình.

## Ba chỗ bản nháp này bị chặn, nói thẳng

### 1. Luật 22 của CI cấm một phiên bản mới thêm mã lỗi

Luật 22 trong `tools/kiem-luat-phu-thuoc.sh` đòi mọi token kiểu gạch nối trong
dấu nháy ngược **ở bất cứ đâu dưới `spec/`** phải có trong
[`spec/0.1/06-ma-loi.md`](../../0.1/vi/06-ma-loi.md), hoặc trong một danh
sách miễn trừ ngắn sửa tay nằm ngay trong kịch bản.

Luật ấy đọc bảng của **0.1** cho tệp của **mọi** phiên bản. Nên không cách nào
gọi tên một mã lỗi mới trong văn xuôi của bất kỳ phiên bản tương lai nào, mà bản
nháp này gọi tên hai mã: `bad-layout` và `bad-scroll`. Cổng kiểm báo đỏ đúng ở
dòng ấy và không ở dòng nào khác.

Luật thì đúng, câu chữ của nó thì sai. Mục đích nó tự nêu là chặn văn xuôi bịa ra
một mã không có trong bảng nào — một lỗi thật nó đã bắt được một lần. Nhưng cách
nó được viết giả định tiêu chuẩn chỉ có đúng một phiên bản. Cách sửa là tra token
theo bảng mã lỗi của **thư mục phiên bản chứa chính tệp ấy**, và lùi về bảng của
0.1 cho tệp nằm ngoài mọi phiên bản. Đó là sửa trong `tools/`, thứ bản nháp này
không sở hữu, nên nó được báo cáo chứ không được sửa.

Nó **không** bị né bằng cách đổi tên hai mã thành từ đơn — làm thế thì qua được
phép kiểm trong khi bỏ mất đúng thứ phép kiểm ấy sinh ra để giữ.
[VERSIONING](../../VERSIONING.md) §3 nói rõ một phiên bản phụ được phép thêm mã
lỗi, nên hiện tiêu chuẩn và CI của nó đang mâu thuẫn nhau.

### 2. Mọi luật khác về đặc tả đều ghim cứng vào 0.1, nên thư mục này không ai canh

Ba luật trong cùng kịch bản làm điều ngược với luật 22 — chúng đơn giản là không
nhìn thấy thư mục này:

| Luật | Kiểm gì | Có ràng buộc `0.2/` không? |
|---|---|---|
| 11 | bản dịch không trôi: đếm tệp `spec/0.1/` với `spec/0.1/vi/`, và hai tập mã lỗi phải y hệt | **Không.** Cả hai đường dẫn viết cứng. Thiếu hay lệch `0.2/vi/` cũng không ai thấy. |
| 23 | không yêu cầu nào của 0.1 tựa vào tài liệu ngoài 0.1 | **Không.** Nó chỉ duyệt `spec/0.1/`, nên các câu **PHẢI** của bản nháp này được phép tựa vào tệp ngoài `0.2/` — và chúng đang tựa thật. |
| 10 | mọi mã trong đặc tả phải tồn tại trong mã nguồn | **Không.** Nó chỉ đọc bảng của 0.1 — và đó là lý do hai mã mới không làm nó đỏ, đúng như phải thế, vì mã của một bản nháp thì chưa nên tồn tại. |
| 12 | không có liên kết chết | **Có**, nó duyệt cả `spec/`. |
| 22 | token gạch nối trong nháy ngược phải là mã thật | **Có**, xem trên. |

Vậy thư mục này chỉ được kiểm liên kết chết và tên mã lỗi, ngoài ra không gì cả.
Bản dịch bên cạnh nó do tay người giữ và không có gì khác giữ. Nên biết điều đó
trước khi tin nó.

### 3. VERSIONING §3 không có dòng nào cho loại thay đổi này

[`VERSIONING.md`](../../VERSIONING.md) §3 phân loại thay đổi theo tác động lên
**gói**: thêm trường, bỏ trường, thu hẹp trường. Phần lớn [05](05-giao-dien.md)
là yêu cầu đặt lên **bộ dựng** — thang bậc khoảng cách phải tăng, không được cắt
phần tràn, vật chứa cuộn phải với tới hết nội dung, tiêu điểm phải được cuộn
vào tầm nhìn. Không gói nào vi phạm được cái nào trong số đó, chúng không đổi
việc gói nào tuân thủ, và bảng ở §3 không xếp loại chúng vào đâu cả.

Điều này quan trọng vì một yêu cầu như thế có thể được thêm vào hoặc bị nới ra mà
không dòng nào của bảng ấy nói là phải tăng phiên bản, trong khi bản cài đặt mà
người dùng đang tin cậy lặng lẽ đổi thứ nó vẽ ra. 0.1 đã có sẵn một yêu cầu kiểu
này ("PHẢI vẽ mỗi ý định khác đi thật") và lỗ hổng ấy áp cho nó y nguyên.

## Trạng thái

`0.2/` — **bản nháp**, mở ngày 22/08/2026, chỉ có bố cục. Chưa đóng băng, chưa
phát hành, chưa cài đặt, không dùng để xét tuân thủ. Xem
[`VERSIONING.md`](../../VERSIONING.md) §6 về những gì một lần phát hành đòi hỏi;
chưa viết được điều nào trong bốn điều ấy.
