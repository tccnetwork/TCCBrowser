//! Màn hình quản lý quyền đã cấp.
//!
//! # Vì sao màn hình này bắt buộc phải có
//!
//! "Cấp" mà không kèm "xem lại và thu hồi" chỉ là nửa hệ thống quyền. Người dùng
//! đồng ý một lần rồi không bao giờ thấy lại quyết định đó nữa — và một quyết
//! định không rút lại được thì lần sau người ta sẽ ngần ngại, hoặc tệ hơn, sẽ
//! bấm bừa vì nghĩ đằng nào cũng không đổi được.
//!
//! # Đây là màn hình CỦA TRÌNH DUYỆT
//!
//! Nên nó chịu đúng luật của hộp thoại hỏi quyền: **ứng dụng không đưa được một
//! byte nào vào đây**. Khung cửa sổ truyền `|_| None` cho trình phục vụ tệp.
//!
//! # Chữ mô tả phạm vi là chữ ĐỌC TỪ ĐĨA
//!
//! Nó chỉ để hiện, không bao giờ để quyết định — xem `ghi_nho::MucQuyen::mo_ta`.
//! Có phép thử chốt rằng `tra()` không đọc nó.

use tcc_ui::{Emphasis, Flow, Gap, Node, Tone, UiError};

use crate::{
    ghi_nho::MucLietKe,
    loi::{Khoa, NgonNgu, nhan},
};

/// Mã nút đóng màn hình.
pub const HANH_DONG_DONG: &str = "dong";

/// Mã nút quên một ứng dụng.
///
/// Tiền tố `quen-` cộng mã ứng dụng. Mã ứng dụng đã bị `AppId::parse` ép về chữ
/// thường ASCII, chữ số và dấu chấm — đúng tập ký tự `ActionId` cho phép, nên
/// ghép vào là ra một mã hành động hợp lệ mà không cần biến đổi gì.
#[must_use]
pub fn ma_quen(ma_ung_dung: &str) -> String {
    format!("quen-{ma_ung_dung}")
}

/// Tách mã ứng dụng ra khỏi mã hành động "quên".
#[must_use]
pub fn ung_dung_can_quen(hanh_dong: &str) -> Option<&str> {
    hanh_dong.strip_prefix("quen-")
}

/// Dựng màn hình quản lý quyền.
///
/// # Errors
/// Chuỗi không dùng được trên giao diện, hoặc quá nhiều ứng dụng làm cây vượt trần.
pub fn dung(ds: &[MucLietKe], ngon_ngu: NgonNgu) -> Result<Node, UiError> {
    let t = |k: Khoa| nhan(k, ngon_ngu);

    let mut man = Node::group(Flow::Column, Gap::Large)
        .child(Node::text_with(t(Khoa::QuanLyTieuDe), Emphasis::Title)?)?;

    if ds.is_empty() {
        man = man.child(Node::text(t(Khoa::QuanLyTrong))?)?;
    } else {
        for m in ds {
            man = man.child(muc_ung_dung(m, ngon_ngu)?)?;
        }
        // Giải thích đứng SAU danh sách, TRƯỚC nút đóng: người dùng đọc nó ngay
        // trước lúc quyết định có bấm "Quên" hay không.
        man = man.child(Node::text_with(
            t(Khoa::QuanLyGiaiThichQuen),
            Emphasis::Subtle,
        )?)?;
    }

    man.child(Node::button(
        t(Khoa::QuanLyNutDong),
        HANH_DONG_DONG,
        Tone::Neutral,
    )?)
}

fn muc_ung_dung(m: &MucLietKe, ngon_ngu: NgonNgu) -> Result<Node, UiError> {
    let t = |k: Khoa| nhan(k, ngon_ngu);

    let mut o = Node::group(Flow::Column, Gap::Small)
        .child(Node::text_with(&m.ma_ung_dung, Emphasis::Normal)?)?
        // Vân tay khoá để người dùng đối chiếu nếu họ muốn — và để hai ứng dụng
        // cùng mã nhưng khác người ký không trông giống hệt nhau.
        .child(Node::text_with(
            format!("{}: {}", t(Khoa::KhoaCu), m.van_tay_khoa),
            Emphasis::Subtle,
        )?)?;

    for q in &m.quyen {
        let trang_thai = if q.cho_phep {
            t(Khoa::QuanLyDaChoPhep)
        } else {
            t(Khoa::QuanLyDaTuChoi)
        };
        let dong = if q.mo_ta.trim().is_empty() {
            format!("{} — {trang_thai}", q.ten)
        } else {
            format!("{} ({}) — {trang_thai}", q.ten, q.mo_ta)
        };
        o = o.child(Node::text_with(&dong, Emphasis::Normal)?)?;
    }

    // Nút "Quên" mang sắc thái MẤT MÁT: nó xoá thứ người dùng đã quyết định, và
    // không có đường hoàn tác.
    o.child(Node::button(
        t(Khoa::QuanLyNutQuen),
        &ma_quen(&m.ma_ung_dung),
        Tone::Danger,
    )?)
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay, không nuốt lỗi"
)]
mod kiem_thu {
    use super::*;
    use crate::ghi_nho::QuyenDaTraLoi;
    use tcc_ui::{AccessNode, NodeKind};

    fn muc(id: &str, cho_phep: bool) -> MucLietKe {
        MucLietKe {
            ma_ung_dung: id.to_owned(),
            van_tay_khoa: "aaaaaaaaaa…bbbbbbbbbb".to_owned(),
            quyen: vec![QuyenDaTraLoi {
                ten: "network".to_owned(),
                mo_ta: "shop.tcc-coin.com".to_owned(),
                cho_phep,
            }],
        }
    }

    fn chu(n: &Node) -> String {
        fn gom(a: &AccessNode, ra: &mut String) {
            if let Some(l) = &a.label {
                ra.push_str(l);
                ra.push('\n');
            }
            for c in &a.children {
                gom(c, ra);
            }
        }
        let mut s = String::new();
        gom(&n.accessibility_tree(), &mut s);
        s
    }

    #[test]
    fn hien_du_ma_ung_dung_pham_vi_va_trang_thai() {
        let s = chu(&dung(&[muc("com.tcc.vi", true)], NgonNgu::En).unwrap());
        assert!(s.contains("com.tcc.vi"), "{s}");
        assert!(s.contains("shop.tcc-coin.com"), "thiếu phạm vi:\n{s}");
        assert!(s.contains(nhan(Khoa::QuanLyDaChoPhep, NgonNgu::En)), "{s}");
    }

    /// Đã từ chối cũng phải hiện ra — nếu không, người dùng tưởng ứng dụng chưa
    /// từng hỏi, và không hiểu vì sao nó không chạy.
    #[test]
    fn quyen_da_tu_choi_cung_hien_ra() {
        let s = chu(&dung(&[muc("com.tcc.a", false)], NgonNgu::En).unwrap());
        assert!(s.contains(nhan(Khoa::QuanLyDaTuChoi, NgonNgu::En)), "{s}");
    }

    #[test]
    fn khong_co_gi_thi_noi_ro_la_khong_co_gi() {
        let s = chu(&dung(&[], NgonNgu::En).unwrap());
        assert!(s.contains(nhan(Khoa::QuanLyTrong, NgonNgu::En)), "{s}");
    }

    /// ⚠️ Nút "Quên" phải mang sắc thái MẤT MÁT.
    ///
    /// Nó xoá thứ người dùng đã quyết định và không có đường hoàn tác — trình
    /// đọc màn hình phải nghe ra điều đó, không chỉ người nhìn thấy màu.
    #[test]
    fn nut_quen_mang_sac_thai_mat_mat() {
        let cay = dung(&[muc("com.tcc.a", true)], NgonNgu::En).unwrap();
        let mut ds = Vec::new();
        gom_nut(&cay, &mut ds);
        let quen = ds
            .iter()
            .find(|(a, _)| a.starts_with("quen-"))
            .expect("có nút quên");
        assert_eq!(quen.1, Tone::Danger, "nút quên không mang sắc thái mất mát");
    }

    fn gom_nut(n: &Node, ra: &mut Vec<(String, Tone)>) {
        if let NodeKind::Button { action, tone, .. } = n.kind() {
            ra.push((action.as_str().to_owned(), *tone));
        }
        for c in n.children() {
            gom_nut(c, ra);
        }
    }

    /// Mã "quên" phải đi và về được nguyên vẹn, và không đụng mã nút đóng.
    #[test]
    fn ma_quen_di_ve_nguyen_ven() {
        for id in ["com.tcc.vi", "com.tcc.vi-du.hello", "a.b"] {
            assert_eq!(ung_dung_can_quen(&ma_quen(id)), Some(id));
        }
        assert_eq!(ung_dung_can_quen(HANH_DONG_DONG), None);
        assert_ne!(ma_quen("dong"), HANH_DONG_DONG);
    }

    /// Nhiều ứng dụng thì mỗi ứng dụng một nút quên RIÊNG — dùng chung một nút
    /// là quên nhầm ứng dụng.
    #[test]
    fn moi_ung_dung_mot_nut_quen_rieng() {
        let cay = dung(
            &[muc("com.tcc.a", true), muc("com.tcc.b", true)],
            NgonNgu::En,
        )
        .unwrap();
        let ds: Vec<&str> = cay.action_ids().iter().map(|a| a.as_str()).collect();
        assert!(ds.contains(&"quen-com.tcc.a"), "{ds:?}");
        assert!(ds.contains(&"quen-com.tcc.b"), "{ds:?}");
        assert!(ds.contains(&HANH_DONG_DONG), "{ds:?}");
        assert_eq!(ds.len(), 3);
    }
}
