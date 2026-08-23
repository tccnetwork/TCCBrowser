//! API component — TRỪU TƯỢNG, không biết mình được dựng bằng gì.
//!
//! VIỆC CỦA CRATE NÀY: định nghĩa cây component mà ứng dụng TCC nhắm tới.
//!
//! ⚠️ LUẬT QUAN TRỌNG NHẤT CỦA CẢ DỰ ÁN:
//!
//! **Crate này KHÔNG được phụ thuộc bộ dựng nào, và KHÔNG được lộ ra tài liệu
//! trang web, thẻ đánh dấu hay bảng kiểu.**
//!
//! Vì sao: giai đoạn đầu ta mượn một máy dựng web làm bộ dựng. Luật này là thứ
//! giữ cho việc bỏ nó khả thi — và ngày 23/08/2026 nó đã được bỏ thật, không
//! một ứng dụng nào phải sửa một dòng. Đó là bằng chứng luật này đáng giá, chứ
//! không phải lý do nới nó: bộ dựng hôm nay cũng chỉ là bộ dựng hôm nay.
//!
//! Ứng dụng chỉ biết tới tầng này. Đổi bộ dựng thì ứng dụng không sửa một dòng.
//!
//! Có CI kiểm luật này, xem `tools/kiem-luat-phu-thuoc.sh` (luật 1 và luật 6).
//!
//! # Bốn tính chất được cưỡng chế bằng KIỂU DỮ LIỆU, không bằng lời nhắc
//!
//! 1. **Trợ năng không có đường bỏ qua.** Không dựng được một nút mà thiếu vai
//!    trò và nhãn. Ảnh trang trí phải nói ra miệng bằng [`Alt::Decorative`] —
//!    không có mặc định nào lặng lẽ dẫn tới đó.
//! 2. **Không có pixel, không có màu.** Ứng dụng khai Ý ĐỊNH ([`Tone::Danger`],
//!    [`Gap::Large`]); bộ dựng quyết định hình thức. Cho ứng dụng đặt màu là mở
//!    đường cho nút "Xoá" sơn xanh lá giống hệt nút "Lưu".
//! 3. **Chuỗi hiện ra người dùng đi qua đúng phép kiểm của bản kê khai.** Dùng
//!    lại [`tcc_spec::check_display_text`], không chép lại.
//! 4. **Cây có trần.** Sâu quá hoặc nhiều nút quá thì bị chặn NGAY LÚC DỰNG, vì
//!    một ứng dụng thù địch chỉ cần một vòng lặp là làm treo bộ dựng.
//!
//! # Ví dụ
//!
//! ```
//! use tcc_ui::{Flow, Gap, Node, Tone};
//!
//! let man_hinh = Node::group(Flow::Column, Gap::Medium)
//!     .child(Node::text("Bạn có 3 TCC")?)?
//!     .child(Node::button("Gửi", "gui-tien", Tone::Primary)?)?;
//!
//! // Cây trợ năng luôn dựng được, không cần ứng dụng làm gì thêm.
//! assert_eq!(man_hinh.accessibility_tree().children.len(), 2);
//! # Ok::<(), tcc_ui::UiError>(())
//! ```

pub mod wire;

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use tcc_spec::{SpecError, TextKind, check_display_text, tree::TreeError, tree::check_path_public};

// `ActionId` sống ở `tcc-spec`, không ở đây: mã hành động xuất hiện ở CẢ cây
// giao diện lẫn bản kê khai (nơi khai hành vi của từng nút), nên nó thuộc về
// tiêu chuẩn chứ không thuộc về tầng giao diện. Định nghĩa hai nơi là để hai
// nơi trôi dạt khỏi nhau.
pub use tcc_spec::ActionId;
// Bộ dựng cần đúng phép kiểm đường dẫn của tiêu chuẩn khi phục vụ tệp trong gói.
pub use tcc_spec::tree as tcc_spec_tree;
use thiserror::Error;

/// Số tầng lồng nhau tối đa.
///
/// 64 tầng đủ cho mọi giao diện thật (giao diện người ta viết tay hiếm khi quá
/// 15 tầng). Trần này để chặn cây do máy sinh ra nhằm làm tràn ngăn xếp khi bộ
/// dựng duyệt cây bằng đệ quy.
///
/// # Vì sao 32 chứ không phải 64
///
/// Trần cũ là 64, và nó **không bao giờ chạm tới được**. Mỗi tầng của cây tốn
/// HAI tầng lồng của JSON (một đối tượng + một mảng), mà `serde_json` mặc định
/// dừng ở 128. Nên cây 64 tầng bị bộ đọc JSON từ chối bằng `bad-json` trước khi
/// phép kiểm này kịp chạy — tức là mã `too-deep` là mã chết, và một cây HỢP LỆ
/// ở đúng trần lại bị từ chối.
///
/// Tệ hơn ở tầng tiêu chuẩn: bản cài đặt nào dùng bộ đọc JSON cho lồng sâu hơn
/// sẽ trả `too-deep`, bản dùng bộ đọc nông hơn trả `bad-json`. Cùng một gói,
/// hai mã lỗi. Trần của TIÊU CHUẨN không được phụ thuộc vào giới hạn đệ quy của
/// thư viện JSON mà bên cài đặt tình cờ chọn.
///
/// 32 tầng tốn 64 tầng JSON — nằm thoải mái dưới mọi giới hạn mặc định thường
/// gặp, nên mọi bản cài đặt đều chạm trần của TIÊU CHUẨN trước.
pub const MAX_DEPTH: usize = 32;

/// Tổng số nút tối đa trong một cây.
pub const MAX_NODES: usize = 10_000;

/// Độ dài tối đa của một chuỗi hiện ra người dùng, tính bằng ký tự.
pub const MAX_TEXT_LEN: usize = 4_096;

#[derive(Debug, Error)]
pub enum UiError {
    #[error("chuỗi giao diện không an toàn: {0}")]
    UnsafeText(#[from] SpecError),

    #[error("đường dẫn ảnh không hợp lệ: {0}")]
    BadImagePath(#[from] TreeError),

    #[error(
        "ảnh \"{0}\" trỏ ra ngoài gói — giao diện chỉ được dùng ảnh trong gói ĐÃ KÝ, \
         vì ảnh tải từ mạng là một cái đèn báo hiệu lộ ra người dùng đang xem gì"
    )]
    ExternalImage(String),

    #[error(
        "mã hành động \"{0}\" không hợp lệ — chỉ chữ thường ASCII, chữ số, dấu gạch ngang và chấm"
    )]
    BadActionId(String),

    #[error("chuỗi dài {0} ký tự, vượt trần {MAX_TEXT_LEN}")]
    TextTooLong(usize),

    #[error("cây giao diện sâu {0} tầng, vượt trần {MAX_DEPTH}")]
    TooDeep(usize),

    #[error("cây giao diện có {0} nút, vượt trần {MAX_NODES}")]
    TooManyNodes(usize),

    #[error("{0} là nút lá — chỉ nhóm mới chứa được nút con")]
    NotAContainer(&'static str),

    /// Một bề khai ở chỗ nó không có nghĩa gì.
    ///
    /// `fill` trong `min`/`max`: một mức TỐI THIỂU là "một phần của khoảng
    /// trống" thì không phải mức tối thiểu của cái gì cả. `none` trong `size`:
    /// một nút không có bề thì không vẽ được.
    ///
    /// Từ chối chứ không lặng lẽ bỏ qua: một lời khai bị bỏ qua trông y hệt một
    /// lời khai có tác dụng, và người viết ứng dụng không có cách nào biết.
    #[error("bề `{1}` không dùng được ở `{0}`")]
    BadExtent(&'static str, &'static str),

    /// Phân số đặt trên trục DỌC, nơi nó không giải ra được.
    ///
    /// Phân số tính theo bề TRONG của cha. Khung của bộ dựng **cuộn được**, nên
    /// bề dọc của nó suy từ nội dung — và suy từ nội dung thì không có con số
    /// nào để lấy một nửa. Mọi nhóm bên dưới thừa hưởng đúng tính chất ấy.
    ///
    /// Đo được ngày 23/08/2026: trên trục NGANG `half` cho ra đúng một nửa
    /// (con ở x=312 trên khung 640, so với 620 khi không khai), còn trên trục
    /// DỌC lời khai **không có tác dụng gì** và nhóm vẫn cao bằng nội dung.
    ///
    /// Từ chối chứ không lặng lẽ bỏ qua, vì cùng một lý do với vùng cuộn: một
    /// lời khai bị bỏ qua trông y hệt một lời khai có tác dụng.
    #[error(
        "`{0}` là phân số trên trục DỌC — bề dọc suy từ nội dung nên không có gì \
         để lấy một phần của"
    )]
    VerticalFraction(&'static str),

    /// `fill` đặt trong một cha không có khoảng trống nào để chia.
    ///
    /// `fill` là "một phần khoảng TRỐNG trên trục chính của CHA". Cha kiểu cột
    /// có trục chính là DỌC, mà bề dọc suy từ nội dung — không có con số nào để
    /// lấy phần trống của.
    ///
    /// Bản đầu không chặn, và tệ hơn một lời khai chết: `fill` khi ấy làm nhóm
    /// **co lại theo nội dung** (đo 23/08/2026 — con nhảy từ x=618 về x=12), tức
    /// là làm một việc người viết không hề xin.
    #[error(
        "`fill` cần khoảng trống trên trục chính của cha, mà cha kiểu cột có \
         trục chính là dọc và bề dọc suy từ nội dung"
    )]
    FillWithoutRoom,

    /// Vùng cuộn khai ra được nhưng chưa bộ dựng nào cắt được nội dung.
    ///
    /// Từ chối chứ không lặng lẽ vẽ đủ: một vùng cuộn không cuộn là một lời hứa
    /// chỉ vỡ trên màn hình nhỏ hơn màn hình của người viết ứng dụng.
    #[error(
        "vùng cuộn chưa dùng được — bộ dựng chưa cắt được nội dung theo nhóm, \
         và một vùng cuộn không cuộn thì tràn ra ngoài trên màn hình nhỏ"
    )]
    ScrollNotSupported,

    /// Ô nhập che chữ là **thứ của khung trình duyệt**, không phải của ứng dụng.
    ///
    /// Ô che chữ chính là hình dạng người dùng được dạy để tin: thấy chấm tròn
    /// là "chỗ này an toàn, gõ bí mật vào đi". Cho ứng dụng dựng ra nó là cho
    /// mọi ứng dụng dựng một lời mời gõ mã PIN ví, trông y hệt lời mời thật.
    ///
    /// Ô nhập THƯỜNG thì ứng dụng vẫn dùng được — ô tìm kiếm là việc chính
    /// đáng. Chỉ `secret: true` bị chặn.
    #[error(
        "ô nhập che chữ chỉ khung trình duyệt mới dựng được — ứng dụng dựng ô \
         che chữ là dựng một lời mời gõ mã PIN trông y hệt lời mời thật"
    )]
    SecretFieldFromApp,
}

impl UiError {
    /// Mã lỗi ỔN ĐỊNH, thuộc về TIÊU CHUẨN.
    ///
    /// ⚠️ Nhánh bọc lỗi trả mã của NGUYÊN NHÂN GỐC, không phải mã lớp vỏ. Một
    /// bản kê khai hỏng vì ký tự giả mạo phải ra `unsafe-display-string`, chứ
    /// `spec` thì không nói lên điều gì và bộ kiểm định không so khớp được.
    #[must_use]
    pub const fn ma(&self) -> &'static str {
        match self {
            Self::UnsafeText(e) => e.ma(),
            Self::BadImagePath(e) => e.ma(),
            Self::ExternalImage { .. } => "external-image",
            Self::BadActionId { .. } => "bad-action-id",
            Self::TextTooLong { .. } => "text-too-long",
            Self::TooDeep { .. } => "too-deep",
            Self::TooManyNodes { .. } => "too-many-nodes",
            Self::NotAContainer { .. } => "not-a-container",
            // `bad-layout` — mã của 0.2, xem `spec/0.2/06-error-codes.md`. Nó
            // xuất hiện ở đây trước khi 0.2 phát hành vì lời khai bố cục đã
            // dựng được: mã lỗi phải có TRƯỚC lời khai đầu tiên bị từ chối, chứ
            // không phải sau.
            Self::BadExtent { .. } | Self::VerticalFraction { .. } | Self::FillWithoutRoom => {
                "bad-layout"
            }
            // `bad-scroll` — mã của 0.2, xem `spec/0.2/06-error-codes.md`.
            Self::ScrollNotSupported => "bad-scroll",
            Self::SecretFieldFromApp => "secret-field-from-app",
        }
    }
}

// ───────────────────────────── Ý định, không phải hình thức ─────────────────

/// Mức nhấn của chữ. KHÔNG phải cỡ chữ — bộ dựng quyết định cỡ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Emphasis {
    /// Tiêu đề của cả màn hình. Chỉ nên có một.
    Title,
    /// Chữ thường. Mặc định trên dây.
    #[default]
    Normal,
    /// Chữ phụ, chú thích.
    Subtle,
    /// **Cảnh báo.** Dòng chữ phải nổi rõ hơn mọi dòng khác quanh nó.
    ///
    /// # Vì sao phải thêm một giá trị vào TIÊU CHUẨN
    ///
    /// `04-quyen-nang.md` bắt buộc quyền ví ký được **PHẢI hiện khác hẳn** mọi
    /// quyền khác. Nhưng từ vựng giao diện chỉ có `title`/`normal`/`subtle` —
    /// không giá trị nào diễn đạt được "khác hẳn". Tức là tiêu chuẩn đòi một
    /// thứ mà chính nó không cung cấp phương tiện để nói.
    ///
    /// Đó là lỗi của tiêu chuẩn, không phải của bản cài đặt, nên sửa ở tiêu
    /// chuẩn. `Emphasis` không đánh dấu `#[non_exhaustive]`, nên mọi bộ dựng
    /// **không biên dịch được** cho tới khi xử lý giá trị mới — cái giá đã ghi
    /// từ đầu, và đây là lần thứ hai trả nó.
    Warning,
}

/// Sắc thái của một hành động. Bộ dựng ánh xạ sang màu — ứng dụng thì không.
///
/// Đây là chỗ bảo mật, không phải chỗ thẩm mỹ: nếu ứng dụng tự đặt được màu thì
/// nút xoá sổ ví có thể trông y hệt nút huỷ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Tone {
    /// Hành động thường. Mặc định trên dây.
    #[default]
    Neutral,
    /// Hành động chính của màn hình.
    Primary,
    /// Hành động MẤT MÁT hoặc KHÔNG THỂ HOÀN TÁC: xoá, chuyển tiền, ký giao dịch.
    /// Bộ dựng bắt buộc phải làm nó trông khác hẳn hai loại trên.
    Danger,
}

/// Hướng xếp các nút con.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Flow {
    Row,
    /// Mặc định trên dây: dọc là chiều đọc tự nhiên.
    #[default]
    Column,
}

/// Khoảng cách giữa các nút con, theo THANG chứ không theo pixel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Gap {
    None,
    Small,
    /// Mặc định trên dây.
    #[default]
    Medium,
    Large,
}

/// Hướng xếp của một nhóm; `None` nếu không phải nhóm.
const fn flow_cua(k: &NodeKind) -> Option<Flow> {
    match k {
        NodeKind::Group { flow, .. } => Some(*flow),
        _ => None,
    }
}

/// Chặn một bề đặt ở chỗ nó không có nghĩa.
fn kiem_be(o: &'static str, be: Extent, cam: &[Extent]) -> Result<(), UiError> {
    if cam.contains(&be) {
        return Err(UiError::BadExtent(
            o,
            match be {
                Extent::Fill => "fill",
                Extent::None => "none",
                _ => "?",
            },
        ));
    }
    Ok(())
}

/// Bề của một trục — **vốn từ ĐÓNG, không con số nào**.
///
/// Chín từ, không có từ thứ mười. Đây không phải chuyện gọn nhẹ: một ứng dụng
/// nói được một độ dài là một ứng dụng vẽ giả được thanh công cụ của trình
/// duyệt. Luật "không pixel, không màu" của 0.1 đứng vững được chính vì gói
/// không bao giờ cầm một con số hình học nào.
///
/// Sáu từ `Full`, `Half`, `Third`, `Quarter`, `TwoThirds`, `ThreeQuarters` là
/// các **phân số**, và chúng tính theo bề **TRONG** của nhóm cha (bề của cha
/// trừ `padding` hai mép). KHÔNG tính theo phần còn lại sau các anh em, và
/// KHÔNG bị `gap` trừ bớt — nên hai con `Half` trong một nhóm có `gap` thì TRÀN,
/// và tràn là đúng: co lén hai con cho vừa là để cả hai lời khai đều không có
/// tác dụng trong khi trông như đều có.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Extent {
    /// Vừa đúng thứ nút này cần trên trục ấy.
    #[default]
    Content,
    /// Một phần bằng nhau của khoảng TRỐNG trên trục chính của cha.
    Fill,
    Full,
    Half,
    Third,
    Quarter,
    TwoThirds,
    ThreeQuarters,
    /// **Không ràng buộc.** Chỉ dùng được ở `min` và `max`.
    None,
}

impl Extent {
    /// Phần của bề trong nhóm cha, nếu đây là một phân số.
    #[must_use]
    pub const fn ti_le(self) -> Option<f32> {
        match self {
            Self::Full => Some(1.0),
            Self::Half => Some(0.5),
            Self::Third => Some(1.0 / 3.0),
            Self::Quarter => Some(0.25),
            Self::TwoThirds => Some(2.0 / 3.0),
            Self::ThreeQuarters => Some(0.75),
            Self::Content | Self::Fill | Self::None => None,
        }
    }
}

/// Bề khai cho hai trục của một nhóm.
///
/// `cross` là `Option` chứ không phải `Extent::Content`, và hai thứ ấy KHÁC
/// nhau: vắng mặt là thứ cho phép `AlignCross::Stretch` kéo giãn nút, còn viết
/// rõ `Content` là **tắt** kéo giãn. Gộp chúng lại là dựng ra một màn hình khác.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Sizing {
    /// Vắng mặt KHÁC `Content`, y như ở `cross`.
    ///
    /// Vắng mặt trên trục chính là luật §8.1: nhóm chiếm trọn bề NGANG của cha,
    /// bất kể `flow` — đó là hình dạng mọi cây 0.1 đang có, và nó không phụ
    /// thuộc trục nào là trục chính. Viết rõ `Content` là tắt luật ấy đi.
    #[serde(default)]
    pub main: Option<Extent>,
    #[serde(default)]
    pub cross: Option<Extent>,
}

/// Chia khoảng trống theo trục CHÍNH.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AlignMain {
    /// Mặc định: khoảng trống dồn về SAU con cuối.
    #[default]
    Start,
    End,
    Center,
}

/// Đặt từng con theo trục PHỤ.
///
/// ⚠️ Mặc định là `Start`, **không** phải `Stretch` như CSS.
///
/// Bản nháp 0.2 lúc đầu chọn `Stretch` cho quen với flexbox. Nhưng mã đang chạy
/// xếp cột theo `Start` — và luật 1 nói điều khoản rút ra từ mã ĐÃ CHẠY, không
/// phải từ thói quen của một tiêu chuẩn khác. Đổi mặc định là đổi hình dạng của
/// cả mười hai màn hình đang có, trong đó có hộp thoại hỏi quyền và màn xác nhận
/// giao dịch: mọi nút trong một cột sẽ giãn hết bề ngang cửa sổ, và khi ấy luật
/// "nút cùng hàng rộng bằng nhau" mất chỗ bám.
///
/// Không mất gì cả: nút nào cần giãn thì viết `size.cross = Full`, hoặc viết rõ
/// `Stretch`. Cái đổi là ý nghĩa của VẮNG MẶT, và vắng mặt nên có nghĩa là "y
/// như trước".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AlignCross {
    #[default]
    Start,
    End,
    Center,
    /// Chiếm trọn bề phụ dành cho nó — chỉ có tác dụng khi `size.cross` VẮNG MẶT.
    Stretch,
}

/// Lời mô tả ảnh cho trình đọc màn hình.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Alt {
    /// Ảnh mang thông tin — bắt buộc có lời mô tả.
    Text(String),
    /// Ảnh chỉ để trang trí, trình đọc màn hình bỏ qua.
    ///
    /// Phải khai RA MIỆNG. Không có đường mặc định nào dẫn tới đây, nên "quên
    /// viết alt" không thể xảy ra — chỉ có "cố tình khai là trang trí".
    Decorative,
}

// ───────────────────────────── Cây component ────────────────────────────────

/// Loại nút.
///
/// ⚠️ CỐ Ý **KHÔNG** đánh dấu `#[non_exhaustive]`.
///
/// Thêm một loại component là ĐỔI TIÊU CHUẨN. Ta muốn mọi bộ dựng ở mọi nơi
/// KHÔNG BIÊN DỊCH ĐƯỢC cho tới khi xử lý loại mới — im lặng bỏ qua một loại nút
/// nghĩa là màn hình thiếu mất một mẩu mà không ai biết. Cái giá là mỗi lần thêm
/// loại nút phải tăng phiên bản tiêu chuẩn; đó là cái giá đúng.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    Text {
        content: String,
        emphasis: Emphasis,
    },
    Button {
        label: String,
        action: ActionId,
        tone: Tone,
    },
    Field {
        label: String,
        value: String,
        /// Ô nhập bí mật: bộ dựng phải che chữ VÀ không được đưa vào gợi ý gõ,
        /// không được ghi vào ảnh chụp màn hình tự động.
        secret: bool,
    },
    Image {
        /// Đường dẫn TRONG GÓI ĐÃ KÝ. Không bao giờ là địa chỉ mạng.
        source: String,
        alt: Alt,
    },
    /// Công tắc bật/tắt — thứ để người dùng chọn TỪNG MỤC.
    ///
    /// Thêm loại này là ĐỔI TIÊU CHUẨN, và mọi bộ dựng sẽ không biên dịch được
    /// cho tới khi xử lý nó. Đó đúng là cái giá đã ghi ở chú thích của
    /// [`NodeKind`], giờ trả lần đầu — im lặng bỏ qua một công tắc nghĩa là
    /// người dùng tưởng mình đã tắt một quyền mà thật ra chưa.
    Toggle {
        label: String,
        /// Trạng thái ban đầu. Hộp thoại hỏi quyền **luôn đặt `false`**: mặc
        /// định của một câu hỏi chưa trả lời phải là "không".
        on: bool,
        action: ActionId,
    },
    Group {
        flow: Flow,
        gap: Gap,
        /// Bề khai cho hai trục. Mặc định `content` trên trục chính, VẮNG MẶT
        /// trên trục phụ — xem [`Sizing`].
        size: Sizing,
        /// Chặn dưới. `min.main` mặc định `content`: một nút KHÔNG bị ép nhỏ hơn
        /// thứ nội dung nó cần trên trục chính. Đây là "automatic minimum size"
        /// của flexbox, và nó được giữ vì đường kia là một nút có chữ bị cắt đôi
        /// — mà luật chuỗi hiện thị của 0.1 sinh ra để chặn đúng việc chữ bị đổi
        /// giữa lúc ký và lúc hiện.
        min: Sizing,
        max: Sizing,
        align_main: AlignMain,
        align_cross: AlignCross,
        /// Đệm bốn mép BẰNG NHAU, theo cùng thang với `gap`.
        ///
        /// Không có đệm theo từng mép, vì cùng lý do không có căn lề theo từng
        /// con: chưa việc nào cần tới, và luật 1 cấm bịa ra ở đây.
        padding: Gap,
        /// Xuống dòng khi hết chỗ.
        ///
        /// `None` = **như 0.1**: luôn xuống dòng. Cây 0.1 không có chữ `wrap`
        /// nào, và một nút bị đẩy khỏi mép là một nút người dùng không bấm được
        /// và không biết là có. `Some(false)` là ứng dụng nói rõ "để nó tràn".
        wrap: Option<bool>,
        /// Vùng cuộn. Mặc định TẮT.
        scroll: bool,
    },
}

impl NodeKind {
    /// Tên loại, dùng trong thông báo lỗi.
    const fn ten(&self) -> &'static str {
        match self {
            Self::Text { .. } => "chữ",
            Self::Button { .. } => "nút bấm",
            Self::Field { .. } => "ô nhập",
            Self::Image { .. } => "ảnh",
            Self::Toggle { .. } => "công tắc",
            Self::Group { .. } => "nhóm",
        }
    }
}

/// Một nút trong cây giao diện.
///
/// Các trường để RIÊNG TƯ có chủ đích: đó là thứ khiến "dựng được nút này" đồng
/// nghĩa với "đã qua mọi phép kiểm". Không có `Node { .. }` viết tay ở đâu cả.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    kind: NodeKind,
    children: Vec<Node>,
    /// Tổng số nút của cây con, kể cả nút này. Đếm sẵn để `child` chạy O(1)
    /// thay vì duyệt lại cả cây mỗi lần thêm một nút con.
    count: usize,
    /// Độ sâu của cây con, tính cả nút này (nút lá = 1).
    depth: usize,
}

/// Kiểm một chuỗi sắp hiện ra cho người dùng.
fn kiem_chu(field: &'static str, s: &str, kind: TextKind) -> Result<(), UiError> {
    // Đếm KÝ TỰ chứ không đếm byte: cắt theo byte thì tiếng Việt có dấu bị thiệt
    // gần một nửa hạn mức so với tiếng Anh.
    let n = s.chars().count();
    if n > MAX_TEXT_LEN {
        return Err(UiError::TextTooLong(n));
    }
    check_display_text(field, s, kind)?;
    Ok(())
}

impl Node {
    /// Một đoạn chữ. Nhiều dòng được phép.
    ///
    /// # Errors
    /// Chuỗi rỗng, quá dài, hoặc chứa ký tự giả mạo hiển thị.
    pub fn text(content: impl Into<String>) -> Result<Self, UiError> {
        Self::text_with(content, Emphasis::Normal)
    }

    /// Như [`Node::text`] nhưng chọn mức nhấn.
    ///
    /// # Errors
    /// Như [`Node::text`].
    pub fn text_with(content: impl Into<String>, emphasis: Emphasis) -> Result<Self, UiError> {
        let content = content.into();
        kiem_chu("chữ", &content, TextKind::Paragraph)?;
        Ok(Self::la(NodeKind::Text { content, emphasis }))
    }

    /// Một nút bấm.
    ///
    /// # Errors
    /// Nhãn hỏng, hoặc mã hành động không hợp lệ.
    pub fn button(label: impl Into<String>, action: &str, tone: Tone) -> Result<Self, UiError> {
        let label = label.into();
        // Nhãn nút là MỘT DÒNG: đây chính là chuỗi mà người dùng đọc trước khi
        // bấm một việc có thể không hoàn tác được.
        kiem_chu("nhãn nút", &label, TextKind::Label)?;
        Ok(Self::la(NodeKind::Button {
            label,
            action: ActionId::parse(action)?,
            tone,
        }))
    }

    /// Một ô nhập liệu. `secret` bật thì bộ dựng phải che chữ.
    ///
    /// # Errors
    /// Nhãn hoặc giá trị hỏng.
    pub fn field(
        label: impl Into<String>,
        value: impl Into<String>,
        secret: bool,
    ) -> Result<Self, UiError> {
        let label = label.into();
        let value = value.into();
        kiem_chu("nhãn ô nhập", &label, TextKind::Label)?;
        // Giá trị được rỗng — ô nhập trống là chuyện thường. Chỉ kiểm khi có chữ.
        if !value.is_empty() {
            kiem_chu("giá trị ô nhập", &value, TextKind::Paragraph)?;
        }
        Ok(Self::la(NodeKind::Field {
            label,
            value,
            secret,
        }))
    }

    /// Một ảnh, lấy từ trong gói đã ký.
    ///
    /// # Errors
    /// Đường dẫn ra ngoài gói, hoặc lời mô tả hỏng.
    pub fn image(source: impl Into<String>, alt: Alt) -> Result<Self, UiError> {
        let source = source.into();
        // Chặn địa chỉ mạng TRƯỚC khi kiểm đường dẫn: `check_path_public` báo
        // "đường dẫn không hợp lệ", còn ở đây ta muốn nói rõ VÌ SAO cấm.
        if source.contains("://") {
            return Err(UiError::ExternalImage(source));
        }
        check_path_public(&source)?;
        if let Alt::Text(t) = &alt {
            kiem_chu("mô tả ảnh", t, TextKind::Label)?;
        }
        Ok(Self::la(NodeKind::Image { source, alt }))
    }

    /// Một công tắc bật/tắt.
    ///
    /// # Errors
    /// Nhãn hỏng, hoặc mã hành động không hợp lệ.
    pub fn toggle(label: impl Into<String>, on: bool, action: &str) -> Result<Self, UiError> {
        let label = label.into();
        // Nhãn công tắc là MỘT DÒNG, như nhãn nút: đây là chữ người dùng đọc
        // trước khi quyết định bật một quyền.
        kiem_chu("nhãn công tắc", &label, TextKind::Label)?;
        Ok(Self::la(NodeKind::Toggle {
            label,
            on,
            action: ActionId::parse(action)?,
        }))
    }

    /// Gắn vốn từ bố cục vào một nhóm. Trả lỗi nếu một bề đặt sai chỗ.
    ///
    /// Không có `Node { .. }` viết tay ở đâu cả, nên đây là **đường duy nhất**
    /// để một cây có bố cục tồn tại — dù nó đến từ mã Rust hay từ đĩa.
    ///
    /// # Errors
    ///
    /// [`UiError::BadExtent`] khi `fill` xuất hiện trong `min`/`max`, hoặc
    /// `none` xuất hiện trong `size`.
    /// [`UiError::NotAContainer`] khi gọi lên một nút lá.
    pub fn with_layout(
        mut self,
        size: Sizing,
        min: Sizing,
        max: Sizing,
        align_main: AlignMain,
        align_cross: AlignCross,
        padding: Gap,
    ) -> Result<Self, UiError> {
        // Đọc hướng xếp TRƯỚC khi mượn khả biến — mọi phép kiểm dưới đây chỉ
        // đọc, và mượn khả biến sớm thì chúng không đọc được nữa.
        let Some(flow) = flow_cua(&self.kind) else {
            return Err(UiError::NotAContainer(self.kind.ten()));
        };
        if let Some(m) = size.main {
            kiem_be("size.main", m, &[Extent::None])?;
        }
        if let Some(c) = size.cross {
            kiem_be("size.cross", c, &[Extent::None])?;
        }
        for (ten, be) in [("min", min), ("max", max)] {
            if let Some(m) = be.main {
                kiem_be(ten, m, &[Extent::Fill])?;
            }
            if let Some(c) = be.cross {
                kiem_be(ten, c, &[Extent::Fill])?;
            }
        }
        // Trục DỌC không nhận phân số — xem [`UiError::VerticalFraction`].
        let doc = |la_chinh: bool| match flow {
            Flow::Row => !la_chinh,
            Flow::Column => la_chinh,
        };
        for (la_chinh, ten, be) in [
            (true, "size.main", size.main),
            (false, "size.cross", size.cross),
            (true, "min.main", min.main),
            (false, "min.cross", min.cross),
            (true, "max.main", max.main),
            (false, "max.cross", max.cross),
        ] {
            if doc(la_chinh) && be.is_some_and(|b| b.ti_le().is_some()) {
                return Err(UiError::VerticalFraction(ten));
            }
        }

        let NodeKind::Group {
            size: s,
            min: mn,
            max: mx,
            align_main: am,
            align_cross: ac,
            padding: p,
            ..
        } = &mut self.kind
        else {
            unreachable!("`flow_cua` vừa xác nhận đây là nhóm")
        };
        *s = size;
        *mn = min;
        *mx = max;
        *am = align_main;
        *ac = align_cross;
        *p = padding;
        Ok(self)
    }

    /// Bật xuống dòng. Không phải nhóm thì không làm gì.
    #[must_use]
    pub fn with_wrap(mut self, bat: Option<bool>) -> Self {
        if let NodeKind::Group { wrap, .. } = &mut self.kind {
            *wrap = bat;
        }
        self
    }

    /// Bật vùng cuộn.
    ///
    /// # ⚠️ Hiện TỪ CHỐI mọi lời khai `true`
    ///
    /// Không bộ dựng nào của dự án cắt được nội dung theo nhóm. Nhận lời khai
    /// rồi vẽ ra y như không có nó thì **một lời khai bị bỏ qua trông y hệt một
    /// lời khai có tác dụng** — người viết ứng dụng khai `scroll: true`, thấy
    /// màn hình dựng lên bình thường, và tin rằng nội dung dài đã được cuộn.
    /// Trên máy họ nó "chạy"; trên màn hình nhỏ hơn nó tràn ra ngoài.
    ///
    /// Đo được ngày 23/08/2026: một nhóm `scroll: true` với bốn con cao hơn
    /// khung — cả bốn vẫn vẽ đủ, không cắt, không cuộn, và con của nhóm sau nó
    /// bị đẩy xuống như thể vùng cuộn không tồn tại.
    ///
    /// Nên từ chối. Ngày bộ dựng cắt được thì hàm này mở lại, và lúc ấy lời khai
    /// mới có nghĩa. Quyền năng không tồn tại cho tới khi được cấp.
    ///
    /// # Errors
    ///
    /// [`UiError::ScrollNotSupported`] khi `bat` là `true`.
    pub fn with_scroll(mut self, bat: bool) -> Result<Self, UiError> {
        if !bat {
            return Ok(self);
        }
        if let NodeKind::Group { scroll, .. } = &mut self.kind {
            *scroll = true;
            return Err(UiError::ScrollNotSupported);
        }
        Ok(self)
    }

    /// Một nhóm chứa các nút khác. Đây là loại DUY NHẤT nhận nút con.
    ///
    /// Chữ ký giữ nguyên hai tham số của 0.1 có chủ đích: một cây 0.1 dựng bằng
    /// hàm này phải ra đúng màn hình như trước khi có vốn từ bố cục. Muốn dùng
    /// vốn từ mới thì gọi tiếp [`Node::with_layout`].
    #[must_use]
    pub fn group(flow: Flow, gap: Gap) -> Self {
        Self {
            kind: NodeKind::Group {
                flow,
                gap,
                size: Sizing::default(),
                min: Sizing {
                    main: Some(Extent::Content),
                    cross: Some(Extent::None),
                },
                max: Sizing {
                    main: Some(Extent::None),
                    cross: Some(Extent::None),
                },
                align_main: AlignMain::Start,
                align_cross: AlignCross::Start,
                padding: Gap::None,
                wrap: None,
                scroll: false,
            },
            children: Vec::new(),
            count: 1,
            depth: 1,
        }
    }

    fn la(kind: NodeKind) -> Self {
        Self {
            kind,
            children: Vec::new(),
            count: 1,
            depth: 1,
        }
    }

    /// Thêm một nút con. Trần độ sâu và số nút được kiểm NGAY TẠI ĐÂY.
    ///
    /// Kiểm lúc dựng chứ không kiểm lúc vẽ, vì lúc vẽ thì cây khổng lồ đã nằm
    /// trong bộ nhớ rồi — chặn muộn không cứu được gì.
    ///
    /// # Errors
    /// Nút này là lá, hoặc cây vượt trần.
    pub fn child(mut self, con: Node) -> Result<Self, UiError> {
        if !matches!(self.kind, NodeKind::Group { .. }) {
            return Err(UiError::NotAContainer(self.kind.ten()));
        }
        let sau_moi = 1 + con.depth;
        if sau_moi > MAX_DEPTH {
            return Err(UiError::TooDeep(sau_moi));
        }
        let dem_moi = self.count + con.count;
        if dem_moi > MAX_NODES {
            return Err(UiError::TooManyNodes(dem_moi));
        }
        // `fill` chỉ có nghĩa khi CHA có khoảng trống trên trục chính của nó —
        // và ở đây chỉ trục NGANG mới có bề xác định. Cha kiểu cột thì trục
        // chính là dọc, dọc thì suy từ nội dung, và suy từ nội dung thì không
        // có khoảng trống nào để chia. Xem [`UiError::FillWithoutRoom`].
        if flow_cua(&self.kind) == Some(Flow::Column)
            && matches!(
                con.kind,
                NodeKind::Group {
                    size: Sizing {
                        main: Some(Extent::Fill),
                        ..
                    },
                    ..
                }
            )
        {
            return Err(UiError::FillWithoutRoom);
        }
        self.depth = self.depth.max(sau_moi);
        self.count = dem_moi;
        self.children.push(con);
        Ok(self)
    }

    #[must_use]
    pub const fn kind(&self) -> &NodeKind {
        &self.kind
    }

    #[must_use]
    pub fn children(&self) -> &[Node] {
        &self.children
    }

    /// Dựng lại cây với các công tắc trong `bat` ở trạng thái **bật**, còn lại
    /// **tắt**.
    ///
    /// # Vì sao trạng thái công tắc do BÊN NGOÀI giữ, không do cây giữ
    ///
    /// Bộ dựng đầu tiên của dự án là một máy dựng web, và nó để trình duyệt giữ
    /// hộ trạng thái công tắc trong tài liệu rồi hỏi lại lúc bấm xác nhận. Bộ
    /// dựng ra pixel không có ai giữ hộ, nên khung phải tự giữ — và hàm này là
    /// chỗ trạng thái ấy quay lại thành một cây vẽ được.
    ///
    /// Cách này hoá ra đúng hơn, và nó sống lâu hơn cái nó thay: trạng thái nằm
    /// ở khung thì cây vẫn bất biến, và một câu trả lời chưa được xác nhận
    /// không bao giờ nằm trong thứ đang được vẽ.
    ///
    /// Cây **bất biến**: hàm trả về cây MỚI. Sửa tại chỗ thì một màn hình đã vẽ
    /// và một màn hình sắp vẽ dùng chung một đối tượng, mà hai thứ ấy phải so
    /// được với nhau — đó là cách bắt được một công tắc bị gạt mà màn hình không
    /// đổi.
    ///
    /// # Errors
    /// Không có: cây vào đã hợp lệ nên cây ra cũng hợp lệ. Kiểu trả về giữ
    /// `Result` vì các hàm dựng đều kiểm, và bỏ qua lỗi của chúng bằng `unwrap`
    /// ở đây là đặt một chỗ hoảng loạn vào đường chạy giao diện.
    pub fn with_toggles(&self, bat: &BTreeSet<String>) -> Result<Self, UiError> {
        self.dat_lai(&|n| match &n.kind {
            NodeKind::Toggle { label, action, .. } => Some(NodeKind::Toggle {
                label: label.clone(),
                on: bat.contains(action.as_str()),
                action: action.clone(),
            }),
            _ => None,
        })
    }

    /// Dựng lại cây với **nội dung ô nhập** lấy từ `noi_dung`, tra theo NHÃN.
    ///
    /// # Vì sao tra theo nhãn chứ không theo mã hành động
    ///
    /// Ô nhập **không có** mã hành động — tiêu chuẩn 0.1 không định nghĩa hành
    /// động nào cho nó. Nhãn là thứ duy nhất phân biệt được hai ô — và nó cũng
    /// chính là thứ trình đọc màn hình đọc lên, nên hai bên nói cùng một tên.
    ///
    /// Nhãn trùng nhau thì hai ô dùng chung một giá trị. Chấp nhận được ở 0.1:
    /// hai ô cùng nhãn vốn đã là một màn hình hỏng — người dùng không phân biệt
    /// nổi chúng.
    ///
    /// # Errors
    /// Như [`Self::with_toggles`].
    pub fn with_fields(&self, noi_dung: &BTreeMap<String, String>) -> Result<Self, UiError> {
        self.dat_lai(&|n| match &n.kind {
            NodeKind::Field { label, secret, .. } => Some(NodeKind::Field {
                label: label.clone(),
                value: noi_dung.get(label).cloned().unwrap_or_default(),
                secret: *secret,
            }),
            _ => None,
        })
    }

    /// Dựng lại cây, thay loại nút nào hàm `doi` trả về `Some`.
    ///
    /// Gom lại vì `with_toggles` và `with_fields` khác nhau đúng một dòng, và
    /// hai bản sao của cùng một phép duyệt cây là hai chỗ phải sửa khi trần độ
    /// sâu hay số nút đổi — sẽ có một chỗ bị quên.
    fn dat_lai(&self, doi: &dyn Fn(&Self) -> Option<NodeKind>) -> Result<Self, UiError> {
        let kind = doi(self).unwrap_or_else(|| self.kind.clone());
        let mut moi = Self::la(kind);
        for c in &self.children {
            moi = moi.child(c.dat_lai(doi)?)?;
        }
        Ok(moi)
    }

    /// Tổng số nút, kể cả nút này.
    #[must_use]
    pub const fn node_count(&self) -> usize {
        self.count
    }

    /// Độ sâu, nút lá là 1.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.depth
    }

    /// Mọi mã hành động có mặt trong cây, theo thứ tự xuất hiện.
    ///
    /// Bộ dựng dùng danh sách này làm DANH SÁCH TRẮNG: hành động nào không có
    /// trên màn hình thì không được nhận. Chính sách nội dung đã chặn kịch bản
    /// của ứng dụng rồi, nên đây là phòng thủ tầng hai — nhưng "hành động ma"
    /// đúng là loại lỗ mà một tầng phòng thủ là không đủ.
    #[must_use]
    pub fn action_ids(&self) -> Vec<&ActionId> {
        let mut ra = Vec::new();
        self.gom_action(&mut ra);
        ra
    }

    fn gom_action<'a>(&'a self, ra: &mut Vec<&'a ActionId>) {
        match &self.kind {
            NodeKind::Button { action, .. } | NodeKind::Toggle { action, .. } => ra.push(action),
            _ => {}
        }
        for c in &self.children {
            c.gom_action(ra);
        }
    }

    /// Cây trợ năng tương ứng.
    ///
    /// Không trả `Option`, không có cờ bật/tắt: mọi nút đều sinh ra được một nút
    /// trợ năng, vì các hàm dựng đã đòi đủ dữ liệu ngay từ đầu.
    #[must_use]
    pub fn accessibility_tree(&self) -> AccessNode {
        let (role, label) = match &self.kind {
            NodeKind::Text { content, .. } => (Role::Text, Some(content.clone())),
            NodeKind::Button { label, tone, .. } => (
                Role::Button {
                    destructive: *tone == Tone::Danger,
                },
                Some(label.clone()),
            ),
            NodeKind::Field { label, secret, .. } => {
                (Role::TextInput { secret: *secret }, Some(label.clone()))
            }
            // Ảnh trang trí: nhãn `None` là tín hiệu cho trình đọc màn hình BỎ
            // QUA. Đọc tên tệp lên là tệ hơn im lặng.
            NodeKind::Image { alt, .. } => (
                Role::Image,
                match alt {
                    Alt::Text(t) => Some(t.clone()),
                    Alt::Decorative => None,
                },
            ),
            // Trạng thái bật/tắt PHẢI nằm trong vai trò, không phải trong nhãn:
            // trình đọc màn hình đọc trạng thái công tắc theo cách riêng, và
            // nhét chữ "đã bật" vào nhãn thì nó đọc thành một phần của tên.
            NodeKind::Toggle { label, on, .. } => (Role::Switch { on: *on }, Some(label.clone())),
            NodeKind::Group { .. } => (Role::Group, None),
        };
        // Chỉ nút và công tắc mới kích hoạt được. Ô nhập KHÔNG: 0.1 không có
        // hành động nào cho ô nhập, và bịa ra một hành động ở đây là bịa ra một
        // thứ không ai khai báo.
        let action = match &self.kind {
            NodeKind::Button { action, .. } | NodeKind::Toggle { action, .. } => {
                Some(action.as_str().to_owned())
            }
            _ => None,
        };
        AccessNode {
            role,
            label,
            action,
            children: self.children.iter().map(Node::accessibility_tree).collect(),
        }
    }
}

// ───────────────────────────── Trợ năng ─────────────────────────────────────

/// Vai trò của một nút, theo cách hệ điều hành hiểu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Text,
    Button {
        /// Hành động mất mát hoặc không hoàn tác được. Hệ điều hành có cách báo
        /// riêng cho loại này; giấu nó đi là lừa người dùng trình đọc màn hình.
        destructive: bool,
    },
    TextInput {
        secret: bool,
    },
    Switch {
        on: bool,
    },
    Image,
    Group,
}

/// Một nút trong cây trợ năng.
///
/// `label` là `None` CHỈ với ảnh trang trí và nhóm — hai thứ mà trình đọc màn
/// hình đi xuyên qua.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessNode {
    pub role: Role,
    pub label: Option<String>,
    /// Mã hành động, nếu nút này **kích hoạt được**. `None` = chữ, ảnh, nhóm.
    ///
    /// # Vì sao cây trợ năng phải mang mã hành động
    ///
    /// Trình đọc màn hình không chỉ ĐỌC — nó còn **bấm**. Một cây trợ năng chỉ
    /// mang nhãn thì người dùng VoiceOver nghe được nút tên gì rồi không làm gì
    /// được với nó, và bộ dựng phải đoán ngược từ vị trí trên màn hình để biết
    /// hệ điều hành vừa yêu cầu bấm cái gì.
    ///
    /// Đoán ngược là chỗ hỏng: bấm nhầm một nút không hoàn tác thì không có
    /// đường lùi. Nên mã hành động đi CÙNG nút, không tra lại.
    pub action: Option<String>,
    pub children: Vec<AccessNode>,
}

// ───────────────────────────── Bộ dựng ──────────────────────────────────────

/// Thứ biến cây component thành hình ảnh trên màn hình.
///
/// Trait này CỐ Ý bé. Mọi thứ khó — bố cục, phông chữ, tăng tốc phần cứng — nằm
/// bên trong bộ dựng. Trait càng bé thì đổi bộ dựng càng rẻ, mà giữ cho việc đổi
/// bộ dựng luôn rẻ chính là lý do tồn tại của cả tầng này.
pub trait Renderer {
    type Error;

    /// Vẽ cây.
    ///
    /// # Errors
    /// Tuỳ bộ dựng.
    fn render(&mut self, tree: &Node) -> Result<(), Self::Error>;

    /// Cây trợ năng mà bộ dựng ĐÃ THẬT SỰ công bố ra hệ điều hành sau lần
    /// [`Renderer::render`] gần nhất.
    ///
    /// Đây là móc để KIỂM ĐỊNH, không phải để ứng dụng gọi. Không có nó thì câu
    /// "bộ dựng phải công bố trợ năng" là một lời hứa suông; có nó thì
    /// [`check_accessibility_parity`] biến lời hứa thành một phép thử chạy được.
    ///
    /// Trả `None` nghĩa là bộ dựng chưa vẽ lần nào.
    fn published_accessibility(&self) -> Option<AccessNode>;
}

#[derive(Debug, Error)]
pub enum ParityError<E> {
    #[error("bộ dựng vẽ thất bại: {0}")]
    Render(E),

    #[error("bộ dựng không công bố cây trợ năng nào sau khi vẽ")]
    NothingPublished,

    #[error(
        "cây trợ năng bộ dựng công bố KHÁC cây đúng — người dùng trình đọc màn hình \
         đang nghe một giao diện khác với giao diện hiện trên màn hình"
    )]
    Mismatch {
        mong_doi: Box<AccessNode>,
        thuc_te: Box<AccessNode>,
    },
}

/// Phép kiểm định số 1 của tiêu chuẩn: bộ dựng phải công bố ĐÚNG cây trợ năng.
///
/// Vẽ đúng mà công bố sai là lỗi im lặng tệ nhất trong giao diện — người sáng
/// mắt thấy màn hình đúng, người dùng trình đọc màn hình nghe một màn hình khác,
/// và không ai trong hai bên phát hiện ra.
///
/// # Errors
/// Bộ dựng vẽ hỏng, không công bố gì, hoặc công bố lệch.
pub fn check_accessibility_parity<R: Renderer>(
    bo_dung: &mut R,
    tree: &Node,
) -> Result<(), ParityError<R::Error>> {
    bo_dung.render(tree).map_err(ParityError::Render)?;
    let thuc_te = bo_dung
        .published_accessibility()
        .ok_or(ParityError::NothingPublished)?;
    let mong_doi = tree.accessibility_tree();
    if thuc_te == mong_doi {
        Ok(())
    } else {
        Err(ParityError::Mismatch {
            mong_doi: Box::new(mong_doi),
            thuc_te: Box::new(thuc_te),
        })
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay"
)]
mod kiem_thu {

    /// **Phân số trên trục DỌC bị TỪ CHỐI; trên trục NGANG thì nhận.**
    ///
    /// Nửa nào dùng được thì giữ, nửa nào không thì chặn — chứ không cấm cả hai
    /// cho gọn, và cũng không nhận cả hai rồi để một nửa lặng lẽ không có tác
    /// dụng.
    #[test]
    fn phan_so_chi_nhan_tren_truc_ngang() {
        let khai = |flow, size| {
            Node::group(flow, Gap::Medium).with_layout(
                size,
                Sizing::default(),
                Sizing::default(),
                AlignMain::Start,
                AlignCross::Start,
                Gap::None,
            )
        };
        let nua = Sizing {
            main: Some(Extent::Half),
            cross: None,
        };
        // HÀNG: trục chính là NGANG → nhận.
        assert!(khai(Flow::Row, nua).is_ok());
        // CỘT: trục chính là DỌC → từ chối.
        let e = khai(Flow::Column, nua).expect_err("phân số dọc phải bị từ chối");
        assert_eq!(e.ma(), "bad-layout", "{e}");

        // Và ngược lại với trục phụ.
        let nua_phu = Sizing {
            main: None,
            cross: Some(Extent::Half),
        };
        assert!(khai(Flow::Column, nua_phu).is_ok());
        assert!(khai(Flow::Row, nua_phu).is_err());

        // `content` và `fill` KHÔNG phải phân số — chúng đi qua ở cả hai trục.
        for be in [Extent::Content, Extent::Fill] {
            assert!(
                khai(
                    Flow::Column,
                    Sizing {
                        main: Some(be),
                        cross: None
                    }
                )
                .is_ok(),
                "{be:?} bị chặn nhầm"
            );
        }
    }

    /// **Vùng cuộn bị TỪ CHỐI, không bị lờ đi.**
    ///
    /// Cây vẫn dựng được `scroll: true` về mặt hình dạng — cái bị chặn là dùng
    /// nó. Phân biệt ấy quan trọng: nếu chỉ lặng lẽ bỏ trường đi thì người viết
    /// ứng dụng khai một đằng, màn hình dựng một nẻo, và không ai báo gì.
    #[test]
    fn vung_cuon_bi_tu_choi_chu_khong_bi_lo_di() {
        let e = Node::group(Flow::Column, Gap::Medium)
            .with_scroll(true)
            .expect_err("phải từ chối");
        assert_eq!(e.ma(), "bad-scroll", "{e}");

        // `false` thì đi qua: đó là trạng thái mặc định, không phải một lời khai.
        assert!(
            Node::group(Flow::Column, Gap::Medium)
                .with_scroll(false)
                .is_ok()
        );

        // Và đường từ ĐĨA cũng bị chặn — không có cửa sau nào cho gói.
        let tu_dia = crate::wire::decode(br#"{"kind":"group","scroll":true}"#)
            .expect_err("gói khai `scroll` phải bị từ chối");
        assert_eq!(tu_dia.ma(), "bad-scroll", "{tu_dia}");
    }

    use super::*;
    use std::fmt::Write as _;

    /// Bộ dựng GIẢ — không dính gì tới WebView.
    ///
    /// Sự tồn tại của nó là bằng chứng cho luật lớn nhất của crate này: nếu
    /// `tcc-ui` có lỡ để lộ một khái niệm của trang web, bộ dựng này sẽ không
    /// viết nổi. Nó vẽ ra chữ, và chữ đó là thứ ta so sánh được trong phép thử.
    #[derive(Default)]
    struct BoDungGia {
        ra: String,
        da_cong_bo: Option<AccessNode>,
    }

    impl BoDungGia {
        fn ve(&mut self, n: &Node, tang: usize) {
            let lui = "  ".repeat(tang);
            match n.kind() {
                NodeKind::Text { content, emphasis } => {
                    let _ = writeln!(self.ra, "{lui}chữ[{emphasis:?}] {content}");
                }
                NodeKind::Button {
                    label,
                    action,
                    tone,
                } => {
                    let _ = writeln!(self.ra, "{lui}nút[{tone:?}] {label} -> {}", action.as_str());
                }
                NodeKind::Field {
                    label,
                    value,
                    secret,
                } => {
                    let v = if *secret {
                        "••••"
                    } else {
                        value.as_str()
                    };
                    let _ = writeln!(self.ra, "{lui}ô[{label}] {v}");
                }
                NodeKind::Image { source, alt } => {
                    let _ = writeln!(self.ra, "{lui}ảnh {source} ({alt:?})");
                }
                NodeKind::Toggle { label, on, action } => {
                    let _ = writeln!(
                        self.ra,
                        "{lui}công tắc[{}] {label} -> {}",
                        if *on { "BẬT" } else { "tắt" },
                        action.as_str()
                    );
                }
                NodeKind::Group { flow, gap, .. } => {
                    let _ = writeln!(self.ra, "{lui}nhóm[{flow:?},{gap:?}]");
                }
            }
            for c in n.children() {
                self.ve(c, tang + 1);
            }
        }
    }

    impl Renderer for BoDungGia {
        type Error = std::convert::Infallible;

        fn render(&mut self, tree: &Node) -> Result<(), Self::Error> {
            self.ra.clear();
            self.ve(tree, 0);
            self.da_cong_bo = Some(tree.accessibility_tree());
            Ok(())
        }

        fn published_accessibility(&self) -> Option<AccessNode> {
            self.da_cong_bo.clone()
        }
    }

    fn man_hinh_vi() -> Node {
        Node::group(Flow::Column, Gap::Medium)
            .child(Node::text_with("Ví TCC", Emphasis::Title).unwrap())
            .unwrap()
            .child(Node::text("Số dư: 3 TCC").unwrap())
            .unwrap()
            .child(Node::field("Mật khẩu", "", true).unwrap())
            .unwrap()
            .child(Node::button("Gửi tiền", "gui-tien", Tone::Danger).unwrap())
            .unwrap()
    }

    // ---- Trợ năng ----

    #[test]
    fn moi_nut_deu_sinh_ra_nut_tro_nang() {
        let t = man_hinh_vi();
        let a = t.accessibility_tree();
        assert_eq!(a.children.len(), t.children().len());
        assert_eq!(a.role, Role::Group);
    }

    /// ⚠️ Nút gây mất mát phải LỘ RA trong cây trợ năng.
    ///
    /// Người dùng trình đọc màn hình không thấy nút đỏ. Nếu vai trò không mang
    /// theo cờ này thì "Gửi tiền" nghe giống hệt "Xem lịch sử".
    #[test]
    fn nut_nguy_hiem_bao_ra_o_cay_tro_nang() {
        let a = man_hinh_vi().accessibility_tree();
        let nut = a
            .children
            .iter()
            .find(|c| matches!(c.role, Role::Button { .. }))
            .unwrap();
        assert_eq!(nut.role, Role::Button { destructive: true });
        assert_eq!(nut.label.as_deref(), Some("Gửi tiền"));
    }

    #[test]
    fn o_nhap_bi_mat_bao_ra_o_cay_tro_nang() {
        let a = man_hinh_vi().accessibility_tree();
        assert!(
            a.children
                .iter()
                .any(|c| c.role == Role::TextInput { secret: true })
        );
    }

    #[test]
    fn anh_trang_tri_khong_co_nhan_anh_thuong_thi_co() {
        let tt = Node::image("anh/vien.png", Alt::Decorative).unwrap();
        assert_eq!(tt.accessibility_tree().label, None);

        let co = Node::image("anh/bieu-do.png", Alt::Text("Biểu đồ giá".into())).unwrap();
        assert_eq!(
            co.accessibility_tree().label.as_deref(),
            Some("Biểu đồ giá")
        );
    }

    // ---- Kiểm định bộ dựng ----

    #[test]
    fn bo_dung_that_tha_thi_qua_kiem_dinh() {
        let mut bd = BoDungGia::default();
        assert!(check_accessibility_parity(&mut bd, &man_hinh_vi()).is_ok());
        assert!(bd.ra.contains("nút[Danger] Gửi tiền -> gui-tien"));
        // Ô bí mật KHÔNG được lọt giá trị ra bản vẽ.
        assert!(bd.ra.contains("••••"));
    }

    /// Phép kiểm định phải BẮT được bộ dựng nói dối, nếu không nó vô dụng.
    #[test]
    fn bo_dung_cong_bo_sai_thi_kiem_dinh_bat_duoc() {
        struct NoiDoi(Option<AccessNode>);
        impl Renderer for NoiDoi {
            type Error = std::convert::Infallible;
            fn render(&mut self, _tree: &Node) -> Result<(), Self::Error> {
                // Vẽ đúng màn hình ví, nhưng công bố một màn hình trống trơn.
                self.0 = Some(AccessNode {
                    role: Role::Group,
                    label: None,
                    action: None,
                    children: vec![],
                });
                Ok(())
            }
            fn published_accessibility(&self) -> Option<AccessNode> {
                self.0.clone()
            }
        }
        let ket = check_accessibility_parity(&mut NoiDoi(None), &man_hinh_vi());
        assert!(matches!(ket, Err(ParityError::Mismatch { .. })));
    }

    #[test]
    fn bo_dung_khong_cong_bo_gi_thi_truot() {
        struct Cam;
        impl Renderer for Cam {
            type Error = std::convert::Infallible;
            fn render(&mut self, _t: &Node) -> Result<(), Self::Error> {
                Ok(())
            }
            fn published_accessibility(&self) -> Option<AccessNode> {
                None
            }
        }
        let ket = check_accessibility_parity(&mut Cam, &man_hinh_vi());
        assert!(matches!(ket, Err(ParityError::NothingPublished)));
    }

    // ---- Chuỗi hiện ra người dùng ----

    /// ⚠️ Đòn giả mạo giao diện: nhãn nút chứa ký tự đảo chiều chữ.
    #[test]
    fn nhan_nut_co_ky_tu_dao_chieu_thi_tu_choi() {
        let ket = Node::button("Huỷ\u{202e}", "huy", Tone::Neutral);
        assert!(ket.is_err(), "nhãn giả mạo lọt vào nút bấm");
    }

    #[test]
    fn nhan_nut_khong_duoc_xuong_dong_nhung_doan_van_thi_duoc() {
        assert!(Node::button("Đồng\ný", "ok", Tone::Neutral).is_err());
        assert!(Node::text("Dòng một\nDòng hai").is_ok());
    }

    #[test]
    fn tieng_viet_co_dau_va_emoji_thi_qua() {
        assert!(Node::text("Chào bạn — số dư 3 TCC 🎉").is_ok());
        assert!(Node::button("Xoá ví", "xoa-vi", Tone::Danger).is_ok());
    }

    #[test]
    fn chuoi_qua_dai_thi_tu_choi() {
        let dai = "a".repeat(MAX_TEXT_LEN + 1);
        assert!(matches!(
            Node::text(dai),
            Err(UiError::TextTooLong(n)) if n == MAX_TEXT_LEN + 1
        ));
    }

    /// Hạn mức tính bằng KÝ TỰ, không phải byte — nếu không thì tiếng Việt có
    /// dấu chỉ được viết bằng một nửa tiếng Anh.
    #[test]
    fn han_muc_tinh_theo_ky_tu_khong_theo_byte() {
        let viet = "ế".repeat(MAX_TEXT_LEN);
        assert!(
            viet.len() > MAX_TEXT_LEN,
            "phép thử tự hỏng: chuỗi chưa đủ nặng byte"
        );
        assert!(Node::text(viet).is_ok(), "tiếng Việt bị cắt hạn mức oan");
    }

    // ---- Mã hành động ----

    #[test]
    fn ma_hanh_dong_chi_nhan_ascii_hep() {
        assert!(ActionId::parse("gui-tien.xac-nhan").is_ok());
        for xau in ["", "Gui", "gửi", "gui tien", "gui/tien", "gui\u{0}"] {
            assert!(ActionId::parse(xau).is_err(), "\"{xau}\" lọt qua");
        }
    }

    // ---- Ảnh ----

    /// ⚠️ Ảnh từ mạng là một cái đèn báo hiệu: chủ máy chủ ảnh biết ai mở màn
    /// hình nào, lúc nào, từ địa chỉ nào — mà ứng dụng thì chưa xin quyền mạng.
    #[test]
    fn anh_tro_ra_mang_thi_tu_choi() {
        for u in [
            "https://theo-doi.example/1.png",
            "http://a/b.png",
            "data://x",
        ] {
            assert!(
                matches!(
                    Node::image(u, Alt::Decorative),
                    Err(UiError::ExternalImage(_))
                ),
                "\"{u}\" lọt qua"
            );
        }
    }

    #[test]
    fn anh_di_ra_khoi_goi_thi_tu_choi() {
        for p in ["../bi-mat.png", "/etc/passwd", "anh//x.png", ""] {
            assert!(Node::image(p, Alt::Decorative).is_err(), "\"{p}\" lọt qua");
        }
    }

    // ---- Trần cây ----

    #[test]
    fn liet_ke_duoc_moi_ma_hanh_dong() {
        let cay = Node::group(Flow::Column, Gap::None)
            .child(Node::button("A", "lam-a", Tone::Neutral).unwrap())
            .unwrap()
            .child(
                Node::group(Flow::Row, Gap::None)
                    .child(Node::button("B", "lam-b", Tone::Danger).unwrap())
                    .unwrap()
                    .child(Node::text("không phải nút").unwrap())
                    .unwrap(),
            )
            .unwrap();
        let ds: Vec<&str> = cay.action_ids().iter().map(|a| a.as_str()).collect();
        assert_eq!(ds, ["lam-a", "lam-b"], "sót hành động lồng sâu");
    }

    #[test]
    fn nut_la_khong_nhan_nut_con() {
        let ket = Node::text("xin chào")
            .unwrap()
            .child(Node::text("con").unwrap());
        assert!(matches!(ket, Err(UiError::NotAContainer("chữ"))));
    }

    #[test]
    fn cay_qua_sau_thi_bi_chan() {
        let mut n = Node::group(Flow::Column, Gap::None);
        // Dựng đúng tới trần thì phải đạt…
        for _ in 1..MAX_DEPTH {
            n = Node::group(Flow::Column, Gap::None).child(n).unwrap();
        }
        assert_eq!(n.depth(), MAX_DEPTH);
        // …thêm một tầng nữa thì hỏng.
        let ket = Node::group(Flow::Column, Gap::None).child(n);
        assert!(matches!(ket, Err(UiError::TooDeep(n)) if n == MAX_DEPTH + 1));
    }

    #[test]
    fn cay_qua_nhieu_nut_thi_bi_chan() {
        let mut g = Node::group(Flow::Column, Gap::None);
        for _ in 1..MAX_NODES {
            g = g.child(Node::text("x").unwrap()).unwrap();
        }
        assert_eq!(g.node_count(), MAX_NODES);
        let ket = g.child(Node::text("giọt tràn ly").unwrap());
        assert!(matches!(ket, Err(UiError::TooManyNodes(n)) if n == MAX_NODES + 1));
    }

    /// Trần phải tính theo CẢ CÂY CON, không phải theo số con trực tiếp — nếu
    /// không thì ghép hai cây vừa đủ trần lại là vượt trần mà không ai chặn.
    #[test]
    fn tran_tinh_theo_ca_cay_con_khong_theo_con_truc_tiep() {
        let mut nua = Node::group(Flow::Row, Gap::None);
        for _ in 0..(MAX_NODES / 2) {
            nua = nua.child(Node::text("x").unwrap()).unwrap();
        }
        let ket = Node::group(Flow::Row, Gap::None)
            .child(nua.clone())
            .unwrap()
            .child(nua);
        assert!(
            matches!(ket, Err(UiError::TooManyNodes(_))),
            "ghép hai cây con lại thì vượt trần mà không bị chặn"
        );
    }

    /// **Gạt một công tắc thì CHỈ công tắc ấy đổi.**
    ///
    /// Cây dựng lại phải giống hệt cây cũ ở mọi chỗ khác — cùng số nút, cùng
    /// chữ, cùng thứ tự. Một hàm dựng lại cây mà đánh rơi một nút là một màn
    /// hình thiếu mất một dòng người dùng cần đọc trước khi cấp quyền.
    #[test]
    fn gat_cong_tac_chi_doi_dung_cong_tac_ay() {
        fn di(n: &Node, ra: &mut Vec<(String, bool)>) {
            if let NodeKind::Toggle { label, on, .. } = n.kind() {
                ra.push((label.clone(), *on));
            }
            for c in n.children() {
                di(c, ra);
            }
        }
        let cay = Node::group(Flow::Column, Gap::Medium)
            .child(Node::text("Ứng dụng này xin:").unwrap())
            .unwrap()
            .child(Node::toggle("Micro", false, "micro").unwrap())
            .unwrap()
            .child(Node::toggle("Camera", false, "camera").unwrap())
            .unwrap()
            .child(Node::button("Xong", "xong", Tone::Primary).unwrap())
            .unwrap();

        let mut bat = std::collections::BTreeSet::new();
        bat.insert("micro".to_owned());
        let moi = cay.with_toggles(&bat).unwrap();

        assert_eq!(moi.node_count(), cay.node_count(), "dựng lại làm rơi nút");
        let trang_thai = |n: &Node| {
            let mut ra = Vec::new();
            di(n, &mut ra);
            ra
        };
        assert_eq!(
            trang_thai(&moi),
            vec![("Micro".to_owned(), true), ("Camera".to_owned(), false)]
        );
        // Cây gốc KHÔNG đổi: hai màn hình phải so được với nhau.
        assert_eq!(
            trang_thai(&cay),
            vec![("Micro".to_owned(), false), ("Camera".to_owned(), false)]
        );
    }

    /// Tập rỗng thì **mọi công tắc tắt** — kể cả công tắc vốn đang bật.
    ///
    /// Mặc định của một câu hỏi chưa trả lời là "không", và hàm này không được
    /// có một đường nào giữ lại trạng thái bật cũ.
    #[test]
    fn tap_rong_thi_moi_cong_tac_tat() {
        let cay = Node::toggle("Micro", true, "micro").unwrap();
        let moi = cay
            .with_toggles(&std::collections::BTreeSet::new())
            .unwrap();
        assert!(matches!(moi.kind(), NodeKind::Toggle { on: false, .. }));
    }

    /// **Nội dung ô nhập dựng lại đúng ô, không đụng ô khác.**
    #[test]
    fn dat_noi_dung_o_nhap_dung_o() {
        fn di(n: &Node, ra: &mut Vec<(String, String)>) {
            if let NodeKind::Field { label, value, .. } = n.kind() {
                ra.push((label.clone(), value.clone()));
            }
            for c in n.children() {
                di(c, ra);
            }
        }
        let cay = Node::group(Flow::Column, Gap::Medium)
            .child(Node::field("Địa chỉ", "", false).unwrap())
            .unwrap()
            .child(Node::field("Ghi nhớ", "cũ", false).unwrap())
            .unwrap()
            .child(Node::text("không phải ô nhập").unwrap())
            .unwrap();

        let mut noi_dung = std::collections::BTreeMap::new();
        noi_dung.insert("Địa chỉ".to_owned(), "chào buổi sáng".to_owned());
        let moi = cay.with_fields(&noi_dung).unwrap();

        assert_eq!(moi.node_count(), cay.node_count(), "dựng lại làm rơi nút");
        let mut o = Vec::new();
        di(&moi, &mut o);
        assert_eq!(
            o,
            vec![
                ("Địa chỉ".to_owned(), "chào buổi sáng".to_owned()),
                // KHÔNG có trong bảng → rỗng. Bảng là nguồn sự thật DUY NHẤT,
                // không phải "giữ giá trị cũ nếu chưa gõ" — giữ lại là hai chỗ
                // nhớ cùng một thứ, và chúng sẽ lệch nhau.
                ("Ghi nhớ".to_owned(), String::new()),
            ]
        );
        // Cây gốc không đổi.
        let mut cu = Vec::new();
        di(&cay, &mut cu);
        assert_eq!(cu[1].1, "cũ");
    }

    /// Ô **che chữ** vẫn che sau khi dựng lại — mất cờ ấy là lộ mật khẩu.
    #[test]
    fn dung_lai_khong_lam_mat_co_che_chu() {
        let cay = Node::field("PIN", "", true).unwrap();
        let mut n = std::collections::BTreeMap::new();
        n.insert("PIN".to_owned(), "1234".to_owned());
        let moi = cay.with_fields(&n).unwrap();
        assert!(matches!(moi.kind(), NodeKind::Field { secret: true, .. }));
    }
}
