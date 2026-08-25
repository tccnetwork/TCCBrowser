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

use std::collections::{BTreeMap, BTreeSet};
#[cfg(feature = "accesskit-platform")]
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

/// Một màn hình trong chuỗi.
///
/// Sở hữu dữ liệu chứ không mượn: màn hình tiếp theo thường được dựng RA TỪ câu
/// trả lời của màn hình trước, nên nó không mượn được của ai cả.
pub struct Screen {
    pub tree: Node,
    pub title: String,
    pub text: ScreenText,
}

/// Bên gọi quyết định đi tiếp hay dừng, sau mỗi màn hình.
pub enum Next {
    Show(Box<Screen>),
    Done,
}

/// Nhiều màn hình nối nhau **trong MỘT cửa sổ và MỘT vòng lặp sự kiện**.
///
/// # ⚠️ Vì sao không gọi [`open_screen`] nhiều lần
///
/// `tao` chỉ cho dựng **một** vòng lặp sự kiện mỗi tiến trình. Gọi lần thứ hai
/// làm nó hoảng loạn trong `app_state.rs`, và thông báo hoảng loạn ấy không nói
/// gì về nguyên nhân — nhìn từ ngoài chỉ thấy chương trình treo. Đường WebView
/// đã vấp đúng chỗ này và giải bằng `dialog_sequence`; đây là cùng lời giải, cho
/// bộ dựng ra pixel.
///
/// Trả về kết quả của **từng** màn hình, theo thứ tự. Bên gọi cần cả chuỗi chứ
/// không chỉ màn cuối: luồng ví đọc lại ô nhập của màn trước.
///
/// Đóng cửa sổ thì dừng cả chuỗi và KHÔNG hỏi `tiep` — đóng cửa sổ không phải
/// một câu trả lời.
///
/// # Errors
/// Cây không vẽ được, hoặc không dựng được cửa sổ.
pub fn open_sequence(
    dau: Screen,
    tiep: impl FnMut(&ScreenOutcome) -> Next,
) -> Result<Vec<ScreenOutcome>, String> {
    chay_chuoi(dau, tiep)
}

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
///
/// Là chuỗi MỘT phần tử — xem [`open_sequence`].
pub fn open_screen(tree: &Node, tieu_de: &str, chu: &ScreenText) -> Result<ScreenOutcome, String> {
    let man = Screen {
        tree: tree.clone(),
        title: tieu_de.to_owned(),
        text: ScreenText {
            destructive_note: chu.destructive_note.clone(),
            destructive_role: chu.destructive_role.clone(),
        },
    };
    let mut ket = open_sequence(man, |_| Next::Done)?;
    // Chuỗi một phần tử luôn trả đúng một kết quả; nhưng nếu không, trả về một
    // kết quả RỖNG chứ không hoảng loạn — rỗng nghĩa là "không bấm gì", và đó là
    // câu trả lời an toàn.
    Ok(ket.pop().unwrap_or(ScreenOutcome {
        action: None,
        toggles_on: BTreeSet::new(),
        fields: BTreeMap::new(),
    }))
}

/// Trạng thái của MỘT phiên màn hình.
///
/// # Vì sao gói lại
///
/// `chay_chuoi` từng giữ tám biến cục bộ mà sáu nhánh sự kiện cùng đọc và cùng
/// sửa. Tách bằng cách moi từng nhánh ra hàm rời đã thử và tệ hơn: mỗi hàm phải
/// nhận bảy tám tham số, trong đó vài cái nằm sau `#[cfg]`.
///
/// Ranh giới ở đây là **cái gì đổi theo màn hình**. Cửa sổ, bề mặt và cầu trợ
/// năng sống suốt cả chuỗi nên chúng ở ngoài; cây, bộ dựng, công tắc, ô nhập,
/// nút đã bấm thì đổi mỗi lần sang màn mới nên chúng vào đây — và `doi` đặt lại
/// đúng một chỗ, thay vì bốn dòng gán nằm rải trong vòng lặp.
struct Phien {
    bo_dung: RasterRenderer,
    man: Screen,
    /// Công tắc đang BẬT. Bắt đầu RỖNG mỗi màn hình, không đọc trạng thái ban
    /// đầu của cây: mặc định của một câu hỏi chưa trả lời là "không", và một hộp
    /// thoại quyền mở ra với sẵn vài mục bật là một hộp thoại đã tự trả lời hộ
    /// người dùng.
    bat: BTreeSet<String>,
    tt: TrangThai,
    da_bam: Option<String>,
    ve_lai: bool,
}

impl Phien {
    fn moi(man: Screen) -> Result<Self, String> {
        let mut bo_dung = RasterRenderer::new();
        bo_dung
            .render(
                &man.tree
                    .with_toggles(&BTreeSet::new())
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
        Ok(Self {
            bo_dung,
            man,
            bat: BTreeSet::new(),
            tt: TrangThai::default(),
            da_bam: None,
            ve_lai: false,
        })
    }

    /// Chữ từ bộ gõ của hệ điều hành, vào ô đang được chọn.
    ///
    /// Chưa chọn ô nào thì chữ **rơi đi**, không vào đâu cả: thà không nhận còn
    /// hơn nhận vào một ô người dùng không nhìn.
    fn nhan_chu(&mut self, chu: &str) {
        let Some(nhan) = self.tt.o_dang_chon.clone() else {
            return;
        };
        let o = self.tt.noi_dung_o.entry(nhan).or_default();
        // ⚠️ Kiểm TRƯỚC khi nhận, không phải lúc vẽ.
        //
        // Chữ về từ bộ gõ của hệ điều hành, và nó KHÔNG đi qua `Node::field` —
        // `with_fields` dựng lại cây từ chuỗi này. Nhận trước rồi mới phát hiện
        // lúc vẽ thì đường duy nhất còn lại là nuốt lỗi, và màn hình đứng im
        // trong khi người dùng vẫn gõ: họ gõ, không thấy gì đổi, gõ tiếp.
        //
        // Từ chối cả cụm vừa nhận chứ không cắt bớt: cắt một chuỗi có dấu là cắt
        // vào giữa một ký tự, và cắt im lặng là đổi thứ người dùng gõ.
        let thu = chen_tai(o, self.tt.con_tro.min(o.chars().count()), chu);
        if tcc_ui::check_field_value(&thu).is_err() {
            return;
        }
        *o = thu;
        self.tt.con_tro += chu.chars().count();
        self.ve_lai = true;
    }

    /// Xoá theo KÝ TỰ, không theo byte: một chữ có dấu là nhiều byte, và cắt
    /// byte là cắt vào giữa một ký tự.
    fn xoa_lui(&mut self) {
        let Some(nhan) = self.tt.o_dang_chon.clone() else {
            return;
        };
        let i = self.tt.con_tro;
        if i == 0 {
            return;
        }
        let Some(v) = self.tt.noi_dung_o.get_mut(&nhan) else {
            return;
        };
        // ⚠️ Kiểm cả khi XOÁ, dù xoá chỉ rút ngắn: `check_field_value` xét cả
        // hình dạng chuỗi, không chỉ độ dài.
        let thu = xoa_truoc(v, i);
        if tcc_ui::check_field_value(&thu).is_err() {
            return;
        }
        *v = thu;
        self.tt.con_tro = i - 1;
        self.ve_lai = true;
    }

    /// Phím `Delete`: xoá chữ NGAY TẠI con trỏ, con trỏ đứng yên.
    fn xoa_tai_cho(&mut self) {
        let Some(nhan) = self.tt.o_dang_chon.clone() else {
            return;
        };
        let i = self.tt.con_tro;
        let Some(v) = self.tt.noi_dung_o.get_mut(&nhan) else {
            return;
        };
        if i >= v.chars().count() {
            return;
        }
        let thu = xoa_tai(v, i);
        if tcc_ui::check_field_value(&thu).is_err() {
            return;
        }
        *v = thu;
        self.ve_lai = true;
    }

    /// Dời con trỏ. `sang_phai = false` là sang trái.
    fn doi_con_tro(&mut self, sang_phai: bool) {
        if self.tt.o_dang_chon.is_none() {
            return;
        }
        self.tt.kep_con_tro();
        let dai = self
            .tt
            .o_dang_chon
            .as_ref()
            .and_then(|n| self.tt.noi_dung_o.get(n))
            .map_or(0, |v| v.chars().count());
        self.tt.con_tro = if sang_phai {
            (self.tt.con_tro + 1).min(dai)
        } else {
            self.tt.con_tro.saturating_sub(1)
        };
        self.ve_lai = true;
    }

    /// `Home` / `End`.
    fn con_tro_ve_dau_hoac_cuoi(&mut self, ve_cuoi: bool) {
        if self.tt.o_dang_chon.is_none() {
            return;
        }
        let dai = self
            .tt
            .o_dang_chon
            .as_ref()
            .and_then(|n| self.tt.noi_dung_o.get(n))
            .map_or(0, |v| v.chars().count());
        self.tt.con_tro = if ve_cuoi { dai } else { 0 };
        self.ve_lai = true;
    }

    /// Lăn chuột — cuộn nội dung, KẸP ở cả hai đầu.
    ///
    /// ⚠️ Không có cuộn thì một cây cao hơn cửa sổ đẩy nút Cho phép / Từ chối ra
    /// ngoài tầm, và người dùng **không bấm được gì cả**. Rà soát 21/08/2026,
    /// F7 — tôi xếp nó là "chuyện dùng được, không phải an ninh", rồi hôm sau nó
    /// chặn đúng một người thật.
    fn cuon(&mut self, delta: &tao::event::MouseScrollDelta, window: &tao::window::Window) {
        let co = window.inner_size().to_logical::<f64>(window.scale_factor());
        #[expect(clippy::cast_precision_loss, reason = "chiều cao ảnh, luôn nhỏ")]
        let cao_anh = self.bo_dung.height() as f64;
        self.tt.cuon = cuon_moi(
            self.tt.cuon,
            buoc_lan(delta, window.scale_factor()),
            cao_anh,
            co.height,
        );
        window.request_redraw();
    }

    /// Vị trí chuột, đổi sang đơn vị LOGIC.
    ///
    /// ⚠️ `tao` báo theo pixel VẬT LÝ, mà bộ dựng làm việc theo đơn vị logic —
    /// trên màn hình Retina hai thứ ấy lệch nhau đúng hệ số 2, và bấm sẽ trúng ô
    /// khác.
    fn chuot_toi(
        &mut self,
        vi_tri: &tao::dpi::PhysicalPosition<f64>,
        window: &tao::window::Window,
    ) {
        let l = vi_tri.to_logical::<f64>(window.scale_factor());
        self.tt.chuot = (l.x, l.y);

        // Vẽ lại CHỈ khi đích dưới chuột ĐỔI. Rê chuột sinh hàng chục sự kiện
        // một giây; xếp lại bố cục ở mỗi cái là cửa sổ giật trong khi không có
        // gì trên màn hình thay đổi.
        #[expect(
            clippy::cast_possible_truncation,
            reason = "toạ độ màn hình, luôn nằm gọn trong f32"
        )]
        let x = l.x as f32;
        let y = cong_cuon(l.y, self.tt.cuon);
        let moi = self.bo_dung.hit_test_field(x, y).map_or_else(
            || {
                self.bo_dung
                    .hit_test(x, y)
                    .map(|h| crate::FocusTarget::Action {
                        id: h.action.to_owned(),
                        toggle: h.toggle,
                    })
            },
            |nhan| Some(crate::FocusTarget::Field(nhan.to_owned())),
        );
        if moi != self.tt.duoi_chuot {
            self.tt.duoi_chuot = moi;
            self.ve_lai = true;
            window.request_redraw();
        }
    }

    /// Kết quả của màn hình vừa xong, và **lấy hẳn ra**.
    ///
    /// Lấy hẳn chứ không chép: màn hình sau bắt đầu từ trạng thái rỗng, và để
    /// lại một bản sao là để lại đường cho công tắc của màn cũ trả lời hộ màn
    /// mới.
    fn ket_man(&mut self) -> ScreenOutcome {
        ScreenOutcome {
            action: self.da_bam.take(),
            toggles_on: core::mem::take(&mut self.bat),
            fields: core::mem::take(&mut self.tt.noi_dung_o),
        }
    }

    /// Sang màn hình mới: vẽ lại, đổi tiêu đề, đặt lại chiều cao cửa sổ.
    fn doi(&mut self, man: Screen, window: &tao::window::Window) -> Result<(), String> {
        self.man = man;
        self.tt = TrangThai::default();
        // ⚠️ Xoá tiêu điểm TRONG BỘ DỰNG nữa, không chỉ trong trạng thái.
        //
        // `TrangThai::default()` nói "không ai đang được chọn", nhưng bộ dựng
        // giữ bản sao riêng để vẽ. Không xoá thì màn mới vẽ ra một viền quanh
        // một ô mà trạng thái khẳng định là không ai chọn — và nếu màn mới tình
        // cờ có cùng mã hành động, viền ấy rơi vào một nút người dùng chưa hề
        // chạm tới. Cùng hạng lỗi với `hang_tro_nang` không được dọn khi đổi
        // màn (F3).
        self.bo_dung.set_focus(None);
        self.bo_dung.set_hover(None);
        self.bo_dung
            .render(&self.man.tree)
            .map_err(|e| e.to_string())?;
        window.set_title(&self.man.title);
        // ⚠️ Đặt lại chiều cao theo màn mới. Không đặt thì một màn ngắn nối sau
        // màn dài để lại khoảng trắng lớn, còn màn dài nối sau màn ngắn thì phần
        // dưới **không nhìn thấy được** — kể cả nút.
        //
        // Cùng một trần với `dung_cua_so`, và vì cùng lý do: cây cao tới
        // `MAX_HEIGHT` mà màn hình thì không.
        let co = window.inner_size().to_logical::<f64>(window.scale_factor());
        let cao_man = window.current_monitor().map_or(900.0, |m| {
            m.size().to_logical::<f64>(m.scale_factor()).height
        });
        #[expect(clippy::cast_precision_loss, reason = "chiều cao ảnh, luôn nhỏ")]
        let cao = (self.bo_dung.height() as f64).min(cao_man * 0.85);
        window.set_inner_size(LogicalSize::new(co.width, cao));
        Ok(())
    }
}

// ⚠️ Hàm này vẫn dài quá trần 100, và đây là lý do — đọc trước khi "dọn" nó.
//
// Đã từng 169 dòng với tám biến cục bộ mà sáu nhánh sự kiện cùng đọc và cùng
// sửa. Nay 134, và tám biến ấy thành `Phien` — một kiểu có tên, có phương thức,
// mỗi phương thức mang theo lý do của nó. Đó là phần đáng làm của món nợ, và nó
// đã trả.
//
// Phần còn lại KHÔNG cắt tiếp, và không phải vì ngại:
//
// - Cửa sổ, `Context`, `Surface` và cầu trợ năng **mượn lẫn nhau** — `Surface`
//   mượn `Context`, `Context` mượn cửa sổ. Moi khối dựng ấy ra một hàm là dựng
//   một kiểu tự tham chiếu, và cái giá ấy đắt hơn ba mươi dòng.
// - Cắt tiếp cái `match` thì mỗi mảnh phải nhận thêm `adapter`, `cay_chung`,
//   `bang_hanh_dong` — ba thứ nằm sau `#[cfg]`. Trải một máy trạng thái ra sáu
//   hàm bảy tham số KHÔNG làm nó dễ đọc hơn, chỉ làm chỗ nối khó thấy hơn.
//
// Nói cách khác: chỗ nào cắt làm mã rõ hơn thì đã cắt; chỗ còn lại cắt chỉ để
// một con số qua ngưỡng. Ngưỡng là công cụ, không phải mục tiêu.
thread_local! {
    /// Vòng lặp sự kiện của CẢ TIẾN TRÌNH, dựng một lần và dùng lại.
    ///
    /// # Vì sao phải là một, và vì sao phải nằm ở đây
    ///
    /// `tao` chỉ cho dựng **một** vòng lặp mỗi tiến trình. Dựng lần thứ hai thì
    /// nó không báo lỗi — nó **abort**, với một thông báo không nói gì về nguyên
    /// nhân (`app_state.rs:387: The panic info must exist here`).
    ///
    /// Ngày 24/08/2026 ứng dụng thật sập đúng vì thế: `open_package_raster` mở
    /// hộp thoại hỏi quyền (vòng thứ nhất), rồi `run_app_raster` mở màn ứng dụng
    /// (vòng thứ hai). Đường WebView cũ dùng chung một vòng; bản cổng sang bộ
    /// dựng ra pixel tách thành hai mà không ai nhận ra.
    ///
    /// Chắn ở CHỖ NÀY chứ không bắt mọi bên gọi tự xâu chuỗi màn hình vào một
    /// `open_sequence`: bên gọi nào quên là một lần abort, và "đừng quên" không
    /// phải một cơ chế. `run_return` sinh ra đúng để vào lại được một vòng đã
    /// dựng — nên dùng lại là cách `tao` muốn được dùng.
    static VONG: std::cell::RefCell<Option<tao::event_loop::EventLoop<()>>> =
        const { std::cell::RefCell::new(None) };
    /// Vòng lặp đã chạy XONG một lần chưa.
    ///
    /// ⚠️ `run_return` trên macOS **không vào lại được** sau `ControlFlow::Exit`:
    /// lần gọi thứ hai trả về NGAY, không giao một sự kiện nào, không chạy bao
    /// đóng lấy một lần. Đo được 24/08/2026: vòng một chờ đủ 2.01s, vòng hai
    /// thoát sau 113ms mà không nhánh thoát nào chạy.
    ///
    /// Trả về rỗng ở đó là **im lặng làm sai** — màn hình loé lên rồi tắt, bên
    /// gọi nhận `Ok(vec![])` và tưởng người dùng đã đóng cửa sổ. Tệ hơn cả lần
    /// abort trước đó, vì abort ít ra còn nói.
    ///
    /// Nên: lần thứ hai là LỖI. Muốn nhiều màn hình thì xâu chúng vào MỘT
    /// `open_sequence` — đó là lý do hàm ấy tồn tại.
    static DA_CHAY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn chay_chuoi(
    dau: Screen,
    tiep: impl FnMut(&ScreenOutcome) -> Next,
) -> Result<Vec<ScreenOutcome>, String> {
    let p = Phien::moi(dau)?;
    VONG.with(|o| {
        // ⚠️ `try_borrow_mut`, không phải `borrow_mut`. Gọi `chay_chuoi` từ
        // BÊN TRONG bao đóng của một `chay_chuoi` khác là một lỗi lập trình —
        // và một lỗi lập trình phải ra một câu nói được nguyên nhân, không phải
        // một lần hoảng loạn của `RefCell`.
        let Ok(mut muon) = o.try_borrow_mut() else {
            return Err("đã có một chuỗi màn hình đang chạy — `tao` chỉ cho một \
                        vòng lặp sự kiện mỗi tiến trình"
                .to_owned());
        };
        if DA_CHAY.with(std::cell::Cell::get) {
            return Err("vòng lặp sự kiện đã chạy xong một lần — `run_return` \
                        không vào lại được. Nhiều màn hình thì xâu vào MỘT \
                        `open_sequence`."
                .to_owned());
        }
        let vong = muon.get_or_insert_with(|| EventLoopBuilder::new().build());
        let ket = chay_trong_vong(vong, p, tiep);
        DA_CHAY.with(|c| c.set(true));
        ket
    })
}

/// Phần thân, tách ra vì nó chạy BÊN TRONG chỗ mượn vòng lặp.
#[expect(
    clippy::too_many_lines,
    reason = "máy trạng thái nối cửa sổ + bề mặt + trợ năng; xem chú thích trên"
)]
fn chay_trong_vong(
    vong: &mut tao::event_loop::EventLoop<()>,
    mut p: Phien,
    mut tiep: impl FnMut(&ScreenOutcome) -> Next,
) -> Result<Vec<ScreenOutcome>, String> {
    let mut ket_qua: Vec<ScreenOutcome> = Vec::new();
    // ⚠️ MÓC KIỂM KHÓI, và nó phải nằm ở TẦNG NÀY.
    //
    // `TCC_TU_DONG_DONG=<giây>` cho cửa sổ tự đóng như thể người dùng đóng nó:
    // không hành động nào, không công tắc nào — vì tự đóng KHÔNG phải một câu
    // trả lời, y hệt đóng cửa sổ.
    //
    // Nó tồn tại vì ngày 24/08/2026 đường chính của sản phẩm ABORT trong khi 351
    // phép thử đều xanh: không phép thử nào chạy chính cái nhị phân ấy đi hết
    // hai màn hình. Móc này là thứ làm việc đó chạy được mà không cần người ngồi
    // bấm.
    //
    // Đặt ở tầng vòng lặp chứ không ở từng bên gọi: bên gọi nào quên nối là một
    // màn hình treo mãi trong CI, và CI treo thì người ta tắt phép kiểm ấy đi.
    let tu_dong_dong = std::env::var("TCC_TU_DONG_DONG")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_secs);
    // ⚠️ `TCC_TU_DONG_BAM=<mã hành động>` — TỰ BẤM một nút khi hết giờ.
    //
    // Tự ĐÓNG một mình không bao giờ đi quá màn đầu: đóng cửa sổ KHÔNG phải một
    // câu trả lời, nên chuỗi dừng ngay. Muốn kiểm khói đi hết đường thì phải trả
    // lời được một màn hình.
    //
    // ⚠️ Móc này TRẢ LỜI ĐƯỢC HỘP THOẠI HỎI QUYỀN. Nó chỉ bấm mã có THẬT trên
    // màn hình đang hiện — không bịa ra hành động nào — nhưng ai đặt biến này
    // là tự cấp cho mình quyền trả lời hộ người dùng. Nó tồn tại vì đường chính
    // của sản phẩm từng abort trong khi 351 phép thử đều xanh, và không phép thử
    // nào chạy được chính cái nhị phân ấy đi hết hai màn hình.
    // DANH SÁCH có thứ tự, không phải một mã lặp lại mãi.
    //
    // ⚠️ Bản đầu nhận MỘT mã và bấm nó ở mọi màn hình. Thử với luồng cụm từ
    // khôi phục thì nó TREO: bấm "tiếp tục" với ô rỗng → màn báo lỗi hiện ra,
    // màn ấy cũng có nút "tiếp tục" → bấm tiếp → mãi mãi. Một móc kiểm khói treo
    // được là một móc sẽ làm CI treo, và CI treo thì người ta tắt phép kiểm ấy
    // đi — đúng điều kịch bản của chính nó tự cảnh báo.
    //
    // Danh sách thì có đáy: hết mã là đóng cửa sổ.
    let mut tu_dong_bam: std::collections::VecDeque<String> = std::env::var("TCC_TU_DONG_BAM")
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_owned())
        .collect();
    // Đồng hồ đặt lại ở MỖI màn hình, không đếm một lần cho cả chuỗi: một móc
    // kiểm khói đếm chung sẽ đóng màn thứ hai ngay khi nó vừa hiện, và khi ấy
    // "đã đi hết đường" là một câu không ai kiểm được.
    let mut bat_dau = Instant::now();

    let window = dung_cua_so(vong, &p.man.title, p.bo_dung.height())?;

    // ⚠️ `Context`/`Surface` phải sống lâu bằng cửa sổ. Thả sớm là mất bề mặt,
    // và triệu chứng là một cửa sổ trắng trơn — trông y như bộ dựng vẽ hỏng.
    let ngu_canh = Context::new(&window).map_err(|e| format!("không mở được bề mặt: {e}"))?;
    let mut be_mat =
        Surface::new(&ngu_canh, &window).map_err(|e| format!("không mở được bề mặt: {e}"))?;

    // ⚠️ Nối trợ năng TRƯỚC khi cửa sổ được hiện hay nhận tiêu điểm lần đầu —
    // `SubclassingAdapter::new` nói rõ điều đó. Nối muộn thì VoiceOver đã hỏi
    // xong và nhận câu "không có gì ở đây", rồi không hỏi lại.
    // Hàng đợi yêu cầu bấm từ trợ năng, và bảng tra `NodeId` → hành động.
    #[cfg(feature = "accesskit-platform")]
    // `(mã nút, có phải yêu cầu ĐẶT TIÊU ĐIỂM không)`.
    let hang_tro_nang: Arc<Mutex<Vec<(u64, bool)>>> = Arc::new(Mutex::new(Vec::new()));
    #[cfg(feature = "accesskit-platform")]
    let mut bang_hanh_dong = bang_hanh_dong_cua(&p.bo_dung);
    #[cfg(feature = "accesskit-platform")]
    let cay_chung: Arc<Mutex<accesskit::TreeUpdate>> = Arc::new(Mutex::new(
        cay_accesskit(&p.bo_dung, &p.man.text).unwrap_or_else(cay_rong),
    ));
    #[cfg(feature = "accesskit-platform")]
    let mut adapter = crate::accesskit_bridge::platform::ScreenReaderLink::attach(
        &window,
        Arc::clone(&cay_chung),
        Arc::clone(&hang_tro_nang),
    );

    // Nối trợ năng xong mới hiện — thứ tự này là cả nội dung của F6.
    window.set_visible(true);

    // Lỗi xảy ra GIỮA vòng lặp — không trả ra được từ bao đóng, nên để đây.
    let mut loi_doi_man: Option<String> = None;
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
            }
            // `Escape` đi CHUNG NHÁNH với đóng cửa sổ, không phải một nhánh
            // giống nó. Cùng nhánh thì hai đường không thể trôi khác nhau, và
            // điều phải giữ là: cả hai thoát mà **không đặt `da_bam`** — huỷ
            // KHÔNG phải một câu trả lời. Người dùng bàn phím vào được hộp
            // thoại mà không có đường ra là một hộp thoại ép trả lời.
            | Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        event:
                            tao::event::KeyEvent {
                                physical_key: tao::keyboard::KeyCode::Escape,
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    },
                ..
            } => *dieu_khien = ControlFlow::Exit,

            // Hết giờ kiểm khói.
            Event::MainEventsCleared if tu_dong_dong.is_some_and(|h| bat_dau.elapsed() >= h) => {
                // Có mã để bấm VÀ mã ấy có thật trên màn hình này thì bấm; nếu
                // không thì thoát y như người dùng đóng cửa sổ.
                let co_that = tu_dong_bam
                    .front()
                    .filter(|ma| {
                        p.man
                            .tree
                            .action_ids()
                            .iter()
                            .any(|a| a.as_str() == ma.as_str())
                    })
                    .cloned();
                match co_that {
                    Some(ma) => {
                        tu_dong_bam.pop_front();
                        ap_ket_qua(SauCuBam::Ket(ma), &mut p.ve_lai, &mut p.da_bam, dieu_khien);
                    }
                    None => *dieu_khien = ControlFlow::Exit,
                }
            }

            // Trạng thái được-chọn phải nói cho cầu trợ năng biết, vì trên
            // AT-SPI (Linux) adapter KHÔNG có thẻ cửa sổ nên nó không tự thấy.
            // macOS và Windows tự thấy, và `set_window_focused` ở đó là lệnh
            // rỗng — nên chỗ này viết một lần cho cả ba, không rẽ nhánh theo nền.
            //
            // Thiếu dòng này thì Orca tin cửa sổ luôn đang được chọn, và người
            // dùng nghe màn hình này đọc chồng lên cửa sổ họ đang thật sự dùng.
            #[cfg(feature = "accesskit-platform")]
            Event::WindowEvent {
                event: WindowEvent::Focused(dang_chon),
                ..
            } => adapter.set_window_focused(dang_chon),

            // ⚠️ Chữ đến từ BỘ GÕ CỦA HỆ ĐIỀU HÀNH, không phải từ mã ta.
            //
            // `tao` cài `NSTextInputClient` trên macOS, nên dấu tiếng Việt —
            // kể cả ca hai tầng như `ổ` — đã được ghép xong trước khi tới đây.
            // Đường WebView cũng nhận đúng từ nguồn ấy. Ta KHÔNG tự viết bộ gõ,
            // và không nên: bộ gõ là thứ người dùng đã chọn và đã quen.
            Event::WindowEvent {
                event: WindowEvent::ReceivedImeText(chu),
                ..
            } => p.nhan_chu(&chu),

            Event::WindowEvent {
                event: WindowEvent::ModifiersChanged(m),
                ..
            } => {
                p.tt.shift = m.shift_key();
            }

            // ── Kéo đổi cỡ cửa sổ: xếp lại bố cục theo chiều rộng mới ──
            //
            // Đặt chiều rộng rồi ĐÁNH DẤU vẽ lại, chứ không vẽ ngay tại đây:
            // lượt kéo sinh ra hàng chục sự kiện một giây, và xếp lại bố cục ở
            // mỗi sự kiện là kéo cửa sổ thành giật.
            Event::WindowEvent {
                event: WindowEvent::Resized(co),
                ..
            } => {
                let l = co.to_logical::<f64>(window.scale_factor());
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "kích thước cửa sổ, luôn dương và nhỏ"
                )]
                p.bo_dung.set_width(l.width as usize);
                p.ve_lai = true;
                window.request_redraw();
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
                p.xoa_lui();
            }

            // ── Mũi tên, Home/End, Delete: đi lại trong ô nhập ──
            //
            // Không có chúng thì sửa một lỗi đánh máy ở giữa nghĩa là xoá sạch
            // mọi thứ phía sau nó. Trên màn khôi phục ví — hai mươi bốn chữ —
            // một lỗi ở chữ thứ ba bắt gõ lại hai mươi mốt chữ.
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        event:
                            tao::event::KeyEvent {
                                physical_key: phim @ (tao::keyboard::KeyCode::ArrowLeft
                                | tao::keyboard::KeyCode::ArrowRight
                                | tao::keyboard::KeyCode::Home
                                | tao::keyboard::KeyCode::End
                                | tao::keyboard::KeyCode::Delete),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                match phim {
                    tao::keyboard::KeyCode::ArrowLeft => p.doi_con_tro(false),
                    tao::keyboard::KeyCode::ArrowRight => p.doi_con_tro(true),
                    tao::keyboard::KeyCode::Home => p.con_tro_ve_dau_hoac_cuoi(false),
                    tao::keyboard::KeyCode::End => p.con_tro_ve_dau_hoac_cuoi(true),
                    _ => p.xoa_tai_cho(),
                }
                window.request_redraw();
            }

            // ── Tab: đi tới đích kế tiếp ──
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        event:
                            tao::event::KeyEvent {
                                physical_key: tao::keyboard::KeyCode::Tab,
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                let thu_tu = p.bo_dung.focus_order();
                let moi = tieu_diem_ke(&thu_tu, p.tt.tieu_diem.as_ref(), p.tt.shift);
                p.tt.dat_tieu_diem(moi);
                p.ve_lai = true;
                window.request_redraw();
            }

            // ── Enter / Space: kích hoạt đích đang chọn ──
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        event:
                            tao::event::KeyEvent {
                                physical_key:
                                    tao::keyboard::KeyCode::Enter
                                    | tao::keyboard::KeyCode::NumpadEnter
                                    | tao::keyboard::KeyCode::Space,
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                if let Some(h) = phim_kich_hoat(p.tt.tieu_diem.as_ref()) {
                    let k = sau_cu_bam(Some(h), &mut p.bat);
                    ap_ket_qua(k, &mut p.ve_lai, &mut p.da_bam, dieu_khien);
                    window.request_redraw();
                }
            }

            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => p.cuon(&delta, &window),

            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => p.chuot_toi(&position, &window),

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
                    &p.bo_dung,
                    &mut p.tt,
                    &mut p.bat,
                    &mut p.ve_lai,
                    &mut p.da_bam,
                    dieu_khien,
                );
            }

            Event::RedrawRequested(_) | Event::MainEventsCleared => {
                // Rút yêu cầu bấm từ trợ năng và cho chạy qua ĐÚNG đường của
                // chuột. Không có nhánh riêng: mọi luật của hộp thoại áp cho
                // chuột đều áp cho đây.
                // ⚠️ `p.da_bam.is_none()` — màn hình đã kết thúc thì KHÔNG rút
                // thêm gì nữa. `tao` còn giao vài sự kiện sau khi ta đặt `Exit`,
                // và không có chắn này thì một yêu cầu còn trong hàng đợi **ghi
                // đè lựa chọn của người dùng**: họ bấm "Từ chối", rồi một
                // `AXPress("cho-phep")` xếp trước đó biến kết quả thành "Cho
                // phép". Rà soát 21/08/2026, F3.
                #[cfg(feature = "accesskit-platform")]
                if p.da_bam.is_none()
                    && let Some(k) =
                        rut_yeu_cau_tro_nang(&hang_tro_nang, &bang_hanh_dong, &mut p.bat, &mut p.tt)
                {
                    ap_ket_qua(k, &mut p.ve_lai, &mut p.da_bam, dieu_khien);
                }
                if core::mem::take(&mut p.ve_lai) {
                    ve_lai_man_hinh(
                        &mut p.bo_dung,
                        &p.man.tree,
                        &p.bat,
                        &p.tt.noi_dung_o,
                        p.tt.tieu_diem.as_ref(),
                        p.tt.duoi_chuot.as_ref(),
                        p.tt.o_dang_chon.clone().map(|n| (n, p.tt.con_tro)),
                    );
                    // Gạt một công tắc mà không báo lại thì VoiceOver vẫn đọc
                    // trạng thái CŨ — người dùng nghe "tắt" trong khi màn hình
                    // hiện "bật". Ở màn hỏi quyền, đó là nghe một đằng cấp một
                    // nẻo.
                    #[cfg(feature = "accesskit-platform")]
                    bao_tro_nang(
                        &p.bo_dung,
                        &p.man.text,
                        &cay_chung,
                        &mut adapter,
                        &mut bang_hanh_dong,
                    );
                }
                trinh_bay(&p.bo_dung, &mut be_mat, &window, p.tt.cuon);
            }
            _ => {}
        }

        // ── Màn hình vừa kết thúc? Hỏi bên gọi xem còn màn nào nữa ──────────
        //
        // ⚠️ Chỉ chạy khi có người BẤM. `CloseRequested` cũng đặt `Exit` nhưng
        // để `da_bam` là `None`, và đóng cửa sổ KHÔNG phải một câu trả lời —
        // hỏi `tiep` ở đó là biến cái đóng cửa sổ thành một lựa chọn.
        if p.da_bam.is_none() {
            return;
        }
        let ket = p.ket_man();
        let sau = tiep(&ket);
        ket_qua.push(ket);
        let Next::Show(m) = sau else {
            *dieu_khien = ControlFlow::Exit;
            return;
        };
        // Không vẽ được màn tiếp thì DỪNG, không ở lại màn cũ: ở lại là người
        // dùng bấm xong mà màn hình không đổi, và họ sẽ bấm lại.
        if let Err(e) = p.doi(*m, &window) {
            loi_doi_man = Some(e);
            *dieu_khien = ControlFlow::Exit;
            return;
        }
        // ⚠️ DỌN hàng đợi trợ năng khi sang màn mới. F3, lần thứ hai — và lần
        // này chính `open_sequence` mở ra đường.
        //
        // Mã nút đánh lại từ 0 MỖI LẦN dựng cây, nên nút số 5 của màn 2 không
        // phải nút số 5 của màn 1. Hàng đợi thì không tự dọn. Và chắn F3
        // (`da_bam.is_none()`) vừa hết tác dụng đúng lúc này, vì `ket_man` vừa
        // lấy `da_bam` ra.
        //
        // Ba thứ ấy ghép lại: một `AXPress` xếp ở màn 1 mà chưa kịp rút sẽ được
        // rút ở màn 2 và tra vào BẢNG MỚI — chạy một hành động không ai bấm,
        // trên một màn hình người dùng còn chưa đọc. Ở luồng nhập ví, màn 2 là
        // màn hỏi mã PIN.
        //
        // Dọn ở ĐÂY chứ không ở `bao_tro_nang`: hàm ấy còn chạy khi vẽ lại cùng
        // một màn (gạt công tắc), mà ở đó hình dạng cây không đổi nên mã nút vẫn
        // đúng và một yêu cầu đang chờ vẫn hợp lệ.
        #[cfg(feature = "accesskit-platform")]
        if let Ok(mut h) = hang_tro_nang.lock() {
            h.clear();
        }
        #[cfg(feature = "accesskit-platform")]
        bao_tro_nang(
            &p.bo_dung,
            &p.man.text,
            &cay_chung,
            &mut adapter,
            &mut bang_hanh_dong,
        );
        // Huỷ cái `Exit` mà `ap_ket_qua` vừa đặt. Đặt rồi gỡ nhìn vòng vo, nhưng
        // đường kia là để `ap_ket_qua` biết nó đang ở trong chuỗi hay không — và
        // một hàm đổi hành vi theo ngữ cảnh gọi là một hàm sẽ bị gọi nhầm ngữ
        // cảnh.
        *dieu_khien = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(50));
        bat_dau = Instant::now();
        window.request_redraw();
    });

    loi_doi_man.map_or(Ok(ket_qua), Err)
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
#[cfg(feature = "accesskit-platform")]
fn rut_yeu_cau_tro_nang(
    hang: &Arc<Mutex<Vec<(u64, bool)>>>,
    bang: &std::collections::BTreeMap<u64, crate::accesskit_bridge::Dich>,
    bat: &mut BTreeSet<String>,
    tt: &mut TrangThai,
) -> Option<SauCuBam> {
    let yeu_cau: Vec<(u64, bool)> = hang
        .lock()
        .map(|mut q| core::mem::take(&mut *q))
        .unwrap_or_default();
    let mut cuoi = None;
    for (id, la_tieu_diem) in yeu_cau {
        // Không tra ra thì BỎ QUA. Yêu cầu tới sau khi cây đã đổi thì con số cũ
        // trỏ vào một nút không còn nữa — đoán bừa ở đây là bấm nhầm nút.
        let Some(dich) = bang.get(&id).cloned() else {
            continue;
        };
        // ⚠️ `Focus` CHỈ được chạm tới ô nhập.
        //
        // Trình đọc màn hình gửi `Focus` mỗi khi người dùng DI CHUYỂN tới một
        // nút. Cho nó kích hoạt nút thì lướt qua "Cho phép" là đã cấp quyền —
        // di chuyển tiêu điểm không phải một câu trả lời.
        let a = match (dich, la_tieu_diem) {
            (crate::accesskit_bridge::Dich::O(nhan), _) => {
                tt.dat_tieu_diem(Some(crate::FocusTarget::Field(nhan)));
                cuoi = Some(SauCuBam::VeLai);
                continue;
            }
            (crate::accesskit_bridge::Dich::Bam(..), true) => continue,
            (crate::accesskit_bridge::Dich::Bam(a, la_cong_tac), false) => (a, la_cong_tac),
        };
        let (a, la_cong_tac) = a;
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
    ///
    /// ⚠️ Trường này **suy ra** từ `tieu_diem`, không được đặt riêng. Xem
    /// [`TrangThai::dat_tieu_diem`].
    o_dang_chon: Option<String>,
    /// Đích bàn phím đang chọn. Nguồn sự thật DUY NHẤT cho cả viền vẽ ra và cho
    /// chỗ chữ gõ vào đi tới.
    tieu_diem: Option<crate::FocusTarget>,
    /// `Shift` đang giữ — để `Shift+Tab` đi ngược.
    shift: bool,
    /// Đích chuột đang rê qua. Tách hẳn khỏi `tieu_diem`: rê chuột KHÔNG được
    /// đổi chỗ bàn phím đang đứng.
    duoi_chuot: Option<crate::FocusTarget>,
    /// Con trỏ trong ô đang chọn — CHỮ THỨ MẤY, tính từ 0.
    con_tro: usize,
}

/// Chèn `chu` vào `goc` ở vị trí chữ thứ `i`. **Hàm thuần.**
///
/// Cắt theo CHỮ rồi ghép, không cắt theo byte: một chữ tiếng Việt có dấu là
/// nhiều byte, và cắt byte là cắt vào giữa một ký tự.
fn chen_tai(goc: &str, i: usize, chu: &str) -> String {
    let truoc: String = goc.chars().take(i).collect();
    let sau: String = goc.chars().skip(i).collect();
    format!("{truoc}{chu}{sau}")
}

/// Xoá chữ NGAY TRƯỚC vị trí `i`. Trả về chuỗi mới. **Hàm thuần.**
fn xoa_truoc(goc: &str, i: usize) -> String {
    if i == 0 {
        return goc.to_owned();
    }
    let truoc: String = goc.chars().take(i - 1).collect();
    let sau: String = goc.chars().skip(i).collect();
    format!("{truoc}{sau}")
}

/// Xoá chữ NGAY TẠI vị trí `i` (phím `Delete`). **Hàm thuần.**
fn xoa_tai(goc: &str, i: usize) -> String {
    let truoc: String = goc.chars().take(i).collect();
    let sau: String = goc.chars().skip(i + 1).collect();
    format!("{truoc}{sau}")
}

impl TrangThai {
    /// Đặt tiêu điểm, và **đồng thời** đặt ô đang chọn.
    ///
    /// Một hàm chứ không hai lệnh gán rời: hai trường ấy lệch nhau nghĩa là
    /// viền vẽ quanh một ô trong khi chữ gõ vào lại chạy sang ô khác — người
    /// dùng nhìn thấy một đằng, máy làm một nẻo.
    fn dat_tieu_diem(&mut self, dich: Option<crate::FocusTarget>) {
        self.o_dang_chon = match dich.as_ref() {
            Some(crate::FocusTarget::Field(nhan)) => Some(nhan.clone()),
            _ => None,
        };
        // Vào một ô thì con trỏ đứng ở CUỐI chữ đang có — chỗ người ta gõ tiếp.
        // Đứng ở đầu thì gõ một chữ là chữ ấy nhảy lên trước mọi thứ đã gõ.
        self.con_tro = self
            .o_dang_chon
            .as_ref()
            .and_then(|n| self.noi_dung_o.get(n))
            .map_or(0, |v| v.chars().count());
        self.tieu_diem = dich;
    }

    /// Kẹp con trỏ vào trong chuỗi hiện tại.
    fn kep_con_tro(&mut self) {
        let dai = self
            .o_dang_chon
            .as_ref()
            .and_then(|n| self.noi_dung_o.get(n))
            .map_or(0, |v| v.chars().count());
        self.con_tro = self.con_tro.min(dai);
    }
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
    tieu_diem: Option<&crate::FocusTarget>,
    duoi_chuot: Option<&crate::FocusTarget>,
    con_tro: Option<(String, usize)>,
) {
    // Đặt TRƯỚC khi vẽ: viền tiêu điểm sinh ra trong chính lượt vẽ ấy.
    bo_dung.set_focus(tieu_diem.cloned());
    bo_dung.set_hover(duoi_chuot.cloned());
    bo_dung.set_caret(con_tro);
    if let Ok(cay_moi) = tree
        .with_toggles(bat)
        .and_then(|c| c.with_fields(noi_dung_o))
    {
        let _ = bo_dung.render(&cay_moi);
    }
}

/// Đích kế tiếp khi bấm `Tab`. **Hàm thuần** — kiểm thử được không cần cửa sổ.
///
/// Chưa chọn gì thì `Tab` vào đích ĐẦU, `Shift+Tab` vào đích CUỐI. Vòng lại ở
/// hai đầu: danh sách hữu hạn mà đi tới cuối rồi kẹt là bàn phím vào được một
/// chỗ không ra được.
///
/// Đích cũ không còn trong danh sách (màn hình vừa đổi) thì coi như chưa chọn —
/// thà quay về đầu còn hơn chọn bừa một ô khác chỉ vì nó cùng thứ tự.
fn tieu_diem_ke(
    thu_tu: &[crate::FocusTarget],
    dang: Option<&crate::FocusTarget>,
    lui: bool,
) -> Option<crate::FocusTarget> {
    if thu_tu.is_empty() {
        return None;
    }
    let vi_tri = dang.and_then(|d| thu_tu.iter().position(|x| x == d));
    let moi = match (vi_tri, lui) {
        (None, false) => 0,
        (None, true) => thu_tu.len() - 1,
        (Some(i), false) => (i + 1) % thu_tu.len(),
        (Some(i), true) => (i + thu_tu.len() - 1) % thu_tu.len(),
    };
    thu_tu.get(moi).cloned()
}

/// Phím `Enter`/`Space` trên đích đang chọn thành cái gì. **Hàm thuần.**
///
/// ⚠️ Ô nhập thì **không** thành gì cả. `Enter` trong một ô nhập mà kích hoạt
/// nút gần nhất là cách người dùng gửi đi một biểu mẫu họ chưa điền xong — và
/// trên hộp thoại quyền, "nút gần nhất" có thể là *Cho phép*.
fn phim_kich_hoat(dang: Option<&crate::FocusTarget>) -> Option<crate::Hit<'_>> {
    match dang {
        Some(crate::FocusTarget::Action { id, toggle }) => Some(crate::Hit {
            action: id.as_str(),
            toggle: *toggle,
        }),
        _ => None,
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
        tt.dat_tieu_diem(Some(crate::FocusTarget::Field(nhan.to_owned())));
        *ve_lai = true;
    } else if let Some(h) = bo_dung.hit_test(x, y) {
        // Bấm chuột cũng ĐẶT tiêu điểm: chuột và bàn phím phải nhìn thấy cùng
        // một giao diện. Không thế thì Tab sau một cú bấm sẽ nhảy về đầu.
        let dich = crate::FocusTarget::Action {
            id: h.action.to_owned(),
            toggle: h.toggle,
        };
        let k = sau_cu_bam(Some(h), bat);
        tt.dat_tieu_diem(Some(dich));
        ap_ket_qua(k, ve_lai, da_bam, dieu_khien);
    } else {
        let k = sau_cu_bam(None, bat);
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
    // ⚠️ QUYẾT ĐỊNH ĐẦU TIÊN THẮNG — bất biến B19.
    //
    // Màn hình đã kết thúc thì không gì đổi được nữa: không nút thứ hai, không
    // công tắc gạt thêm. `tao` còn giao vài sự kiện sau khi ta đặt `Exit`, nên
    // một cú bấm đang xếp hàng vẫn tới đây.
    //
    // Chắn này TỪNG chỉ có ở đường trợ năng (bản vá F3, 21/08/2026), còn đường
    // CHUỘT thì không — trong khi chú thích ở đó lại viết "mọi luật của hộp
    // thoại áp cho chuột đều áp cho đây", tức là tin rằng đường chuột chặt hơn.
    // Nó lỏng hơn. Chuyển chắn vào ĐÂY, chỗ hai đường gặp nhau, thay vì chép nó
    // sang đường thứ hai — chép là để lần sau có đường thứ ba mà quên.
    if da_bam.is_some() {
        return;
    }
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
        // Kéo đổi cỡ được, trong khoảng bộ dựng xếp lại được. Chặn ở đây chứ
        // không chỉ kẹp trong bộ dựng: kẹp thầm thì người dùng kéo hẹp lại mà
        // chữ không hẹp theo, và họ tưởng cửa sổ hỏng.
        .with_resizable(true)
        .with_min_inner_size(LogicalSize::new(crate::MIN_WIDTH as f64, 160.0_f64))
        .with_max_inner_size(LogicalSize::new(crate::MAX_WIDTH as f64, 4096.0_f64))
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
#[cfg(feature = "accesskit-platform")]
fn bang_hanh_dong_cua(
    bd: &RasterRenderer,
) -> std::collections::BTreeMap<u64, crate::accesskit_bridge::Dich> {
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
#[cfg(feature = "accesskit-platform")]
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
#[cfg(feature = "accesskit-platform")]
fn bao_tro_nang(
    bo_dung: &RasterRenderer,
    chu: &ScreenText,
    cay_chung: &Arc<Mutex<accesskit::TreeUpdate>>,
    adapter: &mut crate::accesskit_bridge::platform::ScreenReaderLink,
    bang: &mut std::collections::BTreeMap<u64, crate::accesskit_bridge::Dich>,
) {
    *bang = bang_hanh_dong_cua(bo_dung);
    if let Some(moi) = cay_accesskit(bo_dung, chu) {
        if let Ok(mut c) = cay_chung.lock() {
            *c = moi.clone();
        }
        adapter.publish(moi);
    }
}

/// Cây trợ năng của lần vẽ gần nhất, ở dạng AccessKit.
#[cfg(feature = "accesskit-platform")]
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
/// Vẽ thanh chỉ báo cuộn ở mép phải, **chỉ khi** còn nội dung ngoài khung.
///
/// Cuộn được mà không có dấu hiệu nào thì người dùng không biết là còn nữa —
/// nội dung dưới đáy coi như không tồn tại. Vẽ lên MẶT TRÌNH BÀY chứ không vào
/// ảnh bố cục: ảnh là toàn bộ nội dung, còn thanh này nói về phần đang nhìn.
fn ve_thanh_cuon(ra: &mut [u32], rong: u32, cao: u32, cao_anh_man: f64, cuon: f64) {
    if cao_anh_man <= f64::from(cao) || cao < 8 || rong < 6 {
        return;
    }
    let ty = f64::from(cao) / cao_anh_man;
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "kích thước cửa sổ, luôn dương và nhỏ"
    )]
    let dai = ((f64::from(cao) * ty) as u32).max(16).min(cao);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "kích thước cửa sổ, luôn dương và nhỏ"
    )]
    let dinh = ((cuon * ty) as u32).min(cao.saturating_sub(dai));
    for y in dinh..(dinh + dai).min(cao) {
        for x in rong.saturating_sub(5)..rong.saturating_sub(1) {
            if let Some(o) = ra.get_mut((y * rong + x) as usize) {
                *o = 0x0090_9090;
            }
        }
    }
}

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
            // ⚠️ `bd.width()`, KHÔNG phải hằng `WIDTH`.
            //
            // Chiều rộng thành thuộc tính ngày 25/08/2026, và chỗ này còn dùng
            // hằng thì mỗi hàng bị đọc lệch đi một đoạn — ảnh xiên dần xuống
            // dưới. Lỗi do chính lượt sửa "cho cửa sổ đổi cỡ được" sinh ra.
            let rong_anh = bd.width();
            let xam = if lx < rong_anh && ly < cao_anh {
                u32::from(anh[ly * rong_anh + lx])
            } else {
                255
            };
            if let Some(o) = ra.get_mut((y * rong + x) as usize) {
                *o = (xam << 16) | (xam << 8) | xam;
            }
        }
    }
    #[expect(clippy::cast_precision_loss, reason = "chiều cao ảnh, luôn nhỏ")]
    ve_thanh_cuon(ra, rong, cao, cao_anh as f64 * ty_le, cuon * ty_le);
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay"
)]
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
        // Mọi chỗ dựng `fields:` phải lấy thẳng từ `tt.noi_dung_o`. Soi theo
        // CHỖ chứ không theo một chuỗi cố định: bản trước khớp đúng
        // `fields: tt.noi_dung_o`, rồi việc bọc thêm `core::mem::take` làm nó
        // đỏ trong khi điều nó canh vẫn nguyên. Một phép thử đỏ vì mã được viết
        // lại chứ không vì mã sai là một phép thử người ta sẽ sửa cho im.
        let cho = than.match_indices("fields:").collect::<Vec<_>>();
        assert!(!cho.is_empty(), "kết quả không mang nội dung ô nhập");
        for (i, _) in cho {
            let doan = &than[i..(i + 60).min(than.len())];
            // Bỏ qua chỗ KHAI BÁO trường (`fields: BTreeMap<…>`); chỉ soi chỗ
            // DỰNG giá trị.
            if doan.contains("BTreeMap<") {
                continue;
            }
            assert!(
                // Bản đồ RỖNG cũng được: nó không mang gì, nên không mang được
                // thứ đã đi qua lượt vẽ.
                doan.contains("tt.noi_dung_o") || doan.contains("BTreeMap::new()"),
                "một chỗ dựng `fields:` không lấy từ trạng thái khung: {doan}"
            );
        }
        // Và chỗ DUY NHẤT biến chữ thành chấm phải nằm ở bộ dựng, không ở đây.
        assert!(
            !than.contains('•'),
            "cửa sổ đang tự che chữ — việc ấy thuộc về lúc vẽ, và làm ở đây thì \
             khung nhận về hàng chấm thay vì thứ người dùng gõ"
        );
    }

    /// **Vòng lặp sự kiện dựng MỘT lần cho cả tiến trình.**
    ///
    /// # Cái này đã làm ứng dụng thật SẬP
    ///
    /// `tao` chỉ cho dựng một vòng lặp mỗi tiến trình, và dựng lần thứ hai thì
    /// nó **abort** — không phải trả lỗi. Thông báo (`app_state.rs:387: The
    /// panic info must exist here`) không nói gì về nguyên nhân.
    ///
    /// Ngày 24/08/2026 `tcc-browser examples/hello-tcc` sập đúng vì thế:
    /// `open_package_raster` mở hộp thoại hỏi quyền (vòng thứ nhất), rồi
    /// `run_app_raster` mở màn ứng dụng (vòng thứ hai). Đường chính của sản phẩm
    /// không chạy được, và **lần "kiểm khói" hôm trước báo là chạy** — nó chạy
    /// 12 giây, thấy tiến trình còn sống rồi kết luận, trong khi lúc ấy tiến
    /// trình mới đứng ở hộp thoại và chưa tới vòng thứ hai.
    ///
    /// Phép thử soi mã nguồn, vì thật sự dựng hai vòng trong một lượt `cargo
    /// test` là abort cả lượt chạy — không có `should_panic` nào bắt được một
    /// lần abort.
    #[test]
    fn vong_lap_su_kien_dung_mot_lan_cho_ca_tien_trinh() {
        let nguon = include_str!("window.rs");
        let than = nguon.split("#[cfg(test)]").next().unwrap_or(nguon);
        assert_eq!(
            than.matches("EventLoopBuilder::new()").count(),
            1,
            "có nhiều hơn một chỗ dựng vòng lặp sự kiện — chỗ thứ hai sẽ ABORT, \
             không phải trả lỗi"
        );
        assert!(
            than.contains("get_or_insert_with"),
            "vòng lặp không được dùng lại — mỗi lần gọi lại dựng một vòng mới"
        );
        // Và phải có chắn cho ca gọi LỒNG NHAU: `chay_chuoi` từ trong bao đóng
        // của một `chay_chuoi` khác là lỗi lập trình, phải ra một câu nói được
        // nguyên nhân chứ không phải một lần hoảng loạn của `RefCell`.
        assert!(
            than.contains("try_borrow_mut"),
            "không có chắn cho ca gọi lồng nhau"
        );
    }

    /// **Phím gõ vào bị TỪ CHỐI ngay, không đẩy xuống lượt vẽ.**
    ///
    /// `with_fields` cũng chặn chuỗi hỏng, nhưng chặn ở đó thì lỗi rơi vào
    /// `ve_lai_man_hinh` — nơi không có ai để báo, nên nó nuốt. Người dùng gõ,
    /// màn hình đứng im, họ gõ tiếp. Chặn ở ĐÂY thì ô giữ nguyên giá trị cũ và
    /// không có gì để báo cả.
    #[test]
    fn phim_go_khong_hop_le_bi_tu_choi_ngay() {
        use tcc_ui::{Flow, Gap};
        let cay = Node::group(Flow::Column, Gap::Medium)
            .child(Node::field("Ghi chú", "", false).unwrap())
            .unwrap();
        let mut p = Phien::moi(Screen {
            tree: cay,
            title: "thử".to_owned(),
            text: ScreenText {
                destructive_note: "n".to_owned(),
                destructive_role: "r".to_owned(),
            },
        })
        .expect("dựng được phiên");
        p.tt.o_dang_chon = Some("Ghi chú".to_owned());

        p.nhan_chu("an toàn");
        assert_eq!(
            p.tt.noi_dung_o.get("Ghi chú").map(String::as_str),
            Some("an toàn")
        );

        // Ký tự đảo chiều — làm chữ HIỆN RA khác chữ đã gõ.
        p.ve_lai = false;
        p.nhan_chu("\u{202e}");
        assert_eq!(
            p.tt.noi_dung_o.get("Ghi chú").map(String::as_str),
            Some("an toàn"),
            "ký tự đảo chiều được nhận vào ô"
        );
        assert!(!p.ve_lai, "từ chối rồi mà vẫn xin vẽ lại");

        // Chưa chọn ô nào thì chữ RƠI ĐI, không tạo ô mới.
        p.tt.o_dang_chon = None;
        p.nhan_chu("lạc");
        assert_eq!(
            p.tt.noi_dung_o.len(),
            1,
            "gõ khi chưa chọn ô lại tạo ra ô mới"
        );

        // XOÁ cũng đi qua đúng phép kiểm ấy. Không phải vì xoá làm chuỗi dài
        // ra, mà vì bất biến cần giữ là `noi_dung_o` LUÔN hợp lệ: giữ được thì
        // thứ VẼ RA luôn bằng thứ TRẢ VỀ, và người dùng không xác nhận một
        // chuỗi họ chưa từng nhìn thấy.
        p.tt.o_dang_chon = Some("Ghi chú".to_owned());
        p.xoa_lui();
        assert_eq!(
            p.tt.noi_dung_o.get("Ghi chú").map(String::as_str),
            Some("an toà"),
            "xoá lùi không bỏ đúng một ký tự"
        );
    }

    /// **B19 — quyết định ĐẦU TIÊN thắng, ở CẢ HAI đường vào.**
    ///
    /// `tao` còn giao vài sự kiện sau khi ta đặt `Exit`, nên một cú bấm đang xếp
    /// hàng vẫn tới nơi. Không chắn thì nó ghi đè câu trả lời người dùng đã đưa.
    ///
    /// Chắn này từng CHỈ có ở đường trợ năng (bản vá F3), còn đường chuột thì
    /// không — trong khi chú thích ở đó viết "mọi luật của hộp thoại áp cho chuột
    /// đều áp cho đây", tức là tin rằng đường chuột chặt hơn. Nó lỏng hơn.
    ///
    /// Phép thử gọi thẳng `ap_ket_qua`, nên nó kiểm chỗ HAI ĐƯỜNG GẶP NHAU chứ
    /// không kiểm một trong hai.
    #[test]
    fn quyet_dinh_dau_tien_thang() {
        let mut ve_lai = false;
        let mut da_bam = None;
        let mut dk = ControlFlow::Wait;

        ap_ket_qua(
            SauCuBam::Ket("tu-choi".to_owned()),
            &mut ve_lai,
            &mut da_bam,
            &mut dk,
        );
        assert_eq!(da_bam.as_deref(), Some("tu-choi"));

        // Cú bấm thứ hai — dù là một nút KHÁC — không được đổi gì.
        ap_ket_qua(
            SauCuBam::Ket("cho-phep".to_owned()),
            &mut ve_lai,
            &mut da_bam,
            &mut dk,
        );
        assert_eq!(
            da_bam.as_deref(),
            Some("tu-choi"),
            "cú bấm thứ hai ghi đè câu trả lời của người dùng"
        );

        // Và công tắc gạt thêm sau khi màn hình đã kết thúc cũng không được vẽ
        // lại: màn hình đã xong thì không gì đổi được nữa.
        ve_lai = false;
        ap_ket_qua(SauCuBam::VeLai, &mut ve_lai, &mut da_bam, &mut dk);
        assert!(!ve_lai, "màn hình đã kết thúc mà vẫn nhận lệnh vẽ lại");
    }

    /// **Sang màn mới phải DỌN hàng đợi yêu cầu trợ năng.**
    ///
    /// # Ba mảnh, mỗi mảnh vô hại, ghép lại thì không
    ///
    /// 1. Mã nút trợ năng đánh lại **từ 0 mỗi lần dựng cây**
    ///    (`to_accesskit_with_actions`), nên nút số 5 của màn 2 không phải nút
    ///    số 5 của màn 1.
    /// 2. Hàng đợi `AXPress` không tự dọn — nó chỉ có chỗ đẩy vào và chỗ rút ra.
    /// 3. Chắn F3 (`da_bam.is_none()`) hết tác dụng **đúng lúc đổi màn**, vì
    ///    `ket_man` vừa lấy `da_bam` ra.
    ///
    /// Ghép lại: một yêu cầu xếp ở màn 1 mà chưa kịp rút sẽ được rút ở màn 2 và
    /// tra vào BẢNG MỚI — chạy một hành động không ai bấm, trên một màn hình
    /// người dùng còn chưa đọc. Ở luồng nhập ví, màn 2 là màn hỏi mã PIN.
    ///
    /// Soi mã nguồn, vì phép thử thật cần một vòng lặp sự kiện và một trình đọc
    /// màn hình. Chốt hai điều: có chỗ dọn, và nó đứng TRƯỚC chỗ dựng lại bảng.
    #[test]
    fn sang_man_moi_thi_don_hang_doi_tro_nang() {
        let nguon = include_str!("window.rs");
        let than = nguon.split("#[cfg(test)]").next().unwrap_or(nguon);
        let sau_doi = than
            .split_once("if let Err(e) = p.doi(")
            .map(|(_, sau)| sau)
            .expect("có nhánh sang màn mới");
        let don = sau_doi
            .find("h.clear()")
            .expect("nhánh sang màn mới KHÔNG dọn hàng đợi trợ năng");
        let dung_bang = sau_doi
            .find("bao_tro_nang(")
            .expect("nhánh sang màn mới không dựng lại bảng hành động");
        assert!(
            don < dung_bang,
            "dọn hàng đợi SAU khi dựng lại bảng — giữa hai chỗ ấy một yêu cầu cũ \
             vẫn tra được vào bảng mới"
        );
    }

    /// **Đóng cửa sổ KHÔNG được tính là một câu trả lời.**
    ///
    /// `CloseRequested` và một cú bấm đều đặt `ControlFlow::Exit`, nên nhìn từ
    /// đuôi vòng lặp chúng giống hệt nhau — trừ `da_bam`. Hỏi `tiep` mà không
    /// chắn chỗ ấy là biến cái đóng cửa sổ thành một lựa chọn: người dùng đóng
    /// hộp thoại quyền và luồng đi tiếp như thể họ đã bấm.
    ///
    /// Soi mã nguồn vì phép thử thật cần một vòng lặp sự kiện, mà `tao` chỉ cho
    /// một vòng mỗi tiến trình — hai phép thử như thế trong một lượt `cargo
    /// test` là một lượt hoảng loạn.
    #[test]
    fn dong_cua_so_khong_hoi_ben_goi() {
        let nguon = include_str!("window.rs");
        let than = nguon.split("#[cfg(test)]").next().unwrap_or(nguon);
        let goi = than.find("tiep(&ket)").expect("không thấy chỗ gọi `tiep`");
        let chan = than
            .find("if p.da_bam.is_none() {")
            .expect("không thấy chắn `p.da_bam.is_none()`");
        assert!(
            chan < goi,
            "`tiep` được hỏi trước khi chắn `da_bam.is_none()` — đóng cửa sổ sẽ \
             được tính là một câu trả lời"
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

    fn dich(id: &str, toggle: bool) -> crate::FocusTarget {
        crate::FocusTarget::Action {
            id: id.to_owned(),
            toggle,
        }
    }

    /// **`Tab` đi vòng, không kẹt ở hai đầu.**
    ///
    /// Đi tới cuối rồi dừng nghĩa là bàn phím vào được một chỗ mà không ra được
    /// — người dùng phải với lấy chuột, tức là bàn phím chưa dùng được.
    #[test]
    fn tab_di_vong_hai_dau() {
        let tt = vec![dich("a", false), crate::FocusTarget::Field("Ô".to_owned())];
        assert_eq!(tieu_diem_ke(&tt, None, false), Some(tt[0].clone()));
        assert_eq!(tieu_diem_ke(&tt, Some(&tt[0]), false), Some(tt[1].clone()));
        assert_eq!(
            tieu_diem_ke(&tt, Some(&tt[1]), false),
            Some(tt[0].clone()),
            "tới cuối mà không vòng về đầu"
        );
        // Ngược lại: chưa chọn gì thì Shift+Tab vào đích CUỐI.
        assert_eq!(tieu_diem_ke(&tt, None, true), Some(tt[1].clone()));
        assert_eq!(
            tieu_diem_ke(&tt, Some(&tt[0]), true),
            Some(tt[1].clone()),
            "ở đầu mà Shift+Tab không vòng về cuối"
        );
    }

    /// **Màn hình không có gì chạm được thì `Tab` không chọn bừa.**
    #[test]
    fn tab_tren_man_trong_khong_chon_gi() {
        assert_eq!(tieu_diem_ke(&[], None, false), None);
    }

    /// **Đích cũ biến mất thì quay về đầu, KHÔNG giữ theo thứ tự.**
    ///
    /// Giữ theo thứ tự nghĩa là màn hình đổi xong, tiêu điểm rơi vào một nút
    /// khác hẳn chỉ vì nó đứng cùng chỗ — và `Enter` tiếp theo chạy một việc
    /// người dùng không định chạy.
    #[test]
    fn dich_bien_mat_thi_ve_dau() {
        let cu = dich("da-xoa", false);
        let tt = vec![dich("a", false), dich("b", false)];
        assert_eq!(tieu_diem_ke(&tt, Some(&cu), false), Some(tt[0].clone()));
    }

    /// **`Enter` trong Ô NHẬP không kích hoạt gì.**
    ///
    /// Trên hộp thoại quyền, "nút gần nhất" có thể là *Cho phép*. Một phím
    /// Enter lạc chỗ ở đó là một câu trả lời người dùng chưa hề đưa ra.
    #[test]
    fn enter_trong_o_nhap_khong_kich_hoat_gi() {
        assert!(phim_kich_hoat(Some(&crate::FocusTarget::Field("PIN".to_owned()))).is_none());
        assert!(phim_kich_hoat(None).is_none());
    }

    /// **`Enter` trên công tắc gạt công tắc, KHÔNG đóng màn hình.**
    ///
    /// Cùng luật đã có cho chuột (`gat_cong_tac_khong_dong_hop_thoai`). Bàn
    /// phím mà đóng màn hình khi gạt thì người dùng bàn phím vừa trả lời hộ
    /// mình những mục chưa đọc — đúng cái luật kia sinh ra để chặn.
    #[test]
    fn enter_tren_cong_tac_khong_dong_man() {
        let mut bat = BTreeSet::new();
        let ct = dich("mang", true);
        let h = phim_kich_hoat(Some(&ct)).expect("công tắc kích hoạt được");
        assert_eq!(sau_cu_bam(Some(h), &mut bat), SauCuBam::VeLai);
        assert!(bat.contains("mang"));
    }

    /// **Tiêu điểm và ô-đang-chọn KHÔNG được lệch nhau.**
    ///
    /// Lệch nghĩa là viền vẽ quanh một ô còn chữ gõ vào chạy sang ô khác.
    #[test]
    fn tieu_diem_va_o_dang_chon_luon_khop() {
        let mut tt = TrangThai::default();
        tt.dat_tieu_diem(Some(crate::FocusTarget::Field("Ghi chú".to_owned())));
        assert_eq!(tt.o_dang_chon.as_deref(), Some("Ghi chú"));

        tt.dat_tieu_diem(Some(dich("nut", false)));
        assert!(
            tt.o_dang_chon.is_none(),
            "tiêu điểm sang NÚT mà chữ gõ vào vẫn chạy về ô nhập cũ"
        );

        tt.dat_tieu_diem(None);
        assert!(tt.o_dang_chon.is_none());
    }

    /// **`Escape` phải HUỶ, và huỷ không phải một câu trả lời.**
    ///
    /// Kiểm ở tầng NGUỒN vì nhánh này nằm trong vòng lặp sự kiện — mở được cửa
    /// sổ trong `#[test]` thì `cargo test` trên macOS đã không chạy nổi. Điều
    /// phải giữ là hai phím đi CHUNG một nhánh: cùng nhánh thì không ai sửa
    /// được một bên mà quên bên kia, và `Escape` không bao giờ trôi thành một
    /// câu trả lời "cho phép".
    #[test]
    fn escape_di_chung_nhanh_voi_dong_cua_so() {
        let than = include_str!("window.rs");
        let Some(sau) = than.split("WindowEvent::CloseRequested").nth(1) else {
            panic!("không tìm thấy nhánh đóng cửa sổ")
        };
        // Cắt tới hết nhánh: `=> *dieu_khien = ControlFlow::Exit,`
        let het = sau
            .find("ControlFlow::Exit,")
            .expect("nhánh đóng cửa sổ phải thoát");
        let nhanh = &sau[..het];
        assert!(
            nhanh.contains("KeyCode::Escape"),
            "Escape KHÔNG đi chung nhánh với đóng cửa sổ — hai đường huỷ có thể trôi khác nhau"
        );
        // Bỏ chú thích trước khi soi: chính chú thích ở đó GIẢI THÍCH vì sao
        // không được đụng `da_bam`, và một phép thử bắt được lời giải thích của
        // chính mình là một phép thử đọc sai thứ nó đang đọc.
        let chi_ma: String = nhanh
            .lines()
            .filter(|d| !d.trim_start().starts_with("//"))
            .collect();
        assert!(
            !chi_ma.contains("da_bam"),
            "nhánh huỷ có đụng tới `da_bam` — huỷ đang trở thành một câu trả lời"
        );
    }

    /// **Đưa ảnh lên màn hình phải dùng chiều rộng THẬT của ảnh.**
    ///
    /// Lỗi này do chính lượt "cho cửa sổ đổi cỡ được" sinh ra: chiều rộng thành
    /// thuộc tính, mà chỗ đưa lên màn hình còn dùng hằng cũ. Mỗi hàng đọc lệch
    /// một đoạn nên ảnh xiên dần — và không phép thử nào lúc ấy nhìn tới đây.
    #[test]
    fn dua_len_man_hinh_dung_chieu_rong_that() {
        let mut bd = RasterRenderer::new();
        bd.set_width(400);
        // Một ô có khung: có mực ở cột trái, gần mép trái của ảnh.
        bd.render(&Node::button("X", "x", tcc_ui::Tone::Neutral).unwrap())
            .unwrap();
        assert_eq!(bd.width(), 400);

        let (rong, cao) = (400_u32, bd.height() as u32);
        let mut ra = vec![0xFFFF_FFFF_u32; (rong * cao) as usize];
        to_mau(&bd, &mut ra, rong, cao, 1.0, 0.0);

        // Đối chiếu từng điểm với ảnh gốc. Lệch chiều rộng là lệch NGAY hàng
        // thứ hai, nên so cả ảnh mới bắt được.
        let anh = bd.image();
        for y in 0..cao as usize {
            for x in 0..rong as usize {
                let xam = u32::from(anh[y * 400 + x]);
                assert_eq!(
                    ra[y * 400 + x] & 0xFF,
                    xam,
                    "điểm ({x},{y}) lệch — chiều rộng dùng sai"
                );
            }
        }
    }

    /// **Thanh cuộn chỉ hiện khi CÒN nội dung ngoài khung.**
    ///
    /// Hiện lúc nào cũng thấy thì nó thành trang trí; không hiện bao giờ thì
    /// phần dưới đáy coi như không tồn tại với người dùng.
    #[test]
    fn thanh_cuon_chi_hien_khi_con_noi_dung() {
        let dem = |ra: &[u32]| ra.iter().filter(|p| **p == 0x0090_9090).count();

        let (rong, cao) = (200_u32, 100_u32);
        let mut vua_khit = vec![0xFFFF_FFFF_u32; (rong * cao) as usize];
        ve_thanh_cuon(&mut vua_khit, rong, cao, 100.0, 0.0);
        assert_eq!(dem(&vua_khit), 0, "nội dung vừa khít mà vẫn vẽ thanh cuộn");

        let mut dai = vec![0xFFFF_FFFF_u32; (rong * cao) as usize];
        ve_thanh_cuon(&mut dai, rong, cao, 400.0, 0.0);
        assert!(dem(&dai) > 0, "nội dung dài gấp bốn mà không có thanh cuộn");

        // Cuộn xuống thì thanh phải ĐI XUỐNG, không đứng im.
        let mut duoi = vec![0xFFFF_FFFF_u32; (rong * cao) as usize];
        ve_thanh_cuon(&mut duoi, rong, cao, 400.0, 300.0);
        let dinh = |ra: &[u32]| {
            (0..cao).find(|y| (0..rong).any(|x| ra[(y * rong + x) as usize] == 0x0090_9090))
        };
        assert!(
            dinh(&duoi) > dinh(&dai),
            "cuộn xuống đáy mà thanh vẫn ở nguyên chỗ cũ"
        );
    }

    /// **Chèn và xoá phải cắt theo CHỮ, không theo byte.**
    ///
    /// Một chữ tiếng Việt có dấu là nhiều byte. Cắt theo byte là cắt vào giữa
    /// một ký tự — và chuỗi kết quả thậm chí không còn là UTF-8 hợp lệ.
    #[test]
    fn chen_xoa_cat_theo_chu_khong_theo_byte() {
        assert_eq!(chen_tai("Tiếng Việt", 5, "!"), "Tiếng! Việt");
        assert_eq!(chen_tai("", 0, "ế"), "ế");
        // Chèn quá cuối thì ghép vào cuối, không hoảng loạn.
        assert_eq!(chen_tai("abc", 99, "d"), "abcd");

        assert_eq!(xoa_truoc("Tiếng", 3), "Ting");
        assert_eq!(xoa_truoc("Tiếng", 0), "Tiếng", "xoá ở đầu mà đổi chuỗi");
        assert_eq!(xoa_tai("Tiếng", 2), "Ting");
        assert_eq!(xoa_tai("Tiếng", 99), "Tiếng", "xoá quá cuối mà đổi chuỗi");
    }

    /// **Xoá đúng CHỮ có dấu, không để lại dấu mồ côi.**
    ///
    /// `ế` là một ký tự Unicode nhưng ba byte. Xoá một byte để lại một chuỗi
    /// hỏng; xoá "một dấu" để lại `ê` hay `e` là đổi thứ người dùng đã gõ.
    #[test]
    fn xoa_mot_lan_di_het_mot_chu_co_dau() {
        assert_eq!(xoa_truoc("ế", 1), "");
        assert_eq!("ế".len(), 3, "giả định về số byte đã sai");
    }

    /// **Vào một ô thì con trỏ đứng ở CUỐI chữ đang có.**
    ///
    /// Đứng ở đầu thì gõ một chữ là chữ ấy nhảy lên trước mọi thứ đã gõ — với
    /// một cụm từ khôi phục hai mươi bốn chữ, đó là hỏng cả cụm.
    #[test]
    fn vao_o_thi_con_tro_o_cuoi() {
        let mut tt = TrangThai::default();
        tt.noi_dung_o
            .insert("Ghi chú".to_owned(), "xin chào".to_owned());
        tt.dat_tieu_diem(Some(crate::FocusTarget::Field("Ghi chú".to_owned())));
        assert_eq!(tt.con_tro, 8, "con trỏ không ở cuối chuỗi đang có");
    }

    /// **Con trỏ không đi ra ngoài chuỗi.**
    #[test]
    fn con_tro_bi_kep_trong_chuoi() {
        let mut tt = TrangThai::default();
        tt.noi_dung_o.insert("Ô".to_owned(), "abc".to_owned());
        tt.o_dang_chon = Some("Ô".to_owned());
        tt.con_tro = 99;
        tt.kep_con_tro();
        assert_eq!(tt.con_tro, 3);
    }
}
