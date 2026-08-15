//! Màn xác nhận giao dịch — nơi phép kiểm chống ký mù cứu người dùng.
//!
//! # Vì sao màn này không phải "cho đẹp"
//!
//! Ví web hiện hiện lại **thứ người dùng vừa gõ**, rồi ký một băm 32 byte do
//! máy chủ đưa. Hai thứ ấy không liên quan gì tới nhau. Một RPC bị chiếm trả về
//! băm của giao dịch khác thì màn hình vẫn hiện đúng số tiền người dùng nhập.
//!
//! Màn này hiện **thứ đã giải mã từ chính gói tin sắp được ký**, và chỉ hiện
//! sau khi băm tự tính khớp băm được đưa. Lệch thì không có nút ký — không phải
//! nút mờ đi, mà là **không dựng ra**.
//!
//! # Hai quyết định về cách hiện
//!
//! **Địa chỉ hiện ĐỦ 64 ký tự hex.** Cắt ngắn kiểu `0x1111…1111` là thói quen
//! phổ biến và là một lỗ: kẻ tấn công dò được một địa chỉ trùng cả đầu lẫn đuôi
//! với địa chỉ thật chỉ trong vài giây, vì tám ký tự hex là hai tỉ khả năng.
//! Địa chỉ dài thì khó đọc; địa chỉ cắt ngắn thì dễ đọc và sai.
//!
//! **Số tiền tính bằng số nguyên, không bao giờ bằng số thực.** 18 chữ số thập
//! phân vượt xa độ chính xác của `f64`, và một lần làm tròn ở màn hình xác nhận
//! là người dùng đồng ý với một con số khác con số họ đọc.

use tcc_chain::{ChainError, Transfer, check_signing_message};
use tcc_ui::{Emphasis, Flow, Gap, Node, Tone, UiError};

use crate::text::{Language, TextKey, label as t};

/// Mã hành động của nút ký. Danh sách trắng ở tầng dưới chặn mã lạ.
pub const ACTION_SIGN: &str = "ky-giao-dich";
/// Mã hành động của nút huỷ.
pub const ACTION_CANCEL: &str = "huy-giao-dich";

/// Số chữ số thập phân của TCC — như wei của Ethereum.
const DECIMALS: u32 = 18;

/// Đổi đơn vị nhỏ nhất sang chuỗi người đọc được, **bằng số nguyên**.
///
/// Không dùng số thực ở bất kỳ bước nào: `f64` chỉ giữ được khoảng 15–17 chữ số
/// có nghĩa, mà ở đây có 18 chữ số thập phân cộng phần nguyên. Một lần làm tròn
/// là người dùng đồng ý với con số khác con số họ đọc.
#[must_use]
pub fn format_amount(nho_nhat: u128) -> String {
    let chia = 10u128.pow(DECIMALS);
    let nguyen = nho_nhat / chia;
    let le = nho_nhat % chia;
    if le == 0 {
        return format!("{nguyen}");
    }
    // Bỏ số 0 ở đuôi phần thập phân, nhưng KHÔNG bỏ số 0 ở đầu nó.
    let mut s = format!("{le:0width$}", width = DECIMALS as usize);
    while s.ends_with('0') {
        s.pop();
    }
    format!("{nguyen}.{s}")
}

/// Dựng màn xác nhận, SAU khi đã kiểm thông điệp ký.
///
/// # Errors
/// Băm không khớp — và khi đó **không có màn xác nhận nào được dựng**, vì không
/// có gì đáng để người dùng xác nhận.
pub fn build(
    tx: &Transfer,
    signing_message_tu_may_chu: &[u8; 32],
    ngon_ngu: Language,
) -> Result<Node, XacNhanError> {
    // Kiểm TRƯỚC khi vẽ. Vẽ trước rồi kiểm sau là đã hiện cho người dùng một
    // giao dịch mà ta chưa biết có thật hay không.
    check_signing_message(tx, signing_message_tu_may_chu)?;

    let t = |k: TextKey| t(k, ngon_ngu);
    let mut man = Node::group(Flow::Column, Gap::Large)
        .child(Node::text_with(t(TextKey::GdTieuDe), Emphasis::Title)?)?
        // Câu này mang nhấn mạnh CẢNH BÁO vì nó chuyển tiền — cùng luật với
        // quyền ví ở hộp thoại hỏi quyền.
        .child(Node::text_with(
            t(TextKey::GdChuyenTien),
            Emphasis::Warning,
        )?)?;

    let mut dong = |nhan: &str, gia_tri: String| -> Result<(), UiError> {
        man = core::mem::replace(&mut man, Node::group(Flow::Column, Gap::Small)).child(
            Node::group(Flow::Column, Gap::None)
                .child(Node::text_with(nhan, Emphasis::Subtle)?)?
                .child(Node::text_with(gia_tri, Emphasis::Normal)?)?,
        )?;
        Ok(())
    };

    dong(
        t(TextKey::GdSoTien),
        format!("{} TCC", format_amount(tx.amount)),
    )?;
    // Địa chỉ ĐỦ, không cắt — xem ghi chú đầu tệp.
    dong(t(TextKey::GdNguoiNhan), tx.to.to_string())?;
    dong(
        t(TextKey::GdPhi),
        format!(
            "{} TCC",
            format_amount(u128::from(tx.gas_price) * u128::from(tx.gas_limit))
        ),
    )?;
    dong(t(TextKey::GdMang), format!("{}", tx.chain_id))?;
    dong(t(TextKey::GdThuTu), format!("{}", tx.nonce))?;
    if !tx.memo.is_empty() {
        dong(t(TextKey::GdGhiNho), tx.memo.clone())?;
    }

    Ok(man.child(
        Node::group(Flow::Row, Gap::Medium)
            // Hai nút CÙNG sắc thái. Làm nút ký nổi hơn là đẩy người dùng
            // về một phía, đúng lúc nguy hiểm nhất — luật 04 §hai nút.
            .child(Node::button(
                t(TextKey::GdNutKy),
                ACTION_SIGN,
                Tone::Neutral,
            )?)?
            .child(Node::button(
                t(TextKey::GdNutHuy),
                ACTION_CANCEL,
                Tone::Neutral,
            )?)?,
    )?)
}

#[derive(Debug, thiserror::Error)]
pub enum XacNhanError {
    #[error(transparent)]
    Chain(#[from] ChainError),
    #[error(transparent)]
    Ui(#[from] UiError),
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "kiểm thử: hỏng thì phải nổ ngay")]
mod kiem_thu {
    use super::*;
    use tcc_chain::Address;
    use tcc_render_webview::WebViewRenderer;
    use tcc_ui::Renderer as _;

    fn mau() -> Transfer {
        Transfer {
            version: 1,
            chain_id: 91338,
            from: Address([0x11; 32]),
            to: Address([0x22; 32]),
            nonce: 0,
            amount: 5_000_000_000_000_000_000,
            gas_price: 47_619_047_620,
            gas_limit: 21_000,
            timestamp: 0,
            expires_at: 162_486,
            memo: "chao".to_owned(),
        }
    }

    fn ve(t: &Transfer, bam: &[u8; 32]) -> Result<String, XacNhanError> {
        let cay = build(t, bam, Language::Vi)?;
        let mut bd = WebViewRenderer::new();
        bd.render(&cay).unwrap();
        Ok(bd.body().to_owned())
    }

    /// 18 chữ số thập phân — chỗ một lần làm tròn là mất tiền.
    #[test]
    fn so_tien_tinh_bang_so_nguyen() {
        assert_eq!(format_amount(5_000_000_000_000_000_000), "5");
        assert_eq!(format_amount(0), "0");
        assert_eq!(format_amount(1), "0.000000000000000001");
        assert_eq!(format_amount(1_500_000_000_000_000_000), "1.5");
        // Con số này mất chính xác nếu đi qua f64.
        assert_eq!(
            format_amount(123_456_789_012_345_678_901_234_567_890),
            "123456789012.34567890123456789"
        );
    }

    /// Băm lệch thì KHÔNG dựng màn nào — không có nút để bấm.
    #[test]
    fn bam_lech_thi_khong_co_man_hinh() {
        let t = mau();
        let mut sai = t.signing_message();
        sai[0] ^= 1;
        let loi = build(&t, &sai, Language::Vi).unwrap_err();
        assert!(matches!(loi, XacNhanError::Chain(_)), "{loi}");
    }

    /// Đòn thật: gói tin của kẻ gian, băm của giao dịch người dùng nghĩ mình ký.
    #[test]
    fn may_chu_doi_nguoi_nhan_thi_khong_ve_ra_man_nao() {
        let that = mau();
        let mut ke_gian = mau();
        ke_gian.to = Address([0xEE; 32]);
        assert!(build(&ke_gian, &that.signing_message(), Language::Vi).is_err());
    }

    /// Địa chỉ hiện ĐỦ 64 ký tự — cắt ngắn là lỗ dò trùng đầu-đuôi.
    #[test]
    fn dia_chi_hien_du_khong_cat_ngan() {
        let t = mau();
        let s = ve(&t, &t.signing_message()).unwrap();
        let du = t.to.to_string();
        assert_eq!(du.len(), 66, "địa chỉ phải là 0x + 64 hex");
        assert!(s.contains(&du), "địa chỉ bị cắt ngắn trên màn hình:\n{s}");
        assert!(!s.contains('…'), "màn hình có dấu cắt ngắn");
    }

    /// Màn hình hiện thứ GIẢI MÃ ĐƯỢC, không hiện lại thứ người dùng gõ.
    #[test]
    fn hien_so_tien_va_mang_lay_tu_goi_tin() {
        let t = mau();
        let s = ve(&t, &t.signing_message()).unwrap();
        assert!(s.contains("5 TCC"), "thiếu số tiền:\n{s}");
        assert!(
            s.contains("91338"),
            "thiếu mã mạng — người dùng cần biết đang ký trên mạng nào"
        );
        assert!(s.contains("chao"), "thiếu ghi nhớ");
    }

    /// Câu "việc này chuyển tiền" phải NỔI RÕ, không chìm vào chữ thường.
    #[test]
    fn cau_chuyen_tien_duoc_ve_khac_di() {
        let t = mau();
        let s = ve(&t, &t.signing_message()).unwrap();
        assert!(
            s.contains("data-nhan=\"canh-bao\""),
            "câu chuyển tiền không mang dấu hiệu cảnh báo:\n{s}"
        );
    }

    /// Hai nút cùng sắc thái — không đẩy người dùng về phía ký.
    #[test]
    fn hai_nut_ngang_nhau() {
        let t = mau();
        let cay = build(&t, &t.signing_message(), Language::Vi).unwrap();
        let mut sac = Vec::new();
        gom_sac_thai(&cay, &mut sac);
        assert_eq!(sac.len(), 2, "cần đúng hai nút");
        assert_eq!(
            sac[0], sac[1],
            "hai nút khác sắc thái — đang đẩy người dùng"
        );
    }

    fn gom_sac_thai(n: &Node, ra: &mut Vec<Tone>) {
        if let tcc_ui::NodeKind::Button { tone, .. } = n.kind() {
            ra.push(*tone);
        }
        for c in n.children() {
            gom_sac_thai(c, ra);
        }
    }

    /// Toàn màn phải qua được kiểm định trợ năng của bộ dựng thật.
    #[test]
    fn qua_duoc_kiem_dinh_tro_nang() {
        let t = mau();
        let cay = build(&t, &t.signing_message(), Language::Vi).unwrap();
        let mut bd = WebViewRenderer::new();
        tcc_ui::check_accessibility_parity(&mut bd, &cay)
            .expect("màn xác nhận giao dịch không qua được kiểm định trợ năng");
    }
}
