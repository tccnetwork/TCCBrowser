//! Kiểm CẢ CHUỖI: gói đã ký → cấp/không cấp quyền → bấm nút → hành vi.
//!
//! Chạy: `cargo run -p tcc-shell --features cua-so --example kiem-hanh-vi <gói>`
//!
//! Cố ý KHÔNG mở cửa sổ. Cửa sổ đã có `kiem-bam-nut` kiểm; ở đây chỉ hỏi một
//! câu: **quyền năng có thật sự chặn đường ra ngoài không**.
//!
//! Mạng dùng bản GIẢ, ghi lại mọi lần bị gọi. Đó là điểm mấu chốt: khẳng định
//! "bị từ chối" là chưa đủ, phải khẳng định **không một gói tin nào rời khỏi
//! máy** — kiểm quyền sau khi gọi thì gói tin đã đến nơi, mà với một máy chủ
//! theo dõi thì chỉ cần đến nơi là đủ.

#![allow(
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay, không nuốt lỗi"
)]

use std::{cell::RefCell, path::PathBuf, process::ExitCode};

use tcc_capability::Decision;
use tcc_crypto::HybridEd25519MlDsa;
use tcc_runtime::Mang;

/// Mạng giả, đếm số lần bị gọi.
struct MangGia {
    da_goi: RefCell<Vec<String>>,
}

impl Mang for MangGia {
    fn get(&self, host: &str, path: &str) -> Result<Vec<u8>, String> {
        self.da_goi.borrow_mut().push(format!("{host}{path}"));
        Ok(b"[]".to_vec())
    }
}

fn main() -> ExitCode {
    let Some(duong_dan) = std::env::args().nth(1).map(PathBuf::from) else {
        eprintln!("cần đường dẫn thư mục gói");
        return ExitCode::FAILURE;
    };

    let (app_goc, noi_dung) = match tcc_runtime::verify_from_dir(&duong_dan, &HybridEd25519MlDsa) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("✗ {e}");
            return ExitCode::FAILURE;
        }
    };
    let ds: Vec<String> = app_goc
        .manifest()
        .actions
        .iter()
        .map(|a| a.id.as_str().to_owned())
        .collect();
    if ds.is_empty() {
        eprintln!("✗ gói này không khai hành vi nào — không có gì để kiểm");
        return ExitCode::FAILURE;
    }
    println!("Hành vi khai trong bản kê khai: {}", ds.join(", "));
    println!();

    let mut hong = 0;

    // ---- Trường hợp A: người dùng TỪ CHỐI ----
    let app = tcc_runtime::grant_verified(app_goc, noi_dung.clone(), |_| Decision::Deny)
        .expect("cấp quyền hỏng");
    let m = MangGia {
        da_goi: RefCell::new(Vec::new()),
    };
    for id in &ds {
        let ket = app.thuc_hien(id, &m);
        println!("  [từ chối] {id} → {}", tom_tat(&ket));
        if ket.is_ok() {
            eprintln!("✗ HỎNG: chạy được hành vi dù người dùng đã từ chối");
            hong += 1;
        }
    }
    let so_lan = m.da_goi.borrow().len();
    println!("  gói tin đã rời khỏi máy: {so_lan} (phải là 0)");
    if so_lan != 0 {
        eprintln!(
            "✗ HỎNG NẶNG: đã gọi ra ngoài dù chưa được cấp quyền — {:?}",
            m.da_goi.borrow()
        );
        hong += 1;
    }
    println!();

    // ---- Trường hợp B: người dùng CHO PHÉP ----
    let (app_goc, noi_dung) = tcc_runtime::verify_from_dir(&duong_dan, &HybridEd25519MlDsa)
        .expect("kiểm chữ ký lần hai hỏng");
    let app = tcc_runtime::grant_verified(app_goc, noi_dung, |_| Decision::Allow)
        .expect("cấp quyền hỏng");
    let m = MangGia {
        da_goi: RefCell::new(Vec::new()),
    };
    for id in &ds {
        let ket = app.thuc_hien(id, &m);
        println!("  [cho phép] {id} → {}", tom_tat(&ket));
        if ket.is_err() {
            eprintln!("✗ HỎNG: đã cấp quyền mà hành vi vẫn không chạy");
            hong += 1;
        }
    }
    println!("  đã gọi: {:?}", m.da_goi.borrow());
    println!();

    // ---- Trường hợp C: THU HỒI giữa chừng ----
    app.revoke_all();
    let m2 = MangGia {
        da_goi: RefCell::new(Vec::new()),
    };
    for id in &ds {
        if app.thuc_hien(id, &m2).is_ok() {
            eprintln!("✗ HỎNG: thu hồi rồi mà hành vi vẫn chạy");
            hong += 1;
        }
    }
    println!(
        "  [thu hồi] gói tin rời khỏi máy: {} (phải là 0)",
        m2.da_goi.borrow().len()
    );
    if !m2.da_goi.borrow().is_empty() {
        eprintln!("✗ HỎNG NẶNG: thu hồi rồi mà vẫn gọi ra ngoài");
        hong += 1;
    }

    println!();
    if hong == 0 {
        println!("✓ Quyền năng chặn đúng cả ba chiều: chưa cấp, đã cấp, đã thu hồi.");
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn tom_tat(k: &Result<Vec<u8>, tcc_runtime::ActionError>) -> String {
    match k {
        Ok(v) => format!("chạy được, {} byte", v.len()),
        Err(e) => format!("từ chối ({e})"),
    }
}
