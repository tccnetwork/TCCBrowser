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

> ⚠️ **`kiem-so-lieu.sh` ĐÃ chạy `cargo test --workspace` bên trong nó** (để
> đếm số phép thử). Chạy `cargo test --workspace` riêng rồi lại chạy nó là chạy
> cả bộ thử HAI LẦN cho mỗi lượt kiểm — 26/08/2026 tôi làm thế khoảng mười lăm
> lần trong một phiên. Cần biết đỏ/xanh thì đọc mã thoát của chính cổng ấy.
>
> Đừng "tăng tốc" nó bằng `cargo test -- --list`: `--list` đếm CẢ phép thử
> `#[ignore]`, còn dòng `test result: ok. N` thì không — đổi cách đếm là mở
> đúng hạng lỗi mà cổng này sinh ra để chặn.

```bash
cargo build --workspace        # RẺ NHẤT, chạy TRƯỚC
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
tools/kiem-luat-phu-thuoc.sh   # 24 luật, chạy trước biên dịch trong CI
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
- **Thăm dò một API thì `cargo check --all-targets`, không phải `cargo build`.**
  26/08/2026 tôi viết một `#[cfg(test)]` gọi `tao::clipboard::Clipboard` rồi
  chạy `cargo build`, thấy xanh và tin là API ấy có. `cargo build` KHÔNG biên
  dịch mô-đun test — phép thăm dò chưa hề chạm tới thứ nó đang hỏi.
- **Khi kiểm đột biến bằng tay, phân biệt LỖI BIÊN DỊCH với PHÉP THỬ ĐỎ.**
  25/08/2026 tôi đếm dòng `test result: FAILED` và báo một đột biến là "sống"
  — trong khi nó chỉ không biên dịch được, tức là phép đo vô hiệu chứ không
  phải phép thử yếu. Sửa xong lại đếm `^error` và báo "không biên dịch được"
  cho một lượt mà `error: test failed` chính là **phép thử đỏ**. Đếm
  `^error\[E` và `could not compile` cho lỗi biên dịch, `test result: FAILED`
  cho phép thử đỏ; hai con số, không phải một.
- **Bỏ qua một phép thử thì dùng `#[ignore]`, ĐỪNG trả về sớm.** Trả về sớm khi
  thiếu một biến môi trường là một phép thử XANH GIẢ — dòng kết quả ghi
  "ok, 10 passed" y hệt lúc nó chạy thật. `#[ignore]` thì cargo đếm ra
  "9 passed; 1 ignored", và người đọc thấy ngay có thứ chưa chạy.
- **Phép thử chạm Keychain THẬT có thể treo cổng vô hạn.** 26/08/2026
  `kiem-theo-co.sh` đứng hơn bốn mươi phút ở `-p tcc-keystore --features
  os-keystore`: phép thử ghi một mục thật rồi gọi `unlock`, macOS bật hộp thoại
  xin quyền, và cổng chờ một cú bấm không bao giờ tới. Nay mỗi lệnh chạy dưới
  một hạn giờ, và HẾT GIỜ được báo là hết giờ — không phải "phép thử đỏ".
- **`cargo-mutants` phải chạy VỚI CỜ mà mã sống sau đó.** Trọng tài mặc định là
  `cargo test --workspace`, và lệnh ấy không bật cờ nào — nên mã sau một cờ
  KHÔNG được biên dịch, phép thử của nó KHÔNG chạy, và công cụ ghi mọi mutant
  là "sống sót". 26/08/2026: `tcc-chain` báo **45 sống**; bật
  `--features import-web-wallet` thì con số thật là **25**. (Câu này ban đầu ghi
  thêm `os-keystore` — `tcc-chain` KHÔNG có cờ ấy, nó thuộc `tcc-shell` và
  `tcc-keystore`, nên lệnh chép nguyên văn sẽ lỗi. Sửa 26/08/2026. Dùng
  `tools/kiem-dot-bien.sh`, nó giữ sẵn cờ đúng cho từng hòm.) Hai
  mươi con số kia là "chưa nhìn tới" đội lốt "không bắt được".
- **`cargo-mutants`: `TIMEOUT` KHÔNG phải "sống sót".** 25/08/2026 lượt đo lại
  trả về 61 dòng `TIMEOUT`, 0 dòng `MISSED` — trông y hệt một bộ thử vô dụng.
  Đọc tới ĐUÔI mới thấy `No space left on device`: mỗi việc song song là một
  BẢN SAO cả cây kèm `target/` riêng, ~1,8 GB một cái, và `-j 4` ăn hết chỗ
  trống. Mọi mutant "hết giờ" vì bản dựng chết, không vì phép thử yếu — cùng
  hạng lỗi với mọi thứ khác ở đây, chỉ đảo chiều: lần này "chưa chạy tới"
  trông giống "phép thử không bắt được".
  Trước khi tin một lượt `cargo-mutants`: xem `df -h`, và đọc đuôi đầu ra.
  Cây tạm KHÔNG tự dọn khi lượt chạy bị cắt. 27/08/2026 hai lượt tôi tự dừng để
  lại 3,7 GB xác, và đĩa tụt tới 9 GB giữa lượt thứ ba — suýt phải giết nó ở
  phút thứ 403.

  ⚠️ **Đừng `rm -rf /Volumes/DATA/.tmp/cargo-mutants-*` khi còn lượt đang
  chạy** — bản đang dùng cũng nằm trong đó. Phân biệt bằng HAI tín hiệu, và
  chúng KHÔNG nói cùng một điều:

  | Dấu hiệu | Kết quả hôm ấy |
  |---|---|
  | thời gian sửa cuối (`find -newermt`) | cả ba bản đều "im >3 phút" — **không phân biệt được gì** |
  | `lsof +D <thư mục>` | bản đang chạy có 4 tệp mở, hai bản kia có 0 — **dứt khoát** |

  Tin dấu hiệu đầu — cái tự nhiên nghĩ tới trước — là xoá đúng bản đang chạy.
  Luật chung: **khi hai tín hiệu bất đồng, tin cái ĐO TRẠNG THÁI THẬT (tệp đang
  mở), không tin cái SUY RA (im lặng = chết).**

  Dọn an toàn:
  ```bash
  for d in /Volumes/DATA/.tmp/cargo-mutants-*; do
    [ "$(lsof +D "$d" 2>/dev/null | wc -l)" -eq 0 ] && rm -rf "$d"
  done
  ```
- **ĐỪNG sửa mã trong lúc một cổng đang chạy.** 25/08/2026: `kiem-theo-co.sh`
  chạy 10 phút trong khi tôi sửa `tcc-crypto`, và có lúc cây không biên dịch
  được. Nó báo hỏng 3 lệnh — không phải vì mã hỏng mà vì nó đo một thứ đang
  động. Cùng hạng lỗi với mọi thứ khác ở đây: phép đo phải đo một thứ đứng yên.
- **Đừng cho đầu ra của cổng chạy nền qua `| tail`.** Cùng lần ấy, `tail -6`
  nuốt mất TÊN ba lệnh hỏng, chỉ còn lại con số — phải chạy lại cả 10 phút.
- **`zsh` KHÔNG tách từ tham số chưa trích dẫn.** `for c in "-p x --features y"; do cargo $c; done`
  truyền cả chuỗi làm MỘT đối số; `cargo` chết vì tên gói sai, và nếu vòng lặp
  chỉ `grep` tìm chữ "FAILED" thì nó báo XANH cho bốn bộ thử **chưa hề chạy**.
  Đo mã thoát, và dùng `"$@"`/`eval` thay vì `$c` trần.
- **Thêm phép kiểm mới thì phải KIỂM ĐỘT BIẾN nó.** Một phép thử chưa từng thấy
  đỏ không phải bằng chứng. Điều này áp cho cả 24 luật kiến trúc.

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
