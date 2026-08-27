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
use tcc_shell::text::{TextKey, label};

/// Ghép một tham số vào chuỗi song ngữ.
///
/// `label` trả `&'static str` nên không dùng `format!` được — chỗ ghép viết là
/// `{}` trong chính chuỗi dịch, và thay ở đây. Cách này giữ được luật "mọi chữ
/// người dùng thấy đi qua `text.rs`" mà không phải dựng một bộ định dạng riêng.
fn chu(k: TextKey, n: Language, tham_so: &str) -> String {
    label(k, n).replace("{}", tham_so)
}
use tcc_spec::Manifest;

fn main() -> ExitCode {
    let doi = std::env::args().skip(1).collect::<Vec<_>>();
    // ⚠️ BỎ QUA đối số ĐẦU khi dò cờ ngôn ngữ.
    //
    // Cờ ngôn ngữ là chữ `vi` đặt ở cuối lệnh — mà `vi` CŨNG là tên lệnh con
    // của ví (`tcc-browser vi nhap <tệp>`). Bản trước dò bằng
    // `doi.iter().any(|a| a == "vi")`, nên MỌI lệnh ví luôn chạy tiếng Việt và
    // KHÔNG có cách nào lấy tiếng Anh — kể cả những màn hình đã song ngữ từ
    // đầu. Phát hiện 27/08/2026, ngay khi đưa lớp vỏ dòng lệnh vào song ngữ và
    // thấy lượt mặc định cũng ra tiếng Việt.
    //
    // Mặc định là tiếng ANH: trình duyệt phát cả ra ngoài công ty.
    let ngon_ngu = if doi.iter().skip(1).any(|a| a == "vi") {
        Language::Vi
    } else {
        Language::En
    };

    // `web <https://…>` — TẦNG 2: mở một trang web thật.

    // `corpus <tệp>` — chạy bộ trang thật, đếm chắn. Xem `corpus/50-trang.txt`.

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
    if let Some(duong_dan) = doi
        .iter()
        .find(|a| !a.starts_with('-') && *a != "vi" && *a != "quyen" && *a != "hop-thoai")
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
    match tcc_shell::window_raster::manage_permissions_raster(
        &goi.join(".tcc-quyen.json"),
        ngon_ngu,
    ) {
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
    // ⚠️ MỘT lời gọi, không phải hai.
    //
    // Trước 24/08/2026 chỗ này gọi `open_package_raster` (hỏi quyền) rồi
    // `run_app_raster` (hiện màn ứng dụng). Mỗi lời gọi vào vòng lặp sự kiện một
    // lần, và trên macOS `run_return` KHÔNG vào lại được sau khi đã thoát — nên
    // đường chính của sản phẩm abort, rồi (sau bản vá đầu) im lặng loé màn hai
    // rồi tắt.
    //
    // Hỏi quyền và chạy ứng dụng là hai MÀN HÌNH, không phải hai phiên.
    let mang = mang_that();
    if let Err(e) = tcc_shell::window_raster::open_and_run_raster(
        duong_dan,
        ngon_ngu,
        kho.as_deref(),
        mang.as_ref(),
    ) {
        eprintln!("✗ {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
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
                println!("{}", chu(TextKey::ViKhoiPhucXong, ngon_ngu, &dia_chi));
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("✗ {e}");
                ExitCode::FAILURE
            }
        };
    }

    let (Some("nhap"), Some(tep)) = (lenh, doi.get(2)) else {
        eprintln!("{}", label(TextKey::ViCanMotTrongHai, ngon_ngu));
        eprintln!("{}", label(TextKey::ViCachDungCumTu, ngon_ngu));
        eprintln!("{}", label(TextKey::ViCachDungNhap, ngon_ngu));
        return ExitCode::FAILURE;
    };
    match tcc_shell::wallet_flow::import_from_file(std::path::Path::new(tep), ngon_ngu) {
        Ok(dia_chi) => {
            println!("{}", chu(TextKey::ViNhapXong, ngon_ngu, &dia_chi));
            println!("{}", label(TextKey::ViBanCuVanCon, ngon_ngu));
            println!("{}", chu(TextKey::ViXoaTepDi, ngon_ngu, tep));
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

    // ⚠️ Câu ở đây từng nói `TCC_TU_DONG_DONG` "chưa có tác dụng trên bộ dựng
    // ra pixel". Nó ĐÃ có tác dụng từ 24/08/2026, và câu ấy vẫn in ra — một
    // chương trình nói sai về chính nó, đúng thứ khó phát hiện nhất vì nó nghe
    // như một lời cảnh báo cẩn thận.
    if let Some(h) = tu_dong_dong {
        eprintln!(
            "[khung] kiểm khói: mỗi màn hình tự đóng sau {}s",
            h.as_secs()
        );
    }
    tcc_shell::window_raster::open_permission_dialog_raster(m, ngon_ngu).map(|_| ())
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
