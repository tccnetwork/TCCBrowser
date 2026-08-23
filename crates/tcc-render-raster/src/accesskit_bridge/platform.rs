//! Nối cây trợ năng vào **hệ điều hành thật** — macOS, Windows **và** Linux.
//!
//! # Vì sao có tệp này
//!
//! `window.rs` đã nối được VoiceOver từ giai đoạn 4, nhưng nối bằng đúng một
//! `cfg(target_os = "macos")`. Câu ấy nói ra sự thật lúc đó — và sự thật ấy là:
//! *bộ dựng ra pixel chỉ đọc được trên một trong ba nền*. Một bộ dựng như thế
//! KHÔNG thay được WebView, vì `spec/0.1/05-interface.md` có mục
//! **"Accessibility — no opt-out"** và mục ấy không có phần "trừ Windows và
//! Linux".
//!
//! Tệp này là hai chân còn thiếu.
//!
//! # ⚠️ CHƯA MỘT LẦN NÀO CHẠY VỚI TRÌNH ĐỌC MÀN HÌNH THẬT
//!
//! Nói thẳng ra đây vì nó là thứ người đọc mã cần biết trước mọi thứ khác: mã
//! trong tệp này **chưa từng được thử với NVDA, Narrator hay Orca**. Nó được
//! viết trên macOS, và thứ duy nhất được kiểm là **nó biên dịch được cho hai
//! nền kia** (CI dựng nó trên `windows-latest` và `ubuntu-latest`) cùng vài
//! phép thử thuần logic chạy được ở mọi nơi.
//!
//! "Biên dịch được" và "đọc được" là hai câu khác nhau. Đừng ghi vào tài liệu
//! nào rằng Windows/Linux đã có trợ năng cho tới khi có người bật NVDA/Orca lên
//! và nghe thử — và khi ấy, sửa đúng đoạn này.
//!
//! # Ba nền, ba hình dạng khác nhau
//!
//! | | macOS | Windows | Linux (AT-SPI) |
//! |---|---|---|---|
//! | Cần tay nắm cửa sổ | `NSView` (**`unsafe`**) | `HWND` (an toàn) | **không cần gì** |
//! | Ai giữ tiêu điểm | adapter tự lo | adapter tự lo (`WM_SETFOCUS`) | **ta phải báo** |
//! | Gốc cây phải là gì | tuỳ ý | tuỳ ý | **phải là `Role::Window`** |
//! | Sự kiện sau khi cập nhật | `QueuedEvents::raise` | `QueuedEvents::raise` | tự gửi |
//!
//! Ba khác biệt ấy là toàn bộ nội dung tệp này. Chỗ nào một nền **không làm
//! được** thứ macOS làm được thì nói ra bằng chú thích, không lặng lẽ bỏ qua —
//! xem [`ScreenReaderLink::set_window_focused`] và [`wrap_in_window`].

use std::sync::{Arc, Mutex};

#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(target_os = "ios"),
    not(target_os = "android")
))]
use accesskit::DeactivationHandler;
use accesskit::{ActionHandler, ActionRequest, ActivationHandler, Node, NodeId, Role, TreeUpdate};

// Nền có đường AT-SPI: mọi Unix KHÔNG phải nhà Apple và không phải Android.
// Điều kiện này phải khớp TỪNG CHỮ với `[target.'cfg(...)'.dependencies]` của
// `Cargo.toml` — lệch một vế là `accesskit_unix` không có mặt trên đúng nền cần
// nó, và lỗi hiện ra dưới dạng "unresolved import" chứ không dưới dạng gì gợi ý
// tới đây. Rust không có bí danh `cfg` nếu không có `build.rs`, mà `build.rs`
// nằm ngoài `src/`, nên điều kiện này bị chép tay ở vài chỗ. Sửa một chỗ thì
// tìm hết.
//
// (`unix` trong `cfg` của Rust CÓ bao gồm macOS và iOS — nên phải loại tay.)

/// Tên đường trợ năng đang được biên dịch cho nền này.
///
/// Có mặt để phép thử và nhật ký nói được *đường nào* đang chạy. `"none"` nghĩa
/// là bản dựng này **không có trợ năng** — và đó là một sự thật phải nhìn thấy
/// được, không phải một chỗ im lặng.
pub const BACKEND: &str = backend::NAME;

/// Mã của nút gốc **giả** mà đường AT-SPI cần (xem [`wrap_in_window`]).
///
/// `u64::MAX` chứ không phải một số nhỏ: `to_accesskit_with_actions` phát mã
/// đếm lên từ 0, mỗi nút một số, và số nút bị chặn trên bởi chiều cao ảnh
/// (`MAX_HEIGHT` = 4096 điểm ảnh, mỗi nút chiếm ít nhất một dòng). Muốn chạm
/// tới `u64::MAX` thì phải vẽ nhiều hơn số nguyên tử trong vũ trụ — nên chỗ này
/// không va nhau được. Phép thử `ma_nut_cua_so_khong_va_ma_that` canh điều đó.
const WINDOW_NODE_ID: NodeId = NodeId(u64::MAX);

/// Bọc cây trong một nút gốc `Role::Window` — **chỉ đường AT-SPI cần**.
///
/// # Vì sao Linux khác hai nền kia
///
/// Trên macOS adapter gắn vào `NSView`, trên Windows nó gắn vào `HWND`: ở cả
/// hai chỗ, **hệ điều hành đã biết cửa sổ** và cây của ta chỉ là ruột của nó.
///
/// AT-SPI không có tay nắm cửa sổ nào cả — nó là một giao thức D-Bus, và thứ
/// duy nhất nó biết về cửa sổ là **vai trò của nút gốc**. `accesskit_atspi_common`
/// chỉ gửi `window:create` / `window:activate` khi gốc mang một trong ba vai trò
/// `Window`, `Dialog`, `AlertDialog` (`adapter.rs`, `fn root_window`). Gốc của
/// ta là `Role::GenericContainer` — nên **không có sự kiện nào được gửi**, và
/// Orca không có gì để đọc.
///
/// Nên ở đây thêm một tầng: một nút `Role::Window` mang tiêu đề cửa sổ, có đúng
/// một con là gốc cũ. Nút giả này **không có mã hành động**, nên bảng tra
/// `NodeId` → hành động trong `window.rs` không bị đụng tới; tiêu điểm vẫn trỏ
/// vào gốc cũ, vốn vẫn nằm trong cây.
///
/// Không làm việc này trên macOS/Windows: ở đó nó sẽ là **hai** cửa sổ lồng
/// nhau, và trình đọc màn hình sẽ đọc tên cửa sổ hai lần.
#[must_use]
fn wrap_in_window(mut cap_nhat: TreeUpdate, tieu_de: &str) -> TreeUpdate {
    // Cập nhật KHÔNG mang cây (bản vá một phần) thì gốc không đổi — đụng vào
    // là dựng một cây thứ hai đè lên cây đang chạy. Ta luôn gửi cây đầy đủ,
    // nhưng luật này thuộc về hàm chứ không thuộc về bên gọi.
    let Some(mut cay) = cap_nhat.tree else {
        return cap_nhat;
    };
    let goc_cu = cay.root;
    let mut cua_so = Node::new(Role::Window);
    cua_so.set_label(tieu_de.to_owned());
    cua_so.set_children(vec![goc_cu]);
    cap_nhat.nodes.push((WINDOW_NODE_ID, cua_so));
    cay.root = WINDOW_NODE_ID;
    // Orca in tên bộ công cụ khi gỡ lỗi. Nói thật tên ra thì một báo cáo lỗi từ
    // người dùng Linux chỉ thẳng vào bộ dựng này chứ không vào "một ứng dụng
    // GTK nào đó".
    cay.toolkit_name = Some("tcc-render-raster".to_owned());
    cap_nhat.tree = Some(cay);
    cap_nhat
}

/// Trả cây đầu tiên khi hệ điều hành hỏi tới.
///
/// # ⚠️ Phải là trạng thái CHIA SẺ, không phải một ảnh chụp
///
/// Đây là cùng cái bẫy `window.rs` đã dẫm phải và ghi lại (rà soát 21/08/2026,
/// F2), chép sang đây vì nó áp cho **cả ba** nền: `update_if_active` trả về
/// ngay và **không gọi hàm dựng cây** khi chưa có ai nghe. Giữ một ảnh chụp ở
/// đây nghĩa là mọi lần vẽ lại lúc trình đọc màn hình còn tắt đều bị vứt, rồi
/// `request_initial_tree` — gọi đúng một lần, lúc có người nghe — trả về cây
/// của **lần vẽ số 0**.
///
/// Trên màn hỏi quyền: người dùng gạt một quyền bằng chuột, rồi bật trình đọc
/// màn hình, và nghe **"tắt"** trong khi màn hình hiện **"bật"**.
struct InitialTree {
    /// Cây của lần vẽ gần nhất, do vòng lặp sự kiện ghi vào.
    shared: Arc<Mutex<TreeUpdate>>,
    /// `Some(tiêu đề)` → bọc thêm nút gốc `Role::Window`. Chỉ AT-SPI cần; xem
    /// [`wrap_in_window`].
    wrap_title: Option<String>,
}

impl ActivationHandler for InitialTree {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        // Khoá hỏng (một luồng khác đã hoảng loạn khi đang giữ) thì trả `None`.
        // AccessKit hiểu `None` là "chưa có cây, hỏi lại sau" — thà thế còn hơn
        // hoảng loạn tiếp trong một luồng của hệ điều hành.
        let cay = self.shared.lock().ok().map(|t| t.clone())?;
        Some(match &self.wrap_title {
            Some(tieu_de) => wrap_in_window(cay, tieu_de),
            None => cay,
        })
    }
}

/// Nhận yêu cầu **bấm** từ hệ điều hành, đẩy vào hàng đợi của vòng lặp sự kiện.
///
/// # Cùng một đường với chuột, không phải đường riêng
///
/// Yêu cầu chỉ được đẩy vào hàng đợi. Vòng lặp rút ra rồi cho chạy qua **đúng
/// `sau_cu_bam`** mà cú bấm chuột đi qua — nên không có luật nào của hộp thoại
/// áp cho chuột mà không áp cho trợ năng. Lý do đầy đủ (và một lập luận sai đã
/// hoãn việc này mất một ngày) nằm ở `window.rs`, `mod tro_nang`.
///
/// # Luồng nào gọi hàm này
///
/// Khác nhau theo nền, và đó là lý do hàng đợi phải là `Arc<Mutex<…>>` chứ
/// không phải một `RefCell`: trên macOS/Windows nó *có thể* là luồng cửa sổ,
/// trên Linux `accesskit_unix` nói rõ **luôn luôn là một luồng khác**.
struct ClickQueue(Arc<Mutex<Vec<(u64, bool)>>>);

impl ActionHandler for ClickQueue {
    fn do_action(&mut self, yeu_cau: ActionRequest) {
        // Nhận "bấm" và "đặt tiêu điểm", KHÔNG nhận gì khác — cuộn, đặt giá trị
        // và phần còn lại chưa được nghĩ tới, và im lặng bỏ qua đúng hơn là đoán.
        //
        // ⚠️ Hai hành động này đi kèm một cờ phân biệt, và vòng lặp sự kiện
        // PHẢI dùng nó: `Focus` chỉ được phép chạm tới Ô NHẬP.
        //
        // Nếu `Focus` cũng kích hoạt nút thì người dùng VoiceOver vừa DI CHUYỂN
        // tới "Cho phép" là đã cấp quyền — di chuyển tiêu điểm không phải một
        // câu trả lời, y như đóng cửa sổ không phải một câu trả lời.
        let la_tieu_diem = match yeu_cau.action {
            accesskit::Action::Click => false,
            accesskit::Action::Focus => true,
            _ => return,
        };
        if let Ok(mut q) = self.0.lock() {
            q.push((yeu_cau.target_node.0, la_tieu_diem));
        }
    }
}

/// Trình đọc màn hình vừa tắt đi. Ta **không có gì để thả**.
///
/// Bắt buộc phải có trên đường AT-SPI (`Adapter::new` đòi ba trình xử lý, macOS
/// và Windows chỉ đòi hai). Cây là trạng thái chia sẻ do vòng lặp sự kiện sở
/// hữu, không phải thứ dựng riêng cho một khách nghe — nên khi khách bỏ đi thì
/// không có gì phải dọn, và khi khách quay lại thì [`InitialTree`] đọc lại đúng
/// cây mới nhất.
///
/// Chỉ biên dịch trên nền AT-SPI, và cố ý: một kiểu không ai dựng trên macOS
/// hay Windows là một cảnh báo `dead_code` — tức là trình biên dịch tự nói ra
/// rằng hai nền kia không cần khái niệm này. Điều kiện `cfg` phải khớp nhánh
/// `backend` của Linux bên dưới.
#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(target_os = "ios"),
    not(target_os = "android")
))]
struct Deactivation;

#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(target_os = "ios"),
    not(target_os = "android")
))]
impl DeactivationHandler for Deactivation {
    fn deactivate_accessibility(&mut self) {}
}

/// Đường trợ năng đã nối vào cửa sổ.
///
/// ⚠️ **Dựng TRƯỚC khi cửa sổ được hiện hay nhận tiêu điểm lần đầu.** macOS nói
/// rõ điều đó trong tài liệu; Windows còn cứng rắn hơn — `SubclassingAdapter`
/// của nó **hoảng loạn** nếu cửa sổ đã hiện. `window.rs` dựng cửa sổ với
/// `.with_visible(false)` đúng vì luật này.
///
/// ⚠️ Chưa lần nào chạy với trình đọc màn hình thật trên Windows/Linux — xem
/// chú thích đầu tệp.
pub struct ScreenReaderLink(backend::Backend);

impl ScreenReaderLink {
    /// Nối vào cửa sổ.
    ///
    /// `initial` là cây **chia sẻ** — không phải ảnh chụp; `clicks` là hàng đợi
    /// yêu cầu bấm mà vòng lặp sự kiện rút ra mỗi lần vẽ.
    ///
    /// # Panics
    ///
    /// Trên Windows: nếu `window` **đã được hiện**. Đó là luật của
    /// `accesskit_windows::SubclassingAdapter::new`, không phải của ta, và nó
    /// đúng — nối muộn thì trình đọc màn hình đã hỏi xong và nhận câu "không có
    /// gì ở đây", rồi không hỏi lại.
    #[must_use]
    pub fn attach(
        window: &tao::window::Window,
        initial: Arc<Mutex<TreeUpdate>>,
        clicks: Arc<Mutex<Vec<(u64, bool)>>>,
    ) -> Self {
        Self(backend::Backend::attach(window, initial, clicks))
    }

    /// Đẩy cây mới sang trình đọc màn hình.
    ///
    /// Nhận `TreeUpdate` **đã dựng sẵn** chứ không nhận một bao đóng như
    /// `update_if_active` của AccessKit. Cố ý: bên gọi (`bao_tro_nang` trong
    /// `window.rs`) đằng nào cũng phải dựng cây trước, để ghi nó vào trạng thái
    /// chia sẻ — nên bao đóng ở đây chỉ hoãn được một phép sao chép, đổi lại là
    /// một chữ ký khác nhau ở mỗi nền.
    ///
    /// Sự kiện phát sinh được **gửi ngay tại đây**. Trên macOS/Windows AccessKit
    /// trả về một `QueuedEvents` mà bên gọi phải nhớ `raise()`; quên là cây đổi
    /// mà không ai được báo. Trên Linux không có kiểu ấy. Nuốt khác biệt đó vào
    /// đây để bên gọi không phải nhớ ba luật.
    pub fn publish(&mut self, update: TreeUpdate) {
        self.0.publish(update);
    }

    /// Báo cửa sổ vừa nhận / mất tiêu điểm.
    ///
    /// # Đây là chỗ Linux KHÔNG bằng hai nền kia
    ///
    /// macOS và Windows **tự biết**: adapter của chúng gắn vào `NSView`/`HWND`
    /// nên nó thấy `becomeFirstResponder` / `WM_SETFOCUS` mà không cần ai nói.
    /// Hai nền ấy cài hàm này thành lời báo thẳng cho adapter, và gọi hay không
    /// gọi đều không hỏng.
    ///
    /// AT-SPI thì **không thấy gì cả** — nó không có tay nắm cửa sổ. Nếu
    /// `window.rs` không chuyển tiếp `WindowEvent::Focused` vào đây thì Orca
    /// không nhận được `window:activate`, và người dùng chuyển qua chuyển lại
    /// giữa các cửa sổ sẽ nghe sai cửa sổ.
    ///
    /// [`Self::attach`] đặt sẵn "đang có tiêu điểm" trên đường AT-SPI, vì màn
    /// hình raster là một hộp thoại được hiện ngay sau khi nối và nó cướp tiêu
    /// điểm. Đó là một phép **xấp xỉ đúng ở giây đầu tiên**, không phải một lời
    /// giải: chừng nào `window.rs` chưa gọi hàm này thì Linux vẫn tin rằng cửa
    /// sổ luôn có tiêu điểm.
    pub fn set_window_focused(&mut self, focused: bool) {
        self.0.set_window_focused(focused);
    }
}

// ───────────────────────── macOS ─────────────────────────

#[cfg(target_os = "macos")]
mod backend {
    use std::sync::{Arc, Mutex};

    use accesskit::TreeUpdate;

    use super::{ClickQueue, InitialTree};

    pub(super) const NAME: &str = "macos";

    pub(super) struct Backend(accesskit_macos::SubclassingAdapter);

    impl Backend {
        pub(super) fn attach(
            window: &tao::window::Window,
            initial: Arc<Mutex<TreeUpdate>>,
            clicks: Arc<Mutex<Vec<(u64, bool)>>>,
        ) -> Self {
            use tao::platform::macos::WindowExtMacOS as _;

            // ⚠️ `unsafe` — cùng một chỗ, cùng một lý do như `window.rs`.
            //
            // `Cargo.toml` đặt `unsafe_code = "deny"` toàn workspace và
            // `SECURITY.md` §3.1b đã lường trước đúng đánh đổi này. Không có
            // đường vòng: trao con trỏ `NSView` cho một API của hệ điều hành
            // thì không có bọc an toàn nào, và AccessKit không có bản nào cho
            // `tao`.
            //
            // Chú ý cái KHÔNG có ở hai nền kia: Windows nhận `HWND` — một con
            // số — nên hàm dựng của nó là hàm **an toàn**; Linux không nhận tay
            // nắm nào cả. macOS là nền DUY NHẤT phải trả giá này.
            //
            // Dùng `expect` chứ không `allow`: ngày nào có bọc an toàn, lint tự
            // báo rằng ngoại lệ này thừa, thay vì nằm lại mãi.
            //
            // SAFETY: `ns_view` của `tao` trả về NSView của cửa sổ vừa dựng
            // xong. Bên gọi giữ `window` sống lâu hơn giá trị trả về — vòng lặp
            // sự kiện thả adapter trước khi thả cửa sổ — nên con trỏ còn hợp lệ
            // suốt đời adapter.
            #[expect(
                unsafe_code,
                reason = "trao con trỏ NSView cho AccessKit — không có bọc an toàn nào cho tao"
            )]
            let adapter = unsafe {
                accesskit_macos::SubclassingAdapter::new(
                    window.ns_view(),
                    InitialTree {
                        shared: initial,
                        // macOS đã có cửa sổ thật; bọc thêm một nút `Window` ở
                        // đây là để VoiceOver đọc tên cửa sổ hai lần.
                        wrap_title: None,
                    },
                    ClickQueue(clicks),
                )
            };
            Self(adapter)
        }

        pub(super) fn publish(&mut self, update: TreeUpdate) {
            if let Some(su_kien) = self.0.update_if_active(|| update) {
                su_kien.raise();
            }
        }

        pub(super) fn set_window_focused(&mut self, focused: bool) {
            // Không bắt buộc: adapter gắn vào NSView nên nó tự thấy. Cài đặt
            // vẫn có mặt để chữ ký giống nhau ở cả ba nền.
            if let Some(su_kien) = self.0.update_view_focus_state(focused) {
                su_kien.raise();
            }
        }
    }
}

// ───────────────────────── Windows ─────────────────────────

#[cfg(target_os = "windows")]
mod backend {
    use std::sync::{Arc, Mutex};

    use accesskit::TreeUpdate;

    use super::{ClickQueue, InitialTree};

    pub(super) const NAME: &str = "windows";

    pub(super) struct Backend(accesskit_windows::SubclassingAdapter);

    impl Backend {
        pub(super) fn attach(
            window: &tao::window::Window,
            initial: Arc<Mutex<TreeUpdate>>,
            clicks: Arc<Mutex<Vec<(u64, bool)>>>,
        ) -> Self {
            use tao::platform::windows::WindowExtWindows as _;

            // ⚠️ KHÔNG có `unsafe` ở đây, và đó là một khác biệt thật chứ không
            // phải một chỗ tôi quên.
            //
            // `tao` trả `HWND` dưới dạng `isize` — một con số, không phải một
            // con trỏ đang sống. `accesskit_windows::HWND` là một `struct` bọc
            // `*mut c_void`, và **dựng một con trỏ thô là việc an toàn** trong
            // Rust; chỉ *đọc qua* nó mới cần `unsafe`, và phần đó nằm bên trong
            // AccessKit chứ không nằm ở đây.
            //
            // Nên đường Windows không cần một ngoại lệ `unsafe_code` nào. Nếu
            // sau này có ai thêm một khối `unsafe` vào đây, đó là dấu hiệu họ
            // đang tự đọc bộ nhớ của cửa sổ — và việc ấy phải được hỏi lại.
            let hwnd = accesskit_windows::HWND(window.hwnd() as *mut core::ffi::c_void);
            Self(accesskit_windows::SubclassingAdapter::new(
                hwnd,
                InitialTree {
                    shared: initial,
                    // Windows đã có cửa sổ thật (`HWND`) — như macOS, và khác
                    // Linux. Xem `wrap_in_window`.
                    wrap_title: None,
                },
                ClickQueue(clicks),
            ))
        }

        pub(super) fn publish(&mut self, update: TreeUpdate) {
            if let Some(su_kien) = self.0.update_if_active(|| update) {
                su_kien.raise();
            }
        }

        // `&mut self` không dùng tới, và clippy nói đúng. Giữ nguyên chữ ký
        // chứ không đổi thành hàm rời: ba nền phải gọi được như nhau, còn một
        // nền có chữ ký khác là một nền bên gọi phải nhớ riêng — và sẽ quên.
        #[expect(
            clippy::unused_self,
            reason = "chữ ký phải giống ba nền; Windows tự biết tiêu điểm"
        )]
        pub(super) fn set_window_focused(&mut self, focused: bool) {
            // `SubclassingAdapter` của Windows bắt `WM_SETFOCUS`/`WM_KILLFOCUS`
            // ngay trong thủ tục cửa sổ mà nó cài (`subclass.rs`, `wnd_proc`),
            // nên nó đã biết trước khi ta kịp nói. Không có API nào để nói
            // thêm — và cũng không cần.
            //
            // Đây là chỗ Windows KHÔNG bằng macOS: macOS cho ép trạng thái tiêu
            // điểm bằng tay, Windows thì không. Với ta điều đó không mất gì, vì
            // ta chỉ có một cửa sổ và không tự vẽ ô nhập giả nào.
            let _ = focused;
        }
    }
}

// ───────────────────────── Linux / AT-SPI ─────────────────────────

// Điều kiện phải khớp `[target.'cfg(...)'.dependencies]` trong `Cargo.toml`.
#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(target_os = "ios"),
    not(target_os = "android")
))]
mod backend {
    use std::sync::{Arc, Mutex};

    use accesskit::TreeUpdate;

    use super::{ClickQueue, Deactivation, InitialTree, wrap_in_window};

    pub(super) const NAME: &str = "atspi";

    pub(super) struct Backend {
        adapter: accesskit_unix::Adapter,
        /// Tiêu đề cửa sổ, giữ lại để bọc mỗi lần đẩy cây — xem
        /// [`wrap_in_window`]. `tao` cho đọc lại tiêu đề bất cứ lúc nào, nhưng
        /// `publish` chạy trong vòng vẽ và không được cầm `&Window`.
        title: String,
    }

    impl Backend {
        pub(super) fn attach(
            window: &tao::window::Window,
            initial: Arc<Mutex<TreeUpdate>>,
            clicks: Arc<Mutex<Vec<(u64, bool)>>>,
        ) -> Self {
            // ⚠️ KHÔNG có tay nắm cửa sổ, và cũng KHÔNG có `unsafe`.
            //
            // AT-SPI là một giao thức D-Bus: `accesskit_unix` tự đăng ký ứng
            // dụng với bus, không cần `HWND`, không cần `GdkWindow`, không đụng
            // tới GTK. `tao::platform::unix::WindowExtUnix::gtk_window()` có
            // tồn tại — nhưng dùng nó ở đây là kéo cả `gtk` vào crate này mà
            // không đổi lại được gì.
            //
            // Cái giá của việc không có tay nắm cửa sổ nằm ở hai chỗ khác:
            // `wrap_in_window` (gốc cây phải tự khai mình là cửa sổ) và
            // `set_window_focused` (không ai báo tiêu điểm hộ ta).
            let title = window.title();
            let mut adapter = accesskit_unix::Adapter::new(
                InitialTree {
                    shared: initial,
                    wrap_title: Some(title.clone()),
                },
                ClickQueue(clicks),
                Deactivation,
            );
            // Màn hình raster là một hộp thoại được hiện NGAY sau khi nối, và
            // nó cướp tiêu điểm. Không đặt sẵn thì `is_window_focused` là
            // `false` lúc Orca kết nối, và không có `window:activate` nào được
            // gửi — Orca im lặng.
            //
            // Xấp xỉ, không phải lời giải: xem `ScreenReaderLink::set_window_focused`.
            adapter.update_window_focus_state(true);
            Self { adapter, title }
        }

        pub(super) fn publish(&mut self, update: TreeUpdate) {
            // Mượn tách trường: bao đóng cầm `&self.title` trong khi
            // `self.adapter` đang bị mượn `&mut`.
            let Self { adapter, title } = self;
            // Không có `QueuedEvents` để `raise()` — đường AT-SPI tự gửi sự
            // kiện qua D-Bus. Một khác biệt nữa mà `publish` nuốt hộ bên gọi.
            adapter.update_if_active(|| wrap_in_window(update, title));
        }

        pub(super) fn set_window_focused(&mut self, focused: bool) {
            self.adapter.update_window_focus_state(focused);
        }
    }
}

// ───────────────────────── Nền chưa có đường nào ─────────────────────────

// Phủ nốt phần còn lại (wasm, Android, iOS…). KHÔNG phải để cho đẹp: thiếu nó
// thì bật cờ `accesskit-platform` trên một nền lạ là một lỗi biên dịch nằm ở
// `window.rs` — tệp mà người sửa lỗi ấy không sở hữu.
#[cfg(not(any(
    target_os = "macos",
    target_os = "windows",
    all(
        unix,
        not(target_os = "macos"),
        not(target_os = "ios"),
        not(target_os = "android")
    )
)))]
mod backend {
    use std::sync::{Arc, Mutex};

    use accesskit::TreeUpdate;

    use super::{ClickQueue, InitialTree};

    /// `"none"` — và hằng số này ĐỌC ĐƯỢC ra ngoài đúng vì thế. Một bản dựng
    /// không có trợ năng phải nói ra rằng nó không có, chứ không im lặng giả vờ
    /// như đã nối.
    pub(super) const NAME: &str = "none";

    pub(super) struct Backend {
        /// Giữ hai trình xử lý sống đúng bằng đời adapter, y như ba nền kia —
        /// dù ở đây không có ai gọi chúng.
        ///
        /// Hai lý do, và lý do thứ hai mới là lý do thật. Một: `Arc` tới cây
        /// chia sẻ và tới hàng đợi bấm được giữ y hệt, nên không có nền nào thả
        /// sớm hơn nền khác. Hai: dựng chúng ở đây làm `InitialTree` thành một
        /// kiểu CÓ người dựng trên MỌI nền — mà `InitialTree` là chỗ duy nhất
        /// gọi `wrap_in_window`. Không dựng thì cả nhánh AT-SPI hoá "mã chết"
        /// trong con mắt trình biên dịch, và một cảnh báo `dead_code` giả sẽ
        /// bắt người sau tắt lint ở đúng chỗ không nên tắt.
        ///
        /// Tên bắt đầu bằng `_`: nói với trình biên dịch rằng "giữ, không đọc"
        /// là CÓ CHỦ Ý.
        _handlers: (InitialTree, ClickQueue),
    }

    impl Backend {
        pub(super) fn attach(
            window: &tao::window::Window,
            initial: Arc<Mutex<TreeUpdate>>,
            clicks: Arc<Mutex<Vec<(u64, bool)>>>,
        ) -> Self {
            let _ = window;
            Self {
                _handlers: (
                    InitialTree {
                        shared: initial,
                        wrap_title: None,
                    },
                    ClickQueue(clicks),
                ),
            }
        }

        // `allow` chứ không `expect`: nhánh này KHÔNG biên dịch được trên ba
        // nền ta có, nên không ai xác nhận được lint có nổ hay không — và một
        // `expect` không nổ là lỗi biên dịch trên đúng nền không ai ngồi trước.
        // Đây là chỗ mà một ngoại lệ chặt hơn lại giòn hơn.
        #[allow(
            clippy::unused_self,
            clippy::needless_pass_by_value,
            reason = "chữ ký phải giống ba nền kia; nền này không có gì để làm"
        )]
        pub(super) fn publish(&mut self, update: TreeUpdate) {
            let _ = update;
        }

        #[allow(
            clippy::unused_self,
            reason = "chữ ký phải giống ba nền kia; nền này không có gì để làm"
        )]
        pub(super) fn set_window_focused(&mut self, focused: bool) {
            let _ = focused;
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay"
)]
mod kiem_thu {
    use tcc_ui::{AccessNode, Role as UiRole};

    use super::{BACKEND, WINDOW_NODE_ID, wrap_in_window};
    use crate::accesskit_bridge::{AccessText, to_accesskit, to_accesskit_with_actions};

    fn cay_thu() -> accesskit::TreeUpdate {
        let goc = AccessNode {
            role: UiRole::Group,
            label: None,
            action: None,
            children: vec![
                AccessNode {
                    role: UiRole::Text,
                    label: Some("Ứng dụng muốn dùng micro".to_owned()),
                    action: None,
                    children: Vec::new(),
                },
                AccessNode {
                    role: UiRole::Button { destructive: false },
                    label: Some("Cho phép".to_owned()),
                    action: Some("cho-phep".to_owned()),
                    children: Vec::new(),
                },
            ],
        };
        to_accesskit(&goc, &AccessText::default())
    }

    /// **Gốc cây phải tự khai mình là cửa sổ, nếu không Orca im lặng.**
    ///
    /// `accesskit_atspi_common` chỉ gửi `window:create`/`window:activate` khi
    /// gốc mang vai trò `Window`/`Dialog`/`AlertDialog`. Gốc của ta là
    /// `GenericContainer`. Phép thử này là chỗ duy nhất canh được điều đó từ
    /// máy macOS — nó thuần logic, không cần D-Bus.
    #[test]
    fn goc_duoc_boc_thanh_cua_so() {
        let truoc = cay_thu();
        let goc_cu = truoc.tree.clone().unwrap().root;
        let sau = wrap_in_window(truoc.clone(), "Quyền truy cập");

        let cay = sau.tree.clone().unwrap();
        assert_eq!(cay.root, WINDOW_NODE_ID, "gốc chưa đổi sang nút cửa sổ");

        let (_, nut) = sau
            .nodes
            .iter()
            .find(|(i, _)| *i == WINDOW_NODE_ID)
            .expect("không thêm nút cửa sổ nào");
        assert_eq!(nut.role(), accesskit::Role::Window);
        assert_eq!(nut.label(), Some("Quyền truy cập"));
        assert_eq!(
            nut.children(),
            [goc_cu],
            "nút cửa sổ phải có đúng một con là gốc cũ"
        );
        assert_eq!(
            sau.nodes.len(),
            truoc.nodes.len() + 1,
            "bọc cửa sổ mà làm rụng hoặc nhân đôi nút khác"
        );
    }

    /// Tiêu điểm KHÔNG được đổi sang nút cửa sổ giả.
    ///
    /// Nút ấy không bấm được và không mang chữ nào của ứng dụng. Trỏ tiêu điểm
    /// vào nó nghĩa là người dùng Orca mở hộp thoại lên và nghe tên cửa sổ thay
    /// vì nghe câu hỏi.
    #[test]
    fn tieu_diem_o_lai_trong_noi_dung() {
        let truoc = cay_thu();
        let tieu_diem_cu = truoc.focus;
        let sau = wrap_in_window(truoc, "Quyền truy cập");
        assert_eq!(sau.focus, tieu_diem_cu);
        assert_ne!(sau.focus, WINDOW_NODE_ID);
    }

    /// **Mã nút cửa sổ không được va vào mã thật.**
    ///
    /// Va nhau thì AccessKit dựng sai cây, và trên màn hỏi quyền "sai cây"
    /// nghĩa là một cú bấm rơi vào nút khác.
    #[test]
    fn ma_nut_cua_so_khong_va_ma_that() {
        let cay = cay_thu();
        assert!(
            cay.nodes.iter().all(|(i, _)| *i != WINDOW_NODE_ID),
            "cây thật đã dùng tới u64::MAX"
        );
        let sau = wrap_in_window(cay, "x");
        let mut ma: Vec<_> = sau.nodes.iter().map(|(i, _)| i.0).collect();
        ma.sort_unstable();
        ma.dedup();
        assert_eq!(
            ma.len(),
            sau.nodes.len(),
            "có NodeId trùng nhau sau khi bọc"
        );
    }

    /// Bọc cửa sổ KHÔNG được đụng vào bảng tra `NodeId` → hành động.
    ///
    /// Bảng ấy do `window.rs` dựng từ cây CHƯA bọc. Nếu bọc mà đánh số lại thì
    /// một yêu cầu bấm từ Orca tra ra hành động của nút khác — và không có
    /// đường lùi sau một nút không hoàn tác.
    #[test]
    fn bang_hanh_dong_khong_lech_sau_khi_boc() {
        let goc = AccessNode {
            role: UiRole::Group,
            label: None,
            action: None,
            children: vec![AccessNode {
                role: UiRole::Button { destructive: true },
                label: Some("Xoá ví".to_owned()),
                action: Some("xoa-vi".to_owned()),
                children: Vec::new(),
            }],
        };
        let (cay, bang) = to_accesskit_with_actions(&goc, &AccessText::default());
        let sau = wrap_in_window(cay, "Ví");
        for ma in bang.keys() {
            assert!(
                sau.nodes.iter().any(|(i, _)| i.0 == *ma),
                "mã hành động {ma} không còn nút nào mang nó sau khi bọc"
            );
        }
    }

    /// Cập nhật KHÔNG mang cây thì để nguyên — đụng vào là dựng cây thứ hai.
    #[test]
    fn cap_nhat_mot_phan_khong_bi_boc() {
        let mut u = cay_thu();
        u.tree = None;
        let n = u.nodes.len();
        let sau = wrap_in_window(u, "Quyền truy cập");
        assert!(sau.tree.is_none());
        assert_eq!(sau.nodes.len(), n, "đã thêm nút cửa sổ vào một bản vá");
    }

    /// Bản dựng này nối vào đường nào — và nó KHÔNG được là `"none"` trên ba
    /// nền ta tuyên bố có trợ năng.
    #[test]
    fn co_duong_tro_nang_tren_ba_nen() {
        if cfg!(any(
            target_os = "macos",
            target_os = "windows",
            all(
                unix,
                not(target_os = "macos"),
                not(target_os = "ios"),
                not(target_os = "android")
            )
        )) {
            assert_ne!(BACKEND, "none", "nền này lẽ ra phải có đường trợ năng");
        }
        #[cfg(target_os = "macos")]
        assert_eq!(BACKEND, "macos");
        #[cfg(target_os = "windows")]
        assert_eq!(BACKEND, "windows");
        #[cfg(all(
            unix,
            not(target_os = "macos"),
            not(target_os = "ios"),
            not(target_os = "android")
        ))]
        assert_eq!(BACKEND, "atspi");
    }
}
