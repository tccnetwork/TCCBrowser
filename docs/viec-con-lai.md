# Việc còn lại

> ⚑ **Trọng tâm 27/08/2026: GIAO DIỆN và BỘ CÔNG CỤ cho người viết ứng dụng.**
> Chủ dự án chốt. Việc kiểm đột biến còn lại vẫn chạy nền, nhưng không phải chỗ
> đổ công chính nữa.

## Bộ công cụ cho người viết ứng dụng (SDK)

- [x] **`tcc check <thư-mục>`** — kiểm bản kê khai + `ui.json` mà KHÔNG ký. Vòng phản hồi nhanh nhất cho người viết ứng dụng: hiện đang phải `sign` rồi mới biết sai, mà `sign` đụng khoá. `check` chỉ đọc, trả đúng mã lỗi của đặc tả
- [x] `tcc new` sinh khung nghèo (chỉ text) — thêm mẫu dựng đủ sáu loại thành phần, để người ta thấy được cái gì có sẵn
- [x] Thông báo lỗi của `tcc` có kèm **mã lỗi đặc tả** không? Người viết ứng dụng cần tra được `06-error-codes.md`
- [x] `BUILDING-APPS.md`: thêm mục "vòng làm việc" — sửa `ui.json` xong PHẢI ký lại, nếu không gói không nạp được. Đây là vấp đầu tiên của mọi người mới

## Giao diện



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
- [x] `tcc-keystore`: ĐÃ tách tên dịch vụ bằng `cfg(test)` (không dùng biến môi trường — biến môi trường là một đường vào mới cho mã sản phẩm). Còn: chạy lượt quét cho hòm này, rồi dọn `com.tcc.browser.wallet.KIEM-THU`. (Lý do cũ "treo vì hộp thoại" đã SAI — phép thử ấy là `#[ignore]`, bộ thử chạy 30 giây. Lý do thật: đột biến trên `delete` để lại rác trong Keychain thật)

- [x] Soát nốt `BUILDING-APPS.md` theo tiêu chí "hứa gì với người ngoài thì phải có phép thử canh" — soát đủ 9 khẳng định: 8 có vector/phép thử canh, 1 (`quota_bytes: 0`) không ai canh và đã vá
- [x] **Mệnh đề chắn của `hit_test`/`hit_test_field`: 13 kẻ sống, giết được bằng trạng thái CÓ THẬT.** Vẽ ở 640, gọi `set_width(320)` mà CHƯA vẽ lại → ô ở x>320 nằm ngoài ảnh; lúc ấy mệnh đề chắn là thứ duy nhất chặn cú bấm. Đột biến `||`→`&&` ở vế cuối làm nó không nổ, và phép kiểm hình chữ nhật trả về một ô đã biến mất khỏi màn hình. Tình huống đời thường: THU HẸP CỬA SỔ RỒI BẤM vào vùng vừa mất. Viết cho CẢ HAI hàm song sinh
- [x] `do_net`: ghim ĐÁP ÁN CHÍNH XÁC cho phông đóng gói sẵn (`x` → `(9,17)` ở 16px) để giết `:497 + → -/*`. Bộ gọi lại phát từng điểm ảnh nên `h=1`, đột biến chỉ dịch biên MỘT điểm ảnh — khẳng định thô không thấy. Đổi phông thì phép thử gãy, và đó là điều MONG MUỐN. Ghi rõ `:491` và `:499` là tương đương, đừng đuổi
- [x] `do_o` 12 kẻ sống — cụm lớn nhất còn lại, chưa phân tích
- [ ] **Ghim bất biến "ô không bao giờ có toạ độ ÂM"** — `khong_o_nao_troi_ra_ngoai_anh` chỉ chặn biên TRÊN. Hai kẻ sống ở `hit_test_field:256` (`x < 0.0` → `==`, và `||` → `&&` vế đầu) chỉ đổi kết quả với toạ độ âm, và lập luận "tương đương" của tôi tựa vào giả định `trai ≥ 0` mà KHÔNG ai kiểm. Bẫy 10: kiểm được thì phải kiểm, đừng ghi "tương đương" dựa trên giả định
- [ ] **`do_o:601` `so_dong += 1` → `*=` vẫn sống** — khẳng định "hai dòng cao ít nhất 2× chiều cao dòng" LỌT, vì khi phép đếm đứng ở 0 thì `cao` rơi về `.max(chiều cao NÉT)`, mà nét của chuỗi nhiều dòng vẫn lớn. Nhánh `.max` che mất phép đếm. Điểm phân biệt đúng: với chuỗi nhiều dòng, `cao` phải LỚN HƠN HẲN chiều cao nét (hộp dòng có đệm), còn bản đột biến cho `cao` ĐÚNG BẰNG chiều cao nét
- [ ] `do_o` còn 9 kẻ số học khác (`:590` `DEM*2`, `:591`, `:596`, `:615`, `:633`) — khẳng định "khung rộng hơn thường" quá thô, `DEM+2` vẫn > 0 nên vẫn rộng hơn
- [ ] **`rong_bang_nhau` 9 kẻ sống — ở KHE HỞ, không ở bề rộng.** Phép thử của tôi khẳng định các ô rộng bằng nhau, mà dòng `d.o.rong = rong_nhat` không đột biến nào chạm nên nó luôn đúng. Thứ chúng đổi là CHỖ ĐẶT: `khe` sai thì ô chồng lên nhau hoặc dãn ra; `tong > rong_toi_da` sai thì luật áp cả khi không vừa. Điểm phân biệt: sau khi kéo bằng nhau, **khe giữa các ô kề nhau phải BẰNG NHAU và bằng khe ban đầu**
- [ ] `khung` 10 kẻ sống — số học biên khung, chưa phân tích
- [ ] `ve_o` 51 kẻ sống — cụm lớn nhất, chưa có bản vá nào nhắm tới
- [ ] **`chay_trong_vong` 17 kẻ — GIỚI HẠN TRỌNG TÀI, không phải mã không ai canh.** Hàm nhận `&mut tao::event_loop::EventLoop`, mà `tao` chỉ cho một vòng lặp mỗi tiến trình và `#[test]` không mở được cửa sổ. Dự án CÓ thứ chạm tới nó: `tools/kiem-khoi-ung-dung.sh` chạy chính nhị phân sản phẩm qua năm đường màn hình, kể cả móc `TCC_TU_DONG_DONG` nằm trong hàm này — nhưng nó không phải trọng tài của `cargo-mutants`. Cách đóng: cho `--test-tool`/kịch bản kiểm khói vào trọng tài, hoặc ghi rõ là giới hạn và thôi
- [ ] Chạy cổng số liệu đầy đủ sau khi lượt quét xong (số 399 phép thử hiện CHƯA được cổng kiểm)

- [ ] Quét đột biến `tcc-keystore` (đã mở khoá được), rồi dọn tên dịch vụ kiểm thử

## Ví — CHỐT 27/08: không bỏ được, phải hoàn thiện

- [ ] **Kho khoá Windows — CHỌN ĐƯỜNG TRƯỚC, đừng làm DPAPI theo quán tính.** DPAPI yếu hơn Keychain+`USER_PRESENCE` (không đòi hiện diện, blob nằm trong tệp, mọi tiến trình của người dùng giải mã được). Tương đương thật là Windows Hello + TPM. Ba đường ở `ke-hoach.md`; cần người bảo trì chọn
- [ ] Kho khoá cho **Linux** — hiện `NoKeystore`
- [ ] Quét đột biến `tcc-keystore` (đã xếp hàng chạy sau lượt raster)
- [x] **Lớp vỏ dòng lệnh của ví là tiếng Việt CỨNG** — `apps/tcc-browser/src/main.rs:148-176`. Màn hình trong cửa sổ CÓ song ngữ (nhận tham số `ngon_ngu`); chỉ mấy dòng `println!`/`eprintln!` quanh chúng thì không: "✓ đã khôi phục ví", "⚠ Bản cũ ở ví web VẪN CÒN", và `✗ {e}`. Trên Windows/Linux, dòng cuối ấy là TOÀN BỘ những gì người dùng đọc được về ví — và nó chỉ có tiếng Việt, trong khi luật là song ngữ mặc định tiếng Anh
- [ ] `SECURITY.md` §3.28: kho khoá macOS gần như chưa kiểm — thu hẹp khoảng trống ấy

- [ ] Câu lỗi `✗ {e}` vẫn là tiếng Việt cứng — nó đến từ `Display` của `WalletFlowError`/`WalletStoreError` (qua `thiserror`), không qua `text.rs`. Đưa `Display` của kiểu lỗi vào tầng song ngữ là một QUYẾT ĐỊNH THIẾT KẾ (kiểu lỗi còn dành cho người phát triển đọc), chưa làm

- [ ] **@NGƯỜI quyết: hàng ví CHỈ ĐỌC ĐỊA CHỈ có cần dấu phân biệt không?** Dấu cảnh báo hiện chỉ áp cho hàng KÝ GIAO DỊCH, và hàng ấy nay không dựng ra nữa. Nên hàng ví duy nhất người dùng còn thấy vẽ y hệt hàng quyền mạng. Đọc địa chỉ KHÔNG chuyển tiền (nên không đáng báo động), nhưng nó gắn danh tính với lịch sử trên chuỗi — một mức riêng, chưa có hình dạng riêng
- [ ] Hai bất biến tạm thời KHÔNG VỚI TỚI ĐƯỢC vì ví thu về chỉ-đọc: câu "việc này chuyển tiền" trên hàng ký, và khung đôi B31 ở màn xác nhận giao dịch. Mã và chuỗi vẫn còn, chỉ đường tới là đóng. Ghi vào `SECURITY.md` để người soát không tưởng chúng đang được canh

## Cần NGƯỜI, không phải mã

- [ ] @NGƯỜI Một buổi đọc màn hình thật bằng VoiceOver
- [ ] @NGƯỜI Một cuộc kiểm định an ninh ĐỘC LẬP — cổng chặn mainnet phụ thuộc nó
- [ ] @NGƯỜI Một lượt soát `ttf-parser`
- [ ] @NGƯỜI Người bảo trì quyết ba mã lỗi chưa định nghĩa (`de-nghi-ma-loi-thieu.md`)
