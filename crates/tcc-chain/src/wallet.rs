//! Sinh khoá ví — **giống hệt ví web đang chạy**, không phải một cách khác.
//!
//! # Vì sao phải giống từng byte
//!
//! Người dùng đã có ví ở `network3.tcc-coin.com`. Nếu trình duyệt sinh khoá
//! theo cách riêng thì cùng một cụm từ khôi phục ra **hai địa chỉ khác nhau**,
//! và người dùng mất tiền theo cách khó hiểu nhất: gửi tới địa chỉ họ đọc được
//! trên màn hình này, rồi không thấy nó ở màn hình kia.
//!
//! Nên chuỗi dẫn xuất ở đây bám đúng
//! `dilithium3/src/models/crypto.rs::password_to_seed` của chuỗi TCC:
//!
//! ```text
//! chuỗi hạt giống (ASCII) → BLAKE3 derive_key("tcc_chain_2026_seed_v1", …)
//!                         → 32 byte → ML-DSA-65 KeyGen
//!                         → khoá công khai 1952 byte
//!                         → BLAKE3(khoá công khai) = địa chỉ 32 byte
//! ```
//!
//! # ⚠️ Hai cách dẫn xuất KHÁC NHAU, và lẫn chúng là chuỗi đứng im
//!
//! Cùng 32 byte ấy có hai đường vào:
//!
//! | Đường | Dùng ở đâu |
//! |---|---|
//! | **Băm chuỗi** — [`WalletSecret::from_seed_phrase`] | Ví web, `tcc-keygen`, cụm từ khôi phục |
//! | **Byte nguyên xi** — [`WalletSecret::from_raw_seed`] | `node.key_seed` trong tệp cấu hình nút |
//!
//! Đưa cùng một giá trị vào hai đường ấy ra **hai khoá khác nhau**. Đội chuỗi
//! đã trả giá cho đúng chỗ này ngày 30/07/2026: một nút ký phiếu bằng khoá
//! không phải khoá đã đăng ký, các nút khác lặng lẽ bỏ phiếu ấy, và mạng dừng
//! chốt khối **mà không in ra một dòng lỗi nào**.
//!
//! Vì thế hai đường ở đây là **hai hàm tên khác nhau**, không phải một hàm với
//! một cờ. Chọn nhầm thì đọc tên hàm là thấy; chọn nhầm một tham số `bool` thì
//! không.
//!
//! # Đây KHÔNG phải khoá ký gói TCC
//!
//! Gói TCC ký **lai** (Ed25519+ML-DSA) bằng `tcc-crypto`; ví ký **thuần**
//! ML-DSA bằng tệp này. Hai khoá tách riêng theo mục đích — và ở đây hai kiểu
//! dữ liệu nằm ở hai crate khác nhau, nên trình biên dịch giữ hộ ranh giới ấy.

use core::fmt;

use ml_dsa::{KeyInit as _, Keypair as _, MlDsa65, SigningKey};
use zeroize::{Zeroize as _, ZeroizeOnDrop};

use crate::Address;

/// Bối cảnh BLAKE3 KDF. **Đổi một ký tự là mọi ví cũ thành ví khác.**
///
/// Con số này bám theo chuỗi TCC, không phải do dự án này chọn.
pub const SEED_CONTEXT: &str = "tcc_chain_2026_seed_v1";

/// Độ dài hạt giống ML-DSA-65 theo FIPS 204.
pub const SEED_LEN: usize = 32;

/// Độ dài khoá công khai ML-DSA-65 đã mã hoá.
pub const PUBLIC_KEY_LEN: usize = 1952;

/// Khoá bí mật của ví: **đúng 32 byte hạt giống**, không phải khoá đã bung.
///
/// FIPS 204 cho phép giữ nguyên hạt giống và bung lại mỗi lần ký. Giữ 32 byte
/// thay vì 4032 byte có hai cái lợi thật: kho khoá hệ điều hành nhận gọn, và
/// vùng nhớ phải xoá nhỏ hơn 126 lần.
///
/// Không có `Clone`: một khoá bí mật nhân bản được là một khoá bí mật mà không
/// ai biết còn mấy bản đang sống.
#[derive(ZeroizeOnDrop)]
pub struct WalletSecret([u8; SEED_LEN]);

impl WalletSecret {
    /// Dẫn xuất từ **chuỗi hạt giống** — đường của ví web và của cụm từ khôi phục.
    ///
    /// Đây là đường bạn cần trong 99% trường hợp. Đường kia là
    /// [`Self::from_raw_seed`], và đọc ghi chú đầu tệp trước khi dùng nó.
    #[must_use]
    pub fn from_seed_phrase(phrase: &str) -> Self {
        Self(blake3::derive_key(SEED_CONTEXT, phrase.as_bytes()))
    }

    /// Nhận **32 byte nguyên xi** làm hạt giống, KHÔNG băm gì thêm.
    ///
    /// Chỉ đúng khi byte đã là hạt giống thật (`node.key_seed`, hoặc khoá vừa
    /// lấy ra từ kho khoá). Đưa một chuỗi người gõ vào đây là ra một ví khác
    /// hẳn — xem ghi chú đầu tệp.
    #[must_use]
    pub const fn from_raw_seed(seed: [u8; SEED_LEN]) -> Self {
        Self(seed)
    }

    /// Mượn byte hạt giống để cất vào kho khoá.
    ///
    /// Trả lát mượn chứ không trả bản sao: bên gọi không giữ được thứ sống lâu
    /// hơn `WalletSecret`, nên vùng cần-nhớ-xoá vẫn chỉ có một.
    #[must_use]
    pub const fn expose_seed(&self) -> &[u8; SEED_LEN] {
        &self.0
    }

    /// Khoá công khai ML-DSA-65 đã mã hoá, 1952 byte.
    #[must_use]
    pub fn public_key(&self) -> PublicKey {
        // `new` chứ không `new_from_slice`: bên trên là mảng ĐÚNG 32 byte nên
        // không có nhánh hỏng nào để xử lý. Dùng bản trả `Result` ở đây là tự
        // tạo ra một đường lỗi không bao giờ chạy — rồi phải `expect` nó.
        let sk = SigningKey::<MlDsa65>::new(&self.0.into());
        let mut bytes = sk.verifying_key().encode().to_vec();
        debug_assert_eq!(bytes.len(), PUBLIC_KEY_LEN);
        let out = PublicKey(bytes.clone());
        bytes.zeroize();
        out
    }

    /// Địa chỉ ví: `BLAKE3(khoá công khai)`, hiện ra `0x` + 64 hex.
    #[must_use]
    pub fn address(&self) -> Address {
        self.public_key().address()
    }
}

/// Không in khoá ra nhật ký, dù ai đó gọi `{:?}` trên cả một cấu trúc lớn.
impl fmt::Debug for WalletSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WalletSecret({SEED_LEN} byte, đã giấu)")
    }
}

/// Khoá công khai ML-DSA-65 đã mã hoá.
#[derive(Clone, PartialEq, Eq)]
pub struct PublicKey(Vec<u8>);

impl PublicKey {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// `Address = BLAKE3(khoá công khai)` — quyết định D3 của chuỗi.
    #[must_use]
    pub fn address(&self) -> Address {
        Address(*blake3::hash(&self.0).as_bytes())
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PublicKey({} byte)", self.0.len())
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay"
)]
mod kiem_thu {
    use super::*;

    /// # MỐC NGOÀI — lấy từ chương trình của ĐỘI CHUỖI, không phải của ta
    ///
    /// ```text
    /// $ tcc-keygen address --seed hello
    /// address_hex = 0x6c1be5…
    /// ```
    ///
    /// `tcc-keygen` nằm trong kho `tcc-chain/v4`, dựng từ mã của họ. Nếu hai
    /// con số này lệch nhau thì trình duyệt đang sinh ví trên một mạng khác.
    ///
    /// Phép thử tự so mã của mình với chính nó thì không chứng minh được gì —
    /// bài học đã trả giá khi tôi đọc nhầm tệp nguồn của chuỗi (`docs/vi-thiet-ke.md` §7.3).
    const NEO: &[(&str, &str)] = &[
        (
            "hello",
            "0x6c1be53fb4c791728c9b8739636ef4ed2345aa81cc7764a81ca68b4981bcac77",
        ),
        // Cụm từ khôi phục 24 chữ "abandon…art" ra entropy toàn số 0; đây là
        // chuỗi hạt giống mà ví web sinh ra từ nó. Công khai được, vì ai cũng
        // dựng lại được từ tiêu chuẩn BIP39.
        (
            "0000000000000000000000000000000000000000000000000000000000000000",
            "0x11b22b300e195c44c910d71cdb1515c4617e852393cde5e80c860906b8a2d549",
        ),
    ];

    #[test]
    fn dia_chi_khop_chuong_trinh_cua_doi_chuoi() {
        for (hat, mong_doi) in NEO {
            let v = WalletSecret::from_seed_phrase(hat);
            assert_eq!(
                v.address().to_string(),
                *mong_doi,
                "hạt giống {hat:?} ra địa chỉ khác tcc-keygen"
            );
        }
    }

    /// Khoá công khai ML-DSA-65 là 1952 byte — không phải 1984 như khoá LAI.
    ///
    /// 1984 = 32 (Ed25519) + 1952. Lẫn hai con số là lẫn hai bộ ký.
    #[test]
    fn khoa_cong_khai_dung_1952_byte() {
        let v = WalletSecret::from_seed_phrase("hello");
        assert_eq!(v.public_key().as_bytes().len(), PUBLIC_KEY_LEN);
    }

    /// Hằng số của ta phải khớp hằng số của crate `ml-dsa`.
    #[test]
    fn hat_giong_dung_kich_thuoc_cua_crate() {
        use ml_dsa::KeySizeUser as _;
        assert_eq!(SigningKey::<MlDsa65>::key_size(), SEED_LEN);
    }

    /// **Đây là cái bẫy làm chuỗi dừng chốt khối ngày 30/07/2026.**
    ///
    /// Cùng một giá trị, hai đường dẫn xuất, hai ví. Phép thử này tồn tại để
    /// ngày ai đó "dọn dẹp" hai hàm thành một thì nó đỏ.
    #[test]
    fn bam_chuoi_va_byte_nguyen_xi_ra_hai_vi_khac_nhau() {
        let chuoi = "0000000000000000000000000000000000000000000000000000000000000000";
        let mut byte = [0u8; SEED_LEN];
        hex::decode_to_slice(chuoi, &mut byte).unwrap();

        let a = WalletSecret::from_seed_phrase(chuoi);
        let b = WalletSecret::from_raw_seed(byte);
        assert_ne!(
            a.address(),
            b.address(),
            "hai đường dẫn xuất ra cùng một ví — cái bẫy 30/07/2026 đã bị xoá mất"
        );
    }

    /// Dẫn xuất phải TẤT ĐỊNH: cùng chuỗi, cùng ví, mọi lúc.
    #[test]
    fn cung_hat_giong_ra_cung_vi() {
        let a = WalletSecret::from_seed_phrase("cùng một câu");
        let b = WalletSecret::from_seed_phrase("cùng một câu");
        assert_eq!(a.address(), b.address());
        assert_eq!(a.public_key(), b.public_key());
    }

    /// Đổi một ký tự là đổi hẳn ví — kể cả ký tự trắng ở cuối.
    #[test]
    fn lech_mot_ky_tu_la_vi_khac() {
        let a = WalletSecret::from_seed_phrase("hello");
        let b = WalletSecret::from_seed_phrase("hello ");
        assert_ne!(a.address(), b.address());
    }

    /// Chuỗi hex CHỮ HOA ra ví khác chuỗi hex chữ thường.
    ///
    /// Ví web sinh hex bằng `toString(16)` — **chữ thường**. Ai đó "chuẩn hoá"
    /// cụm từ khôi phục thành chữ hoa là người dùng mất ví, nên chỗ này được
    /// ghim bằng phép thử chứ không bằng lời dặn.
    #[test]
    fn hex_chu_hoa_ra_vi_khac() {
        let thuong = "abcdef0000000000000000000000000000000000000000000000000000000000";
        let hoa = thuong.to_uppercase();
        assert_ne!(
            WalletSecret::from_seed_phrase(thuong).address(),
            WalletSecret::from_seed_phrase(&hoa).address()
        );
    }

    /// `Debug` không được để lọt byte khoá ra nhật ký.
    #[test]
    fn debug_khong_lo_khoa() {
        let v = WalletSecret::from_seed_phrase("bí mật");
        let s = format!("{v:?}");
        assert!(!s.contains("0x"), "{s}");
        for b in v.expose_seed() {
            assert!(!s.contains(&format!("{b}")) || s.contains("32"), "{s}");
        }
    }

    /// Bối cảnh KDF là hằng số neo với chuỗi — đổi nó là mọi ví cũ biến mất.
    #[test]
    fn boi_canh_kdf_khong_duoc_doi() {
        assert_eq!(SEED_CONTEXT, "tcc_chain_2026_seed_v1");
    }
}
