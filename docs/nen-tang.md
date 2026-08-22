# TCC Modern Baseline — nền tảng web công bố được

> Mục 5.1 của [`ke-hoach.md`](ke-hoach.md). Cập nhật **18/08/2026**.
>
> ⚠️ **Đây KHÔNG phải tầng 2.** Tầng 2 — mở trang web ngoài đời — vẫn **0 dòng
> mã**. Tài liệu này trả lời câu đứng trước nó: *nếu làm tầng 2 thì đứng được
> trên cái gì?*

## Nguyên tắc: ĐO, không liệt kê

Một danh sách viết tay là một **lời hứa**. Nó đúng vào ngày viết, trôi ngay hôm
sau, và không ai biết nó đã trôi.

Nên nền tảng ở đây là thứ **đo được**:

```bash
cargo run -p tcc-shell --features window --example do-nen-tang
```

Nó nạp một tài liệu vào bộ máy THẬT, hỏi nó có gì, và in ra bảng. Chạy trong CI
trên cả ba nền.

## Nền tảng công bố được là PHẦN GIAO, không phải phần hợp

Ba nền chạy **ba bộ máy khác nhau**:

| Nền | Bộ máy |
|---|---|
| macOS | WKWebView |
| Linux | WebKitGTK |
| Windows | WebView2 (Chromium) |

Một tính năng chỉ có ở hai trong ba nền thì **không vào nền tảng**. Công bố phần
hợp là hứa một thứ mà một phần ba người dùng không có.

## Phát hiện đầu tiên, và nó đổi cách nghĩ

Đo trên macOS (18/08/2026): **18/20 mục có mặt**. Hai mục vắng:

| Vắng | Vì sao |
|---|---|
| `crypto.subtle` | Cần **ngữ cảnh an toàn** (secure context) |
| `localStorage` | Cần **nguồn gốc** (origin) thật |

Cả hai **không phải do bộ máy thiếu**. Chúng vắng vì tài liệu được nạp qua
`with_html`, tức là chạy trong một **nguồn gốc mờ** — không `https://`, không
tên miền, không origin.

> **Nền tảng phụ thuộc vào CÁCH NẠP nội dung, không chỉ vào bộ máy.**

Hệ quả cho hai tầng, và chúng ngược nhau:

- **Ứng dụng TCC (tầng 1)**: đây là **tin tốt**. Ứng dụng không mang mã, và
  nguồn gốc mờ khiến `localStorage` cùng `crypto.subtle` **không tồn tại** để mà
  phải chặn. Một lớp phòng thủ có sẵn mà không phải viết dòng nào.
- **Tầng 2**: đây là **rào chắn**. Trang web thật cần origin thật, nên tầng 2
  không thể dùng chung cách nạp với tầng 1. Nó cần một giao thức riêng và một
  mô hình nguồn gốc riêng — và đó là thiết kế, không phải cấu hình.

## Bảng đo

Chạy `do-nen-tang` để có bảng mới nhất. Các mục được chọn theo hai tiêu chí:
**thật sự khác nhau giữa các bộ máy**, hoặc **thật sự cần cho tiếng Việt**.

Nhóm tiếng Việt đứng đầu danh sách là có chủ ý — `normalize`, `Intl.Collator`,
`Intl.Segmenter`, `font-variation-settings`. Chúng là thứ quyết định chữ hiện ra
đúng hay sai, và là thứ ít ai kiểm.

Ba mục cuối (`localStorage`, `Notification`, `navigator.geolocation`) **không
phải thứ ta muốn có**. Chúng nằm trong bảng để biết **phải tắt cái gì** — một
tính năng có mặt mà ta quên tắt là một tính năng người dùng bị lộ.

## Đo được ở đâu, và chưa đo được ở đâu

| Nền | Bộ máy | Tình trạng |
|---|---|---|
| macOS | WKWebView | ✅ đo được, **18/20** |
| Linux | WebKitGTK | ✅ đo được (19/08/2026), **18/20 — thiếu ĐÚNG hai mục ấy** |
| Windows | WebView2 | ✅ đo được, **18/20 — thiếu ĐÚNG hai mục ấy** |

**Cả BA bộ máy — WKWebView, WebKitGTK, WebView2 (Chromium) — thiếu đúng cùng
hai mục: `crypto.subtle` và `localStorage`.** Đó không phải trùng hợp: nó xác
nhận hai mục ấy vắng vì **cách nạp tài liệu**, không vì bộ máy. `with_html` cho
tài liệu một nguồn gốc mờ, mà cả hai thứ ấy đều đòi một nguồn gốc thật.

### Vì sao Linux mãi mới đo được, và bài học đắt hơn con số

Bước đo trên Linux mang `continue-on-error` suốt nhiều tháng, kèm chú thích nói
rằng WebKitGTK dưới màn hình ảo không ổn định. **Chú thích ấy sai.**

Câu lỗi là `the underlying handle is not available` — không nhắc một chữ nào về
GTK, nên nó đọc y hệt một màn hình ảo chưa kịp lên. Nhưng trên Linux `wry` gắn
`WebView` vào một widget GTK chứ không vào cửa sổ: `build(&window)` đơn giản là
**gọi sai hàm**.

Cách phát hiện đáng ghi lại hơn cả bản vá: thay vì chạy một lượt rồi đoán, chạy
**cùng một tệp nhị phân ba lượt trong cùng một job**. Kết quả **0/3** — không
phải chập chờn, mà **trượt đều**. Sau khi gọi đúng `build_gtk`: **3/3**.

Chính chỗ này từng dẫm bẫy ngược lại: một lượt xanh, gỡ cờ, lượt sau đỏ, rồi kết
luận "hạ tầng chập chờn". **Một lượt xanh không phân biệt được "đã sửa" với
"vừa may"** — và một phép thử được miễn vì *"hạ tầng không đáng tin"* là chỗ tốt
nhất để một lỗi thật nằm im.

Bốn cờ `continue-on-error` trên Linux đã bỏ hết. Ba phép thử đối kháng trên
WebKitGTK cũng đạt: chữ của ứng dụng không thoát ra khỏi tài liệu, chính sách
nội dung một mình chặn được kịch bản, và màn hình ứng dụng lên đúng từ cây khai
báo.

**Đủ ba số đo từ 19/08/2026** — phần giao là **18/20**, giống nhau trên cả ba
bộ máy. Bảng nền tảng công bố được. Câu dưới đây giữ lại nguyên văn vì nó là
điều kiện đã đặt ra trước khi biết kết quả:

> **Chưa công bố bảng nền tảng nào**, vì phần giao cần đủ ba số đo. Công bố khi
> mới có một là công bố một bảng của macOS và gọi nó là nền tảng.

## Nạp thế nào để có ngữ cảnh an toàn

Khảo sát mã nguồn `wry` 0.52.1 (18/08/2026):

| Cách nạp | Origin | Ngữ cảnh an toàn |
|---|---|---|
| `with_html` — **đang dùng** | `null`, mờ | **Không** trên cả ba (đã đo) |
| custom protocol + `with_url` | `tcc-goi://goi` (Win: `http://tcc-goi.localhost`) | Linux **có** — WebKitGTK gọi `register_uri_scheme_as_secure`; macOS **nhiều khả năng không** — WKWebView không có lời gọi tương đương; Windows chưa đo |
| `with_url("https://…")` | origin thật của site | **Có**, đồng nhất cả ba |

Hai điều rút ra:

**`with_url` thắng `with_html`.** Cả ba bản cài đặt của `wry` đều kiểm `url`
trước; không thể vừa `with_html` vừa mong có origin.

**Custom protocol dồn MỌI trang vào MỘT origin.** Nếu tầng 2 đi đường ấy thì
mọi trang đọc được `localStorage`, `IndexedDB`, cookie **của nhau** — và không
CSP nào vá được. Nên tầng 2 phải nạp `https://` thật; custom protocol giữ đúng
vai trò cũ: phục vụ tệp trong gói đã ký.

⚠️ Ba điều **chưa xác minh**, cần thí nghiệm thật chứ không đọc mã:
`isSecureContext` dưới `tcc-goi://` trên WKWebView; `localStorage` dưới custom
scheme; và cách phân vùng dữ liệu giữa nhiều WebView ngoài Darwin.

## Chưa đo

- **Hiệu năng.** Máy phát triển là Intel Mac, Iris Plus 645 — kế hoạch đã dặn
  đừng kết luận đồ hoạ trên máy này.
- **Khác biệt hành vi.** Có `CSS grid` không có nghĩa ba bộ máy vẽ ra cùng một
  thứ. Bộ 50 trang thật + so ảnh chụp (mục 5.2) mới trả lời được, và nó chưa làm.


## Tầng 2 chặn gì, và không chặn được gì

Khảo sát mã `wry` 0.52.1 (18/08/2026). Trang web mang mã của nó: không chữ ký,
không cổng quyền năng, không ai kiểm trước.

| Đòn | Chặn thế nào |
|---|---|
| Trang chạm `window.ipc` của khung | WebView **riêng**, không IPC, không kịch bản |
| Nhảy sang `file://`, `javascript:`, giao thức riêng của ta | Kiểm **mỗi lần** điều hướng, không chỉ lần nạp đầu |
| `window.open` / `target=_blank` | **Từ chối** — khung ấy ta không dựng nên không có chắn nào ở đó |
| Tải tệp | **Từ chối** — chưa hỗ trợ, thà nói "chưa làm" còn hơn ghi ra đĩa một tệp mà tên và đuôi do trang chọn |
| Đọc trộm bảng nháp | `with_clipboard(false)` |
| Tự phát tiếng | `with_autoplay(false)` — mặc định của wry là **bật** |
| Ghi cookie/đăng nhập ra đĩa | **Không giữ gì** — `with_incognito(true)` trên cả ba máy |
| `http://` | Từ chối. Chặt hơn `external_link` một bậc: đường trần thì ai cũng sửa được, mà ta đặt nó trong cửa sổ mang tên TCC |

### ❌ Không chặn được: micro và camera trên macOS

`wry` viết cứng `WKPermissionDecision::Grant`
(`wkwebview/class/wry_web_view_ui_delegate.rs:74`), không cho ghi đè. Chắn duy
nhất là **tầng hệ điều hành** — gói không khai `NS*UsageDescription` thì macOS
từ chối. **Luật 20** canh điều đó. Xem `SECURITY.md` §3.

### Chưa chặn, và phải nói ra

- **Hộp thoại `alert`/`confirm`/`prompt`** của trang: `wry` không cho móc vào.
- **Theo dõi xuyên trang trong một phiên.** Câu tôi viết ở bản trước — *"cookie
  và `localStorage` dùng chung giữa mọi trang"* — **nói quá**. Máy dựng đã tách
  kho theo nguồn gốc: trang A không đọc được `localStorage` của trang B. Thứ
  thật sự dùng chung là **một hồ sơ**, nên một bên thứ ba nhúng ở cả hai trang
  vẫn nhận ra cùng một người — đúng như mọi trình duyệt dùng chung một hồ sơ.
  Sửa lại cho đúng: đây là chuyện theo dõi, không phải chuyện rò kho.

  Đã chọn cách xử: **không giữ gì trên đĩa** (`with_incognito(true)`). Đóng cửa
  sổ là mất cookie và mất đăng nhập. Lý do chọn thế thay vì "giữ rồi cho xoá":
  `wry` chỉ có `clear_all_browsing_data` — xoá sạch tất cả, không xoá theo tên
  miền — và chưa có màn hình hồ sơ nào để người dùng nhìn thấy thứ đang được
  giữ. Thanh địa chỉ **nói thẳng câu đó ra**, không để người dùng tự phát hiện.
- **Cảnh báo chứng chỉ TLS**: không có móc.


## Bộ 50 trang thật — kết quả đo 18/08/2026

Kế hoạch mục 5.2 viết *"bộ 50 trang thật, so ảnh chụp hằng tuần"*. Phần **so ảnh
chụp** không làm, và đây là lý do.

Tầng 2 dùng máy dựng của hệ điều hành. So từng điểm ảnh ở đây là **đo WebKit của
Apple**, không đo mã của ta — nó sẽ đỏ mỗi lần macOS cập nhật, vì một nguyên nhân
ta không sửa được và cũng không nên sửa. Một phép đo đỏ vì lý do ngoài tầm là
một phép đo người ta học cách bỏ qua.

Thứ **là của ta** ở tầng 2 là chính sách: chỉ `https`, từ chối cửa sổ mới, từ
chối tải tệp, không giữ gì trên đĩa. Nên bộ trang này đo **giá của chính sách ấy
trên trang thật**.

Chạy: `tcc-browser corpus corpus/50-trang.txt 5` — một máy, một lần, macOS 15.6,
21:57–22:02 ngày 18/08/2026. Không có ai bấm gì trong suốt lượt chạy.

| Đo | Số |
|---|---|
| Trang | 50 |
| Nạp xong trong 5 giây | 40 |
| Nạp xong khi cho 15 giây | **45** |
| **Điều hướng bị chặn** | **0** |
| **Cửa sổ mới bị từ chối** | **148** |
| **Tải tệp bị từ chối** | **0** |

### Chỉ `https` không tốn gì cả

**Không một trang nào** trong 50 trang bị chắn điều hướng nổ. Luật chặt nhất của
tầng 2 — chặt hơn tầng 3 một bậc — không cản trở gì trên tập này.

### 148 lần từ chối cửa sổ mới, và chúng dồn vào đâu

Trung bình gần **ba lần mỗi trang**, mà **không ai bấm gì cả** — nên đó là trang
tự gọi, không phải người dùng mở liên kết.

Phân bố mới là điều đáng đọc:

| Nhóm | Từ chối |
|---|---|
| Tài liệu và công cụ (MDN, Rust Book, docs.rs, RFC, arXiv, GitHub, crates.io, PyPI…) | **0** |
| Tin tức và thương mại | 24 (Al Jazeera) · 20 (VnExpress) · 15 (Dân Trí) · 14 (Shopee) · 13 (AP) |

Một chính sách nổ ba lần mỗi trang thường là chính sách sai. Nhưng ở đây nó nổ
**đúng không lần nào** trên mọi trang tài liệu, và dồn hết vào trang nhiều quảng
cáo. Đó là lập luận **ủng hộ** luật, không phải chống.

Phải nói rõ giới hạn: bộ đếm biết một cửa sổ **bị từ chối**, không biết người
dùng có **mất gì** không. Muốn biết điều đó phải có người ngồi bấm.

### 5 trang không bao giờ báo "nạp xong"

BBC News, AP News, `mic.gov.vn`, `sbv.gov.vn`, `vnpt.com.vn` — kể cả với 15 giây.

**Không kết luận là hỏng.** "Nạp xong" là sự kiện của WebKit; một trang còn một
tài nguyên treo (quảng cáo, đo đạc, kết nối chờ dài) có thể không bao giờ phát
sự kiện ấy trong khi vẫn đọc được bình thường. Ta **không hỏi trang** được —
WebView của trang không có IPC và không có kịch bản, và đo đạc không phải lý do
đủ để gỡ chắn lớn nhất của tầng 2.

### Một trang đã liệt kê thiết bị thu hình

Trong lượt chạy, macOS ghi một cảnh báo về `AVCaptureDeviceTypeExternal`. Nghĩa
là **danh sách thiết bị thu hình đã bị liệt kê** — nhiều khả năng qua
`enumerateDevices()`.

Liệt kê **không phải** là bật camera, và không có đèn camera nào sáng. Nhưng nó
là bằng chứng thật, không phải giả định, rằng trang chạm tới được vùng ấy —
đúng vùng `wry` viết cứng `WKPermissionDecision::Grant`. Chắn duy nhất vẫn là
**luật 20**: gói không khai `NS*UsageDescription` thì macOS không có gì để cấp.
Xem `SECURITY.md` §3.

### Lượt chạy thật bắt được một lỗi

Lượt đầu chạy hết 50 trang rồi **hoảng loạn ở nhịp cuối** và mất sạch kết quả:
`ControlFlow::Exit` không phải thoát ngay, `tao` còn giao thêm vài sự kiện, và
nhịp thừa ấy tra chỉ số đã vượt cuối danh sách. Không lượt chạy thật thì không
thấy — cùng bài học với `TX_HEX`: **một mốc tự dựng thì không phải mốc.**


## Mục 5.3 — vì sao KHÔNG có nhãn "TCC Ready"

Kế hoạch viết *"nhãn TCC Ready cho trang đạt chuẩn"*. Nhãn ấy không xuất xưởng,
và lý do là cùng một lý do trình duyệt này không bao giờ hiện "đã xác minh nhà
phát hành".

**Một nhãn nghe như bảo chứng mà chỉ kiểm được một việc hẹp thì người đọc tin
vào phần nó không kiểm.** "TCC Ready" nghe như *"trang này an toàn và chạy
đúng"*. Thứ ta đo được chỉ là: trong lượt nạp, các chắn của ta nổ mấy lần.

Và bộ đếm **không tách được hai chuyện khác hẳn nhau**:

| Trang | Bộ đếm thấy | Sự thật |
|---|---|---|
| Trang tin có quảng cáo tự bật cửa sổ | 20 lần từ chối | Nội dung đọc trọn vẹn, không mất gì |
| Trang chỉ dùng được khi mở được cửa sổ mới | 1 lần từ chối | Hỏng thật |

Gắn nhãn "trượt" cho trang đầu là **đổ lỗi cho trang vì quảng cáo của nó**, mà
người đọc trang ấy không mất gì cả. Muốn tách hai cột đó phải có **người ngồi
bấm**, không có con số nào thay được.

### Thứ ĐO ĐƯỢC, gọi đúng tên nó

Bộ trang chạy khi **không ai bấm gì**. Nên mọi lần chắn nổ đều là **trang tự
đòi**, không phải người dùng yêu cầu. Tính chất ấy có thật và đo lại được, và
tên đúng của nó là **"im khi nạp"** — không phải "đạt chuẩn".

Lượt đo 18/08/2026, cùng bộ 50 trang: **26/50 im khi nạp.**

Bảng dưới cột "im" in ra ngay cạnh câu nói rõ nó không phải nhãn chất lượng và
không phải nhãn an toàn. Câu ấy nằm **cạnh con số**, không nằm trong tài liệu:
ai đọc một con số trần cũng sẽ gán cho nó nghĩa rộng hơn nghĩa nó có.

### Hai lượt chạy, hai con số — và đó là dữ liệu, không phải lỗi

| | Lượt 1 (21:57) | Lượt 2 (22:2x) |
|---|---|---|
| Nạp xong trong 5 giây | 40 | 34 |
| Cửa sổ mới bị từ chối | 148 | 67 |

Chênh **hơn gấp đôi** ở cùng một bộ trang, cùng một máy, cách nhau vài chục
phút. Vì trang tin thay quảng cáo mỗi lượt tải, và mạng mỗi lúc một khác.

Ghi ra chứ không chọn con số đẹp hơn: **một phép đo dao động gấp đôi giữa hai
lượt là phép đo không được phép công bố dưới dạng một con số.** Nó dùng để so
xu hướng qua nhiều lượt, không dùng để tuyên bố "trình duyệt chặn 148 cửa sổ".


## Rà soát đối kháng 21/08/2026 — bốn lỗ ở tầng 2

Một lượt soát nhắm riêng tầng 2. Bốn phát hiện, tôi **kiểm tay từng cái** trước
khi tin, và cả bốn đều đúng.

### 1. Thanh địa chỉ KHÔNG BAO GIỜ cập nhật — nó nói dối được

Đây là lỗ đúng vào thứ cả tầng được đặt tên theo. `open_browser` vẽ tài liệu
khung **một lần**, từ tham số dòng lệnh, và chỗ duy nhất ghi lại vào ô nhập là
nhánh **lỗi**. Không có `on_page_load_handler` nào trên WebView của trang.

Nên: người dùng mở `vnexpress.net`; một mẩu quảng cáo chạy
`location.href = "https://vnexpress-net.evil.example/dang-nhap"`; chắn điều
hướng **cho qua** vì đó vẫn là `https`; trang lừa đảo hiện ra **dưới một thanh
địa chỉ vẫn ghi `vnexpress.net`**.

Tệ hơn không có thanh địa chỉ nào — vì thanh này được dựng **để tin**: có phép
thử chốt rằng nó hiện địa chỉ ĐỦ, không bao giờ cắt ngắn.

Đã vá: máy dựng báo địa chỉ **thật** sau mỗi lần nạp xong, và thanh hiện đúng
chỗ đó. Đây là địa chỉ của máy dựng, không phải chuỗi người dùng gõ.

### 2. `@` trong tên máy đi lọt

`vnexpress.net@evil.example` → tải **evil.example**. Phép kiểm cổng cũ tình cờ
chặn dạng `user:pass@host` (phần sau `:` không toàn chữ số) mà **để lọt** dạng
`user@host`.

Đã vá ở `check_web_url`, nên **cả hai** đường — gõ tay và địa chỉ mở đầu — cùng
chặn. `https://a.com/@ai` vẫn mở được: `@` trong **đường dẫn** là địa chỉ bình
thường.

### 3. `://` trong truy vấn bị chặn nhầm

`example.com/r?u=https://x` bị từ chối, vì phép kiểm xét `contains("://")` trên
cả chuỗi. **Một thanh địa chỉ từ chối địa chỉ hợp lệ là một thanh địa chỉ người
dùng học cách vòng qua** — đó là lý do đây là lỗi an ninh, không phải lỗi tiện
dụng. Giờ xét tiền tố.

### 4. Phép thử canh cả tệp không bắt được đòn tệ nhất

`khong_co_ipc_va_kich_ban` chứng minh *khúc đã đánh dấu thì sạch*. Nó **không**
chứng minh *mọi WebView nạp trang ngoài đều được đánh dấu*, và nó để lọt đột
biến nguy hiểm nhất: đổi WebView của **khung** — nơi có IPC và kịch bản — từ
`with_html` sang `with_url`. Một trang web bất kỳ chạy trong WebView có
`window.ipc`, đúng thứ cả tệp sinh ra để chặn, mà phép thử vẫn xanh.

Phép thử mới khẳng định điều **ngược lại**, đếm trên toàn tệp: mọi `with_url`
phải nằm trong dấu mốc, và mọi `with_ipc_handler` phải đi cùng `with_html`. Cả
hai đột biến đều chết.

### Còn lại từ báo cáo, chưa xử — ghi ra để không quên

- **Micro/camera có thể được cấp qua tiến trình cha.** macOS quy trách nhiệm cho
  *responsible process*; chạy `tcc-browser` từ terminal đã được cấp camera thì
  `getUserMedia()` của trang có thể qua, bất kể gói ta không khai
  `NS*UsageDescription`. **Cần đo thật**, chưa đo.
- **Luật 20 quét hẹp hơn lời nó nói**: chỉ soi `<key>NS…UsageDescription</key>`
  trong `tools/` và `apps/`. Bỏ sót plist trong `crates/`, khoá
  `[package.metadata.bundle]` trong `Cargo.toml`, và cách nhúng plist thẳng vào
  binary bằng `#[link_section]`.
- **`with_clipboard(false)` là lệnh rỗng trên macOS** — `wry` chỉ cài nó cho
  Linux và Windows. Bảng "chặn gì" đang kể công một thứ không tồn tại ở đây.
- **148 "cửa sổ mới bị từ chối"** nhiều khả năng phần lớn là **iframe bị chặn**,
  không phải `window.open`: `wry` định tuyến mọi điều hướng khung con vào
  `new_window_req_handler`. Nếu đúng thì tầng 2 đang **chặn mọi iframe** — một
  hỏng hóc chức năng mà báo cáo cũ đọc thành một thắng lợi.
- **Chắn điều hướng chưa từng được thấy nổ**: 0/50 trang. Phép thử đơn vị kiểm
  *vị từ*, không kiểm *dây nối*.
- Chi tiết còn lại: bộ đếm chưa có trần độ dài, WebView khung không đi qua
  `chan_lai` và không bật chế độ không giữ gì, `doc_dia_chi` không so mã hành
  động.


## Rà soát đối kháng 21/08/2026 — bộ dựng raster: bốn lỗ

Lượt soát thứ hai nhắm bộ dựng ra pixel và đường trợ năng. Nó **chạy mã thật**
để chứng minh phát hiện đầu, và tôi dựng lại được y hệt.

### F1 — bấm được vào một nút KHÔNG có điểm ảnh nào trên màn hình

Luật *"nút cùng hàng rộng bằng nhau"* kéo mọi ô lên bằng ô rộng nhất — nhưng
quyết định xuống dòng đã tính **trước** khi kéo. Một hàng "vừa" bị nới ra quá lề,
và các ô sau trôi hẳn khỏi ảnh. Đo được:

```
ô 2: trái=681,8  rộng=326,9  phải=1008,7     ← ảnh rộng 640
hit_test(700, 17)  = Some("ok2")
hit_test(1000, 17) = Some("ok2")
```

Người dùng kéo rộng cửa sổ, bấm vào khoảng trắng bên phải, và **một nút họ chưa
từng nhìn thấy chạy**.

Phép thử cũ **về mặt cấu trúc không thể bắt được**: `ve_o` cắt phần vẽ ở
`WIDTH - trai - 2`, nên phép kiểm "chạm mép phải" không bao giờ đỏ được; và bộ
sinh cây ngẫu nhiên dùng **cùng một nhãn** cho mọi nút, nên không bao giờ tạo ra
hàng có bề rộng lệch — đúng hình dạng duy nhất kích hoạt lỗi.

Vá ở gốc: chỉ kéo bằng nhau **khi kéo xong vẫn vừa**. Không vừa thì để bề rộng
tự nhiên — một hàng nút không đều đẹp hơn một nút vô hình bấm được.

### F2 — VoiceOver bật muộn đọc ảnh chụp ĐẦU TIÊN, vĩnh viễn

`update_if_active` trả `None` **và không gọi hàm dựng cây** khi chưa có ai nghe.
Nên mọi lần vẽ lại lúc VoiceOver còn tắt đều bị vứt, còn `request_initial_tree`
— gọi đúng một lần khi có người nghe — trả về cây của **lần vẽ số 0**.

Trên màn hỏi quyền: gạt một quyền bằng chuột, rồi bật VoiceOver, và nghe **"tắt"**
trong khi màn hình hiện **"bật"**. Đúng thứ chú thích trong mã nói là nó đang
chặn — chắn ấy chỉ chạy khi adapter **đã** hoạt động sẵn.

### F3 — yêu cầu trợ năng còn trong hàng đợi GHI ĐÈ lựa chọn của người dùng

`tao` còn giao vài sự kiện sau khi ta đặt `Exit`, và phần rút hàng đợi không có
chắn "màn hình đã kết thúc". Người dùng bấm chuột vào **Từ chối**; một
`AXPress("cho-phep")` xếp trước đó biến kết quả thành **Cho phép**.

### F6 — adapter gắn SAU khi cửa sổ đã hiện

Hai chú thích khẳng định nó gắn *trước*. `tao` hiện và lấy tiêu điểm ngay trong
`build`. Giờ dựng ẩn, nối trợ năng, rồi mới hiện.

### Chỗ báo cáo xác nhận là SẠCH

`unsafe` **đúng**: adapter rơi trước cửa sổ, thứ tự thả biến ngược thứ tự khai
báo, và `run_return` chỉ mượn nên không đổi thứ tự ấy. `ControlFlow::Exit` không
bị ghi đè — `tao` chốt nó lại. Máy trạng thái công tắc, xử lý toạ độ, và luật
tiêu đề đều sạch.

### Chưa xử — ghi ra để không quên

- **F5: không ghép nhấn–nhả.** Nhấn ở "Từ chối", rê sang "Cho phép", nhả →
  **"Cho phép" chạy**. Mọi bộ công cụ lớn đều đòi nhấn và nhả trên cùng một nút.
  Cộng thêm `acceptsFirstMouse = YES`: một hộp thoại hiện ra dưới con trỏ giữa
  một cú bấm sẽ vừa kích hoạt ứng dụng vừa bấm nút bên dưới.
- **F4: `let _ = render(...)`.** Vẽ hỏng thì giữ nguyên hình học cũ, mà `bat` đã
  đổi rồi — trạng thái trong bụng lệch khỏi thứ người dùng thấy, vĩnh viễn.
  Cơ chế có thật; khả năng chạm tới **chưa chứng minh được**.
- **F7: không cuộn.** Cây cao hơn màn hình thì nút Cho phép/Từ chối nằm ngoài
  tầm. Hỏng theo hướng ĐÓNG (không bấm được gì thì `decide` trả Từ chối), nên là
  chuyện dùng được, không phải chuyện an ninh.

### Một chắn tôi giữ lại dù KHÔNG kiểm được

`hit_test` từ chối mọi toạ độ ngoài ảnh. Kiểm đột biến: **bỏ hẳn nó đi thì mọi
phép thử vẫn xanh** — vì một khi bố cục đúng thì không ô nào nằm ngoài ảnh, nên
điểm ngoài ảnh trượt hết mọi ô dù có chắn hay không.

Giữ lại có chủ ý, và nói ra rằng nó **không được kiểm**: giá trị của nó là ngày
bố cục hỏng trở lại. Nó biến một lỗi bố cục thành một **nút chết** thay vì một
**nút vô hình bấm được**.


## F7 đã vá — cuộn cho bộ dựng raster (22/08/2026)

Rà soát 21/08 xếp F7 là *"chuyện dùng được, không phải chuyện an ninh"*. Hôm sau
nó chặn đúng một người thật: nút nằm sát đáy, và **không bấm được**.

Đo lại thì **đường WebView vốn đã cuộn** — `body` không có `overflow:hidden`, có
phép thử canh, và tôi lái thử: thu cửa sổ còn 760×300, cuộn xuống đáy, bấm được
nút, nhật ký tăng đúng một dòng. Chỗ hở chỉ ở **bộ dựng raster**.

Vá hai lớp:

1. **Không mở cửa sổ cao bằng cả nội dung.** Cây cao tới 4096 mà màn hình thì
   không. Giới hạn ở 85% chiều cao màn hình. Quan trọng vì hộp thoại hỏi quyền
   để **ứng dụng** quyết định số quyền và độ dài từng câu `reason` — nên ứng
   dụng điều khiển được chiều cao, và nó không được điều khiển luôn việc nút có
   bấm tới được hay không.
2. **Lăn chuột để cuộn**, kẹp ở cả hai đầu.

### Chỗ dễ hỏng nhất của bản vá này

**Cuộn phải được CỘNG vào toạ độ bấm.** Quên là bấm trúng ô khác — cùng hạng lỗi
với F1, chỉ khác nguồn: ở đó là bố cục tràn, ở đây là cuộn. Có phép thử soi chỗ
gọi, và đột biến bỏ phép cộng thì nó đỏ.

Phép kẹp **ban đầu không kiểm được**: tôi viết nó nhận `&Window` cho tiện, và nó
thành thứ chỉ chạy khi có cửa sổ thật — tức là thứ `cargo test` không bao giờ
chạm tới. Tách thành số học thuần rồi mới kiểm được, và đột biến bỏ `clamp` thì
đỏ.
