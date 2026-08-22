//! Chạy một gói đã ký, trên bộ dựng ra pixel.
//!
//! Chạy: `cargo run -p tcc-shell --features window --example man-hinh-raster examples/hello-tcc`
//!
//! # Nó chứng minh câu nào
//!
//! `docs/ke-hoach.md`, cổng ra giai đoạn 4: *"ứng dụng mẫu chạy trên **cả hai**
//! bộ dựng, **không sửa một dòng nào**."* Cổng ấy đã qua ngày 22/08/2026, và
//! hôm sau bộ dựng thứ nhất bị bỏ hẳn — chính vì nó đã qua nên bỏ được.
//!
//! Nay tệp này chứng minh một câu hẹp hơn và vẫn đáng: cùng một thư mục gói,
//! cùng một chữ ký, cùng một `ui.json`, hiện ra màn hình và **bấm được**. Vẽ ra
//! được điểm ảnh mà không ai nhìn thấy chúng thì mới chỉ chứng minh `tcc-ui`
//! không dính thẻ đánh dấu; hiện ra và bấm được mới là một trình duyệt.
//!
//! Không một dòng thư viện dựng web nào, không máy dựng nào của hệ điều hành.

#![allow(
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay, không nuốt lỗi"
)]

use std::{path::PathBuf, process::ExitCode};

use tcc_crypto::HybridEd25519MlDsa;
use tcc_shell::Language;

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

    // `hop-thoai` — chạy HỘP THOẠI HỎI QUYỀN qua bộ dựng ra pixel.
    //
    // Đây mới là màn hình đáng chạy nhất trên bộ dựng thứ hai: nó có CÔNG TẮC,
    // và công tắc là chỗ hai bộ dựng khác nhau thật sự. WebView để trình duyệt
    // giữ trạng thái trong tài liệu; ở đây khung phải tự giữ. Một màn hình chỉ
    // có chữ và nút thì không đá vào chỗ ấy.
    if std::env::args().nth(2).as_deref() == Some("hop-thoai") {
        println!(
            "Hộp thoại hỏi quyền của \"{}\" — RA PIXEL",
            app.manifest().name
        );
        println!("  gạt công tắc rồi bấm một nút.");
        // Tiêu đề do KHUNG đặt, không truyền tay — xem `open_permission_dialog_raster`.
        return match tcc_shell::window_raster::open_permission_dialog_raster(
            app.manifest(),
            Language::Vi,
        ) {
            Ok(ket) => {
                match ket.action {
                    Some(h) => {
                        println!("✓ bấm: {h}");
                        println!(
                            "  quyền được BẬT: {}",
                            if ket.toggles_on.is_empty() {
                                "(không mục nào)".to_owned()
                            } else {
                                ket.toggles_on
                                    .iter()
                                    .cloned()
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            }
                        );
                    }
                    // Đóng cửa sổ KHÔNG phải đồng ý.
                    None => println!("✓ đóng cửa sổ — KHÔNG cấp quyền nào, mọi công tắc bỏ đi."),
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("✗ {e}");
                ExitCode::FAILURE
            }
        };
    }

    println!(
        "Màn hình của \"{}\" — bộ dựng RA PIXEL",
        app.manifest().name
    );
    println!("  nút trong cây khai báo : {}", cay.node_count());
    println!("  bấm một nút để đóng, hoặc đóng cửa sổ.");

    // ⚠️ KHÔNG truyền `manifest().name` làm tiêu đề. Bản đầu của ví dụ này làm
    // thế và mở lại đúng lỗ giả mạo tiêu đề mà `SECURITY.md` §3.1c đã vá cho
    // đường WebView — mã ứng dụng đã ký phải đứng trước tên ứng dụng tự đặt.
    match tcc_shell::window_raster::open_app_screen_raster(app.manifest(), byte, Language::Vi) {
        Ok(ket) => {
            match ket.action {
                Some(h) => {
                    println!("✓ Người dùng bấm hành động: {h}");
                    if !ket.toggles_on.is_empty() {
                        println!(
                            "  công tắc đang bật     : {}",
                            ket.toggles_on
                                .iter()
                                .cloned()
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                    }
                    println!("✓ Gói đã ký lên màn hình và BẤM ĐƯỢC, không qua WebView.");
                }
                // Đóng cửa sổ KHÔNG phải đồng ý — nên công tắc bị bỏ đi, không in ra.
                None => println!("✓ Cửa sổ đóng, không bấm nút nào. Mọi công tắc bỏ đi."),
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("✗ {e}");
            ExitCode::FAILURE
        }
    }
}
