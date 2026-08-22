# Bản cài đặt tham chiếu thế hệ sau — tư vấn kiến trúc

> ⚠️ **Từ vựng: quyết định này đã chốt từ lâu, ở dòng đầu `README.md`.**
>
> *"Sản phẩm thật của kho này là **tiêu chuẩn TCC** — một định dạng cho ứng dụng
> đã ký, có cổng quyền năng, **không mang mã**. Trình duyệt là **bản cài đặt
> tham chiếu** của nó."*
>
> Nên: sản phẩm là **tiêu chuẩn**; thứ ta đang dựng là **bản cài đặt tham
> chiếu**; `tcc-runtime` là **một crate** bên trong nó, không phải tên cả hệ
> thống.
>
> Bản đầu tài liệu này gọi nó là "nền tảng ứng dụng", rồi tôi sửa thành
> "runtime". **Cả hai đều sai**, và sai vì cùng một lý do: tôi đặt tên mới thay
> vì đọc quyết định đã có.

> Viết 22/08/2026, theo yêu cầu: *"hệ thống trình duyệt cũ quá cũ và chắp vá,
> tư vấn một hướng hoàn toàn mới, hiện đại, tối ưu, đủ tính năng."*

## Trước hết: "chắp vá" không phải tai nạn

Trình duyệt web chắp vá vì nó **không được phép nói không**. Mỗi quirk là một
trang thật của một người thật đã hỏng nếu gỡ đi. Chromium mang ba mươi năm nợ
ấy, và nó **không thoát ra được** — đó là ràng buộc, không phải sự cẩu thả.

Lợi thế **duy nhất** của người đi sau là **được phép từ chối**.

Nên bản thiết kế dưới đây không định nghĩa bằng thứ nó làm được, mà bằng
**những thứ nó từ chối** — và mỗi lời từ chối phải trả giá bằng một nhóm người
dùng cụ thể. Viết ra trước, lúc còn tỉnh táo.

## Quyết định số 1 — cái gì chạy mã ứng dụng

Mọi thứ khác suy ra từ đây. Trả lời: **WebAssembly Component Model**.

Và nó phải vào **tiêu chuẩn** trước, vào mã sau — luật 1 của `spec/README.md`
nói ngược lại (*"đặc tả rút ra từ mã ĐÃ CHẠY"*), nên trình tự đúng là: dựng chạy
được ở bản tham chiếu, rồi mới rút thành điều khoản 0.2 kèm vector.

Không phải "WASM" chung chung. Cụ thể là **component model + WIT** — vì nó biến
mô hình quyền năng của dự án thành thứ **máy kiểm được**:

```wit
world ung-dung-tcc {
  import tcc:ui/cay;              // vẽ màn hình
  import tcc:mang/http;           // ← chỉ có nếu người dùng đã cấp
  import tcc:vi/ky;               // ← chỉ có nếu người dùng đã cấp
  export khoi-tao: func();
  export cap-nhat: func(hanh-dong: string);
}
```

**Quyền năng KHÔNG được cấp = hàm KHÔNG tồn tại trong module.** Không phải một
phép kiểm lúc chạy có thể quên; ứng dụng **không gọi được** vì không có gì để
gọi. Đó là quyết định kiến trúc số 2 của dự án, viết thành kiểu dữ liệu.

So với JS: không `eval`, không đối tượng toàn cục, không prototype pollution,
đặc tả **nhỏ và đóng** nên viết vector kiểm định được — đúng văn hoá đã có ở
`conformance/`.

| Mảnh | Chọn | Vì sao |
|---|---|---|
| Máy chạy | `wasmtime` 48 | Rust, đã kiểm định, có component model |
| Giao diện | `wit-bindgen` 0.60 | sinh mã cho Rust, Go, C#, Swift, Python |

**Cái giá:** WASM không chạm DOM, nên ứng dụng WASM web có sẵn **không chạy
được**. Bạn không thừa hưởng hệ sinh thái — bạn tạo nền tảng mới.

## Quyết định số 2 — ứng dụng nói gì với màn hình

**Trả về cây, không sửa cây.**

```
WASM giữ trạng thái  →  cap-nhat(hành-động)  →  trả về CÂY MỚI
khung so cây cũ/mới  →  vẽ phần khác
```

Không `getElementById`, không tay cầm sống, không hai bên cùng sửa một cây. Mất
hẳn một lớp lỗi, và quan trọng hơn: **khung luôn biết toàn bộ màn hình**, nên
trợ năng và kiểm định là chuyện hiển nhiên chứ không phải gắn thêm.

Đây là mô hình của SwiftUI, Flutter, Elm. **Chép, đừng phát minh.**

## Quyết định số 3 — bố cục

**Đừng tự viết.** Dùng `taffy` 0.13 — Flexbox + Grid, thuần Rust, đang chạy
trong Zed, Dioxus, Bevy.

Bố cục là chỗ dễ tưởng đơn giản nhất và tốn nhiều tháng nhất: chảy dòng, co dãn,
kích thước nội tại, vòng phụ thuộc. Flexbox đã được cả ngành mài mười năm.

Điều này **không** kéo CSS vào: ta dùng *thuật toán* Flexbox, không dùng cú pháp
CSS, không cascade, không selector.

## Quyết định số 4 — chữ và vẽ

| Mảnh | Chọn | Ghi chú |
|---|---|---|
| Chữ | `parley` 0.11 | trên `swash`/`cosmic-text` — dự án **đã dùng** `cosmic-text` |
| Vẽ | `vello` 0.10 trên `wgpu` 30 | vẽ 2D bằng GPU, đường cong, gradient, ảnh |
| Cửa sổ | `tao` / `winit` | đã dùng |
| Trợ năng | `accesskit` 0.24 | đã dùng, đã nối macOS |

Chữ là phần khó nhất của mọi giao diện, và dự án **đã qua** nó: đâm thử 0.1 đo
được 0 `.notdef`, dựng sẵn ≡ tổ hợp, dấu tiếng Việt đúng chỗ.

⚠️ **Đừng chuyển sang GPU vội.** `softbuffer` (đang dùng) đủ cho tới khi có
hoạt ảnh hoặc danh sách dài. Chuyển sang `vello` là +1 lớp phụ thuộc lớn, +driver
GPU, +một lớp lỗi mới. Làm khi **đo được** là chậm, không làm vì nghe hiện đại.

## Bản thiết kế, một hình

```
        ứng dụng (Rust/Go/Swift → .wasm component)
                    │  WIT — chỉ những import ĐÃ ĐƯỢC CẤP
        ┌───────────┴────────────┐
        │   wasmtime (hộp cát)   │
        └───────────┬────────────┘
                    │  cây khai báo
        ┌───────────┴────────────┐
        │  taffy  → bố cục       │
        │  parley → chữ          │
        │  vello/softbuffer → vẽ │
        │  accesskit → trợ năng  │
        └───────────┬────────────┘
              tao/winit → cửa sổ
```

Mỗi tầng **thay được**, và ranh giới đã có sẵn trong dự án: luật 4 —
*"`tcc-ui` không được biết bộ dựng nào"*.

## "Đủ tính năng" nghĩa là gì — và giá của nó

| Giai đoạn | Nội dung | Ước lượng |
|---|---|---|
| **1. Bố cục thật** | thay `Flow`/`Gap` bằng `taffy`: kích thước, căn lề, cuộn, danh sách | 2–3 tháng |
| **2. WASM chạy được** | `wasmtime` + WIT, cây từ ứng dụng, vòng sự kiện | 2–3 tháng |
| **3. Quyền năng qua import** | mạng, lưu trữ, ví — mỗi thứ một `world` | 1–2 tháng |
| **4. Chữ giàu** | `parley`: chọn, sao chép, hai chiều, ảnh trong dòng | 2 tháng |
| **5. Nội dung** | ảnh, video, âm thanh — codec **miễn phí bản quyền** | 2–3 tháng |
| **6. Công cụ** | `tcc new`, đóng gói, gỡ lỗi, xem cây đang chạy | 2 tháng |
| **7. Phân phối** | định dạng gói, sổ khoá, cập nhật — **0.2 của đặc tả** | 3 tháng |

**Tổng: 14–20 tháng** một người, tới mức người khác viết ứng dụng thật được.

So với **3–5 năm** cho máy dựng web có JS — và cuối chặng ấy vẫn thua Chromium
ở mọi trang.

## Danh sách TỪ CHỐI — viết trước, mỗi dòng một cái giá

| Từ chối | Mất ai |
|---|---|
| HTML, CSS, DOM | mọi nội dung web có sẵn, mọi người viết web |
| JavaScript | mọi thư viện JS |
| Codec có bản quyền (H.264/HEVC) | video từ nhiều nguồn |
| DRM | Netflix, Spotify — **có chủ đích**: không nạp mã đóng không kiểm định được vào tiến trình người dùng |
| Ứng dụng chạy nền không giới hạn | thông báo tức thì kiểu web |
| Quyền ngầm | mọi ứng dụng quen "cứ gọi rồi tính" |

## Còn tầng 2 — trang web thật

**Giữ WebView.** Không mâu thuẫn: nền tảng mới là chỗ ứng dụng TCC chạy; WebView
là **cửa ra web của người khác**.

Ngày nào đó thay được thì thay. Nhưng đừng buộc hai việc vào nhau — làm thế là
nền tảng mới phải chờ một máy dựng web ba năm nữa mới ra mắt được.

## Ba điều tôi khuyên làm khác đi

**1. Lời khuyên "đừng gọi nó là trình duyệt" của tôi là THỪA.** `README.md` đã
không gọi thế từ đầu: sản phẩm là **tiêu chuẩn**, cái này là **bản cài đặt tham
chiếu**. Việc cần làm không phải đặt tên mới, mà là **giữ đúng tên đã có** —
gồm cả ở đây, chỗ tôi đã tự ý đổi hai lần.

**2. Bắt đầu bằng bố cục, không bằng WASM.** Bố cục là thứ chặn **mọi** màn hình
hôm nay — `ui.json` chỉ có hai khái niệm bố cục. WASM chưa chặn gì cả.

**3. Giữ nguyên phần đã đúng.** Chữ ký lai, cổng quyền năng, đặc tả + 153 vector,
`tcc-ui` không biết bộ dựng — bốn thứ ấy là **tài sản**, và không thứ nào dính
tới HTML. Bản thiết kế này giữ cả bốn.
