# Việc còn lại

> Danh sách MÁY ĐỌC ĐƯỢC cho `tools/nhac-viec.sh`. Mỗi dòng một việc; đánh dấu
> `- [x]` khi xong. Văn xuôi về bối cảnh nằm ở [`dang-lam-gi.md`](dang-lam-gi.md),
> đừng chép sang đây.

## Từ lượt kiểm đột biến 26–27/08/2026

- [x] Quét `tcc-shell` ở cấu hình TỐI THIỂU (bẫy 8) — `RA=/tmp/dot-bien-toi-thieu CO_RIENG= tools/kiem-dot-bien.sh tcc-shell`
- [x] **`read_phrase` cắt cụt khoá dài mà không báo** — chuỗi toàn hex 128 ký tự lọt vào nhánh hạt giống thô, lấy 64 ký tự ĐẦU, trả về một ví KHÁC. Phép thử: 128 hex phải bị TỪ CHỐI (`recovery_screen.rs:242`)
- [x] Phân loại kẻ sống sót của lượt tối thiểu — 53/62 là hiện vật, 9 kẻ thật
- [x] **`action_refused` / `action_done` không phép thử nào chạm** — trả chuỗi rỗng vẫn xanh, tức bấm nút không khai thì im lặng hoàn toàn. `BUILDING-APPS.md` đã HỨA hành vi này (`text.rs:550,562`)
- [x] `kiem-dot-bien.sh`: tự loại hiện vật bằng dep-info `.d` — tệp không được biên dịch ở cấu hình đang chạy thì kẻ sống ở đó không phải kẻ sống
- [x] `do_net`: tìm điểm phân biệt cho đột biến đảo bộ lọc trong suốt (hiện 3/4)
- [ ] Quét lại `tcc-render-raster` sau khi vá, xem 235 kẻ sống còn bao nhiêu
- [ ] `tcc-keystore`: cho `SERVICE` đọc từ biến môi trường khi kiểm thử, quét dưới tên dịch vụ RIÊNG, dọn sạch sau. (Lý do cũ "treo vì hộp thoại" đã SAI — phép thử ấy là `#[ignore]`, bộ thử chạy 30 giây. Lý do thật: đột biến trên `delete` để lại rác trong Keychain thật)

## Cần NGƯỜI, không phải mã

- [ ] Một buổi đọc màn hình thật bằng VoiceOver
- [ ] Một cuộc kiểm định an ninh ĐỘC LẬP — cổng chặn mainnet phụ thuộc nó
- [ ] Một lượt soát `ttf-parser`
- [ ] Người bảo trì quyết ba mã lỗi chưa định nghĩa (`de-nghi-ma-loi-thieu.md`)
