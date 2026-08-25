# TCC v2 — luật vận hành

> Kho này CHỈ có v2 (lõi Rust + tiêu chuẩn TCC). Trình duyệt Electron v1 nằm ở
> nơi khác, đang tạm dừng, và không thuộc phạm vi ở đây.
>
> Mở phiên mới thì đọc [`docs/dang-lam-gi.md`](docs/dang-lam-gi.md) **trước** —
> đang làm tới đâu, còn vướng gì, bẫy nào đã dẫm phải. Người soát bên ngoài thì
> vào [`docs/AUDIT.md`](docs/AUDIT.md).

## Ba câu quyết định kiến trúc

Đọc trước, mọi thứ khác suy ra từ đây.

1. **Ứng dụng KHÔNG mang mã.** Điểm vào là **cây component khai báo**, không
   phải thẻ đánh dấu, không phải kịch bản.
2. **Quyền năng KHÔNG tồn tại cho tới khi được cấp.** Người dùng trả lời **từng
   mục một**.
3. **Chữ ký chứng minh gói KHÔNG BỊ SỬA — nó KHÔNG chứng minh AI ký.** Gói tự
   ký, 0.1 không có sổ khoá. **KHÔNG BAO GIỜ** hiện "đã xác minh nhà phát hành".

## Cổng chặn cứng

> **Không giao dịch mainnet nào trước khi qua kiểm định an ninh ĐỘC LẬP.**

Không hạn chót, không buổi trình diễn, không lần ra mắt nào ghi đè được. Xem
[`SECURITY.md`](SECURITY.md) §3.5 và [`spec/GOVERNANCE.md`](spec/GOVERNANCE.md) §5.

## Trước khi đẩy — theo ĐÚNG thứ tự này

```bash
cargo build --workspace        # RẺ NHẤT, chạy TRƯỚC
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
tools/kiem-luat-phu-thuoc.sh   # 22 luật, chạy trước biên dịch trong CI
cargo test --workspace
cargo run -p tcc-conformance
tools/kiem-so-lieu.sh          # số phép thử/vector ghi trong tài liệu có ĐÚNG không
tools/kiem-khoi-ung-dung.sh    # CHÍNH nhị phân sản phẩm, đi hết đường

tools/kiem-theo-co.sh          # MỌI tổ hợp cờ CI chạy, rút thẳng từ ci.yml (~10 phút)
```

`kiem-theo-co.sh` thay cho danh sách chép tay, 25/08/2026. Danh sách cũ có bốn
dòng và đã trôi ba đường: gọi `--test hai-bo-dung` (bộ thử xoá cùng WebView, gõ
vào là chết), ghi `-p tcc-shell --features window` hai lần, và bỏ hẳn
`accesskit`, `import-web-wallet`, `os-keystore`, `wallet` — bốn tổ hợp CI vẫn
chạy. Bộ rút tìm ra **20** lệnh. Một danh sách chép tay thì trôi; rút từ nguồn
thì không trôi được.

Vì sao nhóm này tồn tại (19/08/2026, sau lần CI đỏ thứ hai trong ngày): mã sau
một cờ mà `cargo test --workspace` không dựng thì **không tồn tại** đối với lượt
kiểm ở máy mình. Đo lại 25/08/2026 cho chắc: nhét một `assert!(false)` sau
`feature = "os-keystore"` thì `cargo test --workspace` vẫn XANH, còn
`cargo test -p tcc-keystore --features os-keystore` đỏ. Và `clippy` cũng phải
theo cờ, không chỉ một lượt workspace — 22/08/2026 CI đỏ vì một `.expect()`
trong phép thử sau cờ: mã sau cờ chỉ được `clippy` soi khi chính cờ ấy bật.

`kiem-so-lieu.sh` nằm trong danh sách này từ 19/08/2026, sau khi CI đỏ vì đúng
việc bỏ sót nó: thêm hai phép thử là ba tài liệu ghi sai con số. Nó chạy lâu
nhất nên dễ bị bỏ qua — mà một cổng chỉ chạy trong CI thì nó không phải cổng của
mình, nó là cổng của người khác.

Thứ tự này có lý do: tôi từng đẩy một bản **không biên dịch được** vì chỉ chạy
bộ kiểm định — mà bộ kiểm định dùng binary đã dựng sẵn nên im lặng hoàn toàn.
Phép kiểm rẻ nhất là phép bắt được nó.

## Thói quen dễ sai — đều đã trả giá

- **Sau mọi lần sửa mã bằng `perl`/`sed`, chạy `cargo build` NGAY.** `sed` của
  BSD (macOS) **không hiểu `\b`**; cả loạt lệnh đổi tên có thể im lặng không làm
  gì trong khi `git mv` vẫn chạy.
- **Khi kiểm đột biến, ĐỪNG lọc đầu ra.** Ba lần tôi `grep` hẹp rồi kết luận
  "không bắt được"; ba lần chạy lại không lọc thì thấy nó bắt được — hoặc thấy
  lỗi biên dịch mà `grep` đã giấu.
- **Đo bộ nhớ thì mỗi đường một TIẾN TRÌNH RIÊNG.** Bộ cấp phát không trả bộ
  nhớ về hệ điều hành ngay, nên đo hai đường trong một tiến trình cho ra con số
  của đường tốn nhiều hơn.
- **`$?` sau một pipeline là mã thoát của lệnh CUỐI**, không phải của lệnh bạn
  đang đo. Đo mã thoát thì đừng pipe.
- **Một phép kiểm không phân biệt được "chạy được" với "chưa chạy tới" thì
  không phải một phép kiểm.** Ba lần trong hai ngày: một phép thăm dò không cho
  `wrap` cơ hội xuống dòng rồi báo tính năng ấy "chết"; một `grep` tự quyết định
  tệp là nhị phân rồi im lặng trả về rỗng; và một lần "kiểm khói" ứng dụng —
  chạy 12 giây, thấy tiến trình còn sống, báo ĐẠT, trong khi nó mới đứng ở hộp
  thoại và chưa tới chỗ sập. Trước khi tin một phép đo, hỏi: **nếu thứ tôi sợ
  xảy ra thật, phép đo này có đổi kết quả không?**
- **`zsh` KHÔNG tách từ tham số chưa trích dẫn.** `for c in "-p x --features y"; do cargo $c; done`
  truyền cả chuỗi làm MỘT đối số; `cargo` chết vì tên gói sai, và nếu vòng lặp
  chỉ `grep` tìm chữ "FAILED" thì nó báo XANH cho bốn bộ thử **chưa hề chạy**.
  Đo mã thoát, và dùng `"$@"`/`eval` thay vì `$c` trần.
- **Thêm phép kiểm mới thì phải KIỂM ĐỘT BIẾN nó.** Một phép thử chưa từng thấy
  đỏ không phải bằng chứng. Điều này áp cho cả 22 luật kiến trúc.

## Ranh giới không được vượt

1. **Đặc tả rút ra từ mã ĐÃ CHẠY, không viết trước.** Luật 1 của
   [`spec/README.md`](spec/README.md).
2. **Mỗi điều khoản trong `spec/` phải có ít nhất một vector.** Luật 16 cưỡng chế.
3. **`tcc-spec` và `tcc-crypto` là crate LÁ.** Người ngoài cài đặt tiêu chuẩn chỉ
   cần `tcc-spec`, không phải kéo cả trình duyệt.
4. **`tcc-ui` không được biết bộ dựng nào.** Mất luật này là mất đường thoát khỏi
   WebView, và ngày có bộ dựng riêng thì mọi ứng dụng phải viết lại.
5. **Chữ ký lai KHÔNG BAO GIỜ tụt xuống một thuật toán.**
   [`spec/VERSIONING.md`](spec/VERSIONING.md) §4.

## Ngôn ngữ

- **Đặc tả: tiếng Anh là bản CHUẨN**, tiếng Việt là bản dịch, luật 11 chặn trôi.
- **Định danh `pub`: tiếng Anh.** Luật 13 cưỡng chế.
- **Chú thích, tên phép thử, biến cục bộ, `docs/`: tiếng Việt.** Chúng là *lập
  luận*, không phải giao diện — và `SECURITY.md` trích tên phép thử làm bằng
  chứng, nên đổi tên là phá đúng thứ chúng ghi lại.
- **Chú thích giải thích VÌ SAO, không giải thích CÁI GÌ.** Mã đã nói nó làm gì.
