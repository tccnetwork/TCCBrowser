# Kiểm đột biến — phương pháp và hồ sơ

> Tài liệu nội bộ, tiếng Việt. Người soát bên ngoài vào
> [`AUDIT.md`](AUDIT.md); mục ngắn dành cho họ là
> [`../SECURITY.md`](../SECURITY.md) §3.31.

Kiểm đột biến trả lời đúng một câu mà "bao nhiêu phép thử" không trả lời được:

> **Sửa mã cho sai đi thì có phép thử nào đỏ lên không?**

Bộ thử xanh mà không đỏ khi mã hỏng thì nó không đo cái nó tưởng nó đo. Phần lớn
phát hiện §3.18–§3.30 của `SECURITY.md` ra từ đây.

Chạy: `tools/kiem-dot-bien.sh` (tất cả) hoặc `tools/kiem-dot-bien.sh tcc-spec`
(một vài hòm). Bảng thô ở `/tmp/dot-bien/bang.txt`.

---

## Vì sao tài liệu này tồn tại

Lượt quét đầu tiên của dự án — những ngày trước 26/08/2026 — **không để lại hồ
sơ theo từng hòm**. Không ai, kể cả người chạy nó, kiểm lại được nó đã phủ tới
đâu. Một phép đo không có hồ sơ là một câu chuyện, không phải một phép đo.

Tệ hơn: lượt ấy còn để lại một con số sai lan vào tài liệu khác — "cả chín hòm
đều đã có số đo", trong khi `crates/` có **mười một** hòm.

---

## Mười cái bẫy — chín cái làm con số sai, cái thứ mười làm KẾT LUẬN sai

Đây là phần đáng đọc nhất của tài liệu này. Cả mười đều đã trả giá thật.

### 1. Hết đĩa giữa chừng đọc y hệt "bộ thử vô dụng"

25/08/2026: lượt chạy trả về **61 `TIMEOUT`, 0 `MISSED`**. Nhìn lướt thì đó là
một bộ thử chẳng bắt được gì. Nguyên nhân nằm ở **cuối** đầu ra, sau hàng chục
dòng TIMEOUT: `No space left on device`.

Mỗi việc song song là một **bản sao cả cây** kèm `target/` riêng, ~1,8 GB một
cái. Bản dựng chết vì đầy đĩa, và mọi đột biến còn lại "hết giờ".

**Chốt:** kịch bản gọi `df` **trước mỗi hòm** và dừng hẳn dưới ngưỡng, thay vì
chạy tiếp rồi sinh ra một bảng số đẹp đẽ và sai.

### 2. `--timeout-multiplier` nhân nhầm thời gian

Nó nhân thời gian bộ thử **của hòm đang bị đột biến**, không phải của lượt chạy
thật. Nhân thời gian `tcc-spec` (~2 giây) rồi áp cho lượt chạy cả không gian làm
việc (~70 giây) là đặt hạn giờ **ngắn hơn** phép thử — mọi thứ "hết giờ".

**Chốt:** `--timeout 300`, số tuyệt đối.

### 3. Không bật cờ thì mã sau cờ KHÔNG được biên dịch

Trọng tài mặc định không bật cờ nào, nên mã sau `#[cfg(feature)]` không được
biên dịch, phép thử của nó không chạy, và công cụ ghi **mọi** đột biến ở đó là
"sống sót". `tcc-chain` báo **45 sống**; bật `--features import-web-wallet` thì
con số thật là **25**. Hai mươi con số kia là *chưa nhìn tới* đội lốt *không bắt
được*.

**Chốt:** kịch bản giữ sẵn cờ cho từng hòm, rút từ `.github/workflows/ci.yml`.

### 4. Trọng tài hẹp: hòm dưới mới là chỗ có phép thử

`cargo mutants -p X` chạy trọng tài là phép thử của **riêng** hòm X. Hòm thư
viện mà hành vi được kiểm ở hòm dưới — `tcc-ui` được `tcc-render-raster` kiểm —
thì mọi đột biến ấy báo "sống sót" trong khi phép thử thật sự **có** bắt được.

Cùng hình dạng với bẫy 3, chỉ khác chỗ nấp.

> ⚠️ **Hệ quả: mọi con số "sống sót" trong bảng dưới là CẬN TRÊN, không phải số
> thật.** Nói ra điều này là bắt buộc; không nói thì bảng lại là một phép đo nói
> quá về chính nó.

**Chốt:** `cargo-mutants` có cờ `--test-package` (đột biến ở hòm này, chạy phép
thử của hòm kia). Đánh đổi: chậm hơn nhiều. Cách đang dùng: giữ trọng tài hẹp
để quét, rồi **chốt lại từng kẻ sống sót** bằng bộ thử rộng.

### 5. Vài hòm KHÔNG quét tự động được, và phải nói ra

- `tcc-keystore --features os-keystore` ghi vào **Keychain thật** rồi gọi
  `unlock`; macOS bật hộp thoại xin quyền và lượt chạy chờ một cú bấm không bao
  giờ tới. Đã treo hơn bốn mươi phút một lần.
- `tcc-shell` với cờ `wallet`/`window` **mở cửa sổ**.

Bỏ im lặng thì bảng nói dối mà không câu nào sai. Kịch bản bắt phải ghi rõ
`BỎ:` kèm lý do, và lý do ấy in ra trong bảng.

### 6. Đếm tiến trình bằng mẫu sai → chạy chồng hai lượt

26/08/2026: tôi đếm bằng `pgrep -f "cargo mutants"` — **dấu cách** — trong khi
tên thật là `cargo-mutants` — **gạch nối**. Phép đếm trả về 0, tôi đọc thành "đã
chết", và chạy lại chồng lên. Hai lượt cùng quét một hòm, tranh một khoá, ghi đè
tệp đầu ra của nhau; bản ghi của cả hai hỏng.

Đây là bẫy chung của cả dự án, chép lại từ `CLAUDE.md`: **một phép đo không phân
biệt được "thứ tôi sợ đã xảy ra" với "phép đo của tôi không chạm tới thứ ấy" thì
không phải một phép đo.**

**Chốt:** một thư mục khoá (`mkdir` là thao tác nguyên tử) — tạo được thì chạy.
Không phụ thuộc vào việc đoán đúng tên tiến trình.

### 7. Đọc tiến độ từ màn hình — nơi chỉ hiện tin xấu

`cargo-mutants` chỉ in ra màn hình những kết quả **đáng lo**: `MISSED`,
`TIMEOUT`, `UNVIABLE`. Đột biến **bị bắt** thì nó im lặng.

26/08/2026 tôi đếm số dòng ấy giữa chừng, chia cho tổng số đột biến, và đọc ra
"142 đã chạy, 138 sống sót" — một tỷ lệ chỉ có thể có nếu bộ thử hỏng. Số thật
nằm trong `mutants.out/`: **313 bị bắt**, 141 sống, 3 hết giờ, 19 không dựng
được, tức 476 đã xử lý và tỷ lệ bắt khoảng 69%.

Nguồn đúng để đếm là bốn tệp `caught.txt` / `missed.txt` / `timeout.txt` /
`unviable.txt`, **không phải** dòng chảy trên màn hình.

Đây là bẫy 6 mặc áo khác: phép đếm không phân biệt được *"không có gì bị bắt"*
với *"cái bị bắt không được in ra"*. Cùng một ngày, cùng một người, ba lần.

### 8. Bộ cờ TỐI ĐA giấu đúng những bất biến sống nhờ cờ TẮT

Hai mặt của cùng một cái bẫy, cả hai đều gặp ngày 26/08/2026 ở `tcc-shell`.

**Mặt một — cờ THIẾU thì tệp không được biên dịch.** `wallet_flow` khai sau
`all(window, import-web-wallet, os-keystore)`. Quét với MỘT cờ thì tệp ấy không
hề được biên dịch; `cargo-mutants` vẫn sinh đột biến từ nguồn, đột biến không
có tác dụng, phép thử xanh, và bảng ghi **18 dòng MISSED** không có thật. Quét
lại với `--features wallet`: 52 kẻ sống co còn **34**, tỷ lệ bắt 60% → **71%**.

**Mặt hai — cờ ĐỦ thì bất biến sống nhờ cờ TẮT trở nên vô hình.**
`permission_dialog::cap_duoc` là:

```rust
Scope::Wallet { .. } => cfg!(feature = "wallet"),
Scope::Network { .. } | Scope::Storage { .. } => true,
```

Bật `wallet` thì MỌI nhánh trả `true`, nên thay cả hàm bằng `true` là **đột biến
tương đương** — không phép thử nào phân biệt được, và không nên cố. Nhưng bất
biến B62 ("bản dựng không có ví thì KHÔNG hỏi người dùng về tiền") sống đúng
trong cấu hình NGƯỢC LẠI. Đo bằng tay:

| Cấu hình | Đột biến `cap_duoc → true` |
|---|---|
| `cargo test -p tcc-shell` (không cờ) | **5 phép thử ĐỎ** |
| `--features wallet` (lượt quét chạy) | xanh hết — tương đương |

**Chốt:** hòm nào có bất biến sống nhờ một cờ TẮT thì phải quét **nhiều cấu
hình**, và cấu hình đáng giá thường là cấu hình TỐI THIỂU. Chạy một lượt với bộ
cờ tối đa rồi tuyên bố "đã phủ" là bỏ sót đúng lớp bất biến mà một bản dựng cắt
gọn sinh ra để bảo vệ.

### 9. Suy từ `#[cfg]` không thay thế được phép ĐO

Bảy bẫy trên đều là công cụ đo sai. Bẫy này thì công cụ không sai — **tôi không
dùng công cụ**.

27/08/2026, phân loại kẻ sống sót của lượt tối thiểu, tôi nhìn dòng
`#[cfg(feature = "window")]` trên `window_raster.rs` rồi kết luận "không được
biên dịch, nên 25 kẻ sống ở đó là hiện vật", và ghi **53/62 là hiện vật** vào hồ
sơ như một phép đo. Đo thật thì:

| | Hiện vật | Thật |
|---|---|---|
| Suy từ `#[cfg]` | 53 | 9 |
| Đọc dep-info | **28** | **34** |

`cargo build -p tcc-shell` KHÔNG kéo `window_raster.rs` vào, nhưng
`cargo test --no-run -p tcc-shell` **CÓ** — lượt dựng phép thử kéo nhiều hơn
lượt dựng thường, và trọng tài chính là lượt phép thử.

**Chốt:** `tools/loc-hien-vat.sh` dựng bằng ĐÚNG lệnh trọng tài dùng
(`cargo test --no-run`) rồi đọc dep-info `.d`. Đúng theo cấu tạo, không đúng nhờ
may.

### Ba hạng, ba cách xử lý khác nhau

Bẫy 9 đẻ ra một hạng chưa có tên ở mục "Đọc bảng cho đúng" bên dưới:

| Hạng | Dấu hiệu | Làm gì |
|---|---|---|
| **Hiện vật** | tệp KHÔNG có trong dep-info của lượt ấy | sửa CÔNG CỤ (bật cờ, đổi cấu hình) |
| **Phép thử yếu** | tệp có, phép thử có, mà đột biến vẫn sống | VIẾT phép thử |
| **Chưa chạm tới được** | tệp có, nhưng hàm cần cửa sổ/Keychain thật | TÁCH phần thuần ra trước — như đã làm cho tầng `Phien` |

Gộp ba hạng làm một là cách nhanh nhất để vá nhầm chỗ.

### 10. Xếp nhầm vào "tương đương" — sai theo hướng ĐÓNG hồ sơ

Chín bẫy trên đều làm con số SAI. Bẫy này làm con số đúng mà **kết luận** sai,
và nó nguy hiểm hơn: báo động nhầm thì tốn công kiểm lại, còn xếp nhầm vào
"tương đương" thì **đóng hồ sơ và không ai mở ra nữa**.

27/08/2026, mệnh đề chắn của `hit_test`:
`if x < 0 || y < 0 || x >= rong || y >= height { return None; }`

Tôi lập luận: đột biến ở đó không đổi được kết quả, vì điểm ngoài ảnh vẫn bị
phép kiểm hình chữ nhật loại; nó chỉ đổi kết quả khi có ô **tràn ra ngoài ảnh**,
mà bố cục không còn sinh ra trạng thái ấy (lỗi F1 đã vá, có phép thử canh).

Từng bước đều đúng. Kết luận sai — vì tôi chỉ nghĩ tới **một** đường sinh ra
ô-ngoài-ảnh. Đường thứ hai nằm ngay trong tên hàm: **`set_width`**. Nó tồn tại
chính vì cửa sổ kéo được, và giữa lúc kéo với lúc vẽ lại, **mọi** ô đã đặt đều
có thể nằm ngoài ảnh. Đó là vận hành bình thường, không phải lỗi.

Phép thử giết được cả 14 kẻ chỉ cần ba dòng: vẽ ở 640, `set_width(320)` mà chưa
vẽ lại, rồi bấm vào ô cũ ở x > 320.

**Chốt — ba câu hỏi trước khi ghi "tương đương":**

1. Tôi đã liệt kê **mọi** đường sinh ra trạng thái phân biệt được chưa, hay chỉ
   đường đầu tiên nghĩ ra?
2. Có **hàm public nào** đưa hệ thống vào trạng thái ấy không? (`set_width` là
   một hàm `pub`.)
3. Chứng minh của tôi là **đọc mã** hay là "tôi không nghĩ ra cách"? Chỉ cái đầu
   mới đủ.

> **"Tôi không nghĩ ra cách kích hoạt" KHÔNG phải "không thể kích hoạt".**

Hai kẻ ở `set_width` thì ngược lại: chứng minh được bằng đọc mã — `<` thành `<=`
tại đúng biên trả về cùng một giá trị vì nhánh `else` trả về chính `rong`. Đó
mới là tương đương thật, và nó đã được ghi ngay tại hàm.

---

## Đọc bảng cho đúng

**"Sống sót" ≠ "lỗ hổng".** Một đột biến sống sót có ba khả năng:

| | |
|---|---|
| **Lỗ thật** | phép thử yếu, và không chỗ nào khác canh |
| **Giới hạn trọng tài** | chỗ khác CÓ canh, chỉ là trọng tài không chạy tới (bẫy 4) |
| **Đột biến tương đương** | mã sau khi đổi vẫn **đúng bằng** mã cũ về mặt toán học; không phép thử nào phân biệt được, và **không nên** cố |

Ví dụ tương đương, đã ghi trong `SECURITY.md` §3.27: trong `mnemonic.rs`, `|`
thành `^` khi đóng gói bit. Sau `bit << 11` thì mười một bit thấp bằng 0 và mọi
chỉ số nhỏ hơn 2¹¹, nên hai toán tử đồng nhất.

Gộp cả ba hạng thành một con số rồi báo động là cách nhanh nhất để lần sau không
ai đọc bảng này nữa — và một hồ sơ không ai đọc thì đúng bằng không có hồ sơ.

Nên **mỗi kẻ sống sót phải được tra tận nơi**, và câu hỏi bắt buộc là:

> **Chỗ khác có canh không?**

Câu ấy đã cứu một lần và bị bỏ sót một lần trong cùng một buổi. Xem
`SECURITY.md` §3.31.

---

## Kết quả — cập nhật 27/08/2026, ĐỦ MƯỜI MỘT HÒM

Bảng dựng bằng `tools/bang-dot-bien.sh`, đọc từ bốn tệp trong `mutants.out/`
chứ không từ dòng chảy màn hình (bẫy 7).

| Hòm | Đột biến | Bị bắt | Sống sót | Hết giờ | Không dựng được | Tỷ lệ bắt |
|---|---:|---:|---:|---:|---:|---:|
| `tcc-capability` | 24 | 19 | **0** | 0 | 5 | 100% |
| `tcc-chain` | 144 | 123 | **7** | 0 | 14 | 94% |
| `tcc-crypto` | 34 | 30 | **3** | 0 | 1 | 90% |
| `tcc-keystore` | 29 | 20 | **5** | 0 | 4 | 80% |
| `tcc-manifest` | 11 | 9 | **0** | 0 | 2 | 100% |
| `tcc-net` | 19 | 8 | **4** | 0 | 7 | 66% |
| `tcc-render-raster` | 622 | 421 | **172** | 3 | 26 | 70% |
| `tcc-runtime` | 49 | 37 | **0** | 0 | 12 | 100% |
| `tcc-shell` | 0 | 0 | **0** | 0 | 0 | 0% |
| `tcc-spec` | 98 | 94 | **2** | 0 | 2 | 97% |
| `tcc-ui` | 83 | 58 | **2** | 0 | 23 | 96% |

⚠️ **`tcc-render-raster` là số đo DANG DỞ.** Phiên Claude Code trước thoát và
kéo theo lượt quét ở **619/665 (93%)**; 46 đột biến cuối chưa đo. Con số cuối có
thể xê dịch, nhưng chiều thì rõ: trước khi vá là 392 bắt / 235 sống (~62%).

### Đã đo lại sau khi vá — và mỗi lần đều xác nhận hoặc bác bỏ một điều

| Hòm | Trước | Sau | Ghi chú |
|---|---|---|---|
| `tcc-keystore` | 44% (14 sống) | **80%** (5 sống) | ba lượt đo trong một buổi — hòm 29 đột biến nên vòng vá→đo dùng được thật |
| `tcc-chain` | 93% (9 sống) | **94%** (7 sống) | đúng bằng con số dự đoán TRƯỚC khi đo |
| `tcc-shell` | 60% (52 sống) | **71%** (34 sống) | phần lớn mức tăng là do sửa CỜ, không phải do vá |
| `tcc-render-raster` | ~62% (235 sống) | ~70% (172 sống, dang dở) | |

**Năm kẻ còn lại của `tcc-keystore` đều đòi ENTITLEMENT** — cất/đọc khoá thật
chỉ chạy trong gói `.app` đã ký. Đó là trần thật của `cargo test` với một nhị
phân không ký, và là chỗ cần NGƯỜI chứ không phải mã.

## Phát hiện — mỗi kẻ sống sót đã được tra tận nơi

Trong **68** kẻ sống sót ngoài `tcc-render-raster`, và **235** trong hòm ấy,
sau khi đọc mã từng chỗ:

### Lỗ thật, xếp theo mức đáng ngại

| # | Chỗ | Vì sao đáng kể |
|---|---|---|
| 1 | **Biên của `hit_test`** (`tcc-render-raster`, 23 kẻ) | `&&`→`\|\|` và `<`→`<=` đều sống: chưa ai thử điểm nằm trong dải ngang mà ngoài dải dọc, cũng chưa ai thử cạnh chung của hai ô kề nhau. Cả mô hình quyền dựa trên việc bấm ĐÚNG một ô |
| 2 | **Tầng `Phien`** (`tcc-render-raster/window.rs`, 14 kẻ) | Xoá HẲN sáu phương thức mà vẫn xanh. **Năm** thử được ngay (`xoa_lui`, `xoa_tai_cho`, `doi_con_tro`, `con_tro_ve_dau_hoac_cuoi`, `ket_man` — chỉ nhận `&mut self`); ba cái còn lại (`cuon`, `chuot_toi`, `doi`) NHẬN `&tao::window::Window` nên phải tách phần thuần ra trước. Đúng những hành vi `thu-tay.md` đang nhờ NGƯỜI bấm tay |
| 3 | **`Debug for ImportedWallet`** (`tcc-chain`) | Giữ hạt giống + cụm từ khôi phục; §3.27 vá đúng lớp lỗi này cho `WalletSecret` rồi dừng ở đó |
| 4 | **`cat_khoa` / `cat_hat_giong` chưa từng được gọi** (`tcc-shell/wallet_flow.rs`) | Thay cả thân bằng `Ok(())` vẫn xanh — tức *"đã lưu ví của bạn"* trong khi không lưu gì. Đây là khoảng trống §3.28 ĐÃ GHI (đường thật ghi vào Keychain và bật hộp thoại xin quyền), được phép đo xác nhận, **không phải phát hiện mới** |
| 5 | **`do_net`** (`tcc-render-raster`, 15 kẻ) | Hàm đo biên mực thật của glyph, tồn tại vì bảng số liệu phông nói dối về dấu chồng tiếng Việt. Trả `(0,0)` luôn vẫn xanh — mà "chữ tiếng Việt dựng đúng" là lời hứa đầu bảng của dự án (câu hỏi 0.1) |
| 6 | **`has_mnemonic`** (`tcc-chain`) | Chỉ nhánh `true` từng chạy; giao diện rẽ theo nó để nói với người dùng có/không có cụm từ khôi phục |

> ⚠️ **Bảng `tcc-shell` ở trên là bản ĐO LẠI.** Lượt đầu chạy với một cờ, nên
> `wallet_flow.rs` không được biên dịch và 18 "kẻ sống sót" ở đó là hiện vật của
> phép đo hỏng (bẫy 8, mặt một). Lỗ số 4 tôi báo cáo lần đầu — "đường hỏng của
> kho khoá không ai canh" — **đã rút lại**: đường ấy không tồn tại trong cấu
> hình có `wallet` trên macOS. Thứ còn đúng là khoảng trống §3.28 đã ghi sẵn.

### Trạng thái vá, 27/08/2026

Năm trên sáu đã vá. **Mỗi bản vá đều tự tay áp lại đúng đột biến nó nhắm và
chứng minh là ĐỎ được** — một phép thử chưa từng thấy đỏ không phải bằng chứng.

| Lỗ | Đột biến bị giết | Phép thử |
|---|---|---|
| 1. biên `hit_test` | 3/3 | `hit_test_dung_o_bien` |
| 2. tầng `Phien` | 5/5 | `phien_sua_chu_dung_o_muc_ky_tu`, `ket_man_mang_theo_noi_dung_o_nhap` |
| 3. `Debug for ImportedWallet` | 1/1 | `debug_vi_da_nhap_khong_lo_hat_giong_lan_cum_tu` |
| 5. `do_net` | **3 giết + 1 tương đương** | `o_chua_duoc_net_that_cua_dau_tieng_viet` |
| 6. `has_mnemonic` | 1/1 | `ban_ghi_khong_cum_tu_thi_bao_khong` |
| 4. kho khoá | — | **RÚT LẠI**, xem trên |

**Cái 3/4 đã giải thích được, 27/08/2026 — nó là đột biến TƯƠNG ĐƯƠNG.** Phép
thử `do_net` giết cả chín biến thể "trả về hằng số"; đột biến đảo bộ lọc điểm
ảnh trong suốt thì không, và lý do đo được chứ không đoán:

- Bộ lọc KHÔNG phải mã chết — đếm được nó chặn **228** điểm ảnh khi đo `x` và
  `ẾỒỖ`.
- Nhưng `do_net` chỉ theo dõi **dải hàng** (`tren.min(y)`, `duoi.max(y + h)`),
  và điểm ảnh trong suốt nằm CÙNG những hàng với điểm ảnh có mực. Đổi bộ lọc là
  đổi tập điểm ảnh được xét, **không đổi dải hàng**.
- Đo cả hai bản: `x` → `(9,17)`, `ẾỒỖ` → `(2,17)`, giống nhau từng số.

Nên `do_net` là **4/4 đã xử lý**: ba giết được, một là tương đương có bằng
chứng. Ghi kèm giới hạn: kết luận này đo trên phông và bộ dựng glyph hiện tại;
bộ dựng nào phát cả hàng rỗng trên/dưới glyph thì hai bản sẽ khác nhau.

Lần đầu tôi ghi mục này là "3/4, không giết được, chưa giải thích được". Giữ lại
câu ấy trong lịch sử vì nó đúng lúc viết — và vì khoảng cách giữa "không giết
được" và "không THỂ giết được" là khoảng cách giữa một khoảng trống và một kết
luận.

### KHÔNG phải lỗ — và vì sao phải nói ra

| Hạng | Ví dụ | |
|---|---|---|
| **Giới hạn trọng tài** | `SpecError::ma`, `ContentHasher` | Bộ kiểm định tuân thủ CÓ canh, có chủ đích. `tcc-conformance/src/main.rs:119` so băm theo luồng với băm một phát VÀ với vector — ba chiều |
| **Xác nhận lời tự khai** | `HttpNetwork::get`, `JsonRpc::call`, `cat_khoa` | Đúng những chỗ `SECURITY.md` §3 đã ghi là chưa chứng minh được. Phép đo độc lập trùng với lời tự nhận — đó là bằng chứng mục §3 không phải văn suông |
| **Đột biến tương đương** | `mnemonic.rs` `\|` → `^` | §3.27 để sống có chủ đích: sau `bit << 11` hai toán tử đồng nhất |
| **Tương đương THEO CẤU HÌNH** | `permission_dialog::cap_duoc → true` | Bật `wallet` thì mọi nhánh trả `true` nên không phân biệt được; TẮT `wallet` thì đúng đột biến ấy làm **5 phép thử đỏ**. Bất biến B62 sống trong sự VẮNG MẶT của cờ — xem bẫy 8 |
| **Toàn vẹn thị giác** | `ve_o`, `khung`, `co_khung` | Hình dạng CÓ phép thử canh (B31, B51, B52); biên chính xác thì không |

> ⚠️ Câu này thoạt đầu tôi viết là *"`Phien` KHÔNG dính cửa sổ nên cả sáu đều
> thử được"*. Đúng nửa: `struct Phien` không GIỮ cửa sổ, nhưng ba phương thức
> NHẬN một cái. Tôi phát hiện lúc ngồi viết bản vá và thấy chữ ký hàm khác thứ
> mình tưởng — tức thứ cứu tôi là việc BẮT TAY LÀM, không phải việc đọc lại.
>
> Bằng chứng sau khi sửa còn **mạnh hơn**: `window.rs` có phép thử tên
> `cuon_bi_kep_o_ca_hai_dau`, mà `replace Phien::cuon with ()` vẫn sống. Phép
> thử ấy canh hàm kẹp THUẦN; dây nối từ sự kiện tới hàm ấy thì không ai canh.

**Một điều ghép từ hai phát hiện:** lỗ 1 (`hit_test` không canh biên) và cụm
`ve_o` (biên hình vẽ không ghim) là **hai nửa của cùng một mối nguy** — cái VẼ
RA và cái BẤM ĐƯỢC có thể trôi khỏi nhau mà mọi cổng vẫn xanh. Chính mã ấy đã
ghi rủi ro này ở `lib.rs:1811`, dưới dạng **chú thích**, không phải phép thử.
