//! Kiểm thử đầu-cuối: chạy THẬT nhị phân `tcc` trên gói THẬT trên đĩa.
//!
//! Kiểm thử đơn vị trong thư viện đã kiểm từng mảnh. Tệp này kiểm cái khác: khi
//! ghép lại thành một công cụ và chạy trên hệ thống tệp thật thì các đòn tấn công
//! có bị chặn không.
//!
//! Mỗi phép thử dựng một đòn cụ thể. Thất bại ở đây nghĩa là một gói độc hại đi
//! lọt được — nên đọc kỹ trước khi sửa cái nào.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay, không nuốt lỗi"
)]

use std::{fs, path::Path, process::Command};

const TCC: &str = env!("CARGO_BIN_EXE_tcc-cli");

/// Điểm vào của một gói mới. **Không phải HTML** — ứng dụng TCC khai báo cây
/// component, xem `tcc_ui::dang_goi`.
const TEP_GIAO_DIEN: &str = "content/ui.json";

struct SanChoi {
    thu_muc: std::path::PathBuf,
}

impl SanChoi {
    /// Dựng một gói đã ký, sẵn sàng để đem ra phá.
    fn moi(ten: &str) -> Self {
        let thu_muc = std::env::temp_dir().join(format!("tcc-thu-{ten}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&thu_muc);
        fs::create_dir_all(&thu_muc).expect("tạo thư mục tạm");
        let s = Self { thu_muc };

        assert!(s.chay(&["new", "goi", "--id", "com.tcc.thu"]).0, "tcc new");
        assert!(s.chay(&["key", "--ra", "khoa.hex"]).0, "tcc key");
        assert!(s.chay(&["sign", "goi", "--khoa", "khoa.hex"]).0, "tcc sign");
        s
    }

    /// Chạy `tcc` với thư mục làm việc là sân chơi. Trả (thành công, đầu ra).
    fn chay(&self, args: &[&str]) -> (bool, String) {
        let ra = Command::new(TCC)
            .args(args)
            .current_dir(&self.thu_muc)
            .output()
            .expect("chạy được tcc");
        let mut s = String::from_utf8_lossy(&ra.stdout).to_string();
        s.push_str(&String::from_utf8_lossy(&ra.stderr));
        (ra.status.success(), s)
    }

    fn goi(&self) -> std::path::PathBuf {
        self.thu_muc.join("goi")
    }
}

impl Drop for SanChoi {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.thu_muc);
    }
}

#[test]
fn goi_vua_ky_thi_kiem_dat() {
    let s = SanChoi::moi("lanh");
    let (ok, ra) = s.chay(&["verify", "goi"]);
    assert!(ok, "gói vừa ký phải kiểm đạt:\n{ra}");
    assert!(ra.contains("Chữ ký hợp lệ"), "{ra}");
}

/// `verify` phải cảnh báo rằng chữ ký KHÔNG chứng minh danh tính.
///
/// Đây không phải chuyện câu chữ: người kiểm gói mà tưởng "hợp lệ = đúng nhà phát
/// hành" thì sẽ cài nhầm gói giả mạo. Cảnh báo này là tầng phòng thủ duy nhất cho
/// tới khi có sổ đăng ký khoá.
#[test]
fn verify_phai_canh_bao_khong_chung_minh_danh_tinh() {
    let s = SanChoi::moi("canhbao");
    let (_, ra) = s.chay(&["verify", "goi"]);
    assert!(
        ra.contains("KHÔNG chứng minh người ký là ai"),
        "thiếu cảnh báo về danh tính:\n{ra}"
    );
}

/// ⚠️ ĐÒN 1: thay ruột, giữ nguyên bản kê khai và chữ ký.
#[test]
fn thay_noi_dung_thi_kiem_hong() {
    let s = SanChoi::moi("thayruot");
    fs::write(s.goi().join(TEP_GIAO_DIEN), "mã độc hại").unwrap();

    let (ok, ra) = s.chay(&["verify", "goi"]);
    assert!(!ok, "đổi nội dung mà vẫn qua:\n{ra}");
    assert!(ra.contains("không khớp bản kê khai"), "{ra}");
}

/// ⚠️ ĐÒN 2: thêm một tệp lạ vào gói đã ký.
///
/// Băm chỉ tính trên các tệp đã biết thì đòn này lọt — tệp mới không nằm trong
/// phép tính nên băm không đổi.
#[test]
fn them_tep_la_thi_kiem_hong() {
    let s = SanChoi::moi("themtep");
    fs::write(s.goi().join("content/cua-sau.js"), "// cửa sau").unwrap();

    let (ok, ra) = s.chay(&["verify", "goi"]);
    assert!(!ok, "thêm tệp mà vẫn qua:\n{ra}");
}

/// ⚠️ ĐÒN 3: sửa bản kê khai để xin thêm quyền ví.
#[test]
fn sua_ban_ke_khai_xin_them_quyen_thi_kiem_hong() {
    let s = SanChoi::moi("themquyen");
    let p = s.goi().join("manifest.json");
    let cu = fs::read_to_string(&p).unwrap();
    let moi = cu.replace(
        r#""capabilities": []"#,
        r#""capabilities": [{"name":"wallet","scope":{"kind":"wallet","may_request_signature":true},"reason":"x"}]"#,
    );
    assert_ne!(cu, moi, "phép thử tự hỏng: không tìm thấy chỗ để sửa");
    fs::write(&p, moi).unwrap();

    let (ok, ra) = s.chay(&["verify", "goi"]);
    assert!(!ok, "xin thêm quyền mà vẫn qua:\n{ra}");
    assert!(ra.contains("chữ ký"), "{ra}");
}

/// ⚠️ ĐÒN 4: liên kết mềm trỏ ra ngoài gói.
///
/// Cái được ký là liên kết, cái được đọc lúc chạy là tệp đích — hai thứ khác nhau.
#[cfg(unix)]
#[test]
fn lien_ket_mem_thi_kiem_hong() {
    let s = SanChoi::moi("lienket");
    std::os::unix::fs::symlink("/etc/passwd", s.goi().join("content/ra-ngoai")).unwrap();

    let (ok, ra) = s.chay(&["verify", "goi"]);
    assert!(!ok, "liên kết mềm mà vẫn qua:\n{ra}");
    assert!(ra.contains("liên kết mềm"), "{ra}");
}

/// Ghi đè tệp khoá là mất vĩnh viễn quyền cập nhật mọi ứng dụng đã ký bằng nó.
#[test]
fn khong_ghi_de_tep_khoa() {
    let s = SanChoi::moi("khoa");
    let (ok, ra) = s.chay(&["key", "--ra", "khoa.hex"]);
    assert!(!ok, "ghi đè khoá mà không chặn:\n{ra}");
    assert!(ra.contains("đã tồn tại"), "{ra}");
}

/// Mã ứng dụng sai phải bị chặn TRƯỚC khi tạo thư mục — tạo rồi mới báo lỗi thì
/// để lại rác cho người dùng dọn.
#[test]
fn ma_ung_dung_sai_thi_khong_tao_thu_muc() {
    let s = SanChoi::moi("masai");
    let (ok, _) = s.chay(&["new", "hong", "--id", "KHONG-HOP-LE"]);
    assert!(!ok);
    assert!(
        !Path::new(&s.thu_muc.join("hong")).exists(),
        "đã tạo thư mục rồi mới báo lỗi — để lại rác"
    );
}

/// Điểm vào biến mất thì `verify` phải nói rõ, không để runtime nạp xong mới chết.
#[test]
fn mat_diem_vao_thi_verify_bao_ro() {
    let s = SanChoi::moi("mmatdiemvao");
    fs::remove_file(s.goi().join(TEP_GIAO_DIEN)).unwrap();
    fs::write(s.goi().join("content/khac.json"), "{}").unwrap();
    assert!(s.chay(&["sign", "goi", "--khoa", "khoa.hex"]).0);

    let (ok, ra) = s.chay(&["verify", "goi"]);
    assert!(!ok, "thiếu điểm vào mà vẫn qua:\n{ra}");
    assert!(ra.contains("điểm vào"), "{ra}");
}

/// ⚠️ Chữ ký hợp lệ mà CÂY GIAO DIỆN hỏng thì gói vẫn không chạy được.
///
/// `verify` phải bắt ở đây, lúc người viết ứng dụng còn ngồi trước máy — không
/// phải lúc người dùng cuối mở gói ra và thấy một cửa sổ trống.
#[test]
fn cay_giao_dien_hong_thi_verify_bao_ro() {
    let s = SanChoi::moi("giaodienhong");
    // Ảnh trỏ ra mạng: JSON hợp lệ, chữ ký sẽ hợp lệ, nhưng cây thì không.
    fs::write(
        s.goi().join(TEP_GIAO_DIEN),
        r#"{"kind":"image","source":"https://theo-doi.example/x.png",
            "alt":{"kind":"decorative"}}"#,
    )
    .unwrap();
    assert!(s.chay(&["sign", "goi", "--khoa", "khoa.hex"]).0);

    let (ok, ra) = s.chay(&["verify", "goi"]);
    assert!(!ok, "cây giao diện hỏng mà vẫn qua:\n{ra}");
    assert!(ra.contains("trỏ ra ngoài gói"), "{ra}");
}

/// Ký gói chưa có thư mục `content/` phải báo rõ thiếu cái gì.
#[test]
fn thieu_thu_muc_noi_dung_thi_bao_ro() {
    let s = SanChoi::moi("thieu");
    fs::remove_dir_all(s.goi().join("content")).unwrap();

    let (ok, ra) = s.chay(&["verify", "goi"]);
    assert!(!ok);
    assert!(
        ra.contains("content"),
        "lỗi phải nêu tên thứ bị thiếu:\n{ra}"
    );
}
