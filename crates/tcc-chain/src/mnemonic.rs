//! Cụm từ khôi phục 24 chữ — **từ điển BIP39, dẫn xuất riêng của TCC**.
//!
//! # Đọc dòng này trước khi hứa gì với người dùng
//!
//! Đây **không phải** BIP39 chuẩn. BIP39 chuẩn đi
//! `cụm từ → PBKDF2-2048 → hạt giống 64 byte`. TCC lấy **thẳng entropy**, viết
//! ra hex chữ thường, rồi đưa chuỗi hex ấy vào BLAKE3 KDF:
//!
//! ```text
//! 24 chữ → entropy 32 byte → hex CHỮ THƯỜNG (64 ký tự)
//!        → WalletSecret::from_seed_phrase → ví
//! ```
//!
//! Hệ quả phải nói thẳng với người dùng: **không ví nào khác khôi phục được 24
//! chữ này**, kể cả ví ghi "hỗ trợ BIP39". Chỉ có từ điển là chung.
//!
//! # Từ điển được ghim bằng băm, không bằng lòng tin
//!
//! `data/bip39-english.txt` lấy ra từ chính ví web đang chạy, và băm SHA-256
//! của nó **khớp `english.txt` công bố trong kho `bitcoin/bips`**:
//! `2f5eed53a4727b4bf8880d8f3f199efc90e58503646d9ff8eff3a2ed3b24dbda`.
//!
//! Việc kiểm ấy đáng làm: một từ điển bị đổi vài từ là một cửa hậu trộm ví hoàn
//! toàn im lặng — cụm từ vẫn "hợp lệ", chỉ ra một ví khác. Phép thử
//! [`kiem_thu::tu_dien_khop_ban_cong_bo`] chạy lại phép băm ấy mỗi lần dựng.
//!
//! # KHÔNG có đường lùi khi tổng kiểm sai
//!
//! Ví web, khi 24 chữ sai tổng kiểm, mời người dùng *"Try as raw seed?"* — và
//! coi cả cụm từ như một chuỗi hạt giống thô. Nó có chắn: màn xác nhận hiện địa
//! chỉ **và số dư** trước khi đi tiếp, nên gõ nhầm thì thấy số dư 0.
//!
//! Trình duyệt này **không làm đường lùi ấy**. Tổng kiểm sinh ra đúng để bắt
//! một chữ gõ nhầm; biến nó thành lời gợi ý là vứt đi thứ duy nhất phân biệt
//! *"bạn gõ nhầm"* với *"đây là ví khác"*. Chuỗi hạt giống thô vẫn nhập được,
//! nhưng phải là một lựa chọn người dùng **tự bấm**, không phải một đề nghị hiện
//! ra đúng lúc họ đang bối rối.

use sha2::{Digest as _, Sha256};

use crate::wallet::WalletSecret;

/// Từ điển tiếng Anh BIP39, 2048 từ.
const TU_DIEN: &str = include_str!("../data/bip39-english.txt");

/// Số chữ TCC dùng. Ví web chỉ sinh 24 chữ (256 bit entropy), nên nhận đúng
/// một độ dài là bớt một nhánh mã mà không bớt thứ gì người dùng có thật.
pub const WORD_COUNT: usize = 24;

/// Số byte entropy tương ứng 24 chữ.
pub const ENTROPY_LEN: usize = 32;

/// Mỗi từ mang 11 bit chỉ số (2^11 = 2048).
const BITS_PER_WORD: usize = 11;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MnemonicError {
    #[error("cần đúng {WORD_COUNT} chữ, nhận được {0}")]
    WordCount(usize),
    /// Chỉ trả **vị trí**, không trả chữ.
    ///
    /// Người dùng cần biết gõ nhầm chữ thứ mấy — thứ ấy giúp họ sửa. Nhật ký
    /// thì không cần biết chữ ấy là gì: một mẩu cụm từ khôi phục lọt ra tệp
    /// nhật ký vẫn là một mẩu cụm từ khôi phục lọt ra ngoài.
    #[error("chữ thứ {0} không có trong từ điển")]
    UnknownWord(usize),
    #[error("tổng kiểm sai — nhiều khả năng có một chữ gõ nhầm")]
    Checksum,
}

/// Tra từ → chỉ số 11 bit.
///
/// Đếm bằng `u32` ngay từ đầu chứ không đổi kiểu về sau: chỉ số từ điển luôn
/// nhỏ hơn 2048, nhưng "luôn" mà phải viết ra thành một phép đổi kiểu có thể
/// hỏng thì lại thành một nhánh lỗi không bao giờ chạy — và một `expect` nữa
/// trong mã đụng tới khoá.
fn tim_chu(chu: &str) -> Option<u32> {
    TU_DIEN
        .lines()
        .zip(0u32..)
        .find(|(t, _)| *t == chu)
        .map(|(_, i)| i)
}

/// Chuẩn hoá y như ví web: thường hoá, gộp mọi khoảng trắng thành một dấu cách.
///
/// Phải khớp, không được "chặt hơn cho an toàn": một cụm từ ví web nhận mà ở
/// đây từ chối thì người dùng tin rằng mình gõ sai, và đi gõ lại thứ vốn đúng.
fn chuan_hoa(cum_tu: &str) -> Vec<String> {
    cum_tu
        .split_whitespace()
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
}

/// 24 chữ → 32 byte entropy, **có kiểm tổng kiểm**.
///
/// # Errors
/// Sai số chữ, có chữ lạ, hoặc tổng kiểm không khớp. Không có nhánh nào "gần
/// đúng thì cho qua" — xem ghi chú đầu tệp.
pub fn entropy_from_mnemonic(cum_tu: &str) -> Result<[u8; ENTROPY_LEN], MnemonicError> {
    let chu = chuan_hoa(cum_tu);
    if chu.len() != WORD_COUNT {
        return Err(MnemonicError::WordCount(chu.len()));
    }

    // 24 × 11 = 264 bit = 256 bit entropy + 8 bit tổng kiểm. Chia hết cho 8,
    // nên không có bit lẻ nào phải xử lý riêng.
    let mut entropy = [0u8; ENTROPY_LEN];
    let mut tong_kiem = 0u8;
    let mut bit = 0u32;
    let mut so_bit = 0usize;
    let mut vi_tri = 0usize;
    for (i, c) in chu.iter().enumerate() {
        bit = (bit << BITS_PER_WORD) | tim_chu(c).ok_or(MnemonicError::UnknownWord(i + 1))?;
        so_bit += BITS_PER_WORD;
        while so_bit >= 8 {
            so_bit -= 8;
            // `to_le_bytes()[0]` lấy byte thấp, không phải phép đổi kiểu có thể
            // cụt — nên không sinh ra nhánh lỗi giả nào.
            let b = ((bit >> so_bit) & 0xff).to_le_bytes()[0];
            if vi_tri < ENTROPY_LEN {
                entropy[vi_tri] = b;
            } else {
                tong_kiem = b;
            }
            vi_tri += 1;
        }
    }
    debug_assert_eq!((so_bit, vi_tri), (0, ENTROPY_LEN + 1));

    if tong_kiem != Sha256::digest(entropy)[0] {
        return Err(MnemonicError::Checksum);
    }
    Ok(entropy)
}

/// 32 byte entropy → 24 chữ. Dùng khi **tạo ví mới**.
#[must_use]
pub fn mnemonic_from_entropy(entropy: &[u8; ENTROPY_LEN]) -> String {
    let tong_kiem = Sha256::digest(entropy)[0];
    let mut bit = 0u32;
    let mut so_bit = 0usize;
    let mut chu = Vec::with_capacity(WORD_COUNT);
    for b in entropy.iter().chain(core::iter::once(&tong_kiem)) {
        bit = (bit << 8) | u32::from(*b);
        so_bit += 8;
        if so_bit >= BITS_PER_WORD {
            so_bit -= BITS_PER_WORD;
            let chi_so = ((bit >> so_bit) & 0x7ff) as usize;
            // `flat_map` thay cho `expect`: chỉ số luôn < 2048 nên `nth` luôn
            // có, nhưng viết thế này thì không cần nói "luôn" ở đâu cả.
            chu.extend(TU_DIEN.lines().nth(chi_so));
        }
    }
    debug_assert_eq!(chu.len(), WORD_COUNT);
    chu.join(" ")
}

impl WalletSecret {
    /// Đường đầy đủ: **24 chữ → ví**, đúng như ví web làm.
    ///
    /// Chỗ dễ mất ví nhất nằm ở một dòng: hex phải là **chữ thường**. Ví web
    /// sinh hex bằng `toString(16)`; viết hoa là ra một ví khác hẳn.
    ///
    /// # Errors
    /// Cụm từ không hợp lệ — xem [`MnemonicError`].
    pub fn from_mnemonic(cum_tu: &str) -> Result<Self, MnemonicError> {
        let entropy = entropy_from_mnemonic(cum_tu)?;
        // `{:02x}` là chữ thường. KHÔNG đổi sang `{:02X}`.
        let mut hex = String::with_capacity(ENTROPY_LEN * 2);
        for b in entropy {
            hex.push(chu_hex(b >> 4));
            hex.push(chu_hex(b & 0x0f));
        }
        Ok(Self::from_seed_phrase(&hex))
    }
}

/// Một nửa byte → ký tự hex **CHỮ THƯỜNG**.
///
/// Viết tay thay vì `format!("{:02x}")` vì `write!` vào `String` trả `Result`
/// không bao giờ hỏng — và một `expect` trong đường sinh khoá là một `expect`
/// quá nhiều.
const fn chu_hex(nua: u8) -> char {
    match nua {
        0..=9 => (b'0' + nua) as char,
        _ => (b'a' + nua - 10) as char,
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

    /// # MỐC NGOÀI — sinh bằng `@scure/bip39@1.3.0`, thư viện của CHÍNH ví web
    ///
    /// Không phải tôi tự nghĩ ra: chạy thư viện đóng gói trong
    /// `web-login/wallet/public/vendor/bip39.js` rồi chép kết quả sang. Bốn cặp
    /// đầu cũng đúng bằng vector thử nghiệm công bố của BIP39.
    const NEO_CUM_TU: &[(&str, &str)] = &[
        (
            "0000000000000000000000000000000000000000000000000000000000000000",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art",
        ),
        (
            "7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f",
            "legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth title",
        ),
        (
            "8080808080808080808080808080808080808080808080808080808080808080",
            "letter advice cage absurd amount doctor acoustic avoid letter advice cage absurd amount doctor acoustic avoid letter advice cage absurd amount doctor acoustic bless",
        ),
        (
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo vote",
        ),
        (
            "68a79eaca2324873eacc50cb9c6eca8cc68ea5d936f98787c60c7ebc74e6ce7c",
            "hamster diagram private dutch cause delay private meat slide toddler razor book happy fancy gospel tennis maple dilemma loan word shrug inflict delay length",
        ),
    ];

    fn byte(hex: &str) -> [u8; ENTROPY_LEN] {
        let mut ra = [0u8; ENTROPY_LEN];
        hex::decode_to_slice(hex, &mut ra).unwrap();
        ra
    }

    #[test]
    fn entropy_ra_cum_tu_khop_thu_vien_cua_vi_web() {
        for (hex, cum_tu) in NEO_CUM_TU {
            assert_eq!(mnemonic_from_entropy(&byte(hex)), *cum_tu, "entropy {hex}");
        }
    }

    #[test]
    fn cum_tu_ra_entropy_khop_thu_vien_cua_vi_web() {
        for (hex, cum_tu) in NEO_CUM_TU {
            assert_eq!(
                entropy_from_mnemonic(cum_tu).unwrap(),
                byte(hex),
                "{cum_tu}"
            );
        }
    }

    /// # MỐC NGOÀI THỨ HAI — nối tới `tcc-keygen` của ĐỘI CHUỖI
    ///
    /// Đây là phép thử đắt giá nhất tệp này: **24 chữ → địa chỉ**, đi qua cả
    /// hai bước, và mỗi bước neo vào một chương trình khác nhau. Hai chương
    /// trình ấy không biết gì về nhau và cũng không biết gì về mã ở đây.
    #[test]
    fn cum_tu_ra_dia_chi_khop_chuong_trinh_cua_doi_chuoi() {
        let neo = [
            (
                NEO_CUM_TU[0].1,
                "0x11b22b300e195c44c910d71cdb1515c4617e852393cde5e80c860906b8a2d549",
            ),
            (
                NEO_CUM_TU[3].1,
                "0xbd59738a794987c1ae931944ab3dc536f01bcb811e34c9eca8bd2f3ea40c6e37",
            ),
            (
                NEO_CUM_TU[4].1,
                "0x8001d9c7d66bfeb823e09553fba551d587488c19b33e022144d0eb15350faeb3",
            ),
        ];
        for (cum_tu, dia_chi) in neo {
            let v = WalletSecret::from_mnemonic(cum_tu).unwrap();
            assert_eq!(v.address().to_string(), dia_chi, "{cum_tu}");
        }
    }

    /// Từ điển phải là bản công bố, không phải bản ai đó sửa vài từ.
    ///
    /// Một từ điển bị đổi là cửa hậu trộm ví hoàn toàn im lặng.
    #[test]
    fn tu_dien_khop_ban_cong_bo() {
        let bam = Sha256::digest(TU_DIEN.as_bytes());
        assert_eq!(
            hex::encode(bam),
            "2f5eed53a4727b4bf8880d8f3f199efc90e58503646d9ff8eff3a2ed3b24dbda",
            "từ điển BIP39 KHÔNG khớp bản công bố trong kho bitcoin/bips"
        );
        assert_eq!(TU_DIEN.lines().count(), 2048);
    }

    /// Từ điển phải xếp theo thứ tự từ điển — BIP39 đòi thế, và mã ở đây tra
    /// bằng vị trí dòng nên một dòng xê dịch là mọi ví xê dịch.
    #[test]
    fn tu_dien_dung_thu_tu_va_khong_trung() {
        let tu: Vec<&str> = TU_DIEN.lines().collect();
        for hai in tu.windows(2) {
            assert!(hai[0] < hai[1], "{} không đứng trước {}", hai[0], hai[1]);
        }

        // ⚠️ ĐÚNG 2048 từ, và đây không phải một con số cho đẹp.
        //
        // Mỗi từ mang 11 bit chỉ số, và cả phép gói bit dựa vào việc chỉ số
        // LUÔN nhỏ hơn 2^11: sau `bit << 11` thì 11 bit thấp bằng 0, nên `|` và
        // `^` cho kết quả y hệt. Thừa một từ là chỉ số 2048 tràn ra bit thứ 12,
        // hai phép ấy tách nhau, và cụm từ mã hoá ra sai — im lặng.
        //
        // Kiểm đột biến 26/08/2026 chỉ ra chỗ này: hai đột biến `|` thành `^`
        // trong `mnemonic.rs` SỐNG, và chúng sống vì tương đương về toán học.
        // Không phép thử nào giết được chúng, nên thứ phải ghim là BẤT BIẾN
        // khiến chúng tương đương — chính là con số này.
        assert_eq!(
            tu.len(),
            1 << BITS_PER_WORD,
            "từ điển không đúng 2^{BITS_PER_WORD} từ — chỉ số sẽ tràn khỏi 11 bit"
        );
    }

    /// **Gõ nhầm một chữ phải BỊ TỪ CHỐI, không được "gần đúng thì cho qua".**
    ///
    /// Đây là lý do tổng kiểm tồn tại, và là chỗ trình duyệt cố ý khác ví web.
    #[test]
    fn go_nham_mot_chu_thi_tu_choi() {
        // "art" → "arm": vẫn là từ hợp lệ, chỉ sai tổng kiểm. Đúng kiểu nhầm
        // mà người thật gõ ra.
        let sai = NEO_CUM_TU[0].1.replace(" art", " arm");
        assert_eq!(entropy_from_mnemonic(&sai), Err(MnemonicError::Checksum));
        assert!(WalletSecret::from_mnemonic(&sai).is_err());
    }

    /// Chữ lạ thì báo VỊ TRÍ, không báo chữ — chữ ấy là mẩu cụm từ khôi phục.
    #[test]
    fn chu_la_bao_vi_tri_chu_khong_bao_chu() {
        let sai = NEO_CUM_TU[0].1.replace(" art", " khongcotutrongtudien");
        let loi = entropy_from_mnemonic(&sai).unwrap_err();
        assert_eq!(loi, MnemonicError::UnknownWord(24));
        assert!(
            !loi.to_string().contains("khongcotutrongtudien"),
            "lỗi làm lọt một chữ của cụm từ khôi phục: {loi}"
        );
    }

    #[test]
    fn sai_so_chu_thi_tu_choi() {
        assert_eq!(
            entropy_from_mnemonic("abandon abandon abandon"),
            Err(MnemonicError::WordCount(3))
        );
        assert_eq!(entropy_from_mnemonic(""), Err(MnemonicError::WordCount(0)));
        let hai_lam = format!("{} zoo", NEO_CUM_TU[0].1);
        assert_eq!(
            entropy_from_mnemonic(&hai_lam),
            Err(MnemonicError::WordCount(25))
        );
    }

    /// Chuẩn hoá phải khớp ví web: CHỮ HOA và khoảng trắng thừa đều nhận.
    ///
    /// Chặt hơn ví web không phải "an toàn hơn" — nó làm người dùng tin mình
    /// gõ sai và đi gõ lại thứ vốn đúng.
    #[test]
    fn nhan_chu_hoa_va_khoang_trang_thua_nhu_vi_web() {
        let goc = NEO_CUM_TU[4].1;
        let ban = format!("  {}  ", goc.to_uppercase().replace(' ', "\n  "));
        assert_eq!(
            entropy_from_mnemonic(&ban).unwrap(),
            entropy_from_mnemonic(goc).unwrap()
        );
    }

    /// Vòng lại: entropy → chữ → entropy, trên dữ liệu tự sinh.
    #[test]
    fn vong_lai_khong_mat_gi() {
        for i in 0u8..32 {
            let mut e = [0u8; ENTROPY_LEN];
            for (j, o) in e.iter_mut().enumerate() {
                // Trải đủ kiểu byte, không cần ngẫu nhiên — phép thử phải lặp lại được.
                *o = i
                    .wrapping_mul(37)
                    .wrapping_add(u8::try_from(j).expect("j < ENTROPY_LEN").wrapping_mul(11));
            }
            let cum_tu = mnemonic_from_entropy(&e);
            assert_eq!(cum_tu.split(' ').count(), WORD_COUNT);
            assert_eq!(entropy_from_mnemonic(&cum_tu).unwrap(), e);
        }
    }

    /// Hai cụm từ khác nhau phải ra hai ví khác nhau — nghe hiển nhiên, nhưng
    /// đây là phép thử đỏ nếu ai đó lỡ bỏ qua entropy mà băm thẳng cụm từ.
    #[test]
    fn cum_tu_khac_nhau_ra_vi_khac_nhau() {
        let a = WalletSecret::from_mnemonic(NEO_CUM_TU[0].1).unwrap();
        let b = WalletSecret::from_mnemonic(NEO_CUM_TU[3].1).unwrap();
        assert_ne!(a.address(), b.address());
    }

    /// **Băm thẳng cụm từ KHÁC hẳn đường đúng.**
    ///
    /// Nếu ai đó "đơn giản hoá" `from_mnemonic` thành
    /// `from_seed_phrase(cụm_từ)` thì mọi ví lệch hết mà mã vẫn chạy trơn.
    #[test]
    fn bam_thang_cum_tu_ra_vi_khac_voi_duong_dung() {
        let cum_tu = NEO_CUM_TU[0].1;
        assert_ne!(
            WalletSecret::from_mnemonic(cum_tu).unwrap().address(),
            WalletSecret::from_seed_phrase(cum_tu).address()
        );
    }
}
