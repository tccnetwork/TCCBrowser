//! Kiểm mắt xích CÚ BẤM, đi qua WebKit thật.
//!
//! Chạy: `cargo run -p tcc-shell --features window --example kiem-bam-nut`
//!
//! Nằm ở `examples/` vì trên macOS vòng lặp sự kiện bắt buộc chạy trên luồng
//! chính, mà bộ khung kiểm thử của Rust chạy trên luồng phụ.

#![allow(
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay, không nuốt lỗi"
)]

use std::{process::ExitCode, time::Duration};

use tcc_capability::Decision;
use tcc_render_webview::{WebViewRenderer, window, window::MessageKind};
use tcc_shell::{
    Language,
    permission_dialog::{self, ACTION_ALLOW, ACTION_DENY, decide},
};
use tcc_spec::Manifest;
use tcc_ui::Renderer as _;

fn ke_khai() -> Manifest {
    let s = format!(
        r#"{{"spec_version":"0.1","id":"com.tcc.thu","name":"TCC Store",
"version":"1.0.0","publisher":"{}","scheme":"hybrid-ed25519-mldsa65-v1",
"content_hash":"{}","entry":"index.html","capabilities":[
{{"name":"wallet","scope":{{"kind":"wallet","may_request_signature":true}},
  "reason":"Pay for your order"}}]}}"#,
        "aa".repeat(1992),
        "bb".repeat(48)
    );
    serde_json::from_str(&s).expect("bản kê khai mẫu hỏng")
}

fn main() -> ExitCode {
    let m = ke_khai();
    let cay = match permission_dialog::build(&m, Language::En) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("✗ không dựng được hộp thoại: {e}");
            return ExitCode::FAILURE;
        }
    };
    let hop_le: Vec<String> = cay
        .action_ids()
        .iter()
        .map(|a| a.as_str().to_owned())
        .collect();

    let mut bd = WebViewRenderer::new();
    if let Err(e) = bd.render(&cay) {
        eprintln!("✗ không vẽ được: {e}");
        return ExitCode::FAILURE;
    }

    // Chỉ bấm được MỘT nút mỗi lần chạy: trên macOS một tiến trình chỉ dựng
    // được một vòng lặp sự kiện. Chọn nút theo đối số.
    let doi = std::env::args()
        .nth(1)
        .unwrap_or_else(|| ACTION_ALLOW.to_owned());

    // Chế độ "ma": gửi thẳng một mã bịa ra. Danh sách trắng phải vứt nó đi.
    if doi == "ma" {
        return kiem_danh_sach_trang(bd.document(), &hop_le);
    }
    // Chế độ "bat": bật công tắc THẬT trong WebKit rồi mới bấm xác nhận.
    if doi == "bat" {
        return kiem_cong_tac(bd.document(), &hop_le);
    }
    // Chế độ "ct-ma": hành động hợp lệ nhưng kèm một công tắc bịa.
    if doi == "ct-ma" {
        return kiem_cong_tac_ma(bd.document(), &hop_le);
    }

    let can_bam = doi;
    let mong_doi = if can_bam == ACTION_ALLOW {
        Decision::Allow
    } else {
        Decision::Deny
    };

    println!("Tự bấm nút {can_bam:?} trong WebKit thật…");
    let label = match window::simulate_click(
        bd.document(),
        &hop_le,
        &can_bam,
        MessageKind::Bam,
        Duration::from_secs(20),
    ) {
        Ok(n) => n.map(|t| t.hanh_dong),
        Err(e) => {
            eprintln!("✗ {e}");
            return ExitCode::FAILURE;
        }
    };

    println!("  nhận về  : {label:?}");
    if label.as_deref() == Some("KHONG-TIM-THAY-NUT") {
        eprintln!("✗ HỎNG: không có nút nào mang mã {can_bam:?} trên màn hình");
        return ExitCode::FAILURE;
    }
    if label.is_none() {
        eprintln!(
            "✗ HỎNG: bấm rồi mà không nhận về gì — kịch bản nối sự kiện đứt. \
             Nút bấm vào sẽ KHÔNG ăn, và không có lỗi nào hiện ra."
        );
        return ExitCode::FAILURE;
    }

    // Bấm nút mà KHÔNG bật công tắc nào — nên kể cả "cho phép" cũng phải ra Deny
    // ở mọi quyền. Đó chính là điểm khác của việc hỏi theo từng mục.
    let khong_bat: Vec<String> = Vec::new();
    let qd = decide(label.as_deref(), &khong_bat, "wallet");
    println!("  quyết định khi KHÔNG bật công tắc: {qd:?} (luôn phải là Deny)");
    if qd != Decision::Deny {
        eprintln!("✗ HỎNG: bấm nút mà không bật công tắc vẫn cấp quyền");
        return ExitCode::FAILURE;
    }

    let da_bat = vec![tcc_shell::permission_dialog::toggle_id("wallet")];
    let qd = decide(label.as_deref(), &da_bat, "wallet");
    println!("  quyết định khi ĐÃ bật công tắc  : {qd:?} (mong đợi {mong_doi:?})");
    if qd == mong_doi {
        println!(
            "✓ Cú bấm đi hết đường: WebKit → kịch bản nối sự kiện → danh sách trắng → quyết định."
        );
        ExitCode::SUCCESS
    } else {
        eprintln!("✗ HỎNG: quyết định sai");
        ExitCode::FAILURE
    }
}

/// ⚠️ Kiểm mắt xích CÔNG TẮC, đi qua WebKit thật.
///
/// Công tắc đổi trạng thái tại chỗ, và trạng thái đó phải đi kèm cú bấm nút xác
/// nhận. Đứt ở đây thì người dùng bật công tắc, bấm cho phép, và **không quyền
/// nào được cấp** — mà cũng không có lỗi nào hiện ra. Đúng loại lỗi im lặng
/// khiến người dùng nghĩ trình duyệt hỏng.
fn kiem_cong_tac(document: &str, hop_le: &[String]) -> ExitCode {
    let ma_ct = tcc_shell::permission_dialog::toggle_id("wallet");
    println!("Bật công tắc {ma_ct:?} rồi bấm {ACTION_ALLOW:?} trong WebKit thật…");

    let label = match window::simulate_click(
        document,
        hop_le,
        ACTION_ALLOW,
        MessageKind::BatRoiBam {
            cong_tac: "q-wallet",
        },
        Duration::from_secs(20),
    ) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("✗ {e}");
            return ExitCode::FAILURE;
        }
    };

    let Some(t) = label else {
        eprintln!("✗ HỎNG: bật rồi bấm mà không nhận về gì");
        return ExitCode::FAILURE;
    };
    println!("  nút đã bấm      : {:?}", t.hanh_dong);
    println!("  công tắc đang bật: {:?}", t.bat);

    if !t.bat.contains(&ma_ct) {
        eprintln!(
            "✗ HỎNG: bật công tắc rồi mà trạng thái KHÔNG về tới host — người \
             dùng bấm cho phép nhưng không quyền nào được cấp"
        );
        return ExitCode::FAILURE;
    }

    let qd = decide(Some(&t.hanh_dong), &t.bat, "wallet");
    println!("  quyền ví         : {qd:?} (mong đợi Allow)");
    // Quyền KHÔNG bật phải vẫn bị từ chối — đó là điểm của việc hỏi từng mục.
    let qd_khac = decide(Some(&t.hanh_dong), &t.bat, "network");
    println!("  quyền mạng       : {qd_khac:?} (mong đợi Deny — không bật)");

    if qd == Decision::Allow && qd_khac == Decision::Deny {
        println!("✓ Công tắc đi hết đường, và bật một quyền KHÔNG kéo theo quyền khác.");
        ExitCode::SUCCESS
    } else {
        eprintln!("✗ HỎNG: quyết định sai");
        ExitCode::FAILURE
    }
}

/// ⚠️ Kiểm danh sách trắng áp cho CÔNG TẮC, không chỉ cho hành động.
///
/// Đòn này tinh vi hơn `ma`: hành động qua được danh sách trắng, chỉ danh sách
/// công tắc là bịa. Nếu bộ lọc lọc bớt công tắc lạ rồi cho phần còn lại đi tiếp,
/// thì trang có thể tự cấp cho mình một quyền mà người dùng chưa hề bật.
///
/// Bộ lọc phải vứt **cả thông điệp**: một thông điệp đã pha tạp thì không phần
/// nào của nó đáng tin.
fn kiem_cong_tac_ma(document: &str, hop_le: &[String]) -> ExitCode {
    const CT_MA: &str = "q-camera";
    println!("Gửi hành động HỢP LỆ kèm công tắc bịa {CT_MA:?}…");

    let label = match window::simulate_click(
        document,
        hop_le,
        ACTION_ALLOW,
        MessageKind::CongTacMa { cong_tac: CT_MA },
        Duration::from_secs(8),
    ) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("✗ {e}");
            return ExitCode::FAILURE;
        }
    };

    println!("  nhận về: {label:?}");
    match label {
        None => {
            println!("✓ Cả thông điệp bị vứt. Công tắc ma không cấp được quyền nào.");
            ExitCode::SUCCESS
        }
        Some(t) => {
            eprintln!(
                "✗ HỎNG: thông điệp pha tạp vẫn đi tiếp (bật: {:?}) — trang tự cấp \
                 được quyền mà người dùng chưa bấm",
                t.bat
            );
            ExitCode::FAILURE
        }
    }
}

/// ⚠️ Kiểm DANH SÁCH TRẮNG: một mã bịa ra phải bị vứt lặng lẽ.
///
/// Không có phép thử này thì gỡ bỏ danh sách trắng đi mà mọi phép thử khác vẫn
/// xanh — nới lỏng một bộ lọc là loại đột biến mà phép thử chỉ-gửi-dữ-liệu-hợp-lệ
/// không bao giờ chạm tới. Tôi phát hiện lỗ này đúng bằng cách thử gỡ nó ra.
fn kiem_danh_sach_trang(document: &str, hop_le: &[String]) -> ExitCode {
    const MA_MA: &str = "cho-phep-tat-ca";
    println!("Gửi thẳng mã bịa {MA_MA:?} (không có nút nào mang mã này)…");

    let label = match window::simulate_click(
        document,
        hop_le,
        MA_MA,
        MessageKind::GuiThang,
        Duration::from_secs(8),
    ) {
        Ok(n) => n.map(|t| t.hanh_dong),
        Err(e) => {
            eprintln!("✗ {e}");
            return ExitCode::FAILURE;
        }
    };

    println!("  nhận về: {label:?}");
    match label {
        None => {
            println!("✓ Danh sách trắng vứt mã lạ. Hành động ma không đi tiếp được.");
            ExitCode::SUCCESS
        }
        Some(m) => {
            eprintln!(
                "✗ HỎNG: mã bịa {m:?} lọt qua danh sách trắng — trang có thể tự \
                 sinh ra một quyết định mà người dùng chưa hề bấm"
            );
            ExitCode::FAILURE
        }
    }
}

// Giữ tham chiếu để đổi tên hằng số là phải sửa cả ví dụ này.
const _: &str = ACTION_DENY;
