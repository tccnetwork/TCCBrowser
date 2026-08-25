# Thử tay — mở ứng dụng lên và bấm

> Tài liệu này dành cho người muốn **dùng thử**, không phải người soát mã.
> Người soát vào [`AUDIT.md`](AUDIT.md).
>
> Mọi lệnh chạy từ thư mục `v2/`.

## Dựng một lần

```bash
cargo build -p tcc-browser --features wallet
```

`wallet` đã bao gồm `window`. Bản dựng **không** có cờ `wallet` thì không có
mục ví nào — và đó là bản dựng đúng cho tới khi có hồ sơ cấp phép Apple.

Nhị phân nằm ở `target/debug/tcc-browser`. Năm đường dưới đây đều đã chạy được
tính tới 25/08/2026.

> ⚠️ **Mọi tổ hợp cờ ghi đè cùng một tệp nhị phân.** Chạy `cargo build`
> hay `cargo test` với cờ khác — kể cả `tools/kiem-theo-co.sh` — là
> `target/debug/tcc-browser` bị thay bằng bản KHÔNG có ví, và lệnh ở mục 5 sẽ
> báo *"bản dựng này KHÔNG có ví"*. Gặp câu ấy thì dựng lại đúng dòng ở trên.
> (Tôi dẫm đúng bẫy này lúc soạn tài liệu, nên nó đứng ngay đây.)

---

## 1. Đường đầy đủ — gói đã ký → hỏi quyền → màn ứng dụng

```bash
./target/debug/tcc-browser examples/hello-tcc
```

Sẽ thấy, theo thứ tự:

1. **Hộp thoại hỏi quyền.** Ứng dụng ví dụ xin quyền **mạng** tới
   `example.com`. Trả lời **từng mục một** — không có nút "đồng ý tất cả", và
   đó là cố ý.
2. **Màn hình ứng dụng**, vẽ từ `ui.json` trong gói. Có một nút *"tải trang"*.

**Đáng để ý:** không chỗ nào nói "đã xác minh nhà phát hành". Chữ ký chứng minh
gói **không bị sửa** — nó không chứng minh **ai** ký, và 0.1 chưa có sổ khoá.
Nếu thấy chữ nào ngụ ý ngược lại, đó là lỗi, báo lại.

**Thử phá:** sửa một byte bất kỳ trong `examples/hello-tcc/content/ui.json` rồi
chạy lại. Phải bị chối trước khi có bất cứ thứ gì hiện lên.

## 2. Chỉ xem hộp thoại hỏi quyền

```bash
./target/debug/tcc-browser hop-thoai examples/hello-tcc
```

Dừng ngay sau hộp thoại, không mở ứng dụng. Tiện khi chỉ muốn soi cách trình
bày quyền.

## 3. Xem lại quyền đã cấp

```bash
./target/debug/tcc-browser quyen examples/hello-tcc
```

Những gì đã đồng ý ở bước 1 nằm ở đây, và **thu lại được**. Thu hồi có hiệu lực
ngay với cả bản quyền ứng dụng đang cầm trên tay — không phải đợi khởi động lại.

## 4. Khôi phục ví bằng cụm từ

```bash
./target/debug/tcc-browser vi cum-tu
```

Gõ 24 chữ hoặc hạt giống **thẳng vào cửa sổ**.

> ⚠️ **Đừng dán cụm từ hay khoá riêng vào cửa sổ trò chuyện với trợ lý** — nó sẽ
> nằm lại trong lịch sử phiên và nhật ký terminal. Gõ vào cửa sổ ứng dụng.

Màn hình nói rõ **phiên này không cất gì**: đóng cửa sổ là mất, đúng như thế.

## 5. Nhập ví từ ví web

```bash
./target/debug/tcc-browser vi nhap crates/tcc-chain/data/vi-web-mau.json
```

Tệp mẫu trong kho có **một** ví; PIN của nó là `matkhau-thu-nghiem`. Luồng:
chọn ví → hỏi PIN → mở khoá.

> ⚠️ Tệp mẫu ấy **không phải ví thật**, và không được dùng làm mẫu để bắt
> chước: muối và IV trong đó **cố định** để bản dựng lặp lại được, còn bản ghi
> thật của ví web dùng `crypto.getRandomValues`. Cố định muối chỉ chấp nhận
> được trong một tệp công khai không giữ tiền của ai.

Nhập xong, ứng dụng nói thẳng hai điều mà phần lớn phần mềm không nói:

- bản cũ **vẫn còn** ở ví web, vẫn khoá bằng đúng mã PIN cũ;
- **tệp vừa nhập vẫn đang giữ khoá của bạn** — xoá nó đi.

---

## Đổi sang tiếng Việt

Thêm `vi` vào cuối bất kỳ lệnh nào:

```bash
./target/debug/tcc-browser examples/hello-tcc vi
```

Mặc định là **tiếng Anh**, vì trình duyệt phát cả ra ngoài công ty.

## Nếu muốn chạy không cần ngồi bấm

Hai biến môi trường dành cho kiểm khói, và **chúng là quyền lực thật** — ai đặt
được chúng là tự trả lời thay người dùng:

```bash
TCC_TU_DONG_DONG=2 ./target/debug/tcc-browser examples/hello-tcc   # mỗi màn tự đóng sau 2s
TCC_TU_DONG_BAM="tu-choi" ./target/debug/tcc-browser hop-thoai examples/hello-tcc
```

`tools/kiem-khoi-ung-dung.sh` dùng chúng, và **cố ý bấm TỪ CHỐI**: một kiểm khói
tự đồng ý mọi quyền là kiểm khói dạy rằng đồng ý mới là mặc định.

## Chưa làm được

- **Không có giao dịch mainnet nào.** Cổng chặn cứng: chưa qua kiểm định an ninh
  độc lập thì không có giao dịch thật, không hạn chót nào ghi đè.
- Không mở được trang web bất kỳ — không còn bộ dựng web trong dự án này.
- Chưa thử với trình đọc màn hình thật. Trục trợ năng có mã và có phép thử,
  nhưng **chưa ai ngồi nghe nó đọc** — đó là việc cần người, không phải mã.
