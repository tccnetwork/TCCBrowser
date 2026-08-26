# v2 — đang làm tới đâu

> Ghi chú bàn giao giữa các phiên làm việc, viết bằng tiếng Việt vì đội ngũ bảo
> trì đọc tiếng Việt. Tài liệu dành cho **người soát bên ngoài** thì bằng tiếng
> Anh: [`AUDIT.md`](AUDIT.md) là đường vào, rồi
> [`../SECURITY.md`](../SECURITY.md) và [`../spec/`](../spec/).
>
> Cập nhật lần cuối: **26/08/2026**.

## Đứng ở đâu — 26/08/2026

Nhánh `giai-doan-3.1`, mọi cổng xanh.

**394 phép thử · 154 vector · 62 bất biến · 24 luật kiến trúc · 20 lệnh theo cờ
· bộ kiểm định tuân thủ ĐẠT.**

Phiên này làm **lượt kiểm đột biến đầy đủ đầu tiên của dự án**, chỗ trước đây
con số là **không**.

> ⚠️ Câu này thoạt đầu tôi viết là *"cả **chín** hòm đều đã có số đo"*. Trong
> `crates/` có **mười một** hòm. Tôi chép lại con số đã trôi trong
> `ARCHITECTURE.md` mà không đếm. Và tệ hơn: **bảng kết quả theo từng hòm chưa
> hề được ghi lại**, nên lời khẳng định "đầy đủ" ấy **không kiểm được** — kể cả
> bởi chính tôi. Một phép đo không có hồ sơ thì không lặp lại được, và không lặp
> lại được thì nó là một câu chuyện, không phải một phép đo. Lượt sau phải ghi
> bảng ấy ra. Kiểm đột biến trả lời đúng một
câu mà "bao nhiêu phép thử" không trả lời được: *sửa mã cho sai đi thì có phép
thử nào đỏ lên không?* Bộ thử xanh mà không đỏ khi mã hỏng thì nó không đo cái
nó tưởng nó đo.

Ra mười bốn bất biến mới (**B45–B62**) và bốn luật kiến trúc mới. Và ra khuyết
điểm thật, trong đó có mấy cái đáng ngại:

- **Tôi tự tay mở một lỗ dễ uốn chữ ký** khi sửa một câu báo lỗi cho đúng sự
  thật. Vector tuân thủ `them mot byte thua` bắt được. Nay độ dài chữ ký kiểm
  **bằng đúng**, không phải "đủ dài", và kiểm TRƯỚC khi cắt lát.
- **`Debug` che hạt giống ví không có phép thử nào canh.** Ai gỡ nó đi thì hạt
  giống rơi vào nhật ký, và mọi cổng vẫn xanh.
- **`bad-key` — một mã lỗi bản đặc tả tuyên bố là không thể xảy ra** — vẫn nằm
  trong mã. Đã gỡ hẳn: điểm Ed25519 sai giờ ra `bad-signature`, sai độ dài ra
  `bad-length`.
- **B4 khẳng định bốn đường thu hồi, chứng minh một.**
- **Trần nội dung 256 MiB chưa từng được thử.**
- **Bản nhị phân xuất xưởng KHÔNG có cầu trợ năng** — cờ `window` không kéo theo
  `window-tro-nang`. Luật 24 canh chỗ này.
- **Bản dựng KHÔNG có ví vẫn hỏi người dùng về tiền** (B62). Hộp xin quyền vẽ
  công tắc cho việc mà bản dựng ấy không có cách nào làm. Nay `cap_duoc()` chặn
  từ gốc: không có khả năng thì không hỏi, và trả `Deny`.

**Ba phép đo hỏng, cả ba đều được chẩn đoán chứ không được tin.** Đây là phần
đáng nhớ nhất của phiên:

1. `61 TIMEOUT / 0 MISSED` trông y như một bộ thử vô dụng. Nguyên nhân nằm ở
   **cuối** đầu ra: `No space left on device`. Nay mọi lượt theo dõi có chốt
   `df`.
2. `--timeout-multiplier` **đo nhầm thứ**: nhân thời gian bộ thử của riêng
   `tcc-spec` (~2 giây) rồi áp cho lượt chạy cả không gian làm việc (~70 giây).
   Thay bằng `--timeout 300` tuyệt đối.
3. Chạy `cargo-mutants` **không bật cờ** báo 45 kẻ sống sót trong hòm ví — vì
   `import.rs` nằm sau `import-web-wallet`, không hề được biên dịch. Con số thật
   khi bật cờ: 25.

Cả ba đều là cùng một hạng lỗi với ba việc của phiên 25/08: **phép đo không đo
thứ mình tưởng.** Một con số sai còn tệ hơn không có con số, vì nó ngăn người ta
đi tìm.

**Cổng số bất biến bắt chính người vừa dựng ra nó.** Thêm B62 xong quên sửa con
số trong `AUDIT.md`; cổng viết cách đó một giờ bắt ngay lần đầu có cơ hội. Đó là
dấu hiệu tốt nhất một cổng có thể cho — nó không phân biệt ai đang phạm luật.

**`kiem-theo-co.sh` báo đỏ mà không nói được đỏ vì gì.** Cả hai mươi lệnh cùng
ghi đè một tệp đầu ra, nên khi lệnh thứ mười một đỏ thì chín lệnh sau đã xoá
mất bằng chứng của nó — mà chạy tay đúng lệnh ấy thì xanh. Nay mỗi lệnh giữ đầu
ra riêng, và khi đỏ thì in đuôi đầy đủ kèm mã thoát, không lọc `^error` nữa
(lần này **không có** dòng nào bắt đầu bằng `error`, nên bộ lọc ấy in ra đúng
con số không).

### ⚠️ Chưa giải thích được — `cargo test -p tcc-shell --features window`

26/08/2026 lệnh này thoát khác 0 **một lần** trong cổng, rồi **xanh năm lần
liên tiếp** sau đó (một lượt cổng đầy đủ + bốn lượt lặp riêng). Không dòng nào
bắt đầu bằng `error`, không phép thử nào đỏ, và bằng chứng đã bị các lệnh sau
ghi đè mất.

**Chưa biết vì sao. Đừng coi là đã sửa.** Thứ đã sửa chắc chắn chỉ là việc cổng
vứt mất bằng chứng — lần sau nó tái diễn thì `/tmp/kiem-theo-co/<số>.txt` sẽ
còn nguyên đầu ra. Ai gặp lại: giữ tệp ấy lại rồi hẵng chạy lại.

### Tài liệu đã trôi khỏi mã — và trôi ở chỗ đắt nhất

Đổi thứ tự ưu tiên sang "trình duyệt trước" buộc phải đọc lại kế hoạch, và đọc
lại thì lộ ra một loạt câu **không thể đúng nữa** sau khi gỡ WebView. Chúng
không sai lúc viết; chúng hoá sai khi thứ chúng đối chiếu bị xoá — và **không
cổng nào bắt được hạng lỗi ấy**.

- **`SECURITY.md` dòng phạm vi kiểm định** kể tên `tcc-render-webview` (đã xoá)
  và **bỏ sót bốn hòm có thật**: `tcc-chain`, `tcc-keystore`, `tcc-net`,
  `tcc-render-raster`. Tức là mã nói chuyện với chuỗi, mã giữ khoá ví, và mã
  DUY NHẤT được mở socket đều nằm ngoài phạm vi người soát được trả tiền để
  nhìn. Đây là chỗ đắt nhất trong cả đợt.
- **`ARCHITECTURE.md` §4 khẳng định ở thì hiện tại** rằng bộ dựng WebView "được
  dựng và chạy trên WebKitGTK dưới Linux cũng như WKWebView trên macOS, trong
  CI". Không còn hòm ấy, không còn việc CI ấy. Người soát đọc đoạn đó sẽ tin
  tính thay-thế-được của bộ dựng **đã được chứng minh**. Nó không được chứng
  minh: `Renderer` nay có **đúng một** bản cài đặt sản xuất.
- **`ARCHITECTURE.md` ghi 9 hòm**, thật là 11.
- **`ke-hoach.md`** khoe `cargo tree` cho "0 crate `wry`" như bằng chứng đường
  thoát. Bằng chứng ấy nay **rỗng**: `wry` không còn là phụ thuộc của bất kỳ cờ
  nào, nên câu ấy đúng với mọi dự án Rust chưa từng đụng tới `wry`. Một phép
  thử không thể đỏ thì không phải phép thử.
- **`ke-hoach.md` mục "Quy đổi ra thời gian"** vẫn tựa cả lập luận khả thi vào
  *"kế hoạch này mượn WebView và không bắt đầu bằng bộ dựng hình"* — cả hai vế
  đều đã đảo ngược. Bảng Servo / Ladybird / Chromium vẫn đúng số, nhưng nay đọc
  theo chiều ngược lại.
- Hai xác chết của việc xoá bằng máy: câu *"cờ `window` tách khỏi cờ `window`"*
  trong kế hoạch, và trong `Cargo.toml` chú thích của dòng WebView **dính sang
  dòng `tcc-ui`**, khiến `tcc-ui` mang nhãn "bộ dựng #1: giàn giáo, sẽ tháo".
- `crates/tcc-render-webview/` còn lại một thư mục **rỗng**, đủ để `ls crates/`
  nói dối. Git không theo dõi thư mục rỗng nên không ai thấy.

Đã sửa hết, và **giữ nguyên văn câu cũ ở mỗi chỗ** thay vì xoá — người soát cần
thấy tài liệu đã từng nói gì với họ.

**Cổng mới, đã kiểm đột biến năm cách:** `kiem-so-lieu.sh` nay bắt danh sách hòm
trong `SECURITY.md` và số hòm trong `ARCHITECTURE.md` phải khớp `crates/`. Thử
bỏ một tên, thêm một tên ma, sai con số, sinh thêm một hòm thật — cả năm đều bị
bắt, và về nguyên trạng thì im lặng lại.

Thứ **chưa** có cổng, nói ra để không tưởng là đã xong: không gì bắt được một
câu văn xuôi hoá rỗng nghĩa vì thứ nó đối chiếu bị xoá. Đó là việc của người
đọc, và nó chỉ xảy ra khi có ai thật sự đọc lại.

### ⚑ Thứ tự đã chốt: TRÌNH DUYỆT TRƯỚC, VÍ SAU

Chủ dự án chốt 26/08/2026: **vẫn tích hợp ví vào trình duyệt, nhưng làm trình
duyệt trước.** Chi tiết ở [`ke-hoach.md`](ke-hoach.md). Việc ví không bị bỏ, chỉ
bị xếp sau — và ràng buộc an ninh vẫn nguyên: **không một giao dịch mainnet nào
trước khi qua kiểm định an ninh ĐỘC LẬP.**

### Còn treo — cần NGƯỜI, không phải mã

- Một buổi đọc màn hình thật bằng VoiceOver.
- Một cuộc kiểm định an ninh độc lập.
- Một lượt soát `ttf-parser`.
- Ba mã lỗi chưa được định nghĩa, và khoảng trống "bản dựng này không cung cấp
  khả năng ấy" trong bản đặc tả — đề nghị nằm ở
  [`de-nghi-ma-loi-thieu.md`](de-nghi-ma-loi-thieu.md), chờ người bảo trì quyết.

Vẫn **chưa chứng minh được**, và nói to ra ở đây để không ai tưởng là đã xong:
`HttpNetwork::get` và `JsonRpc::call` (cần một máy chủ TLS trong phép thử), và
gần hết `tcc-keystore/src/macos.rs` (cần một Keychain tạm).

## Đứng ở đâu — 25/08/2026

Nhánh `giai-doan-3.1`, mọi cổng xanh.

**394 phép thử · 154 vector · 24 luật kiến trúc · 20 lệnh theo cờ · bộ kiểm định
tuân thủ ĐẠT.**

Phiên này đi soát lại ~20 phép thử màn hình đã viết lại lúc gỡ WebView. Ba việc
ra khỏi đó, và cả ba đều cùng một hạng lỗi: **phép đo không phân biệt được thứ
tôi quan tâm với thứ nằm cạnh nó.**

1. **Chín phép thử hỏi "có dấu cảnh báo ở ĐÂU ĐÓ không"** thay vì hỏi câu nào
   mang dấu. `cau_chuyen_tien_duoc_ve_khac_di` — phép thử tồn tại để chứng minh
   câu *"việc này chuyển tiền"* nổi hơn xung quanh — **vẫn xanh** khi dời dấu
   sang dòng khác. Thêm `do_cay::co_canh_bao(cay, cau)`, đổi cả chín chỗ, kiểm
   đột biến bằng cách **dời** dấu chứ không xoá. B45, SECURITY.md §3.15.

2. **Lỗi thật nấp sau phép thử yếu ấy.** Màn hồi phục **hỏng** đặt dấu trên dòng
   chẩn đoán, còn câu người dùng phải hành động theo — *chưa cất được gì, khoá
   mất rồi* — thì là chữ thường giữa bốn dòng khác. Màn anh em đánh dấu đúng câu
   cùng loại. Phép thử tên `man_hong_noi_ro_khong_luu_gi` đã xanh suốt vì dấu
   trên dòng chẩn đoán trả lời được câu hỏi nó thực sự đặt ra. Đã sửa.

3. **Danh sách "lệnh theo cờ" chép tay trong CLAUDE.md đã trôi khỏi CI** — gọi
   một bộ thử đã xoá, ghi trùng một dòng, thiếu bốn tổ hợp. Thay bằng
   `tools/kiem-theo-co.sh`, rút thẳng từ `ci.yml`: **20** lệnh, không phải 4.
   Và trong lúc đo, chính vòng lặp `for` tôi viết để kiểm cũng dính bẫy: `zsh`
   không tách từ `$c` chưa trích dẫn, nên bốn bộ thử **chưa hề chạy** mà vòng
   lặp báo xanh cả bốn.

Hai cổng mới, cả hai đã kiểm đột biến: `kiem-so-lieu.sh` giờ kiểm tên `--test`/
`--bin` (trước chỉ kiểm gói và cờ, nên mù với `--test hai-bo-dung`), và kiểm con
số 20 kia khỏi trôi.

**B24 khẳng định ba việc, chứng minh một.** Hàng ấy ghi "chỉ HTTPS, có hạn
giờ, có trần" và cột bằng chứng chỉ có hai chữ `tcc-net`. HTTPS thì bốn phép
thử thuần lo đủ; hai việc kia **không phép thử nào** chạm tới. Soát ra ba thứ:

- `ureq` **báo lỗi** chứ không cắt cụt im lặng khi vượt trần (đọc
  `body/limit.rs:22-24`, không nhớ) — nhưng nó lỗi ở lần đọc SAU khi hết hạn
  ngạch, nên thân đúng bằng `MAX_BYTES` bị **từ chối**. Trần là "nhỏ hơn ngặt",
  không phải "nhỏ hơn hoặc bằng". Lệch về phía an toàn nên giữ, tài liệu sửa.
- **Hai chỗ đọc thân ánh xạ lỗi sai, theo hai hướng ngược nhau.** `lib.rs` nuốt
  mọi lỗi đọc thành "quá lớn" (đứt mạng cũng báo tệp to); `rpc.rs` làm mất tín
  hiệu "quá lớn" vào câu "gọi thất bại". Nay một hàm thuần chung, và `TooLarge`
  mang trần CỦA CHÍNH KÊNH ẤY (8 MiB gói, 1 MiB RPC).
- Độ hợp lý của hai hằng chuyển thành `const { assert! }` — đặt trần về 0 thì
  **hỏng bản dựng**, to hơn một phép thử đỏ. Clippy chỉ ra chỗ này khi tôi viết
  nó thành `#[test]`.

Vẫn **chưa chứng minh được**: `ureq` có thật sự tôn trọng hạn giờ không, và trần
có nổ trước một máy chủ thù địch thật không — cần một máy chủ TLS đứng trong
phép thử. 15/45 hàng B còn lại chưa trích phép thử nào; mười trong số đó là
khẳng định cấu trúc nên đúng, năm đã nghỉ hưu.

**Chạy `cargo-mutants` lần đầu — 34/132 đột biến sống sót ở `tcc-spec` và
`tcc-crypto`.** Đọc kỹ thì tách làm hai loại rất khác nhau:

- **Sáu cái là lỗ thật, và đều nằm ở ranh giới của chính tiêu chuẩn.** Đổi `>`
  thành `>=` ở tên máy chủ (253), nhãn (63), mã ứng dụng (128), số dấu chồng —
  không phép thử nào đỏ. `spec/0.1` ghi dải **bao gồm hai đầu**: "1–253
  characters, each label 1–63". Phép thử cũ toàn dùng tên ngắn hợp lệ hoặc tên
  hỏng-hình-dạng; không cái nào đứng ở mép. Bản cài đặt thứ hai đọc đặc tả sẽ
  nhận 253, bản này sẽ chối — bất đồng ở đúng chỗ tiêu chuẩn nói rõ nhất. Đã
  thêm phép thử ranh giới, mỗi cái kiểm lại bằng chính đột biến từng sống.
- **Phần còn lại là lỗi của PHƯƠNG PHÁP, không phải của mã.** `cargo-mutants`
  lấy `cargo test` làm trọng tài, mà `tcc-conformance` là một `main.rs` **không
  có `#[test]` nào**. 154 vector chỉ chạy khi ai đó gõ `cargo run`. Chúng CÓ
  chạy — trong CI và trong danh sách trước-khi-đẩy — nhưng đứng ngoài tầm nhìn
  của trọng tài, nên mọi đột biến mà chỉ vector bắt được đều bị báo "sống". Đổi
  `SpecError::ma` thành `"xyzzy"` — chính những mã lỗi cả tiêu chuẩn so khớp
  bằng — mà `cargo test --workspace` vẫn XANH.

Bộ kiểm định nay tách thành thư viện + vỏ dòng lệnh, chín phép thử trong
`tests/tuan-thu.rs` (một cho mỗi nhóm, không gộp: gộp thì trượt ở đâu cũng chỉ
một dòng đỏ và mất thứ đáng giá nhất — nhóm nào). `cargo test --workspace` giờ
chạy vector, và đúng đột biến ấy đỏ.

**Một mã lỗi tiêu chuẩn nói là KHÔNG THỂ tồn tại, bản này bắn ra thật.** Kéo
tiếp sợi chỉ đột biến: `CryptoError::ma` không giết được vì `bad-key` không nằm
trong vector nào, và nó không nằm trong vector nào vì `06-error-codes.md` xếp
nó vào "ba mã đã gỡ vì không thể xảy ra". Lý lẽ khi gỡ: "thư viện Ed25519
thường kiểm điểm LƯỜI nên khoá không giải nén được hiện ra thành
`bad-signature`". `ed25519-dalek` 3.0 kiểm NGAY trong `from_bytes` — 32 byte
`0x7f` cho ra `bad-key`. Đã đưa về `bad-signature` theo đặc tả; sáu chỗ `BadKey`
còn lại đều là sai độ dài nên dùng `bad-length` (mã có thật). Gỡ hẳn biến thể.

Thêm **luật 10b** hỏi chiều ngược của luật 10 (mã nguồn → đặc tả). Nó bắt ngay
ba mã nữa đặc tả không định nghĩa: `symlink`, `package-too-large`, `bad-scroll`
— hai cái ở tầng gói. Sửa một đặc tả ĐÃ CÔNG BỐ không phải việc của một lượt
dọn mã (`GOVERNANCE.md` §4 đòi nêu ai vỡ và kiểm bằng vector nào), nên viết
thành đề nghị: [`de-nghi-ma-loi-thieu.md`](de-nghi-ma-loi-thieu.md). Cổng **kê
tên chúng ra mỗi lượt chạy**, không miễn trừ im lặng.

Gỡ `bad-key` khỏi mã lại làm **luật 10 đỏ**: nó quét cả bảng "ba mã đã gỡ" nên
đòi mã nguồn phải có đúng cái mã đặc tả vừa rút. Bản Python của luật 16 cắt
đúng chỗ ấy từ đầu, bản shell thì không — và lỗi ấy nằm im được đúng chừng nào
bản cài đặt còn sai theo chiều bù lại.

**Luật 13 bỏ sót vì máy dò yếu.** Nó khớp theo ranh giới `_` nên tên CamelCase
lọt sạch, và không quét biến thể enum. Siết lại thì lộ 87 định danh — nhưng 73
là khoá chuỗi giao diện của `tcc-shell`, không phải "bề mặt người viết bản cài
đặt thứ hai đọc" như chính luật tự định nghĩa. Thu phạm vi về sáu crate làm nên
tiêu chuẩn thì còn **bốn**: `Goi`, `LienKetMem`, `ThieuTep`, `ChuKySaiDoDai`.

**Đối chiếu hết 34 đột biến, không bỏ sót cái nào.** Phân loại xong thì ra bốn
nhóm — và chỉ một nhóm là lỗ chưa vá:

- **Đã chết nhờ phép thử ranh giới**: sáu cái ở `check_host`, `AppId::parse`,
  `check_display_text`. Cộng thêm `check_path` (trần 1024) vá nốt hôm nay —
  đặc tả ghi "at most 1024", bao gồm.
- **Đã chết nhờ đưa vector vào `cargo test`**: `SpecError::ma`, `TreeError::ma`
  (ba trong bốn mã có vector; `duplicate-path` là miễn trừ đã ghi lý do —
  một đối tượng JSON không thể có hai khoá trùng), `CryptoError::ma`, và cả ba
  cái `ContentHasher` — nhóm `canonical` gọi nó trực tiếp.
- **Đã chết nhờ phép thử bộ đọc**: `FileTree::is_empty` / `paths` / `len`. Lý
  do chúng sống không phải phép thử yếu mà là **không ai gọi**.
- **Cố ý KHÔNG ghim**: năm cái `+` trong `hybrid.rs` chỉ đi vào con số "chờ bao
  nhiêu byte" trong *thông báo lỗi*. Chính dự án quy định thông báo là văn xuôi
  được phép sửa, chỉ MÃ mới ổn định — ghim văn xuôi lại là mâu thuẫn với luật
  ấy. Ghi ra đây để lần sau không ai tưởng là bỏ sót.

**Quét thêm một hạng cùng loại: hàm `pub` không ai gọi.** Ở sáu crate làm nên
tiêu chuẩn còn đúng hai: `NetworkCapability::hosts` (đã ghim — phải đi qua
`grant`, vì kiểu này CỐ Ý không dựng được từ ngoài) và `VerifiedApp::copy_content`
— **đã gỡ**. Chú thích của nó nói rõ nó tồn tại "để trao cho trình phục vụ tệp
của bộ dựng", mà trình phục vụ ấy bị xoá cùng bộ dựng web hai ngày trước. Nó
phát ra bản sao CẢ CÂY đã ký, rộng hơn `read(path)` đang dùng, và không ai gọi.

**Đo lại sau khi vá.** Cùng hai tệp `tcc-spec/src/{lib,tree}.rs`: lượt đầu để
lại **22 mutant sống + 3 chưa phán được**; sau khi thêm phép thử ranh giới và
đưa vector vào `cargo test` → **98 mutant, 96 bị bắt, 2 không dựng được, 0
sống, 0 hết giờ**.

Mất ba lượt mới ra con số ấy, và hai lượt đầu **không phải kết quả**:

- Lượt 2: 61 `TIMEOUT`, **0** `MISSED` — nhìn lướt y hệt "bộ thử chẳng bắt được
  gì". Nguyên nhân nằm ở đuôi đầu ra: `No space left on device`. Mỗi việc song
  song là một bản sao cả cây kèm `target/` riêng (~1,8 GB), `-j 4` ăn hết ổ.
- Lượt 3, ổ còn rộng, vẫn 39 `TIMEOUT`: `--timeout-multiplier 10` nhân với thời
  gian chạy phép thử của RIÊNG `tcc-spec` (~2 giây) rồi áp hạn 20 giây ấy cho
  các lượt chạy CẢ WORKSPACE (~70 giây). Đo một thứ, áp cho một thứ khác.

Cả hai lượt đều không phải "phép thử yếu", và `TIMEOUT` là **chưa phán được**,
không phải "sống". Cùng hạng lỗi với mọi thứ hôm nay, chỉ đảo chiều: mọi lần
trước "chưa chạy tới" trông giống ĐẠT; lần này nó trông giống KHÔNG BẮT ĐƯỢC.

**Bộ vector bắt được một chữ ký DẺO — do chính tôi vừa tạo ra (B48).**

Đo đột biến `tcc-crypto`: 34 mutant, 26 bị bắt, 7 sống. Cả 7 là nhóm `+` trong
phép tính `want` — đọc thân `take` thì `total` chỉ đi vào trường `expected` của
thông báo, còn quyết định cắt lát dùng `at..at + len`, nên hành vi không đổi.

Vẫn ghim, và lý do phải tách bạch với luật "thông báo là văn xuôi được phép
sửa": đây không phải câu chữ mà là một CON SỐ. Báo "chờ 3968 byte" trong khi
con số thật là 1984 là câu SAI SỰ THẬT, người đọc sẽ đi làm một khoá dài bằng
một con số không tồn tại.

Viết phép thử ấy thì lòi ra chỗ tệ hơn. Với CHỮ KÝ, độ dài chờ đợi được suy ra
từ chính đầu vào đang bị nghi (`signature.len().saturating_sub(64)`), nên chữ
ký 10 byte bị báo là "chờ 64" trong khi con số thật là 3373. Thay bằng hằng của
thuật toán làm câu lỗi đúng — và **mở ra chữ ký dẻo**: cách suy cũ đang lặng lẽ
gánh việc thứ hai là ÉP TỔNG ĐỘ DÀI KHỚP CHÍNH XÁC (thừa một byte thì nửa hậu
lượng tử dài 3310 và `Signature::try_from` chối). Bỏ nó đi thì `take` cắt đủ
3309 byte, byte thừa BỊ BỎ QUA, và cùng một thông điệp có vô số chữ ký hợp lệ.

Vector `them mot byte thua` đỏ ngay khi bản sửa biên dịch xong — và nó đỏ dưới
`cargo test --workspace`, điều mà SÁNG CÙNG NGÀY còn chưa làm được. Lập luận
cho việc đưa vector vào bộ thử lúc viết còn là lý thuyết; nó thôi là lý thuyết
trong cùng một ngày.

Sửa bằng phép kiểm `!=` trên tổng độ dài TRƯỚC khi cắt. Ghim cả hai chiều ở
tầng đơn vị để lần sau không phải chạy cả bộ kiểm định mới biết. Kiểm đột biến:
nới `!=` thành `<` → đỏ; xoá hẳn phép kiểm → đỏ.

**Bài học không phải "đừng sửa mã mật mã".** Là: một dòng có thể đang gánh hai
việc mà chỉ một việc được ghi lại. Khi thay nó, hỏi việc thứ hai là gì — và nếu
không tìm ra, đó chưa phải bằng chứng rằng không có.

**Miễn trừ không phải tha bổng.** Luật 16 tha `duplicate-path` khỏi cần vector
(một đối tượng JSON không thể có hai khoá trùng — đúng), nhưng rồi không gì đòi
một phép thử ghim MÃ ấy. Phép thử `chan_duong_dan_trung` có tồn tại và khẳng
định BIẾN THỂ, mà tên biến thể là chuyện nội bộ; bản cài đặt thứ hai so khớp
bằng mã. Nay luật đòi phép thử ghim mã, và phép thử đã ghim.

**Đo `tcc-manifest` + `tcc-capability` (chưa từng chạy đột biến lần nào): 35
mutant, 26 bị bắt, 2 sống.** Cả hai đều là lỗ thật, và một trong hai nằm dưới
một hàng B đã tuyên bố là có bằng chứng:

- `WalletCapability::allow_read_address` thay CẢ THÂN bằng `Ok(())` mà không
  phép thử nào đỏ. Hàm ấy chỉ làm đúng một việc — hỏi quyền còn sống không —
  nên "thay cả thân vẫn xanh" nghĩa là việc duy nhất ấy chưa từng được kiểm.
  Quyền ví ĐÃ THU HỒI vẫn đọc được địa chỉ. B4 chỉ có bằng chứng cho đường
  MẠNG.
- Trần bản kê khai 64 KiB: `>` thành `>=` không ai đỏ. Đặc tả ghi "at most 64
  KiB" — bao gồm. Cùng hạng sáu ranh giới đã vá ở `tcc-spec`.

**Và một lỗ thứ ba mà CÔNG CỤ KHÔNG TÌM RA.** `StorageCapability::allow_write`
cũng chỉ canh thu hồi bằng một dòng, nhưng thân nó còn phép kiểm hạn mức nên
`-> Ok(())` bị bắt. Phải tự tay gỡ đúng dòng `life.touch()?` mới thấy: mọi phép
thử vẫn xanh. Danh mục đột biến của `cargo-mutants` là hữu hạn, và **im lặng
của nó không phải bằng chứng** — đúng câu tài liệu này viết đi viết lại về mọi
phép đo khác.

Nay một phép thử duy nhất đi qua **cả bốn** lối vào sau khi thu hồi, không phải
một đường mẫu: thêm quyền năng mới mà quên thu hồi thì chỗ ấy là nơi phải sửa.
Gỡ từng dòng canh một để chắc phép thử nhìn thấy — bốn lần, bốn lần đỏ. B49.

**Chữ ký dẻo còn một bậc nữa: KHOÁ.** `take` chỉ hỏi "có đủ byte ở khoảng này
không", nên khoá công khai thừa byte vẫn cắt đủ 1984 byte đầu và vẫn xác minh
được. Đường sản phẩm không hở (tầng bản kê khai kiểm độ dài hex), nhưng
`tcc-crypto` là crate LÁ mà bản cài đặt thứ hai dùng trực tiếp. Nay một hàm
thuần `dung_do_dai` canh cả ba đường — khoá bí mật, khoá công khai, chữ ký — và
có vector `publisher key is one byte too long` đứng cạnh ca 200-byte đã có: hai
ca là hai đầu của cùng một bất biến, mà trước đó chỉ đầu NGẮN được thử.

**Cửa sổ trước 25/08/2026 KHÔNG dùng được bằng bàn phím.** Khung xử lý đúng
hai thứ: chữ, và `Backspace`. Không `Tab`, không `Enter`, không viền tiêu điểm.
Mọi nút và mọi ô chỉ với tới được **bằng chuột**.

Đó là lỗi trợ năng, nhưng nó cũng là lỗi an ninh — và vế thứ hai mới là lý do
nó nằm trong SECURITY.md: người không dùng được chuột thì **không trả lời được
hộp thoại quyền**, mà một hộp thoại không trả lời được là một hộp thoại sẽ được
người khác trả lời hộ.

Hai luật của đường bàn phím mới, cùng lý lẽ với luật đã có cho chuột:

- **`Enter` trong ô nhập không kích hoạt gì.** Trên hộp thoại quyền, "nút gần
  nhất" có thể là *Cho phép*.
- **`Enter` trên công tắc gạt công tắc và Ở LẠI màn hình**, y như bấm chuột.

Viền tiêu điểm kẻ **bên ngoài** ô, chừa khe hở, vì nút mất mát đã dùng **khung
đôi phía trong** (B31) — vẽ chồng vào trong là gộp hai tín hiệu khác nhau
("nguy hiểm" và "đang chọn") thành một hình. Bộ dựng chỉ có một kênh mực, nên
tín hiệu phải là hình dạng, không bao giờ là màu.

Chuột nay cũng đặt tiêu điểm: trước đó bấm chuột rồi `Tab` là nhảy về đầu —
hai lối vào nhìn thấy hai giao diện khác nhau.

**Và cửa sổ không kéo đổi cỡ được.** Chiều rộng bố cục là hằng biên dịch, nên
kéo rộng ra thì chữ vẫn xuống dòng ở cột cũ, phần thừa là dải trắng. Nay là
thuộc tính của bộ dựng, kẹp trong khoảng dùng được, và `Resized` xếp lại bố
cục. Kẹp không phải để đẹp: kéo cửa sổ gần khép lại là thao tác bình thường, nó
không được thành một ảnh rộng 0.

**Bản sản phẩm KHÔNG hề có cầu trợ năng.** Adapter AccessKit nối từ
19/08/2026 và tài liệu vẫn trích dẫn nó từ đó. Điều không ai kiểm: `tcc-browser`
— crate thành cái nhị phân người ta chạy — **không có đường nào** bật cờ
`window-tro-nang`. Mọi bản từng chạy đều có cửa sổ và không có cây trợ năng;
một trình đọc màn hình chĩa vào đó chỉ thấy một mảng điểm ảnh.

Không phải một quyết định. Cờ ấy tồn tại để `cargo test` khỏi kéo ba adapter
nền, rồi bản sản phẩm đơn giản là không được nối lại. Nay cờ `window` của ứng
dụng kéo theo nó, và **luật 24** làm đỏ nếu ai tách ra lần nữa.

Lặp lại gần như y hệt bài học của bộ vector: ở đó vector có thật nhưng trọng
tài không nhìn thấy; ở đây cầu trợ năng có thật nhưng **thứ xuất xưởng không
mang nó**. Cả hai lần, việc đã làm, tài liệu đã trích, và cái thực sự đến tay
người dùng thì không có.

**Dấu nháy trong ô nhập** — ô nhập không nháy trông y hệt ô chữ có khung. Vẽ ở
CUỐI chữ vì khung chỉ nhận thêm ở cuối; di chuyển con trỏ giữa chuỗi chưa có, và
vẽ một dấu nháy ở chỗ không gõ được là nói dối.

⚠️ Phép thử đầu tiên tôi viết cho dấu nháy **vô nghĩa**: nó so mực giữa "chọn ô
nhập" và "chọn nút", mà hai ô to nhỏ khác nhau nên viền đã khác mực sẵn — kiểm
đột biến bỏ hẳn phần vẽ nháy mà nó vẫn xanh. Bản sau soi ĐÚNG CỘT nơi nháy phải
nằm.

**Chữ chỉ sửa được ở CUỐI chuỗi.** Gõ và `Backspace` tác động vào cuối; không
có con trỏ để dời, nên một lỗi ở giữa chỉ với tới được bằng cách xoá sạch mọi
thứ phía sau nó.

Màn hình chịu hậu quả nặng nhất là **khôi phục ví — hai mươi bốn chữ**. Một lỗi
đánh máy ở chữ thứ ba bắt gõ lại hai mươi mốt chữ. Người gặp cảnh ấy sẽ **dán**
— mà dán cụm từ khôi phục đúng là thói quen dự án này bảo người ta đừng tập.

Nay sửa tại con trỏ, có `←` `→` `Home` `End` `Delete`. Ba điều đáng nói vì đó là
chỗ loại mã này hay sai:

- **Cắt theo CHỮ, không theo byte.** `ế` là một ký tự và ba byte. Cắt byte để
  lại chuỗi không còn là UTF-8 hợp lệ; "xoá một dấu" để lại `ê` hay `e` là đổi
  thứ người dùng đã gõ.
- **Vào ô thì con trỏ ở CUỐI** chữ đang có. Ở đầu thì chữ gõ tiếp nhảy lên
  trước mọi thứ — với một cụm từ khôi phục, đó là hỏng cả cụm.
- **Con trỏ vẽ ĐÚNG chỗ nó đang đứng.** Đứng sai chỗ còn tệ hơn không có: nó
  chỉ vào nơi chữ KHÔNG rơi vào.

⚠️ Phép thử cho việc này phải viết **ba lần**, và tôi ghi cả hai lần hỏng vào
chú thích của nó. Lần đầu so mực giữa ô nhập và nút — vô nghĩa, vì hai ô to nhỏ
khác nhau nên viền của chúng khác mực sẵn; bỏ hẳn phần vẽ nháy mà nó vẫn xanh.
Lần hai dò một cột cố định, và vỡ ngay khi con trỏ đi lại được.

**Và `doi()` không xoá tiêu điểm trong bộ dựng.** `TrangThai::default()` nói
"không ai đang được chọn", nhưng bộ dựng giữ bản sao riêng để vẽ. Đổi màn xong,
viền có thể rơi vào một nút người dùng chưa hề chạm tới — nếu màn mới tình cờ
có cùng mã hành động. Cùng hạng với F3: hai bản sao của một trạng thái, chỉ một
bản được dọn.

**Bấm một nút là mất sạch chữ vừa gõ.** Mọi lần đổi màn đi qua MỘT đường, và
đường ấy xoá sạch trạng thái khung giữ: công tắc, nội dung ô, tiêu điểm. Với
một MÀN MỚI, xoá là tính chất AN NINH — công tắc màn cũ còn lại là màn cũ trả
lời hộ màn mới, một mã PIN gõ dưới một nhãn hiện lại dưới cùng nhãn ở màn khác
là rò rỉ. Nhưng màn KẾT QUẢ trong ứng dụng (bấm nút → một dòng) được dựng như
màn mới, vì nó là cây cũ cộng một dòng.

Hai việc khác nhau mang một cái tên. Nay hai tên: `Next::Show` (màn mới — xoá)
và `Next::Update` (cùng màn, cây đổi — giữ). `Show` vẫn là mặc định; ai muốn
giữ phải NÓI RA ở chỗ gọi. Gộp lại thì hỏng theo một trong hai kiểu, và kiểu
thứ hai là một lỗ an ninh chứ không phải phiền toái. B55.

**KHÔNG làm sao chép/dán, và ghi rõ vì sao.** `tao` 0.34 đã bỏ mô-đun bộ nhớ
tạm; thư viện thay thế (`arboard` 3.6) kéo theo cả `image` — một thư viện GIẢI
MÃ ẢNH — cho một việc chép chữ. Trong dự án lấy "không có trình phân tích nào
giữa byte của gói và màn hình" làm luận điểm chính, cái giá ấy không tương
xứng. Đã gỡ phần mã đã viết thay vì để nó nằm lại chờ ai đó bật lên.

Nếu sau này quyết làm, luật đã nghĩ sẵn: **ô BÍ MẬT không sao chép được** — ô
ấy che chữ có chủ đích, sao chép nó là đặt đúng thứ vừa che lên một bảng nháp
toàn hệ thống. Dán thì cho, vì đó là người dùng đưa dữ liệu VÀO, và chuỗi dán
vẫn phải đi qua đúng phép kiểm của chữ gõ tay.

⚠️ Thăm dò đầu tiên cho `tao::clipboard` SAI: tôi đặt nó trong `#[cfg(test)]`
rồi chạy `cargo build` — mà `cargo build` không biên dịch mô-đun test, nên nó
chưa hề kiểm gì. Thăm dò một API thì dùng `cargo check --all-targets`.

**Luật tôi viết hôm qua mang đúng lỗi của hôm qua.** Luật 10b — thêm 25/08 để
hỏi chiều ngược của luật 10 — đọc CẢ bảng "ba mã đã gỡ vì không thể xảy ra",
nên một mã đặc tả tuyên bố đã RÚT vẫn được tính là có định nghĩa. Đó là **lần
thứ ba** cùng một lỗi trong cùng một tệp: luật 16 cắt đúng từ đầu, luật 10
quên (vá 25/08), luật 10b quên — và luật 10b do chính tôi viết ra vài giờ sau
khi vá luật 10.

Vá xong nó bắt thêm `not-a-container` và `publisher-not-hex`. Cả hai ĐỀU được
dựng trong mã, y hệt `bad-key`. Khác ở chỗ lần này đặc tả **đúng**, và nay có
bằng chứng chứ không phải lý lẽ:

- `publisher-not-hex`: kiểm hình dạng chối trước bằng `not-hex`. Kiểm đột biến —
  **bỏ `validate_shape()` thì gói bắn ra `publisher-not-hex` thật**.
- `not-a-container`: nút lá không có trường `children` nên bộ giải mã chối bằng
  `bad-json` trước. Kiểm bằng một tải trọng thật.

Chúng còn trong mã vì API Rust gọi tới được; một GÓI thì không — và phân biệt ấy
là toàn bộ nội dung lời đặc tả nói, nên nay nó là phép thử. Miễn trừ của luật
10b **không phải tha bổng**: cổng kiểm hai phép thử ấy còn tồn tại, xoá một cái
là đỏ.

**Và trần nội dung chưa từng được kiểm.** Đo `tcc-runtime`: 73 mutant, 55 bị
bắt, 5 sống — cả 5 ở đúng một chỗ, `MAX_CONTENT_BYTES`, trần 256 MiB cho nội
dung đọc TRƯỚC khi xác thực. `>` thành `>=`, `+=` thành `*=`, và cả hai dấu `*`
trong `256 * 1024 * 1024`.

Không ai dựng 256 MiB tệp trong một phép thử, nên trần ấy không bao giờ được
kiểm. Nay trần là THAM SỐ, phép thử đặt nó bằng hai mươi byte, và kiểm hai điều
đáng: chặn đúng ở mép, và **cộng dồn qua thư mục con** — kiểm từng tệp riêng lẻ
thì bao nhiêu tệp cũng lọt. Số học của chính hằng ấy là một `const assert`, nên
đổi nó làm hỏng BẢN DỰNG chứ không phải một phép thử.

**Một phép che không ai kiểm, ở đúng kiểu giữ hạt giống ví.** `WalletSecret`
mang 32 byte hạt giống và có bản `Debug` viết tay, chú thích ghi rõ mục đích:
"không in khoá ra nhật ký, dù ai đó gọi `{:?}` trên cả một cấu trúc lớn".

Không gì đọc thứ nó in ra. Thay cả thân hàm bằng `Ok(())` — in ra chuỗi rỗng —
mà mọi phép thử vẫn xanh.

Đột biến đáng sợ không phải `Ok(())`. Là ai đó thay bản viết tay bằng
`#[derive(Debug)]`, và từ lúc ấy hạt giống chảy vào mọi dòng nhật ký in một cấu
trúc có chứa nó. Không công cụ nào sinh ra đột biến ấy, và trước phép thử này
thì không gì chặn.

Phép thử kiểm HAI chiều, và chiều thứ hai là chiều người ta hay quên: không
được lộ, VÀ không được rỗng. Một bản `Debug` in chuỗi rỗng cũng qua vế "không
lộ" trong khi xoá sạch thứ người soát cần đọc.

**Hai mutant cố ý để sống.** `|` thành `^` trong phép gói bit ở `mnemonic.rs`:
sau `bit << 11` thì 11 bit thấp bằng 0 và mọi chỉ số nhỏ hơn 2¹¹, nên hai phép
ấy **y hệt nhau về toán học**. Không phép thử nào giết được, và không nên cố.
Thứ ghim được là BẤT BIẾN khiến chúng tương đương — từ điển đúng 2048 từ — mà
điều ấy cũng chưa ai khẳng định. Nay có: đổi `BITS_PER_WORD` là phép thử từ
điển đỏ.

⚠️ **Lượt đo đầu của crate này KHÔNG phải một phép đo.** Với trọng tài
`cargo test --workspace`, nó báo **45 mutant sống** ở crate ví. `import.rs` nằm
trọn sau cờ `import-web-wallet`, mà lệnh ấy không bật cờ nào — nên mã bị đột
biến chưa từng được biên dịch, phép thử của nó chưa từng chạy, và mọi mutant
trong đó bị ghi là sống. Bật cờ lên: con số thật là **25**.

Lần thứ ba trong hai ngày "chưa chạy tới" được báo thành "phép thử yếu" — sau
bộ vector ngoài trọng tài và lượt đĩa đầy. Lần này cái giá của việc tin nhầm sẽ
là một ngày đi vá hai mươi chỗ không hỏng, ở đúng crate không được phép làm
hỏng.

**Đo nốt hai crate cuối — `tcc-ui` và `tcc-net`. Cả CHÍN crate nay đã đo ít
nhất một lần.** 101 mutant, 59 bị bắt, 17 sống, bốn trong số đó là lỗ thật.

- **Một dấu `!` bị xoá làm MỌI lời gọi RPC thành công thành lỗi.**
  `v.get("error").filter(|e| !e.is_null())` quyết định một phản hồi JSON-RPC có
  phải lỗi không. Bỏ dấu ấy thì `"error": null` — tình huống BÌNH THƯỜNG — bị
  coi là lỗi, còn lỗi thật thì bị nuốt. Không ai đỏ vì **không ai chạy tới**: cả
  khối nằm trong một hàm chỉ chạy khi có mạng thật. Nay là hàm thuần
  `doc_phan_hoi`. Cùng nước đi với `dich_loi_doc` và `phan_loai`: **kéo phần
  quyết định ra khỏi phần I/O rồi kiểm phần quyết định.**
- **Một "một phần ba" có thể rộng GẤP BA nhóm cha.** `1.0 / 3.0` thành
  `1.0 * 3.0` mà không ai đỏ. Nay ghim cả sáu phân số và bất biến "mọi phân số
  nằm trong (0, 1]". Kèm `kiem_be` — phép chối một bề đặt sai trục — thay cả
  thân bằng `Ok(())` vẫn xanh.
- **Trần cây giao diện 1 MiB**: `>` thành `>=`, không ai thấy.

**Bốn lần cùng một hạng ranh giới trong hai ngày** — 253 ký tự tên máy chủ,
64 KiB bản kê khai, 256 MiB nội dung gói, 1 MiB cây giao diện. Bốn lần thì
không còn là sơ suất mà là **chỗ mù có hệ thống**: viết phép thử thì người ta
tự nhiên chọn một giá trị rõ ràng sai và một giá trị rõ ràng đúng. **Không ai
tự nhiên chọn đúng con số ở mép** — mà đó là giá trị duy nhất phân biệt được
`>` với `>=`, và cũng là con số đặc tả gọi tên.

Hai hằng cũng chốt bằng con số chính xác thay vì một khoảng: `8 * 1024 * 1024`
đổi thành `8 * 1024 + 1024` vẫn nằm trong "lớn hơn 0 và không quá 64 MiB", nên
phép kiểm khoảng cho qua **một trần 9 KiB đội lốt 8 MiB**.

**`AUDIT.md` mời người soát độc lập đọc "40 bất biến" — bảng đã có 61.** Có
cổng cho số phép thử, số vector, số luật kiến trúc, số lệnh theo cờ — mà bỏ sót
chính bảng bất biến, thứ `SECURITY.md` dựng nên để nói "điều này được giữ, và
đây là phép thử giữ nó".

Con số ấy sai nguy hiểm hơn vẻ ngoài: `AUDIT.md` là trang mời người NGOÀI vào
soát. Họ đếm được 61 rồi không biết tin con số nào — hoặc tệ hơn, dừng ở cái
thứ 40 và tưởng đã soát hết. Nay có cổng, và kiểm đột biến: đổi 61 thành 40 là
đỏ ngay.

**Thứ tự ưu tiên chốt 26/08/2026: vẫn tích hợp ví, nhưng làm TRÌNH DUYỆT
trước, ví sau.** Bản dựng chính nay là `--features window` — không có ví. Ghi ở
[`ke-hoach.md`](ke-hoach.md).

**Và ngay khi đổi thứ tự, một lỗ lộ ra.** Bản dựng không có ví VẪN hỏi người
dùng về quyền ví — kèm câu "việc này chuyển tiền" và một công tắc gạt được. Nó
vô hình suốt thời gian ai cũng dựng kèm ví.

Hỏi một câu mà không cấp được câu trả lời là **hộp thoại nói dối**. Vá ở HAI
chỗ, và chỗ thứ hai mới là chỗ đáng:

- Hộp thoại thôi dựng công tắc cho quyền bản dựng không cấp được; nó nói thẳng
  "bản dựng này không có ví — lời xin bị từ chối".
- **Đường CẤP cũng từ chối.** Hộp thoại không phải lối vào duy nhất:
  `.tcc-quyen.json` ghi từ bản CÓ ví mang theo câu "đã đồng ý", và trục trợ
  năng là lối khác. **Câu trả lời do bản dựng KHÁC ghi lại không phải câu trả
  lời cho bản dựng này.**

Năm phép thử cũ đỏ theo — đúng như phải thế, vì chúng khẳng định tính chất của
hàng ví CÓ công tắc. Sửa cho chúng nói đúng sự thật của TỪNG bản dựng, không
tắt đi: bản có ví thì ghim câu "việc này chuyển tiền", bản không ví thì ghim
câu từ chối. Bất biến B45 giữ nguyên ở cả hai — hàng ví phải KHÁC HẲN hàng
mạng; chỉ câu mang dấu là khác.

⚠️ Đặc tả **không có cách nào** để một bản cài đặt nói "tôi không cung cấp
quyền năng này". `unknown-capability` nghĩa là quyền ấy không có trong tiêu
chuẩn — một câu khác hẳn. Từ chối như một lần từ chối thường là cách trung thực
hiện có, nhưng khoảng trống ấy đáng nằm cạnh ba mã lỗi trong
[`de-nghi-ma-loi-thieu.md`](de-nghi-ma-loi-thieu.md).

**Còn cần NGƯỜI, không phải mã:** một buổi thử với trình đọc màn hình thật (sẽ
cho biết bản vá `Focus` của B42 có thật sự có tác dụng hay chỉ nằm im), kiểm
định an ninh độc lập, và soát `ttf-parser`.

## Đứng ở đâu — 23/08/2026

Nhánh `giai-doan-3.1`, CI xanh.

**337 phép thử (thời điểm ấy) · 154 vector · 22 luật kiến trúc · bộ kiểm định tuân thủ ĐẠT.**

### Máy dựng web đã bỏ HẲN (23/08)

Không còn `wry`, không còn `tcc-render-webview`, không còn tầng 2. Một bộ dựng
(ra pixel), một cửa sổ, một đường. −7536 dòng.

**Chỗ chặn thật không phải việc xoá.** Mười hai màn hình raster đã có sẵn, nên
nhìn danh sách hàm thì tưởng chỉ việc đổi lời gọi. Nhưng `tao` chỉ cho MỘT vòng
lặp sự kiện mỗi tiến trình, mà mỗi hàm ấy tự dựng một vòng — nên chúng gọi được
đúng một lần, và mọi luồng nhiều màn hình không chạy được. `open_sequence` là
thứ mở khoá; xoá crate chỉ là việc sau đó.

**Mất gì:** 56 phép thử, thoát ký tự, CSP, `kiem-khoi-tan-cong`, tầng 2 cùng
nhóm chắn của nó, thanh địa chỉ. Danh mục đầy đủ và lý do từng cái ở
[`../SECURITY.md`](../SECURITY.md) §3.7. Điều đáng nhớ nhất: **không còn ai
NGOÀI mã của ta nhìn vào thứ ta vẽ** — trước có thể hỏi WebKit "anh thấy gì",
nay phép thử ta viết đọc lại cây ta cũng tự dựng.

### Bẫy đã dẫm hôm nay

- **Một phép thử xanh trên macOS, đỏ trên Linux — và tôi đoán sai BA lần.**
  Đã xanh cả ba nền từ 23/08. Đáng ghi lại là cách tìm ra, không phải bản sửa:
  lần nào cũng chỉ tiến được khi **đưa thêm số đo vào thông báo lỗi** — đầu tiên
  là hộp, rồi `cao_dong`/`cao`, rồi biên nét thô. Đoán từ một nửa dữ liệu là
  đoán, và tôi đã đoán ba lần trước khi chịu đi lấy nửa còn lại.

  Nguyên nhân thật: bộ tính bố cục làm tròn `o.rong` **xuống**, nên lượt vẽ tạo
  hình ở bề rộng hẹp hơn lượt đo, chuỗi ngắt thành hai dòng, nét cao gấp đôi số
  đã đo. Hai bản sửa đầu — đoán rằng lỗi ở chiều cao dòng — không sai về mã,
  chúng chỉ chữa nhầm chỗ. Bản đúng: làm tròn LÊN, và gộp hai lượt tạo hình
  thành **một hàm duy nhất** để chúng không lệch được nữa.
- **`clippy` chưa bao giờ được chạy theo từng cờ**, chỉ một lượt workspace. Mã
  sau một cờ chỉ được soi khi cờ ấy bật. `CLAUDE.md` đã vá.
- **`grep "webview"` không bắt hết.** Hai bước CI còn sống vì tên chúng là
  "Measure the web platform (WebKitGTK)" và chúng gọi ví dụ `do-nen-tang`.
- **Phép thử so hai TỔNG thì vỡ khi có cách gọi mới.** `cho_goi_raster_dung_cau_
  da_dich` đếm `raster_text(...)` rồi so với số `open_screen(`; `open_sequence`
  xuất hiện, một điểm vào gọi ba lần, hai số rời nhau trong khi mọi chỗ đều
  đúng. Nay soi TỪNG chỗ dựng màn hình.

### Còn nợ, có tên

| Nợ | Ở đâu |
|---|---|
| `chay_chuoi` còn 134 dòng (từ 169) — phần đáng cắt đã cắt, phần còn lại cắt chỉ để qua ngưỡng | `tcc-render-raster/src/window.rs`, lý do ghi ngay trên hàm |
| Chưa trình đọc màn hình nào chạy thật | SECURITY.md §3.1d |
| `ttf-parser` phân tích phông trong tiến trình vẽ nội dung đã ký | SECURITY.md §3.5b |

## Đứng ở đâu — 19/08/2026

Nhánh `giai-doan-3.1`. `main` **cố ý** dừng ở `f738085` (chưa có ví) để người
soát ngoài đọc một cây ổn định.

**383 phép thử · 154 vector · 22 luật kiến trúc · bộ kiểm định tuân thủ ĐẠT.**

| Giai đoạn | Tình trạng |
|---|---|
| 5 — tầng web hiện đại | ✅ **đóng** (19/08). Ba bộ máy đều đo được, đều **18/20** |
| 4 — bộ dựng riêng | 🔶 cổng ra **đạt phần vẽ, bấm, gạt công tắc**; trợ năng nối xong trên macOS |
| 3 — ví, danh tính | 🔶 ví và ký chạy thật; chứng thực chờ sổ khoá của 0.2 |

### Giai đoạn 5 đóng thế nào

Ba bộ máy — WKWebView, WebKitGTK, WebView2 — **thiếu đúng cùng hai mục**
(`crypto.subtle`, `localStorage`). Ba bộ máy, một trong đó không chung dòng mã
nào với hai cái kia: đó là xác nhận, không phải trùng hợp. Nguyên nhân nằm ở
**cách nạp tài liệu** (`with_html` cho nguồn gốc mờ), không ở bộ máy.

**Bộ 50 trang thật** đo *giá của chính sách ta đặt*, không so ảnh chụp — so điểm
ảnh ở tầng 2 là đo WebKit của Apple. Kết quả: `https`-only tốn **0**, tải tệp
**0**, nhưng **148 lần từ chối cửa sổ mới** dồn hết vào trang quảng cáo và
**đúng 0 lần** trên mọi trang tài liệu.

**Nhãn "TCC Ready" không làm** — bộ đếm không tách được "trang cần quyền ấy" khỏi
"quảng cáo của trang thử đòi", nên nhãn đạt/trượt sẽ đổ lỗi cho trang vì quảng
cáo của nó. Thay bằng tính chất gọi đúng tên: **"im khi nạp"**, 26/50.

### Đường thoát khỏi WebView giờ là thật

`cargo run -p tcc-shell --features window-tro-nang --example man-hinh-raster examples/hello-tcc hop-thoai`

Gói **đã ký** lên màn hình, **bấm được**, **gạt công tắc được**, và **VoiceOver
đọc được** — không một dòng `wry`. `cargo tree` xác nhận: 0 crate `wry`.

Ba luật của hộp thoại giữ nguyên trên bộ dựng mới: mở ra **mọi mục tắt**, **đóng
cửa sổ không phải đồng ý**, **gạt công tắc không đóng hộp thoại**.

**`unsafe` đầu tiên và duy nhất** nằm ở đây — trao con trỏ `NSView` cho AccessKit.
`SECURITY.md` §3.1b đã lường trước và hoãn tới giai đoạn 4; đây là giai đoạn 4.

### Còn mở — và phần lớn cần NGƯỜI, không cần mã

| Việc | Ai làm được |
|---|---|
| Kiểm định an ninh **độc lập** | người ngoài — **cổng chặn mainnet** |
| Người ngoài dựng gói **chỉ từ `spec/0.1/`** | người ngoài — phép thử duy nhất của đặc tả |
| Hồ sơ cấp phép Apple (`com.tcc.browser` + Keychain Sharing) | bạn — mở khoá ví trên máy thật |
| Adapter trợ năng Windows, Linux | mã, nhưng **không thử được ở máy này** |
| `ActionHandler` nhận hành động | mã — **cố ý hoãn**, xem dưới |

`ActionHandler` để trống **có chủ ý**: nhận yêu cầu "bấm nút này" từ hệ điều
hành là mở một đường bấm nút không qua chuột, mà trên màn xác nhận giao dịch đó
là đường **ký hộ**. Nó sẽ phải mở, nhưng đi cùng mô hình đe doạ và phép thử
riêng.

### Đặc tả đã bịt bảy chỗ mơ hồ (18–19/08)

Nặng nhất: **thứ tự ưu tiên lỗi**. Mọi vector trước đó chỉ phạm một luật, nên một
bản cài đặt độc lập có thể **qua sạch bộ kiểm định** rồi bất đồng với bản gốc
trên mọi gói hỏng thật. Sáu chỗ còn lại: chuỗi hiển thị của giao diện, khoá JSON
trùng trong tệp giao diện, `children` mặc định, tệp thừa ngoài ba tên đã biết,
một trích dẫn sai mục, và **tuyên bố cái gì mang tính quy phạm** (luật 23 cưỡng
chế).

## ✅ Gõ tiếng Việt trên MÀN HÌNH ỨNG DỤNG THẬT (17/08/2026)

Người dùng gõ `chào buổi sáng bạn iu` bằng Telex vào
`tcc-browser examples/hello-tcc`. Ca khó nhất trong câu là **ổ** — hai tầng
dấu, mũ rồi hỏi — và nó đúng chỗ, không có dấu rời, con trỏ ở cuối.

Đáng ghi riêng vì tới hôm nay **màn hình ứng dụng mới được vẽ ra**: binary
trước đó không bao giờ gọi `run_app`, nên mọi lần kiểm bộ gõ trước đây đều đi
qua hộp thoại quyền hoặc một kịch bản chẩn đoán, không phải màn hình thật.

> **v1 — trình duyệt Electron — KHÔNG nằm trong kho này.** Nó ở thư mục cha trên
> máy của đội ngũ, đang tạm dừng, và có ghi chú riêng. Kho công khai chỉ có v2.

### ✅ GIAI ĐOẠN 1 ĐÓNG ĐỦ CHÍN CỔNG (15/08/2026)

Cổng cuối — **gõ tiếng Việt có dấu bằng bộ gõ hệ thống** — đóng hôm nay. Gõ
`Chào buổi sáng mọi người` bằng Telex của macOS: **24 mã điểm / 24 chữ gốc /
0 dấu rời**, dạng DỰNG SẴN, con trỏ ở cuối, phiên ghép đã chốt.

Nó là cổng duy nhất cần một CON NGƯỜI: bộ gõ thuộc hệ điều hành, và mọi cách
giả lập đều bơm chuỗi đã hoàn chỉnh vào ô nhập — tức là bỏ qua đúng cái cần đo.

Giai đoạn 2 cũng đã xong cả bốn mục; cổng ra của nó cần một NGƯỜI NGOÀI đọc
`spec/0.1/` rồi tự dựng gói, nên không tự đóng được.

**Việc kế tiếp theo kế hoạch: Giai đoạn 3.1 — ví gắn kho khoá hệ điều hành.**
Cổng chặn cứng vẫn nguyên: không giao dịch mainnet trước kiểm định độc lập.

### v2 tới đâu rồi (15/08/2026)

Xong: `tcc-spec`, `tcc-crypto`, `tcc-manifest`, `tcc-capability`, `tcc-runtime`,
`tcc-cli`, `tcc-ui`, `tcc-render-webview`, `tcc-shell`, `apps/tcc-browser`.
**238 phép thử + 136 vector tuân thủ**, clippy sạch, 22 luật kiến trúc 0 vi phạm.
**Đường ống đã nối đủ**: gói đã ký trên đĩa → kiểm chữ ký → hộp thoại hỏi quyền
trong cửa sổ thật → cú bấm quay về → cấp quyền → **vẽ màn hình ứng dụng**.

**KHO ĐÃ CÔNG KHAI** (14/08): `github.com/tccnetwork/TCCBrowser`, Apache-2.0,
CHỈ CÓ v2. v1 nằm ở thư mục gốc và **không còn được git theo dõi ở đâu cả** —
`.git` ở gốc đã gỡ khi tách kho. Đó là rủi ro chưa xử lý.

### Ba ngày vừa rồi làm gì (13→15/08)

| | |
|---|---|
| Tiêu chuẩn | `spec/0.1/` song ngữ, tiếng Anh là bản CHUẨN · `VERSIONING.md` + `GOVERNANCE.md` |
| Kiểm định | 104 → **136 vector**, tám nhóm; thêm nhóm `package` và `verify` (ký ngay lúc chạy để kiểm THỨ TỰ các bước) |
| Luật kiến trúc | 12 → **17**, mỗi luật đều kiểm đột biến hai chiều |
| Fuzz | `tools/tcc-fuzz` (6 mục tiêu, chạy mỗi lần đẩy) + `fuzz/` libFuzzer (chạy đêm) |
| Đo đạc | băm theo luồng (128 MiB → 0 MiB thêm) · kênh biên thời gian · bộ nhớ đỉnh |

**Rà đặc tả như người ngoài** tìm ra bốn lớp khuyết tật mà 237 phép thử mù hoàn
toàn — chúng xanh suốt: điều khoản không ai canh · yêu cầu mà tiêu chuẩn không
cho phương tiện thoả mãn · **bốn mã lỗi không bao giờ nổ được** · artefact được
gọi tên mà chưa định nghĩa (`.tccapp`). Chi tiết ở `v2/spec/README.md`.

⚠️ **Điểm vào KHÔNG phải HTML** (đổi 13/08/2026). `tcc new` từng sinh
`entry: "index.html"` — chạy được nhưng phá luật trung tâm: ứng dụng ship HTML
thì không bao giờ tháo được WebView. Giờ điểm vào là `ui.json`, một cây
component khai báo. Xem `tcc_ui::dang_goi` và B15/B16 trong `v2/SECURITY.md`.

```sh
cd v2
cargo run -p tcc-browser --features window -- <thư-mục-gói>
```

Mười ví dụ đối kháng (KHÔNG nằm trong `cargo test` — cái cần cửa sổ thì trên
macOS vòng lặp sự kiện phải ở luồng chính): `kiem-khoi-tan-cong`,
`kiem-khoi-tan-cong chi-csp`, `kiem-bam-nut cho-phep|tu-choi|bat|ma|ct-ma`,
`kiem-man-hinh-ung-dung <gói>`, `kiem-hanh-vi <gói>`, `kiem-ghi-nho <gói>`.

**Hỏi quyền theo TỪNG MỤC** (đổi 13/08/2026): mỗi quyền một công tắc, mặc định
TẮT. Bấm "Cho phép" mà không bật gì thì không quyền nào được cấp. Thêm loại
component `Toggle` là đổi tiêu chuẩn — và bộ dựng đã **không biên dịch được**
cho tới khi xử lý nó, đúng như thiết kế đã hứa.

**Bộ dựng đã chốt: `wry`** — đo thật (wry+tao 71 crate; objc2 FFI tay 18 crate
nhưng chỉ macOS). Cả hai đều đặt WebKit vào đường vẽ nên FFI tay không mua được
gì mà phải trả bằng `unsafe` trên đúng thứ sẽ tháo. Nằm sau cờ tính năng
`window` để `cargo test` không phải kéo cả tầng cửa sổ.

**Bộ kiểm định tuân thủ đã chạy** (`cargo run -p tcc-conformance`, 136 vector, tám nhóm).
Vector là **dữ liệu JSON**, không phải mã Rust — để bản triển khai bằng ngôn ngữ
khác đọc được. So khớp bằng **mã lỗi ổn định**, không bằng thông báo.
Nhóm `canonical` sinh bằng bản cài đặt Python độc lập; Rust và Python khớp từng
byte ở cả 7 trường hợp.

**Hành vi của nút** khai trong BẢN KÊ KHAI (`manifest.actions`), không trong
`ui.json` — vì chữ ký bao trùm bản kê khai, vì hộp thoại hỏi quyền đọc được nó,
và vì `tcc-ui` không được biết tới mạng. `tcc verify` in ra mỗi nút gọi đi đâu.

**Máy khách HTTP đã nối** (`crates/tcc-net`, dùng `ureq`+rustls — 22 crate so với
`reqwest` 86). Sáu luật cứng: chỉ HTTPS · **KHÔNG đi theo chuyển hướng** · có
thời gian chờ · có trần kích thước · không cookie · không gửi gì thừa.
Chuyển hướng là đòn thoát khỏi quyền năng, xem B23 trong `v2/SECURITY.md`.
Cờ `mang` tách riêng nên dựng được bản **không có mạng**.

**Vector chữ ký đã có** (`conformance/vectors/signature.json`, 15 trường hợp).
Kiểm ba chiều: sinh khoá · ký lại ra đúng byte cũ · kiểm chữ ký. Ký là **tất
định** nên vector tái tạo được hoàn toàn.

**Cả hai nửa nay đều neo ngoài** (13/08/2026): Ed25519 vào RFC 8032, ML-DSA-65
vào **NIST ACVP** — 25/25 ca keyGen khớp. Và việc chạy vector NIST trả lời một
câu ta vẫn đang giả định: **TCC dùng giao diện NGOÀI của FIPS 204, context
RỖNG** (nhóm `external` khớp, nhóm `internal` lệch 3/15). Dùng nhầm giao diện
thì hai bên đều "đúng FIPS 204" mà chữ ký của nhau không kiểm được — bẫy interop
im lặng, nay đã thành một câu của tiêu chuẩn. Chiều KÝ vẫn chưa neo được: ACVP
cho khoá đã bung, thư viện chỉ nạp được hạt giống.

**Kho quyền đã có** (`crates/tcc-shell/src/ghi_nho.rs`, tệp `.tcc-quyen.json`
cạnh gói). Nhớ theo **cả khoá người ký lẫn vân tay phạm vi** — đổi một trong hai
là hỏi lại. Hộp thoại chỉ liệt kê quyền CÒN THIẾU câu trả lời.
`TCC_QUEN_HET=1` để bỏ qua kho và hỏi lại từ đầu.

**Ví dụ `examples/hello-tcc` đã có** — gói ký sẵn, cam kết trong kho,
`cargo run -p tcc-cli -- verify examples/hello-tcc` chạy được ngay. Nó cố ý gồm
đủ mọi loại nút, hai ô nhập (một bí mật), ảnh trong gói, một quyền năng và một
hành vi. Khoá ký là **khoá demo ai cũng có**; luật 9 chặn nó rời khỏi `examples/`.

**GIAI ĐOẠN 1 — chỉ còn MỘT cổng.** Hai cổng cần người ngồi trước máy đã đóng
trong phiên 13/08/2026:

| Cổng | |
|---|---|
| `cargo check` trên Linux | ⚠️ **10/11 crate sạch**; chỉ `tcc-net` chưa kiểm được vì `ring` (thư viện C dưới TLS) cần bộ công cụ biên dịch chéo. CI trên Ubuntu đóng nốt cổng này. |
| **VoiceOver đọc được `examples/hello-tcc`** | ✅ **13/08/2026 — ĐẠT**, sau khi sửa hai lỗi mà chính việc soi cây trợ năng tìm ra. Xem B32/B33 trong `v2/SECURITY.md`. |
| **Gõ tiếng Việt có dấu vào ô nhập** | ✅ **13/08/2026 — ĐẠT**. Gõ "Chào ngày" bằng bộ gõ hệ thống: dấu chồng đúng chỗ, con trỏ ở cuối, không nhảy. Xác nhận bằng ảnh chụp cửa sổ. |

**13/08/2026 — quyền macOS.** Tự động hoá ✅ · Ghi màn hình ✅ · **Trợ năng ✅**
(cả ba cấp trong phiên này, cho **Visual Studio Code**). Dấu hiệu nhận ra Ghi màn hình
chưa có: `screencapture` trả mã 0 nhưng ba ảnh chụp ở ba thời điểm khác nhau có
**cùng mã băm** — macOS trả ảnh nền không cửa sổ. Đừng tin mã thoát 0 ở đây.
Quyền phải cấp cho **Visual Studio Code** (tiến trình cha), không phải cho
`tcc-browser`.

⚠️ **NHÌN VÀ NGHE THẬT TÌM RA NĂM LỖI** mà 211 phép thử mù hoàn toàn.

Chụp cửa sổ (3 lỗi): công tắc quyền không có nhãn nhìn thấy được · ô nhập cũng
vậy · nút `Tone::Danger` trông y hệt nút thường. Xem B30/B31.

Soi cây trợ năng (2 lỗi): ô mật khẩu ra `AXTextField` thường vì tôi thêm
`role="textbox"` — **ARIA đè lên ngữ nghĩa gốc** · `aria-description` không lên
được trục trợ năng của macOS. Xem B32/B33.

**Hai bài học:**
1. Kiểm cây trợ năng chứng minh *người khiếm thị nghe được*, nó KHÔNG chứng minh
   *người sáng mắt nhìn được*. Hai câu khác nhau.
2. Cây trợ năng CỦA TA khớp không có nghĩa cây trợ năng CỦA HỆ ĐIỀU HÀNH đúng.
   Một bất biến tôi thêm vào để tăng an toàn đã làm hỏng đúng thứ nó định bảo vệ.

Câu VoiceOver đọc, đo được sau khi sửa:
```
"Ô bí mật (chữ phải bị che), secure text field"
"Xoá dữ liệu, nút — hành động không hoàn tác được"
```

**Ảnh trong gói đã hiện** (`crates/tcc-render-webview/src/phuc_vu_goi.rs`).
Giao thức `tcc-goi:` đọc từ `FileTree` **đã ký**. Ba luật: đường dẫn qua đúng
`check_path` · chỉ tệp có trong cây đã ký · kiểu nội dung theo **danh sách
trắng** đuôi tệp, **không có SVG** (SVG chạy được kịch bản — nó là tài liệu,
không phải ảnh). Hộp thoại hỏi quyền truyền `|_| None`: ứng dụng không đưa được
byte nào vào màn hình của trình duyệt. Xem B34–B36.

Ví dụ đã dựng sẵn cho đúng hai việc đó: có ô "Gõ thử tiếng Việt", có ô bí mật,
có chữ có dấu ở khắp nơi. Chạy:

```sh
cd v2 && cargo run -p tcc-browser --features window -- examples/hello-tcc
```

**Ghim khoá kiểu tin-lần-đầu đã có.** Khoá ký đổi → hộp thoại cảnh báo, và cảnh
báo đứng TRƯỚC danh sách quyền. Chữ là **sự thật quan sát được** ("trước đây ký
bằng khoá khác"), không phải phán quyết ("giả mạo") — ta không biết ai đúng ai
sai. Xem B29 trong `v2/SECURITY.md`.

⚠️ **DỰ ÁN KHÔNG NẰM TRONG KHO GIT NÀO.** `.github/workflows/ci.yml` có sẵn và
đã được cập nhật (thêm bộ kiểm định, ví dụ, và 9 ví dụ đối kháng chạy trên
macOS), nhưng **nó chưa bao giờ chạy**. Không git nghĩa là: không CI, không lịch
sử, không quay lui được. Chín luật kiến trúc chỉ chạy khi có người gõ tay.

**Màn hình quản lý quyền đã có**: `tcc-browser quyen <thư-mục-gói> [vi]`. Liệt kê
mọi ứng dụng đã trả lời, phạm vi, trạng thái, và một nút "Quên" **riêng cho từng
ứng dụng**. Chữ mô tả phạm vi lưu trên đĩa là chữ **chỉ để hiện** — `tra()` không
bao giờ đọc nó, có phép thử chốt. Xem B37 trong `v2/SECURITY.md`.

**Chữ của bộ dựng đã dịch được** (14/08/2026). Câu cảnh báo trợ năng từng bị
khoá cứng ở tiếng Việt vì nó vừa là chữ cho người vừa là **dấu hiệu cho máy** —
bộ quét so đúng chuỗi đó. Nay tách hẳn: dấu hiệu máy là `data-sac-thai="mat-mat"`
(không bao giờ đổi), chữ tiêm từ `tcc-shell` xuống qua `loi::chu_bo_dung()`.
**Bộ dựng không biết ngôn ngữ và không nên biết.** Mặc định tiếng Anh. Xem B39.

**Chiều KÝ đã neo — bằng đối chiếu chéo, không bằng vector** (14/08/2026).
`sigGen` của ACVP không dùng được (khoá đã bung 4032 byte vs hạt giống 32 byte),
nên dùng `dilithium-py` — bản thuần Python của người khác. Hai bản độc lập ra
**cùng chữ ký từng byte**. Bước bắt buộc: bản Python phải tự khớp NIST trước
(25/25), không thì nó chỉ là ý kiến thứ hai.
Chạy lại: `python3 conformance/doi-chieu-doc-lap.py <thư-mục-vector-ACVP>`

**Nợ "hộp thoại rời khỏi WebKit" đã ĐO** (14/08/2026) và nó nhỏ hơn nhiều so với
cách nó được ghi: mỗi cửa sổ WebView có **tiến trình nội dung riêng**, và hộp
thoại hỏi quyền **không bao giờ sống cùng lúc** với màn hình ứng dụng — kiến
trúc một-vòng-lặp tự nó cấm. Rủi ro còn lại đã thu hẹp còn: cú thoát sandbox từ
nội dung ứng dụng có thể ảnh hưởng hộp thoại mở SAU đó. Xem §3.1b.
⚠️ Mở hộp thoại như cửa sổ con, hoặc chuyển sang vòng lặp đa cửa sổ, sẽ **phá**
guarantee này.

**Khi đo món nợ đó thì tìm ra một lỗ thật**: ứng dụng tự đặt tiêu đề cửa sổ, nên
đặt tên `"TCC — quyền đã cấp"` là có cửa sổ trông y hệt màn hình trình duyệt.
Nay mã ứng dụng đã ký đứng TRƯỚC: `com.tcc.vi-du.hello — Xin chào TCC`. Xem §3.1c.

**`spec/0.1/` ĐÃ VIẾT, SONG NGỮ** (14/08/2026) — trước đó là thư mục **rỗng**,
nghĩa là mọi thứ ta xây chỉ là *một bản triển khai kèm vector*, không phải một
tiêu chuẩn. Bảy tệp mỗi bản: gói · bản kê khai · chữ ký · quyền năng · giao diện
· 32 mã lỗi.

⚠️ **Tiếng Anh là bản CHUẨN** (`spec/0.1/`), tiếng Việt là bản dịch
(`spec/0.1/vi/`). Hai bản mâu thuẫn thì bản tiếng Anh thắng.

Đặc tả gom cả những sự thật chỉ tồn tại rời rạc cho tới hôm nay — nhất là **giao
diện NGOÀI của FIPS 204 với ctx rỗng**, bẫy interop im lặng nhất của cả tiêu
chuẩn: dùng nhầm thì hai bên đều "đúng FIPS 204" mà chữ ký của nhau không kiểm
được.

**Ba luật kiến trúc giữ đặc tả khỏi trôi** (10, 11, 12) — đều đã kiểm đột biến,
xem `tools/kiem-luat-phu-thuoc.sh`:

| Luật | Kiểm gì |
|---|---|
| 10 | Mọi mã lỗi trong đặc tả **tồn tại trong mã** |
| 11 | Bản dịch **không trôi khỏi bản chuẩn** (số tệp, tập mã lỗi, tài liệu chính sách phải có bản dịch) |
| 12 | Đặc tả **không có liên kết chết** (80 liên kết) |

### Chính sách phiên bản + quản trị đã viết (14/08/2026)

`spec/VERSIONING.md` và `spec/GOVERNANCE.md`, song ngữ, áp cho MỌI phiên bản.

Viết chúng lôi ra một **drift thật giữa đặc tả và mã**: đặc tả nói bản cài đặt
*NÊN* từ chối trường lạ, mã lại im lặng bỏ qua (`Manifest`, `CapabilityRequest`,
`Scope` đều thiếu `deny_unknown_fields` — trong khi cây giao diện thì có).

Vì sao nó nguy chứ không chỉ luộm thuộm: **chữ ký phủ lên từng byte của
`manifest.json`, kể cả byte không luật nào đọc tới.** Nặng nhất là ở phạm vi
quyền — `{"kind":"network","hosts":["a.com"],"ports":[443]}` cấp cổng 443 trên
bản hiểu `ports`, và cấp MỌI cổng trên bản bỏ qua nó. Im lặng bỏ một trường chỉ
có thể NỚI quyền, không bao giờ thu hẹp.

Đã sửa: ba struct đóng lại, đặc tả nâng NÊN → **PHẢI**, thêm 3 vector kiểm định
(104 vector). Kiểm đột biến: gỡ `deny` khỏi `Scope` → đỏ đúng vector phạm vi; gỡ
khỏi `Manifest` → đỏ đúng vector gốc.

Hệ quả cho tiêu chuẩn, ghi trong `VERSIONING.md` §3: vì trường lạ bị từ chối,
**mọi trường THÊM vào đều là thay đổi phá vỡ**. Luật "thêm vào thì an toàn"
không áp dụng ở đây, và "chỉ thêm thôi mà" không bao giờ là lý lẽ.

`GOVERNANCE.md` §1 nói thẳng thứ dễ nói tránh nhất: **một tác giả, một bản cài
đặt, một bộ kiểm định — cùng một bên làm ra.** Nên "tuân thủ TCC 0.1" hôm nay chỉ
nghĩa là *đồng ý với một bản cài đặt*. Còn thiếu lớn nhất giờ là **bản cài đặt
thứ hai độc lập**, rồi mới tới cổng ra Giai đoạn 2 (người ngoài dựng gói chỉ từ
đặc tả — không tự kiểm được).

Việc kế tiếp của v2:
1. **Đưa dự án vào git** — xem cảnh báo ở trên. Đây là việc chặn cổng Linux cuối
   cùng của Giai đoạn 1, và là quyết định của bạn.
2. Giai đoạn 4 (bộ dựng riêng) mới là chỗ đúng để hộp thoại rời khỏi WebKit.

---

## Bẫy đã dẫm phải, đừng dẫm lại

Chỉ những bẫy của v2. Bẫy của v1 (chặn quảng cáo, Electron) nằm ở ghi chú của v1.

### 18–19/08/2026 — bốn cái, ba cùng một hình dạng

**`ControlFlow::Wait` làm nút "Đi" thành nút chết.** Tin nhắn từ ô địa chỉ vào
hàng đợi qua IPC, mà đẩy vào hàng đợi **không sinh sự kiện cửa sổ** nào. Vòng
lặp ngủ tiếp; người dùng bấm và **không có gì xảy ra**, tới khi họ tình cờ rê
chuột qua cửa sổ. `window.rs::run_loop` dùng `WaitUntil(50ms)` đúng vì lý do
này — tôi viết tệp mới mà không nhìn sang tệp cũ đã có đáp án.

**CI chỉ `cargo check` cờ `window`.** `check` biên dịch mà **không chạy phép thử
nào**, nên toàn bộ chắn tầng 2 chưa từng chạy ở đâu ngoài máy tôi. Luật 21 chặn
lại; 144 phép thử từ đó mới thật sự chạy.

**Bước đo Linux mang `continue-on-error` kèm chú thích SAI.** Chú thích nói
WebKitGTK dưới màn hình ảo không ổn định. Nó **không hề chập chờn** — trượt đều
**3/3** vì mã ta gọi `build(&window)` trong khi Linux cần `build_gtk`. Câu lỗi
`the underlying handle is not available` không nhắc chữ GTK nào, nên đọc y hệt
một màn hình ảo chưa lên. *Một phép thử được miễn vì "hạ tầng không đáng tin" là
chỗ tốt nhất để một lỗi thật nằm im.*

Ba cái trên **cùng một hình dạng**: thứ trông như đang canh thì không canh.

**Cách phát hiện đáng nhớ hơn cả ba bản vá:** chạy **cùng một tệp nhị phân ba
lượt trong một job**. Một lượt xanh không phân biệt được "đã sửa" với "vừa may",
và một lượt đỏ cũng thế.

### 22/08/2026 — "bấm không được", và tôi báo sai một lần trên đường tìm

Người dùng bấm **"Tải trang mẫu" 13 lần**. Cả 13 lần đều chạy đúng và tải về
559 byte. Ba thứ cộng lại làm nó **đọc như một nút chết**:

1. Kết quả chỉ ra `stderr`, màn hình không đổi gì — **lần thứ BA** dự án dẫm bẫy
   này. Luật cũ ghi *"mọi nhánh KẾT THÚC cần một màn hình"*, và lần này lọt vì
   đây không phải nhánh kết thúc: cửa sổ vẫn mở, chỉ là không có gì đổi. Cùng
   một hậu quả — với người dùng, việc ấy không xảy ra.
2. Nút **không có `:hover`, không có `:active`** — trông y hệt trước, trong và
   sau khi bấm.
3. Nút nằm sát đáy cửa sổ, và **không có cuộn** (F7 của rà soát 21/08).

**Tôi báo sai một phát hiện trên đường tìm.** Tôi kết luận *"`AXPress` không tới
được ứng dụng ở đường WebView"* và định gọi đó là lỗ trợ năng. Sai: lệnh
`entire contents` của System Events **hết giờ IM LẶNG** trên cây sâu, `try` nuốt
mất lỗi, nên tôi đọc "không thấy nút" thành "không bấm được". Hỏi thẳng theo
đường dẫn AX thì `AXPress` chạy đúng.

*Một công cụ đo hỏng im lặng thì nó không báo "tôi hỏng" — nó báo một kết quả,
và kết quả ấy trông như một phát hiện.*

Một thứ nữa đo được trên đường đi: **`tao::Window::title()` trả về bản đệm cũ**.
`set_title` chạy đúng — hỏi macOS thì thấy tiêu đề đã đổi — nhưng hàm đọc của
`tao` vẫn trả chuỗi ban đầu. Tin hàm đọc ấy là kết luận ngược.

Và phép thử `dinh_kieu_khong_mo_cua_cho_ve_de` **bắt được bản vá đầu của tôi**:
tôi dùng `transform:translateY(1px)` cho trạng thái nhấn, mà `transform` bị cấm
vì nó dời được một phần tử lên trên câu cảnh báo. Hiệu ứng nhấn giờ đổi **màu**,
không đổi **vị trí**.

### Sửa mã bằng script thì phải kiểm NGAY

Dẫm **hai lần trong một tuần**. Cả hai lần lệnh `python`/`perl` khớp vào **hư
không** — sai tên biến, và `cargo fmt` đã tách một `println!` thành nhiều dòng
nên chuỗi neo không còn tồn tại. Bản dựng **vẫn xanh**, vì chẳng có gì thay đổi.
Lần một chỉ lộ ra khi **kiểm đột biến** thấy đột biến sống sót; lần hai chỉ lộ
ra nhờ một cảnh báo `unused import`.


- **Phép thử có thể VÔ DỤNG mà vẫn xanh (13/08/2026).** Tôi viết phép thử "đổi
  khoá người ký thì quyền cũ bị xoá" bằng cách cho cả hai khoá xin CÙNG một
  quyền — và nó không kiểm được gì, vì `insert` đè lên rồi, `clear()` không có
  tác dụng quan sát được. Chỉ lộ ra khi kiểm đột biến: gỡ `clear()` mà mọi phép
  thử vẫn xanh. **Luật rút ra**: mỗi phép thử bảo vệ một bất biến thì phải thử
  gỡ đúng cái bất biến đó ra xem phép thử có đỏ không.

- **Chuỗi qua được mọi phép kiểm vẫn có thể là địa chỉ trỏ đi nơi khác (13/08/2026).**
  `shop.tcc-coin.com:8080@evil.example` là ASCII, không rỗng, không ký tự đại
  diện — qua hết. Nhưng dựng thành URL thì phần trước `@` là userinfo, máy chủ
  thật là `evil.example`. **Luật rút ra**: chuỗi nào sắp đi vào một cú pháp khác
  (URL, đường dẫn, câu lệnh) thì phải kiểm theo cú pháp ĐÍCH, không phải theo
  "có ký tự lạ không". Xem L9 trong `v2/SECURITY.md`.

- **`serde` KHÔNG đi qua hàm dựng của bạn (13/08/2026) — dẫm hai lần.**
  Kiểu dữ liệu giữ bất biến bằng hàm dựng có kiểm (`AppId::parse`,
  `Node::button`) thì `#[derive(Deserialize)]` **bỏ qua sạch**: nó nhồi thẳng
  vào trường. Lần một suýt dẫm ở cây giao diện — bịt bằng kiểu `UiNode` riêng.
  Lần hai dẫm thật ở `AppId` với `#[serde(transparent)]`, và **34 phép thử đơn
  vị mù hoàn toàn** vì chúng luôn dựng `AppId` bằng `parse`. Bộ kiểm định tuân
  thủ tìm ra, vì nó nạp bản kê khai từ JSON như người dùng thật.
  **Luật rút ra**: ở đâu có hàm dựng bảo vệ bất biến, ở đó phải hỏi "giải mã có
  đi qua nó không" — mặc định của serde là KHÔNG.

- **`vong.run()` của tao KHÔNG BAO GIỜ TRẢ VỀ (13/08/2026).** Nó gọi thẳng
  `exit()`, nên không có đường nào mang kết quả ra khỏi vòng lặp. Muốn lấy giá
  trị về thì phải dùng `run_return` (`tao::platform::run_return`). Trình biên
  dịch bắt được cái này.

- **Ghi đè `ControlFlow` ở ĐẦU mỗi vòng lặp là nuốt mất lệnh `Exit` (13/08/2026).**
  Vòng lặp chạy lại cho mỗi sự kiện; đặt `ControlFlow::Exit` xong, sự kiện kế
  tiếp đặt lại thành `Wait` là cửa sổ không đóng. Lộ ra vì dòng "tự đóng" in 5
  lần — nhưng hậu quả thật nặng hơn: bấm nút đóng cửa sổ cũng có thể bị nuốt.
  Sửa bằng một cờ `dang_thoat` kiểm ở đầu hàm.

- **Kiểm "không chèn được mã" bằng `contains("onclick=")` là SAI (13/08/2026).**
  Chuỗi đã thoát thành `&quot; onclick=&quot;` vẫn chứa `onclick=` nhưng là chữ
  trơ. Phép thử đỏ oan. Bằng chứng đúng là ĐỌC NGƯỢC: quét lại đánh dấu vừa
  sinh, nhãn phải về nguyên vẹn — phá được ra khỏi giá trị thuộc tính thì cây
  đọc ra sẽ khác.

- **`macOS`: `#[test]` KHÔNG mở được cửa sổ.** Vòng lặp sự kiện bắt buộc chạy
  trên luồng chính, mà bộ khung kiểm thử của Rust chạy mỗi phép thử trên luồng
  phụ. Phép thử cần cửa sổ phải nằm trong `examples/`.

- **`runner.sh` đã GỠ khỏi kho (15/08/2026).** Nó là công cụ cho môi trường làm
  việc của trợ lý — một trình nền đọc lệnh từ thư mục hàng đợi rồi chạy — chứ
  không phải một phần của TCC. Nằm ở gốc kho công khai thì nó vừa là nhiễu cho
  người soát, vừa là một kịch bản "chạy lệnh tuỳ ý" đặt sai chỗ trong một kho
  nói về an ninh. Bản gốc còn ở `~/.codetrail/template/runner.sh`, và lịch sử
  git vẫn giữ bản trong kho. Bẫy dưới đây vẫn đáng nhớ vì nó là bẫy của CÔNG CỤ,
  và mọi dự án chép mẫu ấy đều thừa hưởng:

- **`runner.sh exec` BÁO THÀNH CÔNG khi lệnh còn đang chạy (13/08/2026).**
  Lỗi nằm trong khuôn mẫu codetrail, không phải cấu hình của ta. Tiến trình nền
  ghi phần đầu tệp kết quả *trước* khi chạy lệnh, còn `exec` chỉ chờ **tệp xuất
  hiện** rồi `cat` ngay. Lệnh hỏng-ngay thì may mà kịp; `cargo test --workspace`
  thì in ra mỗi phần đầu rồi trả mã 0 — **im lặng và rất dễ tin nhầm là đã đạt**.
  **Đã sửa trong bản của dự án này**: chờ dòng `exit: ` (dòng cuối cùng tiến
  trình nền ghi) thay vì chờ tệp. Bản gốc ở `~/.codetrail/template/runner.sh`
  vẫn còn lỗi — dự án khác chép về thì dính lại.
  Danh sách cho phép cũng phải ghi `--manifest-path v2/Cargo.toml`, vì runner
  `cd` về gốc dự án mà workspace Rust nằm trong `v2/`.

- **BỘ NHỚ ĐỆM CŨ LÀM MÃ ĐÚNG VẪN CHẠY SAI — bẫy nặng nhất tới giờ (12/08/2026).**
