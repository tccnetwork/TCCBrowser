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

    // `web <https://…>` — TẦNG 2: mở một trang web thật.
    if doi.first().map(String::as_str) == Some("web") {
        return lenh_web(&doi, ngon_ngu);
    }

    // `corpus <tệp>` — chạy bộ trang thật, đếm chắn. Xem `corpus/50-trang.txt`.
    if doi.first().map(String::as_str) == Some("corpus") {
        return lenh_corpus(&doi);
    }

    // `vi nhap <tệp>` — nhập ví từ bản kết xuất của ví web, NGAY TRONG cửa sổ.
    if doi.first().map(String::as_str) == Some("vi") {
        return lenh_vi(&doi, ngon_ngu);
    }

    // `hop-thoai <thư-mục-gói>` — chỉ xem HỘP THOẠI HỎI QUYỀN, không mở ứng dụng.
    //
    // Giữ đường này sau khi `mo_goi_that` chuyển sang mở màn hình ứng dụng:
    // nó là chỗ duy nhất chạy được `TCC_KIEM_KHOI` (bảo WebKit kể lại nó nhìn
    // thấy gì) và `TCC_TU_DONG_DONG` (tự đóng để kiểm khói không treo).
    if doi.first().map(String::as_str) == Some("hop-thoai") {
        let Some(d) = doi.get(1) else {
            eprintln!("cần đường dẫn thư mục gói: tcc-browser hop-thoai <thư-mục>");
            return ExitCode::FAILURE;
        };
        return xem_hop_thoai(std::path::Path::new(d), ngon_ngu);
    }

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
    if let Some(duong_dan) = doi.iter().find(|a| {
        !a.starts_with('-') && *a != "vi" && *a != "quyen" && *a != "hop-thoai" && *a != "web"
    }) {
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
    let app = match tcc_shell::window::open_package(duong_dan, ngon_ngu, kho.as_deref()) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("✗ {e}");
            return ExitCode::FAILURE;
        }
    };

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

    // ⚠️ MÀN HÌNH ỨNG DỤNG — thiếu đúng lời gọi này cho tới 17/08/2026.
    //
    // `open_package` chỉ kiểm chữ ký và hỏi quyền. Trước đây `main` dừng ngay
    // sau đó, nên trình duyệt nạp gói, hỏi quyền, in ba dòng rồi THOÁT: lần đầu
    // chạy thì thấy hộp thoại quyền và tưởng là xong, lần sau có kho quyền rồi
    // thì cửa sổ không hiện ra nữa và không ai hiểu vì sao.
    //
    // `run_app` chứ không `show_app`: người dùng bấm được nút, và mỗi cú bấm đi
    // qua đúng cổng quyền năng ở `tcc-runtime`.
    let mang = mang_that();
    if let Err(e) = tcc_shell::window::run_app(&app, ngon_ngu, mang.as_ref()) {
        eprintln!("✗ {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

/// Đường ra ngoài thật.
///
/// Cờ `window` của crate này kéo theo `tcc-shell/network` (xem `Cargo.toml`),
/// nên có cửa sổ là có mạng — không có nhánh "cửa sổ nhưng không mạng".
///
/// Bản đầu tôi viết hai nhánh `cfg` cho hai trường hợp, và clippy chỉ ra rằng
/// crate này **không có** cờ `network` nào để mà hỏi. Một nhánh `cfg` hỏi về
/// một cờ không tồn tại thì im lặng không bao giờ chạy — đúng loại mã trông như
/// có phòng bị mà không phòng gì.
/// `web <https://…>` — **tầng 2**: mở một trang web thật.
///
/// # Ba điều nói thẳng
///
/// 1. Trang web **mang mã của nó**. Không chữ ký, không cổng quyền năng.
/// 2. Nó chạy trong một WebView **riêng**, không có IPC và không có kịch bản
///    của khung — nếu chung thì trang gọi được `postMessage` của ta.
/// 3. **Chỉ `https://`.** `http://` bị từ chối: trang tải qua đường trần thì ai
///    trên đường cũng sửa được, mà ta lại đặt nó trong cửa sổ mang tên TCC.
#[cfg(feature = "window")]
fn lenh_web(doi: &[String], ngon_ngu: Language) -> ExitCode {
    let Some(url) = doi.get(1) else {
        eprintln!("cần: tcc-browser web https://…");
        return ExitCode::FAILURE;
    };
    println!("⚠ Tầng 2: trang web mang mã của nó. Ở đó không có thứ gì của TCC che chắn.");
    match tcc_shell::window::open_web(url, ngon_ngu) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("✗ {e}");
            ExitCode::FAILURE
        }
    }
}

/// `corpus <tệp> [giây]` — chạy bộ trang thật.
#[cfg(feature = "window")]
fn lenh_corpus(doi: &[String]) -> ExitCode {
    let tep = doi.get(1).map_or("corpus/50-trang.txt", String::as_str);
    let giay = doi.get(2).and_then(|g| g.parse().ok()).unwrap_or(6);
    match tcc_shell::window::run_corpus(std::path::Path::new(tep), giay) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("✗ {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(feature = "window"))]
fn lenh_corpus(_doi: &[String]) -> ExitCode {
    eprintln!("✗ bộ trang thật cần bản dựng có cửa sổ: --features window");
    ExitCode::FAILURE
}

#[cfg(not(feature = "window"))]
fn lenh_web(_doi: &[String], _ngon_ngu: Language) -> ExitCode {
    eprintln!("✗ mở trang web cần bản dựng có cửa sổ: --features window");
    ExitCode::FAILURE
}

/// `vi nhap <tệp>` — nhập ví trong cửa sổ.
///
/// Bản dựng KHÔNG có cờ `wallet` trả lời thẳng là không có ví, chứ không im
/// lặng bỏ qua: một lệnh không làm gì mà cũng không nói gì là lệnh người dùng
/// tưởng đã chạy.
#[cfg(feature = "wallet")]
fn lenh_vi(doi: &[String], ngon_ngu: Language) -> ExitCode {
    let lenh = doi.get(1).map(String::as_str);

    // `vi cum-tu` — gõ THẲNG 24 chữ hoặc hạt giống. Không cần tệp nào.
    if lenh == Some("cum-tu") {
        return match tcc_shell::wallet_flow::restore_from_phrase(ngon_ngu) {
            Ok(dia_chi) => {
                println!("✓ đã khôi phục ví {dia_chi}");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("✗ {e}");
                ExitCode::FAILURE
            }
        };
    }

    let (Some("nhap"), Some(tep)) = (lenh, doi.get(2)) else {
        eprintln!("cần một trong hai:");
        eprintln!("    tcc-browser vi cum-tu                  # gõ 24 chữ / hạt giống");
        eprintln!("    tcc-browser vi nhap <tệp-ví-web.json>  # nhập từ ví web, hỏi PIN");
        return ExitCode::FAILURE;
    };
    match tcc_shell::wallet_flow::import_from_file(std::path::Path::new(tep), ngon_ngu) {
        Ok(dia_chi) => {
            println!("✓ đã nhập ví {dia_chi}");
            println!("⚠ Bản cũ ở ví web VẪN CÒN, vẫn khoá bằng đúng mã PIN cũ.");
            println!("⚠ Và xoá {tep} đi — nó vẫn đang giữ khoá của bạn.");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("✗ {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(feature = "wallet"))]
fn lenh_vi(_doi: &[String], _ngon_ngu: Language) -> ExitCode {
    eprintln!("✗ bản dựng này KHÔNG có ví.");
    eprintln!("  Dựng lại với: cargo build -p tcc-browser --features wallet");
    eprintln!("  (và ví chỉ cất được khoá khi gói ứng dụng đã ký — docs/vi-thiet-ke.md §19)");
    ExitCode::FAILURE
}

/// Xem hộp thoại hỏi quyền của một gói, không mở ứng dụng.
///
/// # Errors
/// Gói không hợp lệ, hoặc bộ dựng hỏng.
fn xem_hop_thoai(duong_dan: &std::path::Path, ngon_ngu: Language) -> ExitCode {
    // Kiểm chữ ký TRƯỚC. Không có đường nào dựng hộp thoại từ gói chưa kiểm.
    let (app, _) = match tcc_runtime::verify_from_dir(duong_dan, &HybridEd25519MlDsa) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("✗ gói không hợp lệ: {e}");
            return ExitCode::FAILURE;
        }
    };
    match run_loop(app.manifest(), ngon_ngu) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("✗ {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(feature = "window")]
fn mang_that() -> Box<dyn tcc_runtime::Network> {
    Box::new(tcc_shell::HttpNetwork::new())
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
