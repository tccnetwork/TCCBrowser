//! Đường chạy mặc định của binary sản phẩm.
//!
//! # Vì sao tệp này tồn tại
//!
//! Người soát độc lập đo ngày 16/08/2026 rằng `apps/tcc-browser` là **đơn vị
//! duy nhất vừa nằm trên đường chạy sản phẩm vừa có 0 phép thử** — và đó cũng
//! đúng là tệp chứa phát hiện F1 của họ. Họ viết thêm: *"không phải trùng
//! hợp"*. Đúng.
//!
//! Nên phép thử ở đây không kiểm chữ nghĩa của thông báo. Nó kiểm đúng hai
//! điều mà F1 vi phạm: chạy không tham số **không** dựng ra được giao diện từ
//! dữ liệu tự bịa, và đường có gói thì **kiểm chữ ký trước** khi hiện bất cứ
//! thứ gì.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay"
)]

use std::process::Command;

const BINARY: &str = env!("CARGO_BIN_EXE_tcc-browser");

/// **Không tham số thì KHÔNG dựng giao diện nào.**
///
/// Trước 16/08/2026 nhánh này dựng hộp thoại hỏi quyền từ một bản kê khai nhúng
/// cứng, có `publisher: ""` và một quyền `wallet`. Phép thử này đỏ ngày ai đó
/// thêm lại một "dữ liệu mẫu cho tiện xem".
#[test]
fn khong_tham_so_thi_khong_dung_giao_dien_nao() {
    let ra = Command::new(BINARY).output().expect("chạy được binary");
    assert!(!ra.status.success(), "phải thoát với mã lỗi");

    let s = String::from_utf8_lossy(&ra.stdout).to_string() + &String::from_utf8_lossy(&ra.stderr);
    assert!(
        !s.contains("permission") && !s.contains("hỏi quyền"),
        "đang dựng hộp thoại hỏi quyền mà chưa có gói nào:\n{s}"
    );
    // Và phải chỉ cho người ta đường đi tiếp, không chỉ báo lỗi cụt.
    assert!(s.contains("examples/hello-tcc"), "{s}");
}

/// Gói THẬT thì kiểm chữ ký trước, rồi mới hiện.
#[test]
fn goi_that_thi_kiem_chu_ky_truoc() {
    let goi = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/hello-tcc");
    let ra = Command::new(BINARY).arg(goi).output().expect("chạy được");
    let s = String::from_utf8_lossy(&ra.stdout).to_string();
    assert!(ra.status.success(), "gói ví dụ phải nạp được:\n{s}");
    assert!(s.contains("chữ ký hợp lệ"), "không kiểm chữ ký:\n{s}");
}

/// Gói bị sửa một byte thì **từ chối**, và không hiện gì của nó.
#[test]
fn goi_bi_sua_thi_tu_choi() {
    let goc = std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/hello-tcc"
    ));
    let tam = std::env::temp_dir().join("tcc-goi-bi-sua");
    let _ = std::fs::remove_dir_all(&tam);
    chep(goc, &tam);

    // Lật một byte trong bản kê khai. Chữ ký ký lên byte thô nên đây là đủ.
    let kk = tam.join("manifest.json");
    let mut b = std::fs::read(&kk).expect("đọc bản kê khai");
    let i = b.len() / 2;
    b[i] ^= 0x20;
    std::fs::write(&kk, &b).expect("ghi lại");

    let ra = Command::new(BINARY).arg(&tam).output().expect("chạy được");
    let s = String::from_utf8_lossy(&ra.stdout).to_string();
    assert!(!ra.status.success(), "gói bị sửa mà vẫn nạp:\n{s}");
    assert!(
        !s.contains("chữ ký hợp lệ"),
        "báo chữ ký hợp lệ cho gói đã bị sửa:\n{s}"
    );
    let _ = std::fs::remove_dir_all(&tam);
}

fn chep(tu: &std::path::Path, den: &std::path::Path) {
    std::fs::create_dir_all(den).expect("tạo thư mục");
    for m in std::fs::read_dir(tu).expect("đọc thư mục") {
        let m = m.expect("mục");
        let dich = den.join(m.file_name());
        if m.path().is_dir() {
            chep(&m.path(), &dich);
        } else {
            std::fs::copy(m.path(), dich).expect("chép tệp");
        }
    }
}
