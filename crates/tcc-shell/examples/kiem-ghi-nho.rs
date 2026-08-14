//! Kiểm kho quyền trên ĐĨA THẬT, qua cả đường ống.
//!
//! Chạy: `cargo run -p tcc-shell --example kiem-ghi-nho <gói>`
//!
//! Không mở cửa sổ — kiểm phần quyết định, không kiểm phần vẽ.

#![allow(
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay, không nuốt lỗi"
)]

use std::{path::PathBuf, process::ExitCode};

use tcc_capability::Decision;
use tcc_crypto::HybridEd25519MlDsa;
use tcc_shell::ghi_nho::GhiNho;

fn main() -> ExitCode {
    let Some(goi) = std::env::args().nth(1).map(PathBuf::from) else {
        eprintln!("cần đường dẫn thư mục gói");
        return ExitCode::FAILURE;
    };
    let kho = std::env::temp_dir().join(format!("tcc-kiem-ghinho-{}.json", std::process::id()));
    let _ = std::fs::remove_file(&kho);

    let (app, _) = match tcc_runtime::verify_from_dir(&goi, &HybridEd25519MlDsa) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("✗ {e}");
            return ExitCode::FAILURE;
        }
    };
    let m = app.manifest().clone();
    if m.capabilities.is_empty() {
        eprintln!("✗ gói này không xin quyền nào — không có gì để kiểm");
        return ExitCode::FAILURE;
    }
    let xin = &m.capabilities[0];
    println!("Quyền đem kiểm: {}", xin.name);
    let mut hong = 0;

    // ---- Lần đầu: chưa có câu trả lời ----
    let mut g = GhiNho::mo(&kho);
    println!("  lần đầu           → {:?} (phải là None)", g.tra(&m, xin));
    if g.tra(&m, xin).is_some() {
        eprintln!("✗ HỎNG: chưa hỏi mà đã có câu trả lời");
        hong += 1;
    }

    // ---- Nhớ, ghi ra đĩa, mở lại ----
    g.nho(&m, xin, Decision::Allow);
    g.ghi().expect("ghi kho hỏng");
    let g2 = GhiNho::mo(&kho);
    println!(
        "  mở lại từ đĩa     → {:?} (phải là Allow)",
        g2.tra(&m, xin)
    );
    if g2.tra(&m, xin) != Some(Decision::Allow) {
        eprintln!("✗ HỎNG: nhớ rồi mà mở lại không thấy");
        hong += 1;
    }

    // ---- ⚠️ Ứng dụng nới rộng phạm vi ----
    let mut moi = m.clone();
    if let tcc_spec::Scope::Network { hosts } = &mut moi.capabilities[0].scope {
        hosts.push("thu-thap.example".to_owned());
        println!("  (giả lập bản mới xin thêm thu-thap.example)");
        let ket = g2.tra(&moi, &moi.capabilities[0]);
        println!("  sau khi nới rộng  → {ket:?} (phải là None)");
        if ket.is_some() {
            eprintln!(
                "✗ HỎNG NẶNG: quyền cũ phủ lên phạm vi mới — người dùng chưa bao \
                 giờ đồng ý với máy chủ thứ hai"
            );
            hong += 1;
        }
    }

    // ---- ⚠️ Khoá người ký đổi ----
    let mut gia = m.clone();
    gia.publisher = "ff".repeat(1992);
    let ket = g2.tra(&gia, &gia.capabilities[0]);
    println!("  đổi khoá người ký → {ket:?} (phải là None)");
    if ket.is_some() {
        eprintln!("✗ HỎNG NẶNG: gói ký bằng khoá khác thừa hưởng quyền của gói thật");
        hong += 1;
    }

    // ---- Tệp bị sửa hỏng ----
    std::fs::write(&kho, "{ hỏng").expect("ghi hỏng");
    let ket = GhiNho::mo(&kho).tra(&m, xin);
    println!("  tệp bị sửa hỏng   → {ket:?} (phải là None)");
    if ket.is_some() {
        eprintln!("✗ HỎNG: tệp hỏng mà vẫn cho ra câu trả lời");
        hong += 1;
    }

    let _ = std::fs::remove_file(&kho);
    println!();
    if hong == 0 {
        println!("✓ Kho quyền: nhớ đúng, và mọi thay đổi đáng ngờ đều dẫn tới hỏi lại.");
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
