//! Chạy phép kiểm chống ký mù trên MỘT PHẢN HỒI RPC THẬT.
//!
//! ```text
//! cargo run -p tcc-shell --example kiem-chong-ky-mu -- <unsigned_tx_hex> <signing_message_hex>
//! ```
//!
//! # Vì sao cần một ví dụ riêng thay vì một phép thử
//!
//! Phép thử dùng gói tin tôi tự chép vào mã. Ví dụ này nhận gói tin **máy chủ
//! vừa trả về**, nên nó trả lời một câu khác hẳn: bộ giải mã có đọc được thứ
//! chuỗi thật phát ra hôm nay không — với `nonce` thật, `expires_at` thật,
//! `timestamp` thật, memo có dấu cách.
//!
//! Không cần khoá nào. Toàn bộ phép kiểm chạy xong TRƯỚC khi khoá được dùng tới,
//! và đó chính là điểm của thiết kế.

use std::process::ExitCode;

use tcc_shell::{signing_flow, text::Language};

fn main() -> ExitCode {
    let doi: Vec<String> = std::env::args().skip(1).collect();
    let (Some(tx_hex), Some(msg_hex)) = (doi.first(), doi.get(1)) else {
        eprintln!("cần hai tham số: <unsigned_tx_hex> <signing_message_hex>");
        return ExitCode::FAILURE;
    };

    match signing_flow::review(tx_hex, msg_hex, Language::Vi) {
        Ok((cho, man)) => {
            let tx = cho.transaction();
            println!("✓ băm tự tính KHỚP băm máy chủ đưa");
            println!("  mạng      : {}", tx.chain_id);
            println!("  người gửi : {}", tx.from);
            println!("  người nhận: {}", tx.to);
            println!(
                "  số tiền   : {} TCC",
                tcc_shell::transaction_screen::format_amount(tx.amount)
            );
            println!("  thứ tự    : {}", tx.nonce);
            println!("  hết hạn   : {}", tx.expires_at);
            println!("  ghi nhớ   : {:?}", tx.memo);
            println!("  màn xác nhận: {} nút", man.node_count());
            println!();
            println!("Chưa ký gì cả — phép kiểm này KHÔNG cần khoá.");
            ExitCode::SUCCESS
        }
        Err(e) => {
            println!("✗ TỪ CHỐI: {e}");
            println!();
            println!("Không có màn hình nào được dựng, và không có gì để ký.");
            ExitCode::FAILURE
        }
    }
}
