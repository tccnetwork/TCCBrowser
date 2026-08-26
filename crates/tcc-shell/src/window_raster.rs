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
#[cfg(feature = "window")]
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
#[cfg(feature = "window")]
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
#[cfg(feature = "window")]
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
#[cfg(feature = "window")]
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
#[cfg(feature = "window")]
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
#[cfg(feature = "window")]
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
#[cfg(feature = "window")]
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
#[cfg(feature = "window")]
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
#[cfg(feature = "window")]
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
#[cfg(feature = "window")]
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
/// Sau cả cờ `window` lẫn `import-web-wallet`: `import_screen` chỉ tồn
/// tại khi có cờ sau, và một hàm gọi tới một module không có thì không biên
/// dịch được — nên hai cờ phải đi cùng nhau ở đây.
///
/// # Errors
/// Chuỗi không dùng được, quá nhiều ví làm cây vượt trần, hoặc không dựng được
/// cửa sổ.
#[cfg(all(feature = "window", feature = "import-web-wallet"))]
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
/// # ⚠️ Kết quả mang mã PIN — thả nó ngay sau khi dùng
///
/// `ScreenOutcome::fields` chứa **chữ thật người dùng gõ**, không phải hàng
/// chấm: hàng chấm là việc của lúc vẽ. Nên bên gọi cầm một bí mật trong tay —
/// không ghi ra nhật ký, không giữ lâu hơn một lần dùng.
///
/// Và **đóng cửa sổ không phải là gửi đi**: `action` là `None` thì bỏ cả
/// `fields`, đúng như đã làm với `toggles_on`.
///
/// Nói ra ở đây chứ không để bên gọi tự phát hiện — cả hai chiều đều nguy: một
/// hàm im lặng về việc KHÔNG trả PIN thì người ta dùng rồi mới biết, còn một
/// hàm im lặng về việc CÓ trả PIN thì người ta ghi nó vào nhật ký.
///
/// # Errors
/// Chuỗi không dùng được, hoặc không dựng được cửa sổ.
#[cfg(all(feature = "window", feature = "import-web-wallet"))]
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

/// Lấy chữ người dùng đã gõ vào một ô, theo nhãn — và **chỉ khi họ bấm nút**.
///
/// # Vì sao là một hàm chứ không phải một dòng `.get()`
///
/// Vì cái dòng ấy dễ viết thiếu vế thứ hai. Đóng cửa sổ **không phải** là gửi
/// đi: người dùng gõ nửa cụm từ khôi phục rồi đóng cửa sổ là họ **đổi ý**, và
/// đọc `fields` lúc ấy là đọc một bí mật họ vừa từ chối đưa.
///
/// Cùng luật đã áp cho `toggles_on`, viết thành mã để không ai phải nhớ.
#[cfg(feature = "window")]
#[must_use]
pub fn field_if_submitted<'a>(
    ket: &'a tcc_render_raster::window::ScreenOutcome,
    nhan: &str,
) -> Option<&'a str> {
    ket.action.as_ref()?;
    ket.fields.get(nhan).map(String::as_str)
}

// ───────── Điểm vào của TRÌNH DUYỆT, trên bộ dựng ra pixel ─────────
//
// Bốn hàm dưới đây là bản raster của `window.rs`. Chúng ở cuối tệp chứ không
// trộn vào nhóm 12 hàm màn-hình-đơn phía trên, vì chúng khác hạng: nhóm trên mở
// MỘT màn hình rồi trả lời; nhóm này điều khiển cả một phiên.

/// **Mở gói và chạy nó — TRỌN ĐƯỜNG, trong MỘT chuỗi màn hình.**
///
/// # Vì sao phải là một hàm, không phải hai lời gọi
///
/// Trước 24/08/2026 đường này là `open_package_raster` (mở hộp thoại hỏi quyền)
/// rồi `run_app_raster` (mở màn ứng dụng). Mỗi lời gọi vào vòng lặp sự kiện một
/// lần — và trên macOS `run_return` **không vào lại được** sau khi đã thoát: lần
/// thứ hai trả về ngay, không giao một sự kiện nào.
///
/// Triệu chứng đi qua ba dạng, mỗi dạng ồn ào hơn dạng trước:
///
/// 1. Dựng hai `EventLoop` → `tao` **abort**, thông báo không nói gì về nguyên
///    nhân.
/// 2. Dùng chung một `EventLoop` nhưng gọi `run_return` hai lần → màn hai **loé
///    rồi tắt**, bên gọi nhận `Ok` và tưởng người dùng đã đóng cửa sổ. Im lặng,
///    nên tệ hơn abort.
/// 3. Nay lần vào thứ hai là LỖI có câu chữ — và hàm này là đường không cần vào
///    lần thứ hai.
///
/// Hỏi quyền và chạy ứng dụng là hai MÀN HÌNH, không phải hai phiên.
///
/// # Errors
/// Chữ ký hỏng, gói hỏng, cấp quyền thất bại, hoặc không mở được cửa sổ.
#[cfg(feature = "window")]
pub fn open_and_run_raster(
    duong_dan: &std::path::Path,
    ngon_ngu: Language,
    kho_quyen: Option<&std::path::Path>,
    mang: &dyn tcc_runtime::Network,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::permission_store::PermissionStore;
    use tcc_render_raster::window::{Next, Screen};

    let (app, noi_dung) = tcc_runtime::verify_from_dir(duong_dan, &tcc_crypto::HybridEd25519MlDsa)?;
    let m = app.manifest().clone();
    let mut nho = kho_quyen.map(PermissionStore::open);
    let (nguoi_ky, con_phai_hoi) = con_thieu_gi(&m, nho.as_ref());

    let tieu_de_app = app_window_title(&m);

    // Cấp quyền từ MỘT câu trả lời. Gọi đúng một lần, dù đi đường nào.
    let mut cho = Some((app, noi_dung));
    let m_cap = m.clone();
    let mut cap = move |nho: &mut Option<PermissionStore>,
                        hanh_dong: Option<&str>,
                        bat: &[String]|
          -> Result<tcc_runtime::LoadedApp, Box<dyn std::error::Error>> {
        let (a, c) = cho.take().ok_or("đã cấp quyền một lần rồi")?;
        let a = tcc_runtime::grant_verified(a, c, |xin| {
            // ⚠️ CHẶN Ở ĐÂY, không chỉ ở hộp thoại.
            //
            // Hộp thoại đã thôi hỏi về quyền bản dựng không cấp được — nhưng
            // hỏi không phải đường duy nhất tới đây. `.tcc-quyen.json` ghi từ
            // một bản dựng CÓ ví, đọc lại trên bản KHÔNG ví, sẽ mang theo câu
            // "đã đồng ý" và cấp cho một thứ không tồn tại. Trục trợ năng cũng
            // là một đường vào khác.
            //
            // Một câu trả lời do bản dựng khác ghi lại KHÔNG phải câu trả lời
            // cho bản dựng này.
            if !permission_dialog::cap_duoc(&xin.scope) {
                return tcc_capability::Decision::Deny;
            }
            let qd = nho
                .as_ref()
                .and_then(|n| n.lookup(&m_cap, xin))
                .unwrap_or_else(|| permission_dialog::decide(hanh_dong, bat, &xin.name));
            if let Some(n) = nho.as_mut() {
                n.remember(&m_cap, xin, qd);
            }
            qd
        })?;
        if let Some(n) = nho.as_ref()
            && let Err(e) = n.save()
        {
            // Ghi hỏng KHÔNG làm hỏng phiên đang chạy — chỉ là lần sau hỏi lại.
            eprintln!("[khung] không ghi được kho quyền: {e} — lần sau sẽ hỏi lại");
        }
        // Ba dòng này là cách người dùng ở dòng lệnh biết gói nào vừa được nạp
        // và quyền nào đã được cấp. Chúng ở ĐÂY chứ không ở `main` vì `main`
        // không còn cầm `LoadedApp` — nó chỉ gọi một hàm và chờ.
        let man = a.manifest();
        println!("✓ Đã nạp \"{}\" ({})", man.name, man.id.as_str());
        println!(
            "  điểm vào : {} ({} byte)",
            man.entry,
            a.entry_content().len()
        );
        println!(
            "  quyền mạng: {}",
            if a.capabilities().network().is_some() {
                "ĐƯỢC CẤP"
            } else {
                "không"
            }
        );
        Ok(a)
    };

    // Đã trả lời hết thì KHÔNG mở hộp thoại: bắt người dùng đọc lại thứ họ đã
    // quyết định là đường dẫn tới bấm bừa.
    let (man_dau, da_cap) = if con_phai_hoi.is_empty() {
        let a = cap(&mut nho, None, &[])?;
        let cay = tcc_ui::wire::decode(a.entry_content())?;
        (
            Screen {
                tree: cay,
                title: tieu_de_app.clone(),
                text: crate::text::raster_text(ngon_ngu),
            },
            Some(a),
        )
    } else {
        (man_hoi_quyen(&m, con_phai_hoi, &nguoi_ky, ngon_ngu)?, None)
    };

    let mut app_dang_chay = da_cap;
    let mut goc_app: Option<tcc_ui::Node> = None;
    let mut loi: Option<String> = None;

    tcc_render_raster::window::open_sequence(man_dau, |k| {
        let Some(hanh_dong) = k.action.as_deref() else {
            return Next::Done;
        };

        // ── Màn 1 → 2: hộp thoại vừa trả lời, cấp quyền rồi hiện ứng dụng ──
        if app_dang_chay.is_none() {
            let bat: Vec<String> = k.toggles_on.iter().cloned().collect();
            let a = match cap(&mut nho, Some(hanh_dong), &bat) {
                Ok(a) => a,
                Err(e) => {
                    loi = Some(e.to_string());
                    return Next::Done;
                }
            };
            let Ok(cay) = tcc_ui::wire::decode(a.entry_content()) else {
                loi = Some("điểm vào không đọc được thành cây hợp lệ".to_owned());
                return Next::Done;
            };
            goc_app = Some(cay.clone());
            app_dang_chay = Some(a);
            return Next::Show(Box::new(Screen {
                tree: cay,
                title: tieu_de_app.clone(),
                text: crate::text::raster_text(ngon_ngu),
            }));
        }

        // ── Trong ứng dụng: mỗi cú bấm đi qua đúng cổng quyền năng ──
        let (Some(a), Some(goc)) = (app_dang_chay.as_ref(), goc_app.as_ref()) else {
            return Next::Done;
        };
        bam_trong_ung_dung(a, goc, hanh_dong, ngon_ngu, mang, &tieu_de_app)
    })?;

    loi.map_or(Ok(()), |e| Err(e.into()))
}

/// Màn hỏi quyền, chỉ liệt kê những quyền CÒN THIẾU câu trả lời.
///
/// Hiện lại cả những quyền đã đồng ý từ trước là bắt người dùng đọc lại thứ họ
/// đã quyết định — và đọc lại quá nhiều lần thì thành bấm bừa.
#[cfg(feature = "window")]
fn man_hoi_quyen(
    m: &Manifest,
    con_phai_hoi: Vec<tcc_spec::CapabilityRequest>,
    nguoi_ky: &crate::permission_store::SignerStatus,
    ngon_ngu: Language,
) -> Result<tcc_render_raster::window::Screen, Box<dyn std::error::Error>> {
    let mut chi_con_thieu = m.clone();
    chi_con_thieu.capabilities = con_phai_hoi;
    let cay = permission_dialog::build_with_signer(&chi_con_thieu, ngon_ngu, nguoi_ky)?;
    // Tiêu đề của TRÌNH DUYỆT, không mang mã ứng dụng — `SECURITY.md` §3.1c.
    let tieu_de =
        crate::text::label(crate::text::TextKey::HoiQuyenTieuDeCuaSo, ngon_ngu).to_owned();
    Ok(tcc_render_raster::window::Screen {
        tree: cay,
        title: tieu_de,
        text: crate::text::raster_text(ngon_ngu),
    })
}

/// Người ký có đổi khoá không, và quyền nào CHƯA có câu trả lời.
///
/// Hỏi trạng thái người ký TRƯỚC khi kho ghi đè khoá mới lên — sau đó không còn
/// gì để so.
#[cfg(feature = "window")]
fn con_thieu_gi(
    m: &Manifest,
    nho: Option<&crate::permission_store::PermissionStore>,
) -> (
    crate::permission_store::SignerStatus,
    Vec<tcc_spec::CapabilityRequest>,
) {
    use crate::permission_store::SignerStatus;
    let nguoi_ky = nho.map_or(SignerStatus::LanDau, |n| n.signer_status(m));
    if let SignerStatus::DoiKhoa { van_tay_cu } = &nguoi_ky {
        eprintln!(
            "[khung] ⚠️ \"{}\" trước đây ký bằng khoá khác ({van_tay_cu})",
            m.name
        );
    }
    // `lookup` trả `None` với mọi trường hợp không rõ ràng, nên
    // "còn thiếu" luôn nghiêng về phía hỏi thêm.
    let con_phai_hoi = m
        .capabilities
        .iter()
        .filter(|c| nho.and_then(|n| n.lookup(m, c)).is_none())
        .cloned()
        .collect();
    (nguoi_ky, con_phai_hoi)
}

/// Một cú bấm TRONG ứng dụng: chạy hành vi, rồi vẽ lại kèm câu trả lời.
///
/// Cổng quyền năng nằm ở `tcc-runtime`: hàm này chỉ chuyển mã hành động xuống
/// `perform`, và `perform` kiểm trước khi chạm mạng.
#[cfg(feature = "window")]
fn bam_trong_ung_dung(
    a: &tcc_runtime::LoadedApp,
    goc: &tcc_ui::Node,
    hanh_dong: &str,
    ngon_ngu: Language,
    mang: &dyn tcc_runtime::Network,
    tieu_de: &str,
) -> tcc_render_raster::window::Next {
    use tcc_render_raster::window::{Next, Screen};
    let cau = match a.perform(hanh_dong, mang) {
        Ok(du_lieu) => crate::text::action_done(hanh_dong, du_lieu.len(), ngon_ngu),
        // Bị quyền năng từ chối KHÔNG phải lỗi của trình duyệt — đó là hệ thống
        // làm đúng việc. Nói ra, rồi chạy tiếp.
        Err(_) => crate::text::action_refused(hanh_dong, ngon_ngu),
    };
    let Ok(cay_moi) = bao_duoi_cay(goc, &cau) else {
        return Next::Done;
    };
    // `Update`, KHÔNG phải `Show`: cây mới là cây cũ cộng một dòng kết quả —
    // vẫn là màn hình ấy. `Show` xoá sạch trạng thái, nên nó sẽ xoá luôn chữ
    // người dùng vừa gõ vào ô bên trên, và họ chỉ thấy chữ mình biến mất sau
    // khi bấm một nút chẳng liên quan.
    Next::Update(Box::new(Screen {
        tree: cay_moi,
        title: tieu_de.to_owned(),
        text: crate::text::raster_text(ngon_ngu),
    }))
}

/// Cây của ứng dụng, kèm một dòng của KHUNG ở dưới.
#[cfg(feature = "window")]
fn bao_duoi_cay(goc: &tcc_ui::Node, cau: &str) -> Result<tcc_ui::Node, tcc_ui::UiError> {
    tcc_ui::Node::group(tcc_ui::Flow::Column, tcc_ui::Gap::Medium)
        .child(goc.clone())?
        .child(tcc_ui::Node::text(cau)?)
}

/// Màn hình quản lý quyền, và các nút "Quên".
///
/// # Đây là màn hình CỦA TRÌNH DUYỆT
///
/// Ứng dụng không đưa được một byte nào vào đây — không có trình phục vụ tệp,
/// không có ảnh từ gói. Cho phép là mở đường vẽ đè lên chính danh sách quyền mà
/// người dùng đang đọc để quyết định thu hồi.
///
/// # Errors
/// Dựng cây hỏng, hoặc không mở được cửa sổ.
#[cfg(feature = "window")]
pub fn manage_permissions_raster(
    kho_quyen: &std::path::Path,
    ngon_ngu: Language,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::{permission_screen, permission_store::PermissionStore};
    use tcc_render_raster::window::{Next, Screen};

    let ds = PermissionStore::open(kho_quyen).list_all();
    if ds.is_empty() {
        println!("[khung] chưa có quyền nào được trả lời");
    }
    let tieu_de = crate::text::label(crate::text::TextKey::QuanLyTieuDeCuaSo, ngon_ngu).to_owned();
    let duong_dan = kho_quyen.to_path_buf();

    tcc_render_raster::window::open_sequence(
        Screen {
            tree: permission_screen::build(&ds, ngon_ngu)?,
            title: tieu_de.clone(),
            text: crate::text::raster_text(ngon_ngu),
        },
        move |k| {
            let Some(hanh_dong) = k.action.as_deref() else {
                return Next::Done;
            };
            if hanh_dong == permission_screen::ACTION_CLOSE {
                return Next::Done;
            }
            let Some(id) = permission_screen::app_to_forget(hanh_dong) else {
                return Next::Done;
            };
            let mut g = PermissionStore::open(&duong_dan);
            g.forget(id);
            match g.save() {
                Ok(()) => println!("[khung] đã quên \"{id}\" — lần sau nó sẽ hỏi lại"),
                Err(e) => eprintln!("[khung] KHÔNG ghi được kho quyền: {e}"),
            }
            // ⚠️ VẼ LẠI, không đóng.
            //
            // Đường WebView phải đóng cửa sổ sau mỗi lần quên, vì `run_loop`
            // giữ một tài liệu cố định và vẽ lại danh sách cần dựng lại cả tài
            // liệu. Ở đây màn hình là một CÂY, và dựng cây mới thì rẻ — nên
            // danh sách cập nhật ngay trước mắt người dùng, đúng thứ họ vừa
            // bấm để thấy.
            //
            // Kho mở LẠI từ đĩa chứ không sửa một bản trong bộ nhớ: một danh
            // sách lệch với đĩa là một danh sách nói dối về thứ đã được thu hồi.
            let moi = PermissionStore::open(&duong_dan).list_all();
            let Ok(cay) = permission_screen::build(&moi, ngon_ngu) else {
                return Next::Done;
            };
            Next::Show(Box::new(Screen {
                tree: cay,
                title: tieu_de.clone(),
                text: crate::text::raster_text(ngon_ngu),
            }))
        },
    )?;
    Ok(())
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
            let than_ham = khuc[..het].to_owned();
            // ĐIỂM VÀO = hàm MỞ một màn hình, không phải mọi `pub fn`.
            //
            // Tệp này còn có hàm đọc kết quả sau khi màn hình đóng. Bắt nó
            // "lấy chữ theo ngôn ngữ" là vô nghĩa — nó không vẽ gì cả. Bản đầu
            // của phép thử gộp cả hai, và nó đỏ ngay lần thêm hàm đầu tiên.
            if than_ham.contains("open_screen(") {
                ra.push((ten, than_ham));
            }
        }
        assert!(
            !ra.is_empty(),
            "không tìm thấy điểm vào nào — phép soi hỏng"
        );
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

    /// **Đóng cửa sổ KHÔNG phải là gửi đi.**
    ///
    /// Người dùng gõ nửa cụm từ khôi phục rồi đóng cửa sổ là họ ĐỔI Ý. Đọc
    /// `fields` lúc ấy là đọc một bí mật họ vừa từ chối đưa.
    #[cfg(feature = "window")]
    #[test]
    fn dong_cua_so_thi_khong_doc_duoc_o_nhap() {
        use tcc_render_raster::window::ScreenOutcome;
        let mut f = std::collections::BTreeMap::new();
        f.insert("PIN".to_owned(), "1234".to_owned());

        let dong = ScreenOutcome {
            action: None,
            toggles_on: std::collections::BTreeSet::new(),
            fields: f.clone(),
        };
        assert_eq!(
            super::field_if_submitted(&dong, "PIN"),
            None,
            "đọc được PIN từ một màn hình người dùng đã ĐÓNG"
        );

        let gui = ScreenOutcome {
            action: Some("mo-khoa".to_owned()),
            toggles_on: std::collections::BTreeSet::new(),
            fields: f,
        };
        assert_eq!(super::field_if_submitted(&gui, "PIN"), Some("1234"));
        assert_eq!(super::field_if_submitted(&gui, "không có"), None);
    }
}
