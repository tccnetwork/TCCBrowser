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

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay"
)]
mod kiem_thu {
    use super::*;

    fn ke_khai(id: &str, ten: &str) -> Manifest {
        serde_json::from_str(&format!(
            r#"{{"spec_version":"0.1","id":"{id}","name":"{ten}","version":"1",
"publisher":"{}","scheme":"hybrid-ed25519-mldsa65-v1","content_hash":"{}",
"entry":"ui.json","capabilities":[]}}"#,
            "ab".repeat(996),
            "bb".repeat(48)
        ))
        .expect("bản kê khai mẫu hỏng")
    }

    /// **Tiêu đề cửa sổ ứng dụng phải mở đầu bằng MÃ ỨNG DỤNG.**
    ///
    /// Chú thích trên [`app_window_title`] là cả một lập luận an ninh (§3.1c):
    /// mã ứng dụng đứng TRƯỚC vì người ta đọc từ trái, và cửa sổ của chính
    /// trình duyệt thì KHÔNG BAO GIỜ mang mã ứng dụng — đó là dấu phân biệt
    /// chống giả mạo tiêu đề.
    ///
    /// Tới 27/08/2026 tệp này có **không một phép thử nào**: kiểm đột biến thay
    /// cả hàm bằng `""` hoặc `"xyzzy"` mà mọi cổng vẫn xanh. Một lập luận an
    /// ninh dài mười dòng, canh bằng con số không.
    #[test]
    fn tieu_de_mo_dau_bang_ma_ung_dung() {
        let m = ke_khai("com.tcc.vi-du.hello", "Xin chào");
        let t = app_window_title(&m);

        assert!(
            t.starts_with("com.tcc.vi-du.hello"),
            "mã ứng dụng phải đứng TRƯỚC — người ta đọc từ trái: {t}"
        );
        assert!(t.contains("Xin chào"), "tên ứng dụng phải có mặt: {t}");
        assert!(!t.is_empty());
    }

    /// **Tên ứng dụng KHÔNG được đẩy mã ứng dụng ra khỏi đầu tiêu đề.**
    ///
    /// Đây là đòn rẻ nhất: đặt tên ứng dụng thành một chuỗi trông y hệt tiêu đề
    /// của trình duyệt, hy vọng người dùng đọc lướt. Mã ứng dụng đứng trước là
    /// thứ chặn nó, và `AppId::parse` ép mã về `a-z0-9.` nên mã KHÔNG bắt chước
    /// nổi tiêu đề của khung.
    #[test]
    fn ten_gia_mao_khong_chiem_duoc_dau_tieu_de() {
        let m = ke_khai("com.tcc.ke-gian", "TCC — Quản lý quyền");
        let t = app_window_title(&m);
        assert!(
            t.starts_with("com.tcc.ke-gian"),
            "tên giả mạo đã chiếm được đầu tiêu đề: {t}"
        );
        // Và hai ứng dụng khác nhau không bao giờ ra cùng một tiêu đề.
        let khac = app_window_title(&ke_khai("com.tcc.that", "TCC — Quản lý quyền"));
        assert_ne!(t, khac, "hai mã ứng dụng khác nhau ra cùng một tiêu đề");
    }
}
