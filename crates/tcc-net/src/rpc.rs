//! JSON-RPC cho **khung trình duyệt**, không cho ứng dụng.
//!
//! # Vì sao không thêm `post` vào `Network`
//!
//! `tcc_runtime::Network` chỉ có `get`, và giữ nguyên như thế là có chủ ý. Một
//! quyền mạng chỉ-`get` yếu hơn hẳn quyền cho phép đẩy dữ liệu ra ngoài: ứng
//! dụng lấy được thứ nó xin, nhưng không dựng được một kênh gửi tuỳ ý.
//!
//! Ví thì cần `POST`. Nên `POST` nằm ở đây, **ngoài trait mà ứng dụng thấy** —
//! khung trình duyệt gọi được, ứng dụng thì không có đường nào chạm tới.
//! Thêm `post` vào `Network` cho tiện là âm thầm nới rộng nghĩa của mọi quyền
//! mạng đã cấp từ trước.
//!
//! # Sáu luật của `tcc-net` vẫn áp ở đây
//!
//! Chỉ HTTPS, không theo chuyển hướng, có thời gian chờ, có trần kích thước,
//! không cookie, không gửi gì thừa.
//!
//! # Và một luật thứ bảy, riêng cho ví
//!
//! **Phản hồi ở đây là dữ liệu KHÔNG TIN ĐƯỢC.** Máy chủ RPC là bên ta đang đề
//! phòng, không phải bên ta đang dựa vào. Mọi thứ nó trả về phải đi qua
//! `signing_flow::review` trước khi có nghĩa gì — xem `docs/vi-thiet-ke.md` §15.

use std::time::Duration;

use crate::NetError;

/// Trần kích thước phản hồi. Một giao dịch chưa ký là vài trăm byte; 1 MiB đã
/// là rộng rãi gấp nghìn lần, và nó chặn máy chủ thù địch nuốt hết bộ nhớ.
const MAX_RESPONSE: u64 = 1024 * 1024;
// Chốt CON SỐ, lúc dựng. Đổi một dấu `*` thành `+` vẫn biên dịch, vẫn chạy, chỉ
// ra một trần khác hẳn — và không phép thử nào nhìn vào một biểu thức hằng.
const _: () = assert!(MAX_RESPONSE == 1_048_576);

/// Thời gian chờ. Ngắn hơn `get` của ứng dụng: đây là một lần gọi ta chủ động,
/// và người dùng đang ngồi chờ trước màn hình.
const TIMEOUT: Duration = Duration::from_secs(20);

/// Máy khách JSON-RPC. Một điểm cuối, không giữ trạng thái gì.
#[derive(Debug, Clone)]
pub struct JsonRpc {
    endpoint: String,
}

/// Đọc thân phản hồi JSON-RPC. **Hàm thuần** — kiểm được không cần máy chủ.
///
/// Tách ra vì phần này mang ba quyết định mà một máy chủ thù địch chạm tới
/// được, và cả ba từng nằm trong một hàm chỉ chạy khi có mạng thật:
///
/// * `"error"` KHÔNG rỗng thì đó là lỗi, dù có `result` hay không;
/// * `"error": null` là chuyện BÌNH THƯỜNG của JSON-RPC, không phải lỗi;
/// * thiếu `result` thì nói rõ là thiếu, đừng trả một giá trị rỗng.
///
/// Kiểm đột biến 26/08/2026: xoá dấu `!` trong `filter(|e| !e.is_null())` thì
/// MỌI lời gọi thành công đều thành lỗi, và không phép thử nào đỏ — vì không
/// phép thử nào chạy tới đây.
fn doc_phan_hoi(chu: &str) -> Result<serde_json::Value, NetError> {
    let v: serde_json::Value =
        serde_json::from_str(chu).map_err(|e| NetError::Goi(format!("không phải JSON: {e}")))?;

    if let Some(loi) = v.get("error").filter(|e| !e.is_null()) {
        return Err(NetError::Goi(format!("máy chủ trả lỗi: {loi}")));
    }
    v.get("result")
        .cloned()
        .ok_or_else(|| NetError::Goi("phản hồi thiếu trường result".to_owned()))
}

impl JsonRpc {
    /// # Errors
    /// Điểm cuối không phải `https://`.
    pub fn new(endpoint: impl Into<String>) -> Result<Self, NetError> {
        let endpoint = endpoint.into();
        if !endpoint.starts_with("https://") {
            return Err(NetError::KhongPhaiHttps(endpoint));
        }
        Ok(Self { endpoint })
    }

    /// Gọi một phương thức. `params` là JSON đã dựng sẵn.
    ///
    /// Trả về **nguyên trường `result`**, chưa diễn giải gì. Diễn giải là việc
    /// của bên gọi, và bên gọi phải coi nó là dữ liệu thù địch.
    ///
    /// # Errors
    /// Mạng hỏng, trạng thái không phải 2xx, chuyển hướng, phản hồi quá trần,
    /// không phải JSON, hoặc máy chủ trả `error`.
    pub fn call(
        &self,
        method: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, NetError> {
        let than = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });

        let cau_hinh = ureq::Agent::config_builder()
            .timeout_global(Some(TIMEOUT))
            // Không theo chuyển hướng. Ở ứng dụng đây là đòn thoát khỏi quyền
            // năng; ở đây nó là đòn đưa ví đi nói chuyện với máy chủ khác.
            .max_redirects(0)
            .build();
        let agent: ureq::Agent = cau_hinh.into();

        let mut tra = agent
            .post(&self.endpoint)
            .header("content-type", "application/json")
            // Tự tuần tự hoá rồi gửi chuỗi, thay vì bật tính năng `json` của
            // `ureq`: bật nó kéo thêm `serde` vào cây phụ thuộc của một crate
            // vốn cố ý mỏng, để đổi lấy đúng một dòng.
            .send(than.to_string())
            .map_err(|e| NetError::Goi(e.to_string()))?;

        // Lớp hai: tự từ chối 3xx. `max_redirects(0)` đã chặn, nhưng lớp này
        // kiểm thử được mà không cần máy chủ thật nên nó không bao giờ mục.
        crate::check_status(tra.status().as_u16())?;

        let chu = tra
            .body_mut()
            .with_config()
            .limit(MAX_RESPONSE)
            .read_to_string()
            .map_err(|e| crate::dich_loi_doc(&e, MAX_RESPONSE))?;

        doc_phan_hoi(&chu)
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay"
)]
mod kiem_thu {
    use super::*;

    /// HTTP trần bị chặn ngay lúc dựng, không đợi tới lúc gọi.
    #[test]
    fn http_tran_bi_chan_ngay_luc_dung() {
        assert!(JsonRpc::new("http://rpc2.tcc-coin.com").is_err());
        assert!(JsonRpc::new("rpc2.tcc-coin.com").is_err());
        assert!(JsonRpc::new("https://rpc2.tcc-coin.com").is_ok());
    }

    /// Chuyển hướng bị từ chối bởi lớp kiểm trạng thái — kiểm được mà không
    /// cần máy chủ thật.
    #[test]
    fn chuyen_huong_bi_tu_choi() {
        for ma in [301, 302, 303, 307, 308] {
            assert!(crate::check_status(ma).is_err(), "{ma} lọt qua");
        }
    }

    /// **Ba quyết định của phản hồi JSON-RPC, cả ba máy chủ chạm tới được.**
    ///
    /// Trước 26/08/2026 chúng nằm trong một hàm chỉ chạy khi có mạng thật, nên
    /// không phép thử nào tới. Kiểm đột biến: xoá dấu `!` trong
    /// `filter(|e| !e.is_null())` thì MỌI lời gọi thành công thành lỗi, mà mọi
    /// phép thử vẫn xanh.
    #[test]
    fn doc_phan_hoi_phan_biet_ba_truong_hop() {
        // `error: null` là chuyện BÌNH THƯỜNG — không phải lỗi.
        assert_eq!(
            doc_phan_hoi(r#"{"result":42,"error":null}"#).unwrap(),
            serde_json::json!(42),
            "`error: null` bị coi là lỗi — mọi lời gọi thành công sẽ hỏng"
        );

        // `error` có thật thì là lỗi, DÙ có `result`.
        let loi = doc_phan_hoi(r#"{"result":42,"error":{"code":-1}}"#).unwrap_err();
        assert!(
            loi.to_string().contains("máy chủ trả lỗi"),
            "máy chủ báo lỗi mà vẫn nhận `result`: {loi}"
        );

        // Thiếu `result` thì nói rõ là thiếu, không trả giá trị rỗng.
        let loi = doc_phan_hoi(r#"{"jsonrpc":"2.0"}"#).unwrap_err();
        assert!(loi.to_string().contains("thiếu trường result"), "{loi}");

        // Không phải JSON thì nói rõ thế.
        let loi = doc_phan_hoi("khong-phai-json").unwrap_err();
        assert!(loi.to_string().contains("không phải JSON"), "{loi}");
    }
}
