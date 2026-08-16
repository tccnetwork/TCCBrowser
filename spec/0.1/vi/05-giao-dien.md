# 05 — Giao diện

## Ứng dụng KHÔNG gửi kèm mã

Điểm vào là một **cây component khai báo**, JSON, tối đa **1 MiB**.

Không phải HTML. Không phải kịch bản. Ứng dụng nói *có gì trên màn hình*; bản
cài đặt quyết định *vẽ ra sao*.

**Vì sao:** nếu ứng dụng gửi thẻ đánh dấu của trang web, thì ngày có bộ dựng
khác, **mọi ứng dụng phải viết lại** — và lúc đó không ai dám đổi bộ dựng nữa.
Giàn giáo hoá thành nhà.

## Ví dụ

```json
{
  "kind": "group",
  "flow": "column",
  "gap": "large",
  "children": [
    { "kind": "text", "content": "Xin chào từ TCC", "emphasis": "title" },
    { "kind": "image", "source": "anh/logo.png",
      "alt": { "kind": "text", "text": "Biểu trưng TCC" } },
    { "kind": "field", "label": "Tìm kiếm" },
    { "kind": "button", "label": "Xoá dữ liệu", "action": "xoa", "tone": "danger" }
  ]
}
```

## Sáu loại nút

| `kind` | Trường | |
|---|---|---|
| `text` | `content` PHẢI · `emphasis` (`title`/`normal`/`subtle`/`warning`, mặc định `normal`) | Đoạn chữ; CHO xuống dòng |
| `button` | `label` PHẢI · `action` PHẢI · `tone` (`neutral`/`primary`/`danger`, mặc định `neutral`) | |
| `field` | `label` PHẢI · `value` (mặc định rỗng) | Ô nhập. `secret` bị **từ chối** — xem bên dưới |
| `toggle` | `label` PHẢI · `action` PHẢI · `on` (mặc định **`false`**) | Công tắc |
| `image` | `source` PHẢI · `alt` **PHẢI** | Ảnh trong gói |
| `group` | `flow` (`row`/`column`, mặc định `column`) · `gap` (`none`/`small`/`medium`/`large`, mặc định `medium`) · `children` | Loại DUY NHẤT nhận nút con |

Trường lạ → **PHẢI** từ chối. Nó gần như luôn là gõ sai tên, và im lặng bỏ qua
nghĩa là người viết tưởng đã đặt được một thuộc tính mà thật ra không.

### `warning` được THÊM vào `emphasis`, và vì sao

[04](04-quyen-nang.md) bắt buộc quyền ví ký được phải **hiện khác hẳn mọi quyền
khác**. Từ vựng ở đây chỉ có `title`, `normal`, `subtle` — không giá trị nào nói
được "khác hẳn". Tiêu chuẩn đòi một thứ mà nó không cho phương tiện để nói.

Đó là lỗi của TIÊU CHUẨN chứ không phải của bản cài đặt, nên sửa ở đây. Thêm một
giá trị là thay đổi phá vỡ ([VERSIONING](../../VERSIONING.md) §3): `emphasis` là
tập đóng, nên bộ dựng **không biên dịch được** cho tới khi xử lý giá trị mới. Cái
giá đó chính là điểm mạnh — xem ghi chú về `toggle` bên dưới.

`warning` nghĩa là: dòng chữ này phải nổi rõ hơn mọi dòng quanh nó. Như mọi ý
định khác, ứng dụng khai ý định còn bản cài đặt quyết định hình thức — nhưng nó
**PHẢI** trông khác thật, theo đúng luật trên.

## Không có pixel, không có màu

Ứng dụng khai **Ý ĐỊNH** (`tone: "danger"`, `gap: "large"`), không khai hình thức.

Đây là chỗ bảo mật, không phải chỗ thẩm mỹ: nếu ứng dụng tự đặt được màu thì nút
xoá sổ ví có thể trông y hệt nút huỷ.

Đổi lại, bản cài đặt **PHẢI** vẽ mỗi ý định khác đi thật. Khai `danger` mà vẽ y
hệt nút thường thì cả tầng sắc thái chỉ là chú thích trong mã.

## Trợ năng — không có đường bỏ qua

Mỗi nút **PHẢI** sinh ra được một nút trợ năng với **vai trò** và **nhãn**.

`alt` của ảnh **KHÔNG có giá trị mặc định**. Thiếu nó là **lỗi**, không phải "coi
như trang trí" — "quên viết mô tả ảnh" phải bị chặn, chứ không được lặng lẽ biến
thành "ảnh này không cần mô tả".

```json
"alt": { "kind": "text", "text": "Biểu đồ giá" }   // ảnh mang thông tin
"alt": { "kind": "decorative" }                     // trang trí, PHẢI khai ra miệng
```

Bản cài đặt **PHẢI**:

- Hiện **nhãn nhìn thấy được** cho `field` và `toggle`. Chỉ đặt nhãn cho trình
  đọc màn hình là để người sáng mắt thấy một ô trống không biết dùng làm gì.
- Cho `secret: true` **do khung trình duyệt dựng** ra một ô nhập mật khẩu **thật**
  của nền tảng, để hệ điều
  hành che chữ và không đưa nội dung vào gợi ý gõ.
- Báo cho trình đọc màn hình biết `tone: "danger"` là hành động **không hoàn tác
  được**, và giữ nguyên thông tin đó là một **nút**.

⚠️ **Đừng thêm chú giải trợ năng khi thẻ gốc đã nói đúng.** Trên nền web, đặt
`role="textbox"` lên một ô mật khẩu sẽ **đè** ngữ nghĩa gốc và kéo nó từ "ô bảo
mật" tụt xuống "ô thường" — trình đọc màn hình khi đó đọc to từng ký tự mật khẩu.

### Gói KHÔNG được dựng ô nhập che chữ

`field` mang `"secret": true` **PHẢI** bị từ chối với mã `secret-field-from-app`.
`field` thường vẫn được phép: ô tìm kiếm là việc bình thường, và cấm cả nó chỉ
đẩy người viết ứng dụng đi vẽ một thứ *trông giống* ô nhập — lúc ấy không ai
phân biệt được đâu là ô nhập thật nữa.

Ô che chữ là **hình dạng người dùng được dạy để tin**. Một hàng chấm tròn nghĩa
là "chỗ này an toàn, gõ bí mật vào đi", và nghĩa ấy không sống nổi nếu ai cũng
dựng ra được: một gói đã ký có thể vẽ *"Nhập mã PIN ví của bạn"* trong một ô mật
khẩu thật, không phân biệt được với ô của chính trình duyệt.

**Điều này làm được trong 0.1 cho tới ngày 16/08/2026**, và ví dụ ngay trong
chính tệp này từng ghi `{"kind": "field", "label": "Mật khẩu", "secret": true}`
— tức là đặc tả đang dạy người ta làm việc ấy. Bỏ một giá trị từng được phép là
**thay đổi phá vỡ** theo [VERSIONING.md](../../VERSIONING.md) §2, và được ghi
lại đúng như thế.

Điều này **không** vá được: gói vẫn vẽ được một ô thường gắn nhãn "PIN". Chắn ở
đó là màn hình của trình duyệt vốn không do gói vẽ ra chút nào, và nó yếu hơn
chắn này. Đừng đọc điều khoản này thành "lời mời gõ bí mật do gói vẽ giờ an
toàn" — nó gỡ đi hình dạng mang niềm tin, không gỡ được câu chữ.

## Trần cây

| | |
|---|---|
| Số nút tối đa | **10 000** |
| Độ sâu tối đa | **32** |
| Độ dài một chuỗi | **4 096 ký tự** (đếm ký tự, KHÔNG đếm byte) |
| Kích thước tệp | **1 MiB** |

> **Vì sao 32 chứ không lớn hơn.** Mỗi tầng của cây tốn **hai** tầng lồng JSON —
> một đối tượng và một mảng. Trần 64 vì thế cần 128 tầng JSON, đúng chỗ nhiều bộ
> đọc JSON phổ biến dừng lại theo mặc định: bộ đọc từ chối tài liệu TRƯỚC khi
> luật này kịp chạy, nên `too-deep` thành mã chết và một cây ở đúng trần đã ghi
> lại bị từ chối. Tệ hơn cho tính liên thông: bản cài đặt có bộ đọc lồng sâu hơn
> trả `too-deep`, bản nông hơn trả `bad-json` — cùng một gói, hai mã lỗi. Trần
> của TIÊU CHUẨN không được phụ thuộc vào giới hạn đệ quy của thư viện JSON mà
> người cài đặt tình cờ chọn.

Trần **PHẢI** kiểm **trong lúc dựng**, không phải sau khi dựng xong: một ứng dụng
thù địch chỉ cần một vòng lặp là dựng ra cây khổng lồ, và lúc đó nó đã nằm trong
bộ nhớ rồi.

Đếm **ký tự** chứ không đếm byte: cắt theo byte thì tiếng Việt có dấu chỉ được
viết bằng khoảng một nửa tiếng Anh.

## ⚠️ Giải mã KHÔNG được đi vòng qua phép kiểm

Cây đến từ đĩa **PHẢI** đi qua **y hệt** mọi phép kiểm như cây dựng trong mã: lọc
ký tự giả mạo, trần, ràng buộc mã hành động, cấm ảnh trỏ ra mạng.

Nhiều thư viện JSON cho phép giải mã thẳng vào cấu trúc đích, nhồi vào từng
trường và **bỏ qua sạch** tầng kiểm. Kẻ gian khi đó không cần tấn công gì — chỉ
cần ship một tệp JSON.

Bản cài đặt **NÊN** dùng **hai kiểu riêng**: một kiểu dữ liệu trần để giải mã, và
một bước dựng lại qua đúng các hàm có kiểm.

## `source` của ảnh

Đường dẫn **trong gói**, chịu ràng buộc ở [01](01-goi.md).

**KHÔNG ĐƯỢC** là địa chỉ mạng. Ảnh tải từ mạng là một cái đèn báo hiệu: chủ máy
chủ ảnh biết ai mở màn hình nào, lúc nào, từ địa chỉ nào — trong khi ứng dụng
chưa hề xin quyền mạng. Mã lỗi: `external-image`.

Bản cài đặt phục vụ ảnh **PHẢI**:

1. Chỉ phục vụ tệp **có trong cây đã ký**
2. Kiểm đường dẫn bằng đúng luật ở [01](01-goi.md), **sau khi** giải mã `%XX` và
   cắt phần truy vấn — `%2e%2e%2f` chính là `../` viết trá hình
3. Chọn kiểu nội dung theo **danh sách trắng đuôi tệp**, và **KHÔNG ĐƯỢC** phục
   vụ SVG: nó chạy được kịch bản và nhúng được tài nguyên ngoài — nó là một tài
   liệu, không phải một tấm ảnh

## `action` — mã hành động

Chỉ **chữ thường ASCII, chữ số, `-`, `.`**; 1–128 ký tự. Mã lỗi: `bad-action-id`.

Mã hành động **không** hiện ra cho người dùng. Nó nối một nút bấm với một hành vi
khai trong bản kê khai — xem dưới.

## Hành vi của nút

Khai trong **bản kê khai**, KHÔNG khai trong cây giao diện:

```json
"actions": [
  { "id": "tai-hang", "effect": { "kind": "fetch", "host": "shop.tcc-coin.com", "path": "/ds" } }
]
```

Ba lý do, lý do nào cũng đủ:

1. **Chữ ký bao trùm bản kê khai** — hành vi là thứ nguy hiểm nhất ứng dụng khai,
   nó không được sửa sau khi ký.
2. **Hộp thoại hỏi quyền đọc bản kê khai** — nên hiện được "nút này gọi
   shop.tcc-coin.com".
3. **Giữ tầng giao diện sạch** — khai ở cây giao diện nghĩa là tầng giao diện
   phải biết tới mạng.

Ràng buộc:

- `id` PHẢI hợp lệ như `action` ở trên, và **KHÔNG ĐƯỢC** trùng nhau
- `path` PHẢI bắt đầu bằng `/`
- ⚠️ `host` **PHẢI nằm trong quyền mạng đã xin**

Điều cuối là phép kiểm quan trọng nhất của hành vi. Không có nó, ứng dụng khai
được một nút gọi `ke-gian.example` trong khi chỉ xin quyền tới `shop.tcc-coin.com`.
Lúc chạy quyền năng vẫn chặn — nhưng **người dùng đã bấm, không thấy gì xảy ra,
và không ai biết vì sao**. Mã lỗi: `action-host-not-granted`.

Luật khớp ở đây **PHẢI** y hệt luật khớp lúc chạy ([04](04-quyen-nang.md)) — lệch
hai bên là một lỗ.

## Chạy hành vi

```text
1. Tra hành động trong bản kê khai ĐÃ KÝ   ← không có thì KHÔNG chạy gì
2. Hỏi QUYỀN NĂNG                          ← chưa cấp thì dừng ở đây
3. Mới gọi ra ngoài
```

Bước 2 **PHẢI** đứng trước bước 3. Gọi trước rồi kiểm sau nghĩa là gói tin **đã
rời khỏi máy** — mà với một máy chủ theo dõi, chỉ cần gói tin đến nơi là đủ.

Mã hành động đến từ cú bấm trên màn hình. Không tra lại trong bản kê khai đã ký
thì một trang bị chiếm quyền tự bịa ra hành động được.
