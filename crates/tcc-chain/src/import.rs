//! Nhập ví cũ từ ví web — **đọc một lần, rồi cất lại tử tế**.
//!
//! Sau cờ `import-web-wallet`. Giá: **14 crate** thêm vào cả workspace
//! (103 → 117) — `aes-gcm`, `pbkdf2` và họ hàng. Đắt cho một việc người dùng
//! làm đúng một lần trong đời, nên nó nằm sau cờ chứ không nằm trong bản dựng
//! của mọi người.
//!
//! # Vì sao phải cài lại thứ đã có
//!
//! Luật dự án nói **không viết lại phần mật mã đã có** — và ở đây không viết
//! lại: `pbkdf2` và `aes-gcm` là crate của RustCrypto, ta chỉ gọi. Thứ phải
//! viết là **bộ đọc một định dạng**, và không có đường nào khác: khoá của người
//! dùng đang nằm trong `localStorage` của trang web, mã hoá bằng lược đồ của
//! trang web. Không đọc được nó thì người đang có ví bị bỏ lại.
//!
//! ```text
//! PIN → PBKDF2-SHA256 100.000 vòng (muối 16 byte) → khoá AES-256
//!     → AES-GCM (IV 12 byte) → 32 byte hạt giống ML-DSA
//! ```
//!
//! # ⚠️ Nhập KHÔNG làm bản cũ biến mất
//!
//! Ví web vẫn giữ nguyên bản của nó trong `localStorage`, vẫn khoá bằng PIN,
//! vẫn yếu đúng như trước. Trình duyệt **không** đụng vào dữ liệu ấy — xoá hộ
//! người dùng thứ họ chưa bảo xoá là một cách hỏng riêng, và tệ hơn nếu bản
//! nhập sang có vấn đề.
//!
//! Nên giao diện **phải nói ra** điều đó sau khi nhập xong: *"ví vẫn còn một
//! bản ở trang web, vẫn khoá bằng PIN"*. Người dùng tưởng mình đã dọn sạch
//! trong khi bản yếu vẫn nằm đó là tình huống xấu nhất — họ mất cảnh giác mà
//! rủi ro không giảm.
//!
//! # Kiểm địa chỉ TRƯỚC khi tin
//!
//! Giải mã xong không có nghĩa là đúng ví. Hạt giống lấy ra được dẫn xuất lại
//! thành địa chỉ và **so với địa chỉ ghi trong bản ghi**; lệch là từ chối. Cùng
//! nguyên tắc với màn xác nhận giao dịch: kiểm trước, tin sau.

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead as _, KeyInit as _},
};
use base64ct::{Base64, Encoding as _};
use serde::Deserialize;
use zeroize::{Zeroize as _, Zeroizing};

use crate::wallet::{SEED_LEN, WalletSecret};

/// Số vòng PBKDF2 của ví web. **Không phải con số ta chọn** — phải khớp đúng.
pub const PBKDF2_ITERATIONS: u32 = 100_000;

/// Phiên bản lược đồ duy nhất đọc được.
pub const SCHEMA_VERSION: u32 = 4;

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("không phải JSON hợp lệ: {0}")]
    Json(String),
    #[error("lược đồ phiên bản {0}, chỉ đọc được phiên bản {SCHEMA_VERSION}")]
    UnsupportedSchema(u32),
    #[error("trường {0} không phải base64 hợp lệ")]
    Base64(&'static str),
    #[error("muối phải 16 byte, IV phải 12 byte")]
    BadParameters,
    /// AES-GCM không phân biệt được "sai PIN" với "dữ liệu hỏng" — thẻ xác thực
    /// hỏng theo cùng một cách. Gộp làm một là trung thực; tách ra là bịa.
    #[error("sai PIN, hoặc dữ liệu đã hỏng")]
    WrongPin,
    /// Ví sinh trước lần chuyển sang ML-DSA giữ khoá đã bung 4032 byte.
    #[error("khoá dài {0} byte, không phải {SEED_LEN} — ví này cũ hơn bản ML-DSA")]
    UnsupportedKeyFormat(usize),
    /// Giải mã được nhưng ra một ví khác — bản ghi hỏng, hoặc đã bị sửa.
    #[error("khoá giải ra KHÔNG khớp địa chỉ ghi trong bản ghi")]
    AddressMismatch,
}

/// Một bản ghi ví, **chưa mở khoá**.
///
/// Liệt kê được mà không cần PIN là có chủ ý: người dùng thấy mình có mấy ví,
/// nhãn gì, địa chỉ nào, rồi mới quyết định gõ PIN cho cái nào.
#[derive(Debug, Deserialize)]
pub struct WebWallet {
    pub address: String,
    #[serde(default)]
    pub label: String,
    #[serde(rename = "encryptedPrivkey")]
    encrypted_privkey: String,
    salt: String,
    iv: String,
    #[serde(rename = "encryptedSeed", default)]
    encrypted_seed: Option<String>,
    #[serde(rename = "seedSalt", default)]
    seed_salt: Option<String>,
    #[serde(rename = "seedIv", default)]
    seed_iv: Option<String>,
}

#[derive(Deserialize)]
struct Export {
    v: u32,
    wallets: std::collections::BTreeMap<String, WebWallet>,
}

/// Ví đã mở khoá, sẵn sàng cất vào kho khoá hệ điều hành.
pub struct ImportedWallet {
    pub secret: WalletSecret,
    /// Cụm từ khôi phục, nếu bản ghi có giữ.
    ///
    /// `Zeroizing` chứ không `String` trần: đây là thứ khôi phục được cả ví,
    /// nên nó phải biến mất khỏi bộ nhớ đúng lúc bên gọi thả nó ra.
    pub mnemonic: Option<Zeroizing<String>>,
}

/// Giấu cả khoá lẫn cụm từ. Chỉ nói **có** cụm từ hay không — thứ ấy vô hại
/// và là thứ duy nhất đáng thấy trong nhật ký.
impl core::fmt::Debug for ImportedWallet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ImportedWallet({}, cụm từ: {})",
            self.secret.address(),
            if self.mnemonic.is_some() {
                "có"
            } else {
                "không"
            }
        )
    }
}

/// Đọc bản kết xuất `tcc_wallets_v4`, **chưa giải mã gì cả**.
///
/// # Errors
/// JSON hỏng, hoặc lược đồ không phải phiên bản 4.
pub fn read_export(json: &str) -> Result<Vec<WebWallet>, ImportError> {
    let x: Export = serde_json::from_str(json).map_err(|e| ImportError::Json(e.to_string()))?;
    if x.v != SCHEMA_VERSION {
        return Err(ImportError::UnsupportedSchema(x.v));
    }
    Ok(x.wallets.into_values().collect())
}

impl WebWallet {
    /// Bản ghi có giữ cụm từ khôi phục không — biết được mà **không cần PIN**.
    #[must_use]
    pub const fn has_mnemonic(&self) -> bool {
        self.encrypted_seed.is_some()
    }

    /// Mở khoá bằng PIN của ví web.
    ///
    /// # Errors
    /// Sai PIN, tham số hỏng, khoá không phải định dạng ML-DSA, hoặc khoá giải
    /// ra không khớp địa chỉ ghi trong bản ghi.
    pub fn unlock(&self, pin: &str) -> Result<ImportedWallet, ImportError> {
        let mut ro = giai_ma(
            &self.encrypted_privkey,
            &self.salt,
            &self.iv,
            pin,
            "privkey",
        )?;
        if ro.len() != SEED_LEN {
            let n = ro.len();
            ro.zeroize();
            return Err(ImportError::UnsupportedKeyFormat(n));
        }
        let mut hat = [0u8; SEED_LEN];
        hat.copy_from_slice(&ro);
        ro.zeroize();
        let secret = WalletSecret::from_raw_seed(hat);

        // Giải mã được KHÔNG có nghĩa là đúng ví. Dẫn xuất lại và so.
        if secret.address().to_string() != self.address {
            return Err(ImportError::AddressMismatch);
        }

        let mnemonic = match (&self.encrypted_seed, &self.seed_salt, &self.seed_iv) {
            (Some(c), Some(s), Some(i)) => {
                let mut b = giai_ma(c, s, i, pin, "seed")?;
                let ra = String::from_utf8(b.clone()).map_err(|_| ImportError::WrongPin)?;
                b.zeroize();
                Some(Zeroizing::new(ra))
            }
            // Thiếu một trong ba thì coi như không có, chứ không báo lỗi: ví
            // sinh bằng bản cũ hơn lược đồ v5 vốn không lưu cụm từ, và chặn
            // người dùng nhập ví chỉ vì thiếu thứ họ chưa từng có là vô lý.
            _ => None,
        };
        Ok(ImportedWallet { secret, mnemonic })
    }
}

fn giai_ma(
    cipher_b64: &str,
    salt_b64: &str,
    iv_b64: &str,
    pin: &str,
    ten: &'static str,
) -> Result<Vec<u8>, ImportError> {
    let cipher = Base64::decode_vec(cipher_b64).map_err(|_| ImportError::Base64(ten))?;
    let salt = Base64::decode_vec(salt_b64).map_err(|_| ImportError::Base64("salt"))?;
    let iv = Base64::decode_vec(iv_b64).map_err(|_| ImportError::Base64("iv"))?;
    if salt.len() != 16 || iv.len() != 12 {
        return Err(ImportError::BadParameters);
    }

    let mut khoa = Zeroizing::new([0u8; 32]);
    pbkdf2::pbkdf2_hmac::<sha2::Sha256>(pin.as_bytes(), &salt, PBKDF2_ITERATIONS, &mut *khoa);

    // `new` không hỏng với 32 byte; `Aes256Gcm::new` nhận mảng đúng cỡ nên
    // không có nhánh lỗi nào để bịa ra.
    let bo = Aes256Gcm::new((&*khoa).into());
    bo.decrypt(Nonce::from_slice(&iv), cipher.as_slice())
        .map_err(|_| ImportError::WrongPin)
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay"
)]
mod kiem_thu {
    use super::*;

    /// Bản ghi ví web **thật**, dựng bằng đúng WebCrypto mà trang web gọi
    /// (`node:crypto`, cùng tham số PBKDF2/AES-GCM), không phải do tôi bịa ra
    /// từ đọc mã.
    ///
    /// Muối và IV trong tệp mẫu **cố định** để bản dựng lặp lại được; bản ghi
    /// thật dùng `crypto.getRandomValues`. Dùng lại muối/IV chỉ chấp nhận được
    /// trong một tệp mẫu công khai không giữ tiền của ai.
    const MAU: &str = include_str!("../data/vi-web-mau.json");
    const PIN: &str = "matkhau-thu-nghiem";
    /// Neo tới `tcc-keygen` của đội chuỗi — xem `wallet.rs`.
    const DIA_CHI: &str = "0x11b22b300e195c44c910d71cdb1515c4617e852393cde5e80c860906b8a2d549";

    fn mot() -> WebWallet {
        let mut v = read_export(MAU).unwrap();
        assert_eq!(v.len(), 1);
        v.remove(0)
    }

    /// Đường đầy đủ: JSON của ví web → PIN → hạt giống → **đúng địa chỉ ấy**.
    #[test]
    fn nhap_ra_dung_vi_cua_nguoi_dung() {
        let ra = mot().unlock(PIN).unwrap();
        assert_eq!(ra.secret.address().to_string(), DIA_CHI);
    }

    /// Cụm từ khôi phục cũng mang sang được — và nó phải ra đúng ví ấy.
    #[test]
    fn cum_tu_khoi_phuc_mang_sang_duoc_va_khop() {
        let ra = mot().unlock(PIN).unwrap();
        let cum_tu = ra.mnemonic.expect("bản ghi mẫu có giữ cụm từ");
        assert_eq!(cum_tu.split(' ').count(), 24);
        assert_eq!(
            WalletSecret::from_mnemonic(&cum_tu)
                .unwrap()
                .address()
                .to_string(),
            DIA_CHI,
            "cụm từ giải ra không mở đúng ví đã nhập"
        );
    }

    /// Liệt kê được mà KHÔNG cần PIN — người dùng chọn ví trước, gõ PIN sau.
    #[test]
    fn liet_ke_duoc_khong_can_pin() {
        let v = mot();
        assert_eq!(v.address, DIA_CHI);
        assert_eq!(v.label, "Ví thử nghiệm");
        assert!(v.has_mnemonic());
    }

    #[test]
    fn sai_pin_thi_tu_choi() {
        let loi = mot().unlock("sai-pin").unwrap_err();
        assert!(matches!(loi, ImportError::WrongPin), "{loi}");
    }

    /// PIN đúng nhưng địa chỉ trong bản ghi bị sửa → **từ chối**.
    ///
    /// Không có tấn công cụ thể nào ở đây; đây là phép kiểm cho chính ta. Nếu
    /// hàm dẫn xuất trong `wallet.rs` trôi đi một ngày nào đó, phép thử này đỏ
    /// trước khi có ai nhập nhầm ví.
    #[test]
    fn dia_chi_bi_sua_thi_tu_choi() {
        let mut v = mot();
        v.address = "0x0000000000000000000000000000000000000000000000000000000000000000".to_owned();
        assert!(matches!(
            v.unlock(PIN).unwrap_err(),
            ImportError::AddressMismatch
        ));
    }

    /// Một byte lật trong bản mã → thẻ AES-GCM hỏng → từ chối.
    #[test]
    fn du_lieu_bi_sua_mot_byte_thi_tu_choi() {
        let mut v = mot();
        let mut b = Base64::decode_vec(&v.encrypted_privkey).unwrap();
        b[0] ^= 1;
        v.encrypted_privkey = Base64::encode_string(&b);
        assert!(matches!(v.unlock(PIN).unwrap_err(), ImportError::WrongPin));
    }

    #[test]
    fn luoc_do_khac_phien_ban_thi_tu_choi() {
        let sai = MAU.replacen("\"v\": 4", "\"v\": 3", 1);
        assert!(matches!(
            read_export(&sai).unwrap_err(),
            ImportError::UnsupportedSchema(3)
        ));
    }

    #[test]
    fn muoi_sai_kich_thuoc_thi_tu_choi() {
        let mut v = mot();
        v.salt = Base64::encode_string(&[0u8; 8]);
        assert!(matches!(
            v.unlock(PIN).unwrap_err(),
            ImportError::BadParameters
        ));
    }

    #[test]
    fn json_hong_thi_bao_loi_chu_khong_hoang_loan() {
        assert!(matches!(read_export("{"), Err(ImportError::Json(_))));
        assert!(matches!(read_export(""), Err(ImportError::Json(_))));
    }

    /// **Kiểm đột biến tìm ra chỗ này.** Bỏ phép kiểm độ dài khoá thì không
    /// phép thử nào đỏ — mà bỏ nó là `copy_from_slice` HOẢNG LOẠN thay vì báo
    /// lỗi sạch, đúng với ví sinh trước lần chuyển sang ML-DSA.
    ///
    /// Bản ghi mẫu giữ khoá đã bung 4032 byte, dựng bằng cùng WebCrypto.
    #[test]
    fn vi_truoc_ban_ml_dsa_bao_loi_chu_khong_hoang_loan() {
        const MAU_CU: &str = include_str!("../data/vi-web-mau-khoa-cu.json");
        let v = read_export(MAU_CU).unwrap().remove(0);
        let loi = v.unlock(PIN).unwrap_err();
        assert!(
            matches!(loi, ImportError::UnsupportedKeyFormat(4032)),
            "{loi}"
        );
        // Câu lỗi phải nói được vì sao, không chỉ nói "hỏng".
        assert!(loi.to_string().contains("ML-DSA"), "{loi}");
    }

    /// Số vòng PBKDF2 là con số của ví web, không phải của ta. Đổi nó là không
    /// ví nào nhập được nữa.
    #[test]
    fn so_vong_pbkdf2_khop_vi_web() {
        assert_eq!(PBKDF2_ITERATIONS, 100_000);
    }
}
