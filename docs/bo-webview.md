# Kế hoạch bỏ hẳn WebView

> Viết 22/08/2026. Đây là **kế hoạch của một quyết định đã đổi**: `ke-hoach.md`
> nói *"mượn WebView, không đua ở phần dựng web"*. Tài liệu này giả định điều
> ngược lại — tự dựng tất cả — và nói thẳng cái giá.

> **Đọc kèm:** [`nen-tang-moi.md`](nen-tang-moi.md) — bản thiết kế một nền tảng
> ứng dụng KHÔNG dùng HTML/CSS/DOM, dùng WASM component model. Tài liệu này nói
> *bỏ WebView tốn bao nhiêu*; tài liệu kia nói *thay bằng cái gì*.

## Hai bài toán, không phải một

Gộp chúng làm một là cách nhanh nhất để hỏng kế hoạch, vì chúng cách nhau **hai
bậc độ lớn**.

| | Tầng 1 — ứng dụng TCC | Tầng 2 — trang web bất kỳ |
|---|---|---|
| Đầu vào | `ui.json`, cây khai báo **ta định nghĩa** | HTML/CSS/JS **người khác viết** |
| Số loại nút | **6**, đóng | vài trăm thẻ, hàng nghìn thuộc tính CSS |
| Mang mã | **không** | có |
| Bộ dựng riêng | **đã chạy** — 2 497 dòng | **chưa có một dòng nào** |
| Còn thiếu | 10 chỗ gọi trong 2 tệp | một công cụ duyệt web |

**Tầng 1 gần xong. Tầng 2 chưa bắt đầu.** Mọi con số dưới đây tách theo hai vế
ấy.

## Giai đoạn A — bỏ WebView khỏi tầng 1 (tuần, không phải năm)

Đo được hôm nay: chỉ **`window.rs` (8 chỗ)** và **`wallet_flow.rs` (2 chỗ)** còn
gọi `WebViewRenderer` ở đường chạy thật. Tám tệp màn hình khác chỉ nhắc tới nó
trong phần kiểm thử — tức là chúng **đã** không phụ thuộc bộ dựng.

Bộ dựng raster đã vẽ đủ **cả sáu loại nút**, có cửa sổ, bấm được, cuộn được,
gạt công tắc được, và VoiceOver đọc được trên macOS.

| # | Việc | Cổng ra |
|---|---|---|
| A1 | Ô nhập **gõ được** ở raster (hiện chỉ vẽ ra, chưa nhận phím) | Gõ `chào buổi sáng` vào màn ví, dấu đúng chỗ |
| A2 | Bộ gõ tiếng Việt trên raster | Ca `ổ` — hai tầng dấu — đúng |
| A3 | `window_raster` phủ **mọi** màn hình khung, không chỉ 2 | Phép thử `moi_man_hinh_qua_duoc_ca_hai_bo_dung` chạy qua đường raster thật |
| A4 | Adapter trợ năng **Windows + Linux** | VoiceOver/NVDA/Orca đọc được |
| A5 | Đổi mặc định: `tcc-browser` dùng raster, WebView thành cờ | `cargo tree` của bản mặc định: **0 crate `wry`** |

**Ước lượng: 3–6 tuần** một người. A1 và A2 là phần thật sự khó — nhận phím,
dựng con trỏ, chọn chữ, và bộ gõ. Phần còn lại là nối dây.

**Sau A, tầng 1 không còn WebView.** Đó đã là một sản phẩm khác hẳn thứ bạn đang
có.

## Giai đoạn B — MỘT câu hỏi quyết định tất cả

> **Tầng 2 có cần JavaScript không?**

Đừng trả lời bằng cảm tính. Trả lời bằng một mũi đâm thử **2 tuần**:

- Viết bộ phân tích HTML đủ để dựng cây DOM tĩnh
- Cascade CSS tối thiểu: chọn theo thẻ/lớp/id, kế thừa, `display`, `font`, màu
- Bố cục: khối lồng nhau + văn bản chảy (đã có `cosmic-text` đo chữ)
- Vẽ ra `softbuffer` (đã có)
- **Dựng 10 trang tài liệu tĩnh**: Wikipedia, MDN, RFC, `doc.rust-lang.org`

**Cổng ra B:** một người **đọc được nội dung** 10 trang ấy, không cần đẹp.

Kết quả mũi đâm quyết định nhánh:

| Nếu | Thì |
|---|---|
| Đọc được, và bạn chấp nhận *"trình duyệt cho tài liệu"* | Đi nhánh **C-hẹp** |
| Cần đăng nhập, biểu mẫu, ứng dụng web | Đi nhánh **C-rộng**, và đọc lại phần "giá" bên dưới |

## Giai đoạn C-hẹp — trình duyệt cho TÀI LIỆU, không có JS

Không JS, không `<canvas>`, không WebGL, không video. Đọc báo, đọc tài liệu, đọc
whitepaper. Từ chối phần còn lại **và nói thẳng ra**.

| # | Việc |
|---|---|
| C1 | HTML5 parser đúng spec (khoan dung với thẻ hỏng — đây là phần bẩn nhất) |
| C2 | CSS: cascade, kế thừa, độ ưu tiên; `flexbox` + `grid` |
| C3 | Bố cục khối/dòng, float, `position`, bảng |
| C4 | Chữ: chảy dòng hai chiều, ghép chữ, dự phòng font, chọn/sao chép |
| C5 | Ảnh: PNG/JPEG/WebP/AVIF, SVG tĩnh |
| C6 | Mạng: HTTP/2, cache, cookie, TLS (dùng `rustls`) |
| C7 | Cuộn, thẻ, lịch sử, tìm trong trang |

**Ước lượng: 12–18 tháng** một người, tới mức "đọc báo hằng ngày được".
Ladybird cần ~7–10 người để tới alpha.

## Giai đoạn C-rộng — có JavaScript

Thêm vào C-hẹp:

| # | Việc |
|---|---|
| D1 | Máy JS — **không tự viết**. Nhúng một cái đã có (`boa` thuần Rust, hoặc V8/JSC qua FFI) |
| D2 | DOM API phơi cho JS: ~vài trăm giao diện |
| D3 | Vòng đời sự kiện, `fetch`, `Promise`, timer |
| D4 | Vẽ lại tăng dần — không có nó thì mọi trang có JS đều giật |

**Ước lượng: 3–5 năm** một người. Đây là chỗ Servo đứng sau 13 năm có Mozilla
tài trợ.

**Và một điều phải nói:** nhúng V8 nghĩa là nhúng **~2 triệu dòng C++ không kiểm
định được** vào một trình duyệt mà cả lý do tồn tại là `unsafe_code = "deny"` và
một `unsafe` duy nhất được ghi chép cẩn thận. `boa` thì thuần Rust nhưng chậm
hơn V8 hàng chục lần và chưa phủ đủ.

Nếu chọn C-rộng, **quyết định máy JS trước mọi thứ khác** — nó định đoạt cả mô
hình an ninh.

## Thứ tự tôi đề nghị

```
A (3–6 tuần)  →  B (2 tuần, mũi đâm)  →  quyết định  →  C-hẹp hoặc dừng
```

**Vì sao A trước:** nó xong nhanh, và nó cho bạn một sản phẩm **không WebView**
ngay — đúng thứ bạn muốn thấy — trước khi đổ tiền vào bài toán lớn.

**Vì sao B trước C:** hai tuần mua được câu trả lời cho một quyết định 12 tháng.
Mũi đâm 0.1 (chữ tiếng Việt bằng Rust) đã cứu dự án khỏi một hướng sai; mũi đâm
này cùng vai trò.

## Ba cổng DỪNG, viết trước khi bắt đầu

Viết ra lúc còn tỉnh táo, vì lúc đã đổ 6 tháng vào thì không ai muốn dừng.

1. **Sau B:** nếu 10 trang tài liệu không đọc được sau 2 tuần → **dừng**, giữ
   WebView cho tầng 2.
2. **Sau 6 tháng C:** nếu chưa đọc được VnExpress ở mức chấp nhận → **dừng**.
3. **Bất cứ lúc nào** một lỗi dựng hình làm người dùng **hiểu sai nội dung**
   (chữ chồng, số bị che, nút sai chỗ) → dừng tính năng, sửa xong mới đi tiếp.
   Trình duyệt hiển thị sai một con số còn tệ hơn trình duyệt không mở được
   trang.

## Thứ KHÔNG mất khi bỏ WebView

Đã dựng sẵn và dùng lại được nguyên vẹn:

- `tcc-ui` — cây component, **không biết bộ dựng nào** (luật 4)
- `tcc-render-raster` — 2 497 dòng, đo chữ, bố cục, vẽ, trợ năng
- `cosmic-text`, `softbuffer`, `tao` — chữ, bề mặt, cửa sổ
- `tcc-net` — đường ra mạng, sau cổng quyền năng
- Chữ ký, quyền năng, ví, đặc tả, 153 vector — **không dính gì tới bộ dựng**

Đó là lý do giai đoạn A rẻ: nền móng đã đúng từ đầu.

## Điều tôi phải nói thẳng

Bỏ WebView ở **tầng 1** là việc kỹ thuật bình thường, vài tuần, và tôi khuyên
làm.

Bỏ WebView ở **tầng 2** là **viết một trình duyệt web**. Không có đường tắt nào
mà tôi biết, và những người đã thử đều mất nhiều năm với nhiều người hơn. Làm
được — nhưng phải vào với con mắt mở, và với ba cái cổng dừng ở trên đã viết sẵn.
