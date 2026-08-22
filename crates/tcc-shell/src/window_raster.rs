//! Cửa sổ của bộ dựng **RA PIXEL** — không một dòng `wry`.
//!
//! Đây là bản song song của `window.rs` cho bộ dựng thứ hai. Nó tồn tại tách
//! biệt vì cờ `window` kéo theo cả một máy dựng web, mà cổng ra giai đoạn 4
//! phải chứng minh được **không cần** thứ đó.

use tcc_spec::Manifest;

use crate::permission_dialog;
use crate::text::Language;
use crate::window_title::app_window_title;

/// Mở màn hình của một ứng dụng bằng bộ dựng **RA PIXEL**.
///
/// # Vì sao hàm này tồn tại thay vì gọi thẳng `open_screen`
///
/// Tiêu đề cửa sổ là một **luật an ninh**, không phải một chuỗi tiện tay: mã
/// ứng dụng đã ký phải đứng trước tên do ứng dụng tự đặt, nếu không một gói tên
/// `"TCC — quyền đã cấp"` có cửa sổ trông y hệt màn hình quản lý quyền của
/// trình duyệt. Xem [`app_window_title`] và `SECURITY.md` §3.1c.
///
/// Luật ấy đã vá ở đường WebView từ 16/08/2026. Ví dụ raster đầu tiên truyền
/// thẳng `manifest().name` và **mở lại đúng cái lỗ ấy** trên bộ dựng mới — nên
/// tiêu đề chuyển vào đây, chỗ mọi bên gọi đều đi qua, thay vì nằm ở bên gọi
/// nơi người tiếp theo lại quên.
///
/// # Errors
/// Cây giao diện hỏng, hoặc không dựng được cửa sổ.
#[cfg(feature = "cua-so-raster")]
pub fn open_app_screen_raster(
    m: &Manifest,
    diem_vao: &[u8],
    ngon_ngu: Language,
) -> Result<tcc_render_raster::window::ScreenOutcome, Box<dyn std::error::Error>> {
    let cay = tcc_ui::wire::decode(diem_vao)?;
    Ok(tcc_render_raster::window::open_screen(
        &cay,
        &app_window_title(m),
        &crate::text::raster_text(ngon_ngu),
    )?)
}

/// Hộp thoại hỏi quyền bằng bộ dựng **RA PIXEL**.
///
/// Tiêu đề là của TRÌNH DUYỆT, không phải của ứng dụng — và nó **không bao giờ
/// mang mã ứng dụng**, vì đó chính là dấu người dùng phân biệt cửa sổ của khung
/// với cửa sổ của một gói.
///
/// # Errors
/// Không dựng được hộp thoại hoặc cửa sổ.
#[cfg(feature = "cua-so-raster")]
pub fn open_permission_dialog_raster(
    m: &Manifest,
    ngon_ngu: Language,
) -> Result<tcc_render_raster::window::ScreenOutcome, Box<dyn std::error::Error>> {
    let cay = permission_dialog::build(m, ngon_ngu)?;
    Ok(tcc_render_raster::window::open_screen(
        &cay,
        crate::text::label(crate::text::TextKey::HoiQuyenTieuDeCuaSo, ngon_ngu),
        &crate::text::raster_text(ngon_ngu),
    )?)
}

// ─────────────────────────────────────────────────────────────────────────────
// Màn hình CỦA TRÌNH DUYỆT
//
// Từ đây xuống, mọi cửa sổ là cửa sổ của KHUNG, không của gói. Ba luật lặp lại
// ở từng hàm chứ không rút gọn thành một hàm chung có tham số `tieu_de`:
//
// 1. Tiêu đề do KHUNG tính, bên gọi KHÔNG truyền vào được. Một tham số `tieu_de`
//    là mời người gọi tiếp theo truyền `manifest().name` — đúng lỗ giả mạo mà
//    `SECURITY.md` §3.1c đã vá một lần rồi mở lại ở ví dụ raster đầu tiên.
// 2. Chuỗi lấy từ `crate::text::raster_text(…)`, không bao giờ là mặc định — xem
//    chú thích trên `ScreenText`: mặc định là chỗ một câu tiếng Anh lọt vào màn
//    hình tiếng Việt mà không ai thấy.
// 3. `open_screen` trả `action: None` khi người dùng đóng cửa sổ, và **đóng cửa
//    sổ không phải là đồng ý**. Bên gọi phải đọc `ScreenOutcome` với giả định
//    ấy; các hàm ở đây không diễn giải hộ.
// ─────────────────────────────────────────────────────────────────────────────

/// Màn hình quản lý quyền đã cấp, bằng bộ dựng **RA PIXEL**.
///
/// Gói ứng dụng không đưa được một byte nào vào đây — bộ dựng raster không có
/// đường phục vụ tệp, nên tính chất mà đường WebView phải giữ bằng `|_| None` ở
/// đây là tính chất của chính bộ dựng.
///
/// # Errors
/// Dựng cây hỏng, hoặc không dựng được cửa sổ.
#[cfg(feature = "cua-so-raster")]
pub fn open_permission_screen_raster(
    ds: &[crate::permission_store::StoredEntry],
    ngon_ngu: Language,
) -> Result<tcc_render_raster::window::ScreenOutcome, Box<dyn std::error::Error>> {
    let cay = crate::permission_screen::build(ds, ngon_ngu)?;
    Ok(tcc_render_raster::window::open_screen(
        &cay,
        crate::text::label(crate::text::TextKey::QuanLyTieuDeCuaSo, ngon_ngu),
        &crate::text::raster_text(ngon_ngu),
    )?)
}

/// Màn xác nhận giao dịch bằng bộ dựng **RA PIXEL**.
///
/// Bản tin ký đi cùng và được kiểm TRƯỚC khi vẽ — `transaction_screen::build`
/// từ chối dựng khi băm không khớp. Giữ nguyên thứ tự ấy ở đây: vẽ trước rồi
/// kiểm sau là đã hiện cho người dùng một giao dịch chưa biết có thật không.
///
/// # Errors
/// Băm không khớp, chuỗi không dùng được, hoặc không dựng được cửa sổ.
#[cfg(feature = "cua-so-raster")]
pub fn open_transaction_screen_raster(
    tx: &tcc_chain::Transfer,
    signing_message_tu_may_chu: &[u8; 32],
    ngon_ngu: Language,
) -> Result<tcc_render_raster::window::ScreenOutcome, Box<dyn std::error::Error>> {
    let cay = crate::transaction_screen::build(tx, signing_message_tu_may_chu, ngon_ngu)?;
    Ok(tcc_render_raster::window::open_screen(
        &cay,
        crate::text::label(crate::text::TextKey::GdTieuDeCuaSo, ngon_ngu),
        &crate::text::raster_text(ngon_ngu),
    )?)
}

/// Màn **ĐÃ GỬI** bằng bộ dựng **RA PIXEL**.
///
/// Kết quả phải nằm TRONG cửa sổ, không chỉ ở terminal: người dùng bấm ký rồi
/// thấy cửa sổ biến mất là trạng thái tệ nhất một ví để lại — xem
/// `transaction_screen::build_sent`.
///
/// # Errors
/// Chuỗi không dùng được, hoặc không dựng được cửa sổ.
#[cfg(feature = "cua-so-raster")]
pub fn open_transaction_sent_raster(
    ma_giao_dich: &str,
    ngon_ngu: Language,
) -> Result<tcc_render_raster::window::ScreenOutcome, Box<dyn std::error::Error>> {
    let cay = crate::transaction_screen::build_sent(ma_giao_dich, ngon_ngu)?;
    Ok(tcc_render_raster::window::open_screen(
        &cay,
        crate::text::label(crate::text::TextKey::XongTieuDeCuaSo, ngon_ngu),
        &crate::text::raster_text(ngon_ngu),
    )?)
}

/// Màn hỏi trước khi ra ngoài (tầng 3) bằng bộ dựng **RA PIXEL**.
///
/// # Địa chỉ được kiểm TRƯỚC khi hiện lên
///
/// `check_url` chạy trước `build_confirm` chứ không sau. Địa chỉ ở đây có thể
/// đến từ một gói ứng dụng, và màn hình này in nó ra NGUYÊN VẸN để người dùng
/// đọc — hiện một chuỗi ta chưa kiểm rồi mới kiểm lúc bấm nghĩa là người dùng
/// đã đọc và tin nó một lượt trước khi ta biết nó là gì.
///
/// # Errors
/// Địa chỉ không phải `http`/`https`, chuỗi không dùng được, hoặc không dựng
/// được cửa sổ.
#[cfg(feature = "cua-so-raster")]
pub fn open_external_link_raster(
    url: &str,
    ngon_ngu: Language,
) -> Result<tcc_render_raster::window::ScreenOutcome, Box<dyn std::error::Error>> {
    crate::external_link::check_url(url)?;
    let cay = crate::external_link::build_confirm(url, ngon_ngu)?;
    Ok(tcc_render_raster::window::open_screen(
        &cay,
        crate::text::label(crate::text::TextKey::RaNgoaiTieuDeCuaSo, ngon_ngu),
        &crate::text::raster_text(ngon_ngu),
    )?)
}

/// Màn gõ cụm từ khôi phục bằng bộ dựng **RA PIXEL**.
///
/// `loi` khác `None` thì màn hình hiện lại kèm câu báo lỗi — người dùng gõ lại
/// chứ không bị đá về từ đầu.
///
/// # Errors
/// Chuỗi không dùng được, hoặc không dựng được cửa sổ.
#[cfg(feature = "cua-so-raster")]
pub fn open_recovery_entry_raster(
    loi: Option<&str>,
    ngon_ngu: Language,
) -> Result<tcc_render_raster::window::ScreenOutcome, Box<dyn std::error::Error>> {
    let cay = crate::recovery_screen::build_entry(loi, ngon_ngu)?;
    Ok(tcc_render_raster::window::open_screen(
        &cay,
        crate::text::label(crate::text::TextKey::CumTuTieuDeCuaSo, ngon_ngu),
        &crate::text::raster_text(ngon_ngu),
    )?)
}

/// Màn gõ cụm từ của **phiên thử** bằng bộ dựng **RA PIXEL**.
///
/// Tiêu đề riêng, không dùng chung với [`open_recovery_entry_raster`]: hai màn
/// hình trông gần giống nhau nhưng khác nhau ở đúng điều quan trọng nhất —
/// phiên này KHÔNG cất khoá đi đâu cả. Dùng chung một tiêu đề là xoá mất chỗ
/// người dùng đọc được sự khác biệt ấy mà không phải đọc hết màn hình.
///
/// # Errors
/// Chuỗi không dùng được, hoặc không dựng được cửa sổ.
#[cfg(feature = "cua-so-raster")]
pub fn open_recovery_session_entry_raster(
    loi: Option<&str>,
    ngon_ngu: Language,
) -> Result<tcc_render_raster::window::ScreenOutcome, Box<dyn std::error::Error>> {
    let cay = crate::recovery_screen::build_session_entry(loi, ngon_ngu)?;
    Ok(tcc_render_raster::window::open_screen(
        &cay,
        crate::text::label(crate::text::TextKey::PhienTieuDeCuaSo, ngon_ngu),
        &crate::text::raster_text(ngon_ngu),
    )?)
}

/// Màn đối chiếu địa chỉ trước khi lưu ví, bằng bộ dựng **RA PIXEL**.
///
/// # Errors
/// Chuỗi không dùng được, hoặc không dựng được cửa sổ.
#[cfg(feature = "cua-so-raster")]
pub fn open_recovery_confirm_raster(
    dia_chi: &str,
    ngon_ngu: Language,
) -> Result<tcc_render_raster::window::ScreenOutcome, Box<dyn std::error::Error>> {
    let cay = crate::recovery_screen::build_confirm(dia_chi, ngon_ngu)?;
    Ok(tcc_render_raster::window::open_screen(
        &cay,
        crate::text::label(crate::text::TextKey::CumTuXacNhanTieuDeCuaSo, ngon_ngu),
        &crate::text::raster_text(ngon_ngu),
    )?)
}

/// Màn **báo hỏng** bằng bộ dựng **RA PIXEL**.
///
/// Hỏng phải hiện TRONG cửa sổ. Trước 17/08/2026 lỗi của luồng ví đi ra
/// `stderr` và cửa sổ đóng im lặng — từ phía người vừa gõ 24 chữ, đó là
/// *"gõ xong thì ứng dụng tắt"*, không biết cụm từ ấy có bị lưu ở đâu không.
///
/// # Errors
/// Chuỗi không dùng được, hoặc không dựng được cửa sổ.
#[cfg(feature = "cua-so-raster")]
pub fn open_recovery_failure_raster(
    chi_tiet: &str,
    ngon_ngu: Language,
) -> Result<tcc_render_raster::window::ScreenOutcome, Box<dyn std::error::Error>> {
    let cay = crate::recovery_screen::build_failure(chi_tiet, ngon_ngu)?;
    Ok(tcc_render_raster::window::open_screen(
        &cay,
        crate::text::label(crate::text::TextKey::HongTieuDeCuaSo, ngon_ngu),
        &crate::text::raster_text(ngon_ngu),
    )?)
}

/// Màn chọn ví để nhập, bằng bộ dựng **RA PIXEL**.
///
/// Sau cả cờ `cua-so-raster` lẫn `import-web-wallet`: `import_screen` chỉ tồn
/// tại khi có cờ sau, và một hàm gọi tới một module không có thì không biên
/// dịch được — nên hai cờ phải đi cùng nhau ở đây.
///
/// # Errors
/// Chuỗi không dùng được, quá nhiều ví làm cây vượt trần, hoặc không dựng được
/// cửa sổ.
#[cfg(all(feature = "cua-so-raster", feature = "import-web-wallet"))]
pub fn open_import_choice_raster(
    ds: &[tcc_chain::import::WebWallet],
    ngon_ngu: Language,
) -> Result<tcc_render_raster::window::ScreenOutcome, Box<dyn std::error::Error>> {
    let cay = crate::import_screen::build_choice(ds, ngon_ngu)?;
    Ok(tcc_render_raster::window::open_screen(
        &cay,
        crate::text::label(crate::text::TextKey::NhapTieuDeCuaSo, ngon_ngu),
        &crate::text::raster_text(ngon_ngu),
    )?)
}

/// Màn hỏi mã PIN, bằng bộ dựng **RA PIXEL**.
///
/// # ⚠️ Hàm này KHÔNG trả mã PIN về
///
/// `ScreenOutcome` chỉ mang nút đã bấm và các công tắc đang bật — bộ dựng raster
/// chưa có đường trả nội dung ô nhập ra ngoài. Nên đây là điểm vào để **hiện**
/// màn hỏi PIN trên bộ dựng thứ hai, chứ chưa phải để chạy trọn luồng nhập ví;
/// luồng ấy vẫn nằm ở `wallet_flow`, trên đường WebView.
///
/// Nói ra ở đây chứ không để bên gọi tự phát hiện: một hàm tên `open_import_pin`
/// mà im lặng về việc không trả PIN là một hàm người ta sẽ dùng rồi mới biết.
///
/// # Errors
/// Chuỗi không dùng được, hoặc không dựng được cửa sổ.
#[cfg(all(feature = "cua-so-raster", feature = "import-web-wallet"))]
pub fn open_import_pin_raster(
    dia_chi: &str,
    ngon_ngu: Language,
) -> Result<tcc_render_raster::window::ScreenOutcome, Box<dyn std::error::Error>> {
    let cay = crate::import_screen::build_pin(dia_chi, ngon_ngu)?;
    Ok(tcc_render_raster::window::open_screen(
        &cay,
        crate::text::label(crate::text::TextKey::NhapPinTieuDeCuaSo, ngon_ngu),
        &crate::text::raster_text(ngon_ngu),
    )?)
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay, không nuốt lỗi"
)]
mod kiem_thu {
    /// Thân của từng `pub fn` trong tệp này, kèm tên hàm.
    ///
    /// Đọc mã nguồn chứ không gọi hàm: mọi hàm ở đây đều MỞ CỬA SỔ, nên không
    /// gọi được trong `cargo test`. Thứ cần chốt lại là **chỗ gọi viết gì**, và
    /// điều đó soi được mà không cần chạy.
    fn than_cac_diem_vao() -> Vec<(String, String)> {
        let nguon = include_str!("window_raster.rs");
        // Cắt bỏ phần kiểm thử trước khi soi — nếu không, chính tệp này chứa
        // các dấu mốc và nó tự xác nhận mình.
        let than = nguon.split("#[cfg(test)]").next().unwrap_or(nguon);
        let mut ra = Vec::new();
        for khuc in than.split("\npub fn ").skip(1) {
            let ten = khuc.split(['(', '<']).next().unwrap_or_default().to_owned();
            let het = khuc.find("\n}").unwrap_or(khuc.len());
            ra.push((ten, khuc[..het].to_owned()));
        }
        ra
    }

    /// Tệp này phải CÓ điểm vào, và phải có nhiều hơn hai.
    ///
    /// Không có phép thử này thì mọi phép thử dưới đây vẫn xanh trên một danh
    /// sách rỗng — một vòng lặp không chạy lần nào không chứng minh điều gì.
    #[test]
    fn co_du_diem_vao_de_kiem() {
        let ds = than_cac_diem_vao();
        assert!(
            ds.len() >= 11,
            "chỉ tìm thấy {} điểm vào raster: {:?}",
            ds.len(),
            ds.iter().map(|(t, _)| t).collect::<Vec<_>>()
        );
    }

    /// **Không điểm vào nào nhận tiêu đề từ bên gọi.**
    ///
    /// Tiêu đề cửa sổ là luật an ninh (`window_title.rs`, `SECURITY.md` §3.1c).
    /// Thêm một tham số `tieu_de` là mời người gọi tiếp theo truyền
    /// `manifest().name` vào một cửa sổ CỦA TRÌNH DUYỆT — đúng lỗ đã vá một lần
    /// rồi mở lại ở ví dụ raster đầu tiên.
    #[test]
    fn khong_diem_vao_nao_nhan_tieu_de_tu_ben_goi() {
        for (ten, than) in than_cac_diem_vao() {
            let chu_ky = than.split(')').next().unwrap_or(&than);
            assert!(
                !chu_ky.contains("tieu_de") && !chu_ky.contains("title"),
                "`{ten}` nhận tiêu đề từ bên gọi — tiêu đề phải do KHUNG tính"
            );
        }
    }

    /// **Mọi điểm vào phải tự tính tiêu đề, và tính đúng loại.**
    ///
    /// Cửa sổ của ứng dụng dùng `app_window_title` (mã ứng dụng đứng trước);
    /// cửa sổ của trình duyệt dùng một khoá `…TieuDeCuaSo` trong `text.rs`.
    /// Không có đường thứ ba — một chuỗi viết thẳng vào đây là một chuỗi không
    /// ai dịch và không ai canh.
    #[test]
    fn moi_diem_vao_dung_tieu_de_cua_khung() {
        for (ten, than) in than_cac_diem_vao() {
            let cua_ung_dung = than.contains("app_window_title");
            let cua_khung = than.contains("TieuDeCuaSo");
            assert!(
                cua_ung_dung ^ cua_khung,
                "`{ten}` không nói rõ cửa sổ này của AI: `app_window_title` cho \
                 cửa sổ của gói, khoá `…TieuDeCuaSo` cho cửa sổ của trình duyệt"
            );
        }
    }

    /// **Mọi màn hình của khung phải MỞ ĐƯỢC bằng bộ dựng ra pixel.**
    ///
    /// # Vì sao đọc thư mục `src/` thay vì viết sẵn một danh sách
    ///
    /// Đây là cổng ra A3 (`docs/bo-webview.md`): *"`window_raster` phủ **mọi**
    /// màn hình khung, không chỉ 2"*. Một danh sách viết tay phủ hết cho tới
    /// đúng lúc ai đó thêm một màn hình — và lúc ấy nó vẫn xanh. Cùng lý do
    /// `khong_bo_sot_man_hinh_nao` trong `tests/hai-bo-dung.rs` đọc mã nguồn.
    ///
    /// Danh sách MIỄN phải NGẮN và mỗi dòng phải nói được vì sao: nó là chỗ
    /// "phủ hết" tự nới lỏng nếu không ai canh.
    #[test]
    fn khong_man_hinh_nao_thieu_diem_vao_raster() {
        let mut thay: Vec<String> = Vec::new();
        for tep in std::fs::read_dir("src").expect("đọc được src/") {
            let tep = tep.expect("mục hỏng").path();
            if tep.extension().is_none_or(|e| e != "rs") {
                continue;
            }
            let ten_tep = tep.file_stem().unwrap().to_string_lossy().to_string();
            let noi_dung = std::fs::read_to_string(&tep).expect("đọc được tệp");
            let phan_that = noi_dung.split("#[cfg(test)]").next().unwrap_or(&noi_dung);
            for dong in phan_that.lines() {
                // ⚠️ GIỮ dấu gạch dưới sau `build` — bỏ nó đi thì
                // `build_with_signer` thành `buildwith_signer`, không khớp dòng
                // nào trong bảng, và phép thử báo thiếu đúng thứ đã có.
                if let Some(sau) = dong.trim_start().strip_prefix("pub fn build") {
                    let ten_ham = sau.split(['(', '<']).next().unwrap_or_default();
                    thay.push(format!("{ten_tep}::build{ten_ham}"));
                }
            }
        }

        // Màn hình → điểm vào raster mở nó.
        let phu: &[(&str, &str)] = &[
            ("permission_dialog::build", "open_permission_dialog_raster"),
            ("permission_screen::build", "open_permission_screen_raster"),
            (
                "transaction_screen::build",
                "open_transaction_screen_raster",
            ),
            (
                "transaction_screen::build_sent",
                "open_transaction_sent_raster",
            ),
            ("external_link::build_confirm", "open_external_link_raster"),
            ("recovery_screen::build_entry", "open_recovery_entry_raster"),
            (
                "recovery_screen::build_session_entry",
                "open_recovery_session_entry_raster",
            ),
            (
                "recovery_screen::build_confirm",
                "open_recovery_confirm_raster",
            ),
            (
                "recovery_screen::build_failure",
                "open_recovery_failure_raster",
            ),
        ];
        let mien: &[(&str, &str)] = &[
            (
                "address_bar::build",
                "vành ngoài của TẦNG 2 — nó bao quanh một WebView thật đang nạp \
                 trang, mà bộ dựng ra pixel chưa có gì để đặt vào trong",
            ),
            (
                "permission_dialog::build_with_signer",
                "biến thể của `build`, khác đúng một dòng cảnh báo đổi khoá",
            ),
            (
                "import_screen::build_done",
                "cần một `ImportedWallet`, tức là một khoá bí mật thật — dựng \
                 một khoá thật chỉ để mở một cửa sổ là đúng thứ không nên có",
            ),
        ];

        // Màn hình sau cờ `import-web-wallet` chỉ có điểm vào khi bật cờ ấy.
        // Không coi là "đã phủ" khi cờ tắt — nếu không, phép thử lại xanh vì lý
        // do sai, đúng thứ nó sinh ra để chặn.
        #[cfg(feature = "import-web-wallet")]
        let phu: Vec<(&str, &str)> = phu
            .iter()
            .copied()
            .chain([
                ("import_screen::build_choice", "open_import_choice_raster"),
                ("import_screen::build_pin", "open_import_pin_raster"),
            ])
            .collect();
        #[cfg(not(feature = "import-web-wallet"))]
        let mien: Vec<(&str, &str)> = mien
            .iter()
            .copied()
            .chain([
                ("import_screen::build_choice", "cần cờ `import-web-wallet`"),
                ("import_screen::build_pin", "cần cờ `import-web-wallet`"),
            ])
            .collect();

        // Bảng trên là lời hứa; đây là chỗ kiểm lời hứa ấy có ai giữ không. Một
        // dòng trỏ tới hàm không tồn tại làm màn hình trông như đã phủ.
        let nguon = include_str!("window_raster.rs");
        let khong_ke_kiem_thu = nguon.split("#[cfg(test)]").next().unwrap_or(nguon);
        let hua_suong: Vec<&str> = phu
            .iter()
            .filter(|(_, diem_vao)| !khong_ke_kiem_thu.contains(&format!("pub fn {diem_vao}")))
            .map(|(man, _)| *man)
            .collect();
        assert!(
            hua_suong.is_empty(),
            "bảng nói những màn hình này đã có điểm vào raster, mà hàm thì không \
             có: {hua_suong:?}"
        );

        let bo_sot: Vec<&String> = thay
            .iter()
            .filter(|t| {
                !phu.iter().any(|(m, _)| m == &t.as_str())
                    && !mien.iter().any(|(m, _)| m == &t.as_str())
            })
            .collect();
        assert!(
            bo_sot.is_empty(),
            "màn hình chưa mở được bằng bộ dựng ra pixel: {bo_sot:?}"
        );
    }

    /// **Mọi điểm vào phải lấy chữ theo NGÔN NGỮ ĐANG DÙNG.**
    ///
    /// `ScreenText` cố ý không có `Default` vì mặc định là chỗ một câu tiếng
    /// Anh lọt vào màn hình tiếng Việt mà không ai thấy. Nhưng "không có
    /// `Default`" chỉ chặn được `Default::default()`; nó không chặn được ai đó
    /// dựng tay một `ScreenText` với chuỗi chép cứng. Phép thử này chặn.
    #[test]
    fn moi_diem_vao_dung_chu_da_dich() {
        for (ten, than) in than_cac_diem_vao() {
            assert!(
                than.contains("raster_text(ngon_ngu)"),
                "`{ten}` không lấy chữ trợ năng theo ngôn ngữ đang dùng"
            );
        }
    }
}
