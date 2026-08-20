//! Mọi màn hình của khung phải qua được **CẢ HAI** bộ dựng.
//!
//! # Vì sao tệp này tồn tại
//!
//! Trước 17/08/2026 chỉ có một bộ dựng, nên câu *"`tcc-ui` không biết bộ dựng
//! nào"* là một luật không ai kiểm được: mọi giả định ngầm về HTML đều nằm im
//! vì không có ai đá vào.
//!
//! Và WebView **không chạy được trong `cargo test`** — trên macOS vòng lặp sự
//! kiện phải ở luồng chính còn bộ khung test chạy ở luồng phụ. Nên phép kiểm
//! ngang bằng trợ năng của các màn hình chỉ chạy trong ví dụ có người bấm.
//!
//! Bộ dựng ra pixel chạy trong một phép thử bình thường. Từ đây, mọi màn hình
//! được kiểm trợ năng **ở CI, trên cả ba nền**, mỗi lần đẩy.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay"
)]

use tcc_render_raster::RasterRenderer;
use tcc_shell::text::Language;
use tcc_ui::Renderer as _;

/// Cả hai bộ dựng phải công bố **cùng một cây trợ năng** cho cùng một màn hình.
///
/// Đây là phép kiểm đắt nhất tệp này: nếu hai bộ dựng nói hai điều khác nhau
/// với trình đọc màn hình thì ít nhất một cái đang nói dối, và cho tới hôm nay
/// không có cách nào biết.
fn hai_bo_dung_noi_cung_mot_dieu(cay: &tcc_ui::Node) {
    let mut raster = RasterRenderer::new();
    tcc_ui::check_accessibility_parity(&mut raster, cay).expect("bộ dựng pixel");

    let mut web = tcc_render_webview::WebViewRenderer::new();
    tcc_ui::check_accessibility_parity(&mut web, cay).expect("bộ dựng WebView");

    assert_eq!(
        raster.published_accessibility(),
        web.published_accessibility(),
        "hai bộ dựng công bố hai cây trợ năng KHÁC nhau"
    );
}

/// **Cây phủ HẾT mọi loại nút**, kể cả những loại màn hình thật chưa dùng tới.
///
/// Kiểm đột biến tìm ra chỗ này: bỏ cờ "không hoàn tác" của nút nguy hiểm thì
/// hai phép thử màn hình thật vẫn XANH — vì màn xác nhận giao dịch cố ý dùng
/// hai nút cùng sắc thái, và màn quản lý quyền rỗng thì không có nút nào.
///
/// Một phép kiểm chéo chỉ chạy trên những màn hình đang có là một phép kiểm
/// chéo che đúng chỗ chưa ai đi qua.
#[test]
fn phu_het_moi_loai_nut() {
    use tcc_ui::{Alt, Emphasis, Flow, Gap, Node, Tone};

    let cay = (|| -> Result<Node, tcc_ui::UiError> {
        Node::group(Flow::Column, Gap::Medium)
            .child(Node::text_with("Tiêu đề", Emphasis::Title)?)?
            .child(Node::text_with("Câu cảnh báo", Emphasis::Warning)?)?
            // Nút KHÔNG HOÀN TÁC — loại mà hai màn hình thật không có.
            .child(Node::button("Xoá hết", "xoa", Tone::Danger)?)?
            .child(Node::button("Huỷ", "huy", Tone::Neutral)?)?
            .child(Node::field("Mã PIN", "", true)?)?
            .child(Node::field("Tìm", "xin chào", false)?)?
            .child(Node::toggle("Cho phép mạng", true, "mang")?)?
            .child(Node::image(
                "anh/logo.png",
                Alt::Text("Biểu trưng".to_owned()),
            )?)?
            .child(Node::image("anh/vien.png", Alt::Decorative)?)
    })()
    .expect("dựng cây phủ hết");
    hai_bo_dung_noi_cung_mot_dieu(&cay);
}

#[test]
fn man_xac_nhan_giao_dich() {
    let tx = tcc_chain::Transfer {
        version: 1,
        chain_id: 91338,
        from: tcc_chain::Address([0x11; 32]),
        to: tcc_chain::Address([0x22; 32]),
        nonce: 0,
        amount: 5_000_000_000_000_000_000,
        gas_price: 47_619_047_620,
        gas_limit: 21_000,
        timestamp: 0,
        expires_at: 162_486,
        memo: "chao".to_owned(),
    };
    let bam = tx.signing_message();
    for ngon_ngu in [Language::En, Language::Vi] {
        let cay =
            tcc_shell::transaction_screen::build(&tx, &bam, ngon_ngu).expect("dựng màn xác nhận");
        hai_bo_dung_noi_cung_mot_dieu(&cay);
    }
}

#[test]
fn man_quan_ly_quyen() {
    for ngon_ngu in [Language::En, Language::Vi] {
        let cay = tcc_shell::permission_screen::build(&[], ngon_ngu).expect("dựng màn quản lý");
        hai_bo_dung_noi_cung_mot_dieu(&cay);
    }
}

/// Màn hình vẽ ra phải có MỰC — một cây trợ năng đúng trên một ảnh trắng trơn
/// vẫn là một màn hình hỏng, và cây trợ năng không nói được điều đó.
#[test]
fn man_hinh_ve_ra_co_muc_that() {
    let cay = tcc_shell::permission_screen::build(&[], Language::Vi).expect("dựng màn quản lý");
    let mut bd = RasterRenderer::new();
    bd.render(&cay).expect("vẽ được");
    assert!(bd.ink() > 200, "màn hình gần như trắng trơn: {}", bd.ink());
}

/// **Hai bộ dựng phải đọc CÙNG MỘT CÂU cho nút không hoàn tác được.**
///
/// AccessKit không có vai trò riêng cho "không hoàn tác", nên câu ấy đi vào
/// `description`. WebView đưa nó qua `aria-description`. Hai đường khác nhau,
/// một câu — và câu ấy phải là câu `text.rs` dịch, không phải câu bộ dựng nào
/// tự bịa.
#[cfg(feature = "accesskit")]
#[test]
fn hai_bo_dung_doc_cung_cau_mat_mat() {
    use tcc_render_raster::accesskit_bridge::{AccessText, to_accesskit};
    use tcc_shell::text::{TextKey, label};

    for ngon_ngu in [Language::En, Language::Vi] {
        let cau = label(TextKey::CauMatMat, ngon_ngu).to_owned();
        let vai_tro = label(TextKey::VaiTroMatMat, ngon_ngu).to_owned();

        let cay = tcc_ui::Node::button("Xoá hết", "xoa", tcc_ui::Tone::Danger).unwrap();
        let mut bd = RasterRenderer::new();
        bd.render(&cay).unwrap();
        let cap_nhat = to_accesskit(
            &bd.published_accessibility().unwrap(),
            &AccessText {
                cau_mat_mat: cau.clone(),
                vai_tro_mat_mat: vai_tro.clone(),
            },
        );
        let nut = &cap_nhat.nodes.first().unwrap().1;
        assert_eq!(nut.description(), Some(cau.as_str()));
        // VAI TRÒ cũng phải khớp: WebView đặt `aria-roledescription`, và bộ dựng
        // ra pixel phải nói đúng chuỗi ấy — nếu không, một nút xoá nghe y hệt
        // một nút huỷ trên đúng bộ dựng ta định dùng để thay thế WebView.
        assert_eq!(nut.role_description(), Some(vai_tro.as_str()));

        // Và WebView phải nhả ra ĐÚNG chuỗi ấy trong tài liệu của nó.
        let mut web = tcc_render_webview::WebViewRenderer::new()
            .with_text(tcc_shell::text::renderer_text(ngon_ngu));
        web.render(&cay).unwrap();
        assert!(
            web.body().contains(&cau),
            "WebView không đọc câu mất mát ({ngon_ngu:?}):\n{}",
            web.body()
        );
    }
}

/// **Hai nút cạnh nhau phải rộng BẰNG NHAU — ở CẢ HAI bộ dựng.**
///
/// Cùng luật với "hai nút cùng sắc thái" của màn xác nhận giao dịch: một nút
/// rộng gấp ba nút kia vẫn là một cái hích, chỉ bằng hình học thay vì bằng màu.
/// Và ở màn ấy, cái hích đẩy về phía KÝ.
///
/// Bộ dựng pixel kéo bằng trong bố cục; WebView đánh dấu hàng rồi để CSS kéo.
/// Hai đường khác nhau, một tính chất — nên phải kiểm cả hai, không kiểm một.
#[test]
fn hai_nut_canh_nhau_khong_hich_nguoi_dung() {
    use tcc_ui::{Flow, Gap, Node, Tone, UiError};

    let cay = (|| -> Result<Node, UiError> {
        Node::group(Flow::Row, Gap::Medium)
            .child(Node::button("Ký giao dịch này", "ky", Tone::Neutral)?)?
            .child(Node::button("Huỷ", "huy", Tone::Neutral)?)
    })()
    .expect("dựng hàng hai nút");

    let mut web = tcc_render_webview::WebViewRenderer::new();
    web.render(&cay).unwrap();
    let than = web.body().to_owned();
    assert!(
        than.contains("data-hang-nut"),
        "WebView không đánh dấu hàng toàn nút:\n{than}"
    );
    let dinh_kieu = tcc_render_webview::markup::document(&cay);
    assert!(
        dinh_kieu.contains("[data-hang-nut]>button{flex:1 1 0"),
        "thiếu luật CSS kéo hai nút bằng nhau"
    );

    // Và hàng KHÔNG toàn nút thì không đánh dấu.
    let lan = (|| -> Result<Node, UiError> {
        Node::group(Flow::Row, Gap::Medium)
            .child(Node::text("Nhãn")?)?
            .child(Node::button("OK", "ok", Tone::Neutral)?)
    })()
    .expect("dựng hàng lẫn");
    let mut web2 = tcc_render_webview::WebViewRenderer::new();
    web2.render(&lan).unwrap();
    assert!(
        !web2.body().contains("data-hang-nut"),
        "hàng lẫn nhãn bị đánh dấu là hàng nút"
    );
}

// ─────────────────── Phủ HẾT màn hình, không phủ vài cái ───────────────────

/// Bản kê khai mẫu, có một quyền để hộp thoại hỏi quyền không rỗng.
fn ke_khai_mau() -> tcc_spec::Manifest {
    serde_json::from_str(&format!(
        r#"{{"spec_version":"0.1","id":"com.tcc.vi-du","name":"Ứng dụng mẫu","version":"1",
"publisher":"{}","scheme":"hybrid-ed25519-mldsa65-v1","content_hash":"{}",
"entry":"ui.json","capabilities":[
  {{"name":"network","reason":"Tải một trang mẫu","scope":{{"kind":"network","hosts":["a.example"]}}}}
]}}"#,
        "aa".repeat(1992),
        "bb".repeat(48)
    ))
    .expect("bản kê khai mẫu hỏng")
}

/// **MỌI màn hình của khung, trên CẢ HAI bộ dựng, trong CẢ HAI ngôn ngữ.**
///
/// # Vì sao phải liệt kê hết thay vì kiểm vài cái
///
/// Trước 20/08/2026 tệp này kiểm ba màn hình. Mười một màn hình còn lại chưa
/// từng chạy qua bộ dựng thứ hai — trong đó có màn nhập ví, màn khôi phục cụm
/// từ, và màn hỏi quyền. Nghĩa là câu *"ứng dụng chạy trên cả hai bộ dựng"* đúng
/// với những màn hình ai đó nhớ ra, không đúng với những màn hình còn lại.
///
/// Bốn lỗi tìm được ngày 19/08 đều thuộc dạng *"chính sách chỉ đúng trên một bộ
/// dựng"*. Phép kiểm chéo chỉ chạy trên vài màn hình thì đúng là chỗ dạng lỗi ấy
/// nấp được.
#[test]
fn moi_man_hinh_qua_duoc_ca_hai_bo_dung() {
    let m = ke_khai_mau();
    for ngon_ngu in [Language::En, Language::Vi] {
        let man: Vec<(&str, tcc_ui::Node)> = vec![
            (
                "permission_dialog::build",
                tcc_shell::permission_dialog::build(&m, ngon_ngu).unwrap(),
            ),
            (
                "permission_screen::build",
                tcc_shell::permission_screen::build(&[], ngon_ngu).unwrap(),
            ),
            (
                "address_bar::build",
                tcc_shell::address_bar::build("https://a.example", ngon_ngu).unwrap(),
            ),
            (
                "external_link::build_confirm",
                tcc_shell::external_link::build_confirm("https://a.example/x", ngon_ngu).unwrap(),
            ),
            (
                "recovery_screen::build_entry",
                tcc_shell::recovery_screen::build_entry(None, ngon_ngu).unwrap(),
            ),
            (
                "recovery_screen::build_entry (có lỗi)",
                tcc_shell::recovery_screen::build_entry(Some("sai cụm từ"), ngon_ngu).unwrap(),
            ),
            (
                "recovery_screen::build_session_entry",
                tcc_shell::recovery_screen::build_session_entry(None, ngon_ngu).unwrap(),
            ),
            (
                "recovery_screen::build_confirm",
                tcc_shell::recovery_screen::build_confirm("0xabc", ngon_ngu).unwrap(),
            ),
            (
                "recovery_screen::build_failure",
                tcc_shell::recovery_screen::build_failure("hỏng", ngon_ngu).unwrap(),
            ),
            (
                "transaction_screen::build_sent",
                tcc_shell::transaction_screen::build_sent("0xdeadbeef", ngon_ngu).unwrap(),
            ),
        ];
        // `import_screen` nằm sau cờ `import-web-wallet`. Thêm vào cùng danh
        // sách chứ không thành một phép thử riêng: một màn hình sau cờ vẫn là
        // một màn hình người dùng nhìn thấy.
        #[cfg(feature = "import-web-wallet")]
        let man = {
            let mut man = man;
            man.push((
                "import_screen::build_choice",
                tcc_shell::import_screen::build_choice(&[], ngon_ngu).unwrap(),
            ));
            man.push((
                "import_screen::build_pin",
                tcc_shell::import_screen::build_pin("0xabc", ngon_ngu).unwrap(),
            ));
            man
        };
        for (ten, cay) in man {
            let mut raster = RasterRenderer::new();
            tcc_ui::check_accessibility_parity(&mut raster, &cay)
                .unwrap_or_else(|e| panic!("{ten} ({ngon_ngu:?}) trượt trên bộ dựng pixel: {e:?}"));
            let mut web = tcc_render_webview::WebViewRenderer::new();
            tcc_ui::check_accessibility_parity(&mut web, &cay)
                .unwrap_or_else(|e| panic!("{ten} ({ngon_ngu:?}) trượt trên WebView: {e:?}"));
            assert_eq!(
                raster.published_accessibility(),
                web.published_accessibility(),
                "{ten} ({ngon_ngu:?}): hai bộ dựng công bố hai cây trợ năng KHÁC nhau"
            );
            // Và màn hình phải có MỰC: một cây trợ năng đúng trên một ảnh trắng
            // trơn vẫn là màn hình hỏng, mà cây trợ năng không nói được điều đó.
            assert!(
                raster.ink() > 100,
                "{ten} ({ngon_ngu:?}) gần như trắng trơn"
            );
        }
    }
}

/// **Danh sách trên KHÔNG được trôi khỏi số màn hình thật có.**
///
/// Một phép kiểm "phủ hết" mà danh sách viết tay thì nó phủ hết cho tới lúc ai
/// đó thêm một màn hình. Phép thử này đọc mã nguồn của khung và đếm.
#[test]
fn khong_bo_sot_man_hinh_nao() {
    let mut thay: Vec<String> = Vec::new();
    for tep in std::fs::read_dir("src").expect("đọc được src/") {
        let tep = tep.expect("mục hỏng").path();
        if tep.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let ten_tep = tep.file_stem().unwrap().to_string_lossy().to_string();
        let noi_dung = std::fs::read_to_string(&tep).expect("đọc được tệp");
        let phan_that = noi_dung.split("#[cfg(test)]").next().unwrap_or(&noi_dung);
        for dong in phan_that.lines() {
            let d = dong.trim_start();
            if let Some(sau) = d.strip_prefix("pub fn build") {
                // ⚠️ GIỮ dấu gạch dưới. Bản đầu `trim_start_matches('_')` và
                // sinh ra `buildwith_signer` — không khớp tên nào trong danh
                // sách, nên phép thử báo "bỏ sót" đúng những màn hình ĐÃ phủ.
                let ten_ham = sau.split(['(', '<']).next().unwrap_or_default();
                thay.push(format!("{ten_tep}::build{ten_ham}"));
            }
        }
    }

    // Những màn hình phép thử trên KHÔNG dựng được, kèm lý do. Danh sách này
    // phải NGẮN và mỗi dòng phải nói được vì sao — nó là chỗ "phủ hết" tự nới
    // lỏng nếu không ai canh.
    let mien: &[(&str, &str)] = &[
        (
            "permission_dialog::build_with_signer",
            "biến thể của `build`, khác đúng một dòng cảnh báo đổi khoá",
        ),
        (
            "import_screen::build_done",
            "cần một `ImportedWallet`, tức là một khoá bí mật thật — dựng một \
             khoá thật chỉ để vẽ một màn hình là đúng thứ không nên có trong \
             phép thử",
        ),
        (
            "transaction_screen::build",
            "đã có phép thử riêng `man_xac_nhan_giao_dich`, vì nó cần một \
             `Transfer` và bản tin ký",
        ),
    ];

    // Màn hình sau cờ `import-web-wallet` chỉ kiểm được khi bật cờ ấy. Không
    // coi là "đã phủ" khi cờ tắt — nếu không, phép thử này lại đúng vì lý do
    // sai, y hệt thứ nó sinh ra để chặn.
    #[cfg(not(feature = "import-web-wallet"))]
    let mien: Vec<(&str, &str)> = mien
        .iter()
        .copied()
        .chain([
            ("import_screen::build_choice", "cần cờ `import-web-wallet`"),
            ("import_screen::build_pin", "cần cờ `import-web-wallet`"),
        ])
        .collect();
    #[cfg(not(feature = "import-web-wallet"))]
    let mien: &[(&str, &str)] = &mien;

    let da_phu = [
        "permission_dialog::build",
        "permission_screen::build",
        "address_bar::build",
        "external_link::build_confirm",
        "recovery_screen::build_entry",
        "recovery_screen::build_session_entry",
        "recovery_screen::build_confirm",
        "recovery_screen::build_failure",
        "transaction_screen::build_sent",
    ];
    #[cfg(feature = "import-web-wallet")]
    let da_phu = {
        let mut v = da_phu.to_vec();
        v.push("import_screen::build_choice");
        v.push("import_screen::build_pin");
        v
    };

    let bo_sot: Vec<&String> = thay
        .iter()
        .filter(|t| !da_phu.contains(&t.as_str()) && !mien.iter().any(|(m, _)| m == &t.as_str()))
        .collect();
    assert!(
        bo_sot.is_empty(),
        "màn hình chưa qua phép kiểm chéo hai bộ dựng: {bo_sot:?}"
    );
}
