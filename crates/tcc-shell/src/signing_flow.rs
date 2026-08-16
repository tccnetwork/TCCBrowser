//! Đường ký một giao dịch — **kiểm trước, hiện, rồi mới ký**.
//!
//! # Thứ tự này được cưỡng chế bằng KIỂU DỮ LIỆU
//!
//! [`PendingTransaction`] có trường riêng tư và **không có hàm dựng công khai
//! nào ngoài [`review`]**. `review` giải mã gói tin, tự tính lại thông điệp ký,
//! so với thứ máy chủ đưa, và chỉ trả về `PendingTransaction` khi khớp.
//!
//! Hệ quả: **không viết ra được đoạn mã nào ký một giao dịch chưa qua kiểm.**
//! Không phải vì có ai nhớ luật, mà vì không có kiểu dữ liệu để cầm.
//!
//! Đây đúng là cách `tcc-capability` chặn một quyền chưa được cấp — trình biên
//! dịch làm người gác, và người gác ấy không quên.
//!
//! # Vì sao đường này tồn tại
//!
//! Ví web gọi `tcc_buildUnsignedTransfer` rồi ký thẳng `signing_message_hex` do
//! máy chủ RPC trả về. Vì thông điệp ấy là một **băm 32 byte**, ví không kiểm
//! được nó ứng với giao dịch nào: người dùng thấy *"gửi 5 TCC cho X"* trên màn
//! hình nhưng ký một chuỗi băm máy chủ đưa. Một RPC bị chiếm trả về băm của
//! giao dịch khác thì màn hình vẫn hiện đúng số tiền người dùng vừa gõ.
//!
//! Máy chủ cũng trả `unsigned_tx_hex` — cả giao dịch, không phải băm của nó.
//! Nên trình duyệt **giải mã được**, và toàn bộ khoảng cách giữa "thứ hiện ra"
//! và "thứ được ký" đóng lại. Trang web không làm được điều này; không phải vì
//! khó, mà vì nó không có chỗ đứng nào an toàn hơn máy chủ nó đang nói chuyện.
//!
//! # `sign` NUỐT `self`
//!
//! Một lần kiểm là một lần ký. Muốn ký lại thì phải kiểm lại — và nếu gói tin
//! đã đổi, lần kiểm ấy sẽ trượt.

use tcc_chain::{
    ChainError, Transfer, check_signing_message,
    wallet::{TxSignature, WalletSecret},
};
use tcc_ui::Node;

use crate::{
    text::Language,
    transaction_screen::{self, XacNhanError},
};

/// Một giao dịch **đã kiểm khớp**, đang chờ người dùng bấm ký.
///
/// Không có cách nào dựng ra nó ngoài [`review`]. Trường riêng tư là cả cơ chế:
///
/// ```compile_fail
/// # use tcc_shell::signing_flow::PendingTransaction;
/// # use tcc_chain::Transfer;
/// // Không dựng thẳng được — trường riêng tư.
/// let tx: Transfer = todo!();
/// let cho = PendingTransaction { tx };
/// ```
#[derive(Debug)]
pub struct PendingTransaction {
    tx: Transfer,
}

impl PendingTransaction {
    /// Giao dịch đã giải mã, để hiện ra hoặc ghi nhật ký.
    #[must_use]
    pub const fn transaction(&self) -> &Transfer {
        &self.tx
    }

    /// Ký. **Nuốt `self`**: một lần kiểm là một lần ký.
    ///
    /// Không có tham số băm nào ở đây, và đó là chủ đích — xem ghi chú đầu tệp.
    #[must_use]
    pub fn sign(self, khoa: &WalletSecret) -> TxSignature {
        khoa.sign_transaction(&self.tx)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SigningFlowError {
    #[error("gói tin giao dịch không phải hex hợp lệ")]
    BadHex,
    #[error("thông điệp ký phải đúng 32 byte, nhận {0}")]
    BadMessageLength(usize),
    #[error(transparent)]
    Chain(#[from] ChainError),
    #[error(transparent)]
    Screen(#[from] XacNhanError),
}

/// Giải mã gói tin, **kiểm khớp**, rồi dựng màn xác nhận.
///
/// Trả về cả giao dịch đang chờ lẫn cây giao diện. Lệch một byte là trả lỗi —
/// và khi đó **không có `PendingTransaction` nào ra đời**, nên không có gì để
/// ký kể cả khi bên gọi bỏ qua lỗi.
///
/// # Errors
/// Hex hỏng, thông điệp không đủ 32 byte, gói tin không giải mã được, băm lệch,
/// hoặc màn hình không dựng được.
pub fn review(
    unsigned_tx_hex: &str,
    signing_message_hex: &str,
    ngon_ngu: Language,
) -> Result<(PendingTransaction, Node), SigningFlowError> {
    let goi_tin = tu_hex(unsigned_tx_hex).ok_or(SigningFlowError::BadHex)?;
    let bam = tu_hex(signing_message_hex).ok_or(SigningFlowError::BadHex)?;
    let bam: [u8; 32] = bam
        .as_slice()
        .try_into()
        .map_err(|_| SigningFlowError::BadMessageLength(bam.len()))?;

    // Giải mã TRƯỚC. Từ đây trở đi mọi thứ hiện ra đều đến từ gói tin, không
    // đến từ thứ người dùng vừa gõ hay thứ máy chủ nói kèm.
    let tx = Transfer::decode(&goi_tin)?;

    // Và kiểm khớp TRƯỚC khi dựng màn hình. `transaction_screen::build` cũng
    // tự kiểm lại — hai lần, cố ý: chỗ này để không sinh ra `PendingTransaction`,
    // chỗ kia để không vẽ ra màn hình. Bỏ một trong hai vẫn còn một.
    check_signing_message(&tx, &bam)?;

    let man = transaction_screen::build(&tx, &bam, ngon_ngu)?;
    Ok((PendingTransaction { tx }, man))
}

fn tu_hex(h: &str) -> Option<Vec<u8>> {
    let h = h.strip_prefix("0x").unwrap_or(h);
    if !h.len().is_multiple_of(2) {
        return None;
    }
    (0..h.len() / 2)
        .map(|i| u8::from_str_radix(h.get(i * 2..i * 2 + 2)?, 16).ok())
        .collect()
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay"
)]
mod kiem_thu {
    use super::*;

    /// Giao dịch THẬT lấy từ testnet chain 91338 — neo ngoài của `tcc-chain`.
    const TX_HEX: &str = "01000000ca640100000000001111111111111111111111111111111111111111111111111111111111111111222222222222222222222222222222222222222222222222222222222222222200000000000000000000f444829163450000000000000000c40051160b00000008520000000000000000000000000000b67a0200000000000000000004000000000000006368616f";
    const BAM: &str = "05d1f926a92678bea9a8d1aae6a7ef86ae295d7a5811301a065e388da97f5b8a";

    /// Đường đầy đủ: gói tin + băm của máy chủ → màn hình → chữ ký.
    #[test]
    fn duong_day_du_chay_duoc() {
        let (cho, man) = review(TX_HEX, BAM, Language::Vi).unwrap();
        assert_eq!(cho.transaction().chain_id, 91338);
        assert!(man.node_count() > 5);

        let khoa = WalletSecret::from_seed_phrase("hello");
        let sig = cho.sign(&khoa);
        assert_eq!(sig.as_bytes()[0], tcc_chain::wallet::SIG_TYPE_ML_DSA_65);
        assert_eq!(
            sig.as_bytes().len(),
            1 + tcc_chain::wallet::RAW_SIGNATURE_LEN
        );
    }

    /// **Đòn thật.** Máy chủ trả băm của một giao dịch KHÁC.
    ///
    /// Đây đúng là thứ ví web không chặn được: nó chỉ có cái băm, không có gói
    /// tin, nên không có gì để so.
    #[test]
    fn may_chu_tra_bam_cua_giao_dich_khac_thi_khong_ky_duoc() {
        let mut bam_khac = BAM.to_owned();
        // Đổi ký tự cuối — vẫn là hex hợp lệ, vẫn 32 byte.
        bam_khac.pop();
        bam_khac.push(if BAM.ends_with('a') { 'b' } else { 'a' });

        let loi = review(TX_HEX, &bam_khac, Language::Vi).unwrap_err();
        assert!(matches!(loi, SigningFlowError::Chain(_)), "{loi}");
    }

    /// Máy chủ đổi NGƯỜI NHẬN trong gói tin nhưng giữ băm cũ → chặn.
    #[test]
    fn may_chu_doi_nguoi_nhan_trong_goi_tin_thi_chan() {
        // 0x22 lặp 32 lần là địa chỉ người nhận; đổi một byte của nó.
        let sua = TX_HEX.replacen("2222222222222222", "22222222222222EE", 1);
        assert_ne!(sua, TX_HEX, "chuỗi thử không đổi được — phép thử vô nghĩa");
        let loi = review(&sua, BAM, Language::Vi).unwrap_err();
        assert!(matches!(loi, SigningFlowError::Chain(_)), "{loi}");
    }

    /// Hex hỏng thì báo lỗi, không hoảng loạn.
    #[test]
    fn hex_hong_thi_bao_loi() {
        assert!(matches!(
            review("zz", BAM, Language::Vi).unwrap_err(),
            SigningFlowError::BadHex
        ));
        assert!(matches!(
            review("010", BAM, Language::Vi).unwrap_err(),
            SigningFlowError::BadHex
        ));
        assert!(matches!(
            review(TX_HEX, "00", Language::Vi).unwrap_err(),
            SigningFlowError::BadMessageLength(1)
        ));
    }

    /// Gói tin cụt thì báo lỗi chứ không đọc quá đuôi.
    #[test]
    fn goi_tin_cut_thi_bao_loi() {
        let cut = &TX_HEX[..40];
        assert!(review(cut, BAM, Language::Vi).is_err());
    }

    /// Chữ ký ra từ đường này phải TRÙNG chữ ký ký thẳng giao dịch ấy.
    ///
    /// Nghe thừa, nhưng nó chốt rằng lớp `PendingTransaction` không lặng lẽ
    /// biến đổi gì trên đường đi.
    #[test]
    fn chu_ky_qua_duong_nay_trung_chu_ky_ky_thang() {
        let khoa = WalletSecret::from_seed_phrase("hello");
        let (cho, _) = review(TX_HEX, BAM, Language::Vi).unwrap();
        let tx = cho.transaction().clone();
        assert_eq!(
            cho.sign(&khoa).as_bytes(),
            khoa.sign_transaction(&tx).as_bytes()
        );
    }
}
