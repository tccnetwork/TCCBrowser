//! Phục vụ tệp TỪ TRONG GÓI ĐÃ KÝ cho WebView, qua giao thức `tcc-goi:`.
//!
//! # Vì sao cần
//!
//! Ảnh trong `ui.json` khai đường dẫn tương đối (`anh/logo.png`). Tài liệu nạp
//! bằng `with_html` **không có địa chỉ gốc**, nên đường dẫn đó không phân giải
//! được — mọi ảnh của mọi ứng dụng TCC đều hỏng. Cần một giao thức riêng.
//!
//! # ⚠️ ĐÂY LÀ MỘT ĐƯỜNG MỚI VÀO NỘI DUNG GÓI
//!
//! Trước tệp này, thứ duy nhất ra khỏi gói là chuỗi đánh dấu ta tự dựng. Giờ có
//! một trình phục vụ nhận **địa chỉ do trang yêu cầu** và trả về byte. Ba luật:
//!
//! | Luật | Chặn cái gì |
//! |---|---|
//! | Đường dẫn qua đúng `check_path` của tiêu chuẩn | `../` đi ra ngoài gói |
//! | Chỉ trả tệp CÓ TRONG cây đã ký | nội dung không nằm trong phạm vi chữ ký |
//! | Kiểu nội dung lấy từ DANH SÁCH TRẮNG theo đuôi tệp | ép trình duyệt coi một tệp là HTML |
//!
//! Luật thứ ba dễ bị coi nhẹ. Chính sách nội dung chỉ mở `img-src tcc-goi:`,
//! nhưng nếu trình phục vụ trả `text/html` thì một ngày ai đó nới chính sách ra
//! là có ngay một trang cùng nguồn gốc do ứng dụng điều khiển. Chặn ở đây, ngay
//! bây giờ, rẻ hơn nhiều so với nhớ ra sau.

use tcc_ui::tcc_spec_tree::check_path_public;

/// Tên giao thức. Khớp với `img-src tcc-goi:` trong chính sách nội dung.
pub const GIAO_THUC: &str = "tcc-goi";

/// Máy chủ giả trong địa chỉ. Không có ý nghĩa gì ngoài việc làm địa chỉ hợp lệ.
pub const MAY_CHU: &str = "goi";

/// Vì sao một yêu cầu bị từ chối.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TuChoi {
    /// Địa chỉ không thuộc giao thức này.
    KhongPhaiGiaoThucNay,
    /// Đường dẫn vi phạm ràng buộc của tiêu chuẩn (`..`, tuyệt đối, …).
    DuongDanXau(String),
    /// Đuôi tệp không nằm trong danh sách trắng.
    DuoiTepKhongCho(String),
    /// Không có tệp đó trong gói đã ký.
    KhongCoTrongGoi(String),
}

impl std::fmt::Display for TuChoi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::KhongPhaiGiaoThucNay => write!(f, "không phải giao thức {GIAO_THUC}"),
            Self::DuongDanXau(p) => write!(f, "đường dẫn \"{p}\" không hợp lệ"),
            Self::DuoiTepKhongCho(p) => write!(
                f,
                "\"{p}\" có đuôi không nằm trong danh sách trắng — chỉ ảnh mới được phục vụ"
            ),
            Self::KhongCoTrongGoi(p) => write!(f, "\"{p}\" không có trong gói đã ký"),
        }
    }
}

/// Danh sách trắng đuôi tệp → kiểu nội dung.
///
/// **Danh sách TRẮNG, không phải danh sách đen.** Đuôi lạ thì từ chối, không
/// đoán. Và cố ý chỉ có ảnh: chính sách nội dung chỉ mở `img-src`, nên phục vụ
/// thứ khác là chuẩn bị sẵn một lỗ cho ngày ai đó nới chính sách ra.
///
/// KHÔNG có `svg`: ảnh SVG chạy được kịch bản và nhúng được tài nguyên ngoài.
/// Nó là một tài liệu, không phải một tấm ảnh.
const KIEU: &[(&str, &str)] = &[
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("gif", "image/gif"),
    ("webp", "image/webp"),
    ("avif", "image/avif"),
];

/// Dựng địa chỉ `tcc-goi:` cho một đường dẫn trong gói.
#[must_use]
pub fn dia_chi(duong_dan: &str) -> String {
    format!("{GIAO_THUC}://{MAY_CHU}/{duong_dan}")
}

/// Tách đường dẫn trong gói ra khỏi địa chỉ, và KIỂM nó.
///
/// # Errors
/// Không phải giao thức này, hoặc đường dẫn vi phạm ràng buộc của tiêu chuẩn.
pub fn duong_dan_tu_dia_chi(dia_chi: &str) -> Result<String, TuChoi> {
    let sau = dia_chi
        .strip_prefix(&format!("{GIAO_THUC}://"))
        .ok_or(TuChoi::KhongPhaiGiaoThucNay)?;
    // Bỏ phần máy chủ. Không kiểm nó bằng gì — địa chỉ nào cũng chỉ tới đúng
    // một gói, cái quyết định là phần đường dẫn.
    let p = sau.split_once('/').map_or("", |(_, p)| p);

    // Cắt phần truy vấn và phần neo TRƯỚC khi kiểm: `anh/logo.png?../../x` mà
    // không cắt thì `check_path` nhìn thấy một chuỗi khác với chuỗi ta tra cứu.
    let p = p.split(['?', '#']).next().unwrap_or("");

    // Giải mã phần trăm: `%2e%2e%2f` chính là `../` viết trá hình. Không giải mã
    // thì `check_path` không nhìn thấy nó, mà trình duyệt thì có.
    let p = giai_ma_phan_tram(p);

    check_path_public(&p).map_err(|_| TuChoi::DuongDanXau(p.clone()))?;
    Ok(p)
}

/// Giải mã `%XX` trong đường dẫn.
fn giai_ma_phan_tram(s: &str) -> String {
    let b = s.as_bytes();
    let mut ra = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            let h = |c: u8| (c as char).to_digit(16);
            if let (Some(a), Some(c)) = (h(b[i + 1]), h(b[i + 2])) {
                ra.push(u8::try_from(a * 16 + c).unwrap_or(b'?'));
                i += 3;
                continue;
            }
        }
        ra.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&ra).into_owned()
}

/// Kiểu nội dung theo đuôi tệp.
///
/// # Errors
/// Đuôi không nằm trong danh sách trắng.
pub fn kieu_noi_dung(duong_dan: &str) -> Result<&'static str, TuChoi> {
    let duoi = duong_dan
        .rsplit_once('.')
        .map(|(_, d)| d.to_ascii_lowercase())
        .unwrap_or_default();
    KIEU.iter()
        .find(|(d, _)| *d == duoi)
        .map(|(_, k)| *k)
        .ok_or_else(|| TuChoi::DuoiTepKhongCho(duong_dan.to_owned()))
}

/// Phục vụ một yêu cầu. `doc_tep` đọc từ cây tệp ĐÃ KÝ.
///
/// # Errors
/// Bất kỳ luật nào trong ba luật ở đầu tệp bị vi phạm.
pub fn phuc_vu(
    dia_chi: &str,
    doc_tep: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> Result<(&'static str, Vec<u8>), TuChoi> {
    let p = duong_dan_tu_dia_chi(dia_chi)?;
    // Kiểu nội dung TRƯỚC khi đọc: đuôi lạ thì không cần chạm tới tệp.
    let kieu = kieu_noi_dung(&p)?;
    let byte = doc_tep(&p).ok_or_else(|| TuChoi::KhongCoTrongGoi(p.clone()))?;
    Ok((kieu, byte))
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay, không nuốt lỗi"
)]
mod kiem_thu {
    use super::*;

    fn goi_mau(p: &str) -> Option<Vec<u8>> {
        match p {
            "anh/logo.png" => Some(b"PNG".to_vec()),
            "bi-mat.txt" => Some(b"bi mat".to_vec()),
            _ => None,
        }
    }

    #[test]
    fn phuc_vu_duoc_anh_trong_goi() {
        let (k, b) = phuc_vu(&dia_chi("anh/logo.png"), &goi_mau).unwrap();
        assert_eq!(k, "image/png");
        assert_eq!(b, b"PNG");
    }

    /// ⚠️ PHÉP THỬ QUAN TRỌNG NHẤT TỆP NÀY: đi ra ngoài gói.
    #[test]
    fn duong_dan_di_ra_ngoai_goi_bi_chan() {
        for p in [
            "../bi-mat.png",
            "../../etc/passwd",
            "/etc/passwd",
            "anh/../../x.png",
            "anh//logo.png",
        ] {
            assert!(
                matches!(phuc_vu(&dia_chi(p), &goi_mau), Err(TuChoi::DuongDanXau(_))),
                "\"{p}\" lọt qua"
            );
        }
    }

    /// ⚠️ `../` viết trá hình bằng mã phần trăm.
    ///
    /// Không giải mã thì `check_path` không nhìn thấy nó — mà trình duyệt thì
    /// có, và nó sẽ yêu cầu đúng tệp ở ngoài gói.
    #[test]
    fn duong_dan_ma_hoa_phan_tram_van_bi_chan() {
        for p in [
            "%2e%2e%2fbi-mat.png",
            "%2E%2E/bi-mat.png",
            "anh%2f%2e%2e%2f%2e%2e%2fx.png",
        ] {
            let d = format!("{GIAO_THUC}://{MAY_CHU}/{p}");
            assert!(
                matches!(phuc_vu(&d, &goi_mau), Err(TuChoi::DuongDanXau(_))),
                "\"{p}\" lọt qua"
            );
        }
    }

    /// Phần truy vấn phải bị cắt TRƯỚC khi kiểm — nếu không, chuỗi đem kiểm
    /// khác chuỗi đem tra cứu.
    #[test]
    fn phan_truy_van_va_neo_bi_cat() {
        for d in [
            dia_chi("anh/logo.png?v=2"),
            dia_chi("anh/logo.png#neo"),
            dia_chi("anh/logo.png?x=../../y"),
        ] {
            let (k, b) = phuc_vu(&d, &goi_mau).unwrap_or_else(|e| panic!("{d} → {e}"));
            assert_eq!(k, "image/png");
            assert_eq!(b, b"PNG");
        }
    }

    /// ⚠️ Danh sách TRẮNG: đuôi lạ thì từ chối, không đoán.
    #[test]
    fn chi_phuc_vu_anh() {
        // Tệp CÓ trong gói, nhưng đuôi không nằm trong danh sách trắng.
        assert!(matches!(
            phuc_vu(&dia_chi("bi-mat.txt"), &goi_mau),
            Err(TuChoi::DuoiTepKhongCho(_))
        ));
        for p in ["a.html", "a.js", "a.json", "a.wasm", "a", "a."] {
            assert!(
                matches!(
                    phuc_vu(&dia_chi(p), &goi_mau),
                    Err(TuChoi::DuoiTepKhongCho(_))
                ),
                "\"{p}\" lọt qua"
            );
        }
    }

    /// ⚠️ SVG là một TÀI LIỆU, không phải một tấm ảnh: nó chạy được kịch bản và
    /// nhúng được tài nguyên ngoài. Cố ý không có trong danh sách trắng.
    #[test]
    fn svg_khong_duoc_phuc_vu() {
        assert!(matches!(
            phuc_vu(&dia_chi("anh/logo.svg"), &goi_mau),
            Err(TuChoi::DuoiTepKhongCho(_))
        ));
    }

    #[test]
    fn tep_khong_co_trong_goi_thi_tu_choi() {
        assert!(matches!(
            phuc_vu(&dia_chi("anh/khong-co.png"), &goi_mau),
            Err(TuChoi::KhongCoTrongGoi(_))
        ));
    }

    #[test]
    fn giao_thuc_khac_thi_tu_choi() {
        for d in [
            "https://ke-gian.example/x.png",
            "file:///etc/passwd",
            "tcc-goi-gia://goi/x.png",
        ] {
            assert!(matches!(
                phuc_vu(d, &goi_mau),
                Err(TuChoi::KhongPhaiGiaoThucNay)
            ));
        }
    }
}
