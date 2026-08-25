//! Khung trình duyệt — **ĐIỂM LẮP RÁP**, nơi duy nhất được chọn bộ dựng cụ thể.
//! Mọi crate khác chỉ thấy trừu tượng.
//!
//! # Crate này HIỆN có gì
//!
//! Hộp thoại hỏi quyền, màn quản lý quyền đã cấp, kho quyền trên đĩa, bảng chữ
//! song ngữ, màn xác nhận giao dịch, màn nhập ví, chỗ mở kho khoá, và khung
//! cửa sổ. Hết.
//!
//! # Những gì dòng đầu tệp này TỪNG hứa mà chưa có
//!
//! Trước 16/08/2026 dòng ấy ghi *"thẻ, thanh địa chỉ, cài đặt, ba tầng nội
//! dung"* và liệt kê cả ba tầng như thể chúng tồn tại. Không có thẻ, không có
//! thanh địa chỉ, không có cài đặt, và **tầng 2–3 có 0 dòng mã** — chúng thuộc
//! Giai đoạn 5, xem `docs/ke-hoach.md`.
//!
//! Người soát độc lập bắt được (F4, 16/08/2026). Một dòng tài liệu nói quá là
//! một dòng người đọc mã tin rồi đi tìm thứ không có — và ở tệp đầu tiên người
//! ta mở thì nó tốn nhiều thời gian nhất.

pub mod external_link;
#[cfg(feature = "import-web-wallet")]
pub mod import_screen;
pub mod permission_dialog;
pub mod permission_screen;
pub mod permission_store;
pub mod recovery_screen;
pub mod signing_flow;
pub mod text;
pub mod transaction_screen;
pub mod wallet_store;

/// Luồng nhập ví trong cửa sổ — cần cả ba: cửa sổ, đọc định dạng ví web, kho khoá.
#[cfg(all(
    feature = "window",
    feature = "import-web-wallet",
    feature = "os-keystore"
))]
pub mod wallet_flow;

pub use tcc_ui::AccessNode;
pub use text::Language;

/// Cửa sổ của bộ dựng RA PIXEL. Không kéo theo `wry`.
#[cfg(feature = "window")]
pub mod window_raster;

pub mod window_title;

/// Đường ra ngoài thật. Chỉ có khi bật cờ `mang`.
///
/// Tách cờ riêng để dựng được một bản trình duyệt **không có mạng** — hữu ích
/// khi soi bảo mật: chạy bản đó thì chắc chắn không có gói tin nào rời máy, dù
/// mã có lỗi gì.
#[cfg(feature = "network")]
pub use tcc_net::HttpNetwork;

/// Đổ một cây giao diện ra CHỮ, để phép thử khẳng định nội dung.
///
/// # Vì sao có mô-đun này
///
/// Các phép thử màn hình trước đây vẽ cây bằng bộ dựng WebView rồi khẳng định
/// trên chuỗi HTML. Chúng khẳng định **nội dung** — "có tên ứng dụng không",
/// "có câu cảnh báo không" — nên buộc chúng đi qua một bộ dựng cụ thể là buộc
/// một câu hỏi trung lập phải mượn từ vựng của một cài đặt.
///
/// Bản đổ này đọc thẳng CÂY. Nó không thuộc bộ dựng nào, nên phép thử không
/// phải đổi lần nữa vào ngày có bộ dựng thứ ba.
#[cfg(test)]
pub(crate) mod do_cay {
    use core::fmt::Write as _;

    use tcc_ui::{Emphasis, Node, NodeKind};

    /// **Câu `cau` có mang dấu CẢNH BÁO không** — hỏi đúng câu, không hỏi cả cây.
    ///
    /// # Vì sao cần hàm này
    ///
    /// Chín phép thử từng viết `s.contains("[cảnh-báo]")`. Câu ấy chỉ hỏi *"có
    /// dấu cảnh báo ở ĐÂU ĐÓ không"* — và kiểm đột biến ngày 25/08/2026 chỉ ra
    /// hậu quả: chuyển dấu ấy từ câu "việc này CHUYỂN TIỀN" sang một dòng khác
    /// thì phép thử **vẫn xanh**, trong khi đúng câu nó sinh ra để canh đã chìm
    /// vào chữ thường.
    ///
    /// Bất biến là "CÂU NÀY nổi rõ", nên phép thử phải nói ra câu nào.
    pub(crate) fn co_canh_bao(cay: &Node, cau: &str) -> bool {
        chu(cay)
            .lines()
            .any(|d| d.contains("[cảnh-báo]") && d.contains(cau))
    }

    /// Cả cây thành chữ: nhãn, nội dung, trạng thái công tắc, mức nhấn.
    pub(crate) fn chu(cay: &Node) -> String {
        let mut ra = String::new();
        ve(cay, &mut ra);
        ra
    }

    fn ve(n: &Node, ra: &mut String) {
        match n.kind() {
            NodeKind::Text { content, emphasis } => {
                // Mức nhấn ĐI CÙNG chữ: vài phép thử hỏi đúng câu "câu này có
                // được đánh dấu cảnh báo không", và đó là một câu hỏi về cây,
                // không phải về thuộc tính HTML nào.
                let dau = match emphasis {
                    Emphasis::Title => "[tiêu-đề] ",
                    Emphasis::Warning => "[cảnh-báo] ",
                    Emphasis::Subtle => "[phụ] ",
                    Emphasis::Normal => "",
                };
                let _ = writeln!(ra, "chữ {dau}{content}");
            }
            NodeKind::Button {
                label,
                tone,
                action,
            } => {
                let _ = writeln!(ra, "nút[{tone:?}] {label} -> {}", action.as_str());
            }
            NodeKind::Field {
                label,
                value,
                secret,
            } => {
                // ⚠️ KHÔNG đổ giá trị của ô che chữ. Bản đổ này đi vào thông báo
                // của phép thử đỏ, và một phép thử đỏ hay được dán vào chỗ khác.
                let hien = if *secret { "•••" } else { value.as_str() };
                let _ = writeln!(
                    ra,
                    "ô nhập[{}] {label}: {hien}",
                    if *secret { "che" } else { "hở" }
                );
            }
            NodeKind::Toggle { label, on, action } => {
                let _ = writeln!(
                    ra,
                    "công tắc[{}] {label} -> {}",
                    if *on { "BẬT" } else { "tắt" },
                    action.as_str()
                );
            }
            NodeKind::Image { source, alt } => {
                let _ = writeln!(ra, "ảnh {source} ({alt:?})");
            }
            NodeKind::Group { flow, gap, .. } => {
                let _ = writeln!(ra, "nhóm[{flow:?},{gap:?}]");
            }
        }
        for c in n.children() {
            ve(c, ra);
        }
    }
}
