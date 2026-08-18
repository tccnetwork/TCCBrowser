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
    let dung = || WebViewBuilder::new().with_url(url);
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
    let trang = WebViewBuilder::new()
        .with_url(url_dau)
        .with_bounds(Rect {
            position: LogicalPosition::new(0.0, CHROME_HEIGHT).into(),
            size: wry::dpi::LogicalSize::new(co.width, co.height - CHROME_HEIGHT).into(),
        })
        .build_as_child(&window)
        .map_err(|e| format!("không dựng được WebView trang: {e}"))?;
    // ▲▲▲ TRANG-KET-THUC ▲▲▲

    vong.run_return(move |su_kien, _, dieu_khien| {
        *dieu_khien = ControlFlow::Wait;
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
            if let Some(url) = doc_dia_chi(&tin)
                && check_web_url(&url).is_ok()
            {
                let _ = trang.load_url(&url);
            }
        }
    });
    Ok(())
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
        let nguon = include_str!("web_tier.rs");
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
    }
}
