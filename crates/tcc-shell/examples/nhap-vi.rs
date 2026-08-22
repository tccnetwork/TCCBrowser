//! Nhập một ví vào kho khoá của hệ điều hành.
//!
//! ```text
//! cargo run -p tcc-shell --features "import-web-wallet,os-keystore" --example nhap-vi -- <tệp>
//! ```
//!
//! Tệp có thể là:
//!
//! | Nội dung | Đường |
//! |---|---|
//! | JSON `tcc_wallets_v4` xuất từ ví web | hỏi PIN, giải mã, cất lại |
//! | 64 ký tự hex — hạt giống 32 byte | cất thẳng |
//! | 24 chữ cách nhau bằng dấu cách | dẫn xuất rồi cất |
//!
//! # Vì sao nhận TỆP chứ không nhận tham số dòng lệnh
//!
//! Hạt giống trên dòng lệnh là hạt giống trong lịch sử shell, trong `ps`, và
//! trong mọi bản ghi terminal. Một tệp thì xoá được, và người dùng biết nó ở đâu.
//!
//! Mã PIN hỏi trên `stdin` với **tiếng vọng tắt** — gõ không hiện ra màn hình.
//!
//! # Vì sao vẫn còn đường `stdin` này
//!
//! Gõ thẳng vào cửa sổ đã chạy được: bộ dựng ra pixel nhận chữ từ bộ gõ của hệ
//! điều hành và khung tự giữ nội dung ô nhập —
//! `tcc_shell::wallet_flow::import_from_file` đi đúng đường ấy. Tệp này ở lại
//! cho máy KHÔNG có màn hình, và vì `stdin` là đường duy nhất kiểm được tự động
//! mà không cần một vòng lặp sự kiện.

use std::process::{Command, ExitCode};

use tcc_chain::wallet::WalletSecret;
use tcc_keystore::SecretKey;
use tcc_shell::wallet_store;

fn main() -> ExitCode {
    let Some(duong_dan) = std::env::args().nth(1) else {
        eprintln!("cần một tệp: <tệp-ví.json | hạt-giống.hex | cụm-từ.txt>");
        return ExitCode::FAILURE;
    };
    let Ok(noi_dung) = std::fs::read_to_string(&duong_dan) else {
        eprintln!("✗ không đọc được {duong_dan}");
        return ExitCode::FAILURE;
    };
    let noi_dung = noi_dung.trim();

    let khoa = match doc_vi(noi_dung) {
        Ok(k) => k,
        Err(e) => {
            eprintln!("✗ {e}");
            return ExitCode::FAILURE;
        }
    };

    let dia_chi = khoa.address().to_string();
    println!("Ví: {dia_chi}");

    let mut kho = match wallet_store::open() {
        Ok(k) => k,
        Err(e) => {
            eprintln!("✗ {e}");
            return ExitCode::FAILURE;
        }
    };
    let ten = wallet_store::key_name(&dia_chi);

    // `store` của Keychain tự chặn ghi đè — ghi đè khoá ví là mất tiền vĩnh
    // viễn. Báo lại cho rõ thay vì để lỗi thô đi ra.
    match kho.store(&ten, SecretKey::new(khoa.expose_seed().to_vec())) {
        Ok(()) => {
            println!("✓ đã cất vào Keychain dưới tên \"{ten}\"");
            println!();
            println!("Gửi thử một giao dịch:");
            println!(
                "  cargo run -p tcc-shell --features \"network,os-keystore\" \\\n    --example gui-giao-dich -- {dia_chi} <dia-chi-nhan> 1000000000000000000 \"thu\""
            );
            println!();
            println!("⚠ Bản cũ ở ví web VẪN CÒN, vẫn khoá bằng đúng mã PIN cũ.");
            println!("  Ở đây không đụng vào nó. Khi nào chắc bản này chạy được thì");
            println!("  tự xoá nó ngay trên trang web.");
            println!();
            println!("⚠ Và xoá {duong_dan} đi — nó vẫn đang giữ khoá của bạn.");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("✗ không cất được: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Nhận ra tệp thuộc loại nào rồi dẫn ra khoá.
fn doc_vi(noi_dung: &str) -> Result<WalletSecret, String> {
    if noi_dung.starts_with('{') {
        return tu_vi_web(noi_dung);
    }
    let goc = noi_dung.strip_prefix("0x").unwrap_or(noi_dung);
    if goc.len() == 64 && goc.chars().all(|c| c.is_ascii_hexdigit()) {
        let mut hat = [0u8; 32];
        for (i, o) in hat.iter_mut().enumerate() {
            *o = u8::from_str_radix(goc.get(i * 2..i * 2 + 2).ok_or("hex cụt")?, 16)
                .map_err(|_| "hex hỏng")?;
        }
        // `from_raw_seed`, KHÔNG phải `from_seed_phrase`: 64 hex ở đây là hạt
        // giống thật, không phải chuỗi đem đi băm. Lẫn hai đường là ra ví khác
        // — cái bẫy đã làm chuỗi dừng chốt khối ngày 30/07/2026.
        return Ok(WalletSecret::from_raw_seed(hat));
    }
    if noi_dung.split_whitespace().count() == 24 {
        return WalletSecret::from_mnemonic(noi_dung).map_err(|e| e.to_string());
    }
    Err("không nhận ra định dạng: cần JSON ví web, 64 hex, hoặc 24 chữ".to_owned())
}

fn tu_vi_web(json: &str) -> Result<WalletSecret, String> {
    let ds = tcc_chain::import::read_export(json).map_err(|e| e.to_string())?;
    let vi = match ds.len() {
        0 => return Err("tệp không có ví nào".to_owned()),
        1 => &ds[0],
        n => {
            println!("Tệp có {n} ví:");
            for (i, v) in ds.iter().enumerate() {
                println!("  {}. {} — {}", i + 1, v.address, v.label);
            }
            let so: usize = hoi("Chọn số: ")?
                .trim()
                .parse()
                .map_err(|_| "không phải số")?;
            ds.get(so.wrapping_sub(1)).ok_or("số ngoài danh sách")?
        }
    };
    println!("Ví: {} — {}", vi.address, vi.label);

    let pin = hoi_kin("Mã PIN của ví web: ")?;
    let ra = vi.unlock(pin.trim()).map_err(|e| e.to_string())?;
    if let Some(cum_tu) = &ra.mnemonic {
        println!(
            "(bản ghi có kèm cụm từ khôi phục {} chữ)",
            cum_tu.split(' ').count()
        );
    }
    Ok(ra.secret)
}

fn hoi(nhac: &str) -> Result<String, String> {
    use std::io::Write as _;
    print!("{nhac}");
    let _ = std::io::stdout().flush();
    let mut s = String::new();
    std::io::stdin()
        .read_line(&mut s)
        .map_err(|e| e.to_string())?;
    Ok(s)
}

/// Hỏi mà KHÔNG hiện chữ ra màn hình.
///
/// Dùng `stty` thay vì gọi `termios` qua FFI: crate này chạy dưới
/// `unsafe_code = "deny"` toàn workspace, và một dòng `unsafe` để tắt tiếng
/// vọng là cái giá quá đắt.
///
/// Nếu `stty` không chạy (không phải terminal), **báo ra rồi vẫn hỏi** — thà
/// người dùng biết chữ mình gõ đang hiện, còn hơn tưởng nó bị che.
fn hoi_kin(nhac: &str) -> Result<String, String> {
    let tat = Command::new("stty").arg("-echo").status().is_ok();
    if !tat {
        eprintln!("⚠ không tắt được tiếng vọng — chữ bạn gõ SẼ hiện ra màn hình.");
    }
    let ra = hoi(nhac);
    if tat {
        let _ = Command::new("stty").arg("echo").status();
        println!();
    }
    ra
}
