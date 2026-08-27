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

- [x] Soát nốt `BUILDING-APPS.md` theo tiêu chí "hứa gì với người ngoài thì phải có phép thử canh" — soát đủ 9 khẳng định: 8 có vector/phép thử canh, 1 (`quota_bytes: 0`) không ai canh và đã vá
- [ ] **Mệnh đề chắn của `hit_test`/`hit_test_field`: 13 kẻ sống, giết được bằng trạng thái CÓ THẬT.** Vẽ ở 640, gọi `set_width(320)` mà CHƯA vẽ lại → ô ở x>320 nằm ngoài ảnh; lúc ấy mệnh đề chắn là thứ duy nhất chặn cú bấm. Đột biến `||`→`&&` ở vế cuối làm nó không nổ, và phép kiểm hình chữ nhật trả về một ô đã biến mất khỏi màn hình. Tình huống đời thường: THU HẸP CỬA SỔ RỒI BẤM vào vùng vừa mất. Viết cho CẢ HAI hàm song sinh
- [ ] Chạy cổng số liệu đầy đủ sau khi lượt quét xong (số 399 phép thử hiện CHƯA được cổng kiểm)

## Cần NGƯỜI, không phải mã

- [ ] @NGƯỜI Một buổi đọc màn hình thật bằng VoiceOver
- [ ] @NGƯỜI Một cuộc kiểm định an ninh ĐỘC LẬP — cổng chặn mainnet phụ thuộc nó
- [ ] @NGƯỜI Một lượt soát `ttf-parser`
- [ ] @NGƯỜI Người bảo trì quyết ba mã lỗi chưa định nghĩa (`de-nghi-ma-loi-thieu.md`)
