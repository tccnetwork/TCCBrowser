//! Hộp thoại hỏi quyền — màn hình quan trọng nhất của cả trình duyệt.
//!
//! Mọi thứ ở các tầng dưới (chữ ký hậu lượng tử, quyền năng không giả mạo được,
//! trần cây giao diện) đổ dồn về đúng một khoảnh khắc: người dùng đọc hộp này và
//! bấm một nút. Hộp này nói dối hoặc nói khó hiểu thì cả hệ thống vô nghĩa.
//!
//! # Bốn luật của màn hình này
//!
//! 1. **Không bao giờ nói "nhà phát hành đã xác minh".** Ta mới kiểm được chữ
//!    ký, tức là gói không bị sửa. Ai ký thì chưa biết. Có phép thử ở `text.rs`.
//! 2. **Nêu LÝ DO của ứng dụng, nguyên văn.** Lý do nằm trong bản kê khai đã ký,
//!    nên sửa lý do là hỏng chữ ký.
//! 3. **Nêu PHẠM VI cụ thể**, không nói chung chung. "Kết nối mạng" là vô dụng;
//!    "kết nối tới shop.tcc-coin.com" mới quyết định được.
//! 4. **Quyền ký giao dịch phải mang sắc thái mất mát.** Đó là quyền duy nhất
//!    chuyển được tiền của người dùng.

use tcc_capability::Decision;
use tcc_spec::{CapabilityRequest, Manifest, Scope};
use tcc_ui::{Emphasis, Flow, Gap, Node, Tone, UiError};

use crate::{
    permission_store::SignerStatus,
    text::{Language, TextKey, label},
};

/// Mã hành động của hai nút.
///
/// Hằng số chứ không phải chuỗi rời ở hai nơi: khung cửa sổ so sánh chuỗi người
/// dùng bấm với đúng hai giá trị này. Lệch một ký tự thì nút "Cho phép" bấm vào
/// không có tác dụng gì — và không có thông báo lỗi nào, vì về mặt kỹ thuật
/// chẳng có gì hỏng cả. Đó đúng là loại lỗi im lặng nguy hiểm nhất ở màn hình
/// này. Có phép thử chốt hằng số thật sự nằm trên cây.
pub const ACTION_ALLOW: &str = "cho-phep";
pub const ACTION_DENY: &str = "tu-choi";

/// Mã công tắc của một quyền.
///
/// Tiền tố `q-` để không bao giờ đụng `cho-phep` / `tu-choi`: một quyền tên
/// "cho-phep" mà trùng mã nút xác nhận thì bật công tắc thành bấm nút.
#[must_use]
pub fn toggle_id(ten_quyen: &str) -> String {
    format!("q-{ten_quyen}")
}

/// Đổi thứ người dùng đã làm thành quyết định cho **MỘT quyền cụ thể**.
///
/// ⚠️ **Mọi đường không rõ ràng đều dẫn tới TỪ CHỐI.** Đóng cửa sổ, bấm nút từ
/// chối, mã lạ lọt qua, hoặc công tắc của quyền này không bật — tất cả ra
/// `Deny`. Chỉ đúng MỘT đường ra `Allow`: bấm nút cho phép **và** công tắc của
/// đúng quyền đó đang bật.
///
/// Hai điều kiện chứ không phải một: bấm "Cho phép" mà không bật công tắc nào
/// thì **không quyền nào được cấp**. Đó là điểm khác căn bản so với bản hỏi một
/// lần cho cả gói — trước kia đồng ý là đồng ý hết.
///
/// Tách thành hàm thuần vì đây là chỗ dễ sai nhất mà lại khó thấy nhất: nhánh
/// mặc định lỡ nghiêng về `Allow` thì một cửa sổ hỏng cũng thành đồng ý, và
/// không ai phát hiện ra cho tới khi mất tiền.
#[must_use]
pub fn decide(hanh_dong: Option<&str>, dang_bat: &[String], ten_quyen: &str) -> Decision {
    if hanh_dong != Some(ACTION_ALLOW) {
        return Decision::Deny;
    }
    if dang_bat.contains(&toggle_id(ten_quyen)) {
        Decision::Allow
    } else {
        Decision::Deny
    }
}

/// Dựng màn hình hỏi quyền cho một ứng dụng ĐÃ KIỂM CHỮ KÝ.
///
/// Nhận `&Manifest` chứ không nhận từng mảnh rời: bản kê khai chỉ lấy ra được từ
/// `VerifiedApp`, nên kiểu dữ liệu tự nó chặn việc dựng hộp thoại từ dữ liệu
/// chưa xác thực.
///
/// # Errors
/// Chuỗi trong bản kê khai không dùng được trên giao diện, hoặc cây vượt trần
/// (ứng dụng xin quá nhiều quyền).
pub fn build(m: &Manifest, ngon_ngu: Language) -> Result<Node, UiError> {
    build_with_signer(m, ngon_ngu, &SignerStatus::LanDau)
}

/// Như [`dung`] nhưng biết ứng dụng này so với lần trước ta thấy nó.
///
/// # Errors
/// Chuỗi trong bản kê khai không dùng được, hoặc cây vượt trần.
pub fn build_with_signer(
    m: &Manifest,
    ngon_ngu: Language,
    nguoi_ky: &SignerStatus,
) -> Result<Node, UiError> {
    let t = |k: TextKey| label(k, ngon_ngu);

    let mut man = Node::group(Flow::Column, Gap::Large)
        .child(Node::text_with(t(TextKey::QuyenTieuDe), Emphasis::Title)?)?
        // Tên ứng dụng lấy từ bản kê khai đã ký. `tcc-spec` đã chặn ký tự đảo
        // chiều chữ ở trường này, nên "Ví TCC" không thể hiện thành thứ khác.
        .child(Node::text_with(&m.name, Emphasis::Normal)?)?
        .child(Node::text_with(t(TextKey::NguonKhongRo), Emphasis::Subtle)?)?;

    // ⚠️ Cảnh báo đổi khoá đứng NGAY SAU tên ứng dụng, TRƯỚC danh sách quyền.
    //
    // Đặt nó ở cuối thì người dùng đã đọc xong danh sách quyền và tay đã ở nút
    // bấm. Tin quan trọng nhất phải đến trước lúc người ta bắt đầu quyết định.
    if let SignerStatus::DoiKhoa { van_tay_cu } = nguoi_ky {
        man = man
            .child(Node::text_with(t(TextKey::DoiKhoaKy), Emphasis::Title)?)?
            .child(Node::text_with(
                t(TextKey::DoiKhoaKyGiaiThich),
                Emphasis::Normal,
            )?)?
            .child(Node::text_with(
                format!("{}: {van_tay_cu}", t(TextKey::KhoaCu)),
                Emphasis::Subtle,
            )?)?;
    }

    if m.capabilities.is_empty() {
        man = man.child(Node::text(t(TextKey::QuyenKhongXinGi))?)?;
    } else {
        for c in &m.capabilities {
            man = man.child(muc_quyen(c, ngon_ngu)?)?;
        }
    }

    // Cảnh báo đứng TRƯỚC hai nút: đọc rồi mới bấm.
    man.child(Node::text_with(
        t(TextKey::QuyenCanhBaoDanhTinh),
        Emphasis::Subtle,
    )?)?
    .child(
        Node::group(Flow::Row, Gap::Medium)
            .child(Node::button(
                t(TextKey::QuyenNutTuChoi),
                "tu-choi",
                Tone::Neutral,
            )?)?
            // Nút cho phép KHÔNG phải `Tone::Primary`. Sắc thái chính làm nó nổi
            // hơn nút từ chối, tức là giao diện đang đẩy người dùng về phía đồng
            // ý. Hai nút phải ngang nhau về mặt thị giác.
            .child(Node::button(
                t(TextKey::QuyenNutChoPhep),
                "cho-phep",
                Tone::Neutral,
            )?)?,
    )
}

/// Một mục quyền: tiêu đề, phạm vi cụ thể, và lý do nguyên văn của ứng dụng.
fn muc_quyen(c: &CapabilityRequest, ngon_ngu: Language) -> Result<Node, UiError> {
    let t = |k: TextKey| label(k, ngon_ngu);

    let (tieu_de, chi_tiet) = match &c.scope {
        Scope::Network { hosts } => (t(TextKey::QuyenMang), hosts.join(", ")),
        Scope::Storage { quota_bytes } => (
            t(TextKey::QuyenLuuTru),
            format!("{} KB", quota_bytes / 1024),
        ),
        Scope::Wallet {
            may_request_signature,
        } => (
            t(TextKey::QuyenVi),
            if *may_request_signature {
                t(TextKey::ViDuocXinChuKy)
            } else {
                t(TextKey::ViChiDocDiaChi)
            }
            .to_owned(),
        ),
    };

    Node::group(Flow::Column, Gap::Small)
        // Công tắc MẶC ĐỊNH TẮT. Mặc định bật là câu hỏi tự trả lời hộ người dùng.
        .child(Node::toggle(tieu_de, false, &toggle_id(&c.name))?)?
        .child(Node::text_with(&chi_tiet, Emphasis::Normal)?)?
        // Lý do là chữ của ỨNG DỤNG, không phải chữ của ta. Hiện nguyên văn.
        .child(Node::text_with(&c.reason, Emphasis::Subtle)?)
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay, không nuốt lỗi"
)]
mod kiem_thu {
    use super::*;
    use tcc_ui::{AccessNode, NodeKind, Renderer as _, Role};

    fn ke_khai(quyen_json: &str) -> Manifest {
        let s = format!(
            r#"{{"spec_version":"0.1","id":"com.tcc.hello","name":"Ví TCC",
"version":"1.0.0","publisher":"{}","scheme":"hybrid-ed25519-mldsa65-v1",
"content_hash":"{}","entry":"index.html","capabilities":{quyen_json}}}"#,
            "aa".repeat(1992),
            "bb".repeat(48)
        );
        serde_json::from_str(&s).expect("bản kê khai mẫu hỏng")
    }

    /// Gom mọi nhãn trong cây trợ năng thành một chuỗi để tìm kiếm.
    fn moi_nhan(a: &AccessNode, ra: &mut String) {
        if let Some(l) = &a.label {
            ra.push_str(l);
            ra.push('\n');
        }
        for c in &a.children {
            moi_nhan(c, ra);
        }
    }

    fn chu_tren_man_hinh(m: &Manifest, n: Language) -> String {
        let mut s = String::new();
        moi_nhan(&build(m, n).unwrap().accessibility_tree(), &mut s);
        s
    }

    const XIN_MANG: &str = r#"[{"name":"network",
        "scope":{"kind":"network","hosts":["shop.tcc-coin.com"]},
        "reason":"tải danh sách sản phẩm"}]"#;

    const XIN_VI_KY: &str = r#"[{"name":"wallet",
        "scope":{"kind":"wallet","may_request_signature":true},
        "reason":"thanh toán đơn hàng"}]"#;

    #[test]
    fn hien_ten_ung_dung_va_pham_vi_cu_the() {
        let s = chu_tren_man_hinh(&ke_khai(XIN_MANG), Language::En);
        assert!(s.contains("Ví TCC"), "thiếu tên ứng dụng:\n{s}");
        assert!(
            s.contains("shop.tcc-coin.com"),
            "chỉ nói chung chung, không nêu máy chủ cụ thể:\n{s}"
        );
    }

    /// Lý do phải là chữ NGUYÊN VĂN của ứng dụng. Ta viết lại hộ là ta nói thay
    /// cho ứng dụng, mà lý do lại chính là thứ người dùng dựa vào để quyết định.
    #[test]
    fn hien_ly_do_nguyen_van_cua_ung_dung() {
        let s = chu_tren_man_hinh(&ke_khai(XIN_MANG), Language::En);
        assert!(s.contains("tải danh sách sản phẩm"), "{s}");
    }

    /// ⚠️ Đổi khoá ký phải HIỆN RA, và hiện TRƯỚC danh sách quyền.
    #[test]
    fn doi_khoa_ky_thi_canh_bao_hien_ra_truoc_danh_sach_quyen() {
        let m = ke_khai(XIN_VI_KY);
        let tt = SignerStatus::DoiKhoa {
            van_tay_cu: "aaaaaaaaaa…bbbbbbbbbb".to_owned(),
        };
        let cay = build_with_signer(&m, Language::En, &tt).unwrap();
        let mut s = String::new();
        moi_nhan(&cay.accessibility_tree(), &mut s);

        let canh_bao = label(TextKey::DoiKhoaKy, Language::En);
        assert!(s.contains(canh_bao), "thiếu cảnh báo đổi khoá:\n{s}");
        assert!(s.contains("aaaaaaaaaa…bbbbbbbbbb"), "thiếu vân tay khoá cũ");

        // Cảnh báo phải đứng TRƯỚC mục quyền đầu tiên.
        let vt_canh_bao = s.find(canh_bao).expect("có cảnh báo");
        let vt_quyen = s
            .find(label(TextKey::QuyenVi, Language::En))
            .expect("có mục quyền");
        assert!(
            vt_canh_bao < vt_quyen,
            "cảnh báo đứng SAU danh sách quyền — lúc đó tay người dùng đã ở nút bấm"
        );
    }

    /// Không đổi khoá thì KHÔNG được doạ người dùng.
    #[test]
    fn khong_doi_khoa_thi_khong_canh_bao() {
        for tt in [SignerStatus::LanDau, SignerStatus::KhopKhoaCu] {
            let cay = build_with_signer(&ke_khai(XIN_VI_KY), Language::En, &tt).unwrap();
            let mut s = String::new();
            moi_nhan(&cay.accessibility_tree(), &mut s);
            assert!(
                !s.contains(label(TextKey::DoiKhoaKy, Language::En)),
                "{tt:?} mà vẫn cảnh báo đổi khoá — doạ nhầm người dùng"
            );
        }
    }

    /// ⚠️ Cảnh báo danh tính phải LUÔN có mặt, kể cả khi ứng dụng không xin gì.
    #[test]
    fn luon_co_canh_bao_danh_tinh() {
        for q in ["[]", XIN_MANG, XIN_VI_KY] {
            for n in [Language::En, Language::Vi] {
                let s = chu_tren_man_hinh(&ke_khai(q), n);
                assert!(
                    s.contains(label(TextKey::QuyenCanhBaoDanhTinh, n)),
                    "thiếu cảnh báo danh tính với quyền {q}, ngôn ngữ {n:?}"
                );
            }
        }
    }

    #[test]
    fn khong_xin_gi_thi_noi_ro_la_khong_xin_gi() {
        let s = chu_tren_man_hinh(&ke_khai("[]"), Language::En);
        assert!(
            s.contains(label(TextKey::QuyenKhongXinGi, Language::En)),
            "{s}"
        );
    }

    /// ⚠️ Quyền ký giao dịch là quyền duy nhất chuyển được tiền. Người dùng phải
    /// thấy nó nói rõ điều đó, không phải một chữ "ví" chung chung.
    #[test]
    fn quyen_ky_giao_dich_noi_ro_la_chuyen_tien() {
        let s = chu_tren_man_hinh(&ke_khai(XIN_VI_KY), Language::En);
        assert!(s.contains("this moves money"), "{s}");

        let sv = chu_tren_man_hinh(&ke_khai(XIN_VI_KY), Language::Vi);
        assert!(sv.contains("chuyển tiền"), "{sv}");
    }

    #[test]
    fn vi_chi_doc_khong_bi_ghi_nham_thanh_ky_duoc() {
        let chi_doc = r#"[{"name":"wallet",
            "scope":{"kind":"wallet","may_request_signature":false},
            "reason":"hiện số dư"}]"#;
        let s = chu_tren_man_hinh(&ke_khai(chi_doc), Language::En);
        assert!(s.contains("Read your wallet address only"), "{s}");
        assert!(!s.contains("this moves money"), "doạ nhầm người dùng:\n{s}");
    }

    /// ⚠️ Phép thử mặc-định-từ-chối. Đây là phép thử rẻ nhất và đắt giá nhất ở
    /// tệp này: nó chốt rằng KHÔNG có đường tắt nào tới `Allow`.
    fn bat(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn moi_duong_khong_ro_rang_deu_ra_tu_choi() {
        let du = bat(&["q-network"]);
        assert_eq!(decide(Some(ACTION_ALLOW), &du, "network"), Decision::Allow);

        for x in [
            None,                    // đóng cửa sổ, không bấm gì
            Some(ACTION_DENY),       // bấm từ chối
            Some(""),                // chuỗi rỗng
            Some("cho-phep "),       // thừa khoảng trắng
            Some("Cho-Phep"),        // khác hoa thường
            Some("cho-phep-tat-ca"), // tiền tố trùng
            Some("xcho-phep"),       // hậu tố trùng
        ] {
            assert_eq!(
                decide(x, &du, "network"),
                Decision::Deny,
                "{x:?} lọt thành ĐỒNG Ý"
            );
        }
    }

    /// ⚠️ Điểm khác căn bản so với bản hỏi một lần cho cả gói.
    ///
    /// Bấm "Cho phép" mà không bật công tắc nào thì **không quyền nào được cấp**.
    /// Trước kia đồng ý là đồng ý hết.
    #[test]
    fn bam_cho_phep_ma_khong_bat_gi_thi_khong_cap_quyen_nao() {
        for q in ["network", "storage", "wallet"] {
            assert_eq!(
                decide(Some(ACTION_ALLOW), &[], q),
                Decision::Deny,
                "quyền \"{q}\" được cấp dù người dùng không bật công tắc nào"
            );
        }
    }

    /// ⚠️ Bật một quyền KHÔNG kéo theo quyền khác. Đây là toàn bộ lý do tồn tại
    /// của việc hỏi theo từng mục.
    #[test]
    fn bat_mot_quyen_khong_keo_theo_quyen_khac() {
        let chi_mang = bat(&["q-network"]);
        assert_eq!(
            decide(Some(ACTION_ALLOW), &chi_mang, "network"),
            Decision::Allow
        );
        assert_eq!(
            decide(Some(ACTION_ALLOW), &chi_mang, "wallet"),
            Decision::Deny,
            "bật quyền mạng lại kéo theo quyền ví"
        );
    }

    /// Mã công tắc phải khớp CHÍNH XÁC, không khớp tiền tố hay hậu tố.
    #[test]
    fn ma_cong_tac_khop_chinh_xac() {
        for gia in [
            "q-network-tat-ca",
            "xq-network",
            "network",
            "Q-NETWORK",
            "q-net",
        ] {
            assert_eq!(
                decide(Some(ACTION_ALLOW), &bat(&[gia]), "network"),
                Decision::Deny,
                "mã công tắc \"{gia}\" lọt thành quyền mạng"
            );
        }
    }

    /// Tiền tố `q-` tồn tại để một quyền tên "cho-phep" không biến công tắc
    /// thành nút xác nhận.
    #[test]
    fn ma_cong_tac_khong_dung_do_ma_nut() {
        assert_ne!(toggle_id("cho-phep"), ACTION_ALLOW);
        assert_ne!(toggle_id("tu-choi"), ACTION_DENY);
    }

    /// ⚠️ Hai mã hành động phải THẬT SỰ có trên cây.
    ///
    /// Khung cửa sổ so chuỗi người dùng bấm với hai hằng số này. Lệch là nút
    /// "Cho phép" bấm không ăn, mà không có lỗi nào hiện ra.
    #[test]
    fn hai_ma_hanh_dong_that_su_nam_tren_cay() {
        let cay = build(&ke_khai(XIN_VI_KY), Language::En).unwrap();
        let ds: Vec<&str> = cay.action_ids().iter().map(|a| a.as_str()).collect();
        assert!(
            ds.contains(&ACTION_ALLOW),
            "hằng số cho phép \"{ACTION_ALLOW}\" không có trên cây: {ds:?}"
        );
        assert!(
            ds.contains(&ACTION_DENY),
            "hằng số từ chối \"{ACTION_DENY}\" không có trên cây: {ds:?}"
        );
        // Mỗi quyền một công tắc — nếu thiếu thì người dùng không có cách nào
        // bật nó, và quyền đó vĩnh viễn không cấp được.
        assert!(
            ds.contains(&"q-wallet"),
            "thiếu công tắc cho quyền ví: {ds:?}"
        );
        assert_eq!(ds.len(), 3, "số mã hành động không như mong đợi: {ds:?}");
    }

    /// ⚠️ MỌI công tắc phải MẶC ĐỊNH TẮT trên hộp thoại hỏi quyền.
    ///
    /// Một công tắc quyền mặc định bật là câu hỏi tự trả lời hộ người dùng — và
    /// ở đây nó tự trả lời "đồng ý".
    #[test]
    fn moi_cong_tac_mac_dinh_tat() {
        let nhieu = r#"[
            {"name":"network","scope":{"kind":"network","hosts":["a.tcc-coin.com"]},"reason":"x"},
            {"name":"storage","scope":{"kind":"storage","quota_bytes":1024},"reason":"y"},
            {"name":"wallet","scope":{"kind":"wallet","may_request_signature":true},"reason":"z"}
        ]"#;
        let cay = build(&ke_khai(nhieu), Language::En).unwrap();
        let mut ds = Vec::new();
        gom_cong_tac(&cay, &mut ds);
        assert_eq!(ds.len(), 3, "thiếu công tắc cho một quyền nào đó");
        for (ten, on) in ds {
            assert!(!on, "công tắc \"{ten}\" mặc định BẬT");
        }
    }

    fn gom_cong_tac(n: &Node, ra: &mut Vec<(String, bool)>) {
        if let NodeKind::Toggle { action, on, .. } = n.kind() {
            ra.push((action.as_str().to_owned(), *on));
        }
        for c in n.children() {
            gom_cong_tac(c, ra);
        }
    }

    /// ⚠️ Hai nút phải NGANG NHAU về sắc thái.
    ///
    /// Cho nút "Cho phép" sắc thái chính là thiết kế đẩy người dùng về phía đồng
    /// ý — đó là mẫu tối, và ở đây nó tối vào đúng chỗ nguy hiểm nhất.
    #[test]
    fn hai_nut_ngang_nhau_khong_day_ve_phia_dong_y() {
        let cay = build(&ke_khai(XIN_VI_KY), Language::En).unwrap();
        let mut sac_thai = Vec::new();
        gom_sac_thai(&cay, &mut sac_thai);
        assert_eq!(sac_thai.len(), 2, "cần đúng hai nút");
        assert_eq!(
            sac_thai[0], sac_thai[1],
            "hai nút khác sắc thái — giao diện đang đẩy người dùng về một phía"
        );
    }

    fn gom_sac_thai(n: &Node, ra: &mut Vec<Tone>) {
        if let NodeKind::Button { tone, .. } = n.kind() {
            ra.push(*tone);
        }
        for c in n.children() {
            gom_sac_thai(c, ra);
        }
    }

    /// Ứng dụng xin rất nhiều quyền thì hộp thoại vẫn phải dựng được, không
    /// được vượt trần cây rồi hỏng — hỏng ở đây nghĩa là không hỏi được người
    /// dùng, mà không hỏi được thì phải từ chối chứ không được cho qua.
    #[test]
    fn xin_nhieu_quyen_van_dung_duoc_hop_thoai() {
        let nhieu: Vec<String> = (0..50)
            .map(|i| {
                format!(
                    r#"{{"name":"network","scope":{{"kind":"network",
                    "hosts":["may{i}.tcc-coin.com"]}},"reason":"lý do {i}"}}"#
                )
            })
            .collect();
        let m = ke_khai(&format!("[{}]", nhieu.join(",")));
        let cay = build(&m, Language::En).expect("hộp thoại hỏng khi xin nhiều quyền");
        assert!(cay.node_count() < tcc_ui::MAX_NODES);
    }

    /// Toàn bộ hộp thoại phải qua được kiểm định trợ năng của bộ dựng thật —
    /// đây là màn hình mà người khiếm thị cần đọc chính xác nhất.
    #[test]
    fn hop_thoai_qua_duoc_kiem_dinh_tro_nang() {
        use tcc_render_webview::WebViewRenderer;
        let cay = build(&ke_khai(XIN_VI_KY), Language::En).unwrap();
        let mut bd = WebViewRenderer::new();
        tcc_ui::check_accessibility_parity(&mut bd, &cay)
            .expect("hộp thoại hỏi quyền không qua được kiểm định trợ năng");

        // Và cây trợ năng phải có đúng hai nút bấm.
        let a = bd.published_accessibility().unwrap();
        let mut n = 0;
        dem_nut(&a, &mut n);
        assert_eq!(n, 2);
    }

    fn dem_nut(a: &AccessNode, n: &mut usize) {
        if matches!(a.role, Role::Button { .. }) {
            *n += 1;
        }
        for c in &a.children {
            dem_nut(c, n);
        }
    }
}
