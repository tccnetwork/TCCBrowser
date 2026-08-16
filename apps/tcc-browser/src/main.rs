//! TCC Browser — trình duyệt thế hệ mới.
//!
//! Tệp này phải mỏng. Mọi logic nằm trong `tcc-shell`. Đây chỉ là điểm khởi động.
//!
//! ```text
//!   cargo run -p tcc-browser                     → in cây hộp thoại ra chữ
//!   cargo run -p tcc-browser --features window   → mở cửa sổ thật
//! ```

use std::process::ExitCode;

// `permission_dialog` chỉ dùng ở nhánh không có cửa sổ — nhập trong nhánh đó để
// bản có cửa sổ không dính cảnh báo nhập thừa.
use tcc_shell::Language;
use tcc_spec::Manifest;

fn main() -> ExitCode {
    let doi = std::env::args().skip(1).collect::<Vec<_>>();
    let ngon_ngu = if doi.iter().any(|a| a == "vi") {
        Language::Vi
    } else {
        Language::En
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

    // KHÔNG có nhánh "chạy thử bằng dữ liệu tự bịa".
    //
    // Trước 16/08/2026 chỗ này dựng hộp thoại hỏi quyền từ một bản kê khai
    // nhúng cứng — `publisher: ""`, `content_hash: ""`, `entry: "index.html"`
    // và một quyền `wallet`. Chạy binary không tham số là rơi vào đó.
    //
    // Ba thứ hỏng chồng lên nhau, và người soát độc lập bắt được cả ba (F1,
    // 16/08/2026): `entry: "index.html"` mâu thuẫn thẳng với bất biến B15 —
    // "ứng dụng không mang mã, điểm vào là ui.json"; dữ liệu giả nằm trên
    // đường chạy sản phẩm chứ không nằm trong `examples/`; và chú thích ngay
    // tại chỗ hứa "Giai đoạn 2 sẽ thay" trong khi Giai đoạn 2 đã tuyên bố đóng.
    //
    // Bản demo giờ là một GÓI THẬT ĐÃ KÝ. Nó vừa demo tốt hơn, vừa không bao
    // giờ trôi khỏi tiêu chuẩn — vì nếu nó trôi thì `tcc verify` từ chối nó.
    eprintln!("cần một thư mục gói đã ký:");
    eprintln!("    cargo run -p tcc-browser -- examples/hello-tcc");
    eprintln!("    cargo run -p tcc-browser -- quyen <thư-mục-gói>");
    ExitCode::FAILURE
}

#[cfg(feature = "window")]
fn quan_ly(goi: &std::path::Path, ngon_ngu: Language) -> ExitCode {
    match tcc_shell::window::manage_permissions(&goi.join(".tcc-quyen.json"), ngon_ngu) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("✗ {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(feature = "window"))]
use tcc_crypto::HybridEd25519MlDsa;

#[cfg(not(feature = "window"))]
fn quan_ly(_g: &std::path::Path, _n: Language) -> ExitCode {
    eprintln!("✗ màn hình quản lý quyền cần bản dựng có cửa sổ: --features window");
    ExitCode::FAILURE
}

#[cfg(feature = "window")]
fn mo_goi_that(duong_dan: &std::path::Path, ngon_ngu: Language) -> ExitCode {
    // Kho quyền đã cấp, cạnh gói. `TCC_QUEN_HET=1` để bỏ qua và hỏi lại từ đầu.
    let kho = if std::env::var("TCC_QUEN_HET").is_ok() {
        None
    } else {
        Some(duong_dan.join(".tcc-quyen.json"))
    };
    match tcc_shell::window::open_package(duong_dan, ngon_ngu, kho.as_deref()) {
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

/// Bản không cửa sổ: kiểm gói THẬT rồi in cây hộp thoại ra chữ.
///
/// Trước 16/08/2026 nhánh này chỉ báo "cần --features window", còn cây hộp
/// thoại thì dựng từ một bản kê khai tự bịa. Đảo lại: bản kê khai đến từ gói
/// đã ký, và bản không cửa sổ vẫn xem được — đó mới là cách xem hộp thoại trên
/// máy chủ không màn hình.
#[cfg(not(feature = "window"))]
fn mo_goi_that(duong_dan: &std::path::Path, ngon_ngu: Language) -> ExitCode {
    // `verify_package` chạy TRƯỚC mọi thứ khác. Không có đường nào dựng được
    // hộp thoại từ một gói chưa kiểm chữ ký.
    let (app, _) = match tcc_runtime::verify_from_dir(duong_dan, &HybridEd25519MlDsa) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("✗ gói không hợp lệ: {e}");
            return ExitCode::FAILURE;
        }
    };
    println!("✓ chữ ký hợp lệ — \"{}\"", app.manifest().name);
    match run_loop(app.manifest(), ngon_ngu) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("✗ {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(feature = "window")]
fn run_loop(m: &Manifest, ngon_ngu: Language) -> Result<(), Box<dyn std::error::Error>> {
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
        let bao = tcc_shell::window::check_escaping(m, ngon_ngu, Duration::from_secs(15))?;
        println!("WebKit báo về:");
        println!("  số nút mang vai trò : {}", bao.so_nut);
        println!("  vai trò            : {}", bao.vai_tro.join(", "));
        println!("  thẻ kịch bản còn sống: {}", bao.so_kich_ban);
        return Ok(());
    }

    tcc_shell::window::show_permission_dialog(m, ngon_ngu, tu_dong_dong)
}

/// Bản không có cửa sổ: in cây hộp thoại ra chữ.
///
/// Không phải đồ chơi — đây là cách xem hộp thoại hỏi quyền trên máy chủ không
/// màn hình, và là cách so sánh hai bản dịch cạnh nhau.
#[cfg(not(feature = "window"))]
fn run_loop(m: &Manifest, ngon_ngu: Language) -> Result<(), Box<dyn std::error::Error>> {
    let cay = tcc_shell::permission_dialog::build(m, ngon_ngu)?;
    println!(
        "Hộp thoại hỏi quyền — {} nút, sâu {} tầng",
        cay.node_count(),
        cay.depth()
    );
    println!();
    in_cay(&cay.accessibility_tree(), 0);
    println!();
    println!("(dựng bằng `--features window` để mở cửa sổ thật)");
    Ok(())
}

#[cfg(not(feature = "window"))]
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
