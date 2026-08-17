//! **Gõ SAI cụm từ thì cửa sổ phải ở lại**, không đóng.
//!
//! ```text
//! cargo run -p tcc-shell --features wallet --example kiem-cum-tu-sai
//! ```
//!
//! # Vì sao cần một ví dụ riêng
//!
//! Ngày 17/08/2026 người dùng gõ sai một chữ và **cả ứng dụng tắt**. Logic
//! quyết định thì đúng — có phép thử thuần chứng minh — nên lỗi nằm ở tầng cửa
//! sổ, và tầng ấy chỉ chạy khi có WebKit thật.
//!
//! Ví dụ này tự gõ một cụm từ sai, tự bấm, rồi hỏi: màn hình có đổi sang màn
//! báo lỗi không, hay cửa sổ đã đóng.

#![allow(
    clippy::expect_used,
    reason = "ví dụ đối kháng: dựng màn hình hỏng thì phải nổ ngay, không đi tiếp"
)]

use std::{
    cell::RefCell,
    process::ExitCode,
    rc::Rc,
    time::{Duration, Instant},
};

use tcc_render_webview::{
    WebViewRenderer,
    window::{Next, Screen, dialog_sequence_driven},
};
use tcc_shell::{
    recovery_screen,
    text::{Language, TextKey, label},
    wallet_flow::{PhraseStep, phrase_step},
};
use tcc_ui::Renderer as _;

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
    const SAI: &str = "day khong phai cum tu khoi phuc";
    let ngon_ngu = Language::Vi;
    let nhan = label(TextKey::CumTuNhan, ngon_ngu);

    // Tự gõ rồi tự bấm — chạy lại sau MỖI lần đổi màn, như người dùng thật.
    let tu_lai = format!(
        r"
document.addEventListener('DOMContentLoaded', function () {{
  var o = document.querySelector('input[aria-label={nhan}]');
  if (!o) {{ return; }}
  o.value = {sai};
  o.dispatchEvent(new Event('input', {{ bubbles: true }}));
  var n = document.querySelector('[data-hanh-dong={nut}]');
  if (n) {{ n.click(); }}
}});
",
        nhan = serde_json::Value::from(nhan),
        sai = serde_json::Value::from(SAI),
        nut = serde_json::Value::from(recovery_screen::ACTION_CONTINUE),
    );

    let so_lan: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
    let dem = Rc::clone(&so_lan);
    let bat_dau = Instant::now();

    let dau = man(
        &recovery_screen::build_entry(None, ngon_ngu).expect("dựng màn nhập"),
        label(TextKey::CumTuTieuDe, ngon_ngu),
    );

    let ket_qua = dialog_sequence_driven(
        &dau,
        move |t| {
            let buoc = phrase_step(&t.hanh_dong, SAI);
            println!("  nhận  : {:?} → {buoc:?}", t.hanh_dong);
            if buoc != PhraseStep::ShowError {
                return Next::Done;
            }
            *dem.borrow_mut() += 1;
            // Hiện lại đủ hai lần là đủ chứng minh cửa sổ ở lại; hơn nữa thì
            // vòng tự bấm chạy mãi.
            if *dem.borrow() >= 2 || bat_dau.elapsed() > Duration::from_secs(20) {
                return Next::Done;
            }
            let cau = label(TextKey::CumTuLoiKhongHopLe, ngon_ngu);
            Next::Show(Box::new(man(
                &recovery_screen::build_entry(Some(cau), ngon_ngu).expect("dựng màn lỗi"),
                label(TextKey::CumTuTieuDe, ngon_ngu),
            )))
        },
        Some(&tu_lai),
    );

    if let Err(e) = ket_qua {
        eprintln!("✗ {e}");
        return ExitCode::FAILURE;
    }
    let lan = *so_lan.borrow();
    println!("  số lần hiện lại màn lỗi: {lan}");
    if lan >= 2 {
        println!("✓ Gõ sai thì màn nhập HIỆN LẠI — cửa sổ ở lại, gõ tiếp được.");
        ExitCode::SUCCESS
    } else {
        println!("✗ Cửa sổ ĐÓNG sau khi gõ sai — người dùng mất cả 24 chữ vừa gõ.");
        ExitCode::FAILURE
    }
}
