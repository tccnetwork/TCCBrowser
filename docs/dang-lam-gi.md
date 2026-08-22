# v2 — đang làm tới đâu

> Ghi chú bàn giao giữa các phiên làm việc, viết bằng tiếng Việt vì đội ngũ bảo
> trì đọc tiếng Việt. Tài liệu dành cho **người soát bên ngoài** thì bằng tiếng
> Anh: [`AUDIT.md`](AUDIT.md) là đường vào, rồi
> [`../SECURITY.md`](../SECURITY.md) và [`../spec/`](../spec/).
>
> Cập nhật lần cuối: **19/08/2026**.

## Đứng ở đâu — 19/08/2026

Nhánh `giai-doan-3.1`. `main` **cố ý** dừng ở `f738085` (chưa có ví) để người
soát ngoài đọc một cây ổn định.

**383 phép thử · 153 vector · 22 luật kiến trúc · bộ kiểm định tuân thủ ĐẠT.**

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

`cargo run -p tcc-shell --features cua-so-raster-tro-nang --example man-hinh-raster examples/hello-tcc hop-thoai`

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
cargo run -p tcc-browser --features cua-so -- <thư-mục-gói>
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
`cua-so` để `cargo test` không phải kéo 71 crate.

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
cd v2 && cargo run -p tcc-browser --features cua-so -- examples/hello-tcc
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
