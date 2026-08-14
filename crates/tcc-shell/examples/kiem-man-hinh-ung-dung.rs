//! Kiểm màn hình ỨNG DỤNG dựng từ cây khai báo, qua WebKit thật.
//!
//! Chạy: `cargo run -p tcc-shell --features cua-so --example kiem-man-hinh-ung-dung <thư-mục-gói>`
//!
//! Cố ý KHÔNG đi qua luồng hỏi quyền — luồng đó đã có `kiem-bam-nut` kiểm riêng.
//! Ở đây chỉ hỏi một câu: cây khai báo trong gói có lên màn hình đúng không.

#![allow(
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay, không nuốt lỗi"
)]

use std::{path::PathBuf, process::ExitCode, time::Duration};

use tcc_crypto::HybridEd25519MlDsa;
use tcc_render_webview::{WebViewRenderer, cua_so};
use tcc_ui::Renderer as _;

fn main() -> ExitCode {
    let Some(duong_dan) = std::env::args().nth(1).map(PathBuf::from) else {
        eprintln!("cần đường dẫn thư mục gói");
        return ExitCode::FAILURE;
    };

    // Kiểm chữ ký trước. Nội dung chưa qua chữ ký thì không đáng đem đi vẽ.
    let (app, noi_dung) = match tcc_runtime::verify_from_dir(&duong_dan, &HybridEd25519MlDsa) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("✗ {e}");
            return ExitCode::FAILURE;
        }
    };

    let byte = noi_dung
        .get(&app.manifest().entry)
        .expect("verify đã kiểm điểm vào tồn tại");

    let cay = match tcc_ui::dang_goi::doc(byte) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("✗ cây giao diện hỏng: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mong_doi = cay.node_count();

    let mut bd = WebViewRenderer::new();
    if let Err(e) = bd.render(&cay) {
        eprintln!("✗ không vẽ được: {e}");
        return ExitCode::FAILURE;
    }

    let bao = match cua_so::kiem_khoi(bd.tai_lieu(), Duration::from_secs(20)) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("✗ WebKit không báo về: {e}");
            return ExitCode::FAILURE;
        }
    };

    println!("Màn hình của \"{}\":", app.manifest().name);
    println!("  nút trong cây khai báo : {mong_doi}");
    println!("  nút WebKit nhìn thấy   : {}", bao.so_nut);
    println!("  vai trò                : {}", bao.vai_tro.join(", "));
    println!("  thẻ kịch bản còn sống  : {}", bao.so_kich_ban);

    if bao.so_kich_ban != 0 {
        eprintln!("✗ HỎNG NẶNG: gói ứng dụng chạy được kịch bản");
        return ExitCode::FAILURE;
    }
    if bao.so_nut != mong_doi {
        eprintln!("✗ HỎNG: số nút lệch — cây khai báo không lên màn hình đầy đủ");
        return ExitCode::FAILURE;
    }
    println!("✓ Cây khai báo trong gói lên màn hình đúng, không một dòng kịch bản nào.");
    ExitCode::SUCCESS
}
