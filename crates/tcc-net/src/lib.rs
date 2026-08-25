//! Đường ra ngoài của trình duyệt — cài đặt `tcc_runtime::Network`.
//!
//! # Vì sao là một crate RIÊNG
//!
//! Để đường ra khỏi máy **nhìn thấy được trong cây phụ thuộc**. `tcc-runtime`
//! không phụ thuộc crate này, nên đọc `Cargo.toml` là biết ngay: bộ nạp ứng
//! dụng không tự mở socket được. Có luật CI kiểm (luật 8).
//!
//! Nó cũng để việc thay máy khách HTTP là việc của một crate, không phải việc
//! của cả trình duyệt.
//!
//! # Sáu luật, mỗi luật chặn một đòn cụ thể
//!
//! | Luật | Chặn cái gì |
//! |---|---|
//! | Chỉ HTTPS | Nghe lén và sửa nội dung trên đường |
//! | **KHÔNG theo chuyển hướng** | Thoát khỏi phạm vi quyền năng |
//! | Có thời gian chờ | Máy chủ thù địch treo trình duyệt |
//! | Trần kích thước trả về | Máy chủ thù địch nuốt hết bộ nhớ |
//! | Không cookie, không phiên | Theo dõi xuyên ứng dụng |
//! | Không gửi gì thừa | Vân tay nhận dạng |
//!
//! ## ⚠️ Chuyển hướng là một đòn THOÁT KHỎI QUYỀN NĂNG
//!
//! Đây là luật quan trọng nhất tệp này. Quyền năng cho phép gọi
//! `shop.tcc-coin.com`. Máy chủ đó trả về `302 → ke-gian.example`. Máy khách nào
//! tự đi theo chuyển hướng thì ứng dụng vừa **chạm tới một máy chủ chưa bao giờ
//! được cấp quyền** — mà cổng quyền năng ở `tcc-runtime` đã đóng lại phía sau và
//! không có cách nào biết.
//!
//! Chặn bằng HAI lớp, cố ý:
//!   1. `max_redirects(0)` trong cấu hình máy khách
//!   2. Mã của ta tự từ chối mọi trạng thái 3xx
//!
//! Lớp 2 kiểm thử được mà không cần máy chủ thật, nên nó không bao giờ mục.

pub mod rpc;

use std::time::Duration;

use tcc_runtime::Network;
use thiserror::Error;

/// Trần kích thước một lần trả về.
///
/// Không có trần thì một máy chủ thù địch chỉ cần trả một luồng vô tận là ngốn
/// hết bộ nhớ — không cần lỗ hổng nào.
///
/// ⚠️ Trần này là **NHỎ HƠN NGẶT**, không phải "nhỏ hơn hoặc bằng". `LimitReader`
/// của `ureq` báo lỗi ở lần đọc **sau khi** hết hạn ngạch, nên thân đúng bằng
/// `MAX_BYTES` byte bị TỪ CHỐI (đọc `ureq-3.4.0/src/body/limit.rs:22-24`, không
/// phải nhớ). Lệch một byte, và lệch về phía an toàn, nên giữ nguyên — nhưng
/// tài liệu phải nói đúng điều mã làm.
pub const MAX_BYTES: u64 = 8 * 1024 * 1024;

/// Thời gian chờ toàn cục cho một lần gọi.
pub const MAX_WAIT: Duration = Duration::from_secs(20);

// Hai hằng trên là bức tường duy nhất chống máy chủ thù địch mà ta kiểm được
// KHÔNG cần dựng máy chủ TLS. Kiểm ở đây, lúc DỰNG, chứ không phải trong một
// `#[test]`: đặt trần về 0 hay bỏ hạn giờ thì bản dựng CHẾT, to hơn một phép
// thử đỏ mà người ta có thể chạy thiếu.
//
// `Duration` không so sánh được trong ngữ cảnh `const`, nên so qua `as_secs`.
const _: () = {
    assert!(MAX_BYTES > 0 && MAX_BYTES <= 64 * 1024 * 1024);
    assert!(MAX_WAIT.as_secs() > 0 && MAX_WAIT.as_secs() <= 60);
};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum NetError {
    #[error(
        "máy chủ trả {0} — chuyển hướng KHÔNG được đi theo, vì đích đến có thể là \
         một máy chủ chưa bao giờ được cấp quyền"
    )]
    ChuyenHuong(u16),

    #[error("máy chủ trả {0}")]
    TrangThaiXau(u16),

    #[error("đường dẫn phải bắt đầu bằng / và không chứa ký tự điều khiển")]
    DuongDanXau,

    #[error("trả về quá {0} byte")]
    TooLarge(u64),

    #[error("gọi thất bại: {0}")]
    Goi(String),

    /// Điểm cuối không phải HTTPS. Bắt ngay lúc dựng, không đợi tới lúc gọi.
    #[error("điểm cuối \"{0}\" không phải https:// — ví không nói chuyện trên đường trần")]
    KhongPhaiHttps(String),
}

/// Xét mã trạng thái. **Hàm thuần** — kiểm thử được mà không cần máy chủ.
///
/// Tách ra vì đây là lớp phòng thủ thứ hai chống chuyển hướng, và một lớp phòng
/// thủ không kiểm thử được là một lớp không biết có tồn tại hay không.
///
/// # Errors
/// 3xx (chuyển hướng) hoặc bất kỳ mã nào ngoài 2xx.
pub fn check_status(ma: u16) -> Result<(), NetError> {
    // Nhánh 3xx phải đứng TRƯỚC nhánh "ngoài 2xx": thông báo riêng cho chuyển
    // hướng là thứ giúp người viết ứng dụng hiểu vì sao bị chặn.
    if (300..400).contains(&ma) {
        return Err(NetError::ChuyenHuong(ma));
    }
    if (200..300).contains(&ma) {
        return Ok(());
    }
    Err(NetError::TrangThaiXau(ma))
}

/// Dịch lỗi đọc thân trả về. **Hàm thuần** — kiểm thử được mà không cần máy chủ.
///
/// Vì sao tách ra: hai chỗ đọc thân đều ánh xạ sai, theo hai hướng ngược nhau.
/// Chỗ này viết `map_err(|_| TooLarge)` — nuốt MỌI lỗi đọc thành "quá lớn", nên
/// đứt mạng giữa chừng cũng báo "tệp quá lớn" và người dùng đi tìm tệp nhỏ hơn
/// mãi mãi. Chỗ kia (`rpc.rs`) viết `map_err(Goi)` — mất hẳn tín hiệu "quá lớn"
/// vào một câu "gọi thất bại" chung chung.
///
/// `ureq` báo trần bằng `Error::BodyExceedsLimit`, và biến thể ấy **sống sót**
/// qua vòng `into_io()` → `From<io::Error>` (đọc `ureq-3.4.0/src/error.rs:198`
/// và `:216`), nên tới đây phân biệt được.
pub(crate) fn dich_loi_doc(e: &ureq::Error, tran: u64) -> NetError {
    match e {
        ureq::Error::BodyExceedsLimit(_) => NetError::TooLarge(tran),
        // `ureq::Error` là `#[non_exhaustive]`: nhánh bắt-tất-cả là BẮT BUỘC,
        // không phải lười.
        khac => NetError::Goi(khac.to_string()),
    }
}

/// Dựng địa chỉ. **Hàm thuần.**
///
/// Tên máy chủ đã được `tcc_spec::check_host` kiểm hình dạng ở tầng bản kê khai,
/// nên tới đây nó không thể chứa `@`, `:`, `/`. Đường dẫn thì chưa ai kiểm —
/// kiểm ở đây.
///
/// # Errors
/// Đường dẫn không bắt đầu bằng `/`, hoặc chứa ký tự điều khiển.
pub fn build_url(host: &str, path: &str) -> Result<String, NetError> {
    if !path.starts_with('/') {
        return Err(NetError::DuongDanXau);
    }
    // Ký tự điều khiển trong đường dẫn là đòn tách yêu cầu kinh điển: `\r\n`
    // chèn thêm một tiêu đề, hoặc cả một yêu cầu thứ hai.
    if path.chars().any(|c| c.is_control() || c == ' ') {
        return Err(NetError::DuongDanXau);
    }
    Ok(format!("https://{host}{path}"))
}

/// Máy khách HTTP của trình duyệt.
#[derive(Debug)]
pub struct HttpNetwork {
    agent: ureq::Agent,
}

impl Default for HttpNetwork {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpNetwork {
    #[must_use]
    pub fn new() -> Self {
        let cau_hinh = ureq::Agent::config_builder()
            // LỚP 1 chống chuyển hướng. Lớp 2 là `check_status`.
            .max_redirects(0)
            .timeout_global(Some(MAX_WAIT))
            .build();
        Self {
            agent: cau_hinh.into(),
        }
    }
}

impl Network for HttpNetwork {
    fn get(&self, host: &str, path: &str) -> Result<Vec<u8>, String> {
        let url_for = build_url(host, path).map_err(|e| e.to_string())?;

        let mut dap = self
            .agent
            .get(&url_for)
            .call()
            .map_err(|e| NetError::Goi(e.to_string()).to_string())?;

        check_status(dap.status().as_u16()).map_err(|e| e.to_string())?;

        // Đọc CÓ TRẦN, không đọc hết rồi mới đo: đọc hết một luồng vô tận thì
        // không bao giờ tới được chỗ đo.
        dap.body_mut()
            .with_config()
            .limit(MAX_BYTES)
            .read_to_vec()
            .map_err(|e| dich_loi_doc(&e, MAX_BYTES).to_string())
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay, không nuốt lỗi"
)]
mod kiem_thu {
    use super::*;

    /// ⚠️ PHÉP THỬ QUAN TRỌNG NHẤT TỆP NÀY.
    ///
    /// Chuyển hướng là đòn thoát khỏi quyền năng: gọi `shop.tcc-coin.com` được
    /// cấp phép, máy chủ trả `302 → ke-gian.example`, và ứng dụng vừa chạm tới
    /// một máy chủ chưa bao giờ được cấp quyền.
    #[test]
    fn moi_chuyen_huong_deu_bi_tu_choi() {
        for ma in [300, 301, 302, 303, 307, 308, 399] {
            assert_eq!(
                check_status(ma),
                Err(NetError::ChuyenHuong(ma)),
                "mã {ma} không bị chặn"
            );
        }
    }

    #[test]
    fn chi_2xx_moi_dat() {
        for ma in [200, 201, 204, 299] {
            assert!(check_status(ma).is_ok(), "mã {ma} bị chặn oan");
        }
        for ma in [100, 199, 400, 403, 404, 500, 503] {
            assert_eq!(check_status(ma), Err(NetError::TrangThaiXau(ma)));
        }
    }

    #[test]
    fn dia_chi_luon_la_https() {
        let d = build_url("shop.tcc-coin.com", "/san-pham").unwrap();
        assert_eq!(d, "https://shop.tcc-coin.com/san-pham");
        assert!(d.starts_with("https://"));
    }

    /// Đòn tách yêu cầu: `\r\n` trong đường dẫn chèn thêm tiêu đề, hoặc cả một
    /// yêu cầu thứ hai tới một máy chủ khác.
    #[test]
    fn duong_dan_co_ky_tu_dieu_khien_bi_chan() {
        for p in [
            "/a\r\nHost: evil.example",
            "/a\nX: 1",
            "/a\0b",
            "/a b",
            "khong-co-gach-cheo",
            "",
        ] {
            assert_eq!(
                build_url("shop.tcc-coin.com", p),
                Err(NetError::DuongDanXau),
                "đường dẫn {p:?} lọt qua"
            );
        }
    }

    #[test]
    fn duong_dan_thuong_van_qua() {
        for p in ["/", "/a/b/c", "/tim?q=ao+dai", "/a%20b", "/#neo"] {
            assert!(
                build_url("shop.tcc-coin.com", p).is_ok(),
                "đường dẫn {p:?} bị chặn oan"
            );
        }
    }

    /// **Trần kích thước phải nói ĐÚNG nó là trần, không nuốt lỗi khác.**
    ///
    /// Bản đầu viết `map_err(|_| TooLarge)`: đứt mạng giữa chừng, máy chủ ngắt,
    /// hết giờ — đều bị báo "tệp quá lớn". Người dùng đọc câu ấy sẽ đi tìm một
    /// tệp nhỏ hơn, mãi mãi, cho một lỗi chẳng liên quan gì tới kích thước.
    #[test]
    fn chi_loi_vuot_tran_moi_bao_la_qua_lon() {
        assert_eq!(
            dich_loi_doc(&ureq::Error::BodyExceedsLimit(MAX_BYTES), MAX_BYTES),
            NetError::TooLarge(MAX_BYTES)
        );

        // Mọi lỗi KHÁC phải giữ nguyên nguyên nhân của nó.
        for khac in [
            ureq::Error::Io(std::io::Error::new(
                std::io::ErrorKind::ConnectionReset,
                "đứt giữa chừng",
            )),
            ureq::Error::HostNotFound,
            ureq::Error::ConnectionFailed,
        ] {
            let ra = dich_loi_doc(&khac, MAX_BYTES);
            assert!(
                !matches!(ra, NetError::TooLarge(_)),
                "{khac:?} bị báo nhầm là quá lớn"
            );
            assert!(
                ra.to_string().contains(&khac.to_string()),
                "mất nguyên nhân thật: {khac:?} → {ra}"
            );
        }
    }

    /// **Trần đi kèm CON SỐ của chính kênh đó.**
    ///
    /// Hai kênh có hai trần khác nhau (8 MiB cho gói, 1 MiB cho RPC). Câu lỗi
    /// dán cứng `MAX_BYTES` thì kênh RPC nói dối người đọc về giới hạn thật.
    #[test]
    fn cau_bao_qua_lon_mang_tran_cua_kenh_ay() {
        let rpc = dich_loi_doc(&ureq::Error::BodyExceedsLimit(9), 1024 * 1024);
        assert!(
            rpc.to_string().contains("1048576"),
            "không nói trần thật của kênh: {rpc}"
        );
        assert!(
            !rpc.to_string().contains(&MAX_BYTES.to_string()),
            "dán cứng trần của kênh kia: {rpc}"
        );
    }
}
