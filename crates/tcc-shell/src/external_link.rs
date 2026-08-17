//! **Tầng 3** — mở một liên kết bằng trình duyệt hệ thống.
//!
//! ```text
//! hiện ĐỦ địa chỉ  →  người dùng bấm  →  giao cho trình duyệt của họ
//! ```
//!
//! # Vì sao tầng này tồn tại
//!
//! `docs/ke-hoach.md`: *"Lối thoát — mở bằng trình duyệt hệ thống. Không giấu,
//! không xin lỗi."* Và: *"Tầng 3 là thứ khiến cả chiến lược khả thi: không có
//! nó, ta bị buộc phải đuổi theo Chromium mãi mãi."*
//!
//! Nó **không** làm TCC Browser thành một trình duyệt web. Nó thừa nhận thẳng
//! rằng có những trang ta không dựng được, và giao chúng cho thứ dựng được.
//!
//! # Ba luật, mỗi luật chặn một đòn
//!
//! | Luật | Chặn cái gì |
//! |---|---|
//! | **Chỉ `http`/`https`** | `file://` đọc trộm đĩa, `javascript:` chạy mã, lược đồ lạ mở ứng dụng khác |
//! | **Không qua vỏ lệnh** | Chèn lệnh: một địa chỉ chứa `;` hay `$( )` là một lệnh chạy trên máy người dùng |
//! | **Hiện ĐỦ địa chỉ trước khi mở** | Người dùng đang rời khỏi mọi thứ TCC che chắn — phải biết mình đi đâu |
//!
//! Luật thứ ba đáng nói nhất: **không cắt ngắn**. Một địa chỉ cắt ngắn
//! `https://ngan-hang.example/…` giấu mất phần quyết định — và phần bị giấu là
//! chỗ kẻ gian đặt tên miền thật của chúng.

use tcc_ui::{Emphasis, Flow, Gap, Node, Tone, UiError};

use crate::text::{Language, TextKey, label};

/// Mã nút mở ra ngoài.
pub const ACTION_OPEN: &str = "ra-ngoai-mo";
/// Mã nút ở lại.
pub const ACTION_STAY: &str = "ra-ngoai-huy";

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum LinkError {
    /// Lược đồ không phải `http`/`https`.
    ///
    /// Gộp mọi lược đồ lạ vào một lỗi là có chủ ý: nói rõ *"`file://` bị chặn"*
    /// là dạy người thử biết lược đồ nào đã được nghĩ tới và lược đồ nào chưa.
    #[error("chỉ mở được liên kết http và https")]
    NotWeb,
    /// Địa chỉ chứa ký tự điều khiển hoặc khoảng trắng.
    ///
    /// Không phải chuyện thẩm mỹ: một ký tự xuống dòng trong địa chỉ là chỗ
    /// tách một dòng thành hai ở bất kỳ tầng nào phía sau chỉ biết đọc dòng.
    #[error("địa chỉ chứa ký tự không được phép")]
    BadChars,
    #[error("không mở được trình duyệt hệ thống: {0}")]
    Spawn(String),
}

/// Kiểm một địa chỉ TRƯỚC khi hiện nó ra hay mở nó.
///
/// # Errors
/// Không phải `http`/`https`, hoặc chứa ký tự không được phép.
pub fn check_url(url: &str) -> Result<(), LinkError> {
    // Ký tự điều khiển và khoảng trắng bị chặn TRƯỚC khi xét lược đồ: một địa
    // chỉ như `https://a.example\n; rm -rf /` có lược đồ hợp lệ.
    if url.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err(LinkError::BadChars);
    }
    let thap = url.to_ascii_lowercase();
    if thap.starts_with("https://") || thap.starts_with("http://") {
        // Phải có gì đó SAU lược đồ. `https://` trơn không mở được cái gì, và
        // nó là dấu hiệu chuỗi bị cắt ở đâu đó.
        let sau = if thap.starts_with("https://") { 8 } else { 7 };
        if url.len() > sau {
            return Ok(());
        }
    }
    Err(LinkError::NotWeb)
}

/// Màn hỏi trước khi ra ngoài.
///
/// # Errors
/// Địa chỉ không hợp lệ, hoặc chuỗi không dùng được trên giao diện.
pub fn build_confirm(url: &str, ngon_ngu: Language) -> Result<Node, UiError> {
    let t = |k: TextKey| label(k, ngon_ngu);
    Node::group(Flow::Column, Gap::Large)
        .child(Node::text_with(t(TextKey::RaNgoaiTieuDe), Emphasis::Title)?)?
        .child(Node::text(t(TextKey::RaNgoaiRoiKhoiTcc))?)?
        // ⚠️ Địa chỉ ĐỦ, không cắt. Phần bị cắt là chỗ kẻ gian đặt tên miền
        // thật của chúng.
        .child(Node::text_with(url.to_owned(), Emphasis::Normal)?)?
        .child(Node::text_with(
            t(TextKey::RaNgoaiKhongConCheChan),
            Emphasis::Warning,
        )?)?
        .child(
            Node::group(Flow::Row, Gap::Medium)
                .child(Node::button(
                    t(TextKey::RaNgoaiNutMo),
                    ACTION_OPEN,
                    Tone::Neutral,
                )?)?
                .child(Node::button(
                    t(TextKey::RaNgoaiNutHuy),
                    ACTION_STAY,
                    Tone::Neutral,
                )?)?,
        )
}

/// Giao địa chỉ cho trình duyệt hệ thống.
///
/// # ⚠️ KHÔNG đi qua vỏ lệnh
///
/// Địa chỉ được đưa làm **một đối số riêng**, không ghép vào một chuỗi lệnh.
/// Ghép chuỗi là mở đường cho `;`, `$( )`, backtick — và địa chỉ ở đây có thể
/// đến từ một gói ứng dụng.
///
/// # Errors
/// Địa chỉ không hợp lệ, hoặc không chạy được lệnh của hệ điều hành.
pub fn open_in_system_browser(url: &str) -> Result<(), LinkError> {
    check_url(url)?;
    let (lenh, doi): (&str, &[&str]) = if cfg!(target_os = "macos") {
        ("open", &[])
    } else if cfg!(target_os = "windows") {
        // `start` là lệnh nội trú của `cmd`, và đối số đầu của nó là TIÊU ĐỀ
        // cửa sổ — bỏ trống, nếu không địa chỉ bị nuốt làm tiêu đề.
        ("cmd", &["/C", "start", ""])
    } else {
        ("xdg-open", &[])
    };
    std::process::Command::new(lenh)
        .args(doi)
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|e| LinkError::Spawn(e.to_string()))
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay"
)]
mod kiem_thu {
    use super::*;
    use tcc_render_webview::WebViewRenderer;
    use tcc_ui::Renderer as _;

    /// **Chỉ `http`/`https`.** Mọi lược đồ khác là một cửa vào máy người dùng.
    #[test]
    fn chi_nhan_http_va_https() {
        for tot in [
            "https://vnexpress.net",
            "http://vi.wikipedia.org/wiki/Tiếng_Việt",
            "HTTPS://TCC-COIN.COM/gi-do",
        ] {
            assert_eq!(check_url(tot), Ok(()), "từ chối nhầm {tot}");
        }
        for xau in [
            "file:///etc/passwd",
            "javascript:alert(1)",
            "data:text/html,<script>",
            "vscode://file/Users",
            "ftp://a.example",
            "https://",
            "http://",
            "",
            "vnexpress.net",
            // Lược đồ hợp lệ nhưng có xuống dòng — chỗ tách một dòng thành hai.
            "https://a.example\nhttps://ke-gian.example",
            "https://a.example; rm -rf /",
        ] {
            assert!(check_url(xau).is_err(), "nhận nhầm {xau:?}");
        }
    }

    /// Câu lỗi KHÔNG nói rõ lược đồ nào bị chặn.
    ///
    /// Nói rõ là dạy người thử biết cái gì đã được nghĩ tới và cái gì chưa.
    #[test]
    fn loi_khong_ke_ten_luoc_do() {
        let cau = check_url("file:///etc/passwd").unwrap_err().to_string();
        assert!(!cau.contains("file"), "{cau}");
    }

    /// **Địa chỉ hiện ĐỦ, không cắt** — phần bị cắt là chỗ kẻ gian đặt tên miền.
    #[test]
    fn dia_chi_hien_du_khong_cat() {
        let dai = "https://ngan-hang-that.example.com/dang-nhap?tiep=rat-dai-de-thu-xem-co-bi-cat-khong&them=nua";
        let cay = build_confirm(dai, Language::Vi).unwrap();
        let mut bd = WebViewRenderer::new();
        bd.render(&cay).unwrap();
        let s = bd.body();
        // `&` phải được THOÁT — một `&` hay `<` thô trong địa chỉ là một lỗ
        // chèn mã. Nên so với dạng đã thoát, không so với chuỗi gốc.
        let da_thoat = dai.replace('&', "&amp;");
        assert!(s.contains(&da_thoat), "địa chỉ bị cắt:\n{s}");
        assert!(!s.contains('…'), "màn hình có dấu cắt ngắn");
        // Và chốt luôn rằng nó ĐÃ thoát, chứ không lọt nguyên `&` ra tài liệu.
        assert!(
            !s.contains("&them=nua"),
            "dấu & lọt ra tài liệu chưa thoát — lỗ chèn mã"
        );
    }

    /// Màn hình phải nói rõ **ở đó không còn thứ gì của TCC che chắn**.
    ///
    /// "Không giấu, không xin lỗi" — kế hoạch, tầng 3.
    #[test]
    fn noi_ro_ra_ngoai_la_mat_che_chan() {
        for ngon_ngu in [Language::En, Language::Vi] {
            let cay = build_confirm("https://a.example", ngon_ngu).unwrap();
            let mut bd = WebViewRenderer::new();
            bd.render(&cay).unwrap();
            let s = bd.body();
            assert!(s.contains(label(TextKey::RaNgoaiRoiKhoiTcc, ngon_ngu)));
            assert!(
                s.contains(label(TextKey::RaNgoaiKhongConCheChan, ngon_ngu)),
                "không nói rõ mất che chắn ({ngon_ngu:?})"
            );
            assert!(s.contains("data-nhan=\"canh-bao\""));
        }
    }

    /// Hai nút cùng sắc thái — không đẩy người dùng ra ngoài.
    #[test]
    fn hai_nut_cung_sac_thai() {
        let cay = build_confirm("https://a.example", Language::Vi).unwrap();
        let mut sac = Vec::new();
        gom(&cay, &mut sac);
        assert_eq!(sac.len(), 2);
        assert_eq!(sac[0], sac[1], "một nút nổi hơn — đang đẩy người dùng");
    }

    fn gom(n: &Node, ra: &mut Vec<Tone>) {
        if let tcc_ui::NodeKind::Button { tone, .. } = n.kind() {
            ra.push(*tone);
        }
        for c in n.children() {
            gom(c, ra);
        }
    }

    /// Địa chỉ hỏng thì **không chạy lệnh nào**, kể cả khi bên gọi bỏ qua lỗi.
    #[test]
    fn dia_chi_hong_thi_khong_chay_lenh() {
        assert_eq!(
            open_in_system_browser("javascript:alert(1)"),
            Err(LinkError::NotWeb)
        );
        assert_eq!(
            open_in_system_browser("https://a.example; rm -rf /"),
            Err(LinkError::BadChars)
        );
    }

    /// Qua được kiểm định trợ năng của bộ dựng thật.
    #[test]
    fn qua_duoc_kiem_dinh_tro_nang() {
        let cay = build_confirm("https://a.example", Language::En).unwrap();
        let mut bd = WebViewRenderer::new();
        tcc_ui::check_accessibility_parity(&mut bd, &cay)
            .expect("màn ra ngoài không qua được kiểm định trợ năng");
    }
}
