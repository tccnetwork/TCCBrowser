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
}

/// Mở màn hình `tree` bằng bộ dựng ra pixel.
///
/// **Nút** kết thúc màn hình. **Công tắc** đổi câu trả lời rồi vẽ lại, ở lại.
///
/// # Errors
/// Cây không vẽ được, hoặc không dựng được cửa sổ.
pub fn open_screen(tree: &Node, tieu_de: &str) -> Result<ScreenOutcome, String> {
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
    let window = WindowBuilder::new()
        .with_title(tieu_de)
        .with_inner_size(LogicalSize::new(WIDTH as f64, cao as f64))
        .build(&vong)
        .map_err(|e| format!("không dựng được cửa sổ: {e}"))?;

    // ⚠️ `Context`/`Surface` phải sống lâu bằng cửa sổ. Thả sớm là mất bề mặt,
    // và triệu chứng là một cửa sổ trắng trơn — trông y như bộ dựng vẽ hỏng.
    let ngu_canh = Context::new(&window).map_err(|e| format!("không mở được bề mặt: {e}"))?;
    let mut be_mat =
        Surface::new(&ngu_canh, &window).map_err(|e| format!("không mở được bề mặt: {e}"))?;

    let mut da_bam: Option<String> = None;
    let mut ve_lai = false;
    // Vị trí chuột gần nhất, đơn vị LOGIC. `tao` báo vị trí theo pixel vật lý,
    // mà bộ dựng làm việc theo đơn vị logic — trên màn hình Retina hai thứ ấy
    // lệch nhau đúng hệ số 2, và bấm sẽ trúng ô khác.
    let mut chuot = (0.0f64, 0.0f64);

    vong.run_return(|su_kien, _, dieu_khien| {
        *dieu_khien = ControlFlow::Wait;
        match su_kien {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *dieu_khien = ControlFlow::Exit,

            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                let l = position.to_logical::<f64>(window.scale_factor());
                chuot = (l.x, l.y);
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
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "toạ độ màn hình, luôn nằm gọn trong f32"
                )]
                if let Some(h) = bo_dung.hit_test(chuot.0 as f32, chuot.1 as f32) {
                    if h.toggle {
                        // Gạt rồi Ở LẠI. Vẽ lại ngay: một công tắc gạt mà màn
                        // hình không đổi là người dùng bấm tiếp lần nữa, và gạt
                        // ngược lại thứ họ vừa bật.
                        let a = h.action.to_owned();
                        if !bat.remove(&a) {
                            bat.insert(a);
                        }
                        ve_lai = true;
                    } else {
                        da_bam = Some(h.action.to_owned());
                        *dieu_khien = ControlFlow::Exit;
                    }
                }
            }

            Event::RedrawRequested(_) | Event::MainEventsCleared => {
                if ve_lai {
                    ve_lai = false;
                    if let Ok(cay_moi) = tree.with_toggles(&bat) {
                        let _ = bo_dung.render(&cay_moi);
                    }
                }
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
                to_mau(&bo_dung, &mut dem, w.get(), h.get(), window.scale_factor());
                let _ = dem.present();
            }
            _ => {}
        }
    });

    Ok(ScreenOutcome {
        action: da_bam,
        toggles_on: bat,
    })
}

/// Đổ ảnh xám của bộ dựng vào đệm màu của cửa sổ.
///
/// Ảnh của bộ dựng là **một byte mỗi điểm**, đơn vị logic. Cửa sổ đòi `u32`
/// `0RGB`, đơn vị vật lý. Nên vừa nhân ba kênh vừa chia tỷ lệ — chia bằng lấy
/// mẫu gần nhất, vì đây là để NHÌN THẤY được, không phải để so ảnh; phép so ảnh
/// chạy trên `image()` ở đơn vị logic, không chạy trên đệm này.
fn to_mau(bd: &RasterRenderer, ra: &mut [u32], rong: u32, cao: u32, ty_le: f64) {
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
                (f64::from(y) / ty_le) as usize,
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
