//! **Cổng ra giai đoạn 4**: đúng gói đã ký ấy, trên bộ dựng KHÔNG phải WebView.
//!
//! Chạy: `cargo run -p tcc-shell --features cua-so-raster --example man-hinh-raster examples/hello-tcc`
//!
//! # Nó chứng minh câu nào
//!
//! `docs/ke-hoach.md`, cổng ra giai đoạn 4: *"ứng dụng mẫu chạy trên **cả hai**
//! bộ dựng, **không sửa một dòng nào**."*
//!
//! Tệp này và `kiem-man-hinh-ung-dung.rs` đọc **cùng một thư mục gói**, kiểm
//! **cùng một chữ ký**, giải mã **cùng một `ui.json`**. Khác đúng một dòng: một
//! bên `WebViewRenderer`, bên kia `RasterRenderer`. Gói không biết, và không cần
//! biết, mình đang được vẽ bằng gì.
//!
//! Trước tệp này, `tcc-render-raster` vẽ ra được điểm ảnh mà **không ai nhìn
//! thấy chúng**. Vẽ ra được thì chứng minh `tcc-ui` không dính HTML; **hiện ra
//! và bấm được** mới chứng minh nó thay thế được WebView. Chỉ câu sau là đường
//! thoát.
//!
//! Không một dòng `wry`, không WebKit, không máy dựng nào của hệ điều hành.

#![allow(
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay, không nuốt lỗi"
)]

use std::{path::PathBuf, process::ExitCode};

use tcc_crypto::HybridEd25519MlDsa;

fn main() -> ExitCode {
    let Some(duong_dan) = std::env::args().nth(1).map(PathBuf::from) else {
        eprintln!("cần đường dẫn thư mục gói");
        return ExitCode::FAILURE;
    };

    // Kiểm chữ ký trước — y hệt đường WebView. Nội dung chưa qua chữ ký thì
    // không đáng đem đi vẽ, và điều đó không phụ thuộc vẽ bằng gì.
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

    let cay = match tcc_ui::wire::decode(byte) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("✗ cây giao diện hỏng: {e}");
            return ExitCode::FAILURE;
        }
    };

    println!(
        "Màn hình của \"{}\" — bộ dựng RA PIXEL",
        app.manifest().name
    );
    println!("  nút trong cây khai báo : {}", cay.node_count());
    println!("  bấm một nút để đóng, hoặc đóng cửa sổ.");

    match tcc_render_raster::window::open_screen(&cay, app.manifest().name.as_str()) {
        Ok(Some(h)) => {
            println!("✓ Người dùng bấm hành động: {h}");
            println!("✓ Gói đã ký lên màn hình và BẤM ĐƯỢC, không qua WebView.");
            ExitCode::SUCCESS
        }
        Ok(None) => {
            println!("✓ Cửa sổ đóng, không bấm hành động nào.");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("✗ {e}");
            ExitCode::FAILURE
        }
    }
}
