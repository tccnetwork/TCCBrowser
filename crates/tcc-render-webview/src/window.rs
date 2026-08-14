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
        .build(&window)?;

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
        .build(&window)
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
        .build(&window)
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogAnswer {
    /// Nút đã bấm.
    pub hanh_dong: String,
    /// Mã của các công tắc đang BẬT lúc bấm. Công tắc không có ở đây là tắt.
    pub bat: Vec<String>,
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
        .with_initialization_script(KICH_BAN_NOI_SU_KIEN)
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
        .build(&window)
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlFlowNext {
    /// Giữ cửa sổ, chờ cú bấm tiếp theo.
    Giu,
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
        .build(&window)
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
            if xu_ly(t) == ControlFlowNext::Dong {
                dang_thoat = true;
                *dieu_khien = ControlFlow::Exit;
                return;
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
    Some(DialogAnswer { hanh_dong, bat })
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
    let mut vong = EventLoopBuilder::new().build();
    let window = WindowBuilder::new()
        .with_visible(false)
        .build(&vong)
        .map_err(|e| format!("không dựng được cửa sổ: {e}"))?;

    let hop: Arc<Mutex<Option<DialogAnswer>>> = Arc::new(Mutex::new(None));
    let hop_ipc = Arc::clone(&hop);
    let hop_le: Vec<String> = hanh_dong_hop_le.to_vec();

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

    let _webview = WebViewBuilder::new()
        .with_html(document)
        .with_initialization_script(KICH_BAN_NOI_SU_KIEN)
        .with_initialization_script(&tu_bam)
        .with_ipc_handler(move |yeu_cau| {
            let body = yeu_cau.body();
            // Dùng ĐÚNG hàm lọc của `ask_dialog`. Viết một hàm lọc riêng cho phép thử
            // là để hai bên trôi dạt khỏi nhau, và lúc đó phép thử xanh trong
            // khi đường thật đã hỏng.
            let t = if body == "KHONG-TIM-THAY-NUT" {
                Some(DialogAnswer {
                    hanh_dong: body.clone(),
                    bat: Vec::new(),
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
        .build(&window)
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
