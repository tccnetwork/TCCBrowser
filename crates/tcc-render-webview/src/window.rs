//! Cửa sổ thật. Chỉ biên dịch khi bật cờ tính năng `window`.
//!
//! # Vì sao để sau cờ tính năng
//!
//! `wry` + `tao` kéo theo 71 crate và cần một màn hình để chạy. Để nó trong bản
//! dựng mặc định nghĩa là mỗi lần `cargo test` phải dựng cả 71 crate đó, và
//! nghĩa là CI không có màn hình sẽ hỏng. Tách ra thì phần logic — nơi có lỗi
//! bảo mật thật — kiểm được nhanh và kiểm được ở mọi nơi.
//!
//! ```text
//!   cargo test                  → chỉ tcc-ui + dịch đánh dấu + quét trợ năng
//!   cargo run --features window → thêm cửa sổ thật
//! ```
//!
//! # Ranh giới bảo mật của tệp này
//!
//! Tài liệu nạp vào WebView qua `with_html`, KHÔNG qua tệp và KHÔNG qua địa chỉ
//! mạng. Nghĩa là trang chạy trong một nguồn gốc trống: nó không có bạn bè cùng
//! nguồn gốc, không đọc được kho lưu trữ của trang nào khác, và chính sách nội
//! dung nhúng trong tài liệu là luật cuối cùng.

use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    platform::run_return::EventLoopExtRunReturn as _,
    window::WindowBuilder,
};
use wry::WebViewBuilder;

use crate::package_server;

/// Dựng một `WebView` phủ kín cửa sổ — **chọn đúng đường của từng nền**.
///
/// # Vì sao không gọi thẳng `build()` ở mọi nơi
///
/// Trên Linux, `wry` dựng `WebView` vào một widget GTK, không vào cửa sổ. Gọi
/// `build(&window)` ở đó trả về *"the underlying handle is not available"* —
/// một câu không nhắc gì tới GTK, nên nó **đọc y như một lỗi của màn hình ảo**.
///
/// Và suốt một thời gian tôi đã đọc đúng như thế: bước đo nền tảng trên Linux
/// mang `continue-on-error` kèm chú thích nói rằng WebKitGTK dưới `xvfb` không
/// ổn định. Nó không hề chập chờn — nó **trượt đều 3/3 lượt**, và trượt vì mã
/// này, không vì máy chạy. Một phép thử được miễn vì "hạ tầng chập chờn" là
/// chỗ tốt nhất để một lỗi thật nằm im.
///
/// # Errors
/// Không dựng được `WebView`, hoặc trên Linux cửa sổ không có hộp chứa mặc định.
pub(crate) trait BuildFilling<'a> {
    /// # Errors
    /// Không dựng được `WebView`, hoặc trên Linux cửa sổ không có hộp chứa GTK.
    fn build_filling(self, window: &'a tao::window::Window) -> Result<wry::WebView, String>;
}

impl<'a> BuildFilling<'a> for WebViewBuilder<'a> {
    fn build_filling(self, window: &'a tao::window::Window) -> Result<wry::WebView, String> {
        #[cfg(target_os = "linux")]
        {
            use tao::platform::unix::WindowExtUnix as _;
            use wry::WebViewBuilderExtUnix as _;
            let hop = window
                .default_vbox()
                .ok_or_else(|| "cửa sổ không có hộp chứa GTK mặc định".to_owned())?;
            self.build_gtk(hop).map_err(|e| e.to_string())
        }
        #[cfg(not(target_os = "linux"))]
        {
            self.build(window).map_err(|e| e.to_string())
        }
    }
}

/// Mở một cửa sổ và nạp tài liệu đã dựng.
///
/// `tu_dong_dong` để kiểm khói: đặt một khoảng thời gian thì cửa sổ tự đóng sau
/// đó. Không có nó thì không có cách nào kiểm "cửa sổ có mở được không" trong
/// một lệnh chạy tự động — nó sẽ treo mãi.
///
/// # Errors
/// Không dựng được cửa sổ hoặc không dựng được WebView.
///
/// # Panics
/// Không.
pub fn open(
    document: &str,
    tieu_de: &str,
    doc_tep: impl Fn(&str) -> Option<Vec<u8>> + 'static,
    tu_dong_dong: Option<Duration>,
) -> Result<(), Box<dyn std::error::Error>> {
    let vong = EventLoopBuilder::new().build();
    let window = WindowBuilder::new()
        .with_title(tieu_de)
        .with_inner_size(tao::dpi::LogicalSize::new(900.0, 640.0))
        .build(&vong)?;

    // `with_html` chứ không phải `with_url`: tài liệu không có nguồn gốc thật,
    // nên không chia sẻ kho lưu trữ hay quyền với bất kỳ trang nào.
    let _webview = WebViewBuilder::new()
        .with_html(document)
        .with_custom_protocol(package_server::SCHEME.to_owned(), move |_id, yc| {
            serve(&doc_tep, &yc)
        })
        .build_filling(&window)?;

    let han = tu_dong_dong.map(|d| Instant::now() + d);
    // ⚠️ Cần cờ này. Vòng lặp chạy lại cho MỖI sự kiện, và dòng đặt `ControlFlow`
    // ở đầu thân hàm sẽ GHI ĐÈ lệnh `Exit` đã đặt ở vòng trước. Bản đầu tôi viết
    // thiếu cờ và nó lộ ra khi chạy thử: dòng "tự đóng" in 5 lần. Hậu quả thật
    // nặng hơn thế — bấm nút đóng cửa sổ cũng có thể bị nuốt nếu ngay sau đó có
    // một sự kiện khác (di chuột, đổi cỡ) đặt lại `ControlFlow`.
    let mut dang_thoat = false;

    vong.run(move |su_kien, _, dieu_khien| {
        if dang_thoat {
            *dieu_khien = ControlFlow::Exit;
            return;
        }
        *dieu_khien = han.map_or(ControlFlow::Wait, ControlFlow::WaitUntil);

        let mut escape_html = |vi_sao: &str| {
            println!("[bộ dựng] đóng cửa sổ: {vi_sao}");
            dang_thoat = true;
            *dieu_khien = ControlFlow::Exit;
        };

        match su_kien {
            Event::NewEvents(StartCause::Init) => println!("[bộ dựng] cửa sổ đã mở"),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => escape_html("người dùng bấm đóng"),
            _ => {
                if han.is_some_and(|h| Instant::now() >= h) {
                    escape_html("hết hẹn giờ");
                }
            }
        }
    });
}

/// Báo cáo mà CHÍNH WebKit gửi ngược ra sau khi dựng xong tài liệu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EscapeReport {
    /// Số phần tử mang vai trò trợ năng mà bộ dựng thật sự nhìn thấy.
    pub so_nut: usize,
    /// Danh sách vai trò, theo đúng thứ tự tài liệu.
    pub vai_tro: Vec<String>,
    /// Số phần tử kịch bản CÓ MẶT trong tài liệu.
    ///
    /// Ở đường ống thật phải là 0 — thoát ký tự không cho thẻ nào hình thành.
    pub so_kich_ban: usize,
    /// Kịch bản có THẬT SỰ CHẠY không.
    ///
    /// Khác `so_kich_ban`: một thẻ kịch bản có mặt nhưng bị chính sách nội dung
    /// chặn thì `so_kich_ban > 0` mà `da_chay == false`. Phân biệt hai điều này
    /// mới đo được từng tầng phòng thủ riêng.
    pub da_chay: bool,
}

/// Kịch bản chạy TRONG trang, chạy ở giai đoạn khởi tạo.
///
/// Nó là kịch bản của BỘ DỰNG, không phải của ứng dụng — chạy trước khi tài liệu
/// được phân tích, nên nó không bị chính sách nội dung của trang chặn. Ứng dụng
/// không có đường nào chèn vào đây.
const KICH_BAN_DO: &str = r"
document.addEventListener('DOMContentLoaded', function () {
  // Đếm '[role]' CỘNG ô nhập không mang role. Ô nhập cố ý không mang ARIA role
  // — ARIA đè lên ngữ nghĩa gốc và kéo ô mật khẩu tụt khỏi AXSecureTextField.
  // Xem B32 trong SECURITY.md. Đếm thiếu ở đây thì phép thử đỏ oan.
  var ds = document.querySelectorAll('[role], input:not([role])');
  var vt = [];
  for (var i = 0; i < ds.length; i++) vt.push(ds[i].getAttribute('role'));
  window.ipc.postMessage(JSON.stringify({
    so_nut: ds.length,
    vai_tro: vt,
    so_kich_ban: document.querySelectorAll('script').length,
    da_chay: window.da_chay !== undefined || window.doi_vi !== undefined
  }));
});
";

/// Kiểm khói ĐI QUA WEBKIT THẬT.
///
/// # Vì sao cần hàm này
///
/// Mọi phép thử khác chạy trên chuỗi đánh dấu do ta tự sinh và tự đọc lại — hai
/// đường khác nhau, nhưng vẫn là hai đường CỦA TA. Hàm này hỏi bên thứ ba: nạp
/// tài liệu vào WebKit thật, rồi bảo WebKit kể lại nó nhìn thấy gì.
///
/// Nếu WebKit hiểu tài liệu khác cách ta hiểu — nuốt một thẻ, đổi vai trò, hoặc
/// (tệ nhất) giữ lại một thẻ kịch bản — thì lệch lộ ra ở đây.
///
/// Cửa sổ mở ở chế độ ẨN: đây là phép kiểm, không phải màn hình cho người xem.
///
/// # Errors
/// Không dựng được cửa sổ, hoặc WebKit không báo về trong thời gian chờ.
pub fn check_escaping(document: &str, cho_toi_da: Duration) -> Result<EscapeReport, String> {
    let vong = EventLoopBuilder::new().build();
    let window = WindowBuilder::new()
        .with_visible(false)
        .build(&vong)
        .map_err(|e| format!("không dựng được cửa sổ: {e}"))?;

    let hop: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let hop_ipc = Arc::clone(&hop);

    let _webview = WebViewBuilder::new()
        .with_html(document)
        .with_initialization_script(KICH_BAN_DO)
        .with_ipc_handler(move |yeu_cau| {
            if let Ok(mut o) = hop_ipc.lock() {
                *o = Some(yeu_cau.body().clone());
            }
        })
        .build_filling(&window)
        .map_err(|e| format!("không dựng được WebView: {e}"))?;

    // `run_return` chứ KHÔNG phải `run`: `run` gọi thẳng `exit()` và không bao
    // giờ trả về, nên không có đường nào mang báo cáo ra ngoài. Đây đúng là chỗ
    // tôi viết sai lần đầu và trình biên dịch bắt được.
    let mut vong = vong;
    let han = Instant::now() + cho_toi_da;

    vong.run_return(|_su_kien, _, dieu_khien| {
        *dieu_khien = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(50));
        let xong = hop.lock().is_ok_and(|o| o.is_some());
        if xong || Instant::now() >= han {
            *dieu_khien = ControlFlow::Exit;
        }
    });

    let tho = hop
        .lock()
        .map_err(|_| "khoá hỏng".to_owned())?
        .clone()
        .ok_or_else(|| "WebKit không báo về trong thời gian chờ".to_owned())?;

    let v: serde_json::Value =
        serde_json::from_str(&tho).map_err(|e| format!("báo cáo không đọc được: {e}"))?;
    Ok(EscapeReport {
        so_nut: usize::try_from(v["so_nut"].as_u64().unwrap_or(0)).unwrap_or(usize::MAX),
        vai_tro: v["vai_tro"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|x| x.as_str().unwrap_or_default().to_owned())
                    .collect()
            })
            .unwrap_or_default(),
        so_kich_ban: usize::try_from(v["so_kich_ban"].as_u64().unwrap_or(0)).unwrap_or(usize::MAX),
        da_chay: v["da_chay"].as_bool().unwrap_or(true),
    })
}

/// Thứ WebKit THẬT SỰ nhận được vào một ô nhập.
///
/// Cổng ra Giai đoạn 1 đòi "gõ tiếng Việt có dấu, dấu chồng đúng, con trỏ đúng
/// chỗ". Ba vế đó không kiểm được bằng mắt: nhìn thấy `ỡ` không phân biệt được
/// nó là MỘT mã điểm dựng sẵn hay `o` cộng hai dấu rời, mà bộ gõ khác nhau cho
/// ra khác nhau — và phép kiểm trần dấu chồng (`MAX_COMBINING_MARKS`) đếm theo
/// mã điểm, nên khác biệt đó quyết định người dùng gõ được hay bị chặn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextInputProbe {
    /// Nguyên văn chuỗi trong ô nhập.
    pub value: String,
    /// Vị trí con trỏ do WebKit báo, tính bằng đơn vị mã UTF-16 của trang.
    pub caret_utf16: usize,
    /// Ô nhập còn đang trong phiên ghép của bộ gõ hay đã chốt.
    pub composing: bool,
}

/// Mở màn hình ứng dụng THẬT rồi hỏi lại WebKit ô nhập chứa gì.
///
/// Cửa sổ mở **nhìn thấy được** — khác mọi phép kiểm khác trong tệp này — vì
/// đây là phép kiểm duy nhất cần một con người gõ phím. Máy không thay được:
/// bộ gõ là của hệ điều hành, không phải của ta.
///
/// # Errors
/// Không dựng được cửa sổ, hoặc hết giờ mà WebKit không báo về.
pub fn probe_text_input(
    document: &str,
    doc_tep: impl Fn(&str) -> Option<Vec<u8>> + 'static,
    cho_toi_da: Duration,
) -> Result<TextInputProbe, String> {
    let vong = EventLoopBuilder::new().build();
    let window = WindowBuilder::new()
        .with_title("TCC — kiểm bộ gõ tiếng Việt")
        .build(&vong)
        .map_err(|e| format!("không dựng được cửa sổ: {e}"))?;

    let hop: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let hop_ipc = Arc::clone(&hop);

    let _webview = WebViewBuilder::new()
        .with_html(document)
        // Đi qua ĐÚNG trình phục vụ của đường chạy thật, không có bản rút gọn
        // cho phép kiểm: nếu ảnh trong gói hiện được ở đây mà không hiện ở kia
        // thì phép kiểm đang đo một thứ khác.
        .with_custom_protocol(package_server::SCHEME.to_owned(), move |_id, yc| {
            serve(&doc_tep, &yc)
        })
        .with_initialization_script(KICH_BAN_DO_BO_GO)
        .with_ipc_handler(move |yeu_cau| {
            if let Ok(mut o) = hop_ipc.lock() {
                // KHÔNG dùng `o.is_none()` như bộ nhận quyết định: ở đây ta muốn
                // trạng thái CUỐI CÙNG người dùng gõ ra, không phải cái đầu tiên.
                *o = Some(yeu_cau.body().clone());
            }
        })
        .build_filling(&window)
        .map_err(|e| format!("không dựng được WebView: {e}"))?;

    let mut vong = vong;
    let han = Instant::now() + cho_toi_da;
    let mut dang_thoat = false;
    vong.run_return(|su_kien, _, dieu_khien| {
        *dieu_khien = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(50));
        if matches!(
            su_kien,
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
        ) {
            dang_thoat = true;
        }
        if dang_thoat || Instant::now() >= han {
            *dieu_khien = ControlFlow::Exit;
        }
    });

    let tho = hop
        .lock()
        .map_err(|_| "khoá hỏng".to_owned())?
        .clone()
        .ok_or_else(|| "chưa gõ gì vào ô nhập nào".to_owned())?;
    let v: serde_json::Value =
        serde_json::from_str(&tho).map_err(|e| format!("báo cáo không đọc được: {e}"))?;
    Ok(TextInputProbe {
        value: v["value"].as_str().unwrap_or_default().to_owned(),
        caret_utf16: usize::try_from(v["caret"].as_u64().unwrap_or(0)).unwrap_or(usize::MAX),
        composing: v["composing"].as_bool().unwrap_or(false),
    })
}

/// Nạp một tài liệu, chạy một kịch bản, **lấy về đúng một chuỗi**.
///
/// Dùng để ĐO bộ máy web: hỏi nó có những gì, và nhận lại câu trả lời thô.
///
/// # Vì sao không đi qua `doc_tra_loi`
///
/// [`ask_dialog`] lọc thông điệp qua danh sách trắng mã hành động, vì ở đó
/// thông điệp là một **quyết định của người dùng**. Ở đây nó là một **phép đo
/// của ta**, gửi bởi kịch bản của chính ta, và nội dung là một bảng JSON tự do
/// — lọc bằng danh sách trắng thì không lọt được gì.
///
/// Đổi lại, hàm này **không** dùng cho tài liệu của ứng dụng: kịch bản đo là
/// kịch bản ta viết, không phải kịch bản ai gửi tới.
///
/// # Errors
/// Không dựng được cửa sổ hoặc `WebView`.
pub fn probe_document(
    document: &str,
    kich_ban: &str,
    cho_toi_da: Duration,
) -> Result<Option<String>, String> {
    let mut vong = EventLoopBuilder::new().build();
    let window = WindowBuilder::new()
        .with_visible(false)
        .build(&vong)
        .map_err(|e| format!("không dựng được cửa sổ: {e}"))?;

    let hop: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let hop_ipc = Arc::clone(&hop);

    let _webview = WebViewBuilder::new()
        .with_html(document)
        .with_initialization_script(kich_ban)
        .with_ipc_handler(move |yeu_cau| {
            if let Ok(mut o) = hop_ipc.lock()
                && o.is_none()
            {
                *o = Some(yeu_cau.body().clone());
            }
        })
        .build_filling(&window)
        .map_err(|e| format!("không dựng được WebView: {e}"))?;

    let het_gio = Instant::now() + cho_toi_da;
    vong.run_return(|_su_kien, _, dieu_khien| {
        *dieu_khien = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(30));
        let xong = hop.lock().map_or(true, |o| o.is_some());
        if xong || Instant::now() >= het_gio {
            *dieu_khien = ControlFlow::Exit;
        }
    });
    Ok(hop.lock().ok().and_then(|o| o.clone()))
}

/// Kịch bản CHẨN ĐOÁN bộ gõ — chỉ dùng cho `probe_text_input`.
///
/// Nó KHÔNG nằm trong đường chạy thật: màn hình ứng dụng bình thường không đọc
/// ngược nội dung ô nhập về host. Đọc ngược là một đường dữ liệu mới, và đường
/// đó chỉ được mở trong một phép kiểm có người ngồi trước máy.
const KICH_BAN_DO_BO_GO: &str = r"
(function () {
  var dang_ghep = false;
  function bao(el) {
    window.ipc.postMessage(JSON.stringify({
      value: el.value, caret: el.selectionStart, composing: dang_ghep
    }));
  }
  document.addEventListener('compositionstart', function (e) { dang_ghep = true; });
  document.addEventListener('compositionend', function (e) { dang_ghep = false; bao(e.target); });
  document.addEventListener('input', function (e) {
    if (e.target && e.target.value !== undefined) { bao(e.target); }
  });
})();
";

/// Kịch bản nối sự kiện, do BỘ DỰNG tiêm vào.
///
/// ⚠️ LUẬT: **bộ dựng nối sự kiện, ứng dụng KHÔNG BAO GIỜ.**
///
/// Ứng dụng chỉ khai một `ActionId`; nó không có, và không được có, một dòng
/// kịch bản nào — chính sách nội dung đặt `script-src 'none'`. Kịch bản này
/// chạy ở giai đoạn khởi tạo nên nó là kịch bản của bộ dựng, không phải của
/// trang, và ứng dụng không có đường nào chèn vào đây.
///
/// Nhờ vậy "ứng dụng chạy mã khi người dùng bấm nút" là chuyện không xảy ra
/// được — người dùng bấm, BỘ DỰNG nhận, rồi bộ dựng quyết định làm gì.
/// Kịch bản nối sự kiện cho **MÀN HÌNH CỦA KHUNG** — đọc thêm nội dung ô nhập.
///
/// # ⚠️ Vì sao là một kịch bản RIÊNG, không phải một cờ
///
/// Đọc ngược nội dung ô nhập từ WebView về Rust là **một đường dữ liệu mới**.
/// Nó cần thiết cho màn nhập ví (ô PIN), và **không cần** cho màn hình ứng
/// dụng: ứng dụng TCC không mang mã, nên không có ai bên trong nhận giá trị ấy
/// cả. Thu thập thứ không ai cần là mở rộng bề mặt mà không đổi lấy gì.
///
/// Tách thành hai hằng số chứ không một cờ `bool`: một cờ đặt sai vẫn biên
/// dịch, còn gọi nhầm hằng số thì đọc mã là thấy. Có phép thử chốt rằng kịch
/// bản của ỨNG DỤNG không chứa chữ `.value`.
pub(crate) const KICH_BAN_KHUNG: &str = r"
document.addEventListener('click', function (e) {
  var n = e.target;
  while (n) {
    if (n.getAttribute) {
      var a = n.getAttribute('data-hanh-dong');
      if (a) {
        if (n.getAttribute('role') === 'switch') { return; }
        var ct = document.querySelectorAll('[role=switch][data-hanh-dong]');
        var bat = [];
        for (var i = 0; i < ct.length; i++) {
          if (ct[i].checked) { bat.push(ct[i].getAttribute('data-hanh-dong')); }
        }
        // Ô nhập: gửi kèm NHÃN và nội dung. Nhãn là thứ khung dùng để biết ô
        // nào là ô nào — mã hành động thì ô nhập không có.
        var o = document.querySelectorAll('input[aria-label]');
        var oo = [];
        for (var j = 0; j < o.length; j++) {
          oo.push([o[j].getAttribute('aria-label'), o[j].value]);
        }
        window.ipc.postMessage(JSON.stringify({ a: a, bat: bat, o: oo }));
        return;
      }
    }
    n = n.parentElement;
  }
});
";

/// Kịch bản nối sự kiện cho **MÀN HÌNH ỨNG DỤNG** — KHÔNG đọc ô nhập.
const KICH_BAN_NOI_SU_KIEN: &str = r"
document.addEventListener('click', function (e) {
  var n = e.target;
  while (n) {
    if (n.getAttribute) {
      var a = n.getAttribute('data-hanh-dong');
      if (a) {
        // Công tắc KHÔNG gửi tin: nó chỉ đổi trạng thái tại chỗ. Người dùng còn
        // đang cân nhắc; chỉ khi bấm nút xác nhận mới chốt.
        if (n.getAttribute('role') === 'switch') { return; }
        // Gom trạng thái MỌI công tắc tại thời điểm bấm. Gửi kèm chứ không gửi
        // dần: người dùng bật rồi tắt lại thì cái được tính là trạng thái cuối.
        var ct = document.querySelectorAll('[role=switch][data-hanh-dong]');
        var bat = [];
        for (var i = 0; i < ct.length; i++) {
          if (ct[i].checked) { bat.push(ct[i].getAttribute('data-hanh-dong')); }
        }
        window.ipc.postMessage(JSON.stringify({ a: a, bat: bat }));
        return;
      }
    }
    n = n.parentElement;
  }
});
";

/// Người dùng đã chốt cái gì.
#[derive(Clone, PartialEq, Eq)]
pub struct DialogAnswer {
    /// Nút đã bấm.
    pub hanh_dong: String,
    /// Mã của các công tắc đang BẬT lúc bấm. Công tắc không có ở đây là tắt.
    pub bat: Vec<String>,
    /// Nội dung các ô nhập, theo `(nhãn, giá trị)`.
    ///
    /// **Chỉ có khi màn hình do KHUNG dựng.** Màn hình ứng dụng luôn để rỗng —
    /// xem [`KICH_BAN_KHUNG`].
    ///
    /// Trường này có thể chứa **mã PIN**. Đừng ghi nó ra nhật ký; `Debug` của
    /// kiểu này đã giấu sẵn giá trị.
    pub o_nhap: Vec<(String, String)>,
}

/// `Debug` che GIÁ TRỊ ô nhập, chỉ để lộ nhãn.
///
/// Một `{:?}` trên cấu trúc này rất dễ rơi vào nhật ký lỗi, và ở màn nhập ví
/// thì giá trị ấy là mã PIN của người dùng.
impl core::fmt::Debug for DialogAnswer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DialogAnswer")
            .field("hanh_dong", &self.hanh_dong)
            .field("bat", &self.bat)
            .field(
                "o_nhap",
                &self
                    .o_nhap
                    .iter()
                    .map(|(nhan, gia_tri)| format!("{nhan}: <{} ký tự>", gia_tri.chars().count()))
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

/// Một màn hình trong chuỗi: tài liệu, tiêu đề, và danh sách trắng của nó.
pub struct Screen {
    pub document: String,
    pub title: String,
    pub allowed: Vec<String>,
}

/// Sau khi người dùng bấm: hiện màn tiếp, hay đóng lại.
pub enum Next {
    Show(Box<Screen>),
    Done,
}

/// Nhiều màn hình nối nhau **trong MỘT cửa sổ và MỘT vòng lặp sự kiện**.
///
/// # ⚠️ Vì sao không gọi [`ask_dialog`] ba lần
///
/// `tao` chỉ cho dựng **một** vòng lặp sự kiện mỗi tiến trình. Gọi `ask_dialog`
/// lần thứ hai làm nó hoảng loạn ở `app_state.rs`, và thông báo hoảng loạn ấy
/// không nói gì về nguyên nhân — nhìn từ ngoài chỉ thấy chương trình treo.
///
/// Phát hiện ngày 17/08/2026, khi luồng nhập ví có ba màn hình. Đường hộp thoại
/// hỏi quyền không lộ ra điều này vì nó chỉ mở **một** cửa sổ mỗi lần chạy.
///
/// Nên ở đây: một cửa sổ, một `WebView`, và mỗi màn hình là một lần
/// `load_html`. Danh sách trắng đổi theo từng màn — mã của màn trước không
/// dùng lại được ở màn sau.
///
/// # Errors
/// Không dựng được cửa sổ hoặc `WebView`.
pub fn dialog_sequence(
    dau: &Screen,
    tiep: impl FnMut(&DialogAnswer) -> Next + 'static,
) -> Result<(), String> {
    dialog_sequence_driven(dau, tiep, None)
}

/// Như [`dialog_sequence`] nhưng chạy thêm một kịch bản tự thao tác.
///
/// Chỉ dùng để KIỂM THỬ. Nó không làm được gì mà một người dùng thật không làm
/// được — nó chỉ thay ngón tay, và nó chạy lại sau MỖI lần đổi màn hình, đúng
/// như người dùng thật gõ tiếp trên màn mới.
///
/// # Errors
/// Không dựng được cửa sổ hoặc `WebView`.
pub fn dialog_sequence_driven(
    dau: &Screen,
    mut tiep: impl FnMut(&DialogAnswer) -> Next + 'static,
    tu_lai: Option<&str>,
) -> Result<(), String> {
    let mut vong = EventLoopBuilder::new().build();
    let window = WindowBuilder::new()
        .with_title(&dau.title)
        .with_inner_size(tao::dpi::LogicalSize::new(620.0, 660.0))
        .build(&vong)
        .map_err(|e| format!("không dựng được cửa sổ: {e}"))?;

    let hang: Arc<Mutex<Vec<DialogAnswer>>> = Arc::new(Mutex::new(Vec::new()));
    let hang_ipc = Arc::clone(&hang);
    // Danh sách trắng ĐỔI theo màn. Giữ trong `Mutex` chứ không chép vào bộ xử
    // lý: chép thì màn sau vẫn nhận mã của màn trước.
    let hop_le: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(dau.allowed.clone()));
    let hop_le_ipc = Arc::clone(&hop_le);

    let mut dung = WebViewBuilder::new()
        .with_html(&dau.document)
        .with_initialization_script(KICH_BAN_KHUNG);
    if let Some(kb) = tu_lai {
        dung = dung.with_initialization_script(kb);
    }
    let webview = dung
        .with_ipc_handler(move |yeu_cau| {
            let danh_sach = hop_le_ipc.lock().map(|d| d.clone()).unwrap_or_default();
            if let Some(t) = doc_tra_loi(yeu_cau.body(), &danh_sach)
                && let Ok(mut q) = hang_ipc.lock()
            {
                q.push(t);
            }
        })
        .build_filling(&window)
        .map_err(|e| format!("không dựng được WebView: {e}"))?;

    let mut dang_thoat = false;
    vong.run_return(|su_kien, _, dieu_khien| {
        if dang_thoat {
            *dieu_khien = ControlFlow::Exit;
            return;
        }
        *dieu_khien = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(50));

        if matches!(
            su_kien,
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
        ) {
            dang_thoat = true;
            *dieu_khien = ControlFlow::Exit;
            return;
        }

        let cho_xu_ly: Vec<DialogAnswer> = hang
            .lock()
            .map(|mut q| q.drain(..).collect())
            .unwrap_or_default();
        for t in cho_xu_ly {
            match tiep(&t) {
                Next::Show(man) => {
                    if let Ok(mut d) = hop_le.lock() {
                        d.clone_from(&man.allowed);
                    }
                    window.set_title(&man.title);
                    if webview.load_html(&man.document).is_err() {
                        dang_thoat = true;
                        *dieu_khien = ControlFlow::Exit;
                        return;
                    }
                }
                Next::Done => {
                    dang_thoat = true;
                    *dieu_khien = ControlFlow::Exit;
                    return;
                }
            }
        }
    });
    Ok(())
}

/// Hiện tài liệu và CHỜ người dùng kích hoạt một hành động.
///
/// Trả `None` khi người dùng đóng cửa sổ mà không bấm gì.
///
/// # ⚠️ Đóng cửa sổ KHÔNG phải là đồng ý
///
/// Chỗ gọi phải coi `None` là TỪ CHỐI. Mặc định của một câu hỏi chưa trả lời
/// luôn phải là "không" — thiết kế nào biến im lặng thành đồng ý là thiết kế
/// lấy được quyền của người dùng bằng cách làm họ mệt.
///
/// # Errors
/// Không dựng được cửa sổ hoặc WebView.
pub fn ask_dialog(
    document: &str,
    tieu_de: &str,
    hanh_dong_hop_le: &[String],
) -> Result<Option<DialogAnswer>, String> {
    let mut vong = EventLoopBuilder::new().build();
    let window = WindowBuilder::new()
        .with_title(tieu_de)
        .with_inner_size(tao::dpi::LogicalSize::new(560.0, 640.0))
        .build(&vong)
        .map_err(|e| format!("không dựng được cửa sổ: {e}"))?;

    let hop: Arc<Mutex<Option<DialogAnswer>>> = Arc::new(Mutex::new(None));
    let hop_ipc = Arc::clone(&hop);
    // DANH SÁCH TRẮNG: chỉ nhận mã thật sự có trên màn hình — áp cho CẢ nút bấm
    // lẫn công tắc. Một thông điệp mang mã lạ bị vứt lặng lẽ.
    let hop_le: Vec<String> = hanh_dong_hop_le.to_vec();

    let _webview = WebViewBuilder::new()
        .with_html(document)
        // Màn hình của KHUNG: đọc cả nội dung ô nhập — xem `KICH_BAN_KHUNG`.
        .with_initialization_script(KICH_BAN_KHUNG)
        .with_ipc_handler(move |yeu_cau| {
            let Some(t) = doc_tra_loi(yeu_cau.body(), &hop_le) else {
                // Vứt LẶNG LẼ. Báo ngược ra trang là nói cho kẻ tấn công biết
                // nó vừa thử trúng cái gì.
                return;
            };
            // QUYẾT ĐỊNH ĐẦU TIÊN THẮNG. Thông điệp sau không ghi đè được — một
            // quyết định đã chốt thì không ai được sửa, kể cả chính trang đó.
            if let Ok(mut o) = hop_ipc.lock()
                && o.is_none()
            {
                *o = Some(t);
            }
        })
        .build_filling(&window)
        .map_err(|e| format!("không dựng được WebView: {e}"))?;

    let mut dang_thoat = false;
    vong.run_return(|su_kien, _, dieu_khien| {
        if dang_thoat {
            *dieu_khien = ControlFlow::Exit;
            return;
        }
        *dieu_khien = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(50));

        let da_bam = hop.lock().is_ok_and(|o| o.is_some());
        let dong = matches!(
            su_kien,
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
        );
        if da_bam || dong {
            dang_thoat = true;
            *dieu_khien = ControlFlow::Exit;
        }
    });

    Ok(hop.lock().map_err(|_| "khoá hỏng".to_owned())?.clone())
}

/// Cách gửi hành động trong phép kiểm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    /// Bấm THẬT lên phần tử — đi qua kịch bản nối sự kiện.
    Bam,
    /// Bật một CÔNG TẮC trước, rồi mới bấm nút xác nhận.
    ///
    /// Đây là mắt xích mà chỉ WebKit thật kiểm được: công tắc đổi trạng thái tại
    /// chỗ, và trạng thái đó phải đi kèm cú bấm nút. Đứt ở đây thì người dùng bật
    /// công tắc, bấm cho phép, và **không quyền nào được cấp** — mà cũng không có
    /// lỗi nào hiện ra.
    BatRoiBam {
        /// Mã công tắc cần bật.
        cong_tac: &'static str,
    },
    /// Gửi một thông điệp có hành động HỢP LỆ nhưng kèm một CÔNG TẮC MA.
    ///
    /// Đòn này khác `GuiThang`: hành động qua được danh sách trắng, chỉ danh
    /// sách công tắc là bịa. Nếu bộ lọc chỉ kiểm hành động mà lọc bớt công tắc
    /// lạ cho qua, thì một quyền được cấp mà người dùng chưa hề bật.
    CongTacMa { cong_tac: &'static str },
    /// Gửi thẳng một mã bịa ra, bỏ qua mọi nút trên màn hình.
    ///
    /// Dùng để kiểm DANH SÁCH TRẮNG. Không có nhánh này thì gỡ bỏ danh sách
    /// trắng đi mà mọi phép thử vẫn xanh — nới lỏng một bộ lọc là loại đột biến
    /// mà phép thử chỉ-gửi-dữ-liệu-hợp-lệ không bao giờ bắt được.
    GuiThang,
}

/// Cửa sổ có nên đóng lại sau khi xử lý xong một cú bấm không.
// `Clone` chứ không `Copy`: một nhánh mang theo câu chữ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlFlowNext {
    /// Giữ cửa sổ, chờ cú bấm tiếp theo.
    Giu,
    /// Giữ cửa sổ và **BÁO một câu lên tiêu đề**.
    ///
    /// # Vì sao tiêu đề, chứ không phải một dòng trong tài liệu
    ///
    /// Tiêu đề là của KHUNG. Ứng dụng không viết được vào đó — nó không mang mã
    /// nào, và `app_window_title` luôn đặt mã ứng dụng đã ký lên trước. Một câu
    /// vẽ trong tài liệu thì ứng dụng bắt chước được, và lúc ấy người dùng không
    /// phân biệt được câu của khung với câu của gói.
    ///
    /// Đây là chỗ tối thiểu để người dùng THẤY rằng có việc vừa xảy ra. Một dải
    /// trạng thái riêng của khung — như thanh địa chỉ ở tầng 2 — mới là câu trả
    /// lời đầy đủ, và nó thuộc về lần thiết kế sau.
    GiuVaBao(String),
    /// Đóng cửa sổ.
    Dong,
}

/// Hiện tài liệu và xử lý NHIỀU cú bấm cho tới khi đóng.
///
/// Khác [`ask_dialog`] ở chỗ [`ask_dialog`] trả về sau cú bấm đầu tiên — đúng cho hộp thoại
/// hỏi quyền, vì ở đó cú bấm đầu tiên LÀ quyết định. Màn hình ứng dụng thì
/// ngược lại: người dùng bấm nhiều lần.
///
/// Một vòng lặp sự kiện duy nhất, vì trên macOS một tiến trình chỉ dựng được
/// một vòng. Gọi [`ask_dialog`] nhiều lần liên tiếp là sai ngay từ thiết kế.
///
/// # Errors
/// Không dựng được cửa sổ hoặc WebView.
pub fn run_loop(
    document: &str,
    tieu_de: &str,
    hanh_dong_hop_le: &[String],
    doc_tep: impl Fn(&str) -> Option<Vec<u8>> + 'static,
    mut xu_ly: impl FnMut(&DialogAnswer) -> ControlFlowNext,
) -> Result<(), String> {
    let mut vong = EventLoopBuilder::new().build();
    let window = WindowBuilder::new()
        .with_title(tieu_de)
        .with_inner_size(tao::dpi::LogicalSize::new(760.0, 620.0))
        .build(&vong)
        .map_err(|e| format!("không dựng được cửa sổ: {e}"))?;

    // HÀNG ĐỢI, không phải một ô nhớ. `ask_dialog` giữ đúng một quyết định rồi thôi;
    // ở đây bấm nhanh hai lần thì cú thứ hai không được phép mất.
    let hang: Arc<Mutex<Vec<DialogAnswer>>> = Arc::new(Mutex::new(Vec::new()));
    let hang_ipc = Arc::clone(&hang);
    let hop_le: Vec<String> = hanh_dong_hop_le.to_vec();

    let _webview = WebViewBuilder::new()
        .with_html(document)
        .with_initialization_script(KICH_BAN_NOI_SU_KIEN)
        .with_custom_protocol(package_server::SCHEME.to_owned(), move |_id, yc| {
            serve(&doc_tep, &yc)
        })
        .with_ipc_handler(move |yeu_cau| {
            if let Some(t) = doc_tra_loi(yeu_cau.body(), &hop_le)
                && let Ok(mut q) = hang_ipc.lock()
            {
                q.push(t);
            }
        })
        .build_filling(&window)
        .map_err(|e| format!("không dựng được WebView: {e}"))?;

    let mut dang_thoat = false;
    vong.run_return(|su_kien, _, dieu_khien| {
        if dang_thoat {
            *dieu_khien = ControlFlow::Exit;
            return;
        }
        *dieu_khien = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(50));

        if matches!(
            su_kien,
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
        ) {
            dang_thoat = true;
            *dieu_khien = ControlFlow::Exit;
            return;
        }

        // Rút hết hàng đợi rồi mới xử lý: giữ khoá trong lúc gọi `xu_ly` là mời
        // gọi kẹt khoá, vì `xu_ly` có thể chạy lâu (gọi mạng chẳng hạn).
        let cho_xu_ly: Vec<DialogAnswer> = hang
            .lock()
            .map(|mut q| std::mem::take(&mut *q))
            .unwrap_or_default();
        for t in &cho_xu_ly {
            match xu_ly(t) {
                ControlFlowNext::Dong => {
                    dang_thoat = true;
                    *dieu_khien = ControlFlow::Exit;
                    return;
                }
                // ⚠️ Người dùng PHẢI thấy có việc vừa xảy ra.
                //
                // Bản trước chỉ `println!` — và người dùng bấm "Tải trang mẫu"
                // **13 lần** vì màn hình không đổi gì, trong khi cả 13 lần đều
                // tải về thành công. Đây là lần thứ BA dự án dẫm đúng bẫy này:
                // nhánh kết thúc của một luồng mà chỉ ra `stderr` thì với người
                // dùng nó không xảy ra.
                ControlFlowNext::GiuVaBao(cau) => {
                    window.set_title(&format!("{tieu_de} — {cau}"));
                }
                ControlFlowNext::Giu => {}
            }
        }
    });
    Ok(())
}

/// Đọc và LỌC thông điệp từ trang.
///
/// Trả `None` với mọi thứ không rõ ràng: JSON hỏng, hành động không có trên màn
/// hình, hoặc **bất kỳ công tắc nào** không có trên màn hình. Không lọc bớt cho
/// qua — một công tắc ma lọt vào danh sách bật nghĩa là quyền được cấp mà người
/// dùng chưa hề bật.
fn doc_tra_loi(body: &str, hop_le: &[String]) -> Option<DialogAnswer> {
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    let hanh_dong = v["a"].as_str()?.to_owned();
    if !hop_le.contains(&hanh_dong) {
        return None;
    }
    let mut bat = Vec::new();
    for x in v["bat"].as_array()? {
        let s = x.as_str()?.to_owned();
        if !hop_le.contains(&s) {
            // Cả thông điệp bị vứt, không phải chỉ mục lạ. Một thông điệp đã
            // pha tạp thì không phần nào của nó đáng tin.
            return None;
        }
        bat.push(s);
    }
    // Ô nhập chỉ có ở màn hình của khung; thiếu trường `o` là chuyện bình
    // thường, không phải lỗi.
    let mut o_nhap = Vec::new();
    if let Some(ds) = v["o"].as_array() {
        for x in ds {
            let cap = x.as_array()?;
            let nhan = cap.first()?.as_str()?.to_owned();
            let gia_tri = cap.get(1)?.as_str()?.to_owned();
            o_nhap.push((nhan, gia_tri));
        }
    }
    Some(DialogAnswer {
        hanh_dong,
        bat,
        o_nhap,
    })
}

/// Chạy MỘT lần: nạp tài liệu, chạy một kịch bản tự thao tác, chờ một thông điệp.
///
/// Dùng chung cho [`simulate_click`] và [`simulate_typing`]. Viết hai bản là để
/// chúng trôi dạt khỏi nhau, và lúc đó một bên xanh trong khi đường thật đã hỏng.
fn chay_mot_lan(
    document: &str,
    hanh_dong_hop_le: &[String],
    kich_ban: &str,
    cho_toi_da: Duration,
) -> Result<Option<DialogAnswer>, String> {
    let mut vong = EventLoopBuilder::new().build();
    let window = WindowBuilder::new()
        .with_visible(false)
        .build(&vong)
        .map_err(|e| format!("không dựng được cửa sổ: {e}"))?;

    let hop: Arc<Mutex<Option<DialogAnswer>>> = Arc::new(Mutex::new(None));
    let hop_ipc = Arc::clone(&hop);
    let hop_le: Vec<String> = hanh_dong_hop_le.to_vec();

    let _webview = WebViewBuilder::new()
        .with_html(document)
        // Kịch bản của KHUNG: phép kiểm cú bấm phải đi đúng đường mà hộp thoại
        // thật đi, kể cả phần đọc ô nhập.
        .with_initialization_script(KICH_BAN_KHUNG)
        .with_initialization_script(kich_ban)
        .with_ipc_handler(move |yeu_cau| {
            let body = yeu_cau.body();
            // Dùng ĐÚNG hàm lọc của `ask_dialog`. Viết một hàm lọc riêng cho phép thử
            // là để hai bên trôi dạt khỏi nhau, và lúc đó phép thử xanh trong
            // khi đường thật đã hỏng.
            let t = if body == "KHONG-TIM-THAY-NUT" {
                Some(DialogAnswer {
                    hanh_dong: body.clone(),
                    bat: Vec::new(),
                    o_nhap: Vec::new(),
                })
            } else {
                doc_tra_loi(body, &hop_le)
            };
            // Không có nhánh `else`: mã lạ bị vứt LẶNG LẼ, cố ý.
            // QUYẾT ĐỊNH ĐẦU TIÊN THẮNG. Thông điệp sau không ghi đè được.
            //
            // Ghi đè thì một trang gửi liên tiếp hai thông điệp sẽ khiến cái sau
            // che mất cái trước — và với `ask_dialog` thì cái trước mới là quyết định
            // thật của người dùng. Nó cũng che luôn lỗi: một đột biến làm công
            // tắc tự gửi tin đã lọt qua phép thử đúng vì lý do này.
            if let Some(t) = t
                && let Ok(mut o) = hop_ipc.lock()
                && o.is_none()
            {
                *o = Some(t);
            }
        })
        .build_filling(&window)
        .map_err(|e| format!("không dựng được WebView: {e}"))?;

    let han = Instant::now() + cho_toi_da;
    vong.run_return(|_su_kien, _, dieu_khien| {
        *dieu_khien = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(50));
        if hop.lock().is_ok_and(|o| o.is_some()) || Instant::now() >= han {
            *dieu_khien = ControlFlow::Exit;
        }
    });

    Ok(hop.lock().map_err(|_| "khoá hỏng".to_owned())?.clone())
}

/// Kiểm mắt xích **GÕ CHỮ**: gõ hộ vào một ô nhập rồi bấm, xem chữ có về không.
///
/// # Vì sao cần một hàm riêng
///
/// [`simulate_click`] chỉ bấm. Đường "người dùng gõ → WebKit → IPC → Rust" là
/// một đường dữ liệu KHÁC, và nó vừa được mở (17/08/2026) để màn nhập ví nhận
/// được mã PIN. Đường mới mà không có phép kiểm đi hết thì hỏng lúc nào không
/// ai biết — vẽ ra ô nhập vẫn đẹp, chỉ là không ai nhận thứ gõ vào.
///
/// Hàm này chỉ dùng để kiểm thử, và nó **không làm được gì mà một người gõ
/// thật không làm được** — nó chỉ thay ngón tay.
///
/// # Errors
/// Không dựng được cửa sổ, hoặc hết giờ chờ.
pub fn simulate_typing(
    document: &str,
    hanh_dong_hop_le: &[String],
    nhan_o: &str,
    chu: &str,
    can_bam: &str,
    cho_toi_da: Duration,
) -> Result<Option<DialogAnswer>, String> {
    // Đặt giá trị rồi phát `input` như bộ gõ thật, sau đó mới bấm. Đặt giá trị
    // mà không phát sự kiện là bỏ qua đúng đoạn mà trang thật chạy.
    let tu_go = format!(
        r"
document.addEventListener('DOMContentLoaded', function () {{
  var o = document.querySelector('input[aria-label={nhan}]');
  if (o) {{
    o.value = {chu};
    o.dispatchEvent(new Event('input', {{ bubbles: true }}));
  }}
  var ds = document.querySelectorAll('[data-hanh-dong]');
  for (var i = 0; i < ds.length; i++) {{
    if (ds[i].getAttribute('data-hanh-dong') === {nut}) {{ ds[i].click(); return; }}
  }}
  window.ipc.postMessage('KHONG-TIM-THAY-NUT');
}});
",
        nhan = serde_json::Value::from(nhan_o),
        chu = serde_json::Value::from(chu),
        nut = serde_json::Value::from(can_bam),
    );
    chay_mot_lan(document, hanh_dong_hop_le, &tu_go, cho_toi_da)
}

/// Kiểm mắt xích CÚ BẤM: tự bấm hộ một nút rồi xem nhận về mã nào.
///
/// # Vì sao cần
///
/// Phần còn lại của đường ống kiểm được hết bằng hàm thuần. Riêng đoạn
/// "người dùng bấm → kịch bản nối sự kiện → IPC → danh sách trắng" thì không:
/// nó chỉ chạy khi có WebKit thật và có một cú bấm thật. Không kiểm thì nút bấm
/// vào không ăn mà **chẳng có lỗi nào hiện ra** — về mặt kỹ thuật không có gì
/// hỏng cả.
///
/// Hàm này chỉ dùng để kiểm thử. Nó KHÔNG mở cửa sổ cho người xem, và nó không
/// làm được gì mà một cú bấm thật không làm được — nó chỉ thay ngón tay.
///
/// # Errors
/// Không dựng được cửa sổ, hoặc không nhận về gì trong thời gian chờ.
pub fn simulate_click(
    document: &str,
    hanh_dong_hop_le: &[String],
    can_bam: &str,
    kieu: MessageKind,
    cho_toi_da: Duration,
) -> Result<Option<DialogAnswer>, String> {
    // Bấm bằng `click()` thật trên phần tử, KHÔNG gọi thẳng `postMessage`. Gọi
    // thẳng thì ta chỉ kiểm được IPC còn kịch bản nối sự kiện thì không — mà
    // kịch bản nối sự kiện mới đúng là thứ dễ hỏng.
    let tu_bam = match kieu {
        MessageKind::CongTacMa { cong_tac } => format!(
            "document.addEventListener('DOMContentLoaded', function () {{
               window.ipc.postMessage(JSON.stringify({{
                 a: {can_bam:?}, bat: [{cong_tac:?}]
               }}));
             }});"
        ),
        MessageKind::BatRoiBam { cong_tac } => format!(
            "document.addEventListener('DOMContentLoaded', function () {{
               var ct = document.querySelector('[data-hanh-dong={cong_tac:?}]');
               if (!ct) {{ window.ipc.postMessage('KHONG-TIM-THAY-NUT'); return; }}
               ct.click();
               var nut = document.querySelector('[data-hanh-dong={can_bam:?}]');
               if (!nut) {{ window.ipc.postMessage('KHONG-TIM-THAY-NUT'); return; }}
               nut.click();
             }});"
        ),
        MessageKind::Bam => format!(
            "document.addEventListener('DOMContentLoaded', function () {{
               var n = document.querySelector('[data-hanh-dong={can_bam:?}]');
               if (n) {{ n.click(); }}
               else {{ window.ipc.postMessage('KHONG-TIM-THAY-NUT'); }}
             }});"
        ),
        // Giả mạo: gửi thẳng một mã KHÔNG có nút nào mang nó. Đây là đòn mà
        // danh sách trắng sinh ra để chặn.
        // Gửi thẳng một thông điệp ĐÚNG ĐỊNH DẠNG nhưng mang mã bịa. Sai định
        // dạng thì nó bị vứt vì lý do khác, và phép thử không chứng minh được
        // danh sách trắng có làm việc hay không.
        MessageKind::GuiThang => format!(
            "document.addEventListener('DOMContentLoaded', function () {{
               window.ipc.postMessage(JSON.stringify({{ a: {can_bam:?}, bat: [] }}));
             }});"
        ),
    };

    chay_mot_lan(document, hanh_dong_hop_le, &tu_bam, cho_toi_da)
}

/// Trả lời một yêu cầu `tcc-goi:` từ WebView.
///
/// Từ chối thì trả **404 rỗng**, không trả thông báo lỗi: thông báo chi tiết là
/// nói cho trang biết cái gì có và cái gì không có trong gói, mà trang thì có
/// thể đang bị chiếm quyền.
fn serve(
    doc_tep: &dyn Fn(&str) -> Option<Vec<u8>>,
    yc: &wry::http::Request<Vec<u8>>,
) -> wry::http::Response<std::borrow::Cow<'static, [u8]>> {
    use std::borrow::Cow;
    match package_server::serve(&yc.uri().to_string(), doc_tep) {
        Ok((kieu, byte)) => wry::http::Response::builder()
            .status(200)
            .header("Content-Type", kieu)
            // Không đoán kiểu: ta đã chọn kiểu từ danh sách trắng, đừng để
            // trình duyệt tự suy ra một kiểu khác.
            .header("X-Content-Type-Options", "nosniff")
            .body(Cow::Owned(byte))
            .unwrap_or_else(|_| wry::http::Response::new(Cow::Borrowed(&[][..]))),
        Err(e) => {
            eprintln!("[bộ dựng] từ chối {}: {e}", yc.uri());
            wry::http::Response::builder()
                .status(404)
                .body(Cow::Borrowed(&[][..]))
                .unwrap_or_else(|_| wry::http::Response::new(Cow::Borrowed(&[][..])))
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay"
)]
mod kiem_thu_o_nhap {
    use super::*;

    /// **Kịch bản của ỨNG DỤNG không được đọc ô nhập.**
    ///
    /// Ứng dụng TCC không mang mã, nên không có ai bên trong nhận giá trị ấy.
    /// Thu thập thứ không ai cần là mở rộng bề mặt mà không đổi lấy gì.
    #[test]
    fn kich_ban_ung_dung_khong_doc_o_nhap() {
        assert!(
            !KICH_BAN_NOI_SU_KIEN.contains(".value"),
            "kịch bản của ứng dụng đang đọc nội dung ô nhập"
        );
        assert!(
            !KICH_BAN_NOI_SU_KIEN.contains("aria-label"),
            "kịch bản của ứng dụng đang quét ô nhập theo nhãn"
        );
        // Còn kịch bản của khung thì PHẢI đọc — nếu không, ô PIN không ai nhận.
        assert!(KICH_BAN_KHUNG.contains(".value"));
    }

    /// Thiếu trường `o` là bình thường — thông điệp của ứng dụng không có nó.
    #[test]
    fn thieu_truong_o_van_doc_duoc() {
        let t = doc_tra_loi(r#"{"a":"ok","bat":[]}"#, &["ok".to_owned()]).unwrap();
        assert!(t.o_nhap.is_empty());
    }

    /// Ô nhập đọc được đúng nhãn và giá trị.
    #[test]
    fn doc_duoc_nhan_va_gia_tri() {
        let t = doc_tra_loi(
            r#"{"a":"ok","bat":[],"o":[["Mã PIN","1234"],["Tìm","xin chào"]]}"#,
            &["ok".to_owned()],
        )
        .unwrap();
        assert_eq!(
            t.o_nhap,
            vec![
                ("Mã PIN".to_owned(), "1234".to_owned()),
                ("Tìm".to_owned(), "xin chào".to_owned())
            ]
        );
    }

    /// **`Debug` KHÔNG được in giá trị ô nhập** — ở màn nhập ví đó là mã PIN.
    #[test]
    fn debug_khong_in_gia_tri_o_nhap() {
        let t = DialogAnswer {
            hanh_dong: "ok".to_owned(),
            bat: Vec::new(),
            o_nhap: vec![("Mã PIN".to_owned(), "bi-mat-that".to_owned())],
        };
        let s = format!("{t:?}");
        assert!(
            !s.contains("bi-mat-that"),
            "giá trị ô nhập lọt vào Debug: {s}"
        );
        assert!(
            s.contains("Mã PIN"),
            "mất cả nhãn thì không gỡ lỗi được: {s}"
        );
        assert!(s.contains("11 ký tự"), "{s}");
    }

    /// Thông điệp pha tạp bị vứt CẢ, không phải chỉ phần hỏng.
    #[test]
    fn o_nhap_hong_thi_vut_ca_thong_diep() {
        assert!(
            doc_tra_loi(
                r#"{"a":"ok","bat":[],"o":[["chỉ-một"]]}"#,
                &["ok".to_owned()]
            )
            .is_none()
        );
        assert!(doc_tra_loi(r#"{"a":"ok","bat":[],"o":[[1,2]]}"#, &["ok".to_owned()]).is_none());
    }
}
