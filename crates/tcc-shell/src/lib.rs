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
