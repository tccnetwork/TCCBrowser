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
    // Tên MỚI mỗi lượt: một lượt chạy bị cắt giữa chừng để lại rác, và lượt sau
    // đo nhầm chính đống rác ấy thay vì đo mã.
    let ten = &format!(
        "tcc-thu-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs())
    );
    // In TỪNG BƯỚC. Treo ở đâu thì phải nhìn ra ngay bước ấy, chứ không phải
    // đoán từ một khoảng lặng.
    println!("1. contains TRƯỚC khi cất (không được hỏi xác thực)…");
    println!("   = {}", kho.contains(ten));
    println!("2. store…");
    match kho.store(ten, SecretKey::new(vec![7u8; 32])) {
        Ok(()) => {
            println!("✓ CẤT ĐƯỢC vào Keychain với USER_PRESENCE");
            println!("3. contains SAU khi cất (đây là chỗ từng treo)…");
            println!("   = {}", kho.contains(ten));
            println!("4. delete… (bước này ĐƯỢC PHÉP hỏi xác thực)");
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
