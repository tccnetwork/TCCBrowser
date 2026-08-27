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

/// Bản dựng này có cấp được quyền năng ấy không.
///
/// ⚠️ Hỏi một câu mà không cấp được câu trả lời là hộp thoại NÓI DỐI. Bản dựng
/// không có ví mà vẫn hiện "cho phép chạm vào ví — việc này chuyển tiền", kèm
/// một công tắc gạt được, là bắt người dùng quyết định về một thứ không tồn
/// tại. Lộ ra ngày 26/08/2026 khi ví chuyển thành việc làm SAU, và bản dựng
/// không ví thành bản chính.
#[must_use]
pub fn cap_duoc(scope: &Scope) -> bool {
    match scope {
        // ⚑ BƯỚC 1 của quyết định 27/08/2026: ví giữ khoá và **CHỈ ĐỌC ĐỊA CHỈ**.
        //
        // `may_request_signature: true` — xin quyền KÝ GIAO DỊCH — bản dựng này
        // KHÔNG cung cấp, dù có cờ `wallet`. Ký giao dịch chỉ mở sau khi qua
        // kiểm định an ninh độc lập; xem `docs/ke-hoach.md`.
        //
        // Vì sao không đơn giản là "bỏ ví đi cho an toàn": một hạt giống không
        // bao giờ dùng được thì KHÔNG an toàn hơn, nó vô dụng — người dùng vẫn
        // phải quay về ví web để giao dịch, và phải NHẬP LẠI hạt giống ở đó.
        // Lúc ấy hạt giống nằm ở HAI chỗ: Keychain và một tệp trong hồ sơ trình
        // duyệt. Tệ hơn lúc đầu.
        //
        // Đọc địa chỉ thì làm được việc thật (hiện số dư, nhận diện người dùng)
        // mà KHÔNG có gì để cổng mainnet phải chặn.
        Scope::Wallet {
            may_request_signature,
        } => cfg!(feature = "wallet") && !*may_request_signature,
        Scope::Network { .. } | Scope::Storage { .. } => true,
    }
}

/// Một mục quyền: tiêu đề, phạm vi cụ thể, và lý do nguyên văn của ứng dụng.
fn muc_quyen(c: &CapabilityRequest, ngon_ngu: Language) -> Result<Node, UiError> {
    let t = |k: TextKey| label(k, ngon_ngu);

    // Không cấp được thì NÓI RA, và KHÔNG dựng công tắc. Một công tắc gạt được
    // là một lời hứa rằng gạt xong sẽ có tác dụng.
    if !cap_duoc(&c.scope) {
        return Node::group(Flow::Column, Gap::Small)
            .child(Node::text(match &c.scope {
                Scope::Wallet { .. } => t(TextKey::QuyenVi).to_owned(),
                _ => c.name.clone(),
            })?)?
            .child(Node::text_with(
                match &c.scope {
                    // Hai lý do KHÁC NHAU, và người dùng cần phân biệt được:
                    // "bản dựng không có ví" khác hẳn "có ví, nhưng bản dựng
                    // này không ký giao dịch". Gộp làm một là nói sai với đúng
                    // người đang phải quyết định.
                    Scope::Wallet {
                        may_request_signature: true,
                    } if cfg!(feature = "wallet") => t(TextKey::ViKhongKyGiaoDich),
                    _ => t(TextKey::ViBanDungKhongCo),
                },
                Emphasis::Warning,
            )?)?
            .child(Node::text_with(&c.reason, Emphasis::Subtle)?);
    }

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
        // Quyền ví KÝ ĐƯỢC phải hiện KHÁC HẲN mọi quyền khác — `04` bắt buộc thế.
        // Nói ra thôi là chưa đủ: B31 đã dạy rằng một ý định khai mà bộ dựng vẽ
        // giống hệt các ý định khác thì người nhìn không phân biệt được gì.
        .child(Node::text_with(
            &chi_tiet,
            if matches!(
                c.scope,
                Scope::Wallet {
                    may_request_signature: true
                }
            ) {
                Emphasis::Warning
            } else {
                Emphasis::Normal
            },
        )?)?
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

    /// **B39 — dấu hiệu MÁY tách khỏi chữ NGƯỜI.**
    ///
    /// Chữ được dịch; mã hành động thì không bao giờ đổi. Trộn hai thứ ấy —
    /// chẳng hạn phái sinh mã từ nhãn — là để một bản dịch làm hỏng cái nút:
    /// người dùng tiếng Việt bấm "Cho phép" và khung nhận về một mã nó không
    /// biết, rồi từ chối im lặng.
    ///
    /// Phép thử giữ bất biến này nằm trong crate bị xoá 23/08/2026, và từ hôm ấy
    /// tới nay không gì canh nó.
    #[test]
    fn doi_ngon_ngu_khong_lam_doi_dau_hieu_may() {
        let m = ke_khai(XIN_VI_KY);
        let ma = |n: Language| {
            let cay = build(&m, n).unwrap();
            let ids: Vec<String> = cay
                .action_ids()
                .iter()
                .map(|a| a.as_str().to_owned())
                .collect();
            (ids, crate::do_cay::chu(&cay))
        };
        let (ma_en, chu_en) = ma(Language::En);
        let (ma_vi, chu_vi) = ma(Language::Vi);
        assert_eq!(ma_en, ma_vi, "đổi ngôn ngữ làm đổi MÃ HÀNH ĐỘNG");
        assert!(
            !ma_en.is_empty(),
            "màn hình không có mã hành động nào để so"
        );
        // Và chữ thì PHẢI đổi — nếu không, "đã dịch" chỉ là lời nói, và phép
        // thử trên xanh vì không có gì để so.
        assert_ne!(chu_en, chu_vi, "hai ngôn ngữ vẽ ra chữ giống hệt nhau");
    }

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

    /// Ví CHỈ ĐỌC ĐỊA CHỈ — hàng ví duy nhất bản dựng này còn cấp được.
    ///
    /// ⚑ 27/08/2026 ví thu về chỉ-đọc-địa-chỉ, nên mọi phép thử nói về *hàng ví
    /// CÓ CÔNG TẮC* phải dùng dữ liệu mẫu này. Dùng [`XIN_VI_KY`] thì hàng ấy
    /// dựng ra là **câu từ chối**, và phép thử đang kiểm một màn hình khác hẳn
    /// màn hình nó tưởng.
    const XIN_VI_DOC: &str = r#"[{"name":"wallet",
        "scope":{"kind":"wallet","may_request_signature":false},
        "reason":"hiện số dư của bạn"}]"#;

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
        // ⚑ ĐỔI 27/08/2026 — bất biến này tách làm HAI, và một nửa tạm thời
        // KHÔNG VỚI TỚI ĐƯỢC.
        //
        // Ví nay chỉ ĐỌC ĐỊA CHỈ; xin quyền ký giao dịch bị từ chối ở mọi bản
        // dựng. Nên hàng ví CÓ CÔNG TẮC — chỗ câu "việc này chuyển tiền" xuất
        // hiện — không còn được dựng ra ở đâu cả.
        //
        // Nửa CÒN KIỂM ĐƯỢC: câu chữ vẫn phải đúng, để ngày mở lại đường ký thì
        // nó không âm thầm biến mất trong lúc không ai nhìn.
        let en = label(TextKey::ViDuocXinChuKy, Language::En);
        let vi = label(TextKey::ViDuocXinChuKy, Language::Vi);
        assert!(en.contains("this moves money"), "{en}");
        assert!(vi.contains("chuyển tiền"), "{vi}");

        // Nửa KHÔNG VỚI TỚI: hàng ấy không dựng ra nữa. Khẳng định sự thật MỚI
        // thay vì bỏ phép thử đi — bỏ đi thì ngày mở lại đường ký, không ai
        // nhớ chỗ này từng được canh.
        if cfg!(feature = "wallet") {
            let s = chu_tren_man_hinh(&ke_khai(XIN_VI_KY), Language::En);
            assert!(
                !s.contains("this moves money"),
                "bản dựng này KHÔNG ký giao dịch, nên không được hỏi như thể có: {s}"
            );
            assert!(
                s.contains("does NOT sign transactions"),
                "phải nói rõ vì sao từ chối: {s}"
            );
        }
    }

    /// `04` §quyền ví: PHẢI hiện khác hẳn mọi quyền khác, và PHẢI nói bằng
    /// tiếng người rằng nó chuyển tiền.
    ///
    /// Trước phép thử này, hộp thoại nói đúng câu "việc này chuyển tiền" nhưng
    /// vẽ hàng ví Y HỆT hàng quyền mạng. Nửa sau của luật được giữ, nửa đầu thì
    /// không — và nửa đầu mới là nửa người dùng NHÌN thấy.
    #[test]
    fn quyen_vi_ky_duoc_hien_khac_han_quyen_khac() {
        let cay = |q: &str| build(&ke_khai(q), Language::En).unwrap();
        // ⚑ 27/08/2026: hàng ví CẤP ĐƯỢC nay là hàng CHỈ ĐỌC ĐỊA CHỈ. Hàng xin
        // ký giao dịch dựng ra câu từ chối, nên so nó với hàng mạng là so hai
        // thứ khác loại — phép thử vẫn xanh mà không còn kiểm điều nó định
        // kiểm: rằng hàng VÍ trông khác hẳn hàng quyền thường.
        let cay_vi = cay(XIN_VI_DOC);
        let cay_mang = cay(r#"[{"name":"network",
                    "scope":{"kind":"network","hosts":["shop.tcc-coin.com"]},
                    "reason":"tải danh sách"}]"#);
        let vi = crate::do_cay::chu(&cay_vi);
        let mang = crate::do_cay::chu(&cay_mang);
        // ⚠️ Vế KHẲNG ĐỊNH phải nói ĐÚNG HÀNG nào mang dấu — hàng quyền ví.
        // Hỏi "có dấu ở đâu đó không" thì chuyển dấu sang một dòng khác vẫn
        // xanh, và bất biến là "HÀNG NÀY khác hẳn", không phải "màn này có một
        // dấu nào đó".
        // Bất biến giữ nguyên ở CẢ HAI bản dựng: hàng ví phải KHÁC HẲN hàng
        // mạng. Chỉ câu mang dấu là khác — bản có ví nói "việc này chuyển
        // tiền", bản không ví nói "bản dựng này không có ví". Ghim CÂU, không
        // ghim "có dấu ở đâu đó" (B45).
        // ⚑ ĐỔI 27/08/2026 — bất biến này gắn với câu "việc này chuyển tiền",
        // và ĐỌC ĐỊA CHỈ thì không chuyển tiền. Đặc tả `04` §"Asking the user"
        // đòi: hỏi từng mục, mặc định tắt, nêu phạm vi CỤ THỂ, lý do nguyên
        // văn, cảnh báo danh tính, hai nút cân nhau. Nó KHÔNG đòi hàng ví phải
        // mang dấu cảnh báo — yêu cầu "khác hẳn" là bất biến của dự án, dựng
        // lên cho hàng KÝ GIAO DỊCH.
        //
        // Nên với hàng chỉ-đọc-địa-chỉ, thứ còn phải đúng là: nó nói rõ đây là
        // VÍ, và nói rõ phạm vi là CHỈ ĐỌC — không mượn giọng báo động của hàng
        // ký, cũng không giả vờ mình là một quyền tầm thường.
        let cho_doi = if cfg!(feature = "wallet") {
            label(TextKey::ViChiDocDiaChi, Language::En)
        } else {
            "This build has no wallet"
        };
        assert!(vi.contains(cho_doi), "thiếu câu {cho_doi:?}:\n{vi}");
        assert!(
            vi.contains(label(TextKey::QuyenVi, Language::En)),
            "hàng ví không tự xưng là ví:\n{vi}"
        );
        assert!(
            !vi.contains("this moves money"),
            "hàng CHỈ ĐỌC ĐỊA CHỈ mượn giọng của hàng ký giao dịch:\n{vi}"
        );
        assert!(
            !mang.contains("[cảnh-báo]"),
            "hàng quyền mạng mang dấu cảnh báo — dấu ấy dành cho chỗ khác"
        );
    }

    #[test]
    fn vi_chi_doc_khong_bi_ghi_nham_thanh_ky_duoc() {
        let chi_doc = r#"[{"name":"wallet",
            "scope":{"kind":"wallet","may_request_signature":false},
            "reason":"hiện số dư"}]"#;
        if !cfg!(feature = "wallet") {
            return;
        }
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
        let cay = build(&ke_khai(XIN_VI_DOC), Language::En).unwrap();
        let ds: Vec<&str> = cay.action_ids().iter().map(|a| a.as_str()).collect();
        assert!(
            ds.contains(&ACTION_ALLOW),
            "hằng số cho phép \"{ACTION_ALLOW}\" không có trên cây: {ds:?}"
        );
        assert!(
            ds.contains(&ACTION_DENY),
            "hằng số từ chối \"{ACTION_DENY}\" không có trên cây: {ds:?}"
        );
        // Mỗi quyền CẤP ĐƯỢC phải có một công tắc — thiếu thì người dùng không
        // có cách nào bật, và quyền đó vĩnh viễn không cấp được. Quyền bản dựng
        // KHÔNG cấp được thì ngược lại: có công tắc mới là sai.
        if cfg!(feature = "wallet") {
            assert!(
                ds.contains(&"q-wallet"),
                "thiếu công tắc cho quyền ví: {ds:?}"
            );
            assert_eq!(ds.len(), 3, "số mã hành động không như mong đợi: {ds:?}");
        } else {
            assert!(
                !ds.contains(&"q-wallet"),
                "bản dựng không ví mà vẫn có công tắc ví: {ds:?}"
            );
            assert_eq!(ds.len(), 2, "số mã hành động không như mong đợi: {ds:?}");
        }
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
            {"name":"wallet","scope":{"kind":"wallet","may_request_signature":false},"reason":"z"}
        ]"#;
        let cay = build(&ke_khai(nhieu), Language::En).unwrap();
        let mut ds = Vec::new();
        gom_cong_tac(&cay, &mut ds);
        // Hàng ví chỉ có công tắc khi bản dựng cấp được quyền ấy.
        let cho_doi = if cfg!(feature = "wallet") { 3 } else { 2 };
        assert_eq!(ds.len(), cho_doi, "thiếu công tắc cho một quyền nào đó");
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
        let cay = build(&ke_khai(XIN_VI_KY), Language::En).unwrap();
        let mut bd = tcc_render_raster::RasterRenderer::new();
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

    /// **Bản dựng KHÔNG có ví thì không được HỎI về ví.**
    ///
    /// Hỏi một câu mà không cấp được câu trả lời là hộp thoại nói dối: người
    /// dùng cân nhắc một quyết định về tiền, gạt công tắc, và không có gì xảy
    /// ra — hoặc tệ hơn, họ tin là có.
    ///
    /// Lộ ra 26/08/2026 khi ví chuyển thành việc làm SAU và bản dựng không ví
    /// thành bản chính: nó vẫn hiện "cho phép chạm vào ví — việc này chuyển
    /// tiền", kèm một công tắc gạt được.
    #[test]
    fn ban_dung_khong_co_vi_thi_khong_hoi_ve_vi() {
        let cay = build(&ke_khai(XIN_VI_DOC), Language::En).unwrap();
        let s = crate::do_cay::chu(&cay);

        if cfg!(feature = "wallet") {
            assert!(
                s.contains("công tắc[") || s.contains("switch["),
                "bản dựng CÓ ví mà không hỏi:\n{s}"
            );
        } else {
            assert!(
                crate::do_cay::co_canh_bao(&cay, "This build has no wallet"),
                "bản dựng KHÔNG ví mà không nói ra:\n{s}"
            );
            // Và tuyệt đối không có công tắc nào cho hàng ví.
            let hang_vi = s
                .lines()
                .skip_while(|d| !d.contains("Access your TCC wallet"))
                .take(3)
                .collect::<Vec<_>>()
                .join("\n");
            assert!(
                !hang_vi.contains("công tắc[") && !hang_vi.contains("switch["),
                "bản dựng không ví mà hàng ví VẪN có công tắc gạt được:\n{hang_vi}"
            );
        }
    }

    /// **Không cấp được thì đường CẤP cũng phải từ chối, không chỉ hộp thoại.**
    ///
    /// Hộp thoại không phải đường duy nhất tới quyết định: `.tcc-quyen.json`
    /// ghi từ một bản dựng CÓ ví mang theo câu "đã đồng ý", và trục trợ năng là
    /// một lối vào khác. Một câu trả lời do bản dựng KHÁC ghi lại không phải
    /// câu trả lời cho bản dựng này.
    #[test]
    fn cap_duoc_noi_dung_su_that_ve_ban_dung() {
        // ⚑ Đổi 27/08/2026: ví giữ khoá và CHỈ ĐỌC ĐỊA CHỈ. Xin quyền KÝ GIAO
        // DỊCH thì KHÔNG bản dựng nào cấp — kể cả bản có cờ `wallet`. Bản trước
        // của phép thử này khẳng định `true` cấp được khi có cờ; nó ghi lại
        // ngữ nghĩa CŨ.
        assert!(
            !cap_duoc(&Scope::Wallet {
                may_request_signature: true
            }),
            "xin quyền KÝ GIAO DỊCH phải bị từ chối ở MỌI bản dựng — ký chỉ mở \
             sau kiểm định an ninh độc lập"
        );
        // Còn chỉ ĐỌC ĐỊA CHỈ thì cấp được, nếu bản dựng có ví.
        assert_eq!(
            cap_duoc(&Scope::Wallet {
                may_request_signature: false
            }),
            cfg!(feature = "wallet"),
            "đọc địa chỉ phải cấp được đúng khi bản dựng có ví"
        );
        assert!(cap_duoc(&Scope::Network {
            hosts: vec!["a.example".to_owned()]
        }));
        assert!(cap_duoc(&Scope::Storage { quota_bytes: 0 }));
    }
}
