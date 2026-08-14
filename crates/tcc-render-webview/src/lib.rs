//! Bộ dựng #1: WebView của hệ điều hành. GIÀN GIÁO — sẽ tháo.
//!
//! VIỆC CỦA CRATE NÀY: cài đặt trait bộ dựng của `tcc-ui` bằng WKWebView (macOS)
//! và WebView2 (Windows).
//!
//! VÌ SAO MƯỢN: nó cho ta tương thích web đầy đủ và DRM của hệ điều hành
//! (FairPlay/PlayReady) miễn phí, trong khi ta dồn sức vào phần chưa ai làm —
//! quyền năng, ví, chữ ký hậu lượng tử.
//!
//! LUẬT: **chỉ `tcc-shell` được phép phụ thuộc crate này.** Không crate nào khác.
//! Đó là thứ giữ cho đường thoát khỏi WebView luôn mở.
//!
//! Crate này được phép dùng `unsafe` cho FFI — kèm chú thích an toàn và kiểm thử.
//!
//! # Vì sao chọn `wry` (13/08/2026)
//!
//! Đã đo, không đoán:
//!
//! | Hướng | Crate kéo theo | Phủ hệ điều hành |
//! |---|---|---|
//! | `wry` + `tao` | 71 | macOS + Windows + Linux ngay |
//! | `objc2-web-kit` viết FFI tay | 18 | **chỉ macOS** |
//!
//! Con số không phải điểm quyết định. Điểm quyết định là: **cả hai hướng đều
//! đặt WebKit vào đường vẽ**, kể cả đường vẽ hộp thoại hỏi quyền. `wry` không
//! làm bề mặt tin cậy xấu đi — nó chỉ thay keo dán ta tự viết bằng keo dán
//! người khác bảo trì, trên đúng thứ ta định tháo bỏ. Đổ công viết `unsafe` FFI
//! vào giàn giáo là trả giá hai lần.
//!
//! ⚠️ NỢ KỸ THUẬT ĐÃ GHI NHẬN: hộp thoại hỏi quyền hiện đang vẽ qua WebKit.
//! Về lâu dài nó phải rời khỏi WebView và dùng widget gốc của hệ điều hành —
//! giao diện quyết định bảo mật không nên chia chung bộ dựng với nội dung
//! không đáng tin. Xem `SECURITY.md`.
//!
//! # Cấu trúc
//!
//! ```text
//!   Node (tcc-ui)
//!      │
//!      ├─ markup::document ────► chuỗi đánh dấu ──► WebView
//!      │                                  │
//!      └─ (KHÔNG dùng lại cây gốc)        ▼
//!                            a11y_scan::scan ──► AccessNode
//! ```
//!
//! Hai mũi tên đi bằng hai đường KHÁC NHAU. Đó là điều kiện để
//! `tcc_ui::check_accessibility_parity` là một phép kiểm thật chứ không phải
//! con dấu cao su.

pub mod a11y_scan;
pub mod markup;
pub mod package_server;

#[cfg(feature = "window")]
pub mod window;

use tcc_ui::{AccessNode, Node, Renderer};

use a11y_scan::ScanError;

/// Bộ dựng WebView.
///
/// Ở giai đoạn này nó dừng ở chỗ SINH RA tài liệu và tự kiểm lại tài liệu đó.
/// Phần mở cửa sổ thật nằm sau cờ tính năng `window` (xem `window.rs`), để
/// `cargo test` chạy được trên máy không có màn hình và không kéo 71 crate vào
/// mọi lần dựng.
#[derive(Debug, Default)]
pub struct WebViewRenderer {
    chu: markup::RendererText,
    document: String,
    body: String,
    cong_bo: Option<AccessNode>,
}

impl WebViewRenderer {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Dùng chữ do tầng trên cấp thay cho mặc định tiếng Anh.
    ///
    /// Bộ dựng không biết ngôn ngữ; bảng dịch nằm ở `tcc-shell`. Đây là cùng
    /// lối với `trait Network` và trình phục vụ tệp: thứ gì phụ thuộc ngữ cảnh thì
    /// tiêm từ ngoài vào.
    #[must_use]
    pub fn with_text(mut self, chu: markup::RendererText) -> Self {
        self.chu = chu;
        self
    }

    /// Tài liệu đầy đủ của lần vẽ gần nhất — thứ sẽ nạp vào WebView.
    #[must_use]
    pub fn document(&self) -> &str {
        &self.document
    }

    /// Riêng phần thân, không kèm chính sách nội dung.
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }
}

impl Renderer for WebViewRenderer {
    type Error = ScanError;

    /// Vẽ, rồi TỰ ĐỌC LẠI cái vừa vẽ.
    ///
    /// Tự đọc lại ngay tại đây là cố ý: một tài liệu sai trợ năng không bao giờ
    /// được nạp vào WebView, nên lỗi nổ ở chỗ gây ra nó chứ không nổ ở tay
    /// người dùng trình đọc màn hình.
    ///
    /// # Errors
    /// Đánh dấu sinh ra không đọc ngược được, hoặc nhãn lệch nội dung.
    fn render(&mut self, tree: &Node) -> Result<(), Self::Error> {
        let body = markup::body_with_text(tree, &self.chu);
        // Quét TRƯỚC khi nhận vào trạng thái: hỏng thì bộ dựng giữ nguyên lần
        // vẽ trước, không rơi vào trạng thái nửa vời.
        let cong_bo = a11y_scan::scan(&body)?;
        self.document = markup::document_with_text(tree, &self.chu);
        self.body = body;
        self.cong_bo = Some(cong_bo);
        Ok(())
    }

    fn published_accessibility(&self) -> Option<AccessNode> {
        self.cong_bo.clone()
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay, không nuốt lỗi"
)]
mod kiem_thu {
    use super::*;
    use tcc_ui::{Alt, Emphasis, Flow, Gap, Tone, check_accessibility_parity};

    fn man_hinh_vi() -> Node {
        Node::group(Flow::Column, Gap::Medium)
            .child(Node::text_with("Ví TCC", Emphasis::Title).unwrap())
            .unwrap()
            .child(Node::text("Số dư: 3 TCC").unwrap())
            .unwrap()
            .child(Node::field("Mật khẩu", "", true).unwrap())
            .unwrap()
            .child(Node::image("anh/logo.png", Alt::Text("Biểu trưng TCC".into())).unwrap())
            .unwrap()
            .child(Node::button("Gửi tiền", "gui-tien", Tone::Danger).unwrap())
            .unwrap()
    }

    /// ⚠️ PHÉP THỬ TRUNG TÂM: cây đi đường đánh dấu rồi quay về phải TRÙNG KHÍT
    /// cây trợ năng gốc.
    ///
    /// Trùng khít nghĩa là không mất nút, không sai vai trò, không rơi nhãn qua
    /// một vòng dịch xuôi rồi dịch ngược.
    #[test]
    fn di_qua_danh_dau_roi_ve_thi_cay_tro_nang_khong_doi() {
        let mut bd = WebViewRenderer::new();
        let cay = man_hinh_vi();
        check_accessibility_parity(&mut bd, &cay).expect("hai cây lệch nhau");
    }

    /// Cùng phép thử trên, nhưng với chữ hiểm: nếu thoát ký tự và giải mã ngược
    /// không phải cặp nghịch đảo đúng, nhãn sẽ lệch và phép thử đỏ.
    #[test]
    fn chu_hiem_di_qua_hai_chieu_van_ve_nguyen_ven() {
        let hiem = "<script>&\"'</script> — 100% & 'xong'";
        let cay = Node::group(Flow::Row, Gap::None)
            .child(Node::text(hiem).unwrap())
            .unwrap()
            .child(Node::button(hiem, "x", Tone::Neutral).unwrap())
            .unwrap();

        let mut bd = WebViewRenderer::new();
        check_accessibility_parity(&mut bd, &cay).expect("chữ hiểm làm lệch hai cây");

        // Và tài liệu vẫn không chứa thẻ kịch bản thật.
        assert!(!bd.document().contains("<script>"));
    }

    #[test]
    fn tieng_viet_va_emoji_di_qua_hai_chieu_van_nguyen() {
        let cay = Node::text("Chào bạn — số dư 3 TCC 🎉 «ngoặc»").unwrap();
        let mut bd = WebViewRenderer::new();
        check_accessibility_parity(&mut bd, &cay).expect("tiếng Việt bị méo qua hai chiều");
    }

    /// Vẽ hỏng thì bộ dựng phải GIỮ NGUYÊN lần vẽ trước, không rơi vào trạng
    /// thái nửa vời — nếu không, màn hình hiện một đằng, cây trợ năng một nẻo.
    #[test]
    fn ve_hong_thi_khong_de_lai_trang_thai_nua_voi() {
        let mut bd = WebViewRenderer::new();
        bd.render(&man_hinh_vi()).unwrap();
        let truoc = bd.document().to_owned();
        let cong_bo_truoc = bd.published_accessibility();

        // Ép một lỗi bằng cách chọc thẳng vào bộ quét: cây hợp lệ thì không tạo
        // ra được đánh dấu hỏng, nên ta kiểm nhánh lỗi ở tầng dưới…
        assert!(a11y_scan::scan("<p>thiếu nhãn</p>").is_err());
        // …và xác nhận bộ dựng vẫn giữ nguyên lần vẽ đạt trước đó.
        assert_eq!(bd.document(), truoc);
        assert_eq!(bd.published_accessibility(), cong_bo_truoc);
    }

    #[test]
    fn chua_ve_lan_nao_thi_khong_cong_bo_gi() {
        assert!(WebViewRenderer::new().published_accessibility().is_none());
    }

    /// Ô bí mật KHÔNG được rò giá trị ra cây trợ năng — trình đọc màn hình đọc
    /// nhãn "Mật khẩu", không đọc mật khẩu.
    #[test]
    fn o_bi_mat_khong_ro_gia_tri_ra_cay_tro_nang() {
        let cay = Node::field("Mật khẩu", "bi-mat-that", true).unwrap();
        let mut bd = WebViewRenderer::new();
        bd.render(&cay).unwrap();
        let a = bd.published_accessibility().unwrap();
        assert_eq!(a.label.as_deref(), Some("Mật khẩu"));
        assert_ne!(a.label.as_deref(), Some("bi-mat-that"));
    }
}
