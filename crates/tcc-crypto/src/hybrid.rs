//! Chữ ký LAI: Ed25519 (cổ điển) + ML-DSA-65 (hậu lượng tử, FIPS 204).
//!
//! # Vì sao lai chứ không thuần hậu lượng tử
//!
//! Năm 2022, **SIKE** — một ứng viên đã vào vòng chung kết NIST — bị phá trên
//! MỘT nhân CPU trong khoảng MỘT GIỜ. Thuật toán hậu lượng tử còn quá trẻ và
//! chưa bị soi kỹ như đường cong elliptic. Ghép hai họ thuật toán khác hẳn nhau
//! thì chữ ký còn an toàn khi **một trong hai** còn đứng vững.
//!
//! # Tính chất bảo mật quan trọng nhất của tệp này
//!
//! `verify` chỉ trả `Ok` khi **CẢ HAI** nửa hợp lệ. Đây là chỗ dễ viết sai nhất:
//! một dấu `||` thay cho `&&`, hoặc một nhánh trả sớm, là chữ ký lai tụt xuống
//! chỉ còn mạnh bằng nửa yếu hơn — mà không có phép thử nào thất bại nếu chỉ thử
//! đường chạy thuận. Xem `kiem_thu::gia_mao_nua_hau_luong_tu` và
//! `kiem_thu::gia_mao_nua_co_dien`.
//!
//! # Bố cục byte — ĐÂY LÀ PHẦN CỦA TIÊU CHUẨN
//!
//! Đổi mấy con số này là phá tương thích với mọi gói đã ký trước đó.
//!
//! ```text
//! Khoá công khai:  [ Ed25519: 32B ][ ML-DSA-65: 1952B ]
//! Chữ ký:          [ Ed25519: 64B ][ ML-DSA-65: 3309B ]
//! ```

use crate::{CryptoError, SignatureScheme};

// `Signer`/`Verifier` là CÙNG một trait từ crate `signature` — cả ed25519-dalek
// lẫn ml-dsa đều cài đặt nó. Nhập một lần, dùng cho cả hai nửa.
use signature::{Signer as _, Verifier as _};

use ed25519_dalek::{
    Signature as EdSignature, SigningKey as EdSigningKey, VerifyingKey as EdVerifyingKey,
};
use ml_dsa::{
    Generate as _, KeyExport as _, KeyInit as _, KeySizeUser as _, Keypair as _, MlDsa65,
    SigningKey as PqSigningKey, VerifyingKey as PqVerifyingKey,
};

/// Ed25519: khoá bí mật 32 byte, khoá công khai 32 byte, chữ ký 64 byte.
const ED_SECRET: usize = 32;
const ED_PUBLIC: usize = 32;
const ED_SIG: usize = 64;

/// Tên ghi vào bản kê khai. Phần của tiêu chuẩn — đổi là phá tương thích.
const SCHEME_NAME: &str = "hybrid-ed25519-mldsa65-v1";

/// Bộ ký lai. Không giữ trạng thái — mọi khoá truyền vào từng lời gọi.
#[derive(Debug, Clone, Copy, Default)]
pub struct HybridEd25519MlDsa;

/// Cặp khoá đã tuần tự hoá, dùng cho `tcc-cli` và kiểm thử.
#[derive(Debug, Clone)]
pub struct KeyPair {
    pub secret: Vec<u8>,
    pub public: Vec<u8>,
}

impl HybridEd25519MlDsa {
    /// Sinh cặp khoá mới từ nguồn ngẫu nhiên của hệ điều hành.
    ///
    /// Khoá bí mật lai = [hạt giống Ed25519 32B][hạt giống ML-DSA 32B]. Giữ dạng
    /// HẠT GIỐNG chứ không phải khoá đã bung: ngắn hơn nhiều, và bung lại được
    /// bất cứ lúc nào theo FIPS 204.
    /// # Panics
    /// Dừng chương trình nếu nguồn ngẫu nhiên của hệ điều hành hỏng. Đây là CHỦ
    /// ĐÍCH: sinh khoá bằng số ngẫu nhiên kém là hỏng âm thầm, và một khoá yếu
    /// còn nguy hiểm hơn không có khoá — người dùng vẫn tưởng mình được bảo vệ.
    #[must_use]
    #[allow(
        clippy::expect_used,
        reason = "nguồn ngẫu nhiên hỏng thì không có cách chạy tiếp an toàn"
    )]
    pub fn generate() -> KeyPair {
        // Lấy thẳng 32 byte từ nguồn ngẫu nhiên của hệ điều hành thay vì kéo
        // thêm một crate RNG vào biên giới tin cậy — luật số 1.
        let mut ed_seed = [0u8; ED_SECRET];
        getrandom::fill(&mut ed_seed)
            .expect("nguồn ngẫu nhiên của hệ điều hành hỏng — không có cách chạy tiếp an toàn");
        let ed = EdSigningKey::from_bytes(&ed_seed);
        let pq = PqSigningKey::<MlDsa65>::generate();

        let mut secret = Vec::with_capacity(ED_SECRET + 32);
        secret.extend_from_slice(ed.as_bytes());
        secret.extend_from_slice(&pq.to_bytes());

        let mut public = Vec::new();
        public.extend_from_slice(ed.verifying_key().as_bytes());
        public.extend_from_slice(&pq.verifying_key().to_bytes());

        KeyPair { secret, public }
    }
}

impl HybridEd25519MlDsa {
    /// Suy khoá công khai từ khoá bí mật.
    ///
    /// Cần cho `tcc sign`: người dùng chỉ giữ khoá bí mật, không nên phải tự dán
    /// khoá công khai vào bản kê khai — dán tay là dán nhầm, và dán nhầm thì gói
    /// ký xong không ai kiểm được.
    ///
    /// # Errors
    /// Khoá bí mật sai độ dài hoặc không bung được.
    pub fn public_from_secret(secret: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let pq_seed_len = PqSigningKey::<MlDsa65>::key_size();
        let want = ED_SECRET + pq_seed_len;

        let ed_bytes = take(secret, 0, ED_SECRET, "khoá bí mật", want)?;
        let pq_bytes = take(secret, ED_SECRET, pq_seed_len, "khoá bí mật", want)?;

        let ed_key: [u8; ED_SECRET] = ed_bytes
            .try_into()
            .map_err(|_| CryptoError::BadKey("Ed25519 phải đúng 32 byte"))?;
        let ed = EdSigningKey::from_bytes(&ed_key);
        let pq = PqSigningKey::<MlDsa65>::new_from_slice(pq_bytes)
            .map_err(|_| CryptoError::BadKey("hạt giống ML-DSA sai độ dài"))?;

        let mut out = Vec::new();
        out.extend_from_slice(ed.verifying_key().as_bytes());
        out.extend_from_slice(&pq.verifying_key().to_bytes());
        Ok(out)
    }
}

/// Cắt một lát có kiểm độ dài, báo lỗi nói rõ trường nào sai.
fn take<'a>(
    data: &'a [u8],
    at: usize,
    len: usize,
    field: &'static str,
    total: usize,
) -> Result<&'a [u8], CryptoError> {
    data.get(at..at + len).ok_or(CryptoError::BadLength {
        field,
        expected: total,
        actual: data.len(),
    })
}

impl SignatureScheme for HybridEd25519MlDsa {
    fn name(&self) -> &'static str {
        SCHEME_NAME
    }

    fn sign(&self, secret: &[u8], message: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let pq_seed_len = PqSigningKey::<MlDsa65>::key_size();
        let want = ED_SECRET + pq_seed_len;

        let ed_bytes = take(secret, 0, ED_SECRET, "khoá bí mật", want)?;
        let pq_bytes = take(secret, ED_SECRET, pq_seed_len, "khoá bí mật", want)?;

        let ed_key: [u8; ED_SECRET] = ed_bytes
            .try_into()
            .map_err(|_| CryptoError::BadKey("Ed25519 phải đúng 32 byte"))?;
        let ed = EdSigningKey::from_bytes(&ed_key);
        let pq = PqSigningKey::<MlDsa65>::new_from_slice(pq_bytes)
            .map_err(|_| CryptoError::BadKey("hạt giống ML-DSA sai độ dài"))?;

        let mut out = Vec::new();
        out.extend_from_slice(&ed.sign(message).to_bytes());
        out.extend_from_slice(&pq.sign(message).encode());
        Ok(out)
    }

    /// Kiểm chữ ký lai.
    ///
    /// ⚠️ CẢ HAI nửa phải hợp lệ. Không có đường tắt nào ở đây — nếu ai đó sửa
    /// hàm này thành trả `Ok` sớm khi một nửa đạt, chữ ký lai mất hết ý nghĩa.
    fn verify(&self, public: &[u8], message: &[u8], signature: &[u8]) -> Result<(), CryptoError> {
        let pq_pub_len = PqVerifyingKey::<MlDsa65>::key_size();
        let want_pub = ED_PUBLIC + pq_pub_len;

        let ed_pub = take(public, 0, ED_PUBLIC, "khoá công khai", want_pub)?;
        let pq_pub = take(public, ED_PUBLIC, pq_pub_len, "khoá công khai", want_pub)?;

        let ed_sig_len = ED_SIG;
        let pq_sig_len = signature.len().saturating_sub(ed_sig_len);
        let ed_sig = take(signature, 0, ed_sig_len, "chữ ký", ed_sig_len + pq_sig_len)?;
        let pq_sig = take(
            signature,
            ed_sig_len,
            pq_sig_len,
            "chữ ký",
            ed_sig_len + pq_sig_len,
        )?;

        // ---- Nửa cổ điển ----
        let ed_pub_arr: [u8; ED_PUBLIC] = ed_pub
            .try_into()
            .map_err(|_| CryptoError::BadKey("Ed25519 phải đúng 32 byte"))?;
        let ed_sig_arr: [u8; ED_SIG] = ed_sig
            .try_into()
            .map_err(|_| CryptoError::BadSignature { part: "Ed25519" })?;
        let ed_key = EdVerifyingKey::from_bytes(&ed_pub_arr)
            .map_err(|_| CryptoError::BadKey("khoá công khai Ed25519 không hợp lệ"))?;
        ed_key
            .verify(message, &EdSignature::from_bytes(&ed_sig_arr))
            .map_err(|_| CryptoError::BadSignature { part: "Ed25519" })?;

        // ---- Nửa hậu lượng tử ----
        // Tới được đây nghĩa là nửa cổ điển đã đạt — nhưng vẫn PHẢI kiểm nốt.
        let pq_key = PqVerifyingKey::<MlDsa65>::new_from_slice(pq_pub)
            .map_err(|_| CryptoError::BadKey("khoá công khai ML-DSA sai độ dài"))?;
        let pq_signature = ml_dsa::Signature::<MlDsa65>::try_from(pq_sig)
            .map_err(|_| CryptoError::BadSignature { part: "ML-DSA-65" })?;
        pq_key
            .verify(message, &pq_signature)
            .map_err(|_| CryptoError::BadSignature { part: "ML-DSA-65" })?;

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "kiểm thử: hỏng thì phải nổ ngay")]
mod kiem_thu {
    use super::*;

    fn bo_ky() -> (HybridEd25519MlDsa, KeyPair) {
        (HybridEd25519MlDsa, HybridEd25519MlDsa::generate())
    }

    #[test]
    fn ky_roi_kiem_thi_dat() {
        let (s, k) = bo_ky();
        let msg = b"ban ke khai ung dung TCC";
        let sig = s.sign(&k.secret, msg).unwrap();
        assert!(s.verify(&k.public, msg, &sig).is_ok());
    }

    #[test]
    fn sua_mot_byte_trong_thong_diep_thi_hong() {
        let (s, k) = bo_ky();
        let sig = s.sign(&k.secret, b"noi dung goc").unwrap();
        assert!(s.verify(&k.public, b"noi dung khac", &sig).is_err());
    }

    /// ⚠️ PHÉP THỬ QUAN TRỌNG NHẤT TỆP NÀY.
    ///
    /// Giữ nguyên nửa Ed25519 hợp lệ, phá nửa ML-DSA. Nếu `verify` dùng `||` hay
    /// trả sớm, phép thử này ĐẠT — và chữ ký lai chỉ còn mạnh bằng Ed25519, tức
    /// vô nghĩa trước máy tính lượng tử.
    #[test]
    fn gia_mao_nua_hau_luong_tu() {
        let (s, k) = bo_ky();
        let msg = b"chuyen 1000 TCC";
        let mut sig = s.sign(&k.secret, msg).unwrap();

        let cuoi = sig.len() - 1;
        sig[cuoi] ^= 0xFF;

        assert_eq!(
            s.verify(&k.public, msg, &sig),
            Err(CryptoError::BadSignature { part: "ML-DSA-65" }),
            "nửa hậu lượng tử hỏng mà vẫn qua — chữ ký lai đã mất tác dụng"
        );
    }

    /// Mặt còn lại của phép thử trên: phá nửa cổ điển, giữ nửa hậu lượng tử.
    #[test]
    fn gia_mao_nua_co_dien() {
        let (s, k) = bo_ky();
        let msg = b"chuyen 1000 TCC";
        let mut sig = s.sign(&k.secret, msg).unwrap();

        sig[0] ^= 0xFF;

        assert_eq!(
            s.verify(&k.public, msg, &sig),
            Err(CryptoError::BadSignature { part: "Ed25519" }),
            "nửa cổ điển hỏng mà vẫn qua"
        );
    }

    #[test]
    fn khoa_cong_khai_cua_nguoi_khac_thi_hong() {
        let (s, k) = bo_ky();
        let nguoi_khac = HybridEd25519MlDsa::generate();
        let msg = b"thong diep";
        let sig = s.sign(&k.secret, msg).unwrap();
        assert!(s.verify(&nguoi_khac.public, msg, &sig).is_err());
    }

    #[test]
    fn chu_ky_cut_thi_bao_loi_ro_rang() {
        let (s, k) = bo_ky();
        let msg = b"thong diep";
        let sig = s.sign(&k.secret, msg).unwrap();
        let ket_qua = s.verify(&k.public, msg, &sig[..10]);
        assert!(matches!(ket_qua, Err(CryptoError::BadLength { .. })));
    }

    #[test]
    fn khoa_cut_thi_bao_loi_ro_rang() {
        let (s, k) = bo_ky();
        assert!(matches!(
            s.sign(&k.secret[..5], b"x"),
            Err(CryptoError::BadLength { .. })
        ));
    }

    /// Tên bộ ký là phần của TIÊU CHUẨN. Đổi nó là phá mọi gói đã ký.
    /// Phép thử này tồn tại để việc đổi tên không bao giờ xảy ra vô tình.
    #[test]
    fn ten_bo_ky_khong_duoc_doi_vo_tinh() {
        assert_eq!(HybridEd25519MlDsa.name(), "hybrid-ed25519-mldsa65-v1");
    }

    /// Khoá công khai suy ra từ khoá bí mật phải TRÙNG với khoá sinh cùng lúc.
    /// Sai chỗ này là gói ký xong không ai kiểm được.
    #[test]
    fn suy_khoa_cong_khai_tu_khoa_bi_mat() {
        let k = HybridEd25519MlDsa::generate();
        assert_eq!(
            HybridEd25519MlDsa::public_from_secret(&k.secret).unwrap(),
            k.public
        );
    }

    #[test]
    fn suy_khoa_tu_khoa_bi_mat_cut_thi_bao_loi() {
        assert!(matches!(
            HybridEd25519MlDsa::public_from_secret(&[0u8; 10]),
            Err(CryptoError::BadLength { .. })
        ));
    }

    /// Bố cục byte cũng là phần của tiêu chuẩn — chốt lại bằng số cụ thể.
    #[test]
    fn bo_cuc_byte_dung_nhu_tieu_chuan() {
        let k = HybridEd25519MlDsa::generate();
        let sig = HybridEd25519MlDsa.sign(&k.secret, b"x").unwrap();
        assert_eq!(k.public.len(), 32 + 1952, "khoá công khai lai");
        assert_eq!(sig.len(), 64 + 3309, "chữ ký lai");
    }
}
