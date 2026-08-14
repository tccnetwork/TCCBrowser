//! Mở màn hình ỨNG DỤNG để nhìn bằng mắt, bỏ qua hộp thoại hỏi quyền.
//!
//! Chạy: `cargo run -p tcc-shell --features window --example xem-man-hinh <gói> [giây]`
//!
//! Không cấp quyền nào — chỉ vẽ. Dùng để soi bố cục, chữ tiếng Việt và ảnh
//! trong gói mà không phải bấm qua hộp thoại.

#![allow(clippy::expect_used, reason = "công cụ soi bằng mắt: hỏng thì nổ ngay")]

use std::{path::PathBuf, process::ExitCode, time::Duration};

use tcc_capability::Decision;
use tcc_crypto::HybridEd25519MlDsa;

fn main() -> ExitCode {
    let Some(goi) = std::env::args().nth(1).map(PathBuf::from) else {
        eprintln!("cần đường dẫn thư mục gói");
        return ExitCode::FAILURE;
    };
    let giay: u64 = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(12);

    let (app, noi_dung) =
        tcc_runtime::verify_from_dir(&goi, &HybridEd25519MlDsa).expect("kiểm chữ ký hỏng");
    // Không cấp quyền nào: ta chỉ xem màn hình, không chạy hành vi.
    let app =
        tcc_runtime::grant_verified(app, noi_dung, |_| Decision::Deny).expect("cấp quyền hỏng");

    match tcc_shell::window::show_app(
        &app,
        tcc_shell::Language::default(),
        Some(Duration::from_secs(giay)),
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("✗ {e}");
            ExitCode::FAILURE
        }
    }
}
