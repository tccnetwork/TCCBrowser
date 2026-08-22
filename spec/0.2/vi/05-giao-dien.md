# 05 — Giao diện: bố cục

> ## ⛔ BẢN NHÁP. CHƯA BẢN CÀI ĐẶT NÀO THOẢ. KHÔNG DÙNG ĐỂ XÉT TUÂN THỦ.
>
> Luật 1 của [`spec/README.md`](../../README.md) nói tiêu chuẩn được **rút ra từ
> mã đang chạy, không bao giờ viết trước mã**. Tệp này là ngoại lệ, và nó ghi rõ
> điều đó ở mọi trang: nó được viết **TRƯỚC** mã, nên nó là **đề xuất về những
> điều khoản mà một hệ bố cục sẽ phải thoả**, không phải mô tả một hệ đang có.
>
> Không điều nào ở đây được viện làm yêu cầu, được trích trong một tuyên bố tuân
> thủ, hay được dùng để từ chối một gói. Bộ vector của nó
> ([`conformance/vectors/layout.json`](../../../conformance/vectors/layout.json))
> **cố ý không được nối vào bộ chạy**, nên chưa bản cài đặt nào đạt hay trượt
> chúng.
>
> ⚠️ **Đây là BẢN DỊCH.** Bản chuẩn là [bản tiếng Anh](../05-interface.md); hai
> bản mâu thuẫn nhau thì bản tiếng Anh thắng.

## Tệp này là gì, và không là gì

Tệp này chỉ soạn thảo **một chủ đề: bố cục.** Nó là một mảnh, không phải một
phiên bản.

[`spec/0.1/README.md`](../../0.1/vi/README.md) nói mọi thứ một bản cài đặt phải
thoả đều nằm trong thư mục phiên bản của chính nó, và không gì ngoài thư mục ấy
mang tính quy phạm. Vậy một bản 0.2 thật phải chép lại **mọi** điều khoản của
0.1 chứ không được trỏ ngược về. Thư mục này chưa làm điều đó, nên nó chưa phải
một phiên bản — nó là bản nháp của những điều khoản mà phiên bản ấy sẽ thêm vào.
Mọi liên kết từ đây vào `0.1/` đều mang tính **tham khảo**: nó cho biết bản nháp
này giả định điều gì vẫn còn đúng, không phải điều 0.2 yêu cầu.

Mọi thứ trong [`0.1/05-interface.md`](../../0.1/vi/05-giao-dien.md) không thuộc
về bố cục — sáu loại nút, luật chuỗi hiển thị, luật trợ năng, lệnh cấm gói tự vẽ
ô nhập che, mã hành động, cách tra hành vi — đều được giả định là không đổi và
**không được chép lại**. Giả định đó là món nợ lớn nhất của bản nháp này.

## Vì sao cần bố cục, và vì sao lại có hình dạng này

0.1 cho ứng dụng đúng hai từ về bố cục: `flow` (`row` hay `column`) và `gap`.
Chừng ấy đủ cho một danh sách và không đủ cho gì khác. Mọi màn hình thật — một
thanh tiêu đề đứng yên trong khi danh sách cuộn bên dưới, một cột bên cạnh phần
nội dung, một hàng nút đẩy về cuối — đều cần ít nhất một cách nói *to bao nhiêu*
và *đặt ở đâu*.

Mô hình soạn ở đây là **Flexbox**, đúng phần con mà crate `taffy` cài đặt, vì
bản cài đặt sẽ dùng `taffy`, và một tiêu chuẩn mô tả mô hình khác với mô hình
mà bản cài đặt tham chiếu của nó đang chạy là một tiêu chuẩn sẽ bị lặng lẽ bỏ
qua ở đúng những chỗ hai bên khác nhau. Hai hành vi của Flexbox bị **bỏ đi** chứ
không mô tả, và mỗi chỗ bỏ đều nói rõ là bỏ và vì sao.

**Không có pixel.** [0.1](../../0.1/vi/05-giao-dien.md) cấm ứng dụng khai hình
thức, và đó là luật bảo mật chứ không phải luật thẩm mỹ: ứng dụng đặt được kích
thước theo đơn vị thiết bị thì vẽ được cái nút trông y hệt nút của chính trình
duyệt. Nên mọi giá trị dưới đây là **một từ trong tập đóng**, không bao giờ là
một con số. Bản nháp này không có đơn vị độ dài nào, và thêm một đơn vị vào là
mở lại đúng cái lỗ ấy.

## 1. Khung, và nút gốc

Bản cài đặt trải cây vào một **khung**: một hình chữ nhật do nó chọn, có kích
thước **xác định** trên cả hai trục — hữu hạn, lớn hơn không, và biết trước khi
bắt đầu tính bố cục.

Nút gốc được trải **như thể nó là con duy nhất của một nhóm ngầm** mang đúng
những giá trị sau:

| | |
|---|---|
| `flow` | `column` |
| `gap` | `none` |
| `padding` | `none` |
| `align_main` | `start` |
| `align_cross` | `stretch` |
| `wrap` | `false` |
| `scroll` | `false` |
| kích thước | chính là khung |

Hai hệ quả, cả hai đều được nói ra vì để ngầm bất kỳ cái nào cũng là lặp lại lỗi
dự án này đã trả giá một lần — 0.1 mất thời gian vì câu *"nút gốc ở độ sâu 0 hay
độ sâu 1"*:

- **Trục chính của nút gốc là trục dọc**, bất kể `flow` của chính nó viết gì.
  `flow` của một nút chi phối **các con của nó**, không bao giờ chi phối chính
  nó. Một nút gốc `flow: "row"` vẫn có trục chính của riêng nó là trục dọc; các
  con của nó mới có trục chính nằm ngang.
- **Hộp của nút gốc CHÍNH LÀ khung.** Kích thước của nó bằng kích thước khung
  trên cả hai trục. Khai `size`, `min` hay `max` trên nút gốc bị từ chối với mã
  `bad-layout` — khung không thương lượng được, nên một lời khai như thế không
  thể có hiệu lực, và lý do 0.1 đưa ra khi từ chối trường lạ áp dụng nguyên vẹn:
  người viết sẽ tưởng đã đặt được một thuộc tính mà thật ra không.

Nhóm ngầm **không phải một nút**. Nó không tính vào giới hạn số nút, không sinh
nút trợ năng, và không địa chỉ hoá được. Chỗ nào luật bên dưới đếm tổ tiên thì
nhóm ngầm không phải một tổ tiên.

### Khung luôn cuộn được

Nội dung có thể vượt ra ngoài khung. Vì khung là mép của mặt vẽ, thứ gì vẽ ra
ngoài nó thì không cách nào nhìn thấy bằng cách di chuyển bất cứ thứ gì bên
trong cây.

Nên: bản cài đặt **PHẢI** làm cho chính cái khung cuộn được đủ xa để **mọi hộp
được vẽ đều có thể đưa hoàn toàn vào tầm nhìn**, trên cả hai trục.

**Vì sao:** không có luật này thì lời hứa ở §9 — rằng không gì gói vẽ ra có thể
trở nên không với tới được — đứt ngay ở đỉnh cây, mà đó đúng là chỗ tràn nhiều
nhất. Một lời cảnh báo bị đẩy quá đáy cửa sổ một dòng là một lời cảnh báo chưa
từng được hiện ra.

Việc cuộn của khung không phải một nút và không tính vào luật lồng nhau ở §9.3.

## 2. Trục

Với một nhóm, `flow` gọi tên **trục chính** của nó: `row` → nằm ngang, `column`
→ nằm dọc. **Trục phụ** là trục còn lại.

**Trục chính và trục phụ của một nút là của nhóm CHA nó**, không phải của chính
nó. Đây là câu người cài đặt hay hiểu sai nhất. `size.main` trên con của một
`row` là kích thước ngang; đúng đoạn JSON ấy trên con của một `column` là kích
thước dọc.

`row` chạy theo **chiều đọc của giao diện**, và các con được đặt dọc trục chính
theo **thứ tự trong mảng**. Không có cách nào đảo ngược cả hai điều đó. **Vì
sao:** một trường đảo chiều sẽ cho phép thứ tự nhìn thấy và thứ tự trong mảng
mâu thuẫn nhau, mà thứ tự trong mảng là thứ cây trợ năng và đường đi bàn phím
bám theo — nên cái nút người dùng thấy dưới con trỏ và cái nút bàn phím đang
đứng có thể là hai nút khác nhau. 0.1 đã từ chối cho gói tách rời hình dạng của
một điều khiển khỏi việc nó làm; đây là đúng lời từ chối ấy ở chiều thứ hai.

⚠️ Hình học trong giao diện đọc từ phải sang trái **chưa được soạn**. Toàn bộ
vector của tệp này được đặc tả cho chiều trái-sang-phải và ghi rõ điều đó. Đây là
một lỗ hổng, không phải một quyết định.

## 3. Tính xác định — luật làm cho bố cục dừng lại được

Mỗi nút có **hai trục vật lý**, và mỗi trục hoặc **xác định** (biết được kích
thước mà không cần đo nội dung của nút) hoặc **suy từ nội dung** (kích thước là
đúng thứ nội dung cần).

Lời khai nào của một nút chi phối trục vật lý nào là do `flow` của **cha** quyết
định (§2). Vậy tính xác định được theo dõi **theo trục vật lý**, không theo tên
`main`/`cross`.

Một trục của nút **N**, con của nhóm **P**, là **xác định** đúng khi một trong
các điều sau đúng:

1. N là nút gốc. Cả hai trục của nó đều xác định (§1).
2. Lời khai chi phối trục ấy trong `size` của N là một **phân số** (§4.1),
   **và** kích thước trong của P trên trục ấy là xác định, **và** — nếu trục ấy
   là trục phụ của P — `P.wrap` là `false`.
3. Lời khai chi phối trục ấy là `fill`, **và** kích thước trong của P trên trục
   ấy là xác định, **và** — nếu trục ấy là trục phụ của P — `P.wrap` là
   `false`. (§4.4: trên trục phụ, `fill` nghĩa là toàn bộ kích thước phụ trong
   của cha, nên nó tính ra đúng như `full`.)
4. Trục ấy là **trục phụ của P**, N không khai `size` trên trục ấy,
   `P.align_cross` là `stretch`, `P.wrap` là `false`, **và** kích thước phụ
   trong của P là xác định. (Đây là kéo giãn: dòng duy nhất lấp đầy vật chứa,
   nên kích thước phụ của con đến từ vật chứa chứ không từ nội dung của nó.)

Ngoài ra thì trục ấy **suy từ nội dung**.

Hai điều kiện `wrap` chính là điều dòng cuối của §8 nói bằng lời: bên trong một
nhóm có gãy dòng, kích thước phụ của một con đến từ **dòng của nó**, mà một dòng
thì lấy kích thước từ nội dung. Nên không lời khai nào làm nó xác định được.

**`fill` hoặc một phân số khai trên trục mà kích thước của cha suy từ nội dung
thì bị từ chối với mã `bad-layout`.**

**Vì sao, và đây là đoạn chịu lực của cả tệp này:** một phân số làm kích thước
của con phụ thuộc vào cha; `content` làm kích thước của cha phụ thuộc vào con.
Cho hai thứ ấy gặp nhau thì quan hệ phụ thuộc thành một **vòng** — không tính
được cha trước khi tính con, và không tính được con trước khi tính cha. CSS gỡ
vòng ấy bằng cách lặng lẽ coi phần trăm là tự động, nghĩa là lời khai của người
viết bị vứt đi không một tiếng nào. TCC không làm thế được: 0.1 từ chối trường lạ
đúng vì lý do này, và một lời khai bị lặng lẽ vứt đi còn tệ hơn một lời khai gõ
sai tên, vì nó nhìn trong tệp thì vẫn đúng.

Luật này còn một tác dụng thứ hai đáng gọi tên. Loại được vòng rồi thì kích thước
tính xong trong **một lượt đi xuống** (kích thước xác định chảy từ khung xuống
lá) rồi **một lượt đi lên** (kích thước suy từ nội dung chảy từ lá lên khung).
Vậy bố cục **tuyến tính theo số nút** và bị chặn bởi giới hạn 10.000 nút sẵn có
của 0.1. Không cần thêm giới hạn cây nào, và không thêm cái nào. Bản cài đặt
không bắt buộc phải lặp tới điểm bất động, và một gói không ép nó lặp được.

⚠️ **Tính xác định là thuộc tính của LỜI KHAI, không phải của kết quả.** Một
nhóm mà nội dung tình cờ rỗng thì vẫn là suy từ nội dung. Bản cài đặt **KHÔNG
ĐƯỢC** quyết định tính xác định bằng cách đo — hai bản đo hai bộ chữ khác nhau
thì sẽ nhận những gói khác nhau.

## 4. Kích thước

Ba trường tuỳ chọn, cho phép trên **mọi** loại nút, cả lá lẫn nhóm: `size`,
`min`, `max`.

Mỗi trường là một đối tượng với hai khoá tuỳ chọn, `main` và `cross`, mỗi khoá
mang một **kích thước** (§4.1):

```json
"size": { "main": "half", "cross": "fill" },
"min":  { "main": "content" },
"max":  { "cross": "third" }
```

- Khoá lạ bên trong bất kỳ trường nào trong ba → `bad-json`.
- Đối tượng **không có** cả `main` lẫn `cross` → `bad-json`. Nó không khai gì
  cả, nên một trong hai cách đọc nó — "không có tác dụng" và "một giá trị mặc
  định nào đó" — là sai, mà người viết không biết là cách nào.
- Giá trị ngoài từ vựng kích thước, hoặc ngoài phần con mà trường ấy cho phép
  (§4.2) → `bad-json`. Từ vựng là tập đóng; lập luận 0.1 đưa ra cho `emphasis`
  áp dụng nguyên vẹn: bản cài đặt bỏ qua một từ kích thước nó không biết sẽ vẽ ra
  một màn hình khác với màn hình đã được ký.

### 4.1 Kích thước — từ vựng, và mỗi từ nghĩa chính xác là gì

Có **chín** từ kích thước. Đó là toàn bộ từ vựng; không có từ thứ mười, và không
từ nào là một con số.

| Từ | Kích thước |
|---|---|
| `content` | Đúng bằng thứ nội dung của chính nút cần trên trục ấy. Với một lá, là kích thước tự nhiên của thứ nó vẽ. Với một nhóm, là phần các con chiếm sau khi trải xong, cộng `padding` của chính nó ở cả hai mép của trục ấy. |
| `fill` | Một phần chia đều của **khoảng trống** của cha trên trục chính của cha (§4.4). |
| `full` | Toàn bộ kích thước **trong** của cha trên trục ấy. |
| `half` | Một nửa của nó. |
| `third` | Một phần ba. |
| `quarter` | Một phần tư. |
| `two_thirds` | Hai phần ba. |
| `three_quarters` | Ba phần tư. |
| `none` | **Không ràng buộc.** Chỉ cho phép trong `min` và `max` (§4.2). |

Sáu từ `full`, `half`, `third`, `quarter`, `two_thirds`, `three_quarters` là các
**phân số**.

**Một phân số được tính theo kích thước TRONG của nhóm cha trên trục ấy**, trong
đó *trong* nghĩa là kích thước của chính cha trên trục ấy **trừ `padding` của nó
ở cả hai mép của trục ấy** (§7).

Ba thứ mà một phân số dứt khoát **không** đo theo, mỗi thứ đều nói ra vì mỗi thứ
là một cách hiểu hợp lý sẽ khiến hai bản cài đặt lệch nhau:

- **không** phải kích thước ngoài của cha — `padding` bị trừ trước;
- **không** phải phần còn lại sau các anh em — `half` là một nửa của cha dù nó là
  con duy nhất hay con thứ năm;
- **không** bị `gap` trừ bớt — nên hai con `half` trong một nhóm có `gap` khác
  `none` **tràn ra khỏi cha, và như thế là đúng**. §9.1 nói phần tràn không bao
  giờ bị huỷ, nên người viết gây ra tràn thì nhìn thấy tràn. Lặng lẽ thu nhỏ hai
  con cho vừa nghĩa là cả hai lời khai đều không có hiệu lực trong khi trông như
  cả hai đều có.

⚠️ Từ **phân số** ở đây nghĩa là một tỉ lệ của kích thước cha. Nó không phải một
con số, không viết như một con số, và gói không có phép tính nào trên nó.
`two_thirds` và `three_quarters` là những từ đơn trong một tập đóng; chúng được
chọn vì một cột bên và một khung nội dung là hai hình dạng mà việc 0.1 thiếu bố
cục chặn lại nhiều nhất, và không vì lý do nào khác. Phiên bản nào cần phân số
thứ bảy thì thêm một **TỪ**, và thêm một từ là tăng phiên bản
([VERSIONING](../../VERSIONING.md) §3).

### 4.2 Mỗi trường cho phép những kích thước nào

| Trường | Kích thước cho phép | Mặc định |
|---|---|---|
| `size.main` | `content`, `fill`, phân số bất kỳ | `content` |
| `size.cross` | `content`, `fill`, phân số bất kỳ | vắng mặt — xem dưới |
| `min.main` | `none`, `content`, phân số bất kỳ | `content` |
| `min.cross` | `none`, `content`, phân số bất kỳ | `none` |
| `max.main` | `none`, `content`, phân số bất kỳ | `none` |
| `max.cross` | `none`, `content`, phân số bất kỳ | `none` |

Ngoài ra là `bad-json`: `fill` trong `min` hay `max` (một cực tiểu "phần chia của
khoảng trống" không phải cực tiểu của cái gì cả), và `none` trong `size` (một nút
không có kích thước thì không vẽ được).

Ba giá trị mặc định không hiển nhiên, và mỗi cái đều được nói ra có chủ ý:

- **`size.cross` KHÔNG có giá trị mặc định; mặc định nó VẮNG MẶT**, và vắng mặt
  không giống `content`. Vắng mặt chính là thứ §3 luật 4 và §6 kiểm tra: một
  kích thước phụ vắng mặt là điều kiện để `align_cross: "stretch"` kéo giãn nút
  ấy. Viết rõ `"cross": "content"` là **TẮT kéo giãn** cho nút ấy. Hai thứ là
  hai lời khai khác nhau cho ra kết quả khác nhau, và bản cài đặt gộp chúng lại
  sẽ vẽ ra một màn hình khác.
- **`min.main` mặc định là `content`.** Một nút không bị co nhỏ dưới mức nội dung
  của nó cần trên trục chính. Đây là cực tiểu tự động của Flexbox, và là thứ gây
  bất ngờ nhất trong Flexbox: nó là lý do một nhãn dài làm cả hàng tràn ra thay
  vì bị bóp lại. Nó được giữ vì lựa chọn còn lại là một cái nút bị cắt mất nửa
  chữ, mà luật chuỗi hiển thị của 0.1 sinh ra để chặn đúng việc chữ bị đổi giữa
  lúc ký và lúc hiện.
- **`min.cross` mặc định là `none`, không phải `content`.** Cực tiểu tự động chỉ
  áp cho trục chính. Hai bản cài đặt lệch nhau ở đây sẽ lệch trên mọi dòng gãy.

### 4.3 Kẹp giá trị

Mỗi trục được tính theo đúng thứ tự này:

1. Lấy kích thước từ `size` (hoặc giá trị mặc định của nó).
2. Kẹp xuống không quá `max` trên trục ấy.
3. Kẹp lên không dưới `min` trên trục ấy.

Vậy `min` thắng `max` ở chỗ hai bên chồng nhau. Nhưng chỗ chồng nhau ấy **không**
với tới được, vì:

**`min` lớn hơn `max` trên cùng một trục thì bị từ chối với mã `bad-layout`.**

So sánh theo đúng tỉ lệ mà các từ gọi tên, và phân số so được với phân số: `half`
lớn hơn `quarter` là vi phạm. `content` **không** so được với phân số — giá trị
của nó phụ thuộc nội dung — nên `min: {"main": "content"}` đi cùng
`max: {"main": "quarter"}` được **chấp nhận**, và bước 3 khi ấy có thể đẩy nút
vượt quá cực đại của chính nó. Chỗ bất đối xứng ấy là cố ý, và là lý do thứ tự
kẹp được viết ra ở trên chứ không để người đọc tự đoán.

**Vì sao từ chối thay vì tự gỡ:** một gói có `min` vượt `max` thì một trong hai
cái đang không làm gì cả. Luật của 0.1 về trường lạ là cùng một phán đoán — lời
khai không thể có hiệu lực thì bị từ chối chứ không được nuốt đi.

### 4.4 `fill`, và khoảng trống

`fill` chỉ áp trên **trục chính của cha**. Trên trục phụ, `fill` nghĩa là toàn bộ
kích thước phụ trong của cha (tương đương `full`); nó được phép ở đó và nghĩa
đúng như vậy.

Với một nhóm, trên trục chính của chính nó:

```text
khoảng trống = kích thước chính trong
             − tổng kích thước chính đã tính của các con
             − tổng các gap giữa chúng
khoảng trống bị kẹp ở không và không bao giờ âm
```

Mỗi con có `size.main` là `fill` nhận **một phần chia đều** của khoảng trống, rồi
phần chia ấy bị kẹp bởi `min` và `max` của chính con ấy (§4.3).

Các biên, đều nói rõ:

- **Một lượt duy nhất.** Nếu việc kẹp làm đổi một phần chia thì phần chênh
  **KHÔNG** được chia lại cho các con `fill` khác. Phần thừa cứ để trống. Flexbox
  chia lại bằng cách đóng băng các vi phạm rồi lặp; vòng lặp ấy là chỗ các bản
  cài đặt lệch nhau nhiều nhất, và kết quả của nó không kiểm được bằng một vector
  không biết kích thước nội dung.
- **Khoảng trống bằng không** — mọi con `fill` không nhận gì ngoài `min` của nó,
  mà mặc định `min` là `content`.
- **Không có trọng số.** Không có cách nào nói "gấp đôi". Các phân số đã diễn đạt
  được tỉ lệ, còn trọng số là một con số, mà §"Không có pixel" loại con số.
- `fill` trên con của một nhóm có kích thước chính trong suy từ nội dung là
  `bad-layout` (§3).

### 4.5 Không có co nhỏ

Nếu tổng kích thước đã tính của các con vượt kích thước trong của cha thì chúng
**KHÔNG** bị giảm. Chúng tràn (§9.1).

**Vì sao:** thuật toán co của Flexbox chia tỉ lệ theo kích thước gốc của từng
phần tử rồi chạy lại khi có vi phạm; hai bản cài đặt khớp nhau tới từng điểm ảnh
ở chỗ đó là việc đã được chứng minh là khó, mà lý do tồn tại của cả cây khai báo
trong TCC là thứ được ký và thứ được vẽ phải khớp nhau. Tràn ở đây an toàn theo
cách nó không an toàn trên web, vì §9 bảo đảm phần tràn luôn với tới được và
không bao giờ bị cắt lặng lẽ.

## 5. Căn theo trục chính

`align_main` chỉ có trên `group`. Tập đóng; ngoài ra là `bad-json`.

| Giá trị | Khoảng trống đi vào |
|---|---|
| `start` (mặc định) | sau con cuối cùng |
| `end` | trước con đầu tiên |
| `center` | một nửa trước con đầu, một nửa sau con cuối |
| `between` | chia đều vào các **khe giữa** các con, cộng thêm vào `gap`; không có gì trước con đầu hay sau con cuối |
| `evenly` | chia đều vào **mọi** khoảng: trước con đầu, giữa từng cặp, sau con cuối |

Các biên:

- **Khi khoảng trống bằng không hoặc âm, cả năm giá trị đều xử sự như `start`.**
  Cụ thể, nếu có bất kỳ con nào là `fill` thì sau §4.4 khoảng trống bằng không,
  nên `align_main` không có tác dụng nhìn thấy được. Đó không phải lỗi; nó được
  nói ra để người viết thấy không có gì xê dịch thì biết vì sao.
- **`between` với đúng một con xử sự như `start`** — không có khe nào để đặt
  khoảng trống vào. Với không con thì không có gì để đặt.
- **`evenly` với không con** để khoảng trống trống.
- Khoảng mà `align_main` chia là **cộng THÊM vào** `gap`, không bao giờ thay cho
  `gap`.
- Trong một nhóm có gãy dòng, `align_main` áp **trong từng dòng, riêng rẽ** (§8).

## 6. Căn theo trục phụ

`align_cross` chỉ có trên `group`. Tập đóng; ngoài ra là `bad-json`.

| Giá trị | Mỗi con được đặt |
|---|---|
| `stretch` (mặc định) | chiếm trọn kích thước phụ dành cho nó |
| `start` | ở mép đầu của trục phụ |
| `end` | ở mép cuối của trục phụ |
| `center` | ở giữa hai mép |

Các biên:

- **`stretch` chỉ áp cho những con KHÔNG khai `size.cross`** (§4.2). Con có kích
  thước phụ khai rõ thì giữ nguyên kích thước ấy và được đặt ở mép **đầu** của
  trục phụ.
- "Kích thước phụ dành cho nó" là kích thước phụ trong của nhóm khi `wrap` là
  `false`, và là kích thước phụ **của dòng** khi `wrap` là `true` (§8).
- **Không có cách ghi đè cho từng con.** Flexbox có; bản nháp này không, vì chưa
  màn hình tham chiếu nào cần tới, và luật 1 của
  [`spec/README.md`](../../README.md) cấm đặc tả trước mã. Thêm vào là tăng
  phiên bản.

## 7. Đệm trong

`padding` chỉ có trên `group`. Cùng tập đóng với `gap`: `none`, `small`,
`medium`, `large`. **Mặc định `none`.** Ngoài ra là `bad-json`.

`padding` áp cho **cả bốn mép như nhau**. Không có đệm riêng từng mép, vì đúng lý
do không có căn phụ riêng từng con: chưa có gì cần tới, và luật 1 cấm bịa ra ở
đây.

- `padding` nằm **bên trong** kích thước của chính nút: một nhóm kích thước E với
  đệm P có kích thước trong là E − 2P trên mỗi trục. Nhóm có kích thước `content`
  thì ngược lại: kích thước của nó là nội dung cộng 2P.
- Kích thước **trong** là thứ các phân số tính theo (§4.1), là thứ khoảng trống
  được tính từ đó (§4.4), và là chỗ các con được đặt vào.

### 7.1 Thang bậc phải thật sự là một thang bậc

**Độ lớn** của `none`/`small`/`medium`/`large` do bản cài đặt chọn — 0.1 cấm ứng
dụng đặt chúng. Nhưng hai yêu cầu làm cho những từ ấy có nghĩa:

- `none` **PHẢI** đúng bằng không.
- Bốn giá trị **PHẢI** tăng ngặt: `none` < `small` < `medium` < `large`.

**Vì sao:** 0.1 đã nói bản cài đặt "PHẢI vẽ mỗi ý định khác đi thật", vì nếu
không thì cả tầng ý định "chỉ là chú thích trong mã". Lập luận ấy áp y nguyên ở
đây mà 0.1 không nêu: không gì trong 0.1 chặn một bản cài đặt vẽ `gap: "large"`
và `gap: "small"` y hệt nhau, và khi đó từ vựng khoảng cách duy nhất của tiêu
chuẩn chẳng nói lên điều gì. **Vậy bản nháp này áp cùng hai yêu cầu ấy cho
`gap`**, tức là một yêu cầu MỚI đặt lên bản cài đặt của một trường đã có sẵn từ
0.1. Xem §11.2 — [VERSIONING](../../VERSIONING.md) §3 không có dòng nào cho loại
thay đổi này.

Đây là yêu cầu đặt lên **bộ dựng**, không đặt lên bộ kiểm. Không gói nào vi phạm
được, nên chúng không mang mã lỗi nào (§11.3).

### 7.2 Đệm và cuộn

Khi một nhóm cuộn được (§9), `padding` của nó thuộc về **phần nội dung được
cuộn**, không thuộc về cái khung quanh nó. Ở vị trí cuộn xa nhất, phần đệm ở
mép cuối **PHẢI** vẫn nhìn thấy được giữa con cuối cùng và mép của vật chứa.

**Vì sao:** đây là một lỗi các trình duyệt thật mang theo nhiều năm — nội dung
cuộn tới cuối thì dính sát mép vật chứa, nên người đọc không phân biệt được dòng
cuối cùng là dòng cuối cùng hay chỉ là dòng cuối cùng *được vẽ*. Với một màn hình
kết thúc bằng một hậu quả ("việc này sẽ xoá mọi khoá"), câu "bên dưới còn gì
nữa không?" không phải câu hỏi trang trí.

## 8. Gãy dòng

`wrap` chỉ có trên `group`. Là một giá trị luận lý JSON; **mặc định `false`**.
Thứ gì không phải luận lý — kể cả `"true"` — là `bad-json`.

Khi `wrap` là `true` và tổng kích thước chính của các con vượt kích thước chính
trong của nhóm, các con tiếp tục trên một **dòng** mới, dịch theo trục phụ.

| | |
|---|---|
| Thứ tự | Các dòng được lấp theo **thứ tự trong mảng**. Một con không bao giờ lùi về dòng trước, và không bao giờ vượt qua một anh em đứng sau. |
| Con quá lớn so với một dòng | chiếm **trọn một dòng** một mình và tràn ra khỏi dòng ấy (§9.1). Nó không bị co (§4.5) và không bị cắt. |
| `gap` | áp **cả** giữa các con trên một dòng **lẫn** giữa các dòng. Flexbox tách chuyện này thành hai thuộc tính; bản nháp này có một từ và nó chi phối cả hai. |
| `align_main` | áp **trong từng dòng, độc lập nhau**. Một dòng cuối ngắn được căn theo riêng nó. |
| `align_cross` | áp **trong từng dòng**: kích thước phụ của mỗi dòng là của con cao (hoặc rộng) nhất, và `stretch` kéo giãn tới **dòng**, không phải tới vật chứa. |
| Xếp dòng | Các dòng xếp từ mép **đầu** của trục phụ, có `gap` giữa chúng. **Không** có cách nào điều khiển việc chia phần trống còn lại theo trục phụ giữa các dòng — thuộc tính căn dòng của Flexbox không có tương đương ở đây. |
| `flow: "column"` + `wrap` | gãy thành **cột**: trục chính nằm dọc, nên một dòng mới là một cột mới, dịch theo chiều ngang. |
| Kích thước phụ bên trong | Con của một nhóm có gãy dòng thì trục phụ của nó **suy từ nội dung**, vì kích thước phụ của một dòng đến từ nội dung của dòng ấy. Vậy `fill` hay một phân số trên trục ấy là `bad-layout` (§3). |

### 8.1 Hai luật bộ dựng tham chiếu đã áp dụng

Luật 1 của [`spec/README.md`](../../README.md) là điều khoản được rút ra từ mã
**đã chạy**. Hai luật này đang chạy, trong bộ dựng tham chiếu của kho này. Chúng được ghi ở
đây vì người đọc cài đặt hết mọi mục trên mà bỏ hai mục này thì từ CÙNG một gói
sẽ dựng ra một màn hình **khác thấy được** — đúng thứ một tiêu chuẩn sinh ra để
chặn.

**Nhóm lồng trong một hàng chiếm trọn dòng của nó.** Một con kiểu `group` của
nhóm `flow: "row"` mở một dòng mới và chiếm trọn dòng ấy, dù nó cần ít tới đâu.

Đây là **mặc định để tương thích 0.1, và 0.2 thay thế nó.** Cây 0.1 không có vốn
từ kích thước nào cả, nên bộ dựng buộc phải quyết thay ứng dụng; §4 trao cho ứng
dụng chữ để tự nói. Bản cài đặt 0.2 **PHẢI** áp luật này cho nhóm KHÔNG khai
`size.main`, và **KHÔNG ĐƯỢC** áp cho nhóm CÓ khai `size.main` — khai một kích
thước chính là ứng dụng lấy lại quyền quyết.

**Nút đứng một mình trên một dòng được vẽ bằng nhau theo trục chính — nhưng chỉ
khi vẽ xong vẫn vừa.** Khi mọi con trên một dòng đã gom xong đều là `button`, và
có từ hai con trở lên, bản cài đặt **PHẢI** vẽ tất cả bằng kích thước của con
rộng nhất, trừ khi làm thế khiến dòng vượt quá vật chứa — khi ấy nó **PHẢI** vẽ
theo kích thước tự nhiên.

Luật này **không** phải mặc định tương thích và 0.2 không thay thế nó: đây là một
yêu cầu an ninh, và nó đúng bất kể ứng dụng khai kích thước nút thế nào.

> Màn xác nhận giao dịch cố ý cho hai nút CÙNG sắc thái, vì làm nút "Ký" nổi hơn
> là đẩy người dùng về một phía đúng lúc nguy hiểm nhất. Kích thước cũng đẩy. Một
> nút to hơn hẳn nút bên cạnh là đúng cái hích ấy, nói bằng hình học thay vì bằng
> màu — mà hình học không nằm trong vốn từ sắc thái, nên không gì khác ở đây chặn
> được nó.

Vế trừ không phải chuyện làm cho đẹp. Bộ dựng tham chiếu từng kéo bằng nhau vô
điều kiện, và ngày 21/08/2026 đo được một nút nằm ở 681,8→1008,7 trên ảnh rộng
640: **không một điểm ảnh nào của nó được vẽ**, mà phép thử va chạm vẫn trả về
nó. Người dùng bấm vào khoảng trắng và một nút họ chưa từng thấy chạy. Một hàng
nút không đều còn hơn một nút vô hình mà bấm được.

Cả hai luật được canh bởi vector hình học (§12) và không mang mã lỗi (§11.3):
không gói nào vi phạm được chúng.


## 9. Tràn và cuộn

### 9.1 Nội dung không bao giờ bị huỷ

Nội dung vượt quá vật chứa thì mặc định được **vẽ ra ngoài mép vật chứa** và
không bị cắt. Đó là mặc định tràn-nhìn-thấy của Flexbox, và ở đây nó là một yêu
cầu chứ không phải một mặc định: bản cài đặt **KHÔNG ĐƯỢC** cắt hộp của một nút
trừ chỗ §9.2 cho phép.

**Vì sao — đây là điều khoản bảo mật của tệp này.** Cắt là sửa một chuỗi người
dùng nhìn thấy sau khi nó đã được ký. Một nút `tone: "danger"` mang nhãn *"Xoá
mọi khoá trên máy này"* bị cắt ở mép vật chứa thì đọc thành *"Xoá mọi khoá"*, mà
0.1 dành hẳn một mục để cấm gói sửa thứ người dùng đọc ngay trước khi hành động.
Bản cài đặt cắt chữ đang làm với cái nhãn đúng cái việc 0.1 cấm gói làm.

⚠️ Đừng đọc câu này thành "tràn là chuyện không sao". Tràn *nhìn thấy được*, và
chính điều đó biến nó thành việc của người viết chứ không phải của người dùng.

### 9.2 Vật chứa cuộn

`scroll` chỉ có trên `group`. Là một giá trị luận lý JSON; **mặc định `false`**.
Thứ gì không phải luận lý là `bad-json`. `scroll` trên một nút lá là trường lạ
của loại nút ấy nên là `bad-json`, theo đúng 0.1.

Nhóm có `scroll: true` là một **vật chứa cuộn**. Nó **được** phép cắt nội dung
của nó, và đổi lại nó gánh một nghĩa vụ:

- **Trục cuộn** của nó là **trục chính** khi `wrap` là `false`, và là **trục
  phụ** khi `wrap` là `true`. (Gãy dòng loại bỏ tràn theo trục chính ngay từ cấu
  tạo và đẩy nó sang trục phụ, nên cuộn theo trục chính sẽ chẳng cuộn gì.)
- **Kích thước của nó trên trục cuộn PHẢI xác định** (§3). Nếu nó suy từ nội
  dung thì vật chứa lớn lên theo nội dung, không bao giờ tràn, nên không bao giờ
  cuộn: `bad-scroll`. Chú ý `size.main` mặc định là `content`, nên **khai
  `scroll` là buộc phải khai rõ một kích thước có chặn** trên trục ấy. Đó là hình
  dạng có chủ ý, không phải tai nạn của các giá trị mặc định.
- Bản cài đặt **PHẢI** đưa được **mọi** phần nội dung của nó vào hoàn toàn trong
  tầm nhìn. Vật chứa cuộn cắt đi phần nó không với tới được thì không tuân thủ.
- Vị trí cuộn **PHẢI** bắt đầu ở mép **đầu** của trục cuộn khi trải lần đầu.
  **Vì sao:** bắt đầu ở chỗ khác là giấu đi phần đầu của nội dung, mà phần đầu là
  chỗ đặt tiêu đề, lời cảnh báo, hay một nhấn mạnh `warning`.
- Chuyển tiêu điểm bàn phím tới một nút bên trong vật chứa cuộn **PHẢI** cuộn
  nút ấy vào tầm nhìn. **Vì sao:** nếu không thì tiêu điểm nằm trên một cái nút
  không có trên màn hình, và phím tiếp theo kích hoạt thứ người dùng không nhìn
  thấy.
- Nút bị cuộn khỏi tầm nhìn **vẫn nằm trong cây trợ năng**, vẫn có vai trò và
  nhãn của nó, và yêu cầu của 0.1 rằng mỗi nút phải sinh ra một nút trợ năng
  không đổi. Trượt không phải một cách gỡ bỏ một nút.

### 9.3 Vật chứa cuộn không lồng nhau trên cùng một trục

Một vật chứa cuộn **KHÔNG ĐƯỢC** có một vật chứa cuộn khác trong số tổ tiên của
nó với **cùng** trục cuộn. Vi phạm là `bad-scroll`.

Lồng nhau trên **hai trục khác nhau** thì được phép, và đó là hình dạng thường
thấy của một dải ngang nằm trong một trang dọc.

Quan hệ tổ tiên đếm trên **toàn cây**, không chỉ cha trực tiếp, và nhóm ngầm ở §1
cùng việc cuộn của chính khung (§1) **không** phải tổ tiên cho mục đích này —
cả hai đều không phải nút — nên một nhóm gốc có `scroll: true` là hợp lệ.

**Vì sao:** hai lý do, chỉ cần một cũng đủ.

1. §9.2 hứa mọi phần nội dung đều đưa vào tầm nhìn được. Có hai bộ cuộn trên
   cùng một trục thì việc một hộp cho trước có với tới được hay không phụ thuộc
   *vị trí* của cả hai, tức là một thuộc tính lúc chạy — nên lời hứa ấy thôi kiểm
   được, mà một lời hứa không kiểm được thì đúng bằng thứ
   [`spec/README.md`](../../README.md) gọi là điều khoản không có gì canh.
2. Vật chứa cuộn lồng nhau cùng trục là cách chuẩn để nuốt một cử chỉ cuộn: vật
   chứa bên trong ăn hết chuyển động, còn nội dung bên ngoài — kể cả thứ nằm dưới
   nếp gấp — thì người dùng không biết phải dời con trỏ trước sẽ không bao giờ
   tới được.

## 10. Bản nháp này KHÔNG có gì

Nói ra để không ai giả định ngược lại, theo đúng lối
[danh sách của 0.1](../../0.1/vi/README.md):

- **Không có độ dài, và không có con số nào** trong bố cục. Chín từ kích thước,
  hai giá trị luận lý, bốn từ khoảng cách.
- **Không có lưới.** `taffy` có cài đặt Grid; bản nháp này chỉ mô tả nửa Flexbox,
  vì chưa có gì được dựng trên Grid ở đây và luật 1 cấm đặc tả trước mã.
- **Không có co nhỏ** (§4.5) và **không có trọng số nở** (§4.4).
- **Không có căn phụ riêng từng con** (§6) và **không có căn dòng** (§8).
- **Không có đệm riêng từng mép** (§7), và không có lề ngoài nào cả. Khoảng giữa
  các anh em là `gap`; khoảng bên trong cha là `padding`. Cơ chế thứ ba là cách
  thứ ba diễn đạt cùng một thứ và là chỗ thứ ba để hai bản cài đặt lệch nhau.
- **Không có tỉ lệ khung hình**, nên một `image` lấy kích thước theo trục nào
  xác định, còn trục kia thì bản nháp này **không nói gì cả**. Đó là một lỗ hổng
  thật và nhiều khả năng là thứ được thêm vào đầu tiên.
- **Không có hình học phải-sang-trái** (§2).
- **Không có tràn trên chỉ một trục.** `scroll` là một giá trị luận lý duy nhất,
  và §9.2 suy ra trục. Vật chứa phải cuộn cả hai trục thì không diễn đạt được.

## 11. Mã lỗi

### 11.1 Mã đã có sẵn, dùng lại nguyên vẹn

| Mã | Dùng ở đây cho |
|---|---|
| `bad-json` | Mọi trường lạ, mọi khoá lạ bên trong `size`/`min`/`max`, mọi giá trị ngoài tập đóng, mọi `wrap`/`scroll` không phải luận lý, đối tượng `size`/`min`/`max` rỗng, và `scroll` hay `padding` đặt trên loại nút không có trường ấy |

`bad-json` có trong [bảng của 0.1](../../0.1/vi/06-ma-loi.md), và 0.1 đã dồn
"trường tiêu chuẩn này không định nghĩa" cùng "loại nút không có trong tiêu
chuẩn" về đó. Mọi vi phạm **hình dạng** trong tệp này thuộc đúng lớp ấy, và cho
nó một mã mới là xẻ một điều kiện thành hai mã mà chẳng được gì.

### 11.2 Hai mã MỚI

Không mã nào có trong bảng của 0.1. Không mã nào tồn tại trong bất kỳ bản cài đặt
nào. Cả hai chỉ là đề xuất của bản nháp này, không hơn:

| Mã | Khi nào |
|---|---|
| `bad-layout` | Một lời khai bố cục không thể có hiệu lực: `size`/`min`/`max` trên nút gốc (§1); `fill` hay một phân số trên trục mà kích thước của cha suy từ nội dung (§3); `min` lớn hơn `max` trên cùng một trục (§4.3) |
| `bad-scroll` | Một lời khai cuộn không thể có hiệu lực hoặc không kiểm được: vật chứa cuộn có kích thước trên trục cuộn suy từ nội dung (§9.2); vật chứa cuộn lồng trong một vật chứa cuộn khác cùng trục (§9.3) |

Cả hai theo đúng lối đặt tên của 0.1: tiền tố `bad-` cho "giá trị của trường này
vi phạm ràng buộc của nó" (`bad-path`, `bad-app-id`, `bad-scope`, `bad-entry`,
`bad-action-id`). Cả hai là điều kiện **liên trường** — không quyết được khi chỉ
nhìn một trường — nên chúng không phải `bad-json`; 0.1 đã tạo tiền lệ ấy với
`action-host-not-granted`, một điều kiện liên trường có mã riêng, kèm lý do rằng
một mã chung chung "chẳng nói lên điều gì và bộ kiểm định không khớp vào đâu
được".

⚠️ **Thêm hai mã này chính là chỗ bản nháp làm luật 22 của CI báo đỏ**: luật ấy
đòi mọi token kiểu gạch nối trong dấu nháy ngược ở bất cứ đâu dưới `spec/` phải
có trong `spec/0.1/06-error-codes.md`. Luật ấy đọc bảng của **0.1** cho tệp của
**mọi** phiên bản, nên nó cấm mọi phiên bản tương lai thêm bất kỳ mã lỗi nào.
Xung đột này được ghi lại trong [README](README.md) chứ không né bằng cách đổi
tên hai mã, vì một mã đặt tên sao cho lọt qua một phép kiểm là một phép kiểm đã
thôi hoạt động.

### 11.3 Những điều khoản không mang mã lỗi, và vì sao đó không phải thiếu sót

Vài yêu cầu ở đây ràng buộc **bản cài đặt**, không ràng buộc gói: §7.1 (thang bậc
khoảng cách phải tăng), §9.1 (không cắt), §9.2 (với tới được, vị trí cuộn ban
đầu, tiêu điểm), §1 (khung cuộn được), §8.1 (nhóm lồng chiếm trọn dòng; nút đứng
một mình trên một dòng vẽ bằng nhau). Không gói nào vi phạm được, nên không có gì
để từ chối và không có mã nào để báo.

0.1 cũng có đúng hình dạng ấy — "PHẢI vẽ mỗi ý định khác đi thật" cũng không có
mã — và [`spec/README.md`](../../README.md) gọi đích danh loại này là kiểu lỗi
đầu tiên mà đợt soát tìm ra: *một điều khoản không có gì canh*. Những điều khoản
này được canh bởi **vector hình học** ở §12, và đó là lý do duy nhất bản nháp này
được phép nêu chúng.

### 11.4 Thứ tự ưu tiên

Các phép kiểm bố cục thuộc bước **10** của bảng ưu tiên trong
[06 của 0.1](../../0.1/vi/06-ma-loi.md) — "Tệp giao diện, quyền năng, hành vi".
Bên trong tệp giao diện, chúng chạy theo thứ tự này, và bản cài đặt **PHẢI** báo
mã của phép kiểm hỏng đầu tiên:

| # | Phép kiểm | Mã |
|---|---|---|
| 1 | Hình dạng: trường, khoá, tập đóng, giá trị luận lý | `bad-json` |
| 2 | Chuỗi hiển thị và các giới hạn cây của 0.1 | `unsafe-display-string`, `text-too-long`, `too-deep`, `too-many-nodes` |
| 3 | Tính xác định và việc kẹp giá trị (§3, §4.3) | `bad-layout` |
| 4 | Trượt (§9.2, §9.3) | `bad-scroll` |

Hình dạng đi trước mọi thứ vì đúng lý do 0.1 đã nêu: chưa biết cây là một cây thì
không quyết được gì khác. Tính xác định đi trước cuộn vì `bad-scroll` cho trường
hợp trục cuộn suy từ nội dung được phát biểu **bằng chính** khái niệm xác định,
nên báo nó trước khi xác lập tính xác định là báo một kết luận rút ra từ một tiền
đề chưa kiểm.

## 12. Vector kiểm định

Mọi điều khoản ở trên đều có ít nhất một vector trong
[`conformance/vectors/layout.json`](../../../conformance/vectors/layout.json),
theo luật 2 của [`spec/README.md`](../../README.md). Mỗi vector ghi tên mục nó
kiểm.

Tệp ấy chứa **hai** loại trường hợp, và loại thứ hai là mới:

- **`cases`** — nhận/từ chối, theo đúng hình dạng
  [`ui.json`](../../../conformance/vectors/ui.json) đang dùng: `tree`,
  `expect_pass`, và `code` ở trường hợp bị từ chối. Bộ chạy sẵn có nào cũng đọc
  được.
- **`geometry`** — các khẳng định về **hộp** mà bố cục sinh ra. Loại này là mới,
  vì hình dạng sẵn có không diễn đạt nổi một yêu cầu bố cục nào cả: nó chỉ nói
  được nhận hay từ chối, mà "hai con `fill` bằng nhau" thì không phải cái nào
  trong hai.

Trường hợp hình học khẳng định **quan hệ giữa các hộp**, không bao giờ khẳng định
hình thức tuyệt đối, nên chúng đúng với mọi độ lớn mà một bản cài đặt chọn cho
`gap` và `padding` — thứ 0.1 để cho bản cài đặt quyết và bản nháp này không đòi
lại. Mô tả từng khoá của hình dạng hình học nằm trong chính đối tượng `format`
của tệp ấy; nó viết ở đó chứ không ở đây vì
[`conformance/FORMAT.md`](../../../conformance/FORMAT.md) thuộc về cả kho, mà bản
nháp này không sở hữu phần nào của nó.

⚠️ Vector hình học **chưa được nối vào bộ chạy của kho này** và chưa nối được, vì
chưa có bản cài đặt nào tính bố cục. Người đọc chạy được chúng, còn ở đây chúng
nằm im. Đó là trạng thái trung thực của một bản nháp viết trước mã của nó, và là
lý do tệp này không được viện làm yêu cầu tuân thủ.
