//! **Tầng 2** — mở một trang web thật.
//!
//! # Ranh giới quan trọng nhất của tệp này
//!
//! Trang web **mang mã của nó**. Nó không có chữ ký, không đi qua cổng quyền
//! năng, và không ai kiểm nó trước.
//!
//! Nên WebView ở đây **KHÔNG có `with_ipc_handler`** và **KHÔNG có
//! `with_initialization_script`**. Đó không phải chuyện tiết kiệm: mọi WebView
//! của khung trình duyệt đều cài hai thứ ấy, và một trang chạy chung WebView ấy
//! sẽ gọi được `window.ipc.postMessage` — tức là **trả lời hộ người dùng** cho
//! hộp thoại hỏi quyền hoặc màn xác nhận giao dịch.
//!
//! ```text
//! WebView của KHUNG   →  có IPC, có kịch bản của ta, KHÔNG nạp trang ngoài
//! WebView của TRANG   →  không IPC, không kịch bản, chỉ nạp https://
//! ```
//!
//! Hai thứ ấy không bao giờ là một, và phép thử `khong_co_ipc_va_kich_ban`
//! chốt điều đó bằng cách đọc chính mã nguồn này.
//!
//! # Chỉ `https://`
//!
//! Cùng luật với `external_link`: `file://` đọc trộm đĩa, `javascript:` chạy mã
//! trong ngữ cảnh ta vừa mở. Nhưng ở đây **chặt hơn một bậc** — `http://` cũng
//! bị từ chối, vì trang tải qua đường trần thì bất kỳ ai trên đường cũng sửa
//! được nội dung, mà ta lại đang đặt nó trong một cửa sổ mang tên TCC.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tao::platform::run_return::EventLoopExtRunReturn as _;
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::{Window, WindowBuilder},
};
use wry::{Rect, WebViewBuilder, dpi::LogicalPosition};

/// Chiều cao thanh địa chỉ, đơn vị logic.
pub const CHROME_HEIGHT: f64 = 52.0;

/// Vì sao không mở được.
#[derive(Debug, PartialEq, Eq)]
pub enum WebTierError {
    /// Không phải `https://`.
    ///
    /// Gộp mọi lược đồ khác vào một lỗi: kể tên lược đồ nào bị chặn là dạy
    /// người thử biết cái gì đã được nghĩ tới.
    NotHttps,
    BadChars,
}

impl core::fmt::Display for WebTierError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotHttps => write!(f, "tầng 2 chỉ mở https://"),
            Self::BadChars => write!(f, "địa chỉ chứa ký tự không được phép"),
        }
    }
}

impl std::error::Error for WebTierError {}

/// Kiểm địa chỉ trước khi mở.
///
/// # Errors
/// Không phải `https://`, hoặc chứa ký tự điều khiển/khoảng trắng.
pub fn check_web_url(url: &str) -> Result<(), WebTierError> {
    // Ký tự điều khiển bị chặn TRƯỚC khi xét lược đồ: `https://a\n…` có lược đồ
    // hoàn toàn hợp lệ.
    if url.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err(WebTierError::BadChars);
    }
    if !url.to_ascii_lowercase().starts_with("https://") || url.len() <= 8 {
        return Err(WebTierError::NotHttps);
    }
    Ok(())
}

/// Dựng WebView của TRANG trên một cửa sổ đã có.
///
/// # ⚠️ Không IPC, không kịch bản
///
/// Xem ghi chú đầu tệp. Đổi hàm này để thêm `with_ipc_handler` là mở đúng cái
/// cửa mà cả tệp này sinh ra để đóng.
///
/// # Errors
/// Địa chỉ không hợp lệ, hoặc không dựng được `WebView`.
pub fn attach_page(window: &Window, url: &str) -> Result<wry::WebView, String> {
    check_web_url(url).map_err(|e| e.to_string())?;
    // ▼▼▼ TRANG-BAT-DAU ▼▼▼ — phép thử `khong_co_ipc_va_kich_ban` soi đúng
    // khúc giữa hai dấu này. Mọi chỗ dựng WebView cho TRANG WEB phải nằm trong
    // đây, và trong đây không được có IPC hay kịch bản của khung.
    let dung = || chan_lai(WebViewBuilder::new().with_url(url));
    // ▲▲▲ TRANG-KET-THUC ▲▲▲
    dung()
        .build(window)
        .map_err(|e| format!("không dựng được WebView: {e}"))
}

/// Mở một cửa sổ chỉ chứa trang web, và chạy tới khi người dùng đóng.
///
/// # Errors
/// Địa chỉ không hợp lệ, hoặc không dựng được cửa sổ.
pub fn open_page(url: &str, tieu_de: &str) -> Result<(), String> {
    use tao::event::{Event, WindowEvent};
    use tao::event_loop::ControlFlow;

    check_web_url(url).map_err(|e| e.to_string())?;
    let vong = EventLoopBuilder::new().build();
    let window = WindowBuilder::new()
        .with_title(tieu_de)
        .with_inner_size(tao::dpi::LogicalSize::new(1100.0, 800.0))
        .build(&vong)
        .map_err(|e| format!("không dựng được cửa sổ: {e}"))?;
    let _webview = attach_page(&window, url)?;

    vong.run(move |su_kien, _, dieu_khien| {
        *dieu_khien = ControlFlow::Wait;
        if matches!(
            su_kien,
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
        ) {
            *dieu_khien = ControlFlow::Exit;
        }
    });
}

/// Mở cửa sổ có **thanh địa chỉ** ở trên và **trang web** bên dưới.
///
/// # Hai WebView, không chồng lấn
///
/// | Vùng | WebView | IPC | Kịch bản |
/// |---|---|---|---|
/// | trên, cao [`CHROME_HEIGHT`] | của KHUNG | có | có |
/// | dưới, phần còn lại | của TRANG | **không** | **không** |
///
/// Đặt hai vùng **không đè nhau** là có chủ ý: chồng lấn thì thứ tự lớp quyết
/// định ai che ai, mà `wry` 0.52 không có API đặt thứ tự lớp — trên macOS nó là
/// thứ tự gọi `addSubview`. Một thanh địa chỉ bị trang che là một thanh địa chỉ
/// nói dối về nơi người dùng đang đứng.
///
/// # Errors
/// Địa chỉ đầu không hợp lệ, hoặc không dựng được cửa sổ/WebView.
pub fn open_browser(url_dau: &str, tieu_de: &str, tai_lieu_khung: &str) -> Result<(), String> {
    check_web_url(url_dau).map_err(|e| e.to_string())?;

    let mut vong = EventLoopBuilder::new().build();
    let window = WindowBuilder::new()
        .with_title(tieu_de)
        .with_inner_size(LogicalSize::new(1100.0, 800.0))
        .build(&vong)
        .map_err(|e| format!("không dựng được cửa sổ: {e}"))?;

    let co = window.inner_size().to_logical::<f64>(window.scale_factor());
    let hang: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let hang_ipc = Arc::clone(&hang);

    // ── WebView của KHUNG: có IPC, có kịch bản ──
    let khung = WebViewBuilder::new()
        .with_html(tai_lieu_khung)
        .with_initialization_script(crate::window::KICH_BAN_KHUNG)
        .with_ipc_handler(move |yc| {
            if let Ok(mut q) = hang_ipc.lock() {
                q.push(yc.body().clone());
            }
        })
        .with_bounds(Rect {
            position: LogicalPosition::new(0.0, 0.0).into(),
            size: wry::dpi::LogicalSize::new(co.width, CHROME_HEIGHT).into(),
        })
        .build_as_child(&window)
        .map_err(|e| format!("không dựng được WebView khung: {e}"))?;

    // ── WebView của TRANG: KHÔNG IPC, KHÔNG kịch bản ──
    // ▼▼▼ TRANG-BAT-DAU ▼▼▼
    let trang = chan_lai(WebViewBuilder::new().with_url(url_dau))
        .with_bounds(Rect {
            position: LogicalPosition::new(0.0, CHROME_HEIGHT).into(),
            size: wry::dpi::LogicalSize::new(co.width, co.height - CHROME_HEIGHT).into(),
        })
        .build_as_child(&window)
        .map_err(|e| format!("không dựng được WebView trang: {e}"))?;
    // ▲▲▲ TRANG-KET-THUC ▲▲▲

    vong.run_return(move |su_kien, _, dieu_khien| {
        // ⚠️ `WaitUntil`, KHÔNG phải `Wait`.
        //
        // Tin nhắn từ ô địa chỉ đi vào hàng đợi qua bộ xử lý IPC, mà việc đẩy
        // vào hàng đợi KHÔNG sinh ra một sự kiện cửa sổ nào. Với `Wait` thì
        // vòng lặp ngủ cho tới khi có sự kiện khác — nên người dùng bấm "Đi"
        // và **không có gì xảy ra**, cho tới khi họ tình cờ rê chuột qua cửa sổ.
        //
        // Đã trả giá 18/08/2026. `window.rs::run_loop` dùng `WaitUntil` đúng vì
        // lý do này, và tôi đã không nhìn sang đó.
        *dieu_khien = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(50));
        match su_kien {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *dieu_khien = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(kich_thuoc),
                ..
            } => {
                let l = kich_thuoc.to_logical::<f64>(window.scale_factor());
                let _ = khung.set_bounds(Rect {
                    position: LogicalPosition::new(0.0, 0.0).into(),
                    size: wry::dpi::LogicalSize::new(l.width, CHROME_HEIGHT).into(),
                });
                let _ = trang.set_bounds(Rect {
                    position: LogicalPosition::new(0.0, CHROME_HEIGHT).into(),
                    size: wry::dpi::LogicalSize::new(l.width, l.height - CHROME_HEIGHT).into(),
                });
            }
            _ => {}
        }

        let cho: Vec<String> = hang
            .lock()
            .map(|mut q| q.drain(..).collect())
            .unwrap_or_default();
        for tin in cho {
            // Địa chỉ đến từ ô nhập của KHUNG, nhưng vẫn kiểm lại: ô nhập là nơi
            // người dùng gõ, và người dùng dán vào đó bất cứ thứ gì.
            let Some(go) = doc_dia_chi(&tin) else {
                continue;
            };
            match normalise_url(&go) {
                Ok(url) => {
                    let _ = trang.load_url(&url);
                }
                // Nói ra. Bản đầu bỏ qua im lặng, và người dùng gõ một địa chỉ
                // hợp lý rồi thấy KHÔNG có gì xảy ra — không biết mình gõ sai,
                // hay trình duyệt hỏng.
                Err(e) => {
                    let _ = khung.evaluate_script(&format!(
                        "(function(){{var o=document.querySelector('input[aria-label]');\
                          if(o){{o.value={};}}}})()",
                        serde_json::Value::from(format!("{go}  ← {e}"))
                    ));
                }
            }
        }
    });
    Ok(())
}

/// Đặt mọi chắn lên WebView của TRANG.
///
/// # Vì sao gom vào một hàm
///
/// Ba chắn dưới đây phải đi cùng nhau. Dựng WebView cho trang ở hai chỗ mà chỉ
/// một chỗ có chắn thì chỗ kia là một cửa mở, và không có gì báo.
///
/// | Chắn | Chặn cái gì |
/// |---|---|
/// | Điều hướng | Trang tự nhảy sang `file://`, `javascript:`, hay chính giao thức riêng của ta |
/// | Cửa sổ mới | `window.open` / `target=_blank` mở ra một khung KHÔNG có chắn nào |
/// | Tải tệp | Engine ghi thẳng ra đĩa, tên và đuôi do TRANG chọn |
/// | Bảng nháp | Trang đọc trộm thứ người dùng vừa sao chép — thường là mật khẩu |
/// | Tự phát | Âm thanh nổ ra từ một cửa sổ người dùng chưa nhìn tới |
fn chan_lai(b: WebViewBuilder<'_>) -> WebViewBuilder<'_> {
    b.with_navigation_handler(|url| allow_navigation(&url))
        .with_new_window_req_handler(|url| allow_new_window(&url))
        .with_download_started_handler(|url, _duong_dan| allow_download(&url))
        // Mặc định của `wry` đã là `false`, nhưng viết ra: một trang đọc được
        // bảng nháp là một trang đọc được thứ người dùng vừa sao chép, mà thứ
        // ấy rất hay là mật khẩu.
        .with_clipboard(false)
        // Mặc định của `wry` là `true`. Tắt: tiếng nổ ra từ một cửa sổ người
        // dùng chưa kịp nhìn là chuyện của quảng cáo, không phải của trình duyệt.
        .with_autoplay(false)
}

/// Cho phép điều hướng tới đâu.
///
/// Kiểm LẠI mỗi lần, không chỉ lần nạp đầu: trang chuyển hướng được sau khi đã
/// nạp, và lần chuyển ấy mới là lần nguy hiểm — nó đưa `file://` hay chính
/// giao thức riêng của ta vào một khung người dùng tưởng là trang web thường.
#[must_use]
pub fn allow_navigation(url: &str) -> bool {
    check_web_url(url).is_ok()
}

/// Cho mở cửa sổ mới không. **Không.**
///
/// `window.open` và `target=_blank` mở ra một WebView mà TA không dựng — nên
/// không có chắn nào ở đó cả: không kiểm điều hướng, không chặn tải tệp.
///
/// Thà một liên kết không mở còn hơn một khung không ai canh. Người dùng dán
/// địa chỉ vào thanh nếu muốn.
#[must_use]
pub const fn allow_new_window(_url: &str) -> bool {
    false
}

/// Cho tải tệp không. **Chưa.**
///
/// Trả `false` là KHÔNG tải. Thà nói "chưa làm" còn hơn ghi ra đĩa một tệp mà
/// tên, đuôi và chỗ đặt đều do TRANG chọn.
#[must_use]
pub const fn allow_download(_url: &str) -> bool {
    false
}

/// Chuẩn hoá thứ người dùng gõ thành một địa chỉ mở được.
///
/// # Vì sao KHÔNG bắt người dùng gõ `https://`
///
/// Không ai gõ `https://` khi nhớ một trang. Bắt gõ đủ thì mọi lần quên là một
/// lần "bấm Đi mà không có gì xảy ra" — và người dùng học rằng trình duyệt này
/// hỏng, chứ không học rằng mình gõ thiếu.
///
/// Nhưng **chỉ thêm `https://`**, không bao giờ thêm `http://`: đoán xuống
/// đường trần là tự hạ bảo vệ hộ người dùng. Ai thật sự muốn `http://` thì gõ
/// đủ, và lúc ấy [`check_web_url`] từ chối — cố ý.
///
/// # Errors
/// Sau khi thêm lược đồ vẫn không hợp lệ.
pub fn normalise_url(go: &str) -> Result<String, WebTierError> {
    let go = go.trim();
    // Đã có lược đồ thì KHÔNG đụng vào: `http://a` phải bị từ chối, không được
    // lặng lẽ nâng thành `https://a` — người dùng gõ gì thì đi tới đúng thứ đó.
    if go.contains("://") {
        check_web_url(go)?;
        return Ok(go.to_owned());
    }
    // ⚠️ `javascript:alert(1)` không có `://`, nên nếu chỉ ghép lược đồ vào thì
    // nó thành `https://javascript:alert(1)` — một chuỗi vô hại vì không phân
    // giải được, NHƯNG người dùng gõ một thứ và trình duyệt mở một thứ khác.
    // Thà từ chối: chỗ duy nhất được phép có `:` trong tên máy là cổng, và cổng
    // là số.
    let may = go.split('/').next().unwrap_or(go);
    if let Some((_, cong)) = may.split_once(':')
        && (cong.is_empty() || !cong.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(WebTierError::NotHttps);
    }
    let du = format!("https://{go}");
    check_web_url(&du)?;
    Ok(du)
}

/// Lấy địa chỉ ra khỏi thông điệp của thanh địa chỉ.
///
/// Thông điệp mang dạng `{"a":"…","o":[[nhãn,giá trị]…]}` — cùng dạng
/// `KICH_BAN_KHUNG` gửi cho mọi màn hình của khung.
fn doc_dia_chi(than: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(than).ok()?;
    v.get("o")?
        .as_array()?
        .iter()
        .find_map(|x| x.as_array()?.get(1)?.as_str().map(str::to_owned))
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay"
)]
mod kiem_thu {
    use super::*;

    /// **Chỉ `https://`** — và `http://` cũng bị từ chối.
    #[test]
    fn chi_nhan_https() {
        assert!(check_web_url("https://vnexpress.net").is_ok());
        assert!(check_web_url("HTTPS://VNEXPRESS.NET").is_ok());
        for xau in [
            "http://vnexpress.net",
            "file:///etc/passwd",
            "javascript:alert(1)",
            "data:text/html,x",
            "https://",
            "",
            "vnexpress.net",
            "https://a.example\nhttps://ke-gian.example",
            "https://a.example; rm -rf /",
        ] {
            assert!(check_web_url(xau).is_err(), "nhận nhầm {xau:?}");
        }
    }

    /// **WebView của TRANG không được có IPC hay kịch bản của khung.**
    ///
    /// Phép thử đọc chính mã nguồn tệp này. Nghe thô, nhưng nó là cách duy nhất
    /// chốt một điều KHÔNG có mặt — và điều không có mặt ấy là cả lý do tệp này
    /// tồn tại: một trang chạy chung WebView với khung sẽ gọi được
    /// `window.ipc.postMessage` và trả lời hộ người dùng.
    #[test]
    fn khong_co_ipc_va_kich_ban() {
        // Cắt bỏ khối kiểm thử TRƯỚC khi tìm dấu mốc: chính khối này cũng chứa
        // hai chuỗi dấu mốc, nên không cắt thì nó tự soi lấy mình và báo là mã
        // của trang thiếu chắn. Bản đầu tôi quên, và nó đỏ ngay.
        let nguon = include_str!("web_tier.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap_or_default();
        // Soi ĐÚNG những khúc dựng WebView cho TRANG, đánh dấu bằng hai mốc.
        //
        // Bản đầu soi cả tệp, và nó đỏ ngay khi thanh địa chỉ ra đời — vì
        // WebView của KHUNG thì PHẢI có IPC. Một phép thử đúng ý mà sai phạm vi
        // thì chỉ dạy người ta tắt nó đi.
        let khuc: String = nguon
            .split("▼▼▼ TRANG-BAT-DAU ▼▼▼")
            .skip(1)
            .filter_map(|p| p.split("▲▲▲ TRANG-KET-THUC ▲▲▲").next())
            .collect();
        assert!(
            khuc.contains("with_url"),
            "không tìm thấy khúc dựng WebView của trang — dấu mốc bị xoá?"
        );
        let ma = khuc
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<String>();
        assert!(
            !ma.contains("with_ipc_handler"),
            "WebView của trang đang cài IPC của khung"
        );
        assert!(
            !ma.contains("with_initialization_script"),
            "WebView của trang đang cài kịch bản của khung"
        );
        // Và MỌI chỗ dựng WebView cho trang phải đi qua `chan_lai`.
        for khuc_mot in nguon
            .split("▼▼▼ TRANG-BAT-DAU ▼▼▼")
            .skip(1)
            .filter_map(|p| p.split("▲▲▲ TRANG-KET-THUC ▲▲▲").next())
        {
            assert!(
                khuc_mot.contains("chan_lai"),
                "một chỗ dựng WebView cho trang KHÔNG đi qua `chan_lai`:\n{khuc_mot}"
            );
        }
    }

    /// **Gõ tên miền trơn thì vẫn mở được** — không bắt gõ `https://`.
    #[test]
    fn ten_mien_tron_duoc_them_https() {
        assert_eq!(
            normalise_url("vnexpress.net").unwrap(),
            "https://vnexpress.net"
        );
        assert_eq!(
            normalise_url("  vnexpress.net/tin  ").unwrap(),
            "https://vnexpress.net/tin"
        );
    }

    /// Nhưng **không bao giờ tự hạ xuống `http://`**.
    ///
    /// Đoán xuống đường trần là tự hạ bảo vệ hộ người dùng.
    #[test]
    fn khong_tu_ha_xuong_http() {
        assert!(normalise_url("http://a.example").is_err());
        assert!(normalise_url("file:///etc/passwd").is_err());
        assert!(normalise_url("javascript:alert(1)").is_err());
        // Đã có lược đồ thì giữ nguyên, không nâng cấp hộ.
        assert_eq!(
            normalise_url("https://a.example").unwrap(),
            "https://a.example"
        );
    }

    /// **Cửa sổ mới và tải tệp đều bị TỪ CHỐI.**
    ///
    /// Kiểm đột biến tìm ra chỗ này: phép thử cũ chỉ chốt rằng `chan_lai` được
    /// gọi, nên đổi `false` thành `true` vẫn xanh. Kiểm sự CÓ MẶT của một chắn
    /// mà không kiểm nó QUYẾT ĐỊNH gì thì chắn ấy có thể mở toang.
    #[test]
    fn cua_so_moi_va_tai_tep_deu_bi_tu_choi() {
        assert!(!allow_new_window("https://a.example"));
        assert!(!allow_new_window("https://ai-cung-the.example"));
        assert!(!allow_download("https://a.example/x.dmg"));
        assert!(!allow_download("https://a.example/vo-hai.txt"));
    }

    /// **Điều hướng bị kiểm LẠI mỗi lần**, không chỉ lần nạp đầu.
    ///
    /// Trang chuyển hướng được sau khi đã nạp, và lần chuyển ấy mới là lần
    /// nguy hiểm: nó đưa `file://` hay giao thức riêng của ta vào một khung
    /// người dùng tưởng là trang web thường.
    #[test]
    fn chan_dieu_huong_dung_cung_luat_voi_lan_nap_dau() {
        // Cùng vị từ với `check_web_url`, nên mọi ca của phép thử kia áp luôn.
        for xau in [
            "file:///etc/passwd",
            "javascript:alert(1)",
            "http://a.example",
            "tcc-goi://goi/x",
        ] {
            assert!(!allow_navigation(xau), "{xau} lọt qua");
        }
        assert!(allow_navigation("https://a.example/trang-khac"));
    }
}
