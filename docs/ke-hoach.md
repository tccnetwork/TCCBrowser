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

### Cổng ra — Giai đoạn 1 xong khi

Trạng thái ngày 14/08/2026. Mỗi mục ghi **bằng chứng chạy được**, không ghi
cảm nhận — cổng nào chỉ dựa vào trí nhớ thì coi như chưa đóng.

- [x] `cargo check --workspace` sạch trên macOS **và** Linux
      → CI GitHub Actions, cả `macos-latest` lẫn `ubuntu-latest`
- [x] `tools/kiem-luat-phu-thuoc.sh` — 0 vi phạm
      → 17 luật, chạy TRƯỚC bước biên dịch trong CI
- [x] `tcc sign` rồi `tcc verify` chạy đúng; **sửa một byte trong gói là kiểm thất bại**
      → đo 14/08: ký `exit 0` · kiểm `exit 0` · lật MỘT bit trong `ui.json` `exit 1`
      · khôi phục `exit 0` · xoá `signature.hex` `exit 1`
- [x] Ứng dụng chưa được cấp quyền **không gọi được** thứ nó không xin
      → `kiem-hanh-vi`, và phép thử đếm số lần đường mạng bị gọi phải bằng **0**
- [x] Ứng dụng mẫu hiện trên màn hình
      → `kiem-man-hinh-ung-dung`, chạy trong CI qua WebKit thật
- [x] **VoiceOver đọc được ứng dụng mẫu**
      → soi cây trợ năng THẬT 13/08; hai lỗi tìm ra và sửa (B32, B33)
- [ ] **Gõ tiếng Việt có dấu bằng bộ gõ hệ thống, dấu chồng đúng, con trỏ đúng chỗ**
      → ⚠️ **CỔNG DUY NHẤT CÒN MỞ.** Cần một người ngồi trước máy: bộ gõ là của
      hệ điều hành, mọi cách giả lập đều bơm thẳng chuỗi hoàn chỉnh vào ô nhập,
      tức là bỏ qua đúng cái cần đo — phiên ghép. Công cụ đã sẵn:

      ```sh
      cargo run -p tcc-shell --features window --example kiem-go-tieng-viet -- examples/hello-tcc
      ```

      Nó hỏi lại WebKit ô nhập chứa gì rồi in mã điểm, số dấu rời, dấu chồng
      nhiều nhất trên một chữ, và vị trí con trỏ. Nhìn bằng mắt không đủ: `ỡ`
      dựng sẵn và `o`+2 dấu hiện ra y hệt nhau mà tốn trần `MAX_COMBINING_MARKS`
      khác hẳn nhau.
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
| 3.1 | Ví: khoá gắn kho khoá hệ điều hành (Keychain / DPAPI) |
| 3.2 | Màn xác nhận giao dịch đọc được bằng tiếng người |
| 3.3 | Danh tính + chứng thực |
| 3.4 | Bảo vệ nội dung TCC — quyền sở hữu chứng minh trên chuỗi |

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
| 4.1 | `tcc-render-gpu` cài cùng trait với `tcc-render-webview` |
| 4.2 | Chữ: shaping, dự phòng font, **dấu tiếng Việt** |
| 4.3 | Bố cục, hợp thành |
| 4.4 | Trợ năng qua AccessKit |

**Cổng ra:** ứng dụng mẫu chạy trên **cả hai** bộ dựng, **không sửa một dòng nào**.
Đó là lúc chứng minh được đường thoát là thật.

---

## Giai đoạn 5 — Tầng web hiện đại

| # | Việc |
|---|---|
| 5.1 | Công bố **TCC Modern Baseline** — chính xác những gì hỗ trợ |
| 5.2 | Bộ 50 trang thật, so ảnh chụp hằng tuần |
| 5.3 | Nhãn "TCC Ready" cho trang đạt chuẩn |
| 5.4 | Tầng 3: nút mở bằng trình duyệt hệ thống |

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
