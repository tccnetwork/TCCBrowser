//! Mở cửa sổ thật. Chỉ biên dịch khi bật cờ `window`.
//!
//! Đây là cửa DUY NHẤT để tầng trên chạm tới một bộ dựng cụ thể. `apps/` gọi
//! hàm ở đây chứ không gọi thẳng `tcc-render-webview` — luật 2 trong
//! `tools/kiem-luat-phu-thuoc.sh` kiểm điều đó, kể cả với `apps/`.
//!
//! Nhờ vậy ngày đổi sang bộ dựng GPU riêng, chỉ tệp này phải sửa.

use std::{path::Path, time::Duration};

use tcc_crypto::HybridEd25519MlDsa;
use tcc_render_webview::{WebViewRenderer, window as bo_dung_cua_so};
use tcc_spec::Manifest;
use tcc_ui::Renderer as _;

use crate::{
    permission_dialog, permission_screen,
    permission_store::{PermissionStore, SignerStatus},
    text::Language,
};

pub use crate::window_title::app_window_title;

/// Hiện hộp thoại hỏi quyền trong một cửa sổ thật.
///
/// `tu_dong_dong` để kiểm khói tự động: không có nó thì cửa sổ chờ người bấm và
/// mọi lệnh chạy tự động sẽ treo.
///
/// # Errors
/// Dựng cây hỏng, bộ dựng vẽ hỏng, hoặc không mở được cửa sổ.
pub fn show_permission_dialog(
    m: &Manifest,
    ngon_ngu: Language,
    tu_dong_dong: Option<Duration>,
) -> Result<(), Box<dyn std::error::Error>> {
    let cay = permission_dialog::build(m, ngon_ngu)?;

    let mut bo_dung = WebViewRenderer::new().with_text(crate::text::renderer_text(ngon_ngu));
    // Vẽ TRƯỚC khi mở cửa sổ: tài liệu hỏng thì hỏng ở đây, không hỏng sau khi
    // người dùng đã thấy một cửa sổ trống.
    bo_dung.render(&cay)?;

    // Hộp thoại hỏi quyền KHÔNG phục vụ tệp nào: nó là màn hình CỦA TRÌNH DUYỆT,
    // không phải của ứng dụng. Cho ứng dụng đưa ảnh vào đây là mở đường vẽ đè
    // lên chính câu cảnh báo.
    bo_dung_cua_so::open(bo_dung.document(), &m.name, |_| None, tu_dong_dong)
}

/// Kiểm khói qua WebKit thật: dựng hộp thoại, nạp vào WebView ẩn, hỏi lại
/// WebKit xem nó nhìn thấy gì.
///
/// # Errors
/// Dựng cây hỏng, vẽ hỏng, hoặc WebKit không báo về kịp.
pub fn check_escaping(
    m: &Manifest,
    ngon_ngu: Language,
    cho_toi_da: Duration,
) -> Result<bo_dung_cua_so::EscapeReport, Box<dyn std::error::Error>> {
    let cay = permission_dialog::build(m, ngon_ngu)?;
    let mut bo_dung = WebViewRenderer::new().with_text(crate::text::renderer_text(ngon_ngu));
    bo_dung.render(&cay)?;
    Ok(bo_dung_cua_so::check_escaping(
        bo_dung.document(),
        cho_toi_da,
    )?)
}

/// Nạp một tài liệu THÔ vào WebKit, không đi qua bộ dịch đánh dấu.
///
/// Chỉ dùng để kiểm RIÊNG tầng chính sách nội dung. Ở đường ống thật, tầng thoát
/// ký tự và bộ quét trợ năng chặn trước nên chính sách nội dung không bao giờ
/// được thử sức — mà một tầng phòng thủ chưa bao giờ được thử là một tầng chưa
/// biết có tồn tại hay không.
///
/// # Errors
/// WebKit không báo về kịp.
pub fn check_raw_document(
    document: &str,
    cho_toi_da: Duration,
) -> Result<bo_dung_cua_so::EscapeReport, Box<dyn std::error::Error>> {
    Ok(bo_dung_cua_so::check_escaping(document, cho_toi_da)?)
}

/// Hiện hộp thoại hỏi quyền và CHỜ người dùng trả lời TỪNG MỤC.
///
/// # ⚠️ Mọi đường không rõ ràng đều dẫn tới TỪ CHỐI
///
/// Đóng cửa sổ, cửa sổ hỏng, bấm nút từ chối — tất cả ra `Decision::Deny`. Chỉ
/// đúng một đường dẫn tới `Allow`: người dùng bấm đúng nút cho phép. Đây là chỗ
/// mà "quên xử lý một nhánh" phải nghiêng về phía an toàn, nên hàm này KHÔNG
/// trả `Result` — không có lỗi nào để chỗ gọi lỡ tay `unwrap_or(Allow)`.
/// Trả về một hàm quyết định cho TỪNG quyền, đã cố định theo câu trả lời của
/// người dùng. Hỏi đúng MỘT lần, dùng lại cho mọi mục.
///
/// Không trả `Result` — cố ý. Có `Result` là có chỗ cho ai đó viết
/// `.unwrap_or(Allow)`. Mọi hỏng hóc đều thành "người dùng chưa bật gì".
fn hoi_quyen_tung_muc(
    m: &Manifest,
    ngon_ngu: Language,
    nguoi_ky: &SignerStatus,
) -> (Option<String>, Vec<String>) {
    let Ok(cay) = permission_dialog::build_with_signer(m, ngon_ngu, nguoi_ky) else {
        eprintln!("[khung] không dựng được hộp thoại hỏi quyền — từ chối tất cả");
        return (None, Vec::new());
    };

    let mut bo_dung = WebViewRenderer::new().with_text(crate::text::renderer_text(ngon_ngu));
    if let Err(e) = bo_dung.render(&cay) {
        eprintln!("[khung] không vẽ được hộp thoại: {e} — từ chối tất cả");
        return (None, Vec::new());
    }

    let hop_le: Vec<String> = cay
        .action_ids()
        .iter()
        .map(|a| a.as_str().to_owned())
        .collect();

    // ⚠️ Tiêu đề là chuỗi CỦA TRÌNH DUYỆT, không phải `m.name`.
    //
    // Trước 19/08/2026 chỗ này truyền thẳng tên do ứng dụng tự khai, nên một gói
    // đặt tên `"TCC — quyền đã cấp"` có được một cửa sổ **của trình duyệt** mang
    // đúng tiêu đề màn hình quản lý quyền. `SECURITY.md` §3.1c đã vá đòn ấy cho
    // cửa sổ CỦA ỨNG DỤNG và bỏ sót cửa sổ này — mà đây mới là cửa sổ người dùng
    // bấm "Cho phép".
    //
    // Tên ứng dụng vẫn hiện, nhưng hiện TRONG hộp thoại, chỗ nó là nội dung
    // được trình bày chứ không phải chỗ nó mạo danh khung.
    match bo_dung_cua_so::ask_dialog(
        bo_dung.document(),
        crate::text::label(crate::text::TextKey::HoiQuyenTieuDeCuaSo, ngon_ngu),
        &hop_le,
    ) {
        Ok(Some(t)) => (Some(t.hanh_dong), t.bat),
        Ok(None) => (None, Vec::new()),
        Err(e) => {
            eprintln!("[khung] cửa sổ hỏng: {e} — từ chối tất cả");
            (None, Vec::new())
        }
    }
}

/// Đường ống đầy đủ: gói đã ký trên đĩa → kiểm chữ ký → hỏi người dùng → cấp
/// quyền → nội dung điểm vào.
///
/// Thứ tự này là tính chất bảo mật, không phải sở thích. Hỏi trước khi kiểm chữ
/// ký nghĩa là hộp thoại hiện tên và lý do LẤY TỪ MỘT BẢN KÊ KHAI CHƯA XÁC
/// THỰC — kẻ gian viết gì người dùng đọc nấy.
///
/// # Errors
/// Thiếu tệp, chữ ký hỏng, thiếu điểm vào, hoặc bản kê khai xin trùng quyền.
pub fn open_package(
    duong_dan: &Path,
    ngon_ngu: Language,
    kho_quyen: Option<&Path>,
) -> Result<tcc_runtime::LoadedApp, Box<dyn std::error::Error>> {
    // 1. Kiểm chữ ký. Chưa hiện gì lên màn hình.
    let (app, noi_dung) = tcc_runtime::verify_from_dir(duong_dan, &HybridEd25519MlDsa)?;
    let m = app.manifest().clone();

    let mut nho = kho_quyen.map(PermissionStore::open);

    // Hỏi TRƯỚC khi `nho` ghi đè khoá mới lên — sau đó không còn gì để so.
    let nguoi_ky = nho
        .as_ref()
        .map_or(SignerStatus::LanDau, |n| n.signer_status(&m));
    if let SignerStatus::DoiKhoa { van_tay_cu } = &nguoi_ky {
        eprintln!(
            "[khung] ⚠️ \"{}\" trước đây ký bằng khoá khác ({van_tay_cu})",
            m.name
        );
    }

    // 2. Quyền nào ĐÃ CÓ CÂU TRẢ LỜI thì không hỏi lại.
    //
    //    `tra` trả `None` với mọi trường hợp không rõ ràng: chưa từng hỏi, khoá
    //    người ký đổi, phạm vi nới rộng. Nên "còn thiếu" luôn nghiêng về phía
    //    hỏi thêm, không bao giờ nghiêng về phía tự cấp.
    let con_phai_hoi: Vec<_> = m
        .capabilities
        .iter()
        .filter(|c| nho.as_ref().and_then(|n| n.lookup(&m, c)).is_none())
        .cloned()
        .collect();

    // Đổi khoá là phải HỎI LẠI TẤT CẢ, kể cả quyền từng đồng ý — vì `tra` đã
    // trả None cho mọi mục, `con_phai_hoi` tự khắc gồm đủ.
    let (hanh_dong, dang_bat) = if con_phai_hoi.is_empty() {
        (None, Vec::new())
    } else {
        // Hộp thoại chỉ liệt kê những quyền CÒN THIẾU câu trả lời. Hiện lại cả
        // những quyền đã đồng ý từ trước là làm người dùng phải đọc lại thứ họ
        // đã quyết định — và đọc lại quá nhiều lần thì thành bấm bừa.
        let mut chi_con_thieu = m.clone();
        chi_con_thieu.capabilities = con_phai_hoi;
        hoi_quyen_tung_muc(&chi_con_thieu, ngon_ngu, &nguoi_ky)
    };

    // 3. Cấp quyền — TỪNG MỤC MỘT.
    let app = tcc_runtime::grant_verified(app, noi_dung, |xin| {
        // Câu trả lời đã nhớ được ưu tiên; chưa có thì lấy từ hộp thoại vừa rồi.
        let qd = nho
            .as_ref()
            .and_then(|n| n.lookup(&m, xin))
            .unwrap_or_else(|| {
                permission_dialog::decide(hanh_dong.as_deref(), &dang_bat, &xin.name)
            });
        if let Some(n) = nho.as_mut() {
            n.remember(&m, xin, qd);
        }
        qd
    })?;

    if let Some(n) = nho.as_ref()
        && let Err(e) = n.save()
    {
        // Ghi hỏng KHÔNG được làm hỏng phiên đang chạy — chỉ là lần sau hỏi lại.
        eprintln!("[khung] không ghi được kho quyền: {e} — lần sau sẽ hỏi lại");
    }

    Ok(app)
}

/// Hiện màn hình CỦA ỨNG DỤNG, sau khi đã cấp quyền.
///
/// Điểm vào là cây component khai báo, **không phải thẻ đánh dấu**. Ứng dụng
/// nói *có gì trên màn hình*; bộ dựng quyết định *vẽ ra sao*. Đó là thứ giữ cho
/// đường thoát khỏi WebView luôn mở — xem `tcc_ui::wire`.
///
/// Cây đọc từ đĩa đi qua **y hệt** các phép kiểm như cây viết tay trong mã: ký
/// tự giả mạo, trần độ sâu, trần số nút, cấm ảnh trỏ ra mạng. Không có ưu ái
/// nào cho nội dung đến từ gói.
///
/// # Errors
/// Điểm vào không đọc được thành cây hợp lệ, hoặc bộ dựng vẽ hỏng.
pub fn show_app(
    app: &tcc_runtime::LoadedApp,
    ngon_ngu: Language,
    tu_dong_dong: Option<Duration>,
) -> Result<(), Box<dyn std::error::Error>> {
    let cay = tcc_ui::wire::decode(app.entry_content())?;

    let mut bo_dung = WebViewRenderer::new().with_text(crate::text::renderer_text(ngon_ngu));
    bo_dung.render(&cay)?;

    // Ảnh đọc TỪ CÂY TỆP ĐÃ KÝ. Đây là đường duy nhất trang lấy được byte từ
    // gói, và nó đi qua đúng phép kiểm đường dẫn của tiêu chuẩn.
    let noi_dung = app.copy_content();
    bo_dung_cua_so::open(
        bo_dung.document(),
        &app_window_title(app.manifest()),
        move |p| noi_dung.get(p).map(<[u8]>::to_vec),
        tu_dong_dong,
    )
}

/// Chạy ứng dụng: vẽ màn hình, và mỗi cú bấm chạy hành vi đã khai trong bản kê
/// khai — qua đúng cổng quyền năng.
///
/// # Cổng quyền năng nằm ở `tcc-runtime`, không ở đây
///
/// Hàm này chỉ chuyển mã hành động xuống `LoadedApp::perform`. Nó KHÔNG tự
/// quyết định được cái gì chạy được: quyền năng do người dùng cấp, và
/// `perform` kiểm trước khi chạm mạng. Khung giao diện không có đường vòng.
///
/// # Errors
/// Điểm vào không đọc được thành cây hợp lệ, hoặc bộ dựng vẽ hỏng.
pub fn run_app(
    app: &tcc_runtime::LoadedApp,
    ngon_ngu: Language,
    mang: &dyn tcc_runtime::Network,
) -> Result<(), Box<dyn std::error::Error>> {
    let cay = tcc_ui::wire::decode(app.entry_content())?;
    let mut bo_dung = WebViewRenderer::new().with_text(crate::text::renderer_text(ngon_ngu));
    bo_dung.render(&cay)?;

    let hop_le: Vec<String> = cay
        .action_ids()
        .iter()
        .map(|a| a.as_str().to_owned())
        .collect();

    let noi_dung = app.copy_content();
    bo_dung_cua_so::run_loop(
        bo_dung.document(),
        &app_window_title(app.manifest()),
        &hop_le,
        // Ảnh đọc TỪ CÂY TỆP ĐÃ KÝ — đường duy nhất trang lấy được byte từ gói.
        move |p| noi_dung.get(p).map(<[u8]>::to_vec),
        |t| {
            match app.perform(&t.hanh_dong, mang) {
                Ok(du_lieu) => {
                    println!("[ứng dụng] {} → {} byte", t.hanh_dong, du_lieu.len());
                }
                // Bị quyền năng từ chối KHÔNG phải lỗi của trình duyệt — đó là
                // hệ thống làm đúng việc. In ra để nhìn thấy được, rồi chạy tiếp.
                Err(e) => println!("[ứng dụng] {} bị từ chối: {e}", t.hanh_dong),
            }
            bo_dung_cua_so::ControlFlowNext::Giu
        },
    )?;
    Ok(())
}

/// Mở màn hình quản lý quyền và xử lý các nút "Quên".
///
/// # Đây là màn hình CỦA TRÌNH DUYỆT
///
/// Truyền `|_| None` cho trình phục vụ tệp: ứng dụng không đưa được một byte
/// nào vào đây. Cho phép là mở đường vẽ đè lên chính danh sách quyền mà người
/// dùng đang đọc để quyết định thu hồi.
///
/// # Errors
/// Dựng cây hỏng, vẽ hỏng, hoặc không mở được cửa sổ.
pub fn manage_permissions(
    kho_quyen: &Path,
    ngon_ngu: Language,
) -> Result<(), Box<dyn std::error::Error>> {
    // Kho mở LẠI sau mỗi lần quên, để màn hình luôn phản ánh đúng đĩa. Giữ một
    // bản trong bộ nhớ rồi vẽ lại từ đó là mở đường cho màn hình lệch với đĩa.
    let ds = PermissionStore::open(kho_quyen).list_all();
    if ds.is_empty() {
        println!("[khung] chưa có quyền nào được trả lời");
    }
    let cay = permission_screen::build(&ds, ngon_ngu)?;

    let mut bo_dung = WebViewRenderer::new().with_text(crate::text::renderer_text(ngon_ngu));
    bo_dung.render(&cay)?;
    let hop_le: Vec<String> = cay
        .action_ids()
        .iter()
        .map(|a| a.as_str().to_owned())
        .collect();

    let duong_dan = kho_quyen.to_path_buf();
    bo_dung_cua_so::run_loop(
        bo_dung.document(),
        crate::text::label(crate::text::TextKey::QuanLyTieuDeCuaSo, ngon_ngu),
        &hop_le,
        |_| None,
        |t| {
            if t.hanh_dong == permission_screen::ACTION_CLOSE {
                return bo_dung_cua_so::ControlFlowNext::Dong;
            }
            if let Some(id) = permission_screen::app_to_forget(&t.hanh_dong) {
                let mut g = PermissionStore::open(&duong_dan);
                g.forget(id);
                match g.save() {
                    Ok(()) => println!("[khung] đã quên \"{id}\" — lần sau nó sẽ hỏi lại"),
                    Err(e) => eprintln!("[khung] KHÔNG ghi được kho quyền: {e}"),
                }
                // Đóng sau khi quên: vẽ lại danh sách cần dựng lại cả tài liệu,
                // mà `run_loop` giữ một tài liệu cố định. Đóng rồi mở lại là hành vi
                // thật thà hơn một danh sách đứng im sau khi người dùng đã bấm.
                return bo_dung_cua_so::ControlFlowNext::Dong;
            }
            bo_dung_cua_so::ControlFlowNext::Giu
        },
    )?;
    Ok(())
}

/// **Tầng 2** — mở một trang web thật, có thanh địa chỉ.
///
/// # Vì sao hàm này nằm ở `tcc-shell`
///
/// Luật 2: chỉ crate này được biết tới một bộ dựng cụ thể. Binary gọi hàm này
/// chứ không gọi thẳng `tcc-render-webview` — mất luật ấy là ngày đổi bộ dựng
/// phải sửa cả những chỗ không ai nhớ là có.
///
/// # Ba điều nói thẳng
///
/// 1. Trang web **mang mã của nó**: không chữ ký, không cổng quyền năng.
/// 2. Nó chạy trong WebView **riêng**, không IPC và không kịch bản của khung.
/// 3. **Chỉ `https://`** — xem `tcc_render_webview::web_tier`.
///
/// # Errors
/// Địa chỉ không hợp lệ, hoặc không dựng được cửa sổ.
pub fn open_web(url: &str, ngon_ngu: Language) -> Result<(), Box<dyn std::error::Error>> {
    let cay = crate::address_bar::build(url, ngon_ngu)?;
    let mut bo_dung = WebViewRenderer::new().with_text(crate::text::renderer_text(ngon_ngu));
    bo_dung.render(&cay)?;
    tcc_render_webview::web_tier::open_browser(url, "TCC — tầng 2", bo_dung.document())?;
    Ok(())
}

/// Đọc bộ trang từ tệp, chạy qua từng trang, in bảng đếm chắn.
///
/// # Errors
/// Không đọc được tệp, tệp không có địa chỉ nào, hoặc không dựng được cửa sổ.
pub fn run_corpus(
    tep: &std::path::Path,
    giay_moi_trang: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let van = std::fs::read_to_string(tep)?;
    let danh_sach: Vec<String> = van
        .lines()
        .map(str::trim)
        .filter(|d| !d.is_empty() && !d.starts_with('#'))
        .map(ToOwned::to_owned)
        .collect();
    if danh_sach.is_empty() {
        return Err(format!("{} không có địa chỉ nào", tep.display()).into());
    }

    let hang = tcc_render_webview::web_tier::run_corpus(&danh_sach, giay_moi_trang)?;

    println!(
        "{:<52} {:>5} {:>5} {:>5} {:>5}",
        "trang", "xong", "đi", "chặn", "cửa+tải"
    );
    let (mut xong, mut chan, mut cua, mut tai, mut im) = (0u64, 0u64, 0u64, 0u64, 0u64);
    for h in &hang {
        let g = h.guards;
        println!(
            "{:<52} {:>5} {:>5} {:>5} {:>5}",
            &h.url[..h.url.len().min(52)],
            if h.finished { "✓" } else { "—" },
            g.navigations_allowed,
            g.navigations_refused,
            g.new_windows_refused + g.downloads_refused
        );
        xong += u64::from(h.finished);
        im += u64::from(h.quiet_on_load());
        chan += g.navigations_refused;
        cua += g.new_windows_refused;
        tai += g.downloads_refused;
    }
    println!(
        "\n{} trang · nạp xong {xong} · im khi nạp {im} · điều hướng bị chặn {chan} · cửa sổ mới bị từ chối {cua} · tải tệp bị từ chối {tai}",
        hang.len()
    );
    // ⚠️ "im khi nạp" KHÔNG phải nhãn đạt/trượt. Nói ra ngay dưới con số, chứ
    // không để trong tài liệu: người đọc một con số trần sẽ tự gán cho nó nghĩa
    // rộng hơn nghĩa nó có. Xem `CorpusRow::quiet_on_load`.
    println!(
        "  (\"im khi nạp\" = không tự đòi quyền năng nào khi KHÔNG ai bấm gì. Không phải nhãn chất lượng, không phải nhãn an toàn.)"
    );
    // Số trang CHƯA chạy cũng phải nói ra: người đọc một bảng thiếu dòng mà
    // không được báo sẽ tưởng mình đang nhìn toàn bộ.
    if hang.len() < danh_sach.len() {
        println!(
            "⚠ dừng sớm: {}/{} trang — cửa sổ bị đóng trước khi chạy hết",
            hang.len(),
            danh_sach.len()
        );
    }
    Ok(())
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay, không nuốt lỗi"
)]
mod kiem_thu {
    use super::*;

    fn ke_khai(ten: &str) -> Manifest {
        serde_json::from_str(&format!(
            r#"{{"spec_version":"0.1","id":"com.tcc.vi-du","name":"{ten}","version":"1",
"publisher":"{}","scheme":"hybrid-ed25519-mldsa65-v1","content_hash":"{}",
"entry":"ui.json","capabilities":[]}}"#,
            "aa".repeat(1992),
            "bb".repeat(48)
        ))
        .expect("bản kê khai mẫu hỏng")
    }

    /// ⚠️ Một ứng dụng đặt tên giống màn hình của trình duyệt.
    ///
    /// Không chặn được nó đặt tên đó — tên là chữ của ứng dụng. Nhưng chặn được
    /// việc tên đó CHIẾM TRỌN tiêu đề: mã ứng dụng đã ký luôn đứng trước.
    #[test]
    fn ten_gia_mao_khong_chiem_duoc_dau_tieu_de() {
        let m = ke_khai("TCC — quyền đã cấp");
        let td = app_window_title(&m);
        assert!(
            td.starts_with("com.tcc.vi-du"),
            "tên do ứng dụng đặt chiếm được đầu tiêu đề: {td}"
        );
        assert!(td.contains("TCC — quyền đã cấp"), "{td}");
    }

    /// **Tiêu đề cửa sổ của TRÌNH DUYỆT không được do ứng dụng đặt.**
    ///
    /// Đây là chỗ `SECURITY.md` §3.1c bỏ sót: nó vá cửa sổ CỦA ỨNG DỤNG, còn
    /// hộp thoại hỏi quyền — cửa sổ người dùng bấm "Cho phép" — vẫn lấy thẳng
    /// tên do ứng dụng tự khai làm tiêu đề.
    #[test]
    fn tieu_de_hoi_quyen_khong_do_ung_dung_dat() {
        for n in [Language::En, Language::Vi] {
            let td = crate::text::label(crate::text::TextKey::HoiQuyenTieuDeCuaSo, n);
            // Không mang mã ứng dụng — dấu phân biệt cửa sổ khung với cửa sổ gói.
            assert!(!td.contains('.'), "{td}");
            // Và KHÔNG chứa tên do ứng dụng khai, dù tên ấy là gì.
            let m = ke_khai("TCC — quyền đã cấp");
            assert!(
                !td.contains(m.name.as_str()),
                "tên ứng dụng lọt vào tiêu đề cửa sổ của trình duyệt: {td}"
            );
        }
    }

    /// Và **chỗ gọi** phải thật sự dùng chuỗi ấy.
    ///
    /// Phép thử trên chỉ kiểm CHUỖI. Đổi chỗ gọi về `&m.name` thì chuỗi vẫn
    /// đúng, phép thử vẫn xanh, và lỗ mở lại — kiểm đột biến chỉ ra đúng điều
    /// đó. Nên phải soi chính đoạn mã dựng hộp thoại.
    #[test]
    fn cho_goi_hoi_quyen_dung_tieu_de_cua_khung() {
        let nguon = include_str!("window.rs");
        // Cắt bỏ phần kiểm thử trước khi soi — nếu không, chính phép thử này
        // chứa dấu mốc và nó tự xác nhận mình.
        let than = nguon.split("#[cfg(test)]").next().unwrap_or(nguon);
        let goi = than
            .split("fn hoi_quyen_tung_muc")
            .nth(1)
            .expect("không tìm thấy `hoi_quyen_tung_muc`");
        let goi = &goi[..goi.find("\n}").unwrap_or(goi.len())];
        assert!(
            goi.contains("HoiQuyenTieuDeCuaSo"),
            "hộp thoại hỏi quyền không dùng tiêu đề của khung — chữ do ứng dụng \
             đặt đang chiếm tiêu đề một cửa sổ của trình duyệt"
        );
    }

    /// Cửa sổ của TRÌNH DUYỆT không bao giờ mang mã ứng dụng — đó là dấu phân biệt.
    #[test]
    fn tieu_de_trinh_duyet_khong_mang_ma_ung_dung() {
        for n in [Language::En, Language::Vi] {
            let td = crate::text::label(crate::text::TextKey::QuanLyTieuDeCuaSo, n);
            assert!(
                !td.contains('.'),
                "tiêu đề của trình duyệt trông giống một mã ứng dụng: {td}"
            );
        }
    }

    /// Mã ứng dụng không bắt chước nổi tiêu đề của trình duyệt: `AppId::parse`
    /// ép về `a-z0-9.`, không có dấu cách và không có gạch ngang dài.
    #[test]
    fn ma_ung_dung_khong_bat_chuoc_noi_tieu_de_trinh_duyet() {
        for bay in ["TCC — quyền đã cấp", "TCC permissions", "tcc quyen"] {
            assert!(
                tcc_spec::AppId::parse(bay).is_err(),
                "\"{bay}\" dùng làm mã ứng dụng được"
            );
        }
    }
}
