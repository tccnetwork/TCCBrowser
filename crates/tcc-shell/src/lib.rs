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

pub mod ghi_nho;
pub mod hop_thoai_quyen;
pub mod loi;
pub mod man_hinh_quyen;

pub use loi::NgonNgu;
pub use tcc_ui::AccessNode;

#[cfg(feature = "cua-so")]
pub mod cua_so;

/// Đường ra ngoài thật. Chỉ có khi bật cờ `mang`.
///
/// Tách cờ riêng để dựng được một bản trình duyệt **không có mạng** — hữu ích
/// khi soi bảo mật: chạy bản đó thì chắc chắn không có gói tin nào rời máy, dù
/// mã có lỗi gì.
#[cfg(feature = "mang")]
pub use tcc_net::MangHttp;
