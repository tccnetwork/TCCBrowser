//! Cây tệp của gói ứng dụng, và cách biến nó thành MỘT chuỗi byte.
//!
//! # Vì sao phải cẩn thận ở đây
//!
//! Toàn bộ chuỗi tin cậy treo vào một giá trị băm duy nhất. Nếu hai cây thư mục
//! KHÁC NHAU có thể cho ra cùng một chuỗi byte, thì kẻ gian ký một gói lành rồi
//! tráo sang gói độc mà chữ ký vẫn hợp lệ.
//!
//! Đòn kinh điển là **nhập nhằng khi nối chuỗi**. Nếu chỉ nối tên tệp với nội
//! dung:
//!
//! ```text
//! tệp "ab", nội dung "c"   →  "ab" + "c"  = "abc"
//! tệp "a",  nội dung "bc"  →  "a" + "bc"  = "abc"    ← TRÙNG
//! ```
//!
//! Hai cây khác hẳn nhau, một chuỗi byte. Cách chặn: **ghi độ dài trước mọi
//! trường**, để không có cách nào đọc ra hai kiểu.
//!
//! # Dạng chuẩn tắc
//!
//! ```text
//! với mỗi tệp, sắp theo thứ tự byte của đường dẫn:
//!     u64 độ dài đường dẫn (big-endian)
//!     byte của đường dẫn
//!     u64 độ dài nội dung (big-endian)
//!     byte của nội dung
//! ```
//!
//! Sắp xếp để hai lần đóng gói cùng một thư mục luôn ra cùng kết quả. Không có
//! thư mục rỗng trong dạng này — thư mục không mang nội dung nên không ký.
//!
//! # Giới hạn đã biết (0.1)
//!
//! `canonical_bytes` dựng toàn bộ gói trong bộ nhớ. Chấp nhận được với gói vài
//! chục megabyte; gói lớn hơn cần băm theo luồng, và đó là việc của phiên bản sau.

use std::collections::BTreeMap;

use thiserror::Error;

/// Trần độ dài một đường dẫn trong gói.
pub const MAX_PATH_LEN: usize = 1024;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TreeError {
    #[error("đường dẫn \"{0}\" rỗng")]
    EmptyPath(String),

    #[error("đường dẫn \"{path}\" không hợp lệ: {why}")]
    BadPath { path: String, why: &'static str },

    #[error("đường dẫn \"{0}\" xuất hiện hai lần")]
    DuplicatePath(String),

    #[error(
        "đường dẫn \"{a}\" và \"{b}\" chỉ khác nhau hoa/thường — trên macOS và \
         Windows chúng là CÙNG một tệp, gói sẽ chạy khác nhau tuỳ hệ điều hành"
    )]
    CaseCollision { a: String, b: String },
}

impl TreeError {
    /// Mã lỗi ỔN ĐỊNH, thuộc về TIÊU CHUẨN.
    ///
    /// Thông báo lỗi là văn xuôi tiếng Việt cho người đọc; nó được phép sửa cho
    /// dễ hiểu hơn bất cứ lúc nào. Mã này thì KHÔNG — bộ kiểm định tuân thủ và
    /// mọi bản triển khai bằng ngôn ngữ khác so khớp bằng nó. **Đổi một mã là
    /// đổi tiêu chuẩn**, phải tăng phiên bản đặc tả.
    #[must_use]
    pub const fn ma(&self) -> &'static str {
        match self {
            Self::EmptyPath { .. } => "empty-path",
            Self::BadPath { .. } => "bad-path",
            Self::DuplicatePath { .. } => "duplicate-path",
            Self::CaseCollision { .. } => "case-collision",
        }
    }
}

/// Cây tệp đã kiểm. Dựng được nó nghĩa là mọi ràng buộc đã qua.
#[derive(Debug, Clone, Default)]
pub struct FileTree {
    // BTreeMap để thứ tự luôn xác định, không phụ thuộc thứ tự chèn.
    files: BTreeMap<String, Vec<u8>>,
}

impl FileTree {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Thêm một tệp.
    ///
    /// # Errors
    /// Đường dẫn vi phạm ràng buộc, trùng, hoặc đụng nhau khi bỏ qua hoa/thường.
    pub fn insert(&mut self, path: &str, content: Vec<u8>) -> Result<(), TreeError> {
        check_path(path)?;
        if self.files.contains_key(path) {
            return Err(TreeError::DuplicatePath(path.to_string()));
        }
        // Trên macOS và Windows, "Logo.png" và "logo.png" là cùng một tệp. Một
        // gói chứa cả hai sẽ giải nén ra khác nhau tuỳ hệ điều hành — tức cùng
        // một chữ ký mà chạy ra hai thứ. Chặn từ đầu.
        let thuong = path.to_lowercase();
        if let Some(da_co) = self.files.keys().find(|k| k.to_lowercase() == thuong) {
            return Err(TreeError::CaseCollision {
                a: da_co.clone(),
                b: path.to_string(),
            });
        }
        self.files.insert(path.to_string(), content);
        Ok(())
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.files.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    #[must_use]
    pub fn paths(&self) -> Vec<&str> {
        self.files.keys().map(String::as_str).collect()
    }

    #[must_use]
    pub fn get(&self, path: &str) -> Option<&[u8]> {
        self.files.get(path).map(Vec::as_slice)
    }

    /// Dạng chuẩn tắc — thứ được đem đi băm.
    ///
    /// Mọi trường đều có độ dài ghi trước, nên không có cách nào đọc ra hai kiểu.
    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for (path, content) in &self.files {
            let p = path.as_bytes();
            out.extend_from_slice(&(p.len() as u64).to_be_bytes());
            out.extend_from_slice(p);
            out.extend_from_slice(&(content.len() as u64).to_be_bytes());
            out.extend_from_slice(content);
        }
        out
    }
}

/// Bản công khai của `check_path`, để `Manifest` kiểm trường `entry` dùng ĐÚNG
/// bộ luật đường dẫn — không có bộ luật thứ hai lệch pha.
///
/// # Errors
/// Đường dẫn vi phạm ràng buộc.
pub fn check_path_public(path: &str) -> Result<(), TreeError> {
    check_path(path)
}

/// Ràng buộc đường dẫn trong gói.
///
/// Chặt là có chủ đích. Mỗi luật chặn một đòn cụ thể:
fn check_path(path: &str) -> Result<(), TreeError> {
    let text = |why| {
        Err(TreeError::BadPath {
            path: path.to_string(),
            why,
        })
    };

    if path.is_empty() {
        return Err(TreeError::EmptyPath(path.to_string()));
    }
    if path.len() > MAX_PATH_LEN {
        return text("dài quá 1024 byte");
    }
    // Thoát ra ngoài thư mục gói — đòn "zip slip" kinh điển.
    if path.split('/').any(|d| d == "..") {
        return text("chứa \"..\" — có thể ghi ra ngoài thư mục gói");
    }
    if path.starts_with('/') {
        return text("là đường dẫn tuyệt đối");
    }
    // Windows: "C:\..." và cả tên ổ đĩa
    if path.contains(':') {
        return text("chứa dấu hai chấm — tên ổ đĩa hoặc luồng dữ liệu phụ trên Windows");
    }
    // Dấu gạch ngược không phải phân cách trong dạng chuẩn tắc. Cho phép nó thì
    // "a\\b" trên Linux là MỘT tệp, trên Windows là HAI cấp — cùng chữ ký, hai kết quả.
    if path.contains('\\') {
        return text("chứa dấu gạch ngược — chỉ dùng \"/\" làm phân cách");
    }
    if path.contains("//") || path.ends_with('/') {
        return text("có đoạn rỗng hoặc kết thúc bằng \"/\"");
    }
    if path.split('/').any(|d| d == ".") {
        return text("chứa \".\"");
    }
    // NUL và ký tự điều khiển: cắt chuỗi ở tầng hệ điều hành, gây lệch giữa cái
    // ta kiểm và cái hệ thống tệp thật sự tạo ra.
    if path.chars().any(char::is_control) {
        return text("chứa ký tự điều khiển");
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "kiểm thử: hỏng thì phải nổ ngay")]
mod kiem_thu {
    use super::*;

    fn cay(files: &[(&str, &[u8])]) -> FileTree {
        let mut t = FileTree::new();
        for (p, c) in files {
            t.insert(p, c.to_vec()).unwrap();
        }
        t
    }

    /// ⚠️ PHÉP THỬ QUAN TRỌNG NHẤT TỆP NÀY.
    ///
    /// Không ghi độ dài trước thì hai cây dưới đây cho ra cùng chuỗi "abc", và
    /// một chữ ký hợp lệ cho cả hai — tức tráo được nội dung mà chữ ký vẫn đúng.
    #[test]
    fn khong_nhap_nhang_khi_noi_chuoi() {
        let a = cay(&[("ab", b"c")]);
        let b = cay(&[("a", b"bc")]);
        assert_ne!(
            a.canonical_bytes(),
            b.canonical_bytes(),
            "hai cây khác nhau cho ra cùng chuỗi byte — chữ ký sẽ nhận nhầm"
        );
    }

    /// Biến thể khác của cùng một đòn: dịch ranh giới giữa hai tệp.
    #[test]
    fn khong_nhap_nhang_giua_hai_tep() {
        let a = cay(&[("a", b"1"), ("b", b"2")]);
        let b = cay(&[("a", b"1b2")]);
        assert_ne!(a.canonical_bytes(), b.canonical_bytes());
    }

    /// Thứ tự chèn không được ảnh hưởng kết quả — nếu không, đóng gói hai lần
    /// cùng một thư mục lại ra hai chữ ký khác nhau.
    #[test]
    fn thu_tu_chen_khong_anh_huong() {
        let a = cay(&[("z.txt", b"1"), ("a.txt", b"2")]);
        let b = cay(&[("a.txt", b"2"), ("z.txt", b"1")]);
        assert_eq!(a.canonical_bytes(), b.canonical_bytes());
    }

    /// Đổi chỗ nội dung giữa hai tệp PHẢI đổi kết quả.
    #[test]
    fn doi_cho_noi_dung_thi_doi_ket_qua() {
        let a = cay(&[("a.txt", b"mot"), ("b.txt", b"hai")]);
        let b = cay(&[("a.txt", b"hai"), ("b.txt", b"mot")]);
        assert_ne!(a.canonical_bytes(), b.canonical_bytes());
    }

    /// Đổi tên tệp mà giữ nội dung cũng phải đổi kết quả.
    #[test]
    fn doi_ten_tep_thi_doi_ket_qua() {
        let a = cay(&[("a.txt", b"x")]);
        let b = cay(&[("b.txt", b"x")]);
        assert_ne!(a.canonical_bytes(), b.canonical_bytes());
    }

    #[test]
    fn chan_thoat_ra_ngoai_thu_muc() {
        let mut t = FileTree::new();
        for xau in [
            "../ngoai.txt",
            "a/../../ngoai.txt",
            "/etc/passwd",
            "a/./b.txt",
            "a//b.txt",
            "a/",
            "C:/windows/x.txt",
            "a\\b.txt",
        ] {
            assert!(t.insert(xau, vec![]).is_err(), "phải chặn: {xau:?}");
        }
    }

    #[test]
    fn chan_ky_tu_dieu_khien_trong_duong_dan() {
        let mut t = FileTree::new();
        assert!(t.insert("a\u{0}b.txt", vec![]).is_err());
        assert!(t.insert("a\nb.txt", vec![]).is_err());
    }

    #[test]
    fn chan_duong_dan_trung() {
        let mut t = FileTree::new();
        t.insert("a.txt", b"1".to_vec()).unwrap();
        assert!(matches!(
            t.insert("a.txt", b"2".to_vec()),
            Err(TreeError::DuplicatePath(_))
        ));
    }

    /// ⚠️ Trên macOS và Windows, "Logo.png" và "logo.png" là CÙNG một tệp.
    /// Gói chứa cả hai sẽ giải nén ra khác nhau tuỳ hệ điều hành — cùng một chữ
    /// ký mà chạy ra hai thứ khác nhau.
    #[test]
    fn chan_ten_chi_khac_hoa_thuong() {
        let mut t = FileTree::new();
        t.insert("Logo.png", b"that".to_vec()).unwrap();
        assert!(matches!(
            t.insert("logo.png", b"gia".to_vec()),
            Err(TreeError::CaseCollision { .. })
        ));
    }

    #[test]
    fn duong_dan_nhieu_cap_thi_dung_duoc() {
        let t = cay(&[("wasm/app.wasm", b"\0asm"), ("anh/logo.png", b"png")]);
        assert_eq!(t.len(), 2);
        assert_eq!(t.get("wasm/app.wasm"), Some(&b"\0asm"[..]));
    }

    #[test]
    fn cay_rong_cho_ra_chuoi_rong() {
        assert!(FileTree::new().canonical_bytes().is_empty());
    }

    /// Tệp rỗng KHÁC với không có tệp — dạng chuẩn tắc phải phân biệt được.
    #[test]
    fn tep_rong_khac_voi_khong_co_tep() {
        let a = cay(&[("a.txt", b"")]);
        assert_ne!(a.canonical_bytes(), FileTree::new().canonical_bytes());
    }
}
