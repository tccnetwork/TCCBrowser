//! Kho khoá của ví — và ranh giới của nó.
//!
//! # Đọc `docs/vi-thiet-ke.md` trước
//!
//! Sự thật quyết định cả thiết kế: **Secure Enclave KHÔNG giữ được khoá của
//! TCC.** Nó chỉ hỗ trợ NIST P-256; Ed25519 không, ML-DSA-65 càng không. TPM
//! trên Windows cũng vậy. Nên:
//!
//! > Khoá bí mật **PHẢI** nằm trong bộ nhớ tiến trình vào lúc ký.
//!
//! Kho khoá bảo vệ khoá **lúc nằm yên**, không bảo vệ khoá **lúc dùng**. Crate
//! này được viết để làm điều đó rõ ràng ở tầng kiểu dữ liệu, chứ không chỉ rõ
//! ràng trong tài liệu mà ai đọc cũng quên.
//!
//! # Vì sao là crate riêng
//!
//! Cùng lý do `tcc-net` là crate riêng: **đọc `Cargo.toml` là biết ngay bộ nạp
//! ứng dụng không với tới được khoá.** Chỉ `tcc-shell` được phụ thuộc crate
//! này, và có luật kiến trúc cưỡng chế điều đó.

#![forbid(unsafe_code)]

use zeroize::ZeroizeOnDrop;

/// Khoá bí mật đang nằm trong bộ nhớ.
///
/// # Ba thứ kiểu này ép buộc
///
/// 1. **Tự xoá khi rời phạm vi** (`ZeroizeOnDrop`). Không xoá thì byte khoá còn
///    nằm lại trong vùng nhớ đã giải phóng, và một ảnh chụp bộ nhớ sau đó vẫn
///    đọc được.
/// 2. **Không in ra được.** `Debug` viết tay chỉ in độ dài. Một dòng
///    `dbg!(khoa)` lúc gỡ lỗi là khoá bí mật nằm trong nhật ký vĩnh viễn — và
///    đó là cách khoá rò ra thường xuyên hơn mọi lỗ hổng mật mã.
/// 3. **Không sao chép ngầm.** Không có `Clone`: mỗi bản sao là một chỗ nữa
///    phải nhớ xoá, và "nhớ xoá" là thứ người ta quên.
///
/// Điều thứ ba được **trình biên dịch** giữ, không phải kỷ luật giữ:
///
/// ```compile_fail
/// let k = tcc_keystore::SecretKey::new(vec![1, 2, 3]);
/// let _hai = k.clone();   // KHÔNG biên dịch được: `SecretKey` không có `Clone`
/// ```
///
/// Và cũng không mượn ra được một `Vec` sống lâu hơn khoá:
///
/// ```compile_fail
/// let muon: &[u8] = {
///     let k = tcc_keystore::SecretKey::new(vec![1, 2, 3]);
///     k.expose()          // KHÔNG biên dịch được: `k` chết ở cuối khối
/// };
/// ```
#[derive(ZeroizeOnDrop)]
pub struct SecretKey(Vec<u8>);

impl SecretKey {
    /// Nhận quyền sở hữu byte khoá.
    #[must_use]
    pub const fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Mượn byte khoá để ký.
    ///
    /// Trả `&[u8]` chứ không trả `Vec` là có chủ ý: bên gọi không cầm được một
    /// bản sao sống lâu hơn `SecretKey`, nên phạm vi cần-nhớ-xoá vẫn là một.
    #[must_use]
    pub fn expose(&self) -> &[u8] {
        &self.0
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// In ra ĐỘ DÀI, không bao giờ in nội dung.
impl core::fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SecretKey({} byte, nội dung bị giấu)", self.0.len())
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum KeystoreError {
    #[error("không có khoá nào tên \"{0}\"")]
    NotFound(String),

    #[error("đã có khoá tên \"{0}\" — KHÔNG ghi đè, vì ghi đè khoá ví là mất tiền")]
    AlreadyExists(String),

    #[error("người dùng từ chối xác thực")]
    UserRefused,

    #[error("kho khoá của hệ điều hành báo lỗi: {0}")]
    Os(String),
}

/// Vì sao khoá đang được lấy ra.
///
/// Kho khoá **PHẢI** hỏi người dùng cho từng lần ký, và để hỏi cho tử tế thì nó
/// cần biết hỏi vì việc gì. Một hộp thoại "Ứng dụng muốn dùng khoá của bạn"
/// không ai đọc; "Ký giao dịch gửi 5 TCC" thì có.
///
/// Đây cũng là lý do tham số này **bắt buộc**: quên đưa lý do là không biên
/// dịch được, chứ không phải là hiện ra một hộp thoại rỗng nghĩa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Purpose {
    /// Câu hiện cho người dùng, nguyên văn.
    pub prompt: String,
}

impl Purpose {
    #[must_use]
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
        }
    }
}

/// Kho khoá.
///
/// # Vì sao `unlock` nhận `Purpose` chứ không phải chỉ tên khoá
///
/// Vì mọi bản cài đặt đều PHẢI hỏi người dùng, và chữ hỏi là một phần của phép
/// hỏi. Cho phép lấy khoá mà không nêu lý do là mở đường cho việc ký lén.
pub trait Keystore {
    /// Cất một khoá mới. **KHÔNG ghi đè** khoá cùng tên.
    ///
    /// # Errors
    /// Đã có khoá cùng tên, hoặc hệ điều hành từ chối.
    fn store(&mut self, name: &str, key: SecretKey) -> Result<(), KeystoreError>;

    /// Lấy khoá ra để dùng, sau khi người dùng xác nhận.
    ///
    /// # Errors
    /// Không có khoá, người dùng từ chối, hoặc hệ điều hành từ chối.
    fn unlock(&self, name: &str, purpose: &Purpose) -> Result<SecretKey, KeystoreError>;

    /// Có khoá tên này không — **không** cần xác thực.
    ///
    /// Tách khỏi `unlock` để giao diện vẽ được "bạn đã có ví" mà không phải bắt
    /// người dùng chạm Touch ID chỉ để biết điều đó.
    fn contains(&self, name: &str) -> bool;

    /// Xoá vĩnh viễn.
    ///
    /// # Errors
    /// Không có khoá, hoặc hệ điều hành từ chối.
    fn delete(&mut self, name: &str) -> Result<(), KeystoreError>;
}

pub mod fake;

/// Kho khoá THẬT của macOS. Chỉ có khi bật cờ `os-keystore`.
///
/// Cờ tách riêng để dựng được bản trình duyệt KHÔNG có ví — chạy bản đó thì
/// chắc chắn không byte khoá nào được đọc, dù mã có lỗi gì. Đó là công cụ khi
/// soi bảo mật, không phải sự cầu kỳ.
#[cfg(all(feature = "os-keystore", target_os = "macos"))]
pub mod macos;

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "kiểm thử: hỏng thì phải nổ ngay")]
mod kiem_thu {
    use super::*;

    #[test]
    fn khoa_bi_mat_khong_in_noi_dung_ra() {
        let k = SecretKey::new(vec![0xAB; 64]);
        let ra = format!("{k:?}");
        assert!(ra.contains("64 byte"), "{ra}");
        assert!(
            !ra.contains("ab"),
            "nội dung khoá LỌT vào chuỗi Debug: {ra}"
        );
        assert!(!ra.contains("171"), "nội dung khoá LỌT ra dạng số: {ra}");
    }

    /// Lý do PHẢI có, không được để rỗng — hộp thoại rỗng nghĩa thì không ai đọc.
    #[test]
    fn ly_do_di_nguyen_van_toi_nguoi_dung() {
        let p = Purpose::new("Ký giao dịch gửi 5 TCC tới tcc1q…");
        assert!(p.prompt.contains("5 TCC"), "lý do bị cắt xén: {}", p.prompt);
    }
}
