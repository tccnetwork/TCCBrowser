//! Thanh địa chỉ của **tầng 2**.
//!
//! # Nó là màn hình của KHUNG, và đó là cả điểm
//!
//! Thanh này chạy trong WebView của khung — có IPC, có kịch bản của ta, nên đọc
//! được thứ người dùng gõ. Trang web chạy trong một WebView **khác**, không có
//! hai thứ ấy.
//!
//! Nếu hai thứ ấy chung một WebView thì trang web gọi được
//! `window.ipc.postMessage` — tức là **tự điền địa chỉ hộ người dùng**, hoặc
//! trả lời hộ một hộp thoại hỏi quyền. Xem `tcc_render_webview::web_tier`.

use tcc_ui::{Flow, Gap, Node, Tone, UiError};

use crate::text::{Language, TextKey, label};

/// Mã nút "đi".
pub const ACTION_GO: &str = "web-di";

/// Dựng thanh địa chỉ.
///
/// # Errors
/// Chuỗi không dùng được trên giao diện.
pub fn build(dia_chi_hien: &str, ngon_ngu: Language) -> Result<Node, UiError> {
    let t = |k: TextKey| label(k, ngon_ngu);
    Node::group(Flow::Column, Gap::Small)
        .child(
            Node::group(Flow::Row, Gap::Small)
                // Ô nhập THƯỜNG, không che chữ: người dùng phải đọc được mình đang ở
                // đâu. Đây cũng là lý do ô che chữ bị cấm với gói ứng dụng — hàng chấm
                // tròn ở thanh địa chỉ thì vô nghĩa và nguy hiểm.
                .child(Node::field(
                    t(TextKey::WebNhanDiaChi),
                    dia_chi_hien.to_owned(),
                    false,
                )?)?
                .child(Node::button(
                    t(TextKey::WebNutDi),
                    ACTION_GO,
                    Tone::Neutral,
                )?)?,
        )?
        // Nói ra ngay trên thanh, không giấu trong tài liệu: người dùng đăng
        // nhập một trang rồi mở lại thấy mất, mà không được báo trước, thì họ
        // kết luận trình duyệt hỏng.
        .child(Node::text_with(
            t(TextKey::WebKhongGiuGi),
            tcc_ui::Emphasis::Subtle,
        )?)
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

    fn ve(cay: &Node) -> String {
        let mut bd = WebViewRenderer::new();
        bd.render(cay).unwrap();
        bd.body().to_owned()
    }

    /// **Ô địa chỉ KHÔNG che chữ.** Người dùng phải đọc được mình đang ở đâu.
    #[test]
    fn o_dia_chi_khong_che_chu() {
        let cay = build("https://a.example", Language::Vi).unwrap();
        let mut che = false;
        gom(&cay, &mut che);
        assert!(!che, "thanh địa chỉ đang che chữ");
    }

    fn gom(n: &Node, ra: &mut bool) {
        if let tcc_ui::NodeKind::Field { secret, .. } = n.kind() {
            *ra = *ra || *secret;
        }
        for c in n.children() {
            gom(c, ra);
        }
    }

    /// Địa chỉ hiện **ĐỦ**, không cắt — phần bị cắt là chỗ kẻ gian đặt tên miền.
    #[test]
    fn dia_chi_hien_du() {
        let dai = "https://ngan-hang-that.example.com/dang-nhap?rat=dai&va=dai-nua&them=nhieu";
        let s = ve(&build(dai, Language::Vi).unwrap());
        assert!(s.contains(&dai.replace('&', "&amp;")), "{s}");
        assert!(!s.contains('…'), "thanh địa chỉ cắt ngắn");
    }

    /// Qua được kiểm định trợ năng của bộ dựng thật.
    #[test]
    fn qua_duoc_kiem_dinh_tro_nang() {
        let cay = build("https://a.example", Language::En).unwrap();
        let mut bd = WebViewRenderer::new();
        tcc_ui::check_accessibility_parity(&mut bd, &cay)
            .expect("thanh địa chỉ không qua được kiểm định trợ năng");
    }
}
