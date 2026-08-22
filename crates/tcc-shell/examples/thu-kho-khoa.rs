//! Thử KHO KHOÁ THẬT trong một gói đã ký — cất, hỏi có, xoá.
//!
//! ```text
//! cargo build --release -p tcc-shell --features os-keystore --example thu-kho-khoa
//! # rồi KÝ binary ấy bằng hồ sơ cấp phép, mới chạy
//! ```
//!
//! # Vì sao chỉ cất chứ không đọc
//!
//! Đọc khoá là hệ điều hành hỏi Touch ID, mà không có ai chạm vào khi phép thử
//! chạy. **Cất được đã là bằng chứng**: đúng bước này từng trả về
//! `A required entitlement isn't present` trên mọi bản dựng chưa có hồ sơ cấp
//! phép — xem `docs/vi-thiet-ke.md` §19.

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "ví dụ chạy tay: kết quả phải hiện ra màn hình"
)]

use std::process::ExitCode;

use tcc_keystore::SecretKey;

fn main() -> ExitCode {
    let mut kho = match tcc_shell::wallet_store::open() {
        Ok(k) => k,
        Err(e) => {
            eprintln!("✗ không mở được kho khoá: {e}");
            return ExitCode::FAILURE;
        }
    };
    let ten = "tcc-thu-quyen-keychain";
    if kho.contains(ten) {
        let _ = kho.delete(ten);
    }
    match kho.store(ten, SecretKey::new(vec![7u8; 32])) {
        Ok(()) => {
            println!("✓ CẤT ĐƯỢC vào Keychain với USER_PRESENCE");
            println!("  contains = {}", kho.contains(ten));
            match kho.delete(ten) {
                Ok(()) => println!("✓ xoá được — Keychain sạch lại, không để rác"),
                Err(e) => println!("⚠ không xoá được: {e}"),
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("✗ KHÔNG cất được: {e}");
            if tcc_shell::wallet_store::is_unsigned_build(&e.to_string()) {
                eprintln!("  → đây là lỗi THIẾU QUYỀN: binary chưa ký kèm hồ sơ cấp phép.");
            }
            ExitCode::FAILURE
        }
    }
}
