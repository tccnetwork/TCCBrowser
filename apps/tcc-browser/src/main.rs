//! TCC Browser — trình duyệt thế hệ mới.
//!
//! Tệp này phải mỏng. Mọi logic nằm trong `tcc-shell`. Đây chỉ là điểm khởi động.
//!
//! ```text
//!   cargo run -p tcc-browser                     → in cây hộp thoại ra chữ
//!   cargo run -p tcc-browser --features cua-so   → mở cửa sổ thật
//! ```

use std::process::ExitCode;

// `hop_thoai_quyen` chỉ dùng ở nhánh không có cửa sổ — nhập trong nhánh đó để
// bản có cửa sổ không dính cảnh báo nhập thừa.
use tcc_shell::NgonNgu;
use tcc_spec::Manifest;

/// Bản kê khai mẫu để xem thử hộp thoại hỏi quyền.
///
/// TẠM THỜI: Giai đoạn 2 sẽ thay bằng gói thật nạp qua `tcc-runtime`, và lúc đó
/// hộp thoại chỉ được dựng SAU khi chữ ký đã hợp lệ.
const KE_KHAI_MAU: &str = r#"{
  "spec_version": "0.1",
  "id": "com.tcc.cua-hang",
  "name": "TCC Store",
  "version": "1.0.0",
  "publisher": "",
  "scheme": "hybrid-ed25519-mldsa65-v1",
  "content_hash": "",
  "entry": "index.html",
  "capabilities": [
    {
      "name": "network",
      "scope": { "kind": "network", "hosts": ["shop.tcc-coin.com"] },
      "reason": "Load the product list"
    },
    {
      "name": "wallet",
      "scope": { "kind": "wallet", "may_request_signature": true },
      "reason": "Pay for your order"
    }
  ]
}"#;

fn main() -> ExitCode {
    let doi = std::env::args().skip(1).collect::<Vec<_>>();
    let ngon_ngu = if doi.iter().any(|a| a == "vi") {
        NgonNgu::Vi
    } else {
        NgonNgu::En
    };

    // `quyen <thư-mục-gói>` — mở màn hình quản lý quyền đã cấp.
    if doi.first().map(String::as_str) == Some("quyen") {
        let Some(d) = doi.get(1) else {
            eprintln!("cần đường dẫn thư mục gói: tcc-browser quyen <thư-mục>");
            return ExitCode::FAILURE;
        };
        return quan_ly(std::path::Path::new(d), ngon_ngu);
    }

    // Đường dẫn gói THẬT trên đĩa. Đây là đường ống đầy đủ: kiểm chữ ký → hỏi
    // người dùng → cấp quyền → nội dung điểm vào.
    if let Some(duong_dan) = doi
        .iter()
        .find(|a| !a.starts_with('-') && *a != "vi" && *a != "quyen")
    {
        return mo_goi_that(std::path::Path::new(duong_dan), ngon_ngu);
    }

    let m: Manifest = match serde_json::from_str(KE_KHAI_MAU) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("✗ bản kê khai mẫu hỏng: {e}");
            return ExitCode::FAILURE;
        }
    };

    match chay(&m, ngon_ngu) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("✗ {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(feature = "cua-so")]
fn quan_ly(goi: &std::path::Path, ngon_ngu: NgonNgu) -> ExitCode {
    match tcc_shell::cua_so::quan_ly_quyen(&goi.join(".tcc-quyen.json"), ngon_ngu) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("✗ {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(feature = "cua-so"))]
fn quan_ly(_g: &std::path::Path, _n: NgonNgu) -> ExitCode {
    eprintln!("✗ màn hình quản lý quyền cần bản dựng có cửa sổ: --features cua-so");
    ExitCode::FAILURE
}

#[cfg(feature = "cua-so")]
fn mo_goi_that(duong_dan: &std::path::Path, ngon_ngu: NgonNgu) -> ExitCode {
    // Kho quyền đã cấp, cạnh gói. `TCC_QUEN_HET=1` để bỏ qua và hỏi lại từ đầu.
    let kho = if std::env::var("TCC_QUEN_HET").is_ok() {
        None
    } else {
        Some(duong_dan.join(".tcc-quyen.json"))
    };
    match tcc_shell::cua_so::mo_goi(duong_dan, ngon_ngu, kho.as_deref()) {
        Ok(app) => {
            let m = app.manifest();
            println!("✓ Đã nạp \"{}\" ({})", m.name, m.id.as_str());
            println!(
                "  điểm vào : {} ({} byte)",
                m.entry,
                app.entry_content().len()
            );
            println!(
                "  quyền mạng: {}",
                if app.capabilities().network().is_some() {
                    "ĐƯỢC CẤP"
                } else {
                    "không"
                }
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("✗ {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(feature = "cua-so"))]
fn mo_goi_that(_d: &std::path::Path, _n: NgonNgu) -> ExitCode {
    eprintln!("✗ mở gói cần bản dựng có cửa sổ: --features cua-so");
    ExitCode::FAILURE
}

#[cfg(feature = "cua-so")]
fn chay(m: &Manifest, ngon_ngu: NgonNgu) -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Duration;

    // `TCC_TU_DONG_DONG=3` để kiểm khói tự động — không có nó thì cửa sổ chờ
    // người bấm và mọi lệnh chạy tự động sẽ treo.
    let tu_dong_dong = std::env::var("TCC_TU_DONG_DONG")
        .ok()
        .and_then(|s| s.parse().ok())
        .map(Duration::from_secs);

    // `TCC_KIEM_KHOI=1`: không mở màn hình cho người xem, mà nạp tài liệu vào
    // WebKit thật rồi bảo WebKit kể lại nó nhìn thấy gì. Đây là cách kiểm được
    // trên máy không có quyền chụp màn hình.
    if std::env::var("TCC_KIEM_KHOI").is_ok() {
        let bao = tcc_shell::cua_so::kiem_khoi(m, ngon_ngu, Duration::from_secs(15))?;
        println!("WebKit báo về:");
        println!("  số nút mang vai trò : {}", bao.so_nut);
        println!("  vai trò            : {}", bao.vai_tro.join(", "));
        println!("  thẻ kịch bản còn sống: {}", bao.so_kich_ban);
        return Ok(());
    }

    tcc_shell::cua_so::hien_hop_thoai_quyen(m, ngon_ngu, tu_dong_dong)
}

/// Bản không có cửa sổ: in cây hộp thoại ra chữ.
///
/// Không phải đồ chơi — đây là cách xem hộp thoại hỏi quyền trên máy chủ không
/// màn hình, và là cách so sánh hai bản dịch cạnh nhau.
#[cfg(not(feature = "cua-so"))]
fn chay(m: &Manifest, ngon_ngu: NgonNgu) -> Result<(), Box<dyn std::error::Error>> {
    let cay = tcc_shell::hop_thoai_quyen::dung(m, ngon_ngu)?;
    println!(
        "Hộp thoại hỏi quyền — {} nút, sâu {} tầng",
        cay.node_count(),
        cay.depth()
    );
    println!();
    in_cay(&cay.accessibility_tree(), 0);
    println!();
    println!("(dựng bằng `--features cua-so` để mở cửa sổ thật)");
    Ok(())
}

#[cfg(not(feature = "cua-so"))]
fn in_cay(a: &tcc_shell::AccessNode, tang: usize) {
    let lui = "  ".repeat(tang);
    match &a.label {
        Some(l) => println!("{lui}{:?} — {l}", a.role),
        None => println!("{lui}{:?}", a.role),
    }
    for c in &a.children {
        in_cay(c, tang + 1);
    }
}
