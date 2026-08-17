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

        let cay = tcc_ui::Node::button("Xoá hết", "xoa", tcc_ui::Tone::Danger).unwrap();
        let mut bd = RasterRenderer::new();
        bd.render(&cay).unwrap();
        let cap_nhat = to_accesskit(
            &bd.published_accessibility().unwrap(),
            &AccessText {
                cau_mat_mat: cau.clone(),
            },
        );
        let nut = &cap_nhat.nodes.first().unwrap().1;
        assert_eq!(nut.description(), Some(cau.as_str()));

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
