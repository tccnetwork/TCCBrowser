//! Khung trình duyệt: thẻ, thanh địa chỉ, cài đặt, ba tầng nội dung.
//!
//! VIỆC CỦA CRATE NÀY: đây là ĐIỂM LẮP RÁP (composition root) — nơi duy nhất
//! được chọn bộ dựng cụ thể. Mọi crate khác chỉ thấy trừu tượng.
//!
//! BA TẦNG NỘI DUNG:
//!   1. Ứng dụng TCC   — WASM + quyền năng, chạy qua `tcc-runtime`
//!   2. Web hiện đại   — WebView, cho trang viết theo chuẩn đã công bố
//!   3. Lối thoát      — mở bằng trình duyệt hệ thống. Không giấu, không xin lỗi.
//!
//! Tầng 3 là thứ khiến cả chiến lược khả thi: không có nó, ta bị buộc phải đuổi
//! theo Chromium mãi mãi.

#[cfg(feature = "import-web-wallet")]
pub mod import_screen;
pub mod permission_dialog;
pub mod permission_screen;
pub mod permission_store;
pub mod text;
pub mod transaction_screen;
pub mod wallet_store;

pub use tcc_ui::AccessNode;
pub use text::Language;

#[cfg(feature = "window")]
pub mod window;

/// Đường ra ngoài thật. Chỉ có khi bật cờ `mang`.
///
/// Tách cờ riêng để dựng được một bản trình duyệt **không có mạng** — hữu ích
/// khi soi bảo mật: chạy bản đó thì chắc chắn không có gói tin nào rời máy, dù
/// mã có lỗi gì.
#[cfg(feature = "network")]
pub use tcc_net::HttpNetwork;
