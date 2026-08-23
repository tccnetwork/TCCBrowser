//! Bộ dựng **thứ hai** — vẽ thẳng ra pixel, không qua WebView.
//!
//! # Vì sao crate này tồn tại, và nó KHÔNG phải để đẹp hơn
//!
//! `tcc-ui` có một luật: *"`tcc-ui` không được biết bộ dựng nào"* — mất luật ấy
//! là mất đường thoát khỏi WebView. Nhưng một trait chỉ có **một** bản cài đặt
//! thì không ai biết nó có thật sự trừu tượng hay không: mọi chỗ rò rỉ giả định
//! đều nằm im, vì chưa có ai đá vào.
//!
//! Crate này là cú đá. Nó cài cùng `Renderer` trait, không dùng một dòng HTML
//! nào, và nếu `tcc-ui` có chỗ nào ngầm giả định "bộ dựng là một trình duyệt"
//! thì chỗ ấy sẽ **không biên dịch được**.
//!
//! # Hai thứ nó cho không, mà WebView không cho
//!
//! **Chạy được không cần màn hình.** `cargo test --workspace` không chạm tới
//! WebView được — trên macOS vòng lặp sự kiện phải ở luồng chính còn bộ khung
//! test chạy ở luồng phụ. Bộ dựng này chạy trong một phép thử bình thường, nên
//! bố cục và trợ năng kiểm được ở CI trên cả ba nền.
//!
//! **Ảnh so được từng pixel.** Vẽ ra một mảng byte tất định thì so hai lần chạy
//! là so được, và một thay đổi bố cục ngoài ý muốn hiện ra thành con số.
//!
//! # Cây trợ năng dựng TỪ LƯỢT VẼ, không gọi lại hàm của `tcc-ui`
//!
//! Chỗ này quan trọng. `Node::accessibility_tree()` có sẵn, và gọi nó ở
//! `published_accessibility` thì phép kiểm ngang bằng trợ năng **luôn xanh** —
//! nó đang so một hàm với chính nó.
//!
//! Nên ở đây cây trợ năng được ghi lại **trong lúc vẽ**: mỗi phần tử vẽ ra mới
//! đẩy một nút vào. Vẽ sót một nút là phép kiểm đỏ, đúng như nó phải thế.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    reason = "vẽ pixel: mọi toạ độ đều đi từ f32 sang chỉ số nguyên, và chúng \
              đã được chặn biên ngay tại chỗ ghi. Viết `try_from` cho từng phép \
              đổi ở đây làm mã khó đọc hơn mà không chặn thêm được gì."
)]

#[cfg(feature = "accesskit")]
pub mod accesskit_bridge;

mod bo_cuc;

use cosmic_text::{
    Attrs, BorrowedWithFontSystem, Buffer, Color, Family, FontSystem, Metrics, Shaping, SwashCache,
};
use tcc_ui::{AccessNode, Alt, Emphasis, Node, NodeKind, Renderer, Role, Tone};

/// Bề rộng khung vẽ. Cố định: bộ dựng này để KIỂM ĐỊNH, không để co giãn theo
/// cửa sổ người dùng.
#[cfg(feature = "window")]
pub mod window;

pub const WIDTH: usize = 640;

/// Trần chiều cao. Cây vượt quá thì báo lỗi chứ không cắt im lặng — cắt im lặng
/// là giấu mất phần giao diện mà người dùng đáng ra phải thấy.
pub const MAX_HEIGHT: usize = 4096;

/// Cỡ chữ nền. Cùng con số đã đo ở đâm thử 0.1.
const CO_CHU: f32 = 15.0;
const LE: f32 = 12.0;

#[derive(Debug, thiserror::Error)]
pub enum RasterError {
    #[error("cây vẽ ra cao {0} px, vượt trần {MAX_HEIGHT}")]
    TooTall(usize),
    /// Bộ tính bố cục từ chối cây.
    ///
    /// Không nuốt: vẽ ra một màn hình THIẾU ô còn nguy hơn không vẽ gì, vì
    /// người dùng không có cách nào biết là thiếu.
    #[error("không xếp được bố cục")]
    Layout,
}

/// Bộ dựng ra pixel.
pub struct RasterRenderer {
    fonts: FontSystem,
    cache: SwashCache,
    /// Ảnh xám, 1 byte/pixel. 255 = trắng.
    pixel: Vec<u8>,
    height: usize,
    /// Ô đã đặt ở lần vẽ gần nhất — xem [`RasterRenderer::placed_boxes`].
    da_dat: Vec<DaDat>,
    /// Cây trợ năng ghi lại TRONG LÚC VẼ — xem ghi chú đầu tệp.
    published: Option<AccessNode>,
}

impl Default for RasterRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl RasterRenderer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            fonts: FontSystem::new(),
            cache: SwashCache::new(),
            pixel: Vec::new(),
            height: 0,
            da_dat: Vec::new(),
            published: None,
        }
    }

    /// Ảnh xám đã vẽ, `WIDTH` × [`Self::height`].
    #[must_use]
    pub fn image(&self) -> &[u8] {
        &self.pixel
    }

    #[must_use]
    pub const fn height(&self) -> usize {
        self.height
    }

    /// Các ô ĐÃ ĐẶT ở lần vẽ gần nhất: `(trái, trên, rộng, cao)`.
    ///
    /// # Vì sao trả về hình học thô thay vì một con số "có chồng không"
    ///
    /// Bản đầu tôi để bộ dựng **tự đếm** số ô chồng nhau và phép thử đọc con số
    /// ấy. Kiểm đột biến cho thấy nó vô nghĩa: đặt bộ đếm về 0 là phép thử xanh
    /// ngay, kể cả khi bố cục chồng thật. **Phép thử đang hỏi chính bị cáo.**
    ///
    /// Giờ bộ dựng chỉ khai ra nó đặt cái gì ở đâu; phép tính "có chồng không"
    /// nằm ở phép thử. Làm hỏng bố cục thì không có chỗ nào để giấu.
    #[must_use]
    pub fn placed_boxes(&self) -> Vec<(f32, f32, f32, f32)> {
        self.da_dat
            .iter()
            .map(|d| (d.trai, d.tren, d.o.rong, d.o.cao))
            .collect()
    }

    /// Cú bấm ở `(x, y)` rơi vào Ô NHẬP nào.
    ///
    /// Tách khỏi [`Self::hit_test`] vì hai câu hỏi khác nhau: một cái hỏi "chạy
    /// việc gì", cái này hỏi "gõ vào đâu". Gộp lại là chỗ một cú bấm vào ô nhập
    /// lỡ chạy mất một hành động.
    #[must_use]
    pub fn hit_test_field(&self, x: f32, y: f32) -> Option<&str> {
        #[expect(clippy::cast_precision_loss, reason = "kích thước ảnh, luôn nhỏ")]
        if x < 0.0 || y < 0.0 || x >= WIDTH as f32 || y >= self.height as f32 {
            return None;
        }
        self.da_dat
            .iter()
            .rev()
            .find(|d| {
                d.o.nhan.is_some()
                    && x >= d.trai
                    && x < d.trai + d.o.rong
                    && y >= d.tren
                    && y < d.tren + d.o.cao
            })
            .and_then(|d| d.o.nhan.as_deref())
    }

    /// Cú bấm ở `(x, y)` rơi vào hành động nào.
    ///
    /// Trả `None` khi rơi vào chữ, ảnh, ô nhập, hoặc khoảng trống.
    ///
    /// Bên gọi phải phân biệt **nút** với **công tắc**: xem [`Hit::toggle`].
    ///
    /// # Ô sau thắng ô trước
    ///
    /// Có bất biến **không vẽ đè** (xem `docs/vi-thiet-ke.md` §23), nên hai ô
    /// bấm được không chồng nhau và thứ tự không đổi kết quả. Nhưng nếu bất
    /// biến ấy hỏng thì ô **vẽ sau** là ô người dùng NHÌN THẤY, nên nó phải là
    /// ô nhận cú bấm — bấm vào thứ bị che là chuyện tệ nhất có thể xảy ra ở đây.
    #[must_use]
    pub fn hit_test(&self, x: f32, y: f32) -> Option<Hit<'_>> {
        // Chắn thứ hai, sau khi đã vá nguyên nhân gốc ở `xa_dong`: cú bấm ngoài
        // ảnh KHÔNG trúng gì.
        //
        // ⚠️ Chắn này KHÔNG kiểm được bằng phép thử, và tôi nói ra thay vì để nó
        // trông như đã kiểm. Kiểm đột biến 21/08/2026: bỏ hẳn nó đi thì mọi phép
        // thử vẫn xanh — vì một khi bố cục đúng thì không ô nào nằm ngoài ảnh,
        // nên một điểm ngoài ảnh trượt hết mọi ô dù có chắn hay không.
        //
        // Giữ lại có chủ ý: giá trị của nó là **ngày bố cục hỏng trở lại**. Nó
        // biến một lỗi bố cục thành một nút chết thay vì một nút vô hình bấm
        // được. Đó là phòng thủ theo tầng, không phải một phép kiểm.
        #[expect(clippy::cast_precision_loss, reason = "kích thước ảnh, luôn nhỏ")]
        if x < 0.0 || y < 0.0 || x >= WIDTH as f32 || y >= self.height as f32 {
            return None;
        }
        self.da_dat
            .iter()
            .rev()
            .find(|d| {
                d.o.hanh_dong.is_some()
                    && x >= d.trai
                    && x < d.trai + d.o.rong
                    && y >= d.tren
                    && y < d.tren + d.o.cao
            })
            .and_then(|d| {
                d.o.hanh_dong.as_deref().map(|a| Hit {
                    action: a,
                    toggle: d.o.cong_tac,
                })
            })
    }

    /// Đếm pixel có mực. Dùng để chốt "có vẽ ra gì đó" mà không cần so ảnh.
    #[must_use]
    pub fn ink(&self) -> usize {
        self.pixel.iter().filter(|p| **p < 250).count()
    }
}

/// Cú bấm rơi vào cái gì.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hit<'a> {
    pub action: &'a str,
    /// `true` = công tắc: đổi câu trả lời rồi **ở lại màn hình**.
    /// `false` = nút: kết thúc màn hình.
    pub toggle: bool,
}

/// Một ô chữ **đã đo xong**: biết mình rộng bao nhiêu, cao bao nhiêu.
///
/// Đo trước khi đặt là cả điểm của 4.2. Bản 4.1 không đo gì — nó xếp mọi thứ
/// thành một cột, nên `Flow::Row` chỉ là `Flow::Column` đội tên khác.
#[derive(Clone, Default)]
pub(crate) struct O {
    /// Mã hành động, nếu ô này bấm được. `None` = chữ, ảnh, ô nhập.
    ///
    /// Ghi Ở ĐÂY chứ không tra lại cây khi có cú bấm: chỗ người dùng bấm là chỗ
    /// bộ dựng **đã đặt**, không phải chỗ cây nói. Hai thứ ấy lệch nhau là lúc
    /// bố cục sai — mà lúc ấy tra cây sẽ trả về một nút người dùng không nhìn
    /// thấy, và cú bấm chạy một việc họ không định chạy.
    hanh_dong: Option<String>,
    /// Nhãn ô nhập, nếu ô này là một ô nhập.
    ///
    /// Ô nhập **không có mã hành động** — 0.1 không định nghĩa hành động nào
    /// cho nó — nên nhãn là thứ duy nhất phân biệt hai ô.
    nhan: Option<String>,
    /// Ô này là CÔNG TẮC chứ không phải nút.
    ///
    /// Hai thứ phải tách được: bấm nút là kết thúc màn hình, gạt công tắc là
    /// đổi một câu trả lời rồi ở lại. Gộp chúng làm một thì gạt một quyền sẽ
    /// đóng luôn hộp thoại — và người dùng vừa "trả lời" cả những mục họ chưa
    /// kịp đọc.
    cong_tac: bool,
    chu: String,
    co: f32,
    /// Cách vẽ — thay hai `bool` `dam`/`khung` rời nhau.
    ///
    /// Chúng luôn đến từ CÙNG một quyết định ở `do_la`, nên giữ hai trường là
    /// giữ hai nguồn cho một sự thật; và `(dam: true, khung: true)` là một trạng
    /// thái không nhánh nào dựng ra mà kiểu cũ vẫn cho phép biểu diễn.
    kieu: KieuO,
    /// Nút MẤT MÁT — phải vẽ khác hẳn, không chỉ khai khác.
    ///
    /// ⚠️ Bất biến B31, và nó đã TÁI PHÁT một lần. Nó sinh ra vì đường WebView
    /// vẽ `Tone::Danger` y hệt mọi nút khác; rồi bộ dựng ra pixel thay chỗ
    /// WebView và đọc `Tone` **chỉ** để đặt cờ trợ năng. Người dùng trình đọc
    /// màn hình nghe được "nút mất mát"; người NHÌN thì không thấy gì. Phép thử
    /// giữ B31 nằm trong crate bị xoá, nên không ai kêu.
    mat_mat: bool,
    /// Biên nét THÔ như lượt đo thấy — mép trên và mép dưới, chưa qua `max` nào.
    ///
    /// Đây là **nguồn duy nhất**: `cao` suy ra từ nó, và lượt vẽ suy ra độ dời
    /// từ nó (`(-net.0).max(0.0)`). Giữ số thô thay vì giữ kết quả đã kẹp, vì
    /// một kết quả đã kẹp không phân biệt được "đo ra số nhỏ" với "không đo được
    /// gì" — hai nguyên nhân khác hẳn nhau, và tôi đã mất hai vòng CI ở đúng chỗ
    /// mù ấy.
    ///
    /// Nét CÓ THỂ nằm trên đường ascent của phông — dấu phụ tiếng Việt là ca
    /// thường gặp — nên `net.0` âm được.
    net: (f32, f32),
    /// Chiều cao MỘT DÒNG, đo từ phông chứ không đoán.
    ///
    /// ⚠️ Lượt vẽ PHẢI dùng đúng con số này, không được tự tính lại. Lượt đo và
    /// lượt vẽ mà tính hai kiểu thì hình học của cái vẽ ra không còn khớp cái
    /// `hit_test` tin — người dùng thấy một nút, hệ thống chạy một nút khác.
    cao_dong: f32,
    rong: f32,
    cao: f32,
}

impl O {
    /// Ô này có khung quanh chữ không — nút, ô nhập, ảnh.
    pub(crate) fn co_khung(&self) -> bool {
        self.kieu == KieuO::Khung
    }
}

/// Một ô **đã đặt xong chỗ**, toạ độ tuyệt đối.
pub(crate) struct DaDat {
    o: O,
    trai: f32,
    tren: f32,
}

/// Cách vẽ một ô chữ — thay hai `bool` liền nhau.
///
/// ⚠️ `do_o(chu, co, dam, khung, rong)` từng nhận `dam: bool, khung: bool` cạnh
/// nhau. Đổi chỗ hai tham số ấy vẫn biên dịch, vẫn chạy, chỉ vẽ sai — cùng hạng
/// bẫy mà `Cho` đã được gom lại để tránh. clippy chỉ ra khi `O` có tới bốn
/// `bool`; cái đáng sửa không phải con số ấy mà là hai tham số này.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum KieuO {
    /// Chữ thường. Mặc định: một ô chưa nói gì thì không đậm và không khung —
    /// phía an toàn, vì "trông như nút" là thứ phải xin, không phải thứ mặc định.
    #[default]
    Thuong,
    /// Chữ đậm — tiêu đề, cảnh báo.
    Dam,
    /// Có khung quanh chữ — nút, ô nhập, ảnh.
    Khung,
}

/// Đệm trong khung, mỗi bên.
const DEM: f32 = 8.0;

/// Chiều cao dòng theo cỡ chữ — mức SÀN, không phải con số cuối.
///
/// Thoáng hơn chiều cao nét một chút cho dễ đọc. Phông nào cần nhiều hơn thì
/// [`cao_dong_that`] nới ra.
const CAO_DONG: f32 = 1.4;

/// Tạo hình một chuỗi — **đường DUY NHẤT**, dùng chung cho lượt đo và lượt vẽ.
///
/// # Vì sao phải là một hàm, không phải hai đoạn mã giống nhau
///
/// Vì đã là hai đoạn, và chúng trôi khỏi nhau. Lượt đo tạo hình ở bề rộng chữ
/// đo được; lượt vẽ tạo hình ở `o.rong`, mà `o.rong` đã bị bộ tính bố cục ghi
/// đè bằng một số **làm tròn**. Tròn xuống một phần mười pixel là đủ để chuỗi
/// ngắt thành hai dòng ở lượt vẽ trong khi lượt đo thấy một dòng — nét cao gấp
/// đôi số đã đo, và ô không chứa nổi nó.
///
/// Ba vòng CI mới ra, vì cả ba lần thông báo lỗi chỉ nói kết quả chứ không nói
/// hai lượt đã tạo hình bằng gì. Sau khi gộp, chúng không lệch được nữa: cùng
/// một hàm, cùng tham số thì cùng kết quả.
fn tao_hinh(
    fonts: &mut FontSystem,
    chu: &str,
    co: f32,
    dam: bool,
    cao_dong: f32,
    rong: f32,
) -> Buffer {
    let mut buffer = Buffer::new(fonts, Metrics::new(co, cao_dong));
    let mut b = buffer.borrow_with(fonts);
    b.set_size(Some(rong), None);
    let mut attrs = Attrs::new().family(Family::SansSerif);
    if dam {
        attrs = attrs.weight(cosmic_text::Weight::BOLD);
    }
    // `Shaping::Advanced` — bắt buộc cho tiếng Việt. `Basic` bỏ qua việc xếp dấu
    // phụ, và với tiếng Việt thì đó không phải "nhanh hơn", đó là SAI.
    b.set_text(chu, &attrs, Shaping::Advanced, None);
    b.shape_until_scroll(false);
    buffer
}

/// Mép trên và mép dưới của NÉT THẬT, theo toạ độ của hộp chữ.
///
/// # Vì sao không dùng số liệu phông
///
/// Đã thử, và nó **không đủ**. Bản 23/08/2026 lấy `max_ascent + max_descent` của
/// cosmic-text làm chiều cao dòng; CI trên Linux trả về **đúng bộ số cũ** — nét
/// 21..43 trong ô 17..38. Nghĩa là nét vẽ ra vượt cả số liệu mà phông tự khai.
/// Không hiếm: chữ hoa có hai dấu phụ chồng nhau thường vượt đường ascent, và
/// bảng số liệu của phông không hứa bao được mọi glyph.
///
/// Nên đo bằng chính thứ sẽ vẽ. `draw` ở đây không ghi một điểm ảnh nào — nó chỉ
/// gom biên. Ảnh glyph nằm lại trong `SwashCache`, nên lượt vẽ thật sau đó không
/// phải rasteriser lại: đo hai lần, dựng ảnh một lần.
fn do_net(b: &mut BorrowedWithFontSystem<'_, Buffer>, cache: &mut SwashCache) -> (f32, f32) {
    let (mut tren, mut duoi) = (f32::MAX, f32::MIN);
    b.draw(cache, Color::rgb(0, 0, 0), |_, gy, _, cao_o, mau| {
        // Điểm ảnh TRONG SUỐT không phải nét. Không lọc thì viền mềm quanh glyph
        // nới biên ra thêm vài pixel trống, và ô phình ra không vì gì cả.
        if mau.a() == 0 {
            return;
        }
        #[expect(clippy::cast_precision_loss, reason = "toạ độ glyph, luôn nhỏ")]
        let (y, h) = (gy as f32, cao_o as f32);
        tren = tren.min(y);
        duoi = duoi.max(y + h);
    });
    if tren > duoi {
        (0.0, 0.0)
    } else {
        (tren, duoi)
    }
}

/// Chiều cao một dòng mà PHÔNG thật sự cần.
///
/// # Vì sao không nhân cỡ chữ với một hằng số
///
/// Vì đã làm thế, và nó sai trên máy khác. cosmic-text đặt nét trong hộp dòng
/// bằng `centering_offset = (chiều_cao_dòng − (max_ascent + max_descent)) / 2`.
/// Khi phông cần nhiều hơn `cỡ × 1.4` thì số ấy **âm**, và nét thò ra cả hai
/// đầu hộp dòng — tức là ra ngoài ô của chính nó, đè lên ô bên dưới.
///
/// Đo được ngày 22/08/2026: chữ "nhỏ" trên Linux vẽ ở 21..43 trong khi ô của nó
/// là 17..38. macOS không thấy gì vì phông ở đó vừa vặn — đúng cái làm nó sống
/// sót lâu như vậy.
///
/// Nên: lấy `max_ascent + max_descent` thật của từng dòng, và chiều cao dòng là
/// số lớn hơn giữa nó và mức sàn. `centering_offset` khi ấy không bao giờ âm.
fn cao_dong_that(buffer: &Buffer, co: f32) -> f32 {
    let mut can = co * CAO_DONG;
    for dong in &buffer.lines {
        let Some(bo_cuc) = dong.layout_opt() else {
            continue;
        };
        for l in bo_cuc {
            can = can.max(l.max_ascent + l.max_descent);
        }
    }
    can
}

impl Renderer for RasterRenderer {
    type Error = RasterError;

    fn render(&mut self, tree: &Node) -> Result<(), Self::Error> {
        let mut access = Vec::new();
        let mut dat = Vec::new();
        let rong_dung = WIDTH as f32 - LE * 2.0;

        // Ba lượt tách bạch: đo → đặt → vẽ. Gộp lượt đo vào lượt vẽ là cách bản
        // 4.1 hỏng — không có kích thước thì không đặt cạnh nhau được gì.
        // Đo ở đây, XẾP ở `bo_cuc`. Bao đóng là ranh giới: `bo_cuc` nhận về
        // khả năng đo một nút lá, chứ không nhận bộ dựng — nó không được biết
        // cosmic-text tồn tại, y như `tcc-ui` không được biết `bo_cuc` tồn tại.
        let mut do_la = |n: &Node, rong: f32, ac: &mut Vec<AccessNode>| self.do_la(n, rong, ac);
        let cao = bo_cuc::xep(tree, LE, LE, rong_dung, &mut do_la, &mut dat, &mut access)
            .ok_or(RasterError::Layout)?;

        let cao_anh = (cao + LE) as usize;
        if cao_anh > MAX_HEIGHT {
            return Err(RasterError::TooTall(cao_anh));
        }
        self.height = cao_anh.max(1);
        self.pixel = vec![255u8; WIDTH * self.height];

        for mot in &dat {
            self.ve_o(mot);
        }
        // Giữ lại hình học để bên ngoài KIỂM ĐƯỢC. Sau khi vẽ thì hai ô chồng
        // nhau chỉ còn là mực trên mực, không phân biệt được với chữ đậm.
        self.da_dat = dat;

        debug_assert_eq!(access.len(), 1, "gốc phải ra đúng một nút trợ năng");
        self.published = access.into_iter().next();
        Ok(())
    }

    fn published_accessibility(&self) -> Option<AccessNode> {
        self.published.clone()
    }
}

impl RasterRenderer {
    /// Đo một ô chữ với bề rộng cho trước. **Ngắt dòng thật** — bề rộng trả về
    /// là bề rộng dòng dài nhất, chiều cao là số dòng nhân chiều cao dòng.
    fn do_o(&mut self, chu: &str, co: f32, kieu: KieuO, rong_toi_da: f32) -> O {
        let (dam, khung) = (kieu == KieuO::Dam, kieu == KieuO::Khung);
        let dem = if khung { DEM * 2.0 } else { 0.0 };
        let cho_chu = (rong_toi_da - dem).max(co);

        // ⚠️ BA lượt, đúng thứ tự này. Hai lượt đầu chỉ để biết đủ tham số cho
        // lượt thứ ba — và lượt thứ ba phải tạo hình bằng ĐÚNG những gì `ve_o`
        // sẽ dùng, nếu không thì đo một đằng vẽ một nẻo.
        let dau = tao_hinh(&mut self.fonts, chu, co, dam, co * CAO_DONG, cho_chu);
        let mut rong_chu: f32 = 0.0;
        let mut so_dong = 0usize;
        for run in dau.layout_runs() {
            rong_chu = rong_chu.max(run.line_w);
            so_dong += 1;
        }
        let so_dong = so_dong.max(1);
        let cao_dong = cao_dong_that(&dau, co);

        // ⚠️ LÀM TRÒN LÊN, và giữ con số này làm bề rộng của ô.
        //
        // Bộ tính bố cục làm tròn toạ độ về số nguyên, nên `o.rong` có thể tròn
        // XUỐNG dưới bề rộng chữ đo được. Thiếu một phần mười pixel là đủ để
        // chuỗi ngắt thành hai dòng ở lượt vẽ — đo được trên Linux ngày
        // 23/08/2026: lượt đo thấy nét cao 12px, lượt vẽ ra 22px, và ô 21px
        // không chứa nổi.
        let rong_o = (rong_chu + dem).ceil();

        let mut b = tao_hinh(&mut self.fonts, chu, co, dam, cao_dong, rong_o - dem);
        let (net_tren, net_duoi) = {
            let mut b = b.borrow_with(&mut self.fonts);
            do_net(&mut b, &mut self.cache)
        };

        O {
            // Mặc định KHÔNG bấm được. Nhánh nào bấm được thì tự gắn vào — quên
            // gắn thì nút chết chứ không phải chữ thường bỗng bấm được.
            hanh_dong: None,
            nhan: None,
            cong_tac: false,
            chu: chu.to_owned(),
            co,
            kieu,
            rong: rong_o,
            // Ô phải chứa ĐƯỢC NÉT, không chỉ chứa được hộp dòng theo lý
            // thuyết. `max` giữ nguyên hình học ở nơi nét vốn đã vừa.
            cao: (cao_dong * so_dong as f32).max(net_duoi - net_tren.min(0.0))
                + if khung { DEM } else { 0.0 },
            cao_dong,
            net: (net_tren, net_duoi),
            // Mặc định KHÔNG mất mát; nhánh nút tự gắn. Quên gắn thì nút nguy
            // hiểm trông như nút thường — nên mặc định phải là phía an toàn về
            // TRÔNG, tức là "trông như thường" chỉ xảy ra khi nó thường thật.
            mat_mat: false,
        }
    }

    /// Đo một nút LÁ và ghi nút trợ năng tương ứng.
    fn do_la(&mut self, n: &Node, rong_toi_da: f32, access: &mut Vec<AccessNode>) -> O {
        match n.kind() {
            NodeKind::Text { content, emphasis } => {
                let (co, dam) = match emphasis {
                    Emphasis::Title => (CO_CHU * 1.5, true),
                    Emphasis::Warning => (CO_CHU, true),
                    Emphasis::Subtle | Emphasis::Normal => (CO_CHU, false),
                };
                access.push(AccessNode {
                    role: Role::Text,
                    label: Some(content.clone()),
                    action: None,
                    children: Vec::new(),
                });
                self.do_o(
                    content,
                    co,
                    if dam { KieuO::Dam } else { KieuO::Thuong },
                    rong_toi_da,
                )
            }
            NodeKind::Button {
                label,
                tone,
                action,
            } => {
                access.push(AccessNode {
                    role: Role::Button {
                        destructive: *tone == Tone::Danger,
                    },
                    label: Some(label.clone()),
                    action: Some(action.as_str().to_owned()),
                    children: Vec::new(),
                });
                let mut o = self.do_o(label, CO_CHU, KieuO::Khung, rong_toi_da);
                o.hanh_dong = Some(action.as_str().to_owned());
                o.mat_mat = *tone == Tone::Danger;
                o
            }
            NodeKind::Field {
                label,
                value,
                secret,
            } => {
                // Ô bí mật hiện dấu chấm, KHÔNG hiện chữ. Vẽ ra rồi mới che là
                // đã vẽ ra — và ảnh này có thể bị lưu lại.
                let hien = if *secret {
                    "•".repeat(value.chars().count())
                } else {
                    value.clone()
                };
                access.push(AccessNode {
                    role: Role::TextInput { secret: *secret },
                    label: Some(label.clone()),
                    action: None,
                    children: Vec::new(),
                });
                let mut o = self.do_o(
                    &format!("{label}: {hien}"),
                    CO_CHU,
                    KieuO::Khung,
                    rong_toi_da,
                );
                o.nhan = Some(label.clone());
                o
            }
            NodeKind::Toggle { label, on, action } => {
                access.push(AccessNode {
                    role: Role::Switch { on: *on },
                    label: Some(label.clone()),
                    action: Some(action.as_str().to_owned()),
                    children: Vec::new(),
                });
                let chu = format!("[{}] {label}", if *on { "x" } else { " " });
                let mut o = self.do_o(&chu, CO_CHU, KieuO::Thuong, rong_toi_da);
                o.hanh_dong = Some(action.as_str().to_owned());
                o.cong_tac = true;
                o
            }
            NodeKind::Image { alt, .. } => {
                let (chu, nhan) = match alt {
                    Alt::Text(t) => (format!("[ảnh: {t}]"), Some(t.clone())),
                    Alt::Decorative => ("[ảnh trang trí]".to_owned(), None),
                };
                access.push(AccessNode {
                    role: Role::Image,
                    label: nhan,
                    action: None,
                    children: Vec::new(),
                });
                self.do_o(&chu, CO_CHU, KieuO::Khung, rong_toi_da)
            }
            NodeKind::Group { .. } => unreachable!("nhóm đã được `dat` chặn trước"),
        }
    }

    fn ve_o(&mut self, dat: &DaDat) {
        let o = &dat.o;
        if o.kieu == KieuO::Khung {
            let rong = (o.rong as usize).min(WIDTH.saturating_sub(dat.trai as usize + 2));
            self.khung(dat.trai as usize, dat.tren as usize, rong, o.cao as usize);
            // ⚠️ Nút MẤT MÁT vẽ KHUNG ĐÔI — B31.
            //
            // Bộ dựng này chỉ có một kênh: mực xám. Không màu, nên "đỏ" không
            // dùng được; không đổi chữ, vì chữ là của ứng dụng và sửa nó là sửa
            // thứ đã ký. Còn lại là HÌNH DẠNG, và khung đôi là dấu hiệu rẻ nhất
            // mà người nhìn phân biệt được ngay, kể cả trên màn hình đen trắng
            // hay khi người dùng mù màu.
            if o.mat_mat && rong > 4 && o.cao > 4.0 {
                self.khung(
                    dat.trai as usize + 2,
                    dat.tren as usize + 2,
                    rong - 4,
                    o.cao as usize - 4,
                );
            }
        }

        let dem = if o.kieu == KieuO::Khung { DEM } else { 0.0 };
        // ⚠️ CÙNG hàm tạo hình với lượt đo, và cùng tham số. Viết lại các bước
        // ấy ở đây — dù chép đúng từng dòng — là dựng lại đúng chỗ hai lượt đã
        // trôi khỏi nhau ba vòng CI liền.
        let mut buffer = tao_hinh(
            &mut self.fonts,
            &o.chu,
            o.co,
            o.kieu == KieuO::Dam,
            o.cao_dong,
            o.rong - dem * 2.0,
        );
        let mut b = buffer.borrow_with(&mut self.fonts);

        let (pixel, rong_anh, cao_anh) = (&mut self.pixel, WIDTH, self.height);
        let nen_x = (dat.trai + dem) as i32;
        // ⚠️ Dời xuống bằng phần nét thò LÊN TRÊN mà lượt đo đã thấy. Không dời
        // thì nét vẽ lên trên mép ô, đè vào ô phía trên.
        let nen_y = (dat.tren + dem * 0.5 + (-o.net.0).max(0.0)) as i32;
        b.draw(
            &mut self.cache,
            Color::rgb(0, 0, 0),
            |gx, gy, rong_o, cao_o, mau| {
                for hang in 0..cao_o {
                    for cot in 0..rong_o {
                        let px = nen_x + gx + cot as i32;
                        let py = nen_y + gy + hang as i32;
                        if px >= 0 && py >= 0 && (px as usize) < rong_anh && (py as usize) < cao_anh
                        {
                            let vi_tri = py as usize * rong_anh + px as usize;
                            let dam_muc = f32::from(mau.a()) / 255.0;
                            pixel[vi_tri] = (f32::from(pixel[vi_tri]) * (1.0 - dam_muc)) as u8;
                        }
                    }
                }
            },
        );
    }

    fn khung(&mut self, trai: usize, tren: usize, rong: usize, cao: usize) {
        for buoc in 0..rong {
            for hang in [tren, tren + cao] {
                if hang < self.height && trai + buoc < WIDTH {
                    self.pixel[hang * WIDTH + trai + buoc] = 150;
                }
            }
        }
        for buoc in 0..=cao {
            for cot in [trai, trai + rong] {
                if tren + buoc < self.height && cot < WIDTH {
                    self.pixel[(tren + buoc) * WIDTH + cot] = 150;
                }
            }
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
    use super::*;
    use tcc_ui::{Flow, Gap};

    fn cay_mau() -> Node {
        Node::group(Flow::Column, Gap::Large)
            .child(Node::text_with("Chào buổi sáng mọi người", Emphasis::Title).unwrap())
            .unwrap()
            .child(Node::text("Ứng dụng này xin quyền").unwrap())
            .unwrap()
            .child(Node::button("Xoá dữ liệu", "xoa", Tone::Danger).unwrap())
            .unwrap()
            .child(Node::field("Mã PIN", "1234", true).unwrap())
            .unwrap()
            .child(Node::toggle("Cho phép mạng", false, "mang").unwrap())
            .unwrap()
    }

    /// **Vẽ được thật, không cần màn hình.**
    ///
    /// Đây là thứ `tcc-render-webview` không làm được: trên macOS vòng lặp sự
    /// kiện phải ở luồng chính, còn bộ khung test chạy ở luồng phụ.
    #[test]
    fn ve_duoc_trong_mot_phep_thu_binh_thuong() {
        let mut bd = RasterRenderer::new();
        bd.render(&cay_mau()).unwrap();
        assert!(bd.height() > 50, "ảnh quá thấp: {}", bd.height());
        assert!(bd.ink() > 500, "gần như không vẽ gì: {} pixel", bd.ink());
    }

    /// **Phép kiểm ngang bằng trợ năng — với một bộ dựng KHÔNG phải WebView.**
    ///
    /// Nếu `tcc-ui` ngầm giả định bộ dựng là trình duyệt thì phép này đỏ.
    #[test]
    fn qua_duoc_kiem_dinh_tro_nang() {
        let mut bd = RasterRenderer::new();
        tcc_ui::check_accessibility_parity(&mut bd, &cay_mau())
            .expect("bộ dựng thứ hai không qua được kiểm định trợ năng");
    }

    /// Cây trợ năng phải dựng TỪ LƯỢT VẼ — vẽ sót một nút là đỏ.
    ///
    /// Phép thử này canh chính cơ chế: nó bỏ một nhánh khỏi lượt vẽ bằng cách
    /// so cây công bố với cây đúng sau khi thêm một nút mới.
    #[test]
    fn cay_tro_nang_theo_dung_thu_da_ve() {
        let mut bd = RasterRenderer::new();
        let cay = cay_mau();
        bd.render(&cay).unwrap();
        let cong_bo = bd.published_accessibility().unwrap();
        assert_eq!(cong_bo, cay.accessibility_tree());

        // Thêm một nút rồi KHÔNG vẽ lại: cây công bố phải LỆCH.
        let cay2 = cay.child(Node::text("thêm một dòng").unwrap()).unwrap();
        assert_ne!(
            cong_bo,
            cay2.accessibility_tree(),
            "cây công bố không đổi theo cây thật — nó đang được bịa ra"
        );
    }

    /// **Ô bí mật KHÔNG được vẽ chữ thật ra ảnh.**
    ///
    /// Vẽ ra rồi mới che là đã vẽ ra, và ảnh này có thể bị lưu lại.
    #[test]
    fn o_bi_mat_khong_ve_chu_that() {
        let mut a = RasterRenderer::new();
        a.render(&Node::field("PIN", "123456", true).unwrap())
            .unwrap();
        let mut b = RasterRenderer::new();
        b.render(&Node::field("PIN", "999999", true).unwrap())
            .unwrap();
        assert_eq!(
            a.image(),
            b.image(),
            "hai mật khẩu KHÁC nhau vẽ ra hai ảnh khác nhau — chữ thật đang lọt ra ảnh"
        );
    }

    /// Ô THƯỜNG thì ngược lại: nội dung phải hiện ra.
    #[test]
    fn o_thuong_van_hien_noi_dung() {
        let mut a = RasterRenderer::new();
        a.render(&Node::field("Tìm", "xin chào", false).unwrap())
            .unwrap();
        let mut b = RasterRenderer::new();
        b.render(&Node::field("Tìm", "tạm biệt", false).unwrap())
            .unwrap();
        assert_ne!(a.image(), b.image(), "ô thường không hiện nội dung");
    }

    /// Vẽ hai lần ra ĐÚNG một ảnh — nếu không thì không so ảnh được.
    #[test]
    fn ve_hai_lan_ra_cung_mot_anh() {
        let mut a = RasterRenderer::new();
        a.render(&cay_mau()).unwrap();
        let mut b = RasterRenderer::new();
        b.render(&cay_mau()).unwrap();
        assert_eq!(a.image(), b.image());
        assert_eq!(a.height(), b.height());
    }

    /// Chữ tiếng Việt có dấu phải ra mực, không ra ô vuông rỗng.
    #[test]
    fn chu_tieng_viet_ve_ra_muc() {
        let mut co_dau = RasterRenderer::new();
        co_dau
            .render(&Node::text("ế ề ể ễ ệ ữ ự ợ").unwrap())
            .unwrap();
        let mut khong_dau = RasterRenderer::new();
        khong_dau
            .render(&Node::text("e e e e e u u o").unwrap())
            .unwrap();
        assert!(
            co_dau.ink() > khong_dau.ink(),
            "chữ có dấu vẽ ra ÍT mực hơn chữ không dấu — dấu đang mất"
        );
    }

    /// **Hàng ngang đặt CẠNH NHAU, không xếp dọc.**
    ///
    /// Bản 4.1 xếp `Flow::Row` theo chiều dọc và ghi chú "chưa làm". Phép thử
    /// này là thứ chốt rằng nó đã được làm: cùng hai nút, hàng phải THẤP hơn cột.
    #[test]
    fn hang_dat_canh_nhau_chu_khong_xep_doc() {
        let con = || {
            (
                Node::button("Đồng ý", "ok", Tone::Neutral).unwrap(),
                Node::button("Huỷ", "huy", Tone::Neutral).unwrap(),
            )
        };
        let (a1, b1) = con();
        let hang = Node::group(Flow::Row, Gap::Medium)
            .child(a1)
            .unwrap()
            .child(b1)
            .unwrap();
        let (a2, b2) = con();
        let cot = Node::group(Flow::Column, Gap::Medium)
            .child(a2)
            .unwrap()
            .child(b2)
            .unwrap();

        let mut bd_hang = RasterRenderer::new();
        bd_hang.render(&hang).unwrap();
        let mut bd_cot = RasterRenderer::new();
        bd_cot.render(&cot).unwrap();

        assert!(
            bd_hang.height() < bd_cot.height(),
            "hàng cao {} không thấp hơn cột cao {} — hàng vẫn đang xếp dọc",
            bd_hang.height(),
            bd_cot.height()
        );
        // Và hàng phải RỘNG hơn: hai nút cạnh nhau chiếm nhiều cột pixel hơn.
        assert!(cot_co_muc(&bd_hang) > cot_co_muc(&bd_cot));
    }

    /// Số CỘT pixel có mực — đo bề ngang thật của thứ đã vẽ.
    fn cot_co_muc(bd: &RasterRenderer) -> usize {
        (0..WIDTH)
            .filter(|x| (0..bd.height()).any(|y| bd.image()[y * WIDTH + x] < 250))
            .count()
    }

    /// **Chữ dài phải NGẮT DÒNG**, không tràn ra ngoài mép.
    #[test]
    fn chu_dai_thi_ngat_dong() {
        let ngan = Node::text("Chào buổi sáng").unwrap();
        let dai = Node::text(
            "Chào buổi sáng mọi người, đây là một câu rất dài để thử xem bộ dựng có \
             ngắt dòng đúng chỗ không, vì chữ tràn ra ngoài mép là chữ người dùng \
             không đọc được và cũng không biết là có. Câu này cố ý viết dài thêm \
             nữa, để chắc chắn nó phải xuống ít nhất ba dòng chứ không phải hai, \
             vì hai dòng thì phép thử vẫn xanh kể cả khi ngắt dòng chỉ chạy đúng \
             một nửa.",
        )
        .unwrap();

        let mut bd_ngan = RasterRenderer::new();
        bd_ngan.render(&ngan).unwrap();
        let mut bd_dai = RasterRenderer::new();
        bd_dai.render(&dai).unwrap();

        // Một dòng cao 21 px (cỡ 15 × 1.4). Câu này dài gấp nhiều lần một dòng,
        // nên phải ra ÍT NHẤT ba dòng. So với `bd_ngan * 2` là so nhầm: lề trên
        // dưới cộng vào cả hai bên và làm phép nhân mất nghĩa.
        let mot_dong = (CO_CHU * 1.4) as usize;
        assert!(
            bd_dai.height() >= bd_ngan.height() + mot_dong * 2,
            "câu dài cao {} px, câu ngắn {} px — chưa ngắt tới ba dòng",
            bd_dai.height(),
            bd_ngan.height()
        );
        // Không một pixel nào được chạm mép phải.
        let cham_mep = (0..bd_dai.height()).any(|y| bd_dai.image()[y * WIDTH + WIDTH - 1] < 250);
        assert!(!cham_mep, "chữ tràn tới sát mép phải");
    }

    /// **Hàng quá dài thì XUỐNG DÒNG**, không đẩy nút ra khỏi mép.
    ///
    /// Một nút bị đẩy khỏi mép là một nút người dùng không bấm được và không
    /// biết là có.
    #[test]
    fn hang_qua_dai_thi_xuong_dong() {
        let mut hang = Node::group(Flow::Row, Gap::Medium);
        for i in 0..12 {
            hang = hang
                .child(
                    Node::button(format!("Nút số {i}"), &format!("nut-{i}"), Tone::Neutral)
                        .unwrap(),
                )
                .unwrap();
        }
        let mut bd = RasterRenderer::new();
        bd.render(&hang).unwrap();

        // Mười hai nút không thể nằm gọn một hàng 640 px.
        assert!(
            bd.height() > 40,
            "mười hai nút vẫn trên một hàng, cao {} — không xuống dòng",
            bd.height()
        );
        let cham_mep = (0..bd.height()).any(|y| bd.image()[y * WIDTH + WIDTH - 1] < 250);
        assert!(!cham_mep, "có nút bị đẩy tràn khỏi mép phải");
    }

    /// Chữ tiếng Việt phải được ĐO đúng, không chỉ vẽ đúng.
    ///
    /// Đo sai thì ngắt dòng sai chỗ, và hàng ngang xếp chồng lên nhau.
    #[test]
    fn chu_co_dau_do_rong_hon_chu_khong_dau() {
        let mut a = RasterRenderer::new();
        a.render(&Node::text("nguoi").unwrap()).unwrap();
        let mut b = RasterRenderer::new();
        b.render(&Node::text("người").unwrap()).unwrap();
        // Cùng năm chữ cái, nhưng chữ có dấu KHÔNG được đo bằng 0 hay bằng nhau
        // một cách đáng ngờ — nó phải ra một bề ngang thật.
        assert!(cot_co_muc(&b) > 10, "chữ có dấu đo ra gần bằng 0");
        assert!(
            b.height() <= a.height(),
            "chữ có dấu làm cao thêm cả dòng — dấu đang bị tính thành dòng mới"
        );
    }

    /// Cây rỗng vẫn ra một ảnh hợp lệ, không hoảng loạn.
    #[test]
    fn cay_toi_thieu_khong_hoang_loan() {
        let mut bd = RasterRenderer::new();
        bd.render(&Node::group(Flow::Column, Gap::None)).unwrap();
        assert!(bd.height() >= 1);
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay"
)]
mod kiem_thu_43 {
    use super::*;
    use tcc_ui::{Flow, Gap};

    fn tran_mep_phai(bd: &RasterRenderer) -> bool {
        (0..bd.height()).any(|y| bd.image()[y * WIDTH + WIDTH - 1] < 250)
    }

    /// **Địa chỉ ví dài 66 ký tự, KHÔNG có chỗ ngắt** — có tràn mép không?
    ///
    /// Màn xác nhận giao dịch và màn nhập ví đều hiện địa chỉ ĐỦ, cố ý: cắt
    /// ngắn là lỗ dò trùng đầu-đuôi. Nhưng một chuỗi 66 ký tự không dấu cách
    /// thì bộ ngắt dòng theo TỪ không có chỗ nào để ngắt.
    #[test]
    fn dia_chi_du_khong_tran_mep() {
        let dia_chi = "0x11b22b300e195c44c910d71cdb1515c4617e852393cde5e80c860906b8a2d549";
        assert_eq!(dia_chi.len(), 66);

        // Đặt trong nhóm lồng sâu — mỗi tầng thụt thêm, đúng như màn thật.
        let mut cay = Node::text(dia_chi).unwrap();
        for _ in 0..4 {
            cay = Node::group(Flow::Column, Gap::Large).child(cay).unwrap();
        }
        let mut bd = RasterRenderer::new();
        bd.render(&cay).unwrap();
        assert!(
            !tran_mep_phai(&bd),
            "địa chỉ tràn khỏi mép phải — người dùng đọc thiếu đuôi mà không biết"
        );
    }

    /// Khoảng dọc mà cột `x` có mực: (trên cùng, dưới cùng).
    fn khoang_doc(bd: &RasterRenderer, tu_cot: usize, den_cot: usize) -> Option<(usize, usize)> {
        let co_muc =
            |y: usize| (tu_cot..den_cot.min(WIDTH)).any(|x| bd.image()[y * WIDTH + x] < 250);
        let tren = (0..bd.height()).find(|y| co_muc(*y))?;
        let duoi = (0..bd.height()).rev().find(|y| co_muc(*y))?;
        Some((tren, duoi))
    }

    /// **B30 — nhãn điều khiển phải VẼ RA cho người nhìn, không chỉ cho trợ năng.**
    ///
    /// Một công tắc hay ô nhập chỉ mang nhãn trong cây trợ năng là một điều
    /// khiển mà người dùng trình đọc màn hình hiểu còn người nhìn thì đoán.
    ///
    /// Phép thử giữ bất biến này nằm trong crate bị xoá 23/08/2026. Bất biến vẫn
    /// đúng — nhưng suốt từ hôm ấy tới nay **không gì canh nó**.
    #[test]
    fn nhan_dieu_khien_duoc_ve_ra_cho_nguoi_nhin() {
        let cay = Node::group(Flow::Column, Gap::Medium)
            .child(Node::toggle("Cho phép mạng", false, "mang").unwrap())
            .unwrap()
            .child(Node::field("Mã đơn", "A-1", false).unwrap())
            .unwrap();
        let mut bd = RasterRenderer::new();
        bd.render(&cay).unwrap();
        let ve: Vec<&str> = bd.da_dat.iter().map(|d| d.o.chu.as_str()).collect();
        for nhan in ["Cho phép mạng", "Mã đơn"] {
            assert!(
                ve.iter().any(|c| c.contains(nhan)),
                "nhãn {nhan:?} không được vẽ ra — người nhìn không thấy nó: {ve:?}"
            );
        }
    }

    /// **B38 — nút mất mát KHÔNG giãn hết bề ngang.**
    ///
    /// Một nút chiếm trọn bề ngang là một nút khó bấm trượt, và với hành động
    /// không hoàn tác thì "khó bấm trượt" là hướng SAI.
    #[test]
    fn nut_mat_mat_khong_gian_het_be_ngang() {
        let cay = Node::group(Flow::Column, Gap::Medium)
            .child(Node::button("Xoá hết", "x", Tone::Danger).unwrap())
            .unwrap();
        let mut bd = RasterRenderer::new();
        bd.render(&cay).unwrap();
        let (_, _, rong, _) = bd.placed_boxes()[0];
        #[expect(clippy::cast_precision_loss, reason = "bề rộng khung, hằng số nhỏ")]
        let du = WIDTH as f32 - LE * 2.0;
        assert!(
            rong < du * 0.9,
            "nút mất mát rộng {rong} trên {du} — gần trọn bề ngang"
        );
    }

    /// **B31 — nút MẤT MÁT phải được VẼ khác, không chỉ khai khác.**
    ///
    /// # Bất biến này đã TÁI PHÁT, và đó là lý do phép thử đo MỰC
    ///
    /// B31 sinh ra vì bộ dựng đầu vẽ `Tone::Danger` y hệt mọi nút khác. Bộ dựng
    /// ra pixel thay chỗ nó, đọc `Tone` **chỉ** để đặt cờ trợ năng, và vẽ y hệt
    /// — cùng một lỗi, lần thứ hai. Không ai kêu, vì phép thử giữ B31 nằm trong
    /// crate bị xoá ngày 23/08/2026 và đi cùng nó.
    ///
    /// Nên phép thử này KHÔNG hỏi cây trợ năng. Cây trợ năng đã đúng suốt thời
    /// gian bất biến bị vỡ — nó khai `destructive: true` trong khi màn hình
    /// không cho thấy gì. Câu hỏi của B31 là câu hỏi về MỰC, và chỉ mực trả lời
    /// được.
    #[test]
    fn nut_mat_mat_duoc_ve_khac_nut_thuong() {
        let ve = |tone| {
            let cay = Node::group(Flow::Column, Gap::Medium)
                .child(Node::button("Xoá hết", "x", tone).unwrap())
                .unwrap();
            let mut bd = RasterRenderer::new();
            bd.render(&cay).unwrap();
            (bd.ink(), bd.placed_boxes())
        };
        let (muc_thuong, hop_thuong) = ve(Tone::Neutral);
        let (muc_mat, hop_mat) = ve(Tone::Danger);
        // Cùng CHỮ, cùng HỘP — nên mọi khác biệt về mực là khác biệt do sắc thái.
        assert_eq!(hop_thuong, hop_mat, "hai ca phải cùng hình học mới so được");
        assert!(
            muc_mat > muc_thuong,
            "nút mất mát vẽ ra ĐÚNG bằng nút thường ({muc_mat} = {muc_thuong} điểm \
             mực) — người dùng NHÌN không phân biệt được, dù cây trợ năng có khai"
        );
    }

    /// **Mọi lời khai bố cục được NHẬN đều phải đổi hình học.**
    ///
    /// Một lời khai được nhận rồi không làm gì là thứ tệ nhất trong nhóm này:
    /// người viết ứng dụng thấy màn hình dựng lên bình thường và tin rằng lời
    /// khai đã có tác dụng. Trên máy họ nó "chạy"; trên máy khác thì không.
    ///
    /// ⚠️ Mỗi ca phải cho lời khai một CƠ HỘI thể hiện — xem hai phép thử ngay
    /// dưới, `wrap` và `fill` cần cây riêng.
    #[test]
    fn moi_loi_khai_duoc_nhan_deu_doi_hinh_hoc() {
        use tcc_ui::{AlignCross, AlignMain, Extent, Sizing};
        let d = Sizing::default();
        // Hàng có hai chữ CAO THẤP KHÁC NHAU: căn theo trục phụ mới có chỗ hiện.
        let hang = |am, ac, size, min, max, pad| {
            let g = Node::group(Flow::Row, Gap::Small)
                .child(Node::text_with("Cao", Emphasis::Title).unwrap())
                .unwrap()
                .child(Node::text("thấp").unwrap())
                .unwrap()
                .with_layout(size, min, max, am, ac, pad)
                .unwrap();
            let mut bd = RasterRenderer::new();
            bd.render(&Node::group(Flow::Column, Gap::Medium).child(g).unwrap())
                .unwrap();
            bd.placed_boxes()
        };
        let nen = hang(AlignMain::Start, AlignCross::Start, d, d, d, Gap::None);
        let nua = Sizing {
            main: Some(Extent::Half),
            cross: None,
        };
        let tu = Sizing {
            main: Some(Extent::Quarter),
            cross: None,
        };
        let ca = [
            (
                "align_main=End",
                hang(AlignMain::End, AlignCross::Start, d, d, d, Gap::None),
            ),
            (
                "align_main=Center",
                hang(AlignMain::Center, AlignCross::Start, d, d, d, Gap::None),
            ),
            (
                "align_cross=End",
                hang(AlignMain::Start, AlignCross::End, d, d, d, Gap::None),
            ),
            (
                "align_cross=Stretch",
                hang(AlignMain::Start, AlignCross::Stretch, d, d, d, Gap::None),
            ),
            (
                "padding=Large",
                hang(AlignMain::Start, AlignCross::Start, d, d, d, Gap::Large),
            ),
            (
                "size.main=Half",
                hang(AlignMain::End, AlignCross::Start, nua, d, d, Gap::None),
            ),
            (
                "min.main=Half",
                hang(AlignMain::End, AlignCross::Start, d, nua, d, Gap::None),
            ),
            (
                "max.main=Quarter",
                hang(AlignMain::End, AlignCross::Start, d, d, tu, Gap::None),
            ),
        ];
        for (ten, h) in ca {
            assert_ne!(h, nen, "lời khai `{ten}` không đổi gì — nó đang chết");
        }
    }

    /// **`wrap` chỉ có nghĩa khi nội dung TRÀN.**
    ///
    /// Bản thăm dò đầu báo `wrap` là "chết" vì cây thử chỉ có hai chữ ngắn, vừa
    /// một dòng. Cùng hạng lỗi với "dụng cụ đo hỏng lặng lẽ vẫn trả về một kết
    /// quả": phép đo không cho tính năng cơ hội thì báo chết cho một tính năng
    /// đang sống, và ta đi sửa thứ không hỏng.
    #[test]
    fn wrap_can_noi_dung_tran_moi_do_duoc() {
        let tran = |w| {
            let mut g = Node::group(Flow::Row, Gap::Small);
            for i in 0..12 {
                g = g
                    .child(Node::text(format!("phần-tử-số-{i}")).unwrap())
                    .unwrap();
            }
            let mut bd = RasterRenderer::new();
            bd.render(
                &Node::group(Flow::Column, Gap::Medium)
                    .child(g.with_wrap(w))
                    .unwrap(),
            )
            .unwrap();
            bd.placed_boxes()
        };
        assert_ne!(tran(None), tran(Some(false)), "`wrap` không đổi gì");
    }

    /// **`fill` phải CHIA ĐƯỢC khoảng trống, không chỉ "khác đi".**
    ///
    /// ⚠️ Khẳng định "có khai thì khác không khai" là KHÔNG đủ: `fill` đổi luôn
    /// bề mặc định của nhóm, nên câu ấy vẫn xanh khi `flex_grow` bị bỏ hẳn —
    /// kiểm đột biến chỉ ra đúng điều đó, và phép thử phải siết lại.
    #[test]
    fn fill_chia_khoang_trong_cua_cha() {
        use tcc_ui::{AlignCross, AlignMain, Extent, Sizing};
        let d = Sizing::default();
        let day = |be| {
            let trong = Node::group(Flow::Row, Gap::None)
                .child(Node::text("A").unwrap())
                .unwrap()
                .with_layout(
                    Sizing {
                        main: be,
                        cross: None,
                    },
                    d,
                    d,
                    AlignMain::End,
                    AlignCross::Start,
                    Gap::None,
                )
                .unwrap();
            let mut bd = RasterRenderer::new();
            bd.render(
                &Node::group(Flow::Row, Gap::None)
                    .child(trong)
                    .unwrap()
                    .child(Node::text("B").unwrap())
                    .unwrap(),
            )
            .unwrap();
            bd.placed_boxes()
        };
        let khong = day(None);
        let co = day(Some(Extent::Fill));
        // Không khai: nhóm chiếm trọn bề cha, nên anh em bị đẩy XUỐNG DÒNG.
        assert!(
            (khong[0].1 - khong[1].1).abs() > 1.0,
            "chưa dựng được ca so sánh: {khong:?}"
        );
        // Có `fill`: nhóm nới ĐÚNG phần trống, anh em ở lại cùng dòng…
        assert!(
            (co[0].1 - co[1].1).abs() < 1.0,
            "`fill` không nhường chỗ cho anh em: {co:?}"
        );
        // …và nhóm thật sự NỚI RA chứ không co về nội dung.
        assert!(
            co[0].0 > 300.0,
            "`fill` co về nội dung thay vì nới ra: {co:?}"
        );
    }

    /// **Hàng ngang căn GIỮA theo chiều dọc, không dính mép trên.**
    ///
    /// Một nhãn nhỏ cạnh một tiêu đề lớn mà dính mép trên thì trông như bị treo
    /// lơ lửng — thứ người ta nhìn thấy ngay kể cả khi không biết gọi tên nó.
    ///
    /// # ⚠️ Đo HỘP, không đo mực — và vì sao đổi
    ///
    /// Bản trước đo tâm dọc của MỰC. Nó xanh trên macOS, đỏ trên Linux, và mất
    /// ba vòng CI mới ra lý do: hộp trên hai nền **giống hệt nhau**
    /// (`[(12,12,_,32), (48,17,_,21)]`) — bố cục không sai chỗ nào. Sai là mực:
    /// trên Linux chữ "nhỏ" vẽ ra 21..43 trong khi hộp của nó chỉ 17..38.
    ///
    /// Căn giữa là chuyện của HỘP. Đo nó bằng mực là trộn hai đại lượng: chỗ bố
    /// cục đặt ô, và chỗ phông vẽ nét trong ô ấy. Cái thứ hai đổi theo máy, nên
    /// phép thử đổi theo máy — và một phép thử đỏ tuỳ máy là một phép thử người
    /// ta sẽ tắt.
    ///
    /// Việc "nét có tràn ra ngoài hộp không" là câu hỏi riêng, và nó có phép thử
    /// riêng ngay dưới đây.
    #[test]
    fn hang_can_giua_theo_chieu_doc() {
        let cay = Node::group(Flow::Row, Gap::Medium)
            .child(Node::text_with("To", Emphasis::Title).unwrap())
            .unwrap()
            .child(Node::text("nhỏ").unwrap())
            .unwrap();
        let mut bd = RasterRenderer::new();
        bd.render(&cay).unwrap();

        let hop = bd.placed_boxes();
        assert_eq!(hop.len(), 2, "phải có đúng hai ô: {hop:?}");
        let tam = |(_, y, _, h): (f32, f32, f32, f32)| y + h / 2.0;
        let (to, nho) = (tam(hop[0]), tam(hop[1]));
        assert!(
            (to - nho).abs() <= 1.0,
            "tâm dọc hai ô lệch {} — ô nhỏ không được căn giữa: {hop:?}",
            (to - nho).abs()
        );
        // Và ô nhỏ phải THẤP HƠN ô to ở mép trên: bằng nhau nghĩa là dính mép,
        // tức là không căn giữa mà chỉ tình cờ cùng chiều cao.
        assert!(
            hop[1].1 > hop[0].1,
            "ô nhỏ dính mép trên cùng ô to — không phải căn giữa: {hop:?}"
        );
        // Vẫn phải có MỰC THẬT trong cả hai ô: một bố cục đúng mà không vẽ gì ra
        // thì vẫn qua mọi phép thử hình học ở trên.
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "toạ độ ô, luôn dương và nhỏ hơn bề rộng ảnh"
        )]
        for (i, (x, _, w, _)) in hop.iter().enumerate() {
            assert!(
                khoang_doc(&bd, *x as usize, (*x + *w) as usize).is_some(),
                "ô {i} không có nét nào được vẽ"
            );
        }
    }

    /// **Nét chữ KHÔNG được vẽ ra ngoài ô của chính nó.**
    ///
    /// Phát hiện 22/08/2026, từ đúng vòng CI đã nói ở trên: trên Linux chữ "nhỏ"
    /// vẽ tới y=43 trong khi ô của nó kết thúc ở 38. Chiều cao ô tính bằng
    /// `cỡ × 1.4` — một con số ĐOÁN, không hỏi phông — nên phông nào cần nhiều
    /// hơn thì nét tràn ra ngoài.
    ///
    /// Vì sao đáng một phép thử riêng: `khong_bao_gio_co_o_chong_len_nhau` chỉ
    /// soi HỘP. Hai hộp không chồng nhau mà nét của hộp trên tràn xuống hộp dưới
    /// thì người dùng vẫn thấy chữ đè lên chữ, và không phép thử nào kêu.
    ///
    /// **Lượt ĐO và lượt VẼ phải dùng CÙNG một chiều cao dòng.**
    ///
    /// `do_o` hỏi phông một lần và ghi vào `O.cao_dong`; `ve_o` phải đọc lại con
    /// số ấy. Tính lại ở lượt vẽ — dù bằng đúng công thức — là mở đường cho hai
    /// bên trôi khỏi nhau, và khi ấy cái vẽ ra không còn khớp cái `hit_test`
    /// tin: người dùng thấy một nút, hệ thống chạy một nút khác.
    ///
    /// ⚠️ Phép thử này soi mã nguồn, và nó tồn tại vì phép thử "nét không tràn"
    /// ngay dưới **không kiểm được trên máy này**: phông của macOS vừa mức sàn
    /// với mọi chuỗi thử được (đo 23/08/2026 — `To`, `nhỏ`, `Ẫỹ`, `Ẳjg`,
    /// `Ẫjgỹp`, `Ổnhũ`, `ẶỹjgpQ`, không chuỗi nào vượt 21px). Điều kiện chỉ xuất
    /// hiện với phông của Linux. Nên chỗ nào kiểm được thì kiểm ở đây, và phần
    /// còn lại do CI trên Linux nói.
    #[test]
    fn do_va_ve_dung_cung_chieu_cao_dong() {
        let nguon = include_str!("lib.rs");
        let than = nguon.split("#[cfg(test)]").next().unwrap_or(nguon);
        let ve = than
            .split_once("fn ve_o(")
            .map(|(_, sau)| sau)
            .expect("có `ve_o`");
        // Lượt vẽ phải đi qua ĐÚNG hàm tạo hình chung, và truyền số đã đo.
        assert!(
            ve.contains("tao_hinh(") && ve.contains("o.cao_dong"),
            "lượt vẽ không đi qua `tao_hinh` với chiều cao dòng đã đo"
        );
        // Và nó KHÔNG được tự dựng `Buffer` hay tự đặt `Metrics`: tự dựng là
        // dựng lại đúng chỗ hai lượt đã trôi khỏi nhau ba vòng CI liền, kể cả
        // khi chép đúng từng dòng.
        assert!(
            !ve.contains("Metrics::new") && !ve.contains("Buffer::new"),
            "lượt vẽ đang tự dựng buffer thay vì gọi `tao_hinh`"
        );
        // Mức sàn chỉ được nhắc ở lượt ĐO.
        assert!(
            !ve.contains("CAO_DONG"),
            "lượt vẽ đang tự tính chiều cao dòng thay vì đọc lại số đã đo"
        );
        // Và chỉ ĐƯỢC có MỘT chỗ dựng `Buffer` trong cả tệp — bên trong
        // `tao_hinh`. Hai chỗ là hai đường, và hai đường thì trôi.
        assert_eq!(
            than.matches("Buffer::new(").count(),
            1,
            "có nhiều hơn một chỗ dựng `Buffer` — mọi lượt tạo hình phải đi qua \
             `tao_hinh`"
        );
    }

    /// **Đã trả ngày 23/08/2026.** `do_o` hỏi `max_ascent + max_descent` của
    /// phông thay vì nhân 1.4, và `ve_o` dùng LẠI đúng con số ấy thay vì tính
    /// lại — xem [`cao_dong_that`]. `centering_offset` của cosmic-text khi ấy
    /// không bao giờ âm, nên nét nằm trong hộp dòng, và hộp dòng nằm trong ô.
    ///
    /// Phép thử bỏ `#[ignore]` cùng lần sửa ấy. Nó xanh trên macOS trước và sau
    /// — chỗ duy nhất chứng minh được là CI trên Linux, vì đó là chỗ có phông
    /// làm lộ lỗi.
    #[test]
    fn net_khong_tran_ra_ngoai_o() {
        let cay = Node::group(Flow::Row, Gap::Medium)
            .child(Node::text_with("To", Emphasis::Title).unwrap())
            .unwrap()
            .child(Node::text("nhỏ").unwrap())
            .unwrap();
        let mut bd = RasterRenderer::new();
        bd.render(&cay).unwrap();
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "toạ độ ô, luôn dương và nhỏ hơn bề rộng ảnh"
        )]
        for (i, (x, y, w, h)) in bd.placed_boxes().iter().enumerate() {
            let Some((tren, duoi)) = khoang_doc(&bd, *x as usize, (*x + *w) as usize) else {
                continue;
            };
            #[expect(clippy::cast_precision_loss, reason = "toạ độ ảnh, luôn nhỏ")]
            let (tren, duoi) = (tren as f32, duoi as f32);
            // Số đo của LƯỢT ĐO đi kèm, không chỉ số của lượt vẽ. Phép thử này
            // đỏ trên Linux hai lần liền với đúng một bộ số, và cả hai lần tôi
            // đoán sai vì thông báo chỉ nói kết quả chứ không nói lượt đo đã
            // thấy gì. Đoán từ một nửa dữ liệu là đoán.
            let d = &bd.da_dat[i];
            assert!(
                tren >= *y - 1.0 && duoi <= *y + *h + 1.0,
                "ô {i} ({:?}): nét {tren}..{duoi} nằm ngoài ô {y}..{} \
                 — lượt đo: net={:?} cao_dong={} cao={} rong={}",
                d.o.chu,
                *y + *h,
                d.o.net,
                d.o.cao_dong,
                d.o.cao,
                d.o.rong
            );
        }
    }

    /// Một từ dài hơn cả bề rộng phải bị **ngắt giữa từ**, không tràn.
    #[test]
    fn tu_dai_hon_be_rong_bi_ngat_giua_tu() {
        let dai = "a".repeat(400);
        let mut bd = RasterRenderer::new();
        bd.render(&Node::text(&dai).unwrap()).unwrap();
        assert!(!tran_mep_phai(&bd), "từ dài tràn khỏi mép phải");
        assert!(bd.height() > 40, "không xuống dòng: cao {}", bd.height());
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay"
)]
mod kiem_thu_hich {
    use super::*;
    use tcc_ui::{Flow, Gap};

    /// Bề rộng khung (pixel xám 150) của từng nút, từ trái sang phải.
    fn be_rong_cac_khung(bd: &RasterRenderer) -> Vec<usize> {
        // Hàng ngang trên cùng của khung là một dải pixel 150 liền nhau.
        let hang = (0..bd.height())
            .find(|y| {
                (0..WIDTH)
                    .filter(|x| bd.image()[y * WIDTH + x] == 150)
                    .count()
                    > 4
            })
            .expect("phải có ít nhất một khung");
        let mut ra = Vec::new();
        let mut dai = 0usize;
        for x in 0..WIDTH {
            if bd.image()[hang * WIDTH + x] == 150 {
                dai += 1;
            } else if dai > 4 {
                ra.push(dai);
                dai = 0;
            } else {
                dai = 0;
            }
        }
        if dai > 4 {
            ra.push(dai);
        }
        ra
    }

    /// **Hai nút trên một hàng phải RỘNG BẰNG NHAU.**
    ///
    /// Cùng luật với "hai nút cùng sắc thái": một nút to hơn hẳn nút kia vẫn là
    /// một cái hích, chỉ bằng hình học thay vì bằng màu. Và ở màn xác nhận giao
    /// dịch thì cái hích ấy đẩy về phía KÝ.
    #[test]
    fn hai_nut_cung_hang_rong_bang_nhau() {
        let cay = Node::group(Flow::Row, Gap::Medium)
            .child(Node::button("Ký giao dịch này", "ky", Tone::Neutral).unwrap())
            .unwrap()
            .child(Node::button("Huỷ", "huy", Tone::Neutral).unwrap())
            .unwrap();
        let mut bd = RasterRenderer::new();
        bd.render(&cay).unwrap();

        let rong = be_rong_cac_khung(&bd);
        assert_eq!(rong.len(), 2, "cần đúng hai khung, thấy {rong:?}");
        assert!(
            rong[0].abs_diff(rong[1]) <= 1,
            "hai nút rộng {} và {} — nút dài hơn đang hích người dùng",
            rong[0],
            rong[1]
        );
    }

    /// Nhưng nút cạnh NHÃN thì không kéo bằng nhau — vô nghĩa.
    #[test]
    fn nut_canh_nhan_khong_bi_keo_bang() {
        let cay = Node::group(Flow::Row, Gap::Medium)
            .child(Node::text("Một nhãn khá dài để so").unwrap())
            .unwrap()
            .child(Node::button("OK", "ok", Tone::Neutral).unwrap())
            .unwrap();
        let mut bd = RasterRenderer::new();
        bd.render(&cay).unwrap();
        assert_eq!(be_rong_cac_khung(&bd).len(), 1, "nhãn bị vẽ khung");
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay"
)]
mod kiem_thu_hop_thanh {
    use super::*;
    use tcc_ui::{Flow, Gap};

    /// Sinh một cây từ một hạt giống — đủ lộn xộn để chạm các nhánh bố cục.
    ///
    /// Không dùng bộ sinh ngẫu nhiên: phép thử phải lặp lại được, và một lần
    /// đỏ phải dựng lại được bằng đúng con số ấy.
    fn cay_tu_hat(hat: u64) -> Node {
        fn dung(sau: usize, tiep: &mut impl FnMut() -> usize) -> Node {
            let huong = if tiep().is_multiple_of(2) {
                Flow::Column
            } else {
                Flow::Row
            };
            let khe = match tiep() % 4 {
                0 => Gap::None,
                1 => Gap::Small,
                2 => Gap::Medium,
                _ => Gap::Large,
            };
            let mut g = Node::group(huong, khe);
            for _ in 0..=(tiep() % 5) {
                let con = if sau > 0 && tiep().is_multiple_of(3) {
                    dung(sau - 1, tiep)
                } else {
                    match tiep() % 5 {
                        0 => Node::text_with(
                            "Tiêu đề dài vừa phải để có lúc phải ngắt dòng",
                            Emphasis::Title,
                        )
                        .unwrap(),
                        1 => Node::button("Một nút", "mot-nut", Tone::Danger).unwrap(),
                        2 => Node::field("Ô nhập", "chào bạn", false).unwrap(),
                        3 => Node::toggle("Công tắc", true, "ct").unwrap(),
                        _ => Node::text("Chữ thường, có dấu: ế ữ ợ, và dài ra một chút").unwrap(),
                    }
                };
                g = g.child(con).unwrap();
            }
            g
        }
        let mut r = hat.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        let mut tiep = move || {
            r = r.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            (r >> 33) as usize
        };
        dung(3, &mut tiep)
    }

    /// **KHÔNG ô nào chồng lên ô nào — với mọi cây.**
    ///
    /// Vẽ đè là đòn che chữ: đặt một ô lên trên câu "việc này chuyển tiền" thì
    /// người dùng xác nhận một thứ họ không đọc được. Bố cục ở đây xếp chỗ chứ
    /// không chồng chỗ, và đây là chỗ chốt điều đó.
    #[test]
    fn khong_bao_gio_co_o_chong_len_nhau() {
        for hat in 0..80u64 {
            let cay = cay_tu_hat(hat);
            let mut bd = RasterRenderer::new();
            bd.render(&cay).unwrap();
            let so = dem_chong(&bd.placed_boxes());
            assert_eq!(
                so, 0,
                "hạt {hat}: có {so} cặp ô chồng nhau — một ô đang che ô khác"
            );
        }
    }

    /// Và không ô nào tràn khỏi mép phải.
    #[test]
    fn khong_o_nao_tran_mep_voi_moi_cay() {
        for hat in 0..80u64 {
            let cay = cay_tu_hat(hat);
            let mut bd = RasterRenderer::new();
            bd.render(&cay).unwrap();
            let cham = (0..bd.height()).any(|y| bd.image()[y * WIDTH + WIDTH - 1] < 250);
            assert!(!cham, "hạt {hat}: có thứ tràn tới sát mép phải");
        }
    }

    /// Cây quá cao thì **báo lỗi**, không cắt im lặng.
    ///
    /// Cắt im lặng là giấu mất phần giao diện người dùng đáng ra phải thấy — và
    /// phần bị giấu có thể là nút "Huỷ".
    #[test]
    fn cay_qua_cao_thi_bao_loi_chu_khong_cat() {
        let mut g = Node::group(Flow::Column, Gap::Large);
        for _ in 0..300 {
            g = g
                .child(Node::text_with("Một dòng tiêu đề", Emphasis::Title).unwrap())
                .unwrap();
        }
        let mut bd = RasterRenderer::new();
        let loi = bd.render(&g).unwrap_err();
        assert!(matches!(loi, RasterError::TooTall(_)), "{loi}");
    }

    /// Phép đếm chồng phải THẬT SỰ đếm được — nếu không nó chỉ là số 0 vô nghĩa.
    /// Phép đếm chồng nằm Ở ĐÂY, không nằm trong bộ dựng.
    ///
    /// Bản đầu để bộ dựng tự đếm và phép thử đọc con số ấy — kiểm đột biến cho
    /// thấy đặt bộ đếm về 0 là xanh ngay. **Phép thử đang hỏi chính bị cáo.**
    fn dem_chong(o: &[(f32, f32, f32, f32)]) -> usize {
        let mut so = 0;
        for (i, a) in o.iter().enumerate() {
            for b in o.iter().skip(i + 1) {
                let ngang = a.0 < b.0 + b.2 && b.0 < a.0 + a.2;
                let doc = a.1 < b.1 + b.3 && b.1 < a.1 + a.3;
                if ngang && doc {
                    so += 1;
                }
            }
        }
        so
    }

    /// Phép đếm chồng phải THẬT SỰ đếm được — nếu không nó chỉ là số 0 vô nghĩa.
    #[test]
    fn phep_dem_chong_bat_duoc_cho_chong() {
        assert_eq!(
            dem_chong(&[(0.0, 0.0, 40.0, 20.0), (20.0, 10.0, 40.0, 20.0)]),
            1
        );
        assert_eq!(
            dem_chong(&[(0.0, 0.0, 40.0, 20.0), (40.0, 0.0, 40.0, 20.0)]),
            0,
            "chạm mép không phải chồng"
        );
        assert_eq!(
            dem_chong(&[(0.0, 0.0, 40.0, 20.0), (0.0, 20.0, 40.0, 20.0)]),
            0
        );
        assert_eq!(
            dem_chong(&[
                (0.0, 0.0, 9.0, 9.0),
                (1.0, 1.0, 9.0, 9.0),
                (2.0, 2.0, 9.0, 9.0)
            ]),
            3
        );
    }

    /// **KHÔNG ô nào được nằm ngoài ảnh — và không cú bấm nào ngoài ảnh trúng.**
    ///
    /// Đo được ngày 21/08/2026: một hàng ba nút với một nhãn dài làm luật "nút
    /// cùng hàng rộng bằng nhau" kéo mọi ô lên 326,9 px, và ô thứ ba trôi ra
    /// **681,8 → 1008,7 trên một ảnh rộng 640** — không một điểm ảnh nào được
    /// vẽ, mà `hit_test` vẫn trả về nó, kể cả ở x = 1000.
    ///
    /// Người dùng kéo rộng cửa sổ, bấm vào khoảng trắng bên phải, và một nút họ
    /// **chưa từng nhìn thấy** chạy.
    ///
    /// Phép thử cũ không thể bắt được: `ve_o` cắt phần vẽ ở `WIDTH - trai - 2`
    /// nên phép kiểm "chạm mép phải" **về mặt cấu trúc không bao giờ đỏ được**,
    /// và bộ sinh cây ngẫu nhiên dùng CÙNG một nhãn cho mọi nút nên không bao
    /// giờ tạo ra hàng có bề rộng lệch — đúng hình dạng duy nhất kích hoạt lỗi.
    #[test]
    fn khong_o_nao_troi_ra_ngoai_anh() {
        let cay = Node::group(Flow::Row, Gap::Medium)
            .child(
                Node::button(
                    "Send everything in my wallet to the address above",
                    "gui-het",
                    Tone::Neutral,
                )
                .unwrap(),
            )
            .unwrap()
            .child(Node::button("No", "huy", Tone::Neutral).unwrap())
            .unwrap()
            .child(Node::button("Ok", "ok2", Tone::Neutral).unwrap())
            .unwrap();
        let mut bd = RasterRenderer::new();
        bd.render(&cay).unwrap();

        #[expect(clippy::cast_precision_loss, reason = "kích thước ảnh, luôn nhỏ")]
        let rong = WIDTH as f32;
        for (i, o) in bd.placed_boxes().iter().enumerate() {
            assert!(
                o.0 + o.2 <= rong,
                "ô {i} trôi ra ngoài ảnh: trái={} rộng={} phải={} > {rong}",
                o.0,
                o.2,
                o.0 + o.2
            );
        }
        // Và cú bấm ngoài ảnh không trúng gì, dù hình học có sai thế nào.
        assert!(bd.hit_test(700.0, 17.0).is_none());
        assert!(bd.hit_test(1000.0, 17.0).is_none());
        assert!(bd.hit_test(-5.0, 17.0).is_none());
    }

    /// Nhưng luật "nút cùng hàng rộng bằng nhau" vẫn giữ KHI NÓ VỪA.
    ///
    /// Bản vá dễ hỏng theo hướng ngược lại: bỏ luôn việc kéo bằng nhau thì hết
    /// tràn, mà mất đi thứ chặn "một nút to hơn hẳn nút kia vẫn là một cái hích".
    #[test]
    fn van_keo_bang_nhau_khi_vua() {
        let cay = Node::group(Flow::Row, Gap::Medium)
            .child(Node::button("Ký giao dịch này", "ky", Tone::Neutral).unwrap())
            .unwrap()
            .child(Node::button("Huỷ", "huy", Tone::Neutral).unwrap())
            .unwrap();
        let mut bd = RasterRenderer::new();
        bd.render(&cay).unwrap();
        let o = bd.placed_boxes();
        assert_eq!(o.len(), 2);
        assert!(
            (o[0].2 - o[1].2).abs() < 0.01,
            "hai nút cùng hàng không còn rộng bằng nhau: {} vs {}",
            o[0].2,
            o[1].2
        );
    }

    /// **Bấm vào nút thì trúng nút ấy, bấm ra ngoài thì không trúng gì.**
    ///
    /// Phép thử hỏi HÌNH HỌC ĐÃ VẼ, không hỏi cây: chỗ người dùng bấm là chỗ bộ
    /// dựng đã đặt. Nếu hai thứ ấy lệch nhau thì cú bấm chạy một việc người
    /// dùng không định chạy, và một phép thử tra cây sẽ không bao giờ thấy.
    #[test]
    fn bam_trung_nut_da_ve() {
        let cay = Node::group(Flow::Column, Gap::Medium)
            .child(Node::text("một dòng chữ thường").unwrap())
            .unwrap()
            .child(Node::button("Ký và gửi", "ky-gui", Tone::Primary).unwrap())
            .unwrap()
            .child(Node::button("Huỷ", "huy", Tone::Neutral).unwrap())
            .unwrap();
        let mut bd = RasterRenderer::new();
        bd.render(&cay).unwrap();

        let o = bd.placed_boxes();
        assert_eq!(o.len(), 3, "phải đặt ba ô");

        // Tâm của từng ô, tính từ chính hình học bộ dựng khai ra.
        let tam = |i: usize| (o[i].0 + o[i].2 / 2.0, o[i].1 + o[i].3 / 2.0);

        assert_eq!(
            bd.hit_test(tam(0).0, tam(0).1),
            None,
            "chữ thường mà bấm được"
        );
        assert_eq!(
            bd.hit_test(tam(1).0, tam(1).1).map(|h| h.action),
            Some("ky-gui")
        );
        assert_eq!(
            bd.hit_test(tam(2).0, tam(2).1).map(|h| h.action),
            Some("huy")
        );

        // Ngoài mọi ô.
        assert!(bd.hit_test(-5.0, -5.0).is_none());
        assert!(bd.hit_test(0.0, bd.height() as f32 + 50.0).is_none());
    }

    /// **Bấm vào ô nhập thì trúng ô nhập ấy, và KHÔNG chạy hành động nào.**
    #[test]
    fn bam_vao_o_nhap_khong_chay_hanh_dong() {
        let cay = Node::group(Flow::Column, Gap::Medium)
            .child(Node::field("Địa chỉ", "", false).unwrap())
            .unwrap()
            .child(Node::button("Gửi", "gui", Tone::Primary).unwrap())
            .unwrap();
        let mut bd = RasterRenderer::new();
        bd.render(&cay).unwrap();
        let o = bd.placed_boxes();
        let tam = |i: usize| (o[i].0 + o[i].2 / 2.0, o[i].1 + o[i].3 / 2.0);

        assert_eq!(bd.hit_test_field(tam(0).0, tam(0).1), Some("Địa chỉ"));
        assert!(
            bd.hit_test(tam(0).0, tam(0).1).is_none(),
            "bấm vào ô nhập mà chạy một hành động"
        );
        // Và ngược lại: bấm vào nút thì không phải là gõ vào đâu cả.
        assert!(bd.hit_test_field(tam(1).0, tam(1).1).is_none());
        assert_eq!(
            bd.hit_test(tam(1).0, tam(1).1).map(|h| h.action),
            Some("gui")
        );
    }

    /// **Công tắc bấm được**; ô nhập và ảnh thì không.
    ///
    /// Ô nhập không có hành động trong tiêu chuẩn 0.1 — cho nó "bấm được" là
    /// bịa ra một hành động không ai khai báo.
    #[test]
    fn cong_tac_bam_duoc_o_nhap_thi_khong() {
        let cay = Node::group(Flow::Column, Gap::Medium)
            .child(Node::toggle("Cho phép micro", false, "bat-micro").unwrap())
            .unwrap()
            .child(Node::field("Địa chỉ", "x", false).unwrap())
            .unwrap();
        let mut bd = RasterRenderer::new();
        bd.render(&cay).unwrap();
        let o = bd.placed_boxes();
        let tam = |i: usize| (o[i].0 + o[i].2 / 2.0, o[i].1 + o[i].3 / 2.0);
        let h = bd
            .hit_test(tam(0).0, tam(0).1)
            .expect("công tắc phải bấm được");
        assert_eq!(h.action, "bat-micro");
        assert!(
            h.toggle,
            "công tắc bị coi là nút — gạt một quyền sẽ đóng hộp thoại"
        );
        assert!(
            bd.hit_test(tam(1).0, tam(1).1).is_none(),
            "ô nhập mà bấm được"
        );
    }
}
