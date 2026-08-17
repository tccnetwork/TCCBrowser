//! Gõ thẳng **cụm từ khôi phục** — đường nhập ví không cần tệp nào.
//!
//! ```text
//! gõ 24 chữ  →  xem địa chỉ nó mở ra  →  lưu
//! ```
//!
//! # Vì sao ô này KHÔNG che chữ
//!
//! Cụm từ khôi phục nhạy cảm ngang khoá bí mật, nên phản xạ đầu tiên là che nó
//! đi. Nhưng che thì **gõ 24 chữ mà không soát lại được**, và một chữ sai ra
//! một ví khác **cũng hợp lệ y như thế** — không có gì báo lỗi, người dùng chỉ
//! thấy ví trống.
//!
//! Nên chữ hiện ra, và màn hình **nói thẳng** rằng ai nhìn được màn hình là lấy
//! được ví. Đổi một rủi ro âm thầm lấy một rủi ro nhìn thấy được, và nói ra cái
//! rủi ro ấy — chứ không giả vờ là không có.
//!
//! # Vì sao có màn xác nhận địa chỉ
//!
//! Tổng kiểm BIP39 bắt được phần lớn lỗi gõ, nhưng **không bắt được hết**: đổi
//! một chữ vẫn có thể ra một cụm từ hợp lệ khác. Khi ấy thứ duy nhất phân biệt
//! là **địa chỉ**, nên nó phải hiện ra ĐỦ trước khi lưu, không phải sau.

use tcc_chain::wallet::WalletSecret;
use tcc_ui::{Emphasis, Flow, Gap, Node, Tone, UiError};

use crate::text::{Language, TextKey, label};

/// Mã nút "tiếp tục" ở màn gõ.
pub const ACTION_CONTINUE: &str = "cum-tu-tiep";
/// Mã nút "lưu ví này" ở màn xác nhận.
pub const ACTION_SAVE: &str = "cum-tu-luu";
/// Mã nút huỷ — **chỉ ở màn gõ**, nơi chưa có gì để quay lại.
pub const ACTION_CANCEL: &str = "cum-tu-huy";

/// Mã nút **quay lại sửa** — ở màn xác nhận và màn báo hỏng.
///
/// Tách khỏi [`ACTION_CANCEL`] chứ không dùng chung: màn xác nhận sinh ra để
/// **bắt lỗi gõ**, nên bấm "không phải ví này" mà mất cả 24 chữ vừa gõ là đúng
/// cách khiến lần sau người ta đi dán từ chỗ khác — mà "chỗ khác" thường là một
/// ô nhập trên web.
///
/// Hai mã riêng để danh sách trắng của từng màn nói đúng thứ màn ấy cho phép.
pub const ACTION_BACK: &str = "cum-tu-quay-lai";

/// Màn 1: gõ cụm từ.
///
/// `loi` khác `None` thì hiện lại kèm câu báo lỗi — người dùng gõ lại chứ không
/// bị đá về từ đầu.
///
/// # Errors
/// Chuỗi không dùng được trên giao diện.
pub fn build_entry(loi: Option<&str>, ngon_ngu: Language) -> Result<Node, UiError> {
    let t = |k: TextKey| label(k, ngon_ngu);

    let mut man = Node::group(Flow::Column, Gap::Large)
        .child(Node::text_with(t(TextKey::CumTuTieuDe), Emphasis::Title)?)?
        .child(Node::text(t(TextKey::CumTuGiaiThich))?)?
        // ⚠️ Câu này CẢNH BÁO, và nó phải đứng TRƯỚC ô nhập: nói sau khi người
        // ta đã gõ xong thì chẳng để làm gì.
        .child(Node::text_with(
            t(TextKey::CumTuAiNhinCungDoc),
            Emphasis::Warning,
        )?)?;

    if let Some(cau) = loi {
        man = man.child(Node::text_with(cau, Emphasis::Warning)?)?;
    }

    // `secret: false` — cố ý. Xem ghi chú đầu tệp.
    man.child(Node::field(t(TextKey::CumTuNhan), "", false)?)?
        .child(
            Node::group(Flow::Row, Gap::Medium)
                .child(Node::button(
                    t(TextKey::CumTuNutTiep),
                    ACTION_CONTINUE,
                    Tone::Neutral,
                )?)?
                .child(Node::button(
                    t(TextKey::NhapNutHuy),
                    ACTION_CANCEL,
                    Tone::Neutral,
                )?)?,
        )
}

/// Màn 2: **địa chỉ mà cụm từ ấy mở ra**, trước khi lưu.
///
/// # Errors
/// Chuỗi không dùng được trên giao diện.
pub fn build_confirm(dia_chi: &str, ngon_ngu: Language) -> Result<Node, UiError> {
    let t = |k: TextKey| label(k, ngon_ngu);

    Node::group(Flow::Column, Gap::Large)
        .child(Node::text_with(
            t(TextKey::CumTuXacNhanTieuDe),
            Emphasis::Title,
        )?)?
        .child(Node::text(t(TextKey::CumTuDayLaVi))?)?
        // Địa chỉ ĐỦ 64 ký tự hex — cắt ngắn là lỗ dò trùng đầu-đuôi, và ở đây
        // còn thêm lý do: người dùng đang so nó với thứ họ nhớ.
        .child(Node::text_with(dia_chi.to_owned(), Emphasis::Normal)?)?
        .child(Node::text_with(
            t(TextKey::CumTuKiemKyTruocKhiCat),
            Emphasis::Warning,
        )?)?
        .child(
            Node::group(Flow::Row, Gap::Medium)
                .child(Node::button(
                    t(TextKey::CumTuNutCat),
                    ACTION_SAVE,
                    Tone::Neutral,
                )?)?
                // QUAY LẠI, không phải huỷ: cả điểm của màn này là sửa được.
                .child(Node::button(
                    t(TextKey::CumTuNutSuaLai),
                    ACTION_BACK,
                    Tone::Neutral,
                )?)?,
        )
}

/// Mã nút đóng ở màn báo hỏng.
pub const ACTION_CLOSE: &str = "hong-dong";

/// Màn **báo hỏng, hiện TRONG cửa sổ**.
///
/// # Vì sao cần
///
/// Trước 17/08/2026 mọi lỗi của luồng ví đi ra `stderr` và cửa sổ **đóng im
/// lặng**. Người dùng trình duyệt không nhìn terminal, nên từ phía họ nó y hệt
/// *"gõ xong thì ứng dụng tắt"* — không biết hỏng ở đâu, không biết cụm từ vừa
/// gõ có bị lưu ở đâu không.
///
/// Câu thứ hai là câu quan trọng: **nói rõ KHÔNG có gì được lưu**. Người vừa gõ
/// 24 chữ vào một cửa sổ vừa tắt cần biết ngay điều đó.
///
/// # Errors
/// Chuỗi không dùng được trên giao diện.
pub fn build_failure(chi_tiet: &str, ngon_ngu: Language) -> Result<Node, UiError> {
    let t = |k: TextKey| label(k, ngon_ngu);
    let mut man = Node::group(Flow::Column, Gap::Large)
        .child(Node::text_with(t(TextKey::HongTieuDe), Emphasis::Title)?)?;

    // Gói chưa ký: dịch câu của hệ điều hành ra tiếng người, và nói rõ KHÔNG
    // phải người dùng gõ sai. Câu gốc vẫn giữ ở dưới, dạng chữ mờ — người soát
    // cần nó, người dùng thì không.
    if crate::wallet_store::is_unsigned_build(chi_tiet) {
        man = man
            .child(Node::text_with(
                t(TextKey::HongChuaKyGoi),
                Emphasis::Warning,
            )?)?
            .child(Node::text(t(TextKey::HongKhongPhaiLoiCuaBan))?)?
            .child(Node::text_with(chi_tiet.to_owned(), Emphasis::Subtle)?)?;
    } else {
        man = man.child(Node::text_with(chi_tiet.to_owned(), Emphasis::Warning)?)?;
    }

    man.child(Node::text(t(TextKey::HongKhongCatDuoc))?)?.child(
        Node::group(Flow::Row, Gap::Medium)
            // Quay lại đứng TRƯỚC: hỏng xong thì việc thường làm là sửa,
            // không phải bỏ cuộc.
            .child(Node::button(
                t(TextKey::CumTuNutSuaLai),
                ACTION_BACK,
                Tone::Neutral,
            )?)?
            .child(Node::button(
                t(TextKey::HongNutDong),
                ACTION_CLOSE,
                Tone::Neutral,
            )?)?,
    )
}

/// Đọc thứ người dùng gõ thành một ví.
///
/// Nhận **24 chữ** hoặc **64 ký tự hex**. Hai đường dẫn xuất khác nhau, và
/// chúng KHÔNG được lẫn: 64 hex là hạt giống thật (`from_raw_seed`), 24 chữ đi
/// qua BIP39 rồi mới thành hạt giống. Lẫn hai đường là ra ví khác — cái bẫy đã
/// làm chuỗi dừng chốt khối ngày 30/07/2026.
///
/// # Errors
/// Không phải định dạng nào trong hai.
pub fn read_phrase(go: &str) -> Result<WalletSecret, PhraseError> {
    let go = go.trim();
    let goc = go.strip_prefix("0x").unwrap_or(go);
    if goc.len() == 64 && goc.chars().all(|c| c.is_ascii_hexdigit()) {
        let mut hat = [0u8; 32];
        for (i, o) in hat.iter_mut().enumerate() {
            let cap = goc
                .get(i * 2..i * 2 + 2)
                .ok_or(PhraseError::NotRecognised)?;
            *o = u8::from_str_radix(cap, 16).map_err(|_| PhraseError::NotRecognised)?;
        }
        return Ok(WalletSecret::from_raw_seed(hat));
    }
    if go.split_whitespace().count() == 24 {
        // Chuẩn hoá khoảng trắng: người ta dán từ nhiều nguồn, và xuống dòng
        // giữa các chữ là chuyện thường.
        let gon = go.split_whitespace().collect::<Vec<_>>().join(" ");
        return WalletSecret::from_mnemonic(&gon).map_err(|_| PhraseError::NotRecognised);
    }
    Err(PhraseError::NotRecognised)
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PhraseError {
    /// Cố ý **KHÔNG nói rõ sai ở đâu**.
    ///
    /// "Chữ thứ 7 không có trong từ điển" hữu ích khi người dùng tự gõ, nhưng
    /// nó cũng là một máy dò cho ai đang thử cụm từ của người khác. Ở màn hình
    /// này người dùng có cả cụm từ trong tay và gõ lại được — nên đổi lấy sự im
    /// lặng là đáng.
    #[error("không phải cụm từ khôi phục hợp lệ")]
    NotRecognised,
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay"
)]
mod kiem_thu {
    use super::*;
    use tcc_render_webview::WebViewRenderer;
    use tcc_ui::Renderer as _;

    const ABANDON: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
    /// Neo tới `tcc-keygen` của đội chuỗi — xem `tcc-chain/src/wallet.rs`.
    const DIA_CHI: &str = "0x11b22b300e195c44c910d71cdb1515c4617e852393cde5e80c860906b8a2d549";

    fn ve(cay: &Node) -> String {
        let mut bd = WebViewRenderer::new();
        bd.render(cay).unwrap();
        bd.body().to_owned()
    }

    /// 24 chữ ra đúng ví, neo bởi chương trình của đội chuỗi.
    #[test]
    fn hai_muoi_bon_chu_ra_dung_vi() {
        assert_eq!(read_phrase(ABANDON).unwrap().address().to_string(), DIA_CHI);
        // Xuống dòng, thừa khoảng trắng — vẫn phải nhận.
        let ban = format!("  {}  ", ABANDON.replace(' ', "\n "));
        assert_eq!(read_phrase(&ban).unwrap().address().to_string(), DIA_CHI);
    }

    /// 64 hex đi đường HẠT GIỐNG THẬT, không đi đường băm chuỗi.
    #[test]
    fn hex_di_duong_hat_giong_that() {
        let hex = "653ef7954a968b00af07b13969e2735d100c01b422550054126d3734cccde406";
        let a = read_phrase(hex).unwrap();
        let b = read_phrase(&format!("0x{hex}")).unwrap();
        assert_eq!(a.address(), b.address(), "tiền tố 0x làm đổi ví");
        // Và KHÁC hẳn đường băm chuỗi — lẫn hai đường là ra ví khác.
        assert_ne!(a.address(), WalletSecret::from_seed_phrase(hex).address());
    }

    /// Gõ nhầm một chữ thì TỪ CHỐI, không âm thầm ra ví khác.
    #[test]
    fn go_nham_mot_chu_thi_tu_choi() {
        let sai = ABANDON.replace(" art", " arm");
        assert!(matches!(read_phrase(&sai), Err(PhraseError::NotRecognised)));
    }

    /// Lỗi KHÔNG nói rõ sai ở đâu — nó cũng là máy dò cho người khác.
    #[test]
    fn loi_khong_chi_ro_cho_sai() {
        let sai = ABANDON.replace(" art", " khongcotutrongtudien");
        let loi = read_phrase(&sai).unwrap_err();
        let cau = loi.to_string();
        assert!(!cau.contains("khongcotutrongtudien"), "{cau}");
        assert!(!cau.contains("24"), "câu lỗi chỉ ra vị trí: {cau}");
    }

    /// **Ô cụm từ KHÔNG che chữ, và màn hình phải nói ra vì sao.**
    #[test]
    fn o_cum_tu_khong_che_nhung_co_canh_bao() {
        for ngon_ngu in [Language::En, Language::Vi] {
            let cay = build_entry(None, ngon_ngu).unwrap();
            let mut co_che = false;
            gom_o(&cay, &mut co_che);
            assert!(!co_che, "ô cụm từ đang che chữ — không soát lại được");

            let s = ve(&cay);
            assert!(
                s.contains(label(TextKey::CumTuAiNhinCungDoc, ngon_ngu)),
                "không cảnh báo người xung quanh đọc được ({ngon_ngu:?})"
            );
            assert!(
                s.contains("data-nhan=\"canh-bao\""),
                "cảnh báo không nổi rõ"
            );
        }
    }

    fn gom_o(n: &Node, ra: &mut bool) {
        if let tcc_ui::NodeKind::Field { secret, .. } = n.kind() {
            *ra = *ra || *secret;
        }
        for c in n.children() {
            gom_o(c, ra);
        }
    }

    /// **Màn xác nhận hiện địa chỉ ĐỦ**, và nói vì sao phải đối chiếu.
    #[test]
    fn man_xac_nhan_hien_dia_chi_du() {
        let s = ve(&build_confirm(DIA_CHI, Language::Vi).unwrap());
        assert!(s.contains(DIA_CHI), "địa chỉ bị cắt ngắn:\n{s}");
        assert!(!s.contains('…'), "màn hình có dấu cắt ngắn");
        assert!(s.contains(label(TextKey::CumTuKiemKyTruocKhiCat, Language::Vi)));
    }

    /// Hai nút cùng sắc thái — không đẩy người dùng về phía lưu.
    #[test]
    fn hai_nut_cung_sac_thai() {
        for cay in [
            build_entry(None, Language::Vi).unwrap(),
            build_confirm(DIA_CHI, Language::Vi).unwrap(),
        ] {
            let mut sac = Vec::new();
            gom_sac(&cay, &mut sac);
            assert_eq!(sac.len(), 2);
            assert_eq!(
                sac[0], sac[1],
                "hai nút khác sắc thái — đang đẩy người dùng"
            );
        }
    }

    fn gom_sac(n: &Node, ra: &mut Vec<Tone>) {
        if let tcc_ui::NodeKind::Button { tone, .. } = n.kind() {
            ra.push(*tone);
        }
        for c in n.children() {
            gom_sac(c, ra);
        }
    }

    /// **Màn báo hỏng phải nói KHÔNG có gì được lưu.**
    ///
    /// Người vừa gõ 24 chữ vào một cửa sổ vừa hỏng cần biết ngay điều đó.
    #[test]
    fn man_hong_noi_ro_khong_luu_gi() {
        for ngon_ngu in [Language::En, Language::Vi] {
            let s = ve(&build_failure("kho khoá từ chối", ngon_ngu).unwrap());
            assert!(s.contains("kho khoá từ chối"), "mất chi tiết lỗi:\n{s}");
            assert!(
                s.contains(label(TextKey::HongKhongCatDuoc, ngon_ngu)),
                "không nói rõ chưa lưu gì ({ngon_ngu:?}):\n{s}"
            );
            assert!(s.contains("data-nhan=\"canh-bao\""), "lỗi không nổi rõ");
        }
    }

    /// **Lỗi "gói chưa ký" phải được DỊCH ra tiếng người.**
    ///
    /// Câu của macOS ("A required entitlement isn't present") nói với lập trình
    /// viên. Người dùng đọc nó chỉ tưởng mình vừa gõ sai cụm từ.
    #[test]
    fn loi_chua_ky_goi_duoc_dich_ra_tieng_nguoi() {
        let tho = "không cất được vào kho khoá: kho khoá của hệ điều hành báo lỗi: \
                   A required entitlement isn't present.";
        for ngon_ngu in [Language::En, Language::Vi] {
            let s = ve(&build_failure(tho, ngon_ngu).unwrap());
            assert!(
                s.contains(label(TextKey::HongChuaKyGoi, ngon_ngu)),
                "không dịch câu của macOS ({ngon_ngu:?}):\n{s}"
            );
            assert!(
                s.contains(label(TextKey::HongKhongPhaiLoiCuaBan, ngon_ngu)),
                "không nói rõ đây không phải lỗi người dùng ({ngon_ngu:?})"
            );
            // Câu gốc VẪN giữ — người soát cần nó.
            assert!(
                s.contains("entitlement"),
                "mất câu gốc, người soát mất manh mối"
            );
        }
    }

    /// Lỗi KHÁC thì hiện nguyên văn, không bịa thêm câu "chưa ký gói".
    #[test]
    fn loi_khac_khong_bi_gan_cau_chua_ky() {
        let s = ve(&build_failure("sai PIN, hoặc dữ liệu đã hỏng", Language::Vi).unwrap());
        assert!(
            !s.contains(label(TextKey::HongChuaKyGoi, Language::Vi)),
            "{s}"
        );
        assert!(s.contains("sai PIN"));
    }

    /// Màn báo hỏng KHÔNG được lộ cụm từ người dùng vừa gõ.
    #[test]
    fn man_hong_khong_lo_cum_tu() {
        let s = ve(&build_failure("sai PIN, hoặc dữ liệu đã hỏng", Language::Vi).unwrap());
        assert!(!s.contains("abandon"), "{s}");
    }

    /// Cả hai màn phải qua kiểm định trợ năng của bộ dựng thật.
    #[test]
    fn qua_duoc_kiem_dinh_tro_nang() {
        for cay in [
            build_entry(Some("lỗi thử"), Language::Vi).unwrap(),
            build_confirm(DIA_CHI, Language::En).unwrap(),
        ] {
            let mut bd = WebViewRenderer::new();
            tcc_ui::check_accessibility_parity(&mut bd, &cay)
                .expect("màn cụm từ không qua được kiểm định trợ năng");
        }
    }
}
