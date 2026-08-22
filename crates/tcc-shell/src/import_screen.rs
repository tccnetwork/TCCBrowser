//! Màn nhập ví cũ từ ví web.
//!
//! Sau cờ `import-web-wallet`, chuyển tiếp xuống `tcc-chain`.
//!
//! # Màn "xong" tồn tại vì một câu duy nhất
//!
//! Nhập ví **không** làm bản cũ biến mất: ví web vẫn giữ bản của nó trong
//! `localStorage`, vẫn khoá bằng PIN, vẫn yếu đúng như trước. Trình duyệt cố ý
//! không đụng vào — xoá hộ người dùng thứ họ chưa bảo xoá là một cách hỏng
//! riêng, và tệ hơn nếu bản nhập sang có vấn đề.
//!
//! Nhưng **im lặng về nó là tệ nhất**: người dùng tưởng mình đã dọn sạch, mất
//! cảnh giác, mà rủi ro không giảm một chút nào. Nên câu ấy mang
//! [`Emphasis::Warning`], cùng mức với câu *"việc này chuyển tiền"* ở màn xác
//! nhận giao dịch, và có phép thử chốt rằng nó nằm trên màn hình.
//!
//! # Địa chỉ hiện ĐỦ, như mọi nơi khác
//!
//! Cắt ngắn kiểu `0x1111…1111` là thói quen phổ biến và là một lỗ — xem ghi chú
//! đầu `transaction_screen`. Ở đây còn thêm một lý do: người dùng đang so địa
//! chỉ trên màn hình này với địa chỉ họ thấy ở trang web.
//!
//! # Lỗi được dịch, không hiện nguyên văn
//!
//! `ImportError` viết cho người đọc mã. Người dùng cần biết **làm gì tiếp**:
//! gõ lại PIN, hay ví này không nhập được ở đây. Nên mỗi nhánh lỗi có một câu
//! riêng trong `text.rs`, và `match` là toàn phần nên thêm nhánh lỗi mới mà
//! quên dịch thì **không biên dịch được**.

use tcc_chain::import::{ImportError, ImportedWallet, WebWallet};
use tcc_ui::{Emphasis, Flow, Gap, Node, Tone, UiError};

use crate::text::{Language, TextKey, label};

/// Mã nút huỷ.
pub const ACTION_CANCEL: &str = "huy-nhap-vi";
/// Mã nút đóng màn "xong".
pub const ACTION_DONE: &str = "xong-nhap-vi";
/// Mã nút mở khoá bằng PIN.
pub const ACTION_UNLOCK: &str = "mo-khoa-vi";

/// Mã nút chọn một ví. Địa chỉ đã là hex `0x…` nên ghép vào là mã hợp lệ.
#[must_use]
pub fn choose_id(dia_chi: &str) -> String {
    format!("nhap-{dia_chi}")
}

/// Tách địa chỉ ra khỏi mã hành động.
#[must_use]
pub fn address_to_import(hanh_dong: &str) -> Option<&str> {
    hanh_dong.strip_prefix("nhap-")
}

/// Màn chọn ví — **chưa cần PIN**.
///
/// Người dùng thấy mình có mấy ví, nhãn gì, địa chỉ nào, rồi mới quyết định gõ
/// PIN cho cái nào. Bắt gõ PIN trước khi biết mình đang mở cái gì là dạy người
/// ta gõ PIN vào mọi ô hỏi PIN.
///
/// # Errors
/// Chuỗi không dùng được trên giao diện, hoặc quá nhiều ví làm cây vượt trần.
pub fn build_choice(ds: &[WebWallet], ngon_ngu: Language) -> Result<Node, UiError> {
    let t = |k: TextKey| label(k, ngon_ngu);

    let mut man = Node::group(Flow::Column, Gap::Large)
        .child(Node::text_with(t(TextKey::NhapTieuDe), Emphasis::Title)?)?;

    if ds.is_empty() {
        man = man.child(Node::text(t(TextKey::NhapTrong))?)?;
    } else {
        man = man.child(Node::text_with(
            t(TextKey::NhapGiaiThich),
            Emphasis::Subtle,
        )?)?;
        for v in ds {
            man = man.child(muc_vi(v, ngon_ngu)?)?;
        }
    }

    man.child(Node::button(
        t(TextKey::NhapNutHuy),
        ACTION_CANCEL,
        Tone::Neutral,
    )?)
}

fn muc_vi(v: &WebWallet, ngon_ngu: Language) -> Result<Node, UiError> {
    let t = |k: TextKey| label(k, ngon_ngu);

    let cum_tu = if v.has_mnemonic() {
        TextKey::NhapCoCumTu
    } else {
        TextKey::NhapKhongCumTu
    };

    Node::group(Flow::Column, Gap::Small)
        // Địa chỉ ĐỦ 64 ký tự — người dùng đang so nó với trang web.
        .child(Node::text_with(v.address.clone(), Emphasis::Normal)?)?
        .child(Node::text_with(
            format!("{}: {}", t(TextKey::NhapNhan), v.label),
            Emphasis::Subtle,
        )?)?
        .child(Node::text_with(t(cum_tu), Emphasis::Subtle)?)?
        .child(Node::button(
            t(TextKey::NhapNutChon),
            &choose_id(&v.address),
            Tone::Neutral,
        )?)
}

/// Màn hỏi PIN.
///
/// # Ô che chữ ở đây là ô của KHUNG TRÌNH DUYỆT
///
/// Từ 16/08/2026 gói ứng dụng **không** dựng được ô che chữ nữa
/// (`secret-field-from-app`, `spec/0.1/05-interface.md`). Ô này dựng bằng mã
/// Rust của khung, không đi qua đường đọc từ đĩa, nên nó là ô che chữ **duy
/// nhất** người dùng gặp — và hàng chấm tròn lấy lại được nghĩa của nó.
///
/// # Hỏi bí mật thì phải nói VÌ SAO
///
/// Câu giải thích đứng **ngay trên** ô nhập, không nằm dưới, không nằm ở màn
/// trước. Hỏi bí mật mà không nói vì sao là dạy người dùng gõ bí mật vào bất kỳ
/// ô nào hỏi — đúng thói quen mà kẻ lừa đảo sống nhờ.
///
/// # Errors
/// Chuỗi không dùng được trên giao diện.
pub fn build_pin(dia_chi: &str, ngon_ngu: Language) -> Result<Node, UiError> {
    let t = |k: TextKey| label(k, ngon_ngu);

    Node::group(Flow::Column, Gap::Large)
        .child(Node::text_with(t(TextKey::NhapPinTieuDe), Emphasis::Title)?)?
        // Địa chỉ ĐỦ: người dùng phải thấy mình đang mở ví NÀO trước khi gõ.
        .child(Node::text_with(dia_chi.to_owned(), Emphasis::Normal)?)?
        .child(Node::text(t(TextKey::NhapPinViSao))?)?
        .child(Node::text_with(
            t(TextKey::NhapPinChiDungMotLan),
            Emphasis::Subtle,
        )?)?
        // `secret: true` — và đây là chỗ DUY NHẤT trong cả trình duyệt mà nó
        // hợp lệ ngoài mã của khung.
        .child(Node::field(t(TextKey::NhapPinNhan), "", true)?)?
        .child(
            Node::group(Flow::Row, Gap::Medium)
                .child(Node::button(
                    t(TextKey::NhapNutMoKhoa),
                    ACTION_UNLOCK,
                    Tone::Neutral,
                )?)?
                .child(Node::button(
                    t(TextKey::NhapNutHuy),
                    ACTION_CANCEL,
                    Tone::Neutral,
                )?)?,
        )
}

/// Màn "xong" — **và câu về bản cũ**.
///
/// Đọc ghi chú đầu tệp trước khi sửa gì ở đây.
///
/// # Errors
/// Chuỗi không dùng được trên giao diện.
pub fn build_done(vi: &ImportedWallet, ngon_ngu: Language) -> Result<Node, UiError> {
    let t = |k: TextKey| label(k, ngon_ngu);

    let mut man = Node::group(Flow::Column, Gap::Large)
        .child(Node::text_with(
            t(TextKey::NhapXongTieuDe),
            Emphasis::Title,
        )?)?
        .child(Node::text_with(
            vi.secret.address().to_string(),
            Emphasis::Normal,
        )?)?;

    if vi.mnemonic.is_some() {
        man = man.child(Node::text(t(TextKey::NhapCumTuDaMangSang))?)?;
    }

    // ⚠️ Hai câu này là lý do màn hình này tồn tại. CẢNH BÁO, không phải chữ mờ.
    man.child(Node::text_with(
        t(TextKey::NhapBanCuVanCon),
        Emphasis::Warning,
    )?)?
    .child(Node::text_with(
        t(TextKey::NhapBanCuLamGi),
        Emphasis::Normal,
    )?)?
    .child(Node::button(
        t(TextKey::NhapNutXong),
        ACTION_DONE,
        Tone::Neutral,
    )?)
}

/// Đổi lỗi kỹ thuật thành câu nói được với người dùng.
///
/// `match` toàn phần: thêm một nhánh `ImportError` mà quên dịch thì **không
/// biên dịch được**, đúng lý do `TextKey` là `enum`.
#[must_use]
pub const fn error_text(loi: &ImportError) -> TextKey {
    match loi {
        ImportError::WrongPin => TextKey::NhapLoiSaiPin,
        ImportError::UnsupportedKeyFormat(_) => TextKey::NhapLoiKhoaCu,
        ImportError::AddressMismatch => TextKey::NhapLoiLechDiaChi,
        // Bốn nhánh còn lại đều là "tệp không đọc được": người dùng không làm
        // gì khác nhau được với chúng, nên tách ra bốn câu chỉ là bốn cách nói
        // cùng một việc.
        ImportError::Json(_)
        | ImportError::UnsupportedSchema(_)
        | ImportError::Base64(_)
        | ImportError::BadParameters => TextKey::NhapLoiDocKhongDuoc,
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
    use tcc_chain::import::read_export;

    const MAU: &str = include_str!("../../tcc-chain/data/vi-web-mau.json");
    const PIN: &str = "matkhau-thu-nghiem";
    const DIA_CHI: &str = "0x11b22b300e195c44c910d71cdb1515c4617e852393cde5e80c860906b8a2d549";

    fn ve(cay: &Node) -> String {
        crate::do_cay::chu(cay)
    }

    fn da_nhap() -> ImportedWallet {
        read_export(MAU).unwrap().remove(0).unlock(PIN).unwrap()
    }

    /// **Câu về bản cũ phải có mặt, và phải NỔI RÕ.**
    ///
    /// Đây là phép thử quan trọng nhất tệp này. Nếu ai đó dọn màn hình cho gọn
    /// và bỏ câu ấy đi thì người dùng tưởng đã dọn sạch trong khi bản yếu vẫn
    /// nằm ở trang web.
    #[test]
    fn man_xong_noi_ro_ban_cu_van_con() {
        for ngon_ngu in [Language::En, Language::Vi] {
            let s = ve(&build_done(&da_nhap(), ngon_ngu).unwrap());
            assert!(
                s.contains(label(TextKey::NhapBanCuVanCon, ngon_ngu)),
                "thiếu câu về bản cũ ({ngon_ngu:?}):\n{s}"
            );
            assert!(
                s.contains("[cảnh-báo]"),
                "câu về bản cũ không mang dấu hiệu cảnh báo ({ngon_ngu:?}):\n{s}"
            );
        }
    }

    /// Và phải nói người dùng LÀM GÌ với bản cũ ấy, không chỉ báo là nó còn đó.
    #[test]
    fn man_xong_noi_lam_gi_voi_ban_cu() {
        let s = ve(&build_done(&da_nhap(), Language::Vi).unwrap());
        assert!(
            s.contains(label(TextKey::NhapBanCuLamGi, Language::Vi)),
            "{s}"
        );
    }

    /// Địa chỉ hiện ĐỦ, không cắt ngắn — người dùng đang so với trang web.
    #[test]
    fn dia_chi_hien_du_o_ca_hai_man() {
        let ds = read_export(MAU).unwrap();
        for s in [
            ve(&build_choice(&ds, Language::Vi).unwrap()),
            ve(&build_done(&da_nhap(), Language::Vi).unwrap()),
        ] {
            assert!(s.contains(DIA_CHI), "địa chỉ bị cắt ngắn:\n{s}");
            assert!(!s.contains('…'), "màn hình có dấu cắt ngắn:\n{s}");
        }
    }

    /// Màn chọn KHÔNG cần PIN và KHÔNG để lọt gì bí mật.
    #[test]
    fn man_chon_khong_lo_bi_mat() {
        let ds = read_export(MAU).unwrap();
        let s = ve(&build_choice(&ds, Language::Vi).unwrap());
        assert!(s.contains("Ví thử nghiệm"));
        assert!(!s.contains(PIN), "PIN lọt ra màn hình");
        // Bản mã cũng không việc gì phải hiện ra.
        assert!(!s.contains("aRG7XXNKpLY"), "bản mã lọt ra màn hình:\n{s}");
    }

    /// Không có ví nào thì nói thẳng, đừng hiện một màn trống.
    #[test]
    fn khong_co_vi_nao_thi_noi_ra() {
        let s = ve(&build_choice(&[], Language::En).unwrap());
        assert!(s.contains(label(TextKey::NhapTrong, Language::En)), "{s}");
    }

    /// Mã hành động phải vòng lại đúng địa chỉ.
    #[test]
    fn ma_hanh_dong_vong_lai_dung() {
        assert_eq!(address_to_import(&choose_id(DIA_CHI)), Some(DIA_CHI));
        assert_eq!(address_to_import("huy-nhap-vi"), None);
    }

    /// Mỗi lỗi ra một câu người dùng dùng được — và sai PIN phải KHÁC hẳn
    /// "ví này không nhập được", vì hai bên người dùng làm hai việc khác nhau.
    #[test]
    fn moi_loi_ra_mot_cau_dung_viec() {
        assert_eq!(error_text(&ImportError::WrongPin), TextKey::NhapLoiSaiPin);
        assert_eq!(
            error_text(&ImportError::UnsupportedKeyFormat(4032)),
            TextKey::NhapLoiKhoaCu
        );
        assert_ne!(
            error_text(&ImportError::WrongPin),
            error_text(&ImportError::UnsupportedKeyFormat(4032)),
            "gõ lại PIN và bỏ cuộc là hai việc khác nhau"
        );
        assert_eq!(
            error_text(&ImportError::Json("x".to_owned())),
            TextKey::NhapLoiDocKhongDuoc
        );
    }

    /// Ô PIN phải là ô CHE CHỮ thật, không phải ô thường.
    #[test]
    fn o_pin_la_o_che_chu() {
        let cay = build_pin(DIA_CHI, Language::Vi).unwrap();
        let mut co = false;
        gom_o_nhap(&cay, &mut co);
        assert!(co, "ô PIN không che chữ — chữ hiện ra màn hình khi gõ");
    }

    fn gom_o_nhap(n: &Node, ra: &mut bool) {
        if let tcc_ui::NodeKind::Field { secret, .. } = n.kind() {
            *ra = *secret;
        }
        for c in n.children() {
            gom_o_nhap(c, ra);
        }
    }

    /// **Phải nói VÌ SAO đang hỏi PIN**, và nói ngay trên màn ấy.
    #[test]
    fn man_pin_noi_vi_sao_va_dia_chi_nao() {
        for ngon_ngu in [Language::En, Language::Vi] {
            let s = ve(&build_pin(DIA_CHI, ngon_ngu).unwrap());
            assert!(
                s.contains(label(TextKey::NhapPinViSao, ngon_ngu)),
                "màn hỏi PIN không nói vì sao ({ngon_ngu:?}):\n{s}"
            );
            assert!(s.contains(DIA_CHI), "không nói đang mở ví nào:\n{s}");
        }
    }

    /// Và phải nói PIN không bị lưu lại.
    #[test]
    fn man_pin_noi_khong_luu_lai() {
        let s = ve(&build_pin(DIA_CHI, Language::Vi).unwrap());
        assert!(
            s.contains(label(TextKey::NhapPinChiDungMotLan, Language::Vi)),
            "{s}"
        );
    }

    /// Cả hai màn phải qua được kiểm định trợ năng của bộ dựng thật.
    #[test]
    fn qua_duoc_kiem_dinh_tro_nang() {
        let ds = read_export(MAU).unwrap();
        for cay in [
            build_choice(&ds, Language::Vi).unwrap(),
            build_pin(DIA_CHI, Language::Vi).unwrap(),
            build_done(&da_nhap(), Language::Vi).unwrap(),
        ] {
            let mut bd = tcc_render_raster::RasterRenderer::new();
            tcc_ui::check_accessibility_parity(&mut bd, &cay)
                .expect("màn nhập ví không qua được kiểm định trợ năng");
        }
    }
}
