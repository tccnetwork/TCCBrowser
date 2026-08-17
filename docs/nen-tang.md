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

## Chưa đo

- **Hiệu năng.** Máy phát triển là Intel Mac, Iris Plus 645 — kế hoạch đã dặn
  đừng kết luận đồ hoạ trên máy này.
- **Khác biệt hành vi.** Có `CSS grid` không có nghĩa ba bộ máy vẽ ra cùng một
  thứ. Bộ 50 trang thật + so ảnh chụp (mục 5.2) mới trả lời được, và nó chưa làm.
