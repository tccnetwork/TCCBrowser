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

        let ed_key: [u8; ED_SECRET] = ed_bytes.try_into().map_err(|_| CryptoError::BadLength {
            field: "khoá Ed25519",
            expected: ED_SECRET,
            actual: ed_bytes.len(),
        })?;
        let ed = EdSigningKey::from_bytes(&ed_key);
        let pq = PqSigningKey::<MlDsa65>::new_from_slice(pq_bytes).map_err(|_| {
            CryptoError::BadLength {
                field: "hạt giống ML-DSA",
                expected: pq_seed_len,
                actual: pq_bytes.len(),
            }
        })?;

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

        let ed_key: [u8; ED_SECRET] = ed_bytes.try_into().map_err(|_| CryptoError::BadLength {
            field: "khoá Ed25519",
            expected: ED_SECRET,
            actual: ed_bytes.len(),
        })?;
        let ed = EdSigningKey::from_bytes(&ed_key);
        let pq = PqSigningKey::<MlDsa65>::new_from_slice(pq_bytes).map_err(|_| {
            CryptoError::BadLength {
                field: "hạt giống ML-DSA",
                expected: pq_seed_len,
                actual: pq_bytes.len(),
            }
        })?;

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

        // ⚠️ Độ dài chờ đợi phải là độ dài THẬT của một chữ ký lai, không phải
        // độ dài suy ra từ chính đầu vào đang bị nghi. Bản đầu viết
        // `pq_sig_len = signature.len().saturating_sub(ED_SIG)`, nên một chữ ký
        // 10 byte làm `pq_sig_len` thành 0 và thông báo báo "chờ 64 byte" —
        // trong khi con số thật là 3373. Người đọc câu ấy đi tìm một chữ ký 64
        // byte, mãi mãi. Cùng hạng lỗi với `bad-key`: câu lỗi kể một chuyện
        // không có thật.
        let ed_sig_len = ED_SIG;
        let pq_sig_len = ml_dsa::EncodedSignature::<MlDsa65>::default().len();
        let want_sig = ed_sig_len + pq_sig_len;
        // ⚠️ Kiểm ĐỘ DÀI TỔNG trước khi cắt, và kiểm bằng `!=` chứ không phải
        // `<`. Cách cũ suy `pq_sig_len` TỪ CHÍNH đầu vào, nên nó vô tình ép
        // tổng phải khớp: thừa một byte thì nửa hậu lượng tử dài 3310 và
        // `Signature::try_from` chối. Bỏ cách suy ấy mà không thay bằng phép
        // kiểm này thì byte thừa bị `take` BỎ QUA và chữ ký vẫn hợp lệ — chữ
        // ký DẺO. Vector `them mot byte thua` bắt được đúng lúc đổi (25/08/2026).
        if signature.len() != want_sig {
            return Err(CryptoError::BadLength {
                field: "chữ ký",
                expected: want_sig,
                actual: signature.len(),
            });
        }
        let ed_sig = take(signature, 0, ed_sig_len, "chữ ký", want_sig)?;
        let pq_sig = take(signature, ed_sig_len, pq_sig_len, "chữ ký", want_sig)?;

        // ---- Nửa cổ điển ----
        let ed_pub_arr: [u8; ED_PUBLIC] =
            ed_pub.try_into().map_err(|_| CryptoError::BadLength {
                field: "khoá công khai Ed25519",
                expected: ED_PUBLIC,
                actual: ed_pub.len(),
            })?;
        let ed_sig_arr: [u8; ED_SIG] = ed_sig
            .try_into()
            .map_err(|_| CryptoError::BadSignature { part: "Ed25519" })?;
        // ⚠️ ĐÂY LÀ CHỖ DUY NHẤT một khoá đúng độ dài vẫn hỏng: 32 byte có thể
        // không phải một điểm trên đường cong. `spec/0.1/06-error-codes.md:147`
        // đã GỠ mã `bad-key` với lý do "thư viện Ed25519 thường kiểm điểm lười,
        // nên khoá không giải nén được hiện ra thành `bad-signature`" — nhưng
        // `ed25519-dalek` 3.0 kiểm NGAY tại `from_bytes`, nên bản này bắn ra
        // `bad-key` cho gói mà bản khác bắn `bad-signature`. Đúng cái bất đồng
        // mà mã ổn định sinh ra để ngăn. Theo đặc tả.
        let ed_key = EdVerifyingKey::from_bytes(&ed_pub_arr)
            .map_err(|_| CryptoError::BadSignature { part: "Ed25519" })?;
        ed_key
            .verify(message, &EdSignature::from_bytes(&ed_sig_arr))
            .map_err(|_| CryptoError::BadSignature { part: "Ed25519" })?;

        // ---- Nửa hậu lượng tử ----
        // Tới được đây nghĩa là nửa cổ điển đã đạt — nhưng vẫn PHẢI kiểm nốt.
        let pq_key = PqVerifyingKey::<MlDsa65>::new_from_slice(pq_pub).map_err(|_| {
            CryptoError::BadLength {
                field: "khoá công khai ML-DSA",
                expected: pq_pub_len,
                actual: pq_pub.len(),
            }
        })?;
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

    /// **Khoá công khai đúng độ dài nhưng KHÔNG phải điểm trên đường cong phải
    /// ra `bad-signature`, không phải một mã ngoài tiêu chuẩn.**
    ///
    /// `spec/0.1/06-error-codes.md:147` liệt kê `bad-key` trong "ba mã đã gỡ vì
    /// không thể xảy ra", với lý lẽ: thư viện Ed25519 thường kiểm điểm LƯỜI, tới
    /// lúc verify mới kiểm, nên khoá không giải nén được hiện ra thành
    /// `bad-signature`.
    ///
    /// Lý lẽ ấy KHÔNG đúng với thư viện ta dùng. `ed25519-dalek` 3.0 kiểm ngay
    /// trong `from_bytes`, nên tới 25/08/2026 bản này bắn ra `bad-key` — một mã
    /// tiêu chuẩn nói là không tồn tại — cho một gói mà bản triển khai khác bắn
    /// `bad-signature`. Đúng cái bất đồng mà mã ổn định sinh ra để ngăn.
    ///
    /// Tìm ra bằng `cargo-mutants`: đột biến trên `CryptoError::ma` sống sót vì
    /// `bad-key` không có trong vector nào, và nó không có trong vector nào vì
    /// tiêu chuẩn bảo nó không thể xảy ra.
    ///
    /// 32 byte `0x7f` là mẫu tìm ra bằng thăm dò: `0xff` và `0x01` đều bị chối
    /// muộn hơn (`bad-signature`), chỉ `0x7f` làm `from_bytes` hỏng sớm.
    #[test]
    fn khoa_khong_phai_diem_van_ra_bad_signature() {
        let (s, k) = bo_ky();
        let msg = b"chuyen 1000 TCC";
        let sig = s.sign(&k.secret, msg).unwrap();

        let mut xau = k.public.clone();
        for b in xau.iter_mut().take(ED_PUBLIC) {
            *b = 0x7f;
        }

        let loi = s.verify(&xau, msg, &sig).unwrap_err();
        assert_eq!(
            loi.ma(),
            "bad-signature",
            "mã ngoài tiêu chuẩn cho khoá hỏng: {loi}"
        );
    }

    /// **Lỗi sai độ dài phải nói ĐÚNG con số nó chờ.**
    ///
    /// `cargo-mutants` ngày 25/08/2026 đổi `+` thành `*` và `-` ở bảy chỗ tính
    /// `want` — cả bảy SỐNG. Đọc thân `take` thì thấy `total` chỉ đi vào trường
    /// `expected` của thông báo, còn quyết định cắt lát dùng `at..at + len`:
    /// hành vi KHÔNG đổi, mọi khoá sai độ dài vẫn bị chối.
    ///
    /// Vẫn ghim, và lý do phân biệt rạch ròi với luật "thông báo là văn xuôi
    /// được phép sửa": ở đây không phải câu chữ mà là một CON SỐ. Báo "chờ
    /// 3968 byte" trong khi thật ra chờ 1984 là một câu SAI SỰ THẬT, và người
    /// đọc nó sẽ đi sửa khoá của mình cho khớp một con số không có thật.
    #[test]
    fn loi_sai_do_dai_noi_dung_con_so_no_cho() {
        let (s, k) = bo_ky();
        let dung = k.secret.len();

        // Thiếu một byte: phải báo đúng độ dài thật, không phải một biến thể
        // của nó.
        let thieu = &k.secret[..dung - 1];
        match s.sign(thieu, b"x").unwrap_err() {
            CryptoError::BadLength {
                expected, actual, ..
            } => {
                assert_eq!(expected, dung, "nói sai độ dài đang chờ");
                assert_eq!(actual, dung - 1, "nói sai độ dài nhận được");
            }
            khac @ CryptoError::BadSignature { .. } => {
                panic!("chờ BadLength, nhận {khac:?}")
            }
        }

        // `public_from_secret` tính `want` bằng MỘT DÒNG RIÊNG — kiểm đột biến
        // chỉ dòng của `sign` thì dòng này vẫn sống. Ba đường, ba lần tính,
        // ba lần phải nói đúng.
        match HybridEd25519MlDsa::public_from_secret(thieu).unwrap_err() {
            CryptoError::BadLength { expected, .. } => {
                assert_eq!(expected, dung, "public_from_secret nói sai độ dài");
            }
            khac @ CryptoError::BadSignature { .. } => {
                panic!("chờ BadLength, nhận {khac:?}")
            }
        }

        let sig = s.sign(&k.secret, b"x").unwrap();

        // Chữ ký ngắn: con số phải là độ dài THẬT của chữ ký lai, không phải
        // độ dài suy ra từ chính đầu vào đang bị nghi.
        let ngan = &sig[..10];
        match s.verify(&k.public, b"x", ngan).unwrap_err() {
            CryptoError::BadLength {
                expected, actual, ..
            } => {
                assert_eq!(expected, sig.len(), "nói sai độ dài chữ ký đang chờ");
                assert_eq!(actual, 10);
            }
            khac @ CryptoError::BadSignature { .. } => {
                panic!("chờ BadLength, nhận {khac:?}")
            }
        }

        // Cùng chuyện với khoá công khai ở đường xác minh.
        let dung_pub = k.public.len();
        match s.verify(&k.public[..dung_pub - 1], b"x", &sig).unwrap_err() {
            CryptoError::BadLength { expected, .. } => {
                assert_eq!(expected, dung_pub, "nói sai độ dài khoá công khai");
            }
            khac @ CryptoError::BadSignature { .. } => {
                panic!("chờ BadLength, nhận {khac:?}")
            }
        }
    }

    /// **Thừa MỘT byte ở cuối chữ ký phải bị chối.**
    ///
    /// Chữ ký dẻo: nếu byte thừa bị bỏ qua thì cùng một thông điệp có VÔ SỐ
    /// chữ ký hợp lệ, và bất kỳ chỗ nào đối chiếu chữ ký theo byte — sổ ghi,
    /// bộ nhớ đệm, phép chống phát lại — đều bị qua mặt.
    ///
    /// Phép thử này tồn tại vì ngày 25/08/2026 tôi TỰ TAY tạo ra lỗ ấy: sửa
    /// `pq_sig_len` từ "suy từ đầu vào" thành hằng của thuật toán, và cách suy
    /// cũ hoá ra đang gánh thêm việc ép tổng độ dài khớp chính xác. Bộ vector
    /// tuân thủ (`them mot byte thua`) bắt được — và nó bắt được vì hôm ấy
    /// vector đã được đưa vào `cargo test`. Ghim thêm ở tầng đơn vị để lần sau
    /// không phải chạy cả bộ kiểm định mới biết.
    #[test]
    fn chu_ky_thua_hay_thieu_mot_byte_deu_bi_choi() {
        let (s, k) = bo_ky();
        let msg = b"chuyen 1000 TCC";
        let sig = s.sign(&k.secret, msg).unwrap();
        assert!(
            s.verify(&k.public, msg, &sig).is_ok(),
            "chữ ký đúng phải qua"
        );

        let mut thua = sig.clone();
        thua.push(0);
        assert!(
            s.verify(&k.public, msg, &thua).is_err(),
            "thừa một byte vẫn qua — chữ ký DẺO"
        );

        assert!(
            s.verify(&k.public, msg, &sig[..sig.len() - 1]).is_err(),
            "thiếu một byte vẫn qua"
        );
    }
}
