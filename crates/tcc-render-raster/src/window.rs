//! Đưa đệm điểm ảnh lên **cửa sổ thật**, và nhận cú bấm.
//!
//! # Vì sao tệp này là cổng ra của giai đoạn 4
//!
//! `docs/ke-hoach.md`, cổng ra giai đoạn 4: *"ứng dụng mẫu chạy trên **cả hai**
//! bộ dựng, **không sửa một dòng nào**. Đó là lúc chứng minh được đường thoát là
//! thật."*
//!
//! Trước tệp này, `tcc-render-raster` vẽ ra được điểm ảnh nhưng **không ai nhìn
//! thấy chúng**: không cửa sổ, không cú bấm. Một bộ dựng không hiện ra được thì
//! chứng minh được `tcc-ui` không dính HTML, nhưng **không** chứng minh được nó
//! thay thế được WebView. Hai câu ấy khác nhau, và chỉ câu sau mới là đường
//! thoát.
//!
//! # Không một dòng HTML, và không một dòng WebView
//!
//! Cả tệp này chỉ dùng `tao` (cửa sổ) và `softbuffer` (đưa điểm ảnh lên màn
//! hình). Không `wry`, không WebKit, không máy dựng nào của hệ điều hành.

use std::collections::BTreeSet;
#[cfg(all(feature = "accesskit-platform", target_os = "macos"))]
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use softbuffer::{Context, Surface};
use tao::{
    dpi::LogicalSize,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    platform::run_return::EventLoopExtRunReturn as _,
    window::WindowBuilder,
};
use tcc_ui::{Node, Renderer as _};

use crate::{RasterRenderer, WIDTH};

/// Chuỗi bộ dựng cần, **tiêm từ ngoài vào**.
///
/// Cố ý **KHÔNG có `Default`**: mặc định là chỗ một câu tiếng Anh lọt vào màn
/// hình tiếng Việt mà không ai thấy. Đã xảy ra đúng một lần — xem `SECURITY.md`
/// §3.1c — và một kiểu dữ liệu không có mặc định thì không tái diễn được.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenText {
    /// Câu mô tả hành động không hoàn tác. Đọc SAU nhãn.
    pub destructive_note: String,
    /// Tên vai trò cho nút không hoàn tác. Đọc THAY chữ "nút".
    pub destructive_role: String,
}

/// Kết thúc của một màn hình raster.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScreenOutcome {
    /// Nút người dùng bấm. `None` = đóng cửa sổ mà không bấm nút nào.
    pub action: Option<String>,
    /// Các công tắc đang **BẬT** lúc màn hình kết thúc.
    ///
    /// Trả về kể cả khi `action` là `None`, và bên gọi phải bỏ chúng đi trong
    /// trường hợp ấy: đóng cửa sổ **không phải** là đồng ý. Nói ra ở đây vì kiểu
    /// dữ liệu không tự nói được — nó chỉ là một tập hợp.
    pub toggles_on: BTreeSet<String>,
    /// Nội dung các ô nhập lúc màn hình kết thúc, tra theo **nhãn**.
    ///
    /// ⚠️ Có thể chứa **bí mật** — cụm từ khôi phục, mã PIN. Bên gọi phải thả
    /// nó ngay sau khi dùng, và **không bao giờ** ghi ra nhật ký. Ô che chữ
    /// (`secret: true`) trả về **chữ thật**, không phải hàng chấm: hàng chấm là
    /// việc của lúc VẼ, còn đây là thứ người dùng đã gõ.
    ///
    /// Như `toggles_on`: đóng cửa sổ mà không bấm nút thì bên gọi phải BỎ nó
    /// đi. Đóng cửa sổ không phải là gửi đi.
    pub fields: std::collections::BTreeMap<String, String>,
}

/// Mở màn hình `tree` bằng bộ dựng ra pixel.
///
/// **Nút** kết thúc màn hình. **Công tắc** đổi câu trả lời rồi vẽ lại, ở lại.
///
/// `cau_mat_mat` là câu mô tả hành động **không hoàn tác**, tiêm từ ngoài vào.
///
/// ⚠️ Đừng để nó mặc định. Bản đầu tệp này gọi `AccessText::default()` — tức là
/// tiếng Anh — bất kể người dùng đang dùng ngôn ngữ nào, trong khi đường WebView
/// tiêm câu đã dịch. Người dùng VoiceOver tiếng Việt nghe một câu tiếng Anh, và
/// **hai bộ dựng nói hai câu khác nhau cho cùng một nút** — đúng thứ mà chú
/// thích đầu `accesskit_bridge.rs` đã cảnh báo.
///
/// # Errors
/// Cây không vẽ được, hoặc không dựng được cửa sổ.
// ⚠️ NỢ ĐÃ GHI TÊN: hàm này dài 125 dòng mã, quá trần 100.
//
// Nó là một MÁY TRẠNG THÁI: dựng cửa sổ, dựng bề mặt, nối trợ năng, rồi một
// vòng lặp sáu nhánh cùng đọc và cùng sửa một mớ trạng thái. Tách tiếp bằng
// cách moi từng nhánh ra hàm rời thì mỗi hàm phải nhận thêm bốn năm tham số —
// trải một máy trạng thái ra sáu hàm KHÔNG làm nó dễ đọc hơn, chỉ làm chỗ nối
// khó thấy hơn.
//
// Lời giải đúng là gói cả phiên vào một struct có phương thức cho từng sự kiện.
// Chưa làm vì tệp này sắp phải sửa lớn, và tôi không muốn refactor hai lần.
// Ghi ra đây để nó là nợ có tên chứ không phải một `expect` lặng lẽ.
#[expect(
    clippy::too_many_lines,
    reason = "máy trạng thái của một phiên màn hình; nợ đã ghi ở chú thích trên"
)]
pub fn open_screen(tree: &Node, tieu_de: &str, chu: &ScreenText) -> Result<ScreenOutcome, String> {
    // `chu` chỉ tới tay trợ năng. Giữ tham số cho MỌI cờ chứ không đổi chữ ký
    // theo cờ: một hàm đổi hình dạng theo cờ là một hàm bên gọi phải nhớ hai
    // dạng, và sẽ có một dạng bị quên.
    #[cfg(not(all(feature = "accesskit-platform", target_os = "macos")))]
    let _ = chu;
    // Trạng thái công tắc do KHUNG giữ, không do cây giữ.
    //
    // Bộ dựng WebView để trình duyệt giữ hộ trong tài liệu rồi hỏi lại lúc bấm
    // xác nhận. Ở đây không có ai giữ hộ. Bắt đầu bằng tập RỖNG chứ không đọc
    // trạng thái ban đầu của cây: mặc định của một câu hỏi chưa trả lời là
    // "không", và một hộp thoại quyền mở ra với sẵn vài mục bật là một hộp thoại
    // đã tự trả lời hộ người dùng.
    let mut bat: BTreeSet<String> = BTreeSet::new();
    let mut bo_dung = RasterRenderer::new();
    bo_dung
        .render(&tree.with_toggles(&bat).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let cao = bo_dung.height();

    let mut vong = EventLoopBuilder::new().build();
    let window = dung_cua_so(&vong, tieu_de, cao)?;

    // ⚠️ `Context`/`Surface` phải sống lâu bằng cửa sổ. Thả sớm là mất bề mặt,
    // và triệu chứng là một cửa sổ trắng trơn — trông y như bộ dựng vẽ hỏng.
    let ngu_canh = Context::new(&window).map_err(|e| format!("không mở được bề mặt: {e}"))?;
    let mut be_mat =
        Surface::new(&ngu_canh, &window).map_err(|e| format!("không mở được bề mặt: {e}"))?;

    // ⚠️ Nối trợ năng TRƯỚC khi cửa sổ được hiện hay nhận tiêu điểm lần đầu —
    // `SubclassingAdapter::new` nói rõ điều đó. Nối muộn thì VoiceOver đã hỏi
    // xong và nhận câu "không có gì ở đây", rồi không hỏi lại.
    // Hàng đợi yêu cầu bấm từ trợ năng, và bảng tra `NodeId` → hành động.
    #[cfg(all(feature = "accesskit-platform", target_os = "macos"))]
    let hang_tro_nang: Arc<Mutex<Vec<u64>>> = Arc::new(Mutex::new(Vec::new()));
    #[cfg(all(feature = "accesskit-platform", target_os = "macos"))]
    let mut bang_hanh_dong = bang_hanh_dong_cua(&bo_dung);
    #[cfg(all(feature = "accesskit-platform", target_os = "macos"))]
    let cay_chung: Arc<Mutex<accesskit::TreeUpdate>> = Arc::new(Mutex::new(
        cay_accesskit(&bo_dung, chu).unwrap_or_else(cay_rong),
    ));
    #[cfg(all(feature = "accesskit-platform", target_os = "macos"))]
    let mut adapter = noi_tro_nang(&window, Arc::clone(&cay_chung), Arc::clone(&hang_tro_nang));

    // Nối trợ năng xong mới hiện — thứ tự này là cả nội dung của F6.
    window.set_visible(true);

    let mut tt = TrangThai::default();
    let mut da_bam: Option<String> = None;
    let mut ve_lai = false;
    // Cuộn, đơn vị LOGIC, tính từ đỉnh nội dung.
    //
    // ⚠️ Không có nó thì một cây cao hơn cửa sổ đẩy nút Cho phép / Từ chối ra
    // ngoài tầm, và người dùng **không bấm được gì cả**. Rà soát 21/08/2026, F7
    // — tôi xếp nó là "chuyện dùng được, không phải an ninh", rồi hôm sau nó
    // chặn đúng một người thật.

    // Nội dung ô nhập do KHUNG giữ, y như trạng thái công tắc — cây bất biến.

    // Ô đang được chọn. `None` = chưa chọn ô nào, và lúc ấy gõ phím KHÔNG đi
    // đâu cả — thà không nhận còn hơn nhận vào một ô người dùng không nhìn.

    // Vị trí chuột gần nhất, đơn vị LOGIC. `tao` báo vị trí theo pixel vật lý,
    // mà bộ dựng làm việc theo đơn vị logic — trên màn hình Retina hai thứ ấy
    // lệch nhau đúng hệ số 2, và bấm sẽ trúng ô khác.

    vong.run_return(|su_kien, _, dieu_khien| {
        // ⚠️ `WaitUntil`, KHÔNG phải `Wait` — đúng cái bẫy đã trả giá 18/08/2026
        // ở `web_tier.rs`, và nó cắn lần thứ hai ở đây.
        //
        // Yêu cầu bấm từ trợ năng đi vào hàng đợi qua `ActionHandler`, mà đẩy
        // vào hàng đợi **không sinh ra một sự kiện cửa sổ nào**. Với `Wait` thì
        // vòng lặp ngủ tiếp: người dùng VoiceOver bấm và không có gì xảy ra,
        // cho tới khi có ai đó rê chuột qua cửa sổ — tức là cho tới khi có một
        // người sáng mắt ngồi cạnh.
        *dieu_khien = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(50));
        match su_kien {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *dieu_khien = ControlFlow::Exit,

            // ⚠️ Chữ đến từ BỘ GÕ CỦA HỆ ĐIỀU HÀNH, không phải từ mã ta.
            //
            // `tao` cài `NSTextInputClient` trên macOS, nên dấu tiếng Việt —
            // kể cả ca hai tầng như `ổ` — đã được ghép xong trước khi tới đây.
            // Đường WebView cũng nhận đúng từ nguồn ấy. Ta KHÔNG tự viết bộ gõ,
            // và không nên: bộ gõ là thứ người dùng đã chọn và đã quen.
            Event::WindowEvent {
                event: WindowEvent::ReceivedImeText(chu),
                ..
            } => {
                if let Some(nhan) = tt.o_dang_chon.clone() {
                    tt.noi_dung_o.entry(nhan).or_default().push_str(&chu);
                    ve_lai = true;
                }
            }

            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        event:
                            tao::event::KeyEvent {
                                physical_key: tao::keyboard::KeyCode::Backspace,
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                if let Some(nhan) = tt.o_dang_chon.clone()
                    && let Some(v) = tt.noi_dung_o.get_mut(&nhan)
                {
                    // Xoá theo KÝ TỰ, không theo byte: một chữ có dấu là nhiều
                    // byte, và cắt byte là cắt vào giữa một ký tự.
                    v.pop();
                    ve_lai = true;
                }
            }

            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => {
                let co = window.inner_size().to_logical::<f64>(window.scale_factor());
                #[expect(clippy::cast_precision_loss, reason = "chiều cao ảnh, luôn nhỏ")]
                let cao_anh = bo_dung.height() as f64;
                tt.cuon = cuon_moi(
                    tt.cuon,
                    buoc_lan(&delta, window.scale_factor()),
                    cao_anh,
                    co.height,
                );
                window.request_redraw();
                window.request_redraw();
            }

            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                let l = position.to_logical::<f64>(window.scale_factor());
                tt.chuot = (l.x, l.y);
            }

            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state: ElementState::Released,
                        button: MouseButton::Left,
                        ..
                    },
                ..
            } => {
                xu_ly_cu_bam(
                    &bo_dung,
                    &mut tt,
                    &mut bat,
                    &mut ve_lai,
                    &mut da_bam,
                    dieu_khien,
                );
            }

            Event::RedrawRequested(_) | Event::MainEventsCleared => {
                // Rút yêu cầu bấm từ trợ năng và cho chạy qua ĐÚNG đường của
                // chuột. Không có nhánh riêng: mọi luật của hộp thoại áp cho
                // chuột đều áp cho đây.
                // ⚠️ `da_bam.is_none()` — màn hình đã kết thúc thì KHÔNG rút
                // thêm gì nữa. `tao` còn giao vài sự kiện sau khi ta đặt `Exit`,
                // và không có chắn này thì một yêu cầu còn trong hàng đợi **ghi
                // đè lựa chọn của người dùng**: họ bấm "Từ chối", rồi một
                // `AXPress("cho-phep")` xếp trước đó biến kết quả thành "Cho
                // phép". Rà soát 21/08/2026, F3.
                #[cfg(all(feature = "accesskit-platform", target_os = "macos"))]
                if da_bam.is_none()
                    && let Some(k) = rut_yeu_cau_tro_nang(&hang_tro_nang, &bang_hanh_dong, &mut bat)
                {
                    ap_ket_qua(k, &mut ve_lai, &mut da_bam, dieu_khien);
                }
                if core::mem::take(&mut ve_lai) {
                    ve_lai_man_hinh(&mut bo_dung, tree, &bat, &tt.noi_dung_o);
                    // Gạt một công tắc mà không báo lại thì VoiceOver vẫn đọc
                    // trạng thái CŨ — người dùng nghe "tắt" trong khi màn hình
                    // hiện "bật". Ở màn hỏi quyền, đó là nghe một đằng cấp một
                    // nẻo.
                    #[cfg(all(feature = "accesskit-platform", target_os = "macos"))]
                    bao_tro_nang(&bo_dung, chu, &cay_chung, &mut adapter, &mut bang_hanh_dong);
                }
                trinh_bay(&bo_dung, &mut be_mat, &window, tt.cuon);
            }
            _ => {}
        }
    });

    Ok(ScreenOutcome {
        action: da_bam,
        toggles_on: bat,
        fields: tt.noi_dung_o,
    })
}

/// Cú bấm dẫn tới cái gì.
#[derive(Debug, Clone, PartialEq, Eq)]
enum SauCuBam {
    /// Không rơi vào thứ bấm được.
    Khong,
    /// Công tắc đã đổi — vẽ lại, **ở lại màn hình**.
    VeLai,
    /// Nút đã bấm — kết thúc màn hình với hành động này.
    Ket(String),
}

/// Đổi trạng thái theo một cú bấm.
///
/// # Vì sao tách khỏi vòng lặp sự kiện
///
/// Ba luật an toàn của hộp thoại hỏi quyền sống ở đây: **gạt công tắc KHÔNG
/// đóng hộp thoại**, gạt hai lần thì **về lại tắt**, và bấm nút mới kết thúc.
/// Nằm trong vòng lặp sự kiện thì chúng chỉ được bảo đảm bằng mắt đọc mã — mà
/// vòng lặp ấy cần một cửa sổ thật nên `cargo test` không chạm tới được.
///
/// Tách ra là chúng kiểm được, và kiểm đột biến được.
fn sau_cu_bam(cham: Option<crate::Hit<'_>>, bat: &mut BTreeSet<String>) -> SauCuBam {
    let Some(h) = cham else {
        return SauCuBam::Khong;
    };
    if !h.toggle {
        return SauCuBam::Ket(h.action.to_owned());
    }
    // Gạt: có thì bỏ, không thì thêm. `remove` trả về `true` khi nó đã có —
    // nên một lần gọi làm cả hai việc, và không có nhánh nào quên mất một chiều.
    let a = h.action.to_owned();
    if !bat.remove(&a) {
        bat.insert(a);
    }
    SauCuBam::VeLai
}

/// Rút mọi yêu cầu bấm từ trợ năng và cho chạy qua **đúng đường của chuột**.
///
/// Không có nhánh riêng cho trợ năng: mọi luật của hộp thoại áp cho chuột đều
/// áp cho đây, vì cả hai đi qua cùng một [`sau_cu_bam`].
#[cfg(all(feature = "accesskit-platform", target_os = "macos"))]
fn rut_yeu_cau_tro_nang(
    hang: &Arc<Mutex<Vec<u64>>>,
    bang: &std::collections::BTreeMap<u64, (String, bool)>,
    bat: &mut BTreeSet<String>,
) -> Option<SauCuBam> {
    let yeu_cau: Vec<u64> = hang
        .lock()
        .map(|mut q| core::mem::take(&mut *q))
        .unwrap_or_default();
    let mut cuoi = None;
    for id in yeu_cau {
        // Không tra ra thì BỎ QUA. Yêu cầu tới sau khi cây đã đổi thì con số cũ
        // trỏ vào một nút không còn nữa — đoán bừa ở đây là bấm nhầm nút.
        let Some((a, la_cong_tac)) = bang.get(&id).cloned() else {
            continue;
        };
        let k = sau_cu_bam(
            Some(crate::Hit {
                action: &a,
                toggle: la_cong_tac,
            }),
            bat,
        );
        // Nút kết thúc màn hình: giữ nó và bỏ phần còn lại của hàng đợi. Chạy
        // tiếp sau một cú bấm kết thúc là chạy những cú bấm vào một màn hình
        // đã đóng.
        if matches!(k, SauCuBam::Ket(_)) {
            return Some(k);
        }
        // Giữ `VeLai` chứ không để lần sau ghi đè: hàng đợi `[công tắc, nút]`
        // mà đánh rơi `VeLai` thì công tắc đổi **không vẽ lại và không báo cho
        // trình đọc màn hình**, rồi màn hình kết thúc mang theo thay đổi ấy.
        if !matches!(cuoi, Some(SauCuBam::VeLai)) {
            cuoi = Some(k);
        }
    }
    cuoi
}

/// Nối adapter trợ năng của macOS vào cửa sổ.
///
/// ⚠️ Gọi **TRƯỚC** khi cửa sổ được hiện hay nhận tiêu điểm lần đầu —
/// `SubclassingAdapter::new` nói rõ điều đó. Nối muộn thì VoiceOver đã hỏi xong
/// và nhận câu "không có gì ở đây", rồi không hỏi lại.
#[cfg(all(feature = "accesskit-platform", target_os = "macos"))]
fn noi_tro_nang(
    window: &tao::window::Window,
    cay: Arc<Mutex<accesskit::TreeUpdate>>,
    hang: Arc<Mutex<Vec<u64>>>,
) -> accesskit_macos::SubclassingAdapter {
    use tao::platform::macos::WindowExtMacOS as _;

    // ⚠️ ĐÂY LÀ CHỖ `unsafe` DUY NHẤT CỦA DỰ ÁN.
    //
    // `Cargo.toml` đặt `unsafe_code = "deny"` toàn workspace, và `SECURITY.md`
    // §3.1b đã lường trước đúng đánh đổi này: *"làm bây giờ nghĩa là thêm
    // `unsafe` FFI, chỉ phủ macOS"* — rồi hoãn nó tới giai đoạn 4. Đây là
    // giai đoạn 4.
    //
    // Không có đường vòng: trao con trỏ `NSView` cho một API của hệ điều
    // hành thì không có bọc an toàn nào, và AccessKit cũng không có bản nào
    // cho `tao`. Chọn giữa MỘT dòng FFI và một bộ dựng VoiceOver không đọc
    // được thì chọn dòng FFI — nhưng chọn công khai, hẹp, và có lý do viết
    // ra ngay đây.
    //
    // Dùng `expect` chứ không `allow`: ngày nào có bọc an toàn, lint tự báo
    // rằng ngoại lệ này thừa, thay vì nằm lại mãi.
    //
    // SAFETY: `ns_view` của `tao` trả về NSView của cửa sổ vừa dựng xong.
    // `window` sống tới cuối hàm — sau `run_return` — nên con trỏ còn hợp lệ
    // suốt đời adapter, và adapter rơi trước `window`.
    #[expect(
        unsafe_code,
        reason = "trao con trỏ NSView cho AccessKit — không có bọc an toàn nào cho tao"
    )]
    unsafe {
        accesskit_macos::SubclassingAdapter::new(
            window.ns_view(),
            tro_nang::KhiKichHoat(cay),
            tro_nang::NhanHanhDong(hang),
        )
    }
}

/// Trạng thái người dùng tạo ra trên một màn hình raster.
///
/// Gom lại vì bốn thứ này **luôn đi cùng nhau**: cú bấm cần cuộn để biết nó rơi
/// vào đâu, và gõ phím cần ô đang chọn để biết chữ đi đâu. Rời rạc thì mỗi hàm
/// nhận thêm một tham số, và sẽ có hàm nhận thiếu một cái.
#[derive(Debug, Default)]
struct TrangThai {
    /// Cuộn, đơn vị logic, tính từ đỉnh nội dung.
    cuon: f64,
    /// Vị trí chuột gần nhất, đơn vị LOGIC. `tao` báo theo pixel vật lý, mà bộ
    /// dựng làm việc theo logic — trên màn hình Retina hai thứ lệch đúng hệ số
    /// 2, và bấm sẽ trúng ô khác.
    chuot: (f64, f64),
    /// Nội dung ô nhập, tra theo NHÃN. Ô nhập không có mã hành động.
    noi_dung_o: std::collections::BTreeMap<String, String>,
    /// Ô đang được chọn. `None` = gõ phím KHÔNG đi đâu cả — thà không nhận còn
    /// hơn nhận vào một ô người dùng không nhìn.
    o_dang_chon: Option<String>,
}

/// Dựng lại cây từ trạng thái khung đang giữ, rồi vẽ.
///
/// Trạng thái công tắc và nội dung ô nhập do KHUNG giữ, không do cây giữ — cây
/// bất biến. Đây là chỗ hai thứ ấy quay lại thành một màn hình.
///
/// Vẽ hỏng thì **giữ nguyên lần vẽ trước**: hình học cũ vẫn khớp với thứ người
/// dùng đang nhìn, nên một cú bấm vẫn trúng đúng ô. Đổi lại, trạng thái trong
/// bụng đã đi trước — xem F4 của rà soát 21/08/2026, vẫn chưa có lời giải.
fn ve_lai_man_hinh(
    bo_dung: &mut RasterRenderer,
    tree: &Node,
    bat: &BTreeSet<String>,
    noi_dung_o: &std::collections::BTreeMap<String, String>,
) {
    if let Ok(cay_moi) = tree
        .with_toggles(bat)
        .and_then(|c| c.with_fields(noi_dung_o))
    {
        let _ = bo_dung.render(&cay_moi);
    }
}

/// Một cú bấm chuột: hoặc CHỌN một ô nhập, hoặc chạy một hành động.
///
/// Hỏi ô nhập TRƯỚC. Một cú bấm chỉ thuộc về một trong hai, và nếu hỏi hành
/// động trước thì một cú bấm vào ô nhập nằm đè lên nút sẽ chạy mất nút ấy.
fn xu_ly_cu_bam(
    bo_dung: &RasterRenderer,
    tt: &mut TrangThai,
    bat: &mut BTreeSet<String>,
    ve_lai: &mut bool,
    da_bam: &mut Option<String>,
    dieu_khien: &mut ControlFlow,
) {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "toạ độ màn hình, luôn nằm gọn trong f32"
    )]
    let x = tt.chuot.0 as f32;
    let y = cong_cuon(tt.chuot.1, tt.cuon);
    if let Some(nhan) = bo_dung.hit_test_field(x, y) {
        tt.o_dang_chon = Some(nhan.to_owned());
    } else {
        let k = sau_cu_bam(bo_dung.hit_test(x, y), bat);
        ap_ket_qua(k, ve_lai, da_bam, dieu_khien);
    }
}

/// Áp kết quả một cú bấm — dùng chung cho chuột VÀ cho trợ năng.
///
/// Một hàm, hai đường vào: mọi luật của hộp thoại áp cho chuột thì áp cho trợ
/// năng, và không có nhánh nào sửa được một bên mà quên bên kia.
fn ap_ket_qua(
    k: SauCuBam,
    ve_lai: &mut bool,
    da_bam: &mut Option<String>,
    dieu_khien: &mut ControlFlow,
) {
    match k {
        SauCuBam::VeLai => *ve_lai = true,
        SauCuBam::Ket(a) => {
            *da_bam = Some(a);
            *dieu_khien = ControlFlow::Exit;
        }
        SauCuBam::Khong => {}
    }
}

/// Toạ độ y trên MÀN HÌNH cộng với cuộn → toạ độ y trong NỘI DUNG.
///
/// ⚠️ Quên phép cộng này là bấm trúng ô khác — người dùng thấy một nút, hệ
/// thống chạy một nút khác. Cùng hạng lỗi với F1 của rà soát 21/08/2026, chỉ
/// khác chỗ sinh ra: ở đó là bố cục tràn, ở đây là cuộn.
#[expect(
    clippy::cast_possible_truncation,
    reason = "toạ độ màn hình, luôn nằm gọn trong f32"
)]
fn cong_cuon(y_man_hinh: f64, cuon: f64) -> f32 {
    (y_man_hinh + cuon) as f32
}

/// Dựng cửa sổ, **giới hạn chiều cao theo màn hình**.
///
/// # Errors
/// Không dựng được cửa sổ.
fn dung_cua_so(
    vong: &tao::event_loop::EventLoop<()>,
    tieu_de: &str,
    cao: usize,
) -> Result<tao::window::Window, String> {
    // ⚠️ KHÔNG mở cửa sổ cao bằng cả nội dung.
    //
    // Cây cao tới `MAX_HEIGHT` = 4096, mà màn hình thì không. Mở đúng chiều cao
    // nội dung nghĩa là nút Cho phép / Từ chối nằm dưới mép màn hình và người
    // dùng **không bấm được gì**. Hộp thoại hỏi quyền để ứng dụng quyết định cả
    // số quyền lẫn độ dài từng câu `reason`, nên độ cao ấy do ứng dụng điều
    // khiển — nó không được điều khiển luôn việc nút có bấm tới được hay không.
    //
    // Chặn hai lớp: giới hạn ở đây, và cuộn được ở dưới.
    let cao_man = vong.primary_monitor().map_or(900.0, |m| {
        m.size().to_logical::<f64>(m.scale_factor()).height
    });
    #[expect(clippy::cast_precision_loss, reason = "chiều cao ảnh, luôn nhỏ")]
    let cao_cua_so = (cao as f64).min(cao_man * 0.85);
    let w = WindowBuilder::new()
        .with_title(tieu_de)
        .with_inner_size(LogicalSize::new(WIDTH as f64, cao_cua_so))
        // ⚠️ DỰNG ẨN, hiện SAU khi đã nối trợ năng.
        //
        // `SubclassingAdapter::new` đòi được gọi trước khi khung nhìn được hiện
        // hay nhận tiêu điểm lần đầu. Chú thích bên dưới từng KHẲNG ĐỊNH điều
        // đó trong khi mã làm ngược lại — `build` của `tao` hiện và lấy tiêu
        // điểm ngay. Hậu quả đúng như chú thích tự mô tả: VoiceOver hỏi trước,
        // nhận "không có gì ở đây", rồi không hỏi lại. Rà soát 21/08/2026, F6.
        .with_visible(false)
        .build(vong)
        .map_err(|e| format!("không dựng được cửa sổ: {e}"))?;
    Ok(w)
}

/// Vị trí cuộn mới sau một nhịp lăn, đã **kẹp trong khoảng hợp lệ**.
///
/// Nhận toàn số — không nhận cửa sổ — để **kiểm được**. Bản đầu nhận
/// `&tao::window::Window` cho tiện, và phép kẹp trở thành thứ chỉ chạy khi có
/// một cửa sổ thật, tức là thứ `cargo test` không bao giờ chạm tới.
///
/// Kẹp là bắt buộc ở cả hai đầu: cuộn quá đáy thì lượt vẽ đọc ngoài ảnh và
/// người dùng nhìn một khoảng trắng không giải thích được; cuộn ngược lên trên
/// đỉnh thì nội dung trôi khỏi khung.
#[must_use]
fn cuon_moi(cuon: f64, buoc: f64, cao_anh: f64, cao_cua_so: f64) -> f64 {
    let toi_da = (cao_anh - cao_cua_so).max(0.0);
    (cuon - buoc).clamp(0.0, toi_da)
}

/// Số điểm ảnh logic một nhịp lăn tương ứng.
fn buoc_lan(delta: &tao::event::MouseScrollDelta, ty_le: f64) -> f64 {
    match *delta {
        // Một "dòng" không có kích thước chuẩn; 24 px là một dòng chữ cỡ thường.
        tao::event::MouseScrollDelta::LineDelta(_, d) => f64::from(d) * 24.0,
        tao::event::MouseScrollDelta::PixelDelta(p) => p.to_logical::<f64>(ty_le).y,
        _ => 0.0,
    }
}

/// Đưa lần vẽ gần nhất lên màn hình.
fn trinh_bay(
    bo_dung: &RasterRenderer,
    be_mat: &mut Surface<&tao::window::Window, &tao::window::Window>,
    window: &tao::window::Window,
    cuon: f64,
) {
    let co = window.inner_size();
    let (Some(w), Some(h)) = (
        core::num::NonZeroU32::new(co.width),
        core::num::NonZeroU32::new(co.height),
    ) else {
        return;
    };
    if be_mat.resize(w, h).is_err() {
        return;
    }
    let Ok(mut dem) = be_mat.buffer_mut() else {
        return;
    };
    to_mau(
        bo_dung,
        &mut dem,
        w.get(),
        h.get(),
        window.scale_factor(),
        cuon,
    );
    let _ = dem.present();
}

/// Bảng tra `NodeId` → (mã hành động, có phải công tắc không).
#[cfg(all(feature = "accesskit-platform", target_os = "macos"))]
fn bang_hanh_dong_cua(bd: &RasterRenderer) -> std::collections::BTreeMap<u64, (String, bool)> {
    use tcc_ui::Renderer as _;
    // Bảng tra không phụ thuộc chữ nghĩa — chỉ cần `NodeId`. Nên chỗ này dùng
    // mặc định là ĐÚNG, khác hẳn `cay_accesskit` bên dưới.
    bd.published_accessibility()
        .map_or_else(Default::default, |goc| {
            crate::accesskit_bridge::to_accesskit_with_actions(
                &goc,
                &crate::accesskit_bridge::AccessText::default(),
            )
            .1
        })
}

/// Cây rỗng — dùng khi chưa vẽ được gì.
#[cfg(all(feature = "accesskit-platform", target_os = "macos"))]
fn cay_rong() -> accesskit::TreeUpdate {
    accesskit::TreeUpdate {
        nodes: Vec::new(),
        tree_id: accesskit::TreeId::ROOT,
        tree: None,
        focus: accesskit::NodeId(0),
    }
}

/// Báo cây mới cho trợ năng: ghi vào cây CHIA SẺ trước, rồi mới đẩy.
///
/// Ghi trước và luôn luôn, vì `update_if_active` vứt cây khi chưa ai nghe.
#[cfg(all(feature = "accesskit-platform", target_os = "macos"))]
fn bao_tro_nang(
    bo_dung: &RasterRenderer,
    chu: &ScreenText,
    cay_chung: &Arc<Mutex<accesskit::TreeUpdate>>,
    adapter: &mut accesskit_macos::SubclassingAdapter,
    bang: &mut std::collections::BTreeMap<u64, (String, bool)>,
) {
    *bang = bang_hanh_dong_cua(bo_dung);
    if let Some(moi) = cay_accesskit(bo_dung, chu) {
        if let Ok(mut c) = cay_chung.lock() {
            *c = moi.clone();
        }
        if let Some(su_kien) = adapter.update_if_active(|| moi) {
            su_kien.raise();
        }
    }
}

/// Cây trợ năng của lần vẽ gần nhất, ở dạng AccessKit.
#[cfg(all(feature = "accesskit-platform", target_os = "macos"))]
fn cay_accesskit(bd: &RasterRenderer, chu: &ScreenText) -> Option<accesskit::TreeUpdate> {
    use tcc_ui::Renderer as _;
    // Chưa vẽ thì chưa có cây. Trả `None` chứ không hoảng loạn: thiếu trợ năng
    // là một màn hình đọc không được, còn hoảng loạn là một cửa sổ biến mất —
    // và cái sau tệ hơn cho đúng người mà tính năng này sinh ra để phục vụ.
    let goc = bd.published_accessibility()?;
    Some(crate::accesskit_bridge::to_accesskit(
        &goc,
        &crate::accesskit_bridge::AccessText {
            cau_mat_mat: chu.destructive_note.clone(),
            vai_tro_mat_mat: chu.destructive_role.clone(),
        },
    ))
}

/// Đổ ảnh xám của bộ dựng vào đệm màu của cửa sổ.
///
/// Ảnh của bộ dựng là **một byte mỗi điểm**, đơn vị logic. Cửa sổ đòi `u32`
/// `0RGB`, đơn vị vật lý. Nên vừa nhân ba kênh vừa chia tỷ lệ — chia bằng lấy
/// mẫu gần nhất, vì đây là để NHÌN THẤY được, không phải để so ảnh; phép so ảnh
/// chạy trên `image()` ở đơn vị logic, không chạy trên đệm này.
fn to_mau(bd: &RasterRenderer, ra: &mut [u32], rong: u32, cao: u32, ty_le: f64, cuon: f64) {
    let anh = bd.image();
    let cao_anh = bd.height();
    for y in 0..cao {
        for x in 0..rong {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "lấy mẫu gần nhất trên toạ độ màn hình"
            )]
            let (lx, ly) = (
                (f64::from(x) / ty_le) as usize,
                (f64::from(y) / ty_le + cuon) as usize,
            );
            // Ngoài ảnh thì trắng: cửa sổ kéo to hơn ảnh là chuyện thường, và
            // để nguyên bộ nhớ chưa ghi thì ra một mảng nhiễu.
            let xam = if lx < WIDTH && ly < cao_anh {
                u32::from(anh[ly * WIDTH + lx])
            } else {
                255
            };
            if let Some(o) = ra.get_mut((y * rong + x) as usize) {
                *o = (xam << 16) | (xam << 8) | xam;
            }
        }
    }
}

/// Nối cây trợ năng vào **hệ điều hành**.
///
/// # Vì sao đây là mục cuối của giai đoạn 4
///
/// `spec/0.1/05-interface.md` có mục **"Accessibility — no opt-out"**. Một bộ
/// dựng mà VoiceOver không đọc được thì không thể thành mặc định, dù nó vẽ đẹp
/// tới đâu — nên chừng nào chưa nối được, đường thoát khỏi WebView vẫn còn
/// thiếu một chân.
///
/// Cây trợ năng đã đúng từ 17/08/2026: nó được ghi lại **trong lúc vẽ**, không
/// gọi lại `Node::accessibility_tree()`. Thiếu đúng đoạn cuối này — đưa nó cho
/// hệ điều hành.
///
/// # Chỉ macOS
///
/// Windows (`accesskit_windows`) và Linux (`accesskit_unix`) chưa nối. Nói ra
/// bằng `cfg` chứ không bằng một chú thích: bản dựng trên hai nền kia **không có
/// mã trợ năng nào cả**, và đó là sự thật cần nhìn thấy được.
#[cfg(all(feature = "accesskit-platform", target_os = "macos"))]
mod tro_nang {
    use std::sync::{Arc, Mutex};

    use accesskit::{ActionHandler, ActionRequest, ActivationHandler, TreeUpdate};

    /// Trả cây đầu tiên khi hệ điều hành hỏi tới.
    ///
    /// Giữ sẵn một bản: hệ điều hành hỏi vào lúc nó muốn, không vào lúc ta vẽ.
    /// # ⚠️ Phải là trạng thái CHIA SẺ, không phải một ảnh chụp
    ///
    /// Bản đầu giữ một `TreeUpdate` chụp trước vòng lặp và trả lại nó mãi. Sai,
    /// và sai đúng theo cách nguy hiểm nhất: `Adapter::update_if_active` trả
    /// `None` **và không gọi hàm dựng cây** khi chưa có ai nghe. Nên mọi lần vẽ
    /// lại lúc VoiceOver còn tắt đều bị vứt, còn `request_initial_tree` — gọi
    /// đúng một lần khi có người nghe — trả về cây của **lần vẽ số 0**.
    ///
    /// Trên màn hỏi quyền: người dùng gạt một quyền bằng chuột, rồi bật
    /// VoiceOver, và nghe **"tắt"** trong khi màn hình hiện **"bật"**. Rà soát
    /// 21/08/2026, F2.
    pub struct KhiKichHoat(pub Arc<Mutex<TreeUpdate>>);

    impl ActivationHandler for KhiKichHoat {
        fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
            self.0.lock().ok().map(|t| t.clone())
        }
    }

    /// Nhận yêu cầu **bấm** từ hệ điều hành, đẩy vào hàng đợi.
    ///
    /// # Tôi đã hoãn việc này một ngày vì một lập luận SAI
    ///
    /// Hôm 19/08 tôi để chỗ này trống, với lý do: nhận yêu cầu "bấm nút này" là
    /// mở một đường bấm nút **không qua chuột**, mà trên màn xác nhận giao dịch
    /// đó là một đường **ký hộ**.
    ///
    /// Lập luận ấy sai ở chỗ nó so với một thế giới không tồn tại. Trên macOS,
    /// một ứng dụng muốn gửi `AXPress` phải được cấp quyền **Accessibility**
    /// trong Cài đặt hệ thống. Mà cùng cái quyền ấy cũng cho phép `CGEventPost`
    /// — tức là **tổng hợp một cú bấm chuột thật**, đi thẳng qua đường chuột của
    /// ta mà không cần API trợ năng nào.
    ///
    /// Nên từ chối `AXPress` **không chặn được kẻ tấn công**: nó đã có một
    /// đường tương đương sau cùng một cánh cổng. Nó chỉ chặn **người dùng
    /// VoiceOver** — những người mà tính năng này sinh ra để phục vụ.
    ///
    /// *Một biện pháp chỉ cản người dùng hợp lệ mà không cản kẻ tấn công thì
    /// không phải biện pháp an ninh, nó là một rào cản.*
    ///
    /// Vẫn còn rủi ro thật, và nó nằm ở **hệ điều hành**: ai cấp quyền
    /// Accessibility cho một ứng dụng lạ thì ứng dụng ấy điều khiển được mọi
    /// cửa sổ trên máy, không riêng của ta. Ghi vào `SECURITY.md` chứ không
    /// giả vờ đóng nó bằng một hàm rỗng.
    ///
    /// # Cùng một đường với chuột, không phải đường riêng
    ///
    /// Yêu cầu chỉ được đẩy vào hàng đợi. Vòng lặp rút ra rồi cho chạy qua
    /// **đúng `sau_cu_bam`** mà cú bấm chuột đi qua — nên không có luật nào của
    /// hộp thoại áp cho chuột mà không áp cho trợ năng.
    pub struct NhanHanhDong(pub Arc<Mutex<Vec<u64>>>);

    impl ActionHandler for NhanHanhDong {
        fn do_action(&mut self, yeu_cau: ActionRequest) {
            // CHỈ nhận "bấm". Mọi hành động khác — cuộn, đặt tiêu điểm, đặt giá
            // trị — chưa được nghĩ tới, và im lặng bỏ qua đúng hơn là đoán.
            if yeu_cau.action == accesskit::Action::Click
                && let Ok(mut q) = self.0.lock()
            {
                q.push(yeu_cau.target_node.0);
            }
        }
    }
}

#[cfg(test)]
mod kiem_thu {
    use super::*;

    fn nut(a: &str) -> crate::Hit<'_> {
        crate::Hit {
            action: a,
            toggle: false,
        }
    }
    fn cong_tac(a: &str) -> crate::Hit<'_> {
        crate::Hit {
            action: a,
            toggle: true,
        }
    }

    /// **Ô che chữ trả về CHỮ THẬT, không phải hàng chấm.**
    ///
    /// Hàng chấm là việc của lúc VẼ. Nếu kết quả trả về cũng là chấm thì màn
    /// nhập PIN vô dụng — khung nhận về `••••` và không mở được ví nào.
    ///
    /// Phép thử soi chỗ dựng kết quả, vì đường chạy thật cần một cửa sổ.
    #[test]
    fn o_che_chu_tra_ve_chu_that() {
        let nguon = include_str!("window.rs");
        let than = nguon.split("#[cfg(test)]").next().unwrap_or(nguon);
        // Kết quả lấy THẲNG từ trạng thái khung, không đi qua bước vẽ nào.
        assert!(
            than.contains("fields: tt.noi_dung_o"),
            "kết quả không mang nội dung ô nhập"
        );
        // Và chỗ DUY NHẤT biến chữ thành chấm phải nằm ở bộ dựng, không ở đây.
        assert!(
            !than.contains('•'),
            "cửa sổ đang tự che chữ — việc ấy thuộc về lúc vẽ, và làm ở đây thì \
             khung nhận về hàng chấm thay vì thứ người dùng gõ"
        );
    }

    /// **Cuộn bị KẸP ở cả hai đầu.**
    ///
    /// Quá đáy thì lượt vẽ đọc ngoài ảnh — người dùng nhìn một khoảng trắng
    /// không giải thích được. Quá đỉnh thì nội dung trôi khỏi khung.
    #[test]
    fn cuon_bi_kep_o_ca_hai_dau() {
        // Nội dung 1000, cửa sổ 300 → cuộn tối đa 700.
        assert!((cuon_moi(0.0, 24.0, 1000.0, 300.0) - 0.0).abs() < f64::EPSILON);
        assert!((cuon_moi(0.0, -24.0, 1000.0, 300.0) - 24.0).abs() < f64::EPSILON);
        assert!((cuon_moi(690.0, -9999.0, 1000.0, 300.0) - 700.0).abs() < f64::EPSILON);
        assert!((cuon_moi(10.0, 9999.0, 1000.0, 300.0) - 0.0).abs() < f64::EPSILON);
        // Nội dung vừa khít hoặc nhỏ hơn cửa sổ → KHÔNG cuộn được chút nào.
        assert!((cuon_moi(0.0, -50.0, 300.0, 300.0) - 0.0).abs() < f64::EPSILON);
        assert!((cuon_moi(0.0, -50.0, 100.0, 300.0) - 0.0).abs() < f64::EPSILON);
    }

    /// **Cuộn phải được CỘNG vào toạ độ bấm.**
    ///
    /// Đây là hạng lỗi F1 mặc áo khác: người dùng thấy một nút, hệ thống chạy
    /// một nút khác. Cuộn xuống 100 px rồi bấm ở y=10 trên màn hình thì cú bấm
    /// ấy thuộc về y=110 trong nội dung.
    ///
    /// Phép thử soi CHỖ GỌI, vì phép tính nằm trong vòng lặp sự kiện — mà vòng
    /// lặp ấy cần một cửa sổ thật nên `cargo test` không chạm tới được. Cùng
    /// cách đã dùng cho tiêu đề hộp thoại hỏi quyền.
    #[test]
    fn cuon_duoc_cong_vao_toa_do_bam() {
        let nguon = include_str!("window.rs");
        let than = nguon.split("#[cfg(test)]").next().unwrap_or(nguon);
        // Soi hàm xử lý cú bấm, không soi `open_screen`: phép tính này đã dời
        // sang đó, và phép thử BẮT ĐƯỢC lúc nó dời — đúng việc của nó.
        let Some(goi) = than.split("fn xu_ly_cu_bam").nth(1) else {
            panic!("không tìm thấy `xu_ly_cu_bam`")
        };
        let goi = &goi[..goi.find("\n}").unwrap_or(goi.len())];
        assert!(
            goi.contains("cong_cuon(tt.chuot.1, tt.cuon)"),
            "phép bấm KHÔNG cộng cuộn — cuộn xuống rồi bấm là trúng ô khác"
        );
        // Và cửa sổ không được mở cao bằng cả nội dung.
        let Some(dung) = than.split("fn dung_cua_so").nth(1) else {
            panic!("không tìm thấy `dung_cua_so`")
        };
        assert!(
            dung.contains("cao_man") && dung.contains(".min("),
            "cửa sổ mở đúng chiều cao nội dung — nút dưới đáy sẽ nằm ngoài màn hình"
        );
    }

    /// **Gạt công tắc KHÔNG đóng hộp thoại.**
    ///
    /// Đây là luật quan trọng nhất của màn hỏi quyền trên bộ dựng này: đóng khi
    /// gạt nghĩa là người dùng vừa "trả lời" cả những mục họ chưa kịp đọc.
    #[test]
    fn gat_cong_tac_khong_dong_hop_thoai() {
        let mut bat = BTreeSet::new();
        assert_eq!(
            sau_cu_bam(Some(cong_tac("micro")), &mut bat),
            SauCuBam::VeLai
        );
        assert!(bat.contains("micro"));
        assert_eq!(
            sau_cu_bam(Some(cong_tac("camera")), &mut bat),
            SauCuBam::VeLai
        );
        assert_eq!(bat.len(), 2);
    }

    /// Gạt hai lần thì **về lại tắt**.
    #[test]
    fn gat_hai_lan_thi_ve_lai_tat() {
        let mut bat = BTreeSet::new();
        sau_cu_bam(Some(cong_tac("micro")), &mut bat);
        sau_cu_bam(Some(cong_tac("micro")), &mut bat);
        assert!(bat.is_empty(), "gạt hai lần mà vẫn bật: {bat:?}");
    }

    /// **Nút mới kết thúc màn hình**, và nó KHÔNG đụng vào công tắc nào.
    #[test]
    fn nut_ket_thuc_va_khong_doi_cong_tac() {
        let mut bat = BTreeSet::new();
        sau_cu_bam(Some(cong_tac("micro")), &mut bat);
        let truoc = bat.clone();
        assert_eq!(
            sau_cu_bam(Some(nut("cho-phep")), &mut bat),
            SauCuBam::Ket("cho-phep".to_owned())
        );
        assert_eq!(bat, truoc, "bấm nút mà trạng thái công tắc đổi");
    }

    /// Bấm vào khoảng trống thì **không đổi gì cả**.
    #[test]
    fn bam_khoang_trong_khong_doi_gi() {
        let mut bat = BTreeSet::new();
        bat.insert("micro".to_owned());
        assert_eq!(sau_cu_bam(None, &mut bat), SauCuBam::Khong);
        assert_eq!(bat.len(), 1);
    }
}
