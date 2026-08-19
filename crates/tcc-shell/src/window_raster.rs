//! Cửa sổ của bộ dựng **RA PIXEL** — không một dòng `wry`.
//!
//! Đây là bản song song của `window.rs` cho bộ dựng thứ hai. Nó tồn tại tách
//! biệt vì cờ `window` kéo theo cả một máy dựng web, mà cổng ra giai đoạn 4
//! phải chứng minh được **không cần** thứ đó.

use tcc_spec::Manifest;

use crate::permission_dialog;
use crate::text::Language;
use crate::window_title::app_window_title;

/// Mở màn hình của một ứng dụng bằng bộ dựng **RA PIXEL**.
///
/// # Vì sao hàm này tồn tại thay vì gọi thẳng `open_screen`
///
/// Tiêu đề cửa sổ là một **luật an ninh**, không phải một chuỗi tiện tay: mã
/// ứng dụng đã ký phải đứng trước tên do ứng dụng tự đặt, nếu không một gói tên
/// `"TCC — quyền đã cấp"` có cửa sổ trông y hệt màn hình quản lý quyền của
/// trình duyệt. Xem [`app_window_title`] và `SECURITY.md` §3.1c.
///
/// Luật ấy đã vá ở đường WebView từ 16/08/2026. Ví dụ raster đầu tiên truyền
/// thẳng `manifest().name` và **mở lại đúng cái lỗ ấy** trên bộ dựng mới — nên
/// tiêu đề chuyển vào đây, chỗ mọi bên gọi đều đi qua, thay vì nằm ở bên gọi
/// nơi người tiếp theo lại quên.
///
/// # Errors
/// Cây giao diện hỏng, hoặc không dựng được cửa sổ.
#[cfg(feature = "cua-so-raster")]
pub fn open_app_screen_raster(
    m: &Manifest,
    diem_vao: &[u8],
    ngon_ngu: Language,
) -> Result<tcc_render_raster::window::ScreenOutcome, Box<dyn std::error::Error>> {
    let cay = tcc_ui::wire::decode(diem_vao)?;
    Ok(tcc_render_raster::window::open_screen(
        &cay,
        &app_window_title(m),
        crate::text::destructive_note(ngon_ngu),
    )?)
}

/// Hộp thoại hỏi quyền bằng bộ dựng **RA PIXEL**.
///
/// Tiêu đề là của TRÌNH DUYỆT, không phải của ứng dụng — và nó **không bao giờ
/// mang mã ứng dụng**, vì đó chính là dấu người dùng phân biệt cửa sổ của khung
/// với cửa sổ của một gói.
///
/// # Errors
/// Không dựng được hộp thoại hoặc cửa sổ.
#[cfg(feature = "cua-so-raster")]
pub fn open_permission_dialog_raster(
    m: &Manifest,
    ngon_ngu: Language,
) -> Result<tcc_render_raster::window::ScreenOutcome, Box<dyn std::error::Error>> {
    let cay = permission_dialog::build(m, ngon_ngu)?;
    Ok(tcc_render_raster::window::open_screen(
        &cay,
        crate::text::label(crate::text::TextKey::HoiQuyenTieuDeCuaSo, ngon_ngu),
        crate::text::destructive_note(ngon_ngu),
    )?)
}
