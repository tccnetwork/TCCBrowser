//! **Thử cả đường ví trong MỘT phiên** — khoá không cất đi đâu cả.
//!
//! ```text
//! cargo run -p tcc-shell --features "wallet,network" --example thu-vi-mot-phien -- \
//!     <dia-chi-nhan> <so-tien-wei> [ghi-nho]
//! ```
//!
//! # Vì sao có ví dụ này
//!
//! **Cất** khoá đang bị chặn: `USER_PRESENCE` cần hồ sơ cấp phép Apple mà máy
//! chưa có (`docs/vi-thiet-ke.md` §19). Nhưng **ký** thì không bị chặn gì.
//!
//! Nên đây là đường thử đầy đủ chạy được hôm nay: gõ cụm từ → dựng giao dịch
//! thật từ testnet → **kiểm chống ký mù** → xác nhận → ký → gửi. Khoá sống
//! trong bộ nhớ đúng một phiên và không được ghi ở đâu.
//!
//! Nó **không hạ tiêu chuẩn nào**: không có gì được cất bằng cách yếu hơn —
//! không có gì được cất cả. Màn hình nói thẳng điều đó TRƯỚC khi người dùng gõ.
//!
//! # Cổng chặn cứng
//!
//! Chỉ testnet 91338. Không giao dịch mainnet nào trước kiểm định an ninh độc lập.

#![allow(
    clippy::expect_used,
    clippy::too_many_lines,
    reason = "ví dụ: dựng màn hình hỏng thì phải nổ ngay, không đi tiếp; và cả \
              đường ví nằm trong MỘT bao đóng vì `tao` chỉ cho một vòng lặp sự \
              kiện mỗi tiến trình — cắt nhỏ ra là phải chuyền trạng thái qua \
              nhiều `Rc<RefCell<…>>`, khó đọc hơn hẳn"
)]

use std::{cell::RefCell, process::ExitCode, rc::Rc};

use tcc_chain::wallet::WalletSecret;
use tcc_net::rpc::JsonRpc;
use tcc_render_webview::{
    WebViewRenderer,
    window::{Next, Screen, dialog_sequence},
};
use tcc_shell::{
    recovery_screen, signing_flow,
    text::{Language, TextKey, label},
    transaction_screen,
};
use tcc_ui::Renderer as _;

const CHAIN_TESTNET: u64 = 91338;
const RPC: &str = "https://rpc2.tcc-coin.com";

fn man(cay: &tcc_ui::Node, tieu_de: &str) -> Screen {
    let mut bd = WebViewRenderer::new();
    bd.render(cay).expect("vẽ được");
    Screen {
        document: bd.document().to_owned(),
        title: tieu_de.to_owned(),
        allowed: cay
            .action_ids()
            .iter()
            .map(|a| a.as_str().to_owned())
            .collect(),
    }
}

fn main() -> ExitCode {
    let doi: Vec<String> = std::env::args().skip(1).collect();
    let (Some(nhan), Some(tien)) = (doi.first(), doi.get(1)) else {
        eprintln!("cần: <dia-chi-nhan> <so-tien-wei> [ghi-nho]");
        return ExitCode::FAILURE;
    };
    let ghi_nho = doi.get(2).cloned().unwrap_or_default();
    let ngon_ngu = Language::Vi;
    let nhan_o = label(TextKey::CumTuNhan, ngon_ngu).to_owned();

    let rpc = match JsonRpc::new(RPC) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("✗ {e}");
            return ExitCode::FAILURE;
        }
    };

    let ket_qua: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let ra = Rc::clone(&ket_qua);
    let nhan_dich = nhan.clone();
    let so_tien = tien.clone();
    // Khoá của phiên. Sống trong bao đóng, chết cùng nó.
    let mut khoa_phien: Option<WalletSecret> = None;
    let mut cho_ky: Option<signing_flow::PendingTransaction> = None;
    let mut goi_b64 = String::new();

    let dau = man(
        &recovery_screen::build_session_entry(None, ngon_ngu).expect("dựng màn gõ"),
        label(TextKey::PhienTieuDe, ngon_ngu),
    );

    let chay = dialog_sequence(&dau, move |t| {
        // ── Gõ cụm từ → dựng giao dịch thật → kiểm chống ký mù ──
        if t.hanh_dong == recovery_screen::ACTION_CONTINUE {
            let go = t
                .o_nhap
                .iter()
                .find(|(n, _)| *n == nhan_o)
                .map(|(_, v)| v.clone())
                .unwrap_or_default();
            let Ok(khoa) = recovery_screen::read_phrase(&go) else {
                let cau = label(TextKey::CumTuLoiKhongHopLe, ngon_ngu);
                let c = recovery_screen::build_session_entry(Some(cau), ngon_ngu)
                    .expect("dựng lại màn gõ");
                return Next::Show(Box::new(man(&c, label(TextKey::PhienTieuDe, ngon_ngu))));
            };
            let gui = khoa.address().to_string();
            println!("ví của phiên: {gui}");

            let tra = match rpc.call(
                "tcc_buildUnsignedTransfer",
                &serde_json::json!([gui, nhan_dich, so_tien, ghi_nho]),
            ) {
                Ok(v) => v,
                Err(e) => {
                    return hong(&format!("máy chủ không dựng được giao dịch: {e}"), ngon_ngu);
                }
            };
            let (Some(tx_hex), Some(bam_hex), Some(b64)) = (
                tra.get("unsigned_tx_hex").and_then(|v| v.as_str()),
                tra.get("signing_message_hex").and_then(|v| v.as_str()),
                tra.get("unsigned_tx_base64").and_then(|v| v.as_str()),
            ) else {
                return hong("phản hồi thiếu trường", ngon_ngu);
            };

            // ⚠️ Kiểm TRƯỚC khi vẽ. Lệch thì không có màn xác nhận nào.
            let (cho, cay) = match signing_flow::review(tx_hex, bam_hex, ngon_ngu) {
                Ok(x) => x,
                Err(e) => return hong(&format!("TỪ CHỐI: {e}"), ngon_ngu),
            };
            if cho.transaction().chain_id != CHAIN_TESTNET {
                return hong("không phải testnet — từ chối", ngon_ngu);
            }
            khoa_phien = Some(khoa);
            b64.clone_into(&mut goi_b64);
            cho_ky = Some(cho);
            return Next::Show(Box::new(man(&cay, "Xác nhận giao dịch")));
        }

        // ── Bấm ký ──
        if t.hanh_dong == transaction_screen::ACTION_SIGN {
            let (Some(khoa), Some(cho)) = (khoa_phien.as_ref(), cho_ky.take()) else {
                return Next::Done;
            };
            let chu_ky_wire = cho.sign(khoa);
            let cong_khai = khoa.public_key();
            return match rpc.call(
                "tcc_submitSignedTransfer",
                &serde_json::json!([
                    goi_b64,
                    hex(chu_ky_wire.as_bytes()),
                    hex(cong_khai.as_bytes())
                ]),
            ) {
                Ok(v) => {
                    let ma = v
                        .get("tx_hash")
                        .and_then(|x| x.as_str())
                        .unwrap_or("(máy chủ không trả mã)")
                        .to_owned();
                    *ra.borrow_mut() = Some(ma.clone());
                    // ⚠️ Hiện TRONG cửa sổ. Đóng cửa sổ ngay sau khi gửi là để
                    // người dùng không biết tiền đã đi hay chưa — trạng thái tệ
                    // nhất một ví có thể để lại.
                    let c = transaction_screen::build_sent(&ma, ngon_ngu).expect("dựng màn đã gửi");
                    Next::Show(Box::new(man(&c, label(TextKey::XongTieuDe, ngon_ngu))))
                }
                Err(e) => hong(&format!("gửi thất bại: {e}"), ngon_ngu),
            };
        }

        Next::Done
    });

    if let Err(e) = chay {
        eprintln!("✗ {e}");
        return ExitCode::FAILURE;
    }
    if let Some(v) = ket_qua.borrow().as_ref() {
        println!("✓ đã gửi: {v}");
    } else {
        println!("Chưa gửi gì. Khoá của phiên đã mất theo cửa sổ.");
    }
    ExitCode::SUCCESS
}

fn hong(cau: &str, ngon_ngu: Language) -> Next {
    eprintln!("✗ {cau}");
    let c = recovery_screen::build_failure(cau, ngon_ngu).expect("dựng màn hỏng");
    Next::Show(Box::new(man(&c, label(TextKey::HongTieuDe, ngon_ngu))))
}

fn hex(b: &[u8]) -> String {
    use core::fmt::Write as _;
    let mut ra = String::with_capacity(b.len() * 2);
    for x in b {
        let _ = write!(ra, "{x:02x}");
    }
    ra
}
