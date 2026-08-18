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
| Linux | WebKitGTK | ⚠️ chạy dưới màn hình ảo **không ổn định** — `the underlying handle is not available`. Cùng lý do các bước đối kháng trên Linux phải để `continue-on-error` |
| Windows | WebView2 | ✅ đo được, **18/20 — thiếu ĐÚNG hai mục ấy** |

**Hai bộ máy hoàn toàn khác nhau (WebKit và Chromium) thiếu cùng hai mục.** Đó
không phải trùng hợp — nó xác nhận rằng hai mục ấy vắng vì **cách nạp**, không
vì bộ máy.

**Chưa công bố bảng nền tảng nào**, vì phần giao cần đủ ba số đo. Công bố khi
mới có một là công bố một bảng của macOS và gọi nó là nền tảng.

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
