//! Mật mã — BIÊN GIỚI TIN CẬY của toàn dự án.
//!
//! BỐN LUẬT CỨNG:
//!
//! 1. **Ít phụ thuộc nhất workspace.** Mỗi phụ thuộc thêm vào đây là mở rộng bề
//!    mặt cần kiểm định. Thêm crate ngoài phải ghi lý do vào SECURITY.md.
//! 2. **LAI, không thuần hậu lượng tử.** Chữ ký hợp lệ khi CẢ HAI thành phần
//!    hợp lệ. SIKE từng vào chung kết NIST và bị phá trong một giờ trên một nhân
//!    CPU — thuật toán hậu lượng tử còn quá trẻ để tin một mình.
//! 3. **Không tự cài đặt thuật toán.** Dùng thư viện đã được soi kỹ.
//! 4. **Thuật toán thay được** qua trait, không `if algo == "..."` rải rác.
//!
//! KHÔNG cần thay mã hoá đối xứng: AES-256 và SHA-384 chỉ bị Grover làm yếu đi
//! một nửa số bit, vẫn thừa an toàn. Chỉ mã hoá BẤT ĐỐI XỨNG mới bị Shor phá.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CryptoError {
    /// Chữ ký không khớp. Nói rõ THÀNH PHẦN NÀO hỏng — "chữ ký sai" là thông báo
    /// vô dụng khi đang gỡ lỗi một chữ ký lai.
    #[error("chữ ký {part} không hợp lệ")]
    BadSignature { part: &'static str },

    #[error("độ dài {field} sai: chờ {expected} byte, nhận {actual}")]
    BadLength {
        field: &'static str,
        expected: usize,
        actual: usize,
    },

    #[error("dữ liệu khoá không đọc được: {0}")]
    BadKey(&'static str),
}

impl CryptoError {
    /// Mã lỗi ỔN ĐỊNH, thuộc về TIÊU CHUẨN.
    ///
    /// Thông báo lỗi là văn xuôi tiếng Việt cho người đọc; nó được phép sửa cho
    /// dễ hiểu hơn bất cứ lúc nào. Mã này thì KHÔNG — bộ kiểm định tuân thủ và
    /// mọi bản triển khai bằng ngôn ngữ khác so khớp bằng nó. **Đổi một mã là
    /// đổi tiêu chuẩn**, phải tăng phiên bản đặc tả.
    #[must_use]
    pub const fn ma(&self) -> &'static str {
        match self {
            Self::BadSignature { .. } => "bad-signature",
            Self::BadLength { .. } => "bad-length",
            Self::BadKey { .. } => "bad-key",
        }
    }
}

/// Một bộ ký. Cài đặt cụ thể nằm SAU trait này để đổi thuật toán mà không phải
/// sửa chỗ gọi — luật số 4.
pub trait SignatureScheme {
    /// Tên ghi vào bản kê khai. Đây là phần của TIÊU CHUẨN, đổi là phá tương thích.
    fn name(&self) -> &'static str;

    /// Ký một thông điệp.
    ///
    /// # Errors
    /// Khoá bí mật sai độ dài, hoặc không giải mã được thành khoá hợp lệ.
    fn sign(&self, secret: &[u8], message: &[u8]) -> Result<Vec<u8>, CryptoError>;

    /// Kiểm một chữ ký.
    ///
    /// # Errors
    /// Khoá công khai hoặc chữ ký sai độ dài, hoặc chữ ký không khớp. Với bộ ký
    /// lai, lỗi nói rõ THÀNH PHẦN nào hỏng — "chữ ký sai" là thông báo vô dụng
    /// khi đang gỡ lỗi hai nửa.
    fn verify(&self, public: &[u8], message: &[u8], signature: &[u8]) -> Result<(), CryptoError>;
}

pub mod hash;
pub mod hybrid;

pub use hash::content_hash_hex;
pub use hybrid::HybridEd25519MlDsa;
