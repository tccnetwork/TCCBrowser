//! Kiểm khói ĐỐI KHÁNG, đi qua WebKit thật.
//!
//! Chạy: `cargo run -p tcc-shell --features cua-so --example kiem-khoi-tan-cong`
//!
//! # Vì sao là `examples/` chứ không phải `tests/`
//!
//! Trên macOS vòng lặp sự kiện BẮT BUỘC chạy trên luồng chính. Bộ khung kiểm thử
//! của Rust chạy mỗi phép thử trên một luồng phụ, nên `#[test]` sẽ chết ngay khi
//! dựng cửa sổ. Ví dụ thì chạy thẳng trên luồng chính.
//!
//! # Phép thử này chứng minh cái gì
//!
//! Mọi phép thử trong thư viện chạy trên chuỗi đánh dấu do TA sinh và TA đọc lại.
//! Hai đường khác nhau, nhưng vẫn là hai đường của ta. Ở đây ta đưa một bản kê
//! khai THÙ ĐỊCH đi hết đường ống rồi hỏi WEBKIT — bên thứ ba, không biết gì về
//! ý định của ta — xem nó nhìn thấy gì.
//!
//! Thẻ kịch bản còn sống > 0 nghĩa là ứng dụng chạy được mã trong ngữ cảnh bộ
//! dựng, tức là thoát khỏi toàn bộ mô hình quyền năng. Đó là hỏng nặng nhất có
//! thể, và đây là chỗ duy nhất phát hiện được nó một cách chắc chắn.

use std::{process::ExitCode, time::Duration};

use tcc_shell::{NgonNgu, cua_so};
use tcc_spec::Manifest;

/// Bản kê khai với mọi trường hiện ra màn hình đều nhồi mã tấn công.
///
/// Lưu ý: `tcc-spec` CHO QUA những chuỗi này, và đúng là phải cho qua — chúng
/// không chứa ký tự giả mạo hiển thị nào. Chặn nằm ở tầng dịch đánh dấu.
fn ke_khai_thu_dich() -> Result<Manifest, Box<dyn std::error::Error>> {
    let s = format!(
        r#"{{
  "spec_version": "0.1",
  "id": "com.tcc.ke-gian",
  "name": "<script>window.doi_vi=1</script>",
  "version": "1.0.0",
  "publisher": "{}",
  "scheme": "hybrid-ed25519-mldsa65-v1",
  "content_hash": "{}",
  "entry": "index.html",
  "capabilities": [
    {{
      "name": "network",
      "scope": {{ "kind": "network", "hosts": ["a.tcc-coin.com"] }},
      "reason": "\" onmouseover=\"window.doi_vi=2"
    }},
    {{
      "name": "wallet",
      "scope": {{ "kind": "wallet", "may_request_signature": true }},
      "reason": "</button><script>window.doi_vi=3</script>"
    }}
  ]
}}"#,
        "aa".repeat(1992),
        "bb".repeat(48)
    );
    Ok(serde_json::from_str(&s)?)
}

/// Tài liệu ĐỘC viết tay, KHÔNG đi qua bộ dịch đánh dấu.
///
/// Dùng để kiểm RIÊNG tầng phòng thủ thứ hai. Ở đường ống thật, tầng thoát ký tự
/// và bộ quét trợ năng chặn trước, nên chính sách nội dung không bao giờ được
/// thử sức. Ở đây ta giả định cả hai tầng trên đã thủng — nếu chính sách nội
/// dung cũng thủng nốt thì ta chỉ còn MỘT tầng phòng thủ chứ không phải ba.
fn tai_lieu_doc() -> String {
    format!(
        "<!doctype html><meta charset=\"utf-8\">\
         <meta http-equiv=\"Content-Security-Policy\" content=\"\
         default-src 'none'; img-src tcc-goi:; style-src 'unsafe-inline'; \
         script-src 'none'; object-src 'none'; frame-src 'none'; \
         form-action 'none'; base-uri 'none'\">\
         <body><div role=\"group\">{}</div></body>",
        "<script>window.da_chay = 1;</script>\
         <img src=\"x\" role=\"presentation\" onerror=\"window.da_chay = 2\">"
    )
}

fn chi_csp() -> ExitCode {
    println!("Chỉ kiểm CHÍNH SÁCH NỘI DUNG — giả định hai tầng trên đã thủng.");
    let bao = match cua_so::kiem_tai_lieu_tho(&tai_lieu_doc(), Duration::from_secs(20)) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("✗ WebKit không báo về: {e}");
            return ExitCode::FAILURE;
        }
    };
    println!("  thẻ kịch bản CÓ MẶT trong tài liệu: {}", bao.so_kich_ban);
    println!("  (có mặt là đúng — ta cố ý nhét vào; câu hỏi là nó có CHẠY không)");

    if bao.da_chay {
        eprintln!("✗ HỎNG NẶNG: kịch bản CHẠY ĐƯỢC — chính sách nội dung vô hiệu");
        ExitCode::FAILURE
    } else {
        println!("✓ Kịch bản có trong tài liệu nhưng KHÔNG chạy — chính sách nội dung giữ được.");
        ExitCode::SUCCESS
    }
}

fn main() -> ExitCode {
    if std::env::args().nth(1).as_deref() == Some("chi-csp") {
        return chi_csp();
    }
    let m = match ke_khai_thu_dich() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("✗ bản kê khai thù địch không dựng được: {e}");
            return ExitCode::FAILURE;
        }
    };

    let cay = match tcc_shell::hop_thoai_quyen::dung(&m, NgonNgu::En) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("✗ không dựng được hộp thoại: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mong_doi = cay.node_count();

    let bao = match cua_so::kiem_khoi(&m, NgonNgu::En, Duration::from_secs(20)) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("✗ WebKit không báo về: {e}");
            return ExitCode::FAILURE;
        }
    };

    println!("Bản kê khai thù địch đã đi hết đường ống. WebKit báo về:");
    println!("  số nút ta dựng        : {mong_doi}");
    println!("  số nút WebKit nhìn thấy: {}", bao.so_nut);
    println!("  thẻ kịch bản còn sống  : {}", bao.so_kich_ban);
    println!();

    let mut hong = 0;

    if bao.so_kich_ban != 0 {
        eprintln!(
            "✗ HỎNG NẶNG: WebKit thấy {} thẻ kịch bản — ứng dụng chạy được mã \
             trong ngữ cảnh bộ dựng, mô hình quyền năng vô hiệu",
            bao.so_kich_ban
        );
        hong += 1;
    }

    // Số nút lệch nghĩa là đánh dấu bị phá cấu trúc: ứng dụng đóng sớm một thẻ
    // và đẻ thêm phần tử, hoặc nuốt mất một phần màn hình.
    if bao.so_nut != mong_doi {
        eprintln!(
            "✗ HỎNG: ta dựng {mong_doi} nút, WebKit thấy {} — cấu trúc đánh dấu \
             bị phá",
            bao.so_nut
        );
        hong += 1;
    }

    if hong == 0 {
        println!("✓ Mọi đòn đều bị chặn: không kịch bản nào sống, cấu trúc nguyên vẹn.");
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
