> **Internal working document, in Vietnamese.** The normative standard is in
> [`spec/`](../spec/) and is English-normative.

# Kế hoạch triển khai

> **Đơn vị là KHỐI CÔNG VIỆC, không phải tuần lịch.** Chưa có số người nên mọi
> mốc thời gian đều là bịa. Cách quy đổi ở cuối tài liệu.

---

## Giai đoạn 0 — Đâm thử tính khả thi

**Mục đích:** trả lời ba câu hỏi bằng **mã chạy được**, trước khi cam kết bất cứ
điều gì. Nếu một trong ba trả lời xấu, kiến trúc phải đổi — và biết sớm thì rẻ.

| # | Câu hỏi | Cách đo | Trả lời xấu nghĩa là gì |
|---|---|---|---|
| 0.1 | Chữ dựng bằng wgpu có đẹp bằng hệ điều hành không? | Vẽ một đoạn **tiếng Việt có dấu**, chụp ảnh so với native, phóng to so từng chữ | Bộ dựng GPU riêng bị lùi vô thời hạn — người dùng nhìn chữ suốt ngày |
| 0.2 | Sandbox WASM có đủ nhanh không? | Chạy `wasmtime`, đo một vòng lặp thật, so với native | Mô hình WASM-first phải xem lại |
| 0.3 | Cô lập tiến trình trên macOS làm được tới đâu? | Thử tách tiến trình ví, đo cái gì thật sự bị chặn | §21 của đặc tả gốc hứa quá lời, phải viết lại |

⚠️ **Máy phát triển hiện tại là Intel Mac, Iris Plus 645.** GPU yếu và **không
giải mã AV1 bằng phần cứng**. Đừng đánh giá hiệu năng đồ hoạ hay video trên máy
này rồi kết luận cho cả dự án.

**Sản phẩm:** ba báo cáo ngắn kèm số đo và ảnh chụp. Không phải mã sản xuất.

**Cổng ra:** ba câu trả lời, và quyết định có đi tiếp không.

### Trạng thái (17/08/2026) — [`../dam-thu/BAO-CAO.md`](../dam-thu/BAO-CAO.md)

| | Câu | Kết quả |
|---|---|---|
| 0.1 | Chữ tiếng Việt dựng bằng Rust | ✅ **ĐẠT** — 0 `.notdef`, dựng sẵn ≡ tổ hợp (0/2560 pixel lệch), mực 17 px trong dòng 21 px |
| 0.2 | Sandbox WASM có đủ nhanh không | ⊘ **câu hỏi đã tan** — kiến trúc bỏ WASM, workspace không phụ thuộc `wasmtime` |
| 0.3 | Cô lập tiến trình trên macOS | ❌ **chưa đo** — nên làm SAU khi có hồ sơ cấp phép |

Giai đoạn 0 chạy **sau** Giai đoạn 1–2, không phải trước. Đó là một sai lệch so
với kế hoạch, và nó có giá: Giai đoạn 4 đã bị chặn suốt thời gian ấy bởi một câu
hỏi chưa ai mở ra xem.

---

## Giai đoạn 1 — Lát cắt mỏng chạy thông suốt

**Mục đích:** một ứng dụng TCC đã ký chạy được từ đầu tới cuối.

Không phải đặc tả trước. Không phải bộ dựng hình trước.

> Tiêu chuẩn viết trước khi có mã chạy phần lớn đều chết — XHTML 2.0, SOAP, cả họ
> WS-*. Tiêu chuẩn thành công thì được **rút ra từ thứ đã chạy thật**: HTML5, HTTP.
> Nên ta xây lát cắt trước, rút đặc tả ra sau.

### Việc

| # | Việc | Crate |
|---|---|---|
| 1.1 | Kiểu dữ liệu bản kê khai + lược đồ JSON | `tcc-spec` |
| 1.2 | Ký/kiểm lai Ed25519 + ML-DSA | `tcc-crypto` |
| 1.3 | Đọc và xác thực thư mục gói | `tcc-manifest` |
| 1.4 | Mô hình quyền năng: cấp, phạm vi, thu hồi | `tcc-capability` |
| 1.5 | API component + trait bộ dựng | `tcc-ui` |
| 1.6 | Bộ dựng WebView (macOS trước) | `tcc-render-webview` |
| 1.7 | Nạp và chạy ứng dụng | `tcc-runtime` |
| 1.8 | Khung: một cửa sổ, mở được một ứng dụng | `tcc-shell` |
| 1.9 | `tcc new` / `tcc sign` / `tcc verify` | `tcc-cli` |
| 1.10 | Ứng dụng mẫu `hello-tcc` | `examples/` |

### Cổng ra — Giai đoạn 1 ✅ ĐÓNG ĐỦ CHÍN CỔNG (15/08/2026)

Trạng thái ngày 14/08/2026. Mỗi mục ghi **bằng chứng chạy được**, không ghi
cảm nhận — cổng nào chỉ dựa vào trí nhớ thì coi như chưa đóng.

- [x] `cargo check --workspace` sạch trên macOS **và** Linux
      → CI GitHub Actions, cả `macos-latest` lẫn `ubuntu-latest`
- [x] `tools/kiem-luat-phu-thuoc.sh` — 0 vi phạm
      → 18 luật, chạy TRƯỚC bước biên dịch trong CI
- [x] `tcc sign` rồi `tcc verify` chạy đúng; **sửa một byte trong gói là kiểm thất bại**
      → đo 14/08: ký `exit 0` · kiểm `exit 0` · lật MỘT bit trong `ui.json` `exit 1`
      · khôi phục `exit 0` · xoá `signature.hex` `exit 1`
- [x] Ứng dụng chưa được cấp quyền **không gọi được** thứ nó không xin
      → `kiem-hanh-vi`, và phép thử đếm số lần đường mạng bị gọi phải bằng **0**
- [x] Ứng dụng mẫu hiện trên màn hình
      → `kiem-man-hinh-ung-dung`, chạy trong CI qua WebKit thật
- [x] **VoiceOver đọc được ứng dụng mẫu**
      → soi cây trợ năng THẬT 13/08; hai lỗi tìm ra và sửa (B32, B33)
- [x] **Gõ tiếng Việt có dấu bằng bộ gõ hệ thống, dấu chồng đúng, con trỏ đúng chỗ**
      → **ĐÓNG 15/08/2026.** Gõ `Chào buổi sáng mọi người` bằng Telex của macOS
      qua `kiem-go-tieng-viet`: **24 mã điểm / 24 chữ gốc / 0 dấu rời**, tức bộ
      gõ cho ra dạng **DỰNG SẴN** — `ổ` là một `U+1ED5` chứ không phải `o` cộng
      hai dấu. Con trỏ ở 24, đúng cuối; phiên ghép đã chốt.

      Vì sao con số ấy quan trọng: dạng tách rời hiện ra y hệt nhưng tốn trần
      `MAX_COMBINING_MARKS` gấp ba, nên một câu bình thường sẽ bò tới gần cái
      trần vốn dựng để chặn kẻ tấn công. Mắt không phân biệt được; chỉ hỏi lại
      WebKit mới thấy.

      ⚠️ Đo trên **Telex của macOS**. Bộ gõ khác (EVKey, OpenKey, hoặc IME của
      Windows/Linux) có thể cho ra dạng tách rời — công cụ vẫn còn đó để chạy
      lại, và đó là việc nên làm trước khi phát hành trên hệ điều hành mới.

- [x] Chưa có mã ví nào chạm khoá riêng thật
      → không có mã ví nào tồn tại; cổng cứng ở `SECURITY.md` §3.5
- [x] Chưa trộn tầng web vào lõi
      → luật 6 cưỡng chế: API ứng dụng không lộ DOM/HTML/CSS

---

## Giai đoạn 2 — Rút tiêu chuẩn ra, làm bộ kiểm định

**Mục đích:** biến thứ đã chạy thành thứ người khác cài đặt được.

| # | Việc |
|---|---|
| 2.1 | Viết `spec/0.1/` từ mã đã chạy — kê khai, quyền năng, ký |
| 2.2 | **Bộ kiểm định tuân thủ** chạy từ dòng lệnh |
| 2.3 | Chính sách phiên bản và khai tử |
| 2.4 | Quy trình quản trị: ai quyết định thay đổi tiêu chuẩn |

**Luật:** mỗi mục trong `spec/` phải có **ít nhất một phép kiểm**. Thêm điều vào
đặc tả mà không thêm phép kiểm là thêm một lời hứa không ai kiểm được.

**Cổng ra:** một người ngoài đọc `spec/0.1/` và tự làm được **thư mục gói** hợp lệ (0.1 không định nghĩa dạng nén nào)
mà **không cần hỏi ai**. Đây là phép thử duy nhất chứng minh đặc tả viết đủ rõ.

---

## Giai đoạn 3 — Ví, danh tính, bảo vệ nội dung

| # | Việc |
|---|---|
| 3.1 | Ví: khoá gắn kho khoá hệ điều hành — ✅ **Keychain chạy thật** (22/08/2026). Gói `.app` đã ký + hồ sơ `MAC_APP_DEVELOPMENT` → cất/xoá được khoá với `USER_PRESENCE`. Xem [`../docs/vi-thiet-ke.md`](../docs/vi-thiet-ke.md) §19b. DPAPI (Windows) chưa |
| 3.2 | Màn xác nhận giao dịch đọc được bằng tiếng người |
| 3.3 | Danh tính + chứng thực — 🔶 **vân tay người ký** đã đúng và đã vào tiêu chuẩn (18/08/2026). **Chứng thực chưa**: cần sổ khoá, thuộc 0.2 — xem [`../docs/vi-thiet-ke.md`](../docs/vi-thiet-ke.md) §30 |
| 3.4 | ~~Bảo vệ nội dung TCC~~ → **kiểm sở hữu trên chuỗi** — 🔶 phần kiểm xong (18/08/2026). **Bảo vệ nội dung KHÔNG làm được** nếu không có máy chủ phát khoá; xem [`../docs/vi-thiet-ke.md`](../docs/vi-thiet-ke.md) §31 |

**Cổng chặn cứng:**

> **Không giao dịch nào lên mainnet trước khi qua kiểm định bảo mật độc lập.**

**Về bảo vệ nội dung — mô hình đe doạ phải trung thực:**

| | |
|---|---|
| ✅ Chặn được | sao chép tuỳ tiện, chia sẻ lại, xem không trả tiền |
| ❌ Không chặn được | người có kỹ thuật và có động lực |
| ✅ Mục tiêu thật | nâng chi phí tấn công cao hơn giá trị nội dung |

Gọi đúng tên là **"bảo vệ nội dung"**, đừng gọi là *"DRM không phá được"*. Không
có DRM phần mềm nào không phá được, kể cả của Google.

---

## Giai đoạn 4 — Bộ dựng riêng (thoát WebView)

Chỉ bắt đầu khi Giai đoạn 0.1 trả lời tốt **và** `tcc-ui` đã ổn định qua ứng dụng
thật.

| # | Việc |
|---|---|
| 4.1 | `tcc-render-gpu` cài cùng trait với `tcc-render-webview` — ✅ **một nửa xong**: `tcc-render-raster` (17/08/2026) đã cài cùng trait, ra pixel, không HTML |
| 4.2 | Chữ: shaping, dự phòng font, **dấu tiếng Việt** — ✅ **xong ở `tcc-render-raster`** (17/08/2026): đo bề rộng thật, ngắt dòng, dấu chồng đúng |
| 4.3 | Bố cục, hợp thành — ✅ **xong** (17/08): hàng/cột, xuống dòng, ngắt giữa từ, căn giữa dọc, nút cùng hàng rộng bằng nhau, và **bất biến KHÔNG vẽ đè**. "Căn lề" cố ý không vào tiêu chuẩn — [`../docs/vi-thiet-ke.md`](../docs/vi-thiet-ke.md) §23 |
| 4.4 | Trợ năng qua AccessKit — 🔶 **ánh xạ xong** (17/08), **adapter macOS đã nối** (19/08/2026, cờ `cua-so-raster-tro-nang`). Còn Windows, Linux, và `ActionHandler` (cố ý để trống — xem dưới) |

**Cổng ra:** ứng dụng mẫu chạy trên **cả hai** bộ dựng, **không sửa một dòng nào**.
Đó là lúc chứng minh được đường thoát là thật.

🔶 **Đạt phần vẽ, bấm và GẠT CÔNG TẮC** (19/08/2026). `cargo run -p tcc-shell --features
cua-so-raster --example man-hinh-raster examples/hello-tcc` mở gói `hello-tcc`
**đã ký** trong một cửa sổ thật, vẽ hoàn toàn bằng Rust (`cosmic-text` +
`softbuffer`), bấm nút chạy đúng hành động. Cùng thư mục gói, cùng chữ ký, cùng
`ui.json` với đường WebView — khác đúng một dòng chọn bộ dựng.

Và chạy được **hộp thoại hỏi quyền** — màn hình đáng chạy nhất trên bộ dựng thứ
hai, vì nó có **công tắc**:

```
cargo run -p tcc-shell --features cua-so-raster \
  --example man-hinh-raster examples/hello-tcc hop-thoai
```

Công tắc là chỗ hai bộ dựng **khác nhau thật sự**. WebView để trình duyệt giữ
trạng thái trong tài liệu rồi hỏi lại lúc bấm xác nhận; bộ dựng ra pixel không
có ai giữ hộ, nên khung phải tự giữ và `Node::with_toggles` đưa trạng thái ấy
quay lại thành một cây vẽ được. Màn hình chỉ có chữ và nút thì không đá vào chỗ
này — nên nếu chỉ chạy `hello-tcc` thì cổng ra vẫn còn một nửa chưa kiểm.

Ba luật của hộp thoại giữ nguyên trên bộ dựng mới: mở ra **mọi mục tắt**,
**đóng cửa sổ không phải đồng ý**, và **gạt công tắc không đóng hộp thoại**.

Cờ `cua-so-raster` **tách khỏi** cờ `window` có chủ ý: `window` kéo theo `wry`
và cả một máy dựng web. `cargo tree` với `cua-so-raster` cho **0 crate `wry`** —
cây phụ thuộc tự nói ra rằng đường thoát không tựa vào thứ nó định thoát khỏi.

🔶 **Trợ năng: đã nối trên macOS** (19/08/2026), cờ `cua-so-raster-tro-nang`.
`accesskit_macos::SubclassingAdapter` gắn vào `NSView` của cửa sổ **trước khi
cửa sổ hiện lần đầu**, và cây được đẩy lại mỗi lần công tắc đổi — gạt một quyền
mà không báo lại thì VoiceOver vẫn đọc trạng thái cũ, tức là nghe một đằng cấp
một nẻo.

❌ **Windows và Linux chưa nối.** Nói ra bằng `cfg` chứ không bằng chú thích: hai
nền ấy dựng ra một bản **không có mã trợ năng nào**, và đó là sự thật cần nhìn
thấy được.

✅ **`ActionHandler` đã nhận `Click`** (19/08/2026) — và tôi **đổi ý** so với hôm
trước. Lập luận cũ ("nhận yêu cầu bấm là mở đường ký hộ") so với một thế giới
không tồn tại: trên macOS, gửi `AXPress` cần quyền **Accessibility**, mà cùng
quyền ấy cũng cho `CGEventPost` — **tổng hợp một cú bấm chuột thật**, đi thẳng
qua đường chuột của ta. *Từ chối `AXPress` không chặn kẻ tấn công, nó chỉ chặn
người dùng VoiceOver.* Xem `SECURITY.md` §3.1d.

Yêu cầu **không có nhánh riêng**: nó chạy qua đúng `sau_cu_bam` mà chuột đi qua.

**`unsafe` đầu tiên và duy nhất của dự án** nằm ở đây: trao con trỏ `NSView` cho
AccessKit. `SECURITY.md` §3.1b đã lường trước đúng đánh đổi này — *"làm bây giờ
nghĩa là thêm `unsafe` FFI, chỉ phủ macOS"* — và hoãn nó tới giai đoạn 4. Đây là
giai đoạn 4. Dùng `#[expect]` chứ không `#[allow]`: ngày có bọc an toàn, lint tự
báo ngoại lệ này đã thừa.

---

## Giai đoạn 5 — Tầng web hiện đại

| # | Việc |
|---|---|
| 5.1 | **TCC Modern Baseline** — ✅ **xong** (19/08/2026), [`../docs/nen-tang.md`](../docs/nen-tang.md). Đủ ba bộ máy: WKWebView, WebKitGTK, WebView2 đều **18/20**, thiếu đúng cùng hai mục. Linux đo được sau khi sửa `build_gtk` — chỗ ấy không hề chập chờn, nó trượt đều 3/3 vì mã ta gọi sai hàm |
| 5.2 | Bộ 50 trang thật — ✅ **xong** (18/08/2026), `corpus/50-trang.txt`, kết quả ở `docs/nen-tang.md`. **Không so ảnh chụp**: tầng 2 dùng máy dựng của hệ điều hành nên so điểm ảnh là đo WebKit của Apple, không đo mã của ta. Đo giá của chính sách ta đặt, thay vào đó |
| 5.3 | Nhãn "TCC Ready" — ❌ **không làm, có chủ đích** (18/08/2026). Bộ đếm không tách được "trang cần quyền năng ấy" khỏi "quảng cáo của trang thử đòi", nên nhãn đạt/trượt sẽ đổ lỗi cho trang vì quảng cáo của nó. Thay bằng tính chất đo được và gọi đúng tên: **"im khi nạp"**. Lý do đầy đủ ở [`../docs/nen-tang.md`](../docs/nen-tang.md) |
| 5.4 | Tầng 3: nút mở bằng trình duyệt hệ thống — ✅ **xong** (17/08/2026), `crates/tcc-shell/src/external_link.rs` |

**Thước đo:** không đuổi theo tỷ lệ WPT tổng — nó vô nghĩa khi ta cố ý không nhắm
tương thích đầy đủ. Đo phần trăm **trên đúng tập đã công bố**, cộng bộ 50 trang thật.

**Codec:** chỉ thứ miễn phí bản quyền — AV1, VP9, Opus, FLAC. Bỏ H.264/HEVC.
**DRM: không hỗ trợ, có chủ đích.** Một runtime bảo mật không nạp mã đóng không
kiểm định được vào tiến trình người dùng. Netflix → Tầng 3.

---

## Rủi ro lớn nhất — và nó không phải kỹ thuật

> **Vì sao một lập trình viên bỏ công triển khai theo cách TCC?**

Tiêu chuẩn không có người dùng là một tệp PDF. Ba câu chưa có đáp án:

1. Ứng dụng đầu tiên nào đủ hay để người ta cài trình duyệt?
2. Cộng đồng TCC hiện có bao nhiêu người thật?
3. Lập trình viên được gì mà web thường không cho — ví có sẵn, danh tính, thanh toán?

Đây là câu hỏi sản phẩm, và nó quyết định thành bại nhiều hơn mọi lựa chọn kiến
trúc trong tài liệu này. **Nên trả lời trước khi vào Giai đoạn 2.**

---

## Quy đổi ra thời gian

Kế hoạch trên đo bằng khối công việc. Muốn ra lịch thì cần biết số người — nhưng
vài mốc để đối chiếu, đừng tự lừa mình:

| Dự án | Nguồn lực | Kết quả |
|---|---|---|
| **Servo** | Mozilla tài trợ từ 2012 | Tới nay chưa dùng hằng ngày được |
| **Ladybird** | ~7–10 kỹ sư toàn thời gian có lương | Nhắm bản alpha 2028 |
| **Chromium** | Hàng nghìn kỹ sư | — |

Đây chính là lý do kế hoạch này **mượn WebView** và **không bắt đầu bằng bộ dựng
hình**. Ta không đua với họ ở phần dựng web. Ta đua ở phần chưa ai làm: quyền
năng, ví trong runtime, chữ ký hậu lượng tử, và một tiêu chuẩn có bộ kiểm định.

Ba thứ đầu **Chromium không thêm vào được nữa** — kiến trúc của nó đã khoá lại rồi.
Đó là lợi thế duy nhất của người đi sau, và là chỗ duy nhất đáng đổ sức.
