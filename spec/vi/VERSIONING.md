# Tiêu chuẩn TCC — phiên bản và khai tử

> ⚠️ **Đây là BẢN DỊCH.** Bản chuẩn là [bản tiếng Anh](../VERSIONING.md); hai bản
> mâu thuẫn thì bản tiếng Anh thắng.
>
> Chính sách này áp cho **tiêu chuẩn**, không áp cho bản cài đặt tham chiếu và
> không áp cho ứng dụng. Trường `version` của ứng dụng là việc của ứng dụng;
> tiêu chuẩn không ràng buộc dạng của nó.

## 1. Một phiên bản là một thư mục, và nó bất biến

Tiêu chuẩn đánh phiên bản theo `spec/<chính>.<phụ>/`. Đã **đóng băng** thì tệp
trong đó không bao giờ đổi nữa — không sửa lỗi chính tả, không làm rõ thêm. Đổi
tức là hai người cùng nói "tôi cài đặt 0.1" lại cài đặt hai thứ khác nhau, mà
trong tay không ai có gì để phát hiện ra.

Đính chính cho phiên bản đã đóng băng ghi vào `spec/<phiên bản>/ERRATA.md`, chỉ
được nối thêm. Đính chính **KHÔNG ĐƯỢC** làm thay đổi thứ gì là hợp lệ; nó chỉ
được làm rõ luật đã có. Nếu một sửa đổi làm thay đổi thứ gì hợp lệ, đó không
phải đính chính — đó là phiên bản kế tiếp.

Bản 0.1 **chưa đóng băng**. Nó là bản nháp làm việc và có thể đổi bất cứ lúc nào
cho tới khi việc đóng băng được công bố theo [`GOVERNANCE.md`](GOVERNANCE.md).

## 2. Không có tương thích tiến, và đó là cố ý

`spec_version` **PHẢI** khớp chính xác. Bản cài đặt 0.1 từ chối gói 0.2; không
đoán, không chạy một nửa, không lùi về mặc định.

Đây là lựa chọn có chủ ý, và nó chính là lý do trường lạ bị từ chối
([02 §Trường lạ](../0.1/vi/02-ban-ke-khai.md)). Phần lớn định dạng mua lấy sự "xuống
cấp êm ái" bằng cách bỏ qua thứ chúng không hiểu. TCC không mua nổi, vì ở định
dạng này thứ bị bỏ qua có thể là một **ranh giới quyền**: một phạm vi mọc thêm
trường thu hẹp sẽ bị *nới rộng* ra bởi mọi bản cài đặt bỏ qua trường đó. Mở
không được gói là một thất bại nhìn thấy được và cứu được. Im lặng cấp nhiều
quyền hơn bản kê khai nói thì không phải cả hai.

Nên đổi chác nói thẳng: **TCC không có xuống cấp êm ái, và đổi lại được điều
này — thứ được ký đúng bằng thứ được kiểm.**

## 3. Thế nào là phá vỡ

Luật quen thuộc "thêm vào thì an toàn" KHÔNG đúng ở đây. Vì trường lạ bị từ
chối, **mọi trường thêm vào đều là thay đổi phá vỡ** — gói dùng nó sẽ không nạp
được trên bất kỳ bản cài đặt cũ nào.

| Thay đổi | Nâng phiên bản |
|---|---|
| Thêm bất kỳ trường nào cho bản kê khai, phạm vi, hay giao diện | **chính hoặc phụ — luôn luôn** |
| Thêm một loại component hay một tên quyền năng | **chính hoặc phụ — luôn luôn** |
| Bỏ hoặc đổi tên bất cứ thứ gì | **chính** |
| Đổi dạng chuẩn tắc hoặc băm nội dung | **chính** |
| Đổi chuỗi byte được ký, hoặc giao diện FIPS 204 | **chính** |
| Thu hẹp thứ mà một trường đang cho phép | **chính** |
| Thêm mã lỗi mới cho trường hợp trước đó chưa có mã | phụ |
| Làm rõ câu chữ, thêm ví dụ, thêm vector kiểm định mà bản cài đặt hiện tại đã đạt | không — đính chính |

**Hai phiên bản phụ KHÔNG tương thích với nhau, theo cả hai chiều.** 0.1 và 0.2
là hai tiêu chuẩn có họ hàng với nhau. Phân biệt chính/phụ ở đây chỉ ghi lại
*bao nhiêu phần thiết kế còn sống sót*, cho người đọc lịch sử — nó không phải
một lời hứa tương thích, và không bản cài đặt nào được suy ra một lời hứa như
vậy.

Mọi thay đổi có thêm trường **PHẢI** kèm vector kiểm định **trượt** trên bản cài
đặt của phiên bản trước. Một lần nâng phiên bản mà không ai đo được thì không
phải nâng phiên bản.

## 4. Khai tử thuật toán mật mã

Đây là phần chắc chắn sẽ phải dùng tới. `hybrid-ed25519-mldsa65-v1` tồn tại vì
không ai biết nửa nào gãy trước.

**Ký và kiểm khai tử TÁCH RỜI nhau, không bao giờ cùng lúc.**

1. **Không khuyến khích.** Bộ ký vẫn ký được, vẫn kiểm được. Công cụ cảnh báo khi
   ký. Không thứ gì người dùng đang có ngừng chạy.
2. **Chỉ kiểm.** Gói mới **KHÔNG ĐƯỢC** ký bằng nó nữa; bản cài đặt **PHẢI** vẫn
   kiểm được gói cũ. Bước này kéo dài ít nhất cho tới khi cả hệ sinh thái đã ký
   lại, và không có giới hạn trên.
3. **Từ chối.** Kiểm chữ ký thất bại. Bước này làm mọi gói từng ký bằng bộ ký đó
   không mở được nữa.

Bước 3 **KHÔNG ĐƯỢC** làm trong cùng phiên bản với bước 2, và **KHÔNG ĐƯỢC** làm
khi còn gói nào đã biết còn dựa vào bộ ký ấy. Bỏ khả năng kiểm là xoá dữ liệu —
ứng dụng không mở được nữa và trạng thái nó lưu thành không với tới được — nên
việc này được xử như phá huỷ dữ liệu, không phải như bảo trì.

**Một ngoại lệ, và nó đảo ngược thứ tự:** nếu bộ ký gãy tới mức *giả mạo được*
trong thực tế, "chỉ kiểm" không còn là chỗ dừng an toàn, vì kiểm một chữ ký giả
mạo được còn tệ hơn từ chối một chữ ký thật. Khi đó bước 3 theo ngay lập tức và
tiêu chuẩn phải nói rõ, nêu đích danh chỗ gãy. Bộ ký chỉ *yếu đi* — chưa giả mạo
được, chỉ là không còn đạt mức an toàn nhắm tới — thì đi theo thứ tự thường.

**Bộ ký mới không bao giờ thay một bộ lai bằng một thuật toán đơn.** Bộ lai tồn
tại để một nửa gãy vẫn sống được. Cho `ed25519` về hưu nghĩa là thay nó bằng một
thuật toán cổ điển khác đứng cạnh nửa hậu lượng tử, chứ không phải tụt xuống còn
mỗi ML-DSA. Ràng buộc này chỉ được gỡ bởi một phiên bản chính có lập luận nằm
ngay trong văn bản tiêu chuẩn.

## 5. Khai tử một quyền năng

Không được im lặng thu hẹp hay bỏ một quyền năng: ứng dụng xin nó thì đã cài,
đã được cấp, và quyết định quyền đã lưu được khoá theo phạm vi
([`permission_store.rs`](../../crates/tcc-shell/src/permission_store.rs)).

Bỏ một quyền năng ở phiên bản N nghĩa là ứng dụng khai nó sẽ không nạp được dưới
N. Điều đó chấp nhận được — nó nhìn thấy được và nó hỏng về phía an toàn. Thứ
**KHÔNG** được phép là giữ nguyên tên quyền mà đổi thứ phạm vi của nó cấp, theo
bất kỳ chiều nào. Nới ra thì im lặng cho ứng dụng đã cài nhiều hơn thứ người
dùng đã đồng ý; thu vào thì im lặng làm nó hỏng mà không ai chẩn đoán được. Đổi
nghĩa thì đổi tên.

## 6. Công bố một phiên bản

Phát hành phiên bản mới không phải là commit tệp lên. Nó đòi, gom trong một chỗ:

- đổi gì, và với mỗi thay đổi, vì sao văn bản cũ sai;
- vector kiểm định nào là mới, và cái nào giờ trượt trên phiên bản trước;
- người đã cài đặt phiên bản trước phải làm gì;
- với mỗi khai tử, đang ở bước nào (§4) và cái gì kích hoạt bước kế tiếp.

Viết không ra những thứ đó thì thay đổi chưa chín.

## 7. Riêng bản 0.1

Bản 0.1 **không có cơ chế nâng cấp** — không có cách thay gói đã cài bằng bản mới
hơn, nên cũng không có chuyện di trú dữ liệu đã lưu. Điều này liệt kê trong
[`0.1/vi/README.md`](../0.1/vi/README.md) như một lỗ hổng đã biết, và nó nghĩa là bản 0.2
thật sự sẽ phải định nghĩa việc cập nhật gói trước khi triển khai bất cứ thứ gì
mong sống lâu hơn nó.

Không thứ gì trong 0.1 được coi là đóng băng cho tới khi việc đóng băng được
công bố. **Không giao dịch mainnet nào được dựa vào bất kỳ phần nào của nó trước
khi có kiểm định an ninh độc lập** — cổng đó nêu ở [`../SECURITY.md`](../../SECURITY.md)
và chính sách này không ghi đè lên nó.
