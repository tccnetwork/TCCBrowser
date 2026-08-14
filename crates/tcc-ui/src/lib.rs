//! API component — TRỪU TƯỢNG, không biết mình được dựng bằng gì.
//!
//! VIỆC CỦA CRATE NÀY: định nghĩa cây component mà ứng dụng TCC nhắm tới.
//!
//! ⚠️ LUẬT QUAN TRỌNG NHẤT CỦA CẢ DỰ ÁN:
//!
//! **Crate này KHÔNG được phụ thuộc bộ dựng nào, và KHÔNG được lộ ra tài liệu
//! trang web, thẻ đánh dấu hay bảng kiểu.**
//!
//! Vì sao: giai đoạn đầu ta mượn WebView làm bộ dựng. Nếu ứng dụng TCC nhìn thấy
//! cây tài liệu của trang web, thì ngày có bộ dựng GPU riêng, MỌI ứng dụng phải
//! viết lại — và lúc đó không ai dám bỏ WebView nữa. Giàn giáo hoá thành nhà.
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

    /// Một nhóm chứa các nút khác. Đây là loại DUY NHẤT nhận nút con.
    #[must_use]
    pub fn group(flow: Flow, gap: Gap) -> Self {
        Self {
            kind: NodeKind::Group { flow, gap },
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
        AccessNode {
            role,
            label,
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
#[allow(clippy::unwrap_used, reason = "kiểm thử: hỏng thì phải nổ ngay")]
mod kiem_thu {
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
                NodeKind::Group { flow, gap } => {
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
}
