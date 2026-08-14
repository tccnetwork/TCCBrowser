# Bảo mật — TCC Browser

> Tài liệu cho **đội kiểm định**. Ghi rõ ba thứ: các bất biến phải giữ, những lỗ
> đã tự tìm ra và sửa, và **những gì CHƯA được soi**. Mục cuối quan trọng nhất —
> một tài liệu bảo mật chỉ liệt kê thành tích là tài liệu gây hiểu nhầm.

Cập nhật: 13/08/2026 · Phạm vi: `tcc-crypto`, `tcc-spec`, `tcc-manifest`,
`tcc-capability`, `tcc-ui`, `tcc-render-webview`, `tcc-runtime`, `tcc-shell`

---

## 1. Bất biến — hỏng cái nào là hỏng cả kiến trúc

| # | Bất biến | Giữ bằng gì |
|---|---|---|
| B1 | Chữ ký lai chỉ hợp lệ khi **CẢ HAI** nửa hợp lệ | `gia_mao_nua_co_dien`, `gia_mao_nua_hau_luong_tu` |
| B2 | Chữ ký bao trùm **cả bản kê khai lẫn nội dung** | `thay_ruot_giu_chu_ky_thi_hong` |
| B3 | Quyền năng **không tồn tại** cho tới khi được cấp | Trường riêng tư + doctest `compile_fail` |
| B4 | Thu hồi giết **mọi bản sao** đang cầm trong tay | `thu_hoi_giet_ca_ban_sao_dang_cam` |
| B5 | Tên miền khớp **chính xác**, không khớp hậu tố | `khong_khop_ten_mien_con_va_khong_khop_hau_to` |
| B6 | Chuỗi hiện cho người dùng **không chứa ký tự giả mạo** | `ten_ung_dung_co_ky_tu_dao_chieu_thi_tu_choi` |
| B7 | Chữ của ứng dụng **không thoát ra được** khỏi tài liệu bộ dựng | `chu_cua_ung_dung_khong_thoat_ra_duoc_tai_lieu` + ví dụ `kiem-khoi-tan-cong` |
| B8 | Cây trợ năng bộ dựng công bố **khớp** cây gốc | `check_accessibility_parity` + `nhan_khac_chu_hien_ra_thi_bao_loi` |
| B9 | Giao diện **không dựng nổi** một nút thiếu vai trò/nhãn | Kiểu dữ liệu: `Alt::Decorative` phải khai ra miệng |
| B10 | Ứng dụng **không đặt được màu**, chỉ khai ý định | `Tone` là enum kín, không có trường màu |
| B11 | Bỏ qua bước kiểm chữ ký là **không biên dịch được** | `grant_verified` chỉ nhận `VerifiedApp` |
| B12 | Mọi đường không rõ ràng đều ra **TỪ CHỐI** | `moi_duong_khong_ro_rang_deu_ra_tu_choi` |
| B13 | **Bộ dựng nối sự kiện, ứng dụng không bao giờ** | `script-src 'none'` + kịch bản khởi tạo của bộ dựng |
| B14 | Hành động ma **không đi tiếp được** | Danh sách trắng + `kiem-bam-nut ma` |
| B15 | Ứng dụng **không ship thẻ đánh dấu**, chỉ ship cây khai báo | `entry` là `ui.json`; `tcc-ui` không lộ khái niệm web |
| B16 | Giải mã từ đĩa **không đi vòng qua hàm dựng** | `UiNode` là kiểu riêng; `TryFrom` dựng lại qua hàm dựng có kiểm |
| B17 | Hỏi quyền **theo từng mục**, công tắc mặc định TẮT | `moi_cong_tac_mac_dinh_tat`, `bam_cho_phep_ma_khong_bat_gi_thi_khong_cap_quyen_nao` |
| B18 | Công tắc ma **vứt cả thông điệp**, không lọc bớt | `doc_tra_loi` + `kiem-bam-nut ct-ma` |
| B19 | **Quyết định đầu tiên thắng**, không cho ghi đè | Kiểm `o.is_none()` trong bộ nhận IPC |
| B20 | Hành vi của nút khai trong **bản kê khai đã ký**, không trong `ui.json` | `Manifest::actions` nằm trong phạm vi chữ ký |
| B21 | Hành vi **không xin được** cái quyền năng chưa cho | `kiem_hanh_vi` + `hanh_vi_goi_may_chu_chua_xin_quyen_thi_tu_choi` |
| B22 | **Không một gói tin nào rời khỏi máy** khi chưa cấp quyền | `chua_cap_quyen_thi_khong_goi_ra_ngoai_mot_lan_nao` |
| B23 | **KHÔNG đi theo chuyển hướng** — đó là đòn thoát khỏi quyền năng | `max_redirects(0)` + `moi_chuyen_huong_deu_bi_tu_choi` |
| B24 | Chỉ HTTPS, có thời gian chờ, có trần kích thước | `tcc-net` |
| B25 | Đường ra ngoài **nhìn thấy được trong cây phụ thuộc** | Luật 8: chỉ `tcc-shell` phụ thuộc `tcc-net` |
| B26 | Quyền đã nhớ gắn với **khoá người ký**, không chỉ mã ứng dụng | `doi_khoa_nguoi_ky_thi_phai_hoi_lai` |
| B27 | Quyền đã nhớ gắn với **phạm vi**, không chỉ tên quyền | `noi_rong_pham_vi_thi_phai_hoi_lai` |
| B28 | Kho quyền hỏng → **hỏi lại**, không ngả về cho phép | `tep_hong_hoac_phien_ban_la_thi_hoi_lai` |
| B29 | Khoá ký đổi → **cảnh báo**, và cảnh báo đứng TRƯỚC danh sách quyền | `doi_khoa_ky_thi_canh_bao_hien_ra_truoc_danh_sach_quyen` |
| B30 | Nhãn điều khiển **hiện ra cho người nhìn**, không chỉ `aria-label` | `nhan_cong_tac_hien_ra_cho_nguoi_nhin_thay`, `nhan_o_nhap_hien_ra_cho_nguoi_nhin_thay` |
| B31 | Sắc thái **mất mát được vẽ khác đi**, không chỉ khai ra | `sac_thai_mat_mat_duoc_ve_khac_di` |
| B32 | Ô nhập **KHÔNG mang ARIA role** — ARIA đè lên ngữ nghĩa gốc | `moi_nut_deu_mang_vai_tro_ro_rang` |
| B33 | Tín hiệu "mất mát" **lên được trục trợ năng của hệ điều hành** | `nut_mat_mat_mang_cau_canh_bao` |
| B34 | Tệp trong gói phục vụ qua **giao thức riêng**, đường dẫn qua đúng `check_path` | `duong_dan_di_ra_ngoai_goi_bi_chan` |
| B35 | Chỉ phục vụ **ảnh**, theo danh sách TRẮNG đuôi tệp — **không có SVG** | `chi_phuc_vu_anh`, `svg_khong_duoc_phuc_vu` |
| B36 | Hộp thoại hỏi quyền **không phục vụ tệp nào** của ứng dụng | `mo(..., \|_\| None, ...)` |
| B37 | Quyết định **không đọc phần mô tả** lưu trên đĩa | `quyet_dinh_khong_doc_phan_mo_ta` |
| B38 | Nút mất mát **không giãn kín bề ngang** | `nut_khong_gian_kin_be_ngang` |
| B39 | **Dấu hiệu MÁY tách khỏi chữ CHO NGƯỜI** — chữ dịch được, dấu hiệu không đổi | `doi_chu_sang_ngon_ngu_khac_khong_lam_mat_dau_hieu_may` |

**B39 — vì sao một câu cảnh báo lại không dịch được suốt mấy phiên.**

Bộ quét trợ năng nhận biết "hành động mất mát" bằng cách so `aria-description`
với đúng chuỗi `"Hành động không hoàn tác được."`. Chuỗi đó vừa là **chữ cho
người đọc** vừa là **dấu hiệu cho máy nhận biết**.

Hệ quả: dịch nó sang tiếng Anh là bộ quét mù, phép kiểm định trợ năng đỏ, và
`Tone::Danger` mất hết ý nghĩa. Nên nó bị khoá cứng ở tiếng Việt — trong một
giao diện mặc định tiếng Anh.

**Gộp hai vai trò vào một chuỗi luôn khoá chặt nó lại như vậy.** Nay tách hẳn:

| | |
|---|---|
| Dấu hiệu máy | `data-sac-thai="mat-mat"` — không bao giờ đổi, không bao giờ hiện ra |
| Chữ cho người | tiêm từ `tcc-shell` xuống, tự do dịch |

Bộ dựng **không biết ngôn ngữ và không nên biết** — bảng dịch nằm ở `tcc-shell`,
và `loi::chu_bo_dung()` là cửa duy nhất đưa chữ xuống. Cùng lối đã dùng với
`trait Mang` và trình phục vụ tệp: thứ gì phụ thuộc ngữ cảnh thì tiêm từ ngoài.

Mặc định của bộ dựng là **tiếng Anh**, đúng mặc định của cả giao diện. Đo được
trên trục trợ năng thật: `"Xoá dữ liệu, button — this cannot be undone"` — nhãn
nút vẫn tiếng Việt vì đó là chữ **của ứng dụng**, dịch hộ là nói thay cho nó.

**B37 — màn hình quản lý quyền, và một lời nói dối phải chặn đường.**

"Cấp" mà không kèm "xem lại và thu hồi" chỉ là nửa hệ thống quyền. Nhưng màn
hình đó cần hiện `shop.tcc-coin.com`, mà kho chỉ lưu **vân tay** phạm vi — vân
tay không đọc ngược ra được. Nên phải lưu thêm chữ mô tả.

Chữ lưu trên đĩa là chữ **sửa được**. Ai sửa tệp có thể làm màn hình hiện
"shop.tcc-coin.com" trong khi vân tay ứng với một phạm vi khác hẳn — tức là màn
hình quản lý quyền **nói dối**, đúng cái màn hình mà lời nói dối gây hại nhất.

Ta chấp nhận rủi ro hiển thị (nó nằm trong mô hình đe doạ đã ghi: người sửa được
tệp đã chiếm được tài khoản). Nhưng phải chặn đường nó ảnh hưởng tới **QUYẾT
ĐỊNH**: `tra()` chỉ đọc vân tay, không bao giờ đọc mô tả. Phép thử sửa mô tả
trên đĩa thành một phạm vi khác hẳn và đòi quyết định **không đổi**.

**B38 — một nút mất mát to bằng cả màn hình là một cái bẫy.**

Trong hộp xếp dọc, phần tử con mặc định giãn hết bề ngang. Nút "Quên ứng dụng
này" chiếm trọn màn hình — dễ bấm nhầm, và nó xoá thứ người dùng đã quyết định.
Cùng gốc với lỗi ảnh 8×8 bị kéo giãn: **bảng kiểu tối giản vẫn có hành vi mặc
định, và hành vi mặc định vẫn phải được xem xét.**

**B34/B35/B36 — trình phục vụ `tcc-goi:`, một ĐƯỜNG MỚI vào nội dung gói.**

Trước đó, ảnh trong gói **không bao giờ hiện**: `ui.json` khai đường dẫn tương
đối, còn tài liệu nạp bằng `with_html` không có địa chỉ gốc nên không phân giải
được. Sửa nó nghĩa là thêm một trình phục vụ nhận **địa chỉ do trang yêu cầu** —
tức là mở một bề mặt tấn công mới. Ba luật:

| Luật | Chặn cái gì |
|---|---|
| Đường dẫn qua đúng `check_path` của tiêu chuẩn | `../` đi ra ngoài gói |
| Chỉ trả tệp CÓ TRONG cây đã ký | nội dung ngoài phạm vi chữ ký |
| Kiểu nội dung từ DANH SÁCH TRẮNG theo đuôi | ép trình duyệt coi một tệp là HTML |

Ba chi tiết dễ bỏ sót, đều có phép thử:

1. **Giải mã phần trăm TRƯỚC khi kiểm.** `%2e%2e%2f` chính là `../` viết trá
   hình. Không giải mã thì `check_path` không nhìn thấy, mà trình duyệt thì có.
2. **Cắt phần truy vấn và phần neo trước khi kiểm.** Không cắt thì chuỗi đem
   kiểm khác chuỗi đem tra cứu.
3. **Không có SVG trong danh sách trắng.** SVG chạy được kịch bản và nhúng được
   tài nguyên ngoài — nó là một tài liệu, không phải một tấm ảnh.

**B36**: hộp thoại hỏi quyền là màn hình CỦA TRÌNH DUYỆT, nên nó truyền
`|_| None` — ứng dụng không đưa được một byte nào vào đó. Cho phép là mở đường
vẽ đè lên chính câu cảnh báo.

Từ chối thì trả **404 rỗng**, không kèm thông báo: nói chi tiết là cho trang
biết cái gì có và cái gì không có trong gói.

**B32/B33 — soi cây trợ năng THẬT tìm ra hai lỗi nữa (13/08/2026).**

Sau khi có quyền Trợ năng, soi được cây mà VoiceOver thật sự thấy. Hai phát hiện,
và **lỗi thứ nhất do chính bản vá trước của tôi gây ra**:

| # | Lỗi | Nguyên nhân |
|---|---|---|
| 1 | Ô mật khẩu ra `AXTextField` thường | Tôi thêm `role="textbox"` cho bất biến "mọi nút mang vai trò rõ ràng" — **ARIA đè lên ngữ nghĩa gốc** và kéo `AXSecureTextField` tụt xuống |
| 2 | Nút mất mát không mang tín hiệu gì | `aria-description` **không lên được** trục trợ năng của macOS |

Lỗi 1 là bài học đắt: một bất biến tôi thêm vào để tăng an toàn lại **làm hỏng
đúng thứ nó định bảo vệ**. Luật ARIA số một — *đừng dùng ARIA khi thẻ gốc đã nói
đúng* — tôi biết mà vẫn vi phạm, vì đuổi theo một con số đếm được.

Bất biến đã sửa: **ô nhập KHÔNG được mang `role`**, mọi loại khác thì có. Công
tắc vẫn giữ `role="switch"` vì đó là ARIA **nâng cấp** (ô đánh dấu → công tắc),
dùng đúng chỗ.

Lỗi 2 sửa bằng `title` (→ AXHelp) cộng `aria-roledescription` (→ AXRoleDescription).
Lưu ý `aria-roledescription` **thay thế** tên vai trò, nên chuỗi của nó phải tự
nhắc đây là nút — nếu không người dùng mất thông tin đó.

Đo được sau khi sửa, đây là câu VoiceOver sẽ đọc:

```
"Gõ thử tiếng Việt, text field"
"Ô bí mật (chữ phải bị che), secure text field"
"Tải trang mẫu, button"
"Xoá dữ liệu, nút — hành động không hoàn tác được"
```

**⚠️ NỢ ĐÃ GHI**: câu cảnh báo cứng bằng tiếng Việt trong bộ dựng, trong khi giao
diện mặc định là tiếng Anh. Người dùng trình đọc màn hình ở chế độ tiếng Anh sẽ
nghe một câu tiếng Việt. Sửa đúng nghĩa là **bộ dựng phải biết ngôn ngữ** — một
thay đổi kiến trúc, không phải một dòng sửa.

**B30/B31 — ba lỗi mà 211 phép thử không bắt được, ảnh chụp bắt ngay.**

Ngày 13/08/2026, sau khi có quyền Ghi màn hình, chụp được cửa sổ lần đầu. Ảnh
lộ ra ba thứ:

| # | Lỗi | Vì sao phép thử mù |
|---|---|---|
| 1 | Công tắc quyền là **ô vuông trống**, nhãn chỉ nằm trong `aria-label` | Cây trợ năng CÓ nhãn nên phép kiểm định trợ năng qua sạch |
| 2 | Ô nhập cũng vậy — cùng một lớp lỗi | Như trên |
| 3 | Nút `Tone::Danger` trông **y hệt** nút thường | Phép thử kiểm thuộc tính `data-sac-thai` có mặt, không kiểm nó có tác dụng gì |

Lỗi 1 nặng nhất: ở hộp thoại hỏi quyền, công tắc **là** nút quyết định. Người
dùng sáng mắt thấy một ô vuông không chữ và không biết mình đang bật cái gì —
trong khi trình đọc màn hình nghe đủ. Cả tầng quyền năng vô nghĩa với người nhìn.

Lỗi 3 làm hỏng B10: ứng dụng khai ý định, bộ dựng quyết định hình thức — nhưng
bộ dựng **không có bảng kiểu nào**, nên mọi ý định khai ra đều vẽ giống hệt nhau.
Đã thêm `BANG_KIEU` tối thiểu: mỗi ý định khai ra phải có một biểu hiện nhìn
thấy được.

**Bài học chung**: kiểm cây trợ năng chứng minh *người khiếm thị nghe được*, nó
**không** chứng minh *người sáng mắt nhìn được*. Hai câu khác nhau, và tôi đã
tưởng câu đầu bao hàm câu sau.

**B29 — ghim khoá kiểu TIN-LẦN-ĐẦU, và giới hạn của nó.**

Nó **không** trả lời "gói này có đúng của nhà phát hành X không" — chưa tầng nào
trả lời được câu đó. Nó trả lời câu hẹp hơn: **"khoá ký lần này có giống lần
trước không"**. Câu hẹp đó bắt đúng một tình huống, và là tình huống nguy hiểm
nhất: gói mang mã ứng dụng quen thuộc nhưng ký bằng khoá lạ.

Trước đây người dùng chỉ thấy hộp thoại hiện lại **như lần đầu** — không có cách
nào biết ứng dụng đã đổi tay.

Hai chi tiết đáng nói:

**Chữ phải là sự thật quan sát được, không phải phán quyết.** "Ứng dụng này
trước đây được ký bằng một khoá KHÁC" — chứ không phải "ứng dụng này giả mạo".
Ta không biết ai đúng ai sai: có thể nhà phát hành đổi khoá hợp lệ. Phép thử
`khong_chuoi_nao_noi_da_xac_minh_nha_phat_hanh` nay cấm thêm "giả mạo", "lừa
đảo", "is fake", "is malicious".

**Vị trí là một phần của cảnh báo.** Nó đứng ngay sau tên ứng dụng, TRƯỚC danh
sách quyền. Đặt ở cuối thì người dùng đã đọc xong danh sách và tay đã ở nút bấm.
Đột biến "dời cảnh báo xuống cuối" bị bắt.

**B26/B27 — hai cách làm hỏng việc "nhớ quyền".**

Hỏi lại mỗi lần chạy là cách nhanh nhất khiến người dùng bấm bừa, nên phải nhớ.
Nhưng nhớ sai thì tệ hơn không nhớ:

| Nhớ theo | Kẻ gian làm gì |
|---|---|
| Chỉ mã ứng dụng | Ship gói mang `com.tcc.vi` → thừa hưởng mọi quyền của ví thật |
| Chỉ tên quyền | Bản 1.0 xin `[shop]` được đồng ý; bản 1.1 xin `[shop, thu-thap]` → quyền cũ **tự phủ lên phạm vi mới** |

Nên bản ghi kèm **cả khoá công khai người ký lẫn vân tay phạm vi**. Đổi một
trong hai là `tra()` trả `None`, tức là hỏi lại.

Vân tay phạm vi dùng **tiền tố độ dài**, đúng lối của `tcc_spec::tree` — không
có tiền tố thì `["ab","c"]` và `["a","bc"]` cho cùng chuỗi byte, và hai phạm vi
khác nhau cùng vân tay là cấp nhầm quyền. Có phép thử chốt.

**Điều kho quyền KHÔNG bảo vệ được.** Người sửa được tệp này là người đã có
quyền vào tài khoản của người dùng — lúc đó họ đọc được cả kho khoá lẫn dữ liệu
duyệt web. Tệp ghi quyền 0600 và ghi qua tệp tạm rồi đổi tên, nhưng đó là chống
hỏng nửa chừng và chống người dùng khác trên cùng máy, **không** phải chống kẻ
đã chiếm được tài khoản. Ghi rõ để không ai tưởng nó mạnh hơn thực tế.

**B23 — chuyển hướng là đòn THOÁT KHỎI QUYỀN NĂNG.**

Quyền năng cho phép gọi `shop.tcc-coin.com`. Máy chủ đó trả `302 → ke-gian.example`.
Máy khách nào tự đi theo thì ứng dụng vừa **chạm tới một máy chủ chưa bao giờ
được cấp quyền** — mà cổng quyền năng ở `tcc-runtime` đã đóng lại phía sau và
không có cách nào biết.

Chặn bằng hai lớp cố ý: `max_redirects(0)` trong cấu hình, **và** mã của ta tự
từ chối mọi trạng thái 3xx. Lớp hai kiểm thử được mà không cần máy chủ thật nên
nó không bao giờ mục. Đã kiểm với máy chủ thật: `http.badssl.com` trả 301 và bị
từ chối đúng như thiết kế.

**B25 — vì sao `tcc-net` là crate RIÊNG.**

Để đọc `Cargo.toml` là biết ngay bộ nạp ứng dụng không tự mở socket được:
`tcc-runtime` không phụ thuộc `tcc-net`, nó chỉ gọi qua `trait Mang` tiêm từ
ngoài vào. Luật 8 trong CI chốt điều đó.

Cờ tính năng `mang` tách riêng, nên dựng được một bản trình duyệt **không có
mạng** — hữu ích khi soi bảo mật: chạy bản đó thì chắc chắn không gói tin nào
rời máy, dù mã có lỗi gì.

**Chọn `ureq` + rustls**: đã đo, 22 crate so với `reqwest` 86 (kéo theo cả
runtime async). Rustls chứ không phải OpenSSL — hợp `unsafe_code = deny` và
tránh cả một lịch sử lỗi.

**B20 — vì sao hành vi ở bản kê khai chứ không ở `ui.json`.**

Ba lý do, lý do nào cũng đủ: chữ ký bao trùm bản kê khai nên hành vi không sửa
được sau khi ký; bản kê khai là thứ hộp thoại hỏi quyền đọc, nên về sau hiện
được "nút này gọi shop.tcc-coin.com"; và khai ở `ui.json` nghĩa là `tcc-ui` phải
biết tới mạng — mà crate đó không được biết gì ngoài giao diện.

**B21 — bắt sự KHÔNG NHẤT QUÁN, không chỉ bắt vi phạm.**

Ứng dụng khai được một nút gọi `ke-gian.example` trong khi chỉ xin quyền tới
`shop.tcc-coin.com`. Lúc chạy quyền năng vẫn chặn — nhưng người dùng đã bấm,
không thấy gì xảy ra, và **không ai biết vì sao**. Chặn ở `validate_shape` nghĩa
là `tcc verify` báo cho người viết ứng dụng lúc họ còn ngồi trước máy. Luật khớp
tên miền ở đây phải **y hệt** luật của `tcc-capability` (chính xác, không khớp
tên miền con) — lệch hai bên là lỗ, có phép thử chốt.

**B22 — khẳng định "bị từ chối" là CHƯA ĐỦ.**

Phép thử không chỉ kiểm hàm trả lỗi mà đếm số lần đường mạng bị gọi, và đòi số
đó bằng **0**. Kiểm quyền sau khi gọi thì gói tin đã đến nơi — mà với một máy
chủ theo dõi, chỉ cần gói tin đến là đủ, nội dung trả về không quan trọng. Đột
biến "gọi trước, kiểm sau" bị bắt bởi đúng phép thử này.

Đường ra mạng được **tiêm từ ngoài vào** (`trait Mang`): `tcc-runtime` không mở
socket, nên nó kiểm thử được mà không đụng mạng thật, và mọi đường ra khỏi máy
đều nhìn thấy được ngay tại chỗ gọi — không có lối đi ngầm chôn trong thư viện.

**B17 — hỏi từng mục, đổi 13/08/2026.**

Trước kia một nút "Cho phép" cấp **toàn bộ** quyền ứng dụng xin. Giờ mỗi quyền
một công tắc, mặc định TẮT, và `Allow` cần **hai** điều kiện: bấm nút cho phép
**và** công tắc của đúng quyền đó đang bật. Bấm "Cho phép" mà không bật gì thì
không quyền nào được cấp.

Thêm loại component `Toggle` là **đổi tiêu chuẩn**. Đúng như tài liệu đã hứa,
`NodeKind` không đánh dấu `#[non_exhaustive]` nên bộ dựng **không biên dịch
được** cho tới khi xử lý loại mới. Đó là cái giá đã ghi từ đầu, giờ trả lần đầu
— và nó hoạt động đúng như thiết kế.

**B19 — một điểm yếu do KIỂM ĐỘT BIẾN lộ ra.**

Đột biến thử: bỏ chặn `role === 'switch'` trong kịch bản nối sự kiện, để công
tắc tự gửi tin ngay khi bấm. Phép thử vẫn XANH — vì hai cú bấm xảy ra liên tiếp
trong cùng một nhịp và thông điệp sau **ghi đè** thông điệp trước, nên host vẫn
nhận đúng `cho-phep`.

Ghi đè được là điểm yếu thật, không chỉ là lỗ hổng của phép thử: một quyết định
đã chốt thì không ai được sửa, kể cả chính trang đó. Sửa bằng `o.is_none()` —
quyết định đầu tiên thắng. Sau khi sửa, đột biến bị bắt ngay.

**B15 — một mâu thuẫn ở tầng TIÊU CHUẨN, tự tìm ra 13/08/2026.**

`tcc new` từng sinh ra `entry: "index.html"`. Nó chạy được, và nó phá đúng luật
trung tâm của cả dự án: ứng dụng ship HTML nghĩa là ngày có bộ dựng GPU riêng,
**mọi ứng dụng phải viết lại** — và lúc đó không ai dám bỏ WebView nữa. Giàn
giáo hoá thành nhà. Không phép thử nào bắt được vì không có gì "hỏng"; nó lộ ra
khi ngồi đối chiếu tệp mẫu với luật đã viết.

Đã sửa: điểm vào là **cây component khai báo** (`ui.json`). Ứng dụng nói *có gì
trên màn hình*, bộ dựng quyết định *vẽ ra sao*.

**B16 đáng nói riêng — cạm bẫy suýt dẫm.**

Gắn `#[derive(Deserialize)]` thẳng lên `Node` là xong về mặt biên dịch, và
**thủng toàn bộ tầng kiểm tra**: `Node` để mọi trường riêng tư chính là để mỗi
nút chỉ ra đời qua một hàm dựng có kiểm, còn giải mã trực tiếp thì nhồi thẳng
vào trường. Bỏ qua sạch: trần độ sâu, trần số nút, lọc ký tự giả mạo, ràng buộc
mã hành động, cấm ảnh trỏ ra mạng. Kẻ gian không cần tấn công gì — chỉ cần ship
một tệp JSON.

Nên có hai kiểu riêng: `UiNode` là dữ liệu trần để giải mã, rồi `TryFrom` dựng
lại **qua đúng những hàm dựng đó**. Sáu phép thử chốt: nhãn giật gân, ảnh ra
mạng, mã hành động bậy, vượt trần số nút, vượt trần độ sâu, thiếu mô tả ảnh —
tất cả đều bị chặn khi đến từ JSON y như khi viết tay trong mã Rust.

Một chi tiết đáng ghi: phép thử ký tự đảo chiều chữ phải dựng chuỗi LÚC CHẠY, vì
`rustc` **từ chối biên dịch** tệp nguồn chứa ký tự đó (phòng thủ thêm sau vụ
"Trojan Source"). Trình biên dịch đang cưỡng chế đúng cái luật ta cưỡng chế lúc
chạy.

**B12 đáng nói riêng.** `hoi_quyen` KHÔNG trả `Result` — cố ý. Có `Result` là có
chỗ cho ai đó viết `.unwrap_or(Allow)`. Đóng cửa sổ, cửa sổ hỏng, không dựng nổi
hộp thoại, mã hành động lạ — tất cả ra `Deny`. Đúng MỘT đường ra `Allow`: người
dùng bấm đúng nút. Phần quyết định tách thành hàm thuần `quyet_dinh()` nên kiểm
được mà không cần màn hình, và phép thử liệt kê cả các trường hợp gần đúng
(`"cho-phep "`, `"Cho-Phep"`, `"cho-phep-tat-ca"`) — mọi cái đều phải ra `Deny`.

**B13 đáng nói riêng.** Ứng dụng chỉ khai một `ActionId`; nó không có và không
được có một dòng kịch bản nào. Kịch bản nối sự kiện chạy ở giai đoạn khởi tạo
nên nó là kịch bản của BỘ DỰNG — ứng dụng không có đường chèn vào. Nhờ vậy
"ứng dụng chạy mã khi người dùng bấm nút" là chuyện không xảy ra được.

**B14 và một lỗ hổng của chính phép thử.** Danh sách trắng chỉ nhận hành động
thật sự có trên cây. Nhưng phép thử ban đầu chỉ gửi hành động HỢP LỆ, nên khi
tôi thử gỡ bỏ danh sách trắng thì **mọi phép thử vẫn xanh** — nới lỏng một bộ
lọc là loại đột biến mà phép thử chỉ-gửi-dữ-liệu-hợp-lệ không bao giờ chạm tới.
Đã bịt bằng chế độ `kiem-bam-nut ma`: gửi thẳng một mã bịa ra, phải không nhận
được gì.

**B7 đáng nói riêng — ba tầng, mỗi tầng đã được thử RIÊNG.**

| Tầng | Chặn cái gì | Thử riêng bằng cách nào |
|---|---|---|
| 1. Thoát ký tự | `<script>` không thành thẻ | Gỡ thoát `"` → phép thử đỏ |
| 2. Bộ quét trợ năng | Tài liệu không đọc ngược được thì KHÔNG nạp | Gỡ thoát `<` → bộ quét từ chối |
| 3. Chính sách nội dung | Kịch bản có mặt cũng không chạy | Nạp tài liệu độc THÔ, bỏ qua tầng 1–2 |

Tầng 3 phải kiểm riêng vì ở đường ống thật nó **không bao giờ được thử sức** —
tầng 1 và 2 chặn trước. Một tầng phòng thủ chưa bao giờ được thử là một tầng
chưa biết có tồn tại hay không. Chạy:

```sh
cargo run -p tcc-shell --features cua-so --example kiem-khoi-tan-cong          # cả đường ống
cargo run -p tcc-shell --features cua-so --example kiem-khoi-tan-cong chi-csp  # chỉ tầng 3
```

**B8 đáng nói riêng.** `published_accessibility()` rất dễ cài đặt gian: trả
`tree.accessibility_tree()` là luôn đạt. Bộ dựng thật KHÔNG làm thế — nó dựng
lại cây từ **chính chuỗi đánh dấu sắp nạp vào WebView** (`quet_tro_nang.rs`), nên
hai cây đi bằng hai đường khác nhau. Bộ quét còn kiểm chữ hiện trên màn hình có
trùng nhãn đọc lên không: một nút hiện "Huỷ" mà đọc lên "Xác nhận" là dạng lừa
dối mà cả tầng trợ năng sinh ra để chặn.

**B3 đáng nói riêng.** Nó không được giữ bằng kỷ luật hay bằng soi mã, mà bằng
**trình biên dịch**: `NetworkCapability` có mọi trường riêng tư, nên không dựng
được từ ngoài crate. Không có quyền thì không có giá trị; không có giá trị thì
không biên dịch nổi. "Quên một lần kiểm" là chuyện không xảy ra được.

---

## 2. Lỗ tự tìm ra khi soi lại, và đã sửa

Sáu lỗ dưới đây **không phải do kiểm thử phát hiện** — mọi phép thử đều xanh khi
chúng còn ở đó. Chúng lộ ra khi ngồi đọc lại mã và tự hỏi "kẻ gian sẽ làm gì".

| # | Lỗ | Khai thác thế nào | Sửa ở đâu |
|---|---|---|---|
| L1 | Xin trùng một quyền hai lần | Mục đầu vô hại cho người duyệt đọc, mục sau mới là mục thật được cấp | `tcc-spec` + `tcc-capability`, **chặn ở cả hai tầng** |
| L2 | Không có trần kích thước `manifest.json` | Gửi tệp hàng trăm MB, ta phân tích hết **trước khi** kiểm được chữ ký | `MAX_MANIFEST_BYTES`, kiểm ở **bước 0** |
| L3 | `name` / `reason` không lọc ký tự | `U+202E` đảo chiều chữ làm hộp hỏi quyền đọc ra nghĩa khác | `check_display_safe` |
| L4 | Tên máy chủ ngoài ASCII | `shоp.tcc-coin.com` với "о" Kirin nhìn y hệt bản thật | Bắt buộc ASCII/punycode |
| L6 | `grant()` không tự chặn khai trùng | Bên gọi quên `validate_shape` là lọt | Chặn ngay trong `grant` |
| L7 | Phân loại ký tự sai thứ tự nhánh | `\r` rơi nhầm dải, nhánh sau không chạy tới | Nhánh cụ thể đặt trước dải rộng |
| L8 | **Mã ứng dụng không kiểm khi giải mã JSON** | Ship `id: "com.TCC.hello"` — hai danh tính trông y hệt nhau | `AppId::parse` trong `validate_shape` |
| L9 | **Tên máy chủ không kiểm hình dạng** | `shop.tcc-coin.com:8080@evil.example` — giả mạo userinfo | `check_host` |
| L10 | **Chồng dấu vô hạn lên một chữ** | 500 dấu sắc vẽ thành vệt dọc trùm lên câu cảnh báo | `MAX_DAU_KET_HOP` |

**L10 — chồng dấu che mất cảnh báo (13/08/2026).**

Dấu kết hợp không có giới hạn tự nhiên. Nhãn nút `"Huỷ" + 500 dấu sắc` qua hết
mọi phép kiểm cũ — nó không có ký tự điều khiển, không đảo chiều chữ, không rộng
bằng không. Nhưng bộ dựng vẽ nó thành một **vệt dọc trùm lên phần màn hình bên
trên**, mà trong hộp thoại hỏi quyền phần bên trên chính là câu cảnh báo danh
tính — cái người dùng phải đọc trước khi bấm.

Không cấm hẳn dấu kết hợp được: **tiếng Việt sống bằng nó**. Nên đặt trần 8 dấu
liên tiếp trên một chữ:

| | Số dấu tối đa trên một chữ |
|---|---|
| Tiếng Việt (`ỡ` = o + móc + ngã) | 2 |
| Thái, Devanagari — cụm nặng nhất | ~4–6 |
| **Trần** | **8** |
| UAX #15 cho trao đổi dữ liệu | 30 |

UAX #15 nới tới 30 vì nó lo việc **trao đổi** dữ liệu; ta lo việc **hiển thị**
trên một màn hình quyết định bảo mật.

Dùng `unicode-general-category` (1 crate, không phụ thuộc gì) thay vì tự đoán
dải mã — tự đoán thì sót, mà sót ở đây là lọt đòn.

**Kiểm đột biến hai chiều** — đây là hình dạng đúng của một cái trần: nới lỏng
thì phép thử Zalgo đỏ, siết xuống 1 thì phép thử tiếng Việt đỏ. Chặn Zalgo thì
dễ; chặn mà **không giết tiếng Việt** mới là phần khó.

Lỗ này tìm ra khi đang chuẩn bị cho cổng "gõ tiếng Việt có dấu" của Giai đoạn 1
— hỏi "dấu kết hợp đi qua được thì chồng bao nhiêu cũng đi qua được?"

**L9 — đòn giả mạo userinfo (13/08/2026).**

Trước đây `hosts` chỉ kiểm ASCII, không rỗng, không có ký tự đại diện. Bản kê
khai khai được:

```json
"hosts": ["shop.tcc-coin.com:8080@evil.example"]
```

Chuỗi đó qua hết mọi phép kiểm cũ. Nhưng khi dựng địa chỉ,
`shop.tcc-coin.com:8080` thành phần **userinfo** còn máy chủ THẬT là
`evil.example`. Hộp thoại hỏi quyền hiện nguyên chuỗi, và người đọc lướt thấy
"shop.tcc-coin.com".

**Vì sao không phép thử nào chạm tới**: mọi phép thử đều dùng tên máy chủ hợp
lệ. Lỗ chỉ lộ ra khi ngồi hỏi *"tên máy chủ đi thẳng vào việc dựng địa chỉ — nó
đã được kiểm hình dạng chưa?"* — đúng lúc sắp viết máy khách HTTP. Nếu viết máy
khách trước rồi mới hỏi thì lỗ đã sống trong một bản có mạng.

Đã chặn ở **cả hai đường**: `Scope::Network.hosts` và `Effect::Fetch.host`. Chặn
một đường thì đường kia vẫn dựng được địa chỉ trỏ đi nơi khác.

**L8 — bộ kiểm định tuân thủ tìm ra, không phải kiểm thử đơn vị (13/08/2026).**

`AppId` khai `#[serde(transparent)]`, nên giải mã từ JSON lấy thẳng chuỗi và
**không đi qua `AppId::parse`**. `validate_shape` cũng không kiểm lại. Gói ship
được `id: "hello"` (thiếu đoạn) hoặc `id: "com.TCC.hello"` — mà mã khác hoa
thường là hai danh tính trông y hệt nhau, đúng cái mà `AppId::parse` sinh ra để
chặn.

**Vì sao 34 phép thử đơn vị mù hoàn toàn**: chúng luôn dựng `AppId` bằng
`AppId::parse`, nên không lần nào đi qua đường giải mã JSON. Bộ kiểm định thì
nạp bản kê khai từ JSON **như người dùng thật**, nên chạm ngay.

Đây là **cùng một lớp lỗ** với B16 (giải mã cây giao diện đi vòng qua hàm dựng).
Bài học rộng hơn: **ở đâu có kiểu dữ liệu bảo vệ bất biến bằng hàm dựng, ở đó
phải hỏi "giải mã có đi qua hàm dựng đó không"** — và câu trả lời mặc định của
serde là KHÔNG.

**L5 — nghi ngờ nhưng KHÔNG phải lỗ.** Tôi nghi khoá JSON trùng lặp có thể khiến
công cụ hiển thị và bên kiểm chữ ký thấy hai giá trị khác nhau. Kiểm chứng bằng
mã chạy thật: `serde` từ chối thẳng với lỗi `duplicate field`. Không thêm mã
phòng thủ cho vấn đề không tồn tại — nhưng **đã ghim hành vi đó bằng phép thử**
`khoa_json_trung_lap_bi_tu_choi`, vì giờ ta đang dựa vào nó và việc đổi thư viện
JSON sau này phải làm phép thử đó gãy.

**L7 do clippy bắt, không phải do tôi.** Kiểm thử của tôi chỉ thử `\n` và `\u{0}`
nên không lộ ra. Đây là lý do CI chạy `clippy -D warnings` **trước** bước kiểm thử.

---

## 3. ⚠️ Những gì CHƯA được soi — đọc kỹ mục này

### 3.1 Chữ ký chứng minh TOÀN VẸN, không chứng minh DANH TÍNH

Khoá công khai nằm ngay trong bản kê khai — gói **tự ký**. Bất kỳ ai cũng sinh
được cặp khoá rồi ký gói của mình.

Muốn biết "khoá này có đúng của nhà phát hành X không" cần một tầng nữa: sổ đăng
ký, hoặc tin-lần-đầu rồi ghim khoá. **Tầng đó chưa có ở 0.1.**

> **Luật cho giao diện:** không được hiện chữ "đã xác minh nhà phát hành" khi mới
> chỉ kiểm được chữ ký. Câu đúng là "chữ ký hợp lệ".

Luật này giờ **có phép thử cưỡng chế**: `loi::kiem_thu::khong_chuoi_nao_noi_da_xac_minh_nha_phat_hanh`
quét toàn bộ bảng dịch tìm sáu cụm cấm ở cả hai ngôn ngữ. Ai thêm chuỗi vi phạm
sẽ bị chặn ngay, kể cả khi chưa đọc tệp này. Hộp thoại hỏi quyền còn luôn hiện
"Unknown publisher / Không rõ nhà phát hành" và câu cảnh báo đầy đủ hai vế.

### 3.1b Hộp thoại hỏi quyền vẽ qua WebKit — ĐÃ ĐO, 14/08/2026

Món nợ này từng ghi bằng một câu lo chung chung. Nay đã đo, và nó **nhỏ hơn
nhiều** so với cách nó được ghi:

| Đo được | |
|---|---|
| Một cửa sổ WebView | sinh ra **một tiến trình nội dung RIÊNG** của WebKit; đóng cửa sổ là nó biến mất |
| Hộp thoại hỏi quyền và màn hình ứng dụng | **không bao giờ tồn tại cùng lúc** |

Điều thứ hai không phải may mắn: `tao` chạy **một vòng lặp sự kiện tại một
thời điểm**, và `hoi()` dùng `run_return` nên nó trả về *sau khi cửa sổ đã
đóng*. Màn hình ứng dụng chỉ mở sau đó. Kiến trúc một-vòng-lặp tự nó cấm hai
cửa sổ cùng sống.

**⚠️ Điều gì sẽ phá guarantee này**: mở hộp thoại như cửa sổ con của cửa sổ ứng
dụng, hoặc chuyển sang vòng lặp đa cửa sổ. Ai định làm một trong hai thì phải
đọc lại mục này trước.

**Rủi ro còn lại, đã thu hẹp**: hai tiến trình vẫn đều là WebKit. Một cú thoát
sandbox từ nội dung ứng dụng có thể tồn tại qua thời gian và ảnh hưởng tới một
hộp thoại mở sau đó. Đây mới là thứ widget gốc sửa được — và cũng chỉ thứ đó.

**Vì sao CHƯA làm bây giờ**: nó thuộc Giai đoạn 4 của kế hoạch ("thoát WebView"),
mà Giai đoạn 1 còn chưa đóng. Làm ngay nghĩa là thêm `unsafe` FFI, chỉ phủ macOS,
và dựng lại từ đầu toàn bộ tầng trợ năng vừa xây — trên đúng thứ sẽ được thay.

### 3.1c ⚠️ Giả mạo tiêu đề cửa sổ — tìm ra khi đang ĐO món nợ trên

Ứng dụng tự khai `name`, và tên đó từng là **toàn bộ** tiêu đề cửa sổ của nó.
Một ứng dụng đặt tên `"TCC — quyền đã cấp"` có cửa sổ mang tiêu đề y hệt màn
hình quản lý quyền của trình duyệt — rồi vẽ một danh sách quyền giả với một nút
"Cho phép" giả bên trong.

Không chặn được nó đặt tên đó (tên là chữ của ứng dụng), nhưng chặn được việc
tên đó **chiếm trọn tiêu đề**. Nay: `com.tcc.vi-du.hello — Xin chào TCC`.

Mã ứng dụng không giả được: nó nằm trong phạm vi chữ ký và bị `AppId::parse` ép
về `a-z0-9.` — không dấu cách, không gạch ngang dài, nên **không bắt chước nổi**
tiêu đề của trình duyệt. Có phép thử chốt cả hai chiều: tên giả mạo không chiếm
được đầu tiêu đề, và tiêu đề của trình duyệt không trông giống một mã ứng dụng.

Đây **không phải lời giải trọn vẹn** cho giả mạo tiêu đề — không có lời giải
trọn vẹn nào bằng phần mềm. Nó chặn đúng đòn rẻ nhất.

### 3.2 `ml-dsa` còn ở 0.1.1

Chưa có kiểm định độc lập nào được công bố cho thư viện này. Chọn nó vì thuần
Rust (hợp `unsafe_code = deny`) và cùng hệ trait với phần còn lại.

Giảm rủi ro bằng hai cách: chữ ký **lai** — Ed25519 vẫn đứng nếu ML-DSA hỏng — và
đặt sau trait `SignatureScheme` để đổi thư viện mà không sửa chỗ gọi.

### 3.3 ~~Nội dung là MỘT khối byte~~ — ĐÃ XỬ LÝ 13/08/2026

`verify_package` giờ nhận `&FileTree`, và băm lên **dạng chuẩn tắc** định nghĩa ở
`tcc_spec::tree`:

```text
với mỗi tệp, sắp theo thứ tự byte của đường dẫn:
    u64 độ dài đường dẫn (BE) ‖ đường dẫn ‖ u64 độ dài nội dung (BE) ‖ nội dung
```

**Ghi độ dài trước mọi trường** là thứ chặn đòn nhập nhằng: tệp `"ab"` nội dung
`"c"` và tệp `"a"` nội dung `"bc"` nếu chỉ nối chuỗi thì cùng ra `"abc"` — hai cây
khác hẳn nhau, một chữ ký hợp lệ cho cả hai. Phép thử
`khong_trao_duoc_cay_khac_ma_giu_chu_ky` chốt lại.

Kèm theo, `FileTree::insert` chặn: `..` (thoát thư mục), đường dẫn tuyệt đối,
`\` (Linux thấy một tệp, Windows thấy hai cấp), dấu hai chấm (ổ đĩa Windows), ký
tự điều khiển, và **tên chỉ khác nhau hoa/thường** — vì trên macOS/Windows
`Logo.png` và `logo.png` là cùng một tệp, nên gói chứa cả hai sẽ giải nén ra khác
nhau tuỳ hệ điều hành: cùng một chữ ký, hai kết quả.

**Còn lại:** `canonical_bytes` dựng cả gói trong bộ nhớ. Chấp nhận được ở 0.1;
gói lớn cần băm theo luồng.

### 3.4 Chưa soi kênh biên

Chưa đo thời gian chạy của `verify`. Với dữ liệu công khai (chữ ký, băm nội dung)
thì rò rỉ thời gian không lộ bí mật, nhưng **chưa ai kiểm chứng** điều đó.

### 3.5 Chưa có mã ví nào chạm khoá riêng thật

Đúng theo kế hoạch. Cổng chặn cứng:

> **Không giao dịch nào lên mainnet trước khi qua kiểm định bảo mật độc lập.**

### 3.6 Chưa fuzzing

Chưa có bộ fuzz nào cho bước phân tích bản kê khai. Đây là đầu vào không tin cậy
đầu tiên mà mã ta chạm tới, nên đáng làm sớm.

---

## 3bis. Bộ kiểm định tuân thủ

`conformance/vectors/*.json` — **dữ liệu, không phải mã**, để bản triển khai bằng
ngôn ngữ khác đọc được đúng những tệp đó. So khớp bằng **mã lỗi ổn định**
(`unsafe-display-string`, `bad-app-id`…), không bằng thông báo: thông báo là văn
xuôi tiếng Việt và được phép sửa, mã thì không.

| Nhóm | Số vector | Kiểm cái gì |
|---|---|---|
| `canonical` | 7 | Dạng chuẩn tắc + băm — **interop** |
| `signature` | 15 | Chữ ký lai — **interop**, ba chiều |
| `acvp-mldsa65` | 26 | **Mốc ngoài NIST** cho nửa hậu lượng tử |
| `manifest` | 28 | Nhận/từ chối bản kê khai, hành vi của nút, hình dạng tên máy chủ |
| `ui` | 17 | Nhận/từ chối cây giao diện |
| `capability` | 8 | Khớp phạm vi quyền mạng |

Nhóm `canonical` được sinh bằng **một bản cài đặt Python ĐỘC LẬP**, không lấy từ
mã Rust — nếu không thì vector chỉ nói "chúng tôi khớp với chính chúng tôi". Cây
rỗng cho ra `af1349b9f5f9a1a6…`, khớp KAT Blake3 công khai của chuỗi rỗng, nên
bản Python được neo vào một mốc bên ngoài. Rust và Python khớp **từng byte** ở
cả 7 trường hợp.

### Nhóm `signature` kiểm BA chiều, không phải một

| Chiều | Vì sao cần |
|---|---|
| **Sinh khoá** | Cùng khoá bí mật phải suy ra cùng khoá công khai |
| **Ký** | Ký lại phải ra ĐÚNG chuỗi byte cũ (ký ở đây tất định) |
| **Kiểm** | Chữ ký hợp lệ phải đạt, sáu đòn phá phải hỏng |

Chỉ kiểm chiều thứ ba là không đủ: một bản triển khai kiểm được chữ ký của ta mà
sinh ra chữ ký ta không kiểm được thì **vẫn không dùng chung gói được**.

Sáu đòn phá: lật bit trong nửa Ed25519 · lật bit trong nửa ML-DSA · lật bit
CUỐI CÙNG · cắt ngắn · thêm byte thừa · **đảo thứ tự hai nửa** (bố cục byte là
một phần của tiêu chuẩn, không phải chi tiết cài đặt).

**Cả hai nửa nay đều có mốc ngoài (13/08/2026).**

| Nửa | Mốc ngoài | Số ca |
|---|---|---|
| Ed25519 | RFC 8032 §7.1 TEST 1 | 1 |
| ML-DSA-65 keyGen | **NIST ACVP** (`ML-DSA-keyGen-FIPS204`) | **25 / 25 khớp** |
| ML-DSA-65 sigVer | **NIST ACVP** (`ML-DSA-sigVer-FIPS204`) | 1 |

**⚠️ PHÁT HIỆN QUAN TRỌNG HƠN CẢ VECTOR: giao diện FIPS 204.**

FIPS 204 có **hai** giao diện ký. Giao diện *ngoài* tính
`M' = 0x00 ‖ len(ctx) ‖ ctx ‖ M` rồi mới ký; giao diện *trong* ký thẳng `M`.

Chạy vector sigVer của NIST qua bản triển khai này: nhóm `external` khớp 1/1,
nhóm `internal` **lệch 3/15** — và chỉ lệch ở những ca NIST bảo ĐẠT. Đó đúng là
dấu hiệu của một bên dùng giao diện ngoài.

Kết luận, nay là **một câu của tiêu chuẩn** chứ không còn là giả định:
**TCC dùng giao diện NGOÀI, context RỖNG.**

Một bản triển khai TCC dùng nhầm giao diện sẽ sinh ra chữ ký mà bên kia không
kiểm được — mà **cả hai bên đều "đúng FIPS 204"**. Đây là bẫy interop im lặng,
và trước hôm nay nó không nằm ở đâu trong đặc tả.

**Chiều KÝ neo bằng ĐỐI CHIẾU CHÉO, không bằng vector (14/08/2026).**

Nhóm `sigGen` của ACVP không dùng được: nó cho khoá bí mật ở dạng đã BUNG 4032
byte, còn thư viện `ml-dsa` chỉ nạp được HẠT GIỐNG 32 byte. Không có đường ghép.

Nên đi đường khác: `dilithium-py` 1.4.0 — bản cài đặt **thuần Python, viết bởi
người khác, không dùng chung một dòng mã nào** với bản Rust. Ký cùng thông điệp
bằng cùng hạt giống, so từng byte.

**Bước bắt buộc trước:** bản Python phải tự khớp vector NIST (25/25 keyGen).
Không có bước đó thì nó chỉ là *ý kiến thứ hai* — hai bản cùng sai theo một kiểu
vẫn khớp nhau, và ta sẽ tin nhầm.

Kết quả: **thống nhất từng byte** trên cả ba thông điệp. Mạnh hơn vài vector rời,
vì nó khớp trên ĐÚNG cách dùng của dự án — giao diện ngoài, context rỗng, tất định.

```sh
python3 conformance/doi-chieu-doc-lap.py <thư-mục-vector-ACVP>
```

**Một điều kịch bản này dạy lại về chữ ký lai.** Bản đầu tôi đưa cả sáu đòn phá
cho bản Python kiểm, và nó **nhận** đòn "lật một bit trong nửa Ed25519" — đúng,
vì đòn đó không đụng nửa ML-DSA. Chữ ký lai vẫn hỏng, nhưng hỏng ở nửa kia.
Kịch bản kiểm sai, không phải mã sai. Nó cũng là minh hoạ sống cho B1: **phá một
nửa không lan sang nửa kia, và đó chính là lý do dùng chữ ký lai.**

```sh
cargo run -p tcc-conformance                 # 101 trường hợp
cargo run -p tcc-conformance -- --chi-tiet
```

---

## 4. Cách chạy lại toàn bộ

```bash
cargo test --workspace                              # 234 phép thử
cargo test --workspace --features tcc-shell/cua-so  # 237 — thêm 3 phép thử cần cửa sổ
cargo run -p tcc-conformance                        # 101 vector tuân thủ
python3 conformance/doi-chieu-doc-lap.py <vector>   # đối chiếu chéo dilithium-py
cargo clippy --workspace --all-targets -- -D warnings
tools/kiem-luat-phu-thuoc.sh                        # 12 luật kiến trúc
```

Cả ba phải sạch. `kiem-luat-phu-thuoc.sh` chạy **trước** bước biên dịch trong CI:
mã có chạy được mà sai kiến trúc thì vẫn là sai.

Ba lệnh trên **không** đụng tới WebKit. Phần đi qua bộ dựng thật phải chạy riêng
trên máy có màn hình:

```bash
cargo run -p tcc-shell --features cua-so --example kiem-khoi-tan-cong          # cả đường ống
cargo run -p tcc-shell --features cua-so --example kiem-khoi-tan-cong chi-csp  # riêng CSP
cargo run -p tcc-shell --features cua-so --example kiem-bam-nut cho-phep       # cú bấm → Allow
cargo run -p tcc-shell --features cua-so --example kiem-bam-nut tu-choi        # cú bấm → Deny
cargo run -p tcc-shell --features cua-so --example kiem-bam-nut bat           # bật công tắc → Allow
cargo run -p tcc-shell --features cua-so --example kiem-bam-nut ma             # hành động ma bị vứt
cargo run -p tcc-shell --features cua-so --example kiem-bam-nut ct-ma          # công tắc ma bị vứt
cargo run -p tcc-shell --features cua-so --example kiem-man-hinh-ung-dung <gói>  # màn hình ứng dụng
cargo run -p tcc-shell --example kiem-hanh-vi <gói>                              # cổng quyền năng ba chiều
cargo run -p tcc-shell --example kiem-ghi-nho <gói>                              # kho quyền trên đĩa thật
```

Hai lệnh này nạp bản kê khai thù địch vào WebKit thật rồi hỏi lại WebKit nó nhìn
thấy gì. Chúng **không** nằm trong `cargo test` vì trên macOS vòng lặp sự kiện
bắt buộc chạy trên luồng chính, mà bộ khung kiểm thử của Rust chạy trên luồng
phụ. Đừng bỏ qua chúng chỉ vì `cargo test` xanh.

---

## 5. Báo lỗi bảo mật

Đừng mở issue công khai. Liên hệ bộ phận công nghệ thông tin của TCC.
