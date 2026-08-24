//! **Mọi màn hình THẬT, vẽ qua bộ dựng thật.**
//!
//! # Vì sao tệp này tồn tại
//!
//! Hai bất biến hình học — không ô nào chồng ô nào, không ô nào trôi ra ngoài
//! ảnh — đã có phép thử trong `tcc-render-raster`, và chúng chạy trên cây SINH
//! NGẪU NHIÊN. Đó là chỗ mạnh của chúng: hạt ngẫu nhiên tìm ra hình dạng không
//! ai nghĩ tới.
//!
//! Nó cũng là chỗ yếu: cây ngẫu nhiên không phải màn hình người dùng nhìn thấy.
//! Ngày 22/08/2026 cả bộ tính bố cục bị thay, và suốt từ đó **không phép thử
//! nào vẽ hộp thoại hỏi quyền hay màn cụm từ khôi phục** — những màn hình mà một
//! lỗi bố cục ở đó là một nút bấm nhầm đúng lúc nguy hiểm nhất.
//!
//! Phép thử này đi ngược lại: ít cây, nhưng là ĐÚNG những cây sẽ hiện ra.
//!
//! # Nó KHÔNG thay được phép thử ngẫu nhiên, và đây là bằng chứng
//!
//! Kiểm đột biến ngày 24/08/2026, ba lần:
//!
//! | đột biến | bắt được? |
//! |---|---|
//! | lề âm (`LE = -20`) → ô ra ngoài ảnh | ✅ |
//! | bỏ xuống dòng (`FlexWrap::NoWrap`) | ❌ |
//! | kéo nút bằng nhau VÔ ĐIỀU KIỆN — chính lỗi F1 | ❌ |
//!
//! Hai cái không bắt được có cùng một lý do: những màn hình này xếp CỘT, và
//! hàng nút duy nhất — màn xác nhận giao dịch — có hai nút vừa thoải mái trong
//! khung. Ca biên của F1 cần một nhãn nút dài, và không màn hình thật nào có.
//!
//! Đó là phân công đúng, không phải thiếu sót: cây ngẫu nhiên tìm ca BIÊN, tệp
//! này canh cây THẬT. Bịa thêm một màn hình có nút dài chỉ để chạm ca biên là
//! biến tệp này thành một bản sao tồi của phép thử ngẫu nhiên.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay"
)]

use tcc_chain::{Address, Transfer};
use tcc_render_raster::RasterRenderer;
use tcc_shell::{
    Language, external_link, permission_dialog, permission_screen, recovery_screen,
    transaction_screen,
};
use tcc_ui::{AccessNode, Node, Renderer as _};

/// Bản kê khai mẫu — xin đủ loại quyền để hộp thoại có gì mà vẽ.
fn ke_khai() -> tcc_spec::Manifest {
    let s = format!(
        r#"{{"spec_version":"0.1","id":"com.tcc.hello","name":"Vi TCC",
"version":"1.0.0","publisher":"{}","scheme":"hybrid-ed25519-mldsa65-v1",
"content_hash":"{}","entry":"ui.json","capabilities":[
{{"name":"network","scope":{{"kind":"network","hosts":["shop.tcc-coin.com"]}},
  "reason":"tai danh sach san pham"}},
{{"name":"wallet","scope":{{"kind":"wallet","may_request_signature":true}},
  "reason":"thanh toan don hang"}}]}}"#,
        "aa".repeat(1992),
        "bb".repeat(48)
    );
    serde_json::from_str(&s).expect("ban ke khai mau hong")
}

/// Mọi màn hình dựng được mà không cần cờ ví.
fn man_hinh(n: Language) -> Vec<(&'static str, Node)> {
    let m = ke_khai();
    vec![
        (
            "permission_dialog::build",
            permission_dialog::build(&m, n).unwrap(),
        ),
        (
            "permission_screen::build",
            permission_screen::build(&[], n).unwrap(),
        ),
        (
            "external_link::build_confirm",
            external_link::build_confirm("https://ngan-hang.example.com/dang-nhap?a=1&b=2", n)
                .unwrap(),
        ),
        (
            "recovery_screen::build_entry",
            recovery_screen::build_entry(None, n).unwrap(),
        ),
        (
            "recovery_screen::build_entry (co loi)",
            recovery_screen::build_entry(Some("cum tu khong hop le"), n).unwrap(),
        ),
        (
            "recovery_screen::build_session_entry",
            recovery_screen::build_session_entry(None, n).unwrap(),
        ),
        (
            "recovery_screen::build_confirm",
            recovery_screen::build_confirm("0x1111111111111111111111111111111111111111", n)
                .unwrap(),
        ),
        (
            "recovery_screen::build_failure",
            recovery_screen::build_failure("kho khoa tu choi", n).unwrap(),
        ),
        // ⚠️ Màn XÁC NHẬN GIAO DỊCH: hai nút CÙNG HÀNG, tức là ca F1 —
        // chỗ duy nhất trong danh sách này mà luật "nút cùng dòng rộng bằng
        // nhau" có tác dụng, và chỗ một lỗi bố cục là một cú bấm nhầm lúc
        // chuyển tiền.
        (
            "transaction_screen::build",
            transaction_screen::build(&chuyen_tien(), &chuyen_tien().signing_message(), n).unwrap(),
        ),
        (
            "transaction_screen::build_sent",
            transaction_screen::build_sent(
                "0xc06d6191c039ece24cc87ff8d4b4dae82257f657bbaf32c48e473c5c38017ade",
                n,
            )
            .unwrap(),
        ),
    ]
}

/// Một giao dịch mẫu — cùng số liệu với phép thử trong `transaction_screen`.
fn chuyen_tien() -> Transfer {
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

/// Hai ô có chồng lên nhau không.
fn chong(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)) -> bool {
    let (ax, ay, aw, ah) = a;
    let (bx, by, bw, bh) = b;
    ax < bx + bw && bx < ax + aw && ay < by + bh && by < ay + ah
}

fn hanh_dong(n: &AccessNode, ra: &mut Vec<String>) {
    if let Some(a) = &n.action {
        ra.push(a.clone());
    }
    for c in &n.children {
        hanh_dong(c, ra);
    }
}

/// **Không màn hình thật nào có ô chồng ô, ô trôi ra ngoài, hay ô câm.**
#[test]
fn moi_man_hinh_that_ve_ra_hop_le() {
    for n in [Language::En, Language::Vi] {
        for (ten, cay) in man_hinh(n) {
            let mut bd = RasterRenderer::new();
            bd.render(&cay)
                .unwrap_or_else(|e| panic!("{ten} ({n:?}) khong ve duoc: {e}"));
            let hop = bd.placed_boxes();
            assert!(!hop.is_empty(), "{ten} ({n:?}) khong ve ra o nao");

            for (i, a) in hop.iter().enumerate() {
                for b in hop.iter().skip(i + 1) {
                    assert!(!chong(*a, *b), "{ten} ({n:?}): {a:?} chong {b:?}");
                }
            }

            #[allow(clippy::cast_precision_loss, reason = "kich thuoc anh, luon nho")]
            let (rong_anh, cao_anh) = (tcc_render_raster::WIDTH as f32, bd.height() as f32);
            for (x, y, w, h) in &hop {
                assert!(
                    *x >= 0.0 && *y >= 0.0 && x + w <= rong_anh && y + h <= cao_anh,
                    "{ten} ({n:?}): o {x},{y} {w}x{h} nam ngoai anh {rong_anh}x{cao_anh}"
                );
            }

            assert!(bd.ink() > 0, "{ten} ({n:?}) khong co net nao duoc ve");

            let mut doc = Vec::new();
            hanh_dong(&bd.published_accessibility().expect("da ve"), &mut doc);
            doc.sort();
            assert!(!doc.is_empty(), "{ten} ({n:?}) khong co hanh dong nao");
        }
    }
}
