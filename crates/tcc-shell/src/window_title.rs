//! Tiêu đề cửa sổ — **một luật an ninh, không phải một chuỗi tiện tay**.
//!
//! # Vì sao tệp này KHÔNG nằm sau cờ bộ dựng
//!
//! Luật này từng nằm trong `window.rs`, tức là sau cờ `window` — cờ kéo theo cả
//! `wry`. Nên khi bộ dựng ra pixel có cửa sổ riêng, nó **không với tới được**
//! luật ấy, và ví dụ đầu tiên truyền thẳng `manifest().name` làm tiêu đề: mở
//! lại đúng đòn giả mạo mà `SECURITY.md` §3.1c đã vá.
//!
//! Một luật an ninh nằm sau cờ của một bộ dựng cụ thể là một luật chỉ đúng trên
//! bộ dựng ấy. Nó thuộc về khung, và khung không có bộ dựng nào.

use tcc_spec::Manifest;

/// Tiêu đề cửa sổ CỦA ỨNG DỤNG.
///
/// # ⚠️ Mã ứng dụng đứng TRƯỚC, tên do ứng dụng đặt đứng SAU
///
/// Ứng dụng tự khai `name`. Đặt tên là "TCC — quyền đã cấp" thì cửa sổ của nó
/// có tiêu đề y hệt màn hình quản lý quyền của trình duyệt — rồi nó vẽ một danh
/// sách quyền giả với một nút "Cho phép" giả bên trong.
///
/// Mã ứng dụng thì KHÔNG giả được: nó nằm trong phạm vi chữ ký và bị
/// `AppId::parse` ép về `a-z0-9.` — không có dấu cách, không có gạch ngang dài,
/// nên nó không bắt chước nổi tiêu đề của trình duyệt.
///
/// Đặt nó ĐỨNG TRƯỚC vì thứ người ta đọc đầu tiên là thứ bên trái. Và cửa sổ
/// của trình duyệt thì KHÔNG BAO GIỜ mang mã ứng dụng — đó là dấu phân biệt.
///
/// Đây không phải lời giải trọn vẹn cho việc giả mạo tiêu đề (không có lời giải
/// trọn vẹn nào bằng phần mềm), nhưng nó chặn đúng đòn rẻ nhất.
#[must_use]
pub fn app_window_title(m: &Manifest) -> String {
    format!("{} — {}", m.id.as_str(), m.name)
}
