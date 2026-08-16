//! Gửi một giao dịch — cả vòng, với chuỗi thật.
//!
//! ```text
//! cargo run -p tcc-shell --features "network,os-keystore" --example gui-giao-dich -- \
//!     <dia-chi-gui> <dia-chi-nhan> <so-tien-wei> [ghi-nho]
//! ```
//!
//! # Khoá KHÔNG đi qua dòng lệnh
//!
//! Không có tham số nào nhận hạt giống, cụm từ khôi phục hay mã PIN, và sẽ
//! không bao giờ có. Khoá lấy từ kho khoá của hệ điều hành, và hệ điều hành hỏi
//! Touch ID ngay lúc lấy.
//!
//! Đưa khoá qua tham số dòng lệnh là đưa nó vào lịch sử shell, vào `ps`, và vào
//! bất kỳ bản ghi terminal nào — ba chỗ không ai nhớ dọn.
//!
//! # Thứ tự, và vì sao nó không đảo được
//!
//! 1. Hỏi máy chủ dựng giao dịch chưa ký
//! 2. **Giải mã, tự tính lại băm, so với băm máy chủ đưa** — lệch là dừng ở đây
//! 3. Hiện ra thứ ĐÃ GIẢI MÃ, không hiện lại thứ người dùng vừa gõ
//! 4. Người dùng gõ `ky` để đồng ý
//! 5. Mở khoá ví (Touch ID), ký, gửi
//!
//! Bước 2 xong trước bước 5 không phải vì tôi xếp thế, mà vì `sign` đòi một
//! `PendingTransaction`, và chỉ bước 2 sinh ra nó.
//!
//! # Cổng chặn cứng
//!
//! Ví dụ này **từ chối chạy trên mainnet**. Không có giao dịch mainnet nào
//! trước khi qua kiểm định an ninh độc lập — `SECURITY.md` §3.5.

use std::io::Write as _;
use std::process::ExitCode;

use tcc_net::rpc::JsonRpc;
use tcc_shell::{signing_flow, text::Language, wallet_store};

/// Testnet. Mainnet có mã khác, và ví dụ này từ chối mọi mã khác — xem
/// `chan_mainnet` bên dưới.
const CHAIN_TESTNET: u64 = 91338;
const RPC: &str = "https://rpc2.tcc-coin.com";

fn main() -> ExitCode {
    let doi: Vec<String> = std::env::args().skip(1).collect();
    let (Some(gui), Some(nhan), Some(tien)) = (doi.first(), doi.get(1), doi.get(2)) else {
        eprintln!("cần: <dia-chi-gui> <dia-chi-nhan> <so-tien-wei> [ghi-nho]");
        return ExitCode::FAILURE;
    };
    let ghi_nho = doi.get(3).cloned().unwrap_or_default();

    let rpc = match JsonRpc::new(RPC) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("✗ {e}");
            return ExitCode::FAILURE;
        }
    };

    // ── 1. Máy chủ dựng giao dịch chưa ký ──
    let tra = match rpc.call(
        "tcc_buildUnsignedTransfer",
        &serde_json::json!([gui, nhan, tien, ghi_nho]),
    ) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("✗ không dựng được giao dịch: {e}");
            return ExitCode::FAILURE;
        }
    };

    let (Some(tx_hex), Some(bam_hex), Some(tx_b64)) = (
        tra.get("unsigned_tx_hex").and_then(|v| v.as_str()),
        tra.get("signing_message_hex").and_then(|v| v.as_str()),
        tra.get("unsigned_tx_base64").and_then(|v| v.as_str()),
    ) else {
        eprintln!("✗ phản hồi thiếu trường — máy chủ này không nói giao thức ta hiểu");
        return ExitCode::FAILURE;
    };

    // ── 2. Kiểm TRƯỚC. Không có gì để ký nếu bước này trượt. ──
    let (cho, _man) = match signing_flow::review(tx_hex, bam_hex, Language::Vi) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("✗ TỪ CHỐI: {e}");
            eprintln!();
            eprintln!("Máy chủ đưa một băm không khớp giao dịch nó gửi kèm.");
            eprintln!("Không ký gì cả, và không có gì để ký.");
            return ExitCode::FAILURE;
        }
    };

    let tx = cho.transaction();
    if tx.chain_id != CHAIN_TESTNET {
        eprintln!(
            "✗ mạng {} không phải testnet {CHAIN_TESTNET} — ví dụ này KHÔNG chạy trên mạng khác.",
            tx.chain_id
        );
        eprintln!(
            "  Không giao dịch mainnet nào trước kiểm định an ninh độc lập (SECURITY.md §3.5)."
        );
        return ExitCode::FAILURE;
    }

    // ── 3. Hiện thứ ĐÃ GIẢI MÃ ──
    println!("✓ băm tự tính KHỚP băm máy chủ đưa");
    println!();
    println!("  người nhận: {}", tx.to);
    println!(
        "  số tiền   : {} TCC",
        tcc_shell::transaction_screen::format_amount(tx.amount)
    );
    println!(
        "  phí tối đa: {} TCC",
        tcc_shell::transaction_screen::format_amount(
            u128::from(tx.gas_price) * u128::from(tx.gas_limit)
        )
    );
    println!("  mạng      : {} (testnet)", tx.chain_id);
    println!("  thứ tự    : {}", tx.nonce);
    if !tx.memo.is_empty() {
        println!("  ghi nhớ   : {}", tx.memo);
    }
    println!();
    println!("Việc này CHUYỂN TIỀN và không hoàn tác được.");

    // ── 4. Người dùng đồng ý ──
    print!("Gõ đúng chữ `ky` để ký, bất cứ thứ gì khác là huỷ: ");
    let _ = std::io::stdout().flush();
    let mut tra_loi = String::new();
    if std::io::stdin().read_line(&mut tra_loi).is_err() || tra_loi.trim() != "ky" {
        println!("Đã huỷ. Không ký gì cả.");
        return ExitCode::SUCCESS;
    }

    ky_va_gui(&rpc, cho, gui, tx_b64)
}

/// Bước 5 tách riêng, không phải để `main` ngắn đi.
///
/// Hàm này là chỗ DUY NHẤT khoá thật đi qua, và nó chỉ nhận được một
/// `PendingTransaction` — thứ chỉ `review` sinh ra được. Đọc chữ ký hàm là thấy
/// ranh giới ấy; đọc một `main` dài trăm dòng thì không.
fn ky_va_gui(
    rpc: &JsonRpc,
    cho: signing_flow::PendingTransaction,
    dia_chi: &str,
    tx_b64: &str,
) -> ExitCode {
    let kho = match wallet_store::open() {
        Ok(k) => k,
        Err(e) => {
            eprintln!("✗ {e}");
            return ExitCode::FAILURE;
        }
    };
    let muc = wallet_store::key_name(dia_chi);
    let ly_do = wallet_store::signing_purpose(dia_chi);
    // Lý do hiện ở màn hình CỦA TA, vì nó không tới được hộp thoại của macOS —
    // xem `docs/vi-thiet-ke.md` §10.
    println!("{}", ly_do.prompt);

    let khoa_byte = match kho.unlock(&muc, &ly_do) {
        Ok(k) => k,
        Err(e) => {
            eprintln!("✗ không mở được ví: {e}");
            eprintln!("  (cất hạt giống 32 byte vào Keychain dưới tên \"{muc}\")");
            return ExitCode::FAILURE;
        }
    };
    let Ok(hat) = <[u8; 32]>::try_from(khoa_byte.expose()) else {
        eprintln!("✗ khoá trong kho không phải 32 byte hạt giống");
        return ExitCode::FAILURE;
    };
    let khoa = tcc_chain::wallet::WalletSecret::from_raw_seed(hat);

    // Kiểm khoá đúng ví TRƯỚC khi ký. Ký bằng nhầm ví thì chuỗi cũng từ chối,
    // nhưng thông báo của nó không nói được đâu là chỗ sai.
    if khoa.address().to_string() != *dia_chi {
        eprintln!(
            "✗ khoá trong kho ra địa chỉ {} — không phải {dia_chi}",
            khoa.address()
        );
        return ExitCode::FAILURE;
    }

    let cong_khai = khoa.public_key();
    let chu_ky = cho.sign(&khoa);

    match rpc.call(
        "tcc_submitSignedTransfer",
        &serde_json::json!([tx_b64, hex(chu_ky.as_bytes()), hex(cong_khai.as_bytes())]),
    ) {
        Ok(v) => {
            println!();
            println!("✓ đã gửi: {v}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("✗ gửi thất bại: {e}");
            ExitCode::FAILURE
        }
    }
}

fn hex(b: &[u8]) -> String {
    use core::fmt::Write as _;
    let mut ra = String::with_capacity(b.len() * 2);
    for x in b {
        let _ = write!(ra, "{x:02x}");
    }
    ra
}
