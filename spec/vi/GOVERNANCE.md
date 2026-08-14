# Tiêu chuẩn TCC — quản trị

> ⚠️ **Đây là BẢN DỊCH.** Bản chuẩn là [bản tiếng Anh](../GOVERNANCE.md); hai bản
> mâu thuẫn thì bản tiếng Anh thắng.

## 1. Thực tế đang đứng ở đâu

Tính đến 14/08/2026, tiêu chuẩn TCC có **một tác giả, một bản cài đặt, và một bộ
kiểm định tuân thủ, tất cả do cùng một bên làm ra.** Không có nhóm làm việc,
không có hội đồng, không có bản cài đặt thứ hai, không có ai bên ngoài soát.

Điều này viết ra thay vì nói tránh, vì mọi thứ khác trong kho này đều đưa ra
tuyên bố về an ninh, mà một tuyên bố an ninh chỉ đáng bằng quy trình đứng sau
nó. Một tiêu chuẩn, một bản cài đặt tham chiếu, và bộ kiểm định phán xử cả hai,
cùng một tác giả viết ra, không thể bắt được lỗi trong *cách hiểu* của tác giả —
chỉ bắt được lỗi gõ. Ba lỗi an ninh thật trong dự án này được tìm ra bằng cách
đặt một câu hỏi mà không phép thử nào mã hoá được; loại thứ tư, loại mà cả mô
hình đều sai, đúng là loại một tác giả đơn độc mù trước nó.

Nói cụ thể: **tuân thủ tiêu chuẩn này hiện nghĩa là đồng ý với một bản cài
đặt.** Chừng nào chưa có bản cài đặt thứ hai độc lập, đó là tất cả những gì cụm
từ ấy trung thực nói được, và không ai được trình bày nó thành hơn thế.

Hôm nay có hai thứ giảm nhẹ một phần, và giới hạn của chúng đáng nói rõ:

| Thứ giảm nhẹ | Nó thật sự loại trừ được gì |
|---|---|
| Vector ML-DSA-65 neo vào **NIST ACVP** | Nửa hậu lượng tử không phải phương ngữ riêng. Nó không nói gì về phần còn lại của tiêu chuẩn. |
| Đối chiếu chéo với **`dilithium-py`**, bản cài đặt của bên thứ ba | Đồng thuận độc lập về số học của nửa đó. Nó không chung đóng gói, không chung bản kê khai, không chung mô hình quyền năng. |

Mọi thứ ngoài hai dòng đó đều dựa trên phán đoán của một tác giả.

## 2. Ai quyết định, hôm nay

Thay đổi do người bảo trì kho này quyết. Không có kháng nghị, và cũng không giả
vờ là có.

Sắp xếp đó chính đáng đúng chừng nào **chưa ai bên ngoài cam kết theo tiêu
chuẩn.** Ngay khi có bên thứ hai phát hành bản cài đặt hoặc ký gói mà người khác
dựa vào, thay đổi đơn phương thôi không còn là đặc quyền của người bảo trì nữa
mà thành bội tín với sự tin cậy đó — và tài liệu này phải được thay TRƯỚC khi
điều đó xảy ra, không phải sau.

## 3. Điều kiện để coi đây là một tiêu chuẩn thật

> Một người chưa đọc mã nguồn đọc `spec/0.1/` rồi làm ra một gói `.tccapp` hợp lệ
> mà bản cài đặt tham chiếu chấp nhận — **không hỏi ai một câu nào.**

Việc này **CHƯA** làm. Chừng nào chưa làm, `spec/0.1/` là mô tả một bản cài đặt,
không phải một đặc tả, dù câu chữ có cẩn thận tới đâu. Tác giả của đặc tả không
thể tự thực hiện phép thử này; biết trước ý mình định nói chính là thứ đang bị
đem ra thử.

Điều kiện thứ hai, đi ngay sau: một bản cài đặt viết chỉ từ đặc tả đạt 100%
vector kiểm định. Lỗi đầu tiên tìm ra theo đường đó sẽ nói được nhiều hơn mọi
phép thử trong kho này.

## 4. Đề xuất thay đổi

Một đề xuất phải nêu:

1. **Vấn đề**, dưới dạng quan sát được — một gói đáng lẽ nạp được mà không nạp
   được, một chỗ mơ hồ mà hai người đọc hiểu khác nhau, một luật không cưỡng chế
   được.
2. **Ai hỏng**, theo [`VERSIONING.md`](VERSIONING.md) §3.
3. **Kiểm bằng cách nào** — vector kiểm định nào, và theo luật 2 của
   [`README.md`](../README.md), không điều nào vào tiêu chuẩn mà không có ít nhất
   một vector.
4. **Đã thử gì khác.** Một thay đổi lẽ ra chỉ cần là đính chính, là sửa bản cài
   đặt, hay là không làm gì cả, thì nên là một trong ba thứ đó.

Đề xuất THÊM một trường gánh nặng chứng minh cao hơn đề xuất BỎ một trường. Vì
trường lạ bị từ chối, một trường thêm vào phá mọi bản cài đặt đang có
([`VERSIONING.md`](VERSIONING.md) §3), nên "chỉ thêm vào thôi mà" không bao giờ
là một lý lẽ ở đây.

## 5. Những luật không thuộc quyền nới của người bảo trì

Những điều sau đúng bất kể sau này ai quản trị tiêu chuẩn, và đề xuất đổi một
trong số đó là đề xuất từ bỏ thiết kế:

1. **Không giao dịch mainnet trước khi có kiểm định an ninh độc lập.** Nêu ở
   [`../SECURITY.md`](../../SECURITY.md). Không hạn chót, không buổi trình diễn,
   không lần ra mắt nào ghi đè được.
2. **Không bao giờ hiện "đã xác minh nhà phát hành".** Chữ ký chứng minh gói
   không bị sửa. Nó không chứng minh gì về việc AI ký, và 0.1 không có sổ khoá.
   Bản cài đặt hiện huy hiệu xác minh nhà phát hành là không tuân thủ, dù câu chữ
   bên cạnh có cẩn thận tới đâu.
3. **Quyền năng không tồn tại cho tới khi được cấp**, và người dùng trả lời từng
   mục một. Gộp các mục xin quyền lại là phá mô hình, dù giao diện có sức ép tới
   đâu.
4. **Chữ ký lai không bao giờ tụt xuống một thuật toán.** Xem
   [`VERSIONING.md`](VERSIONING.md) §4.

Luật 2 và 3 nói về thứ bản cài đặt hiện ra cho một con người. Chúng nằm trong
tiêu chuẩn vì mật mã trở nên vô giá trị nếu giao diện thuật lại sai thứ nó đã
chứng minh — mà thuật sai là cách rẻ nhất để trông có vẻ đáng tin.

## 6. Được phép tuyên bố gì

Tiêu chuẩn và bản cài đặt tham chiếu phát hành theo **Apache-2.0**, có kèm điều
khoản cấp phép sáng chế (§3). Đó là một phần có chủ ý của việc làm cho bản cài
đặt thứ hai khả thi: được phép đọc không có nghĩa là được phép dựng.

| Tuyên bố | Được không |
|---|---|
| "Đạt bộ vector kiểm định TCC 0.1" | Được, nếu đúng thế — vector công khai và tuyên bố này kiểm được |
| "Tuân thủ TCC 0.1" | Được, kèm cảnh báo thường trực ở §1: chỉ có một bản cài đặt để mà đồng ý |
| "Đã kiểm định" / "đã xác minh nhà phát hành" / "an toàn lượng tử" | **Không.** Chưa có kiểm định nào; 0.1 không có danh tính nhà phát hành; và không ai hứa nổi rằng một thuật toán còn đứng vững |

Không có tổ chức cấp chứng nhận, không có nhãn hiệu, không có sổ đăng ký các bản
cài đặt tuân thủ. Không ai được nhận là phát ngôn thay tiêu chuẩn, kể cả người
bảo trì nó, ngoài những gì các tài liệu này nói.

## 7. Thay tài liệu này

Đây là sắp xếp cho một tiêu chuẩn chưa có người dùng bên ngoài. Nó được dự là sẽ
không đủ — và sẽ phải thay TRƯỚC, không phải sau, khi bên ngoài đầu tiên dựa vào
nó. Bản thay thế tối thiểu phải nói: ai được đóng băng một phiên bản, ai được
tuyên bố khai tử một bộ ký, và chuyện gì xảy ra khi người bảo trì biến mất.

Câu cuối cùng đó hôm nay chưa có lời đáp.
