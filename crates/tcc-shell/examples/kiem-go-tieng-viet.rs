//! Cổng ra Giai đoạn 1: **gõ tiếng Việt có dấu bằng bộ gõ hệ thống**.
//!
//! Chạy: `cargo run -p tcc-shell --features window --example kiem-go-tieng-viet <gói> [giây]`
//!
//! # Vì sao phép kiểm này KHÔNG tự động được
//!
//! Bộ gõ là của hệ điều hành. Máy không gõ thay người: mọi cách giả lập đều
//! bơm thẳng chuỗi đã hoàn chỉnh vào ô nhập, tức là bỏ qua đúng cái đang cần
//! đo — phiên ghép của bộ gõ. Nên phép kiểm này cần một người ngồi trước máy,
//! và đó là lý do nó không nằm trong CI.
//!
//! # Vì sao KHÔNG nhìn bằng mắt là đủ
//!
//! Nhìn thấy `ỡ` không cho biết nó là MỘT mã điểm dựng sẵn (U+1EE1) hay là `o`
//! cộng hai dấu rời. Hai dạng đó hiện ra y hệt nhau mà đi qua phép kiểm khác
//! hẳn nhau: trần dấu chồng `MAX_COMBINING_MARKS` đếm theo mã điểm, nên một bộ
//! gõ cho ra dạng tách rời sẽ tiêu tốn trần nhanh gấp ba. Chuyện đó không thấy
//! được bằng mắt, chỉ thấy bằng cách hỏi lại WebKit.

#![allow(
    clippy::expect_used,
    clippy::print_stdout,
    reason = "công cụ chẩn đoán có người ngồi xem: hỏng thì nổ ngay, và nó phải in ra"
)]

use std::{path::PathBuf, process::ExitCode, time::Duration};

use tcc_capability::Decision;
use tcc_crypto::HybridEd25519MlDsa;
use tcc_render_webview::{WebViewRenderer, window::probe_text_input};
use tcc_shell::text::Language;
use tcc_ui::Renderer as _;
use unicode_general_category::{GeneralCategory, get_general_category};

fn la_dau(c: char) -> bool {
    matches!(
        get_general_category(c),
        GeneralCategory::NonspacingMark
            | GeneralCategory::SpacingMark
            | GeneralCategory::EnclosingMark
    )
}

fn main() -> ExitCode {
    let Some(goi) = std::env::args().nth(1).map(PathBuf::from) else {
        eprintln!("cần đường dẫn thư mục gói");
        return ExitCode::FAILURE;
    };
    let giay: u64 = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(45);

    let (app, noi_dung) =
        tcc_runtime::verify_from_dir(&goi, &HybridEd25519MlDsa).expect("kiểm chữ ký hỏng");
    let app = tcc_runtime::grant_verified(app, noi_dung, |_| Decision::Deny)
        .expect("dựng tập quyền hỏng");

    let cay = tcc_ui::wire::decode(app.entry_content()).expect("điểm vào không đọc được");
    let mut bo_dung =
        WebViewRenderer::new().with_text(tcc_shell::text::renderer_text(Language::Vi));
    bo_dung.render(&cay).expect("bộ dựng vẽ hỏng");

    println!("Cửa sổ sắp mở. Bật bộ gõ tiếng Việt rồi gõ vào ô \"Gõ thử tiếng Việt\".");
    println!("Gợi ý gõ: chữ có dấu chồng hai tầng, ví dụ  nhưng  ·  chuỗi  ·  ưỡn");
    println!("Gõ xong thì ĐÓNG cửa sổ (hoặc chờ {giay} giây).\n");

    let tep = app.copy_content();
    let bao_cao = match probe_text_input(
        bo_dung.document(),
        move |p| tep.get(p).map(<[u8]>::to_vec),
        Duration::from_secs(giay),
    ) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("✗ {e}");
            return ExitCode::FAILURE;
        }
    };

    phan_tich(&bao_cao)
}

/// Tách riêng vì phần đo được thì nên đọc được một mình, không lẫn với phần
/// dựng cửa sổ.
fn phan_tich(bao_cao: &tcc_render_webview::window::TextInputProbe) -> ExitCode {
    let v = &bao_cao.value;
    let ma_diem: Vec<char> = v.chars().collect();
    let so_dau = ma_diem.iter().copied().filter(|c| la_dau(*c)).count();
    let so_goc = ma_diem.len() - so_dau;
    let mut lien_tiep = 0usize;
    let mut dau_nhieu_nhat = 0usize;
    for c in &ma_diem {
        if la_dau(*c) {
            lien_tiep += 1;
            dau_nhieu_nhat = dau_nhieu_nhat.max(lien_tiep);
        } else {
            lien_tiep = 0;
        }
    }

    println!("Nhận về : {v:?}");
    print!("Mã điểm : ");
    for c in &ma_diem {
        print!("U+{:04X} ", *c as u32);
    }
    println!("\n");

    println!("  số mã điểm      : {}", ma_diem.len());
    println!("  chữ gốc         : {so_goc}");
    println!("  dấu rời         : {so_dau}");
    println!(
        "  dấu chồng nhiều nhất trên một chữ: {dau_nhieu_nhat} (trần {})",
        tcc_spec::MAX_COMBINING_MARKS
    );
    println!("  con trỏ (UTF-16): {}", bao_cao.caret_utf16);
    println!("  còn đang ghép   : {}", bao_cao.composing);
    println!();

    let mut hong = Vec::new();

    if v.is_empty() {
        hong.push("ô nhập rỗng — chưa gõ được gì".to_owned());
    }
    if v.is_ascii() {
        hong.push("toàn ký tự ASCII — chưa có chữ tiếng Việt nào".to_owned());
    }
    if bao_cao.composing {
        hong.push("phiên ghép của bộ gõ CHƯA chốt — chuỗi này chưa phải kết quả cuối".to_owned());
    }
    if dau_nhieu_nhat > tcc_spec::MAX_COMBINING_MARKS {
        hong.push(format!(
            "vượt trần dấu chồng ({dau_nhieu_nhat} > {}) — bộ gõ này sẽ BỊ CHẶN khi ứng dụng \
             dùng chuỗi đó làm nhãn",
            tcc_spec::MAX_COMBINING_MARKS
        ));
    }
    // Con trỏ phải ở CUỐI sau khi gõ xong. Nó nhảy về đầu là dấu hiệu bộ dựng
    // dựng lại ô nhập giữa chừng phiên ghép — lỗi kinh điển của bộ gõ.
    let cuoi_utf16 = v.encode_utf16().count();
    if !v.is_empty() && bao_cao.caret_utf16 != cuoi_utf16 {
        hong.push(format!(
            "con trỏ ở {} chứ không ở cuối ({cuoi_utf16}) — ô nhập bị dựng lại giữa chừng?",
            bao_cao.caret_utf16
        ));
    }

    if so_dau == 0 && so_goc > 0 && !v.is_ascii() {
        println!("Dạng chuẩn hoá: DỰNG SẴN (NFC) — mỗi chữ có dấu là một mã điểm.");
        println!("  Đây là dạng tốn trần dấu chồng ÍT nhất, và là dạng mong đợi trên macOS.");
    } else if so_dau > 0 {
        println!("Dạng chuẩn hoá: TÁCH RỜI (có {so_dau} dấu đứng riêng).");
        println!(
            "  Hiện ra giống hệt NFC nhưng tốn trần gấp bội. Đây chính là thứ mắt không thấy được."
        );
    }
    println!();

    if hong.is_empty() {
        println!("✓ Cổng \"gõ tiếng Việt có dấu\" ĐẠT — dấu chồng đúng, con trỏ đúng chỗ.");
        ExitCode::SUCCESS
    } else {
        for h in &hong {
            println!("✗ {h}");
        }
        ExitCode::FAILURE
    }
}
