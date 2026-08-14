//! Dạng cây giao diện KHI NẰM TRONG GÓI — tức là dạng trên dây, dạng của tiêu
//! chuẩn.
//!
//! # Vì sao ứng dụng KHÔNG ship thẻ đánh dấu
//!
//! Bản đầu, `tcc new` sinh ra `entry: "index.html"`. Nó chạy được, và nó **phá
//! luật trung tâm của cả dự án**: ứng dụng ship HTML nghĩa là ngày có bộ dựng
//! GPU riêng, mọi ứng dụng phải viết lại — và lúc đó không ai dám bỏ WebView
//! nữa. Giàn giáo hoá thành nhà.
//!
//! Nên điểm vào là cây component KHAI BÁO. Ứng dụng nói *có gì trên màn hình*;
//! bộ dựng quyết định *vẽ ra sao*. Đổi bộ dựng thì ứng dụng không sửa một dòng.
//!
//! # ⚠️ CẠM BẪY: giải mã KHÔNG được đi vòng qua hàm dựng
//!
//! Gắn thẳng `#[derive(Deserialize)]` lên [`crate::Node`] là xong về mặt biên
//! dịch — và **thủng toàn bộ tầng kiểm tra**. `Node` có mọi trường riêng tư
//! chính là để mỗi nút chỉ ra đời qua một hàm dựng có kiểm; giải mã trực tiếp
//! sẽ nhồi thẳng vào trường, bỏ qua sạch: trần độ sâu, trần số nút, phép lọc ký
//! tự giả mạo, ràng buộc mã hành động, cấm ảnh trỏ ra mạng.
//!
//! Kẻ gian không cần tấn công gì cả — chỉ cần ship một tệp JSON.
//!
//! Nên ở đây là hai kiểu RIÊNG BIỆT: [`UiNode`] chỉ là dữ liệu trần để giải mã,
//! rồi `TryFrom` dựng lại `Node` **qua đúng những hàm dựng đó**. Một cây từ đĩa
//! đi qua y hệt các phép kiểm như một cây viết tay trong mã Rust.
//!
//! # Dạng trên dây dùng TỪ TIẾNG ANH
//!
//! Chú thích trong mã viết tiếng Việt, nhưng tên trường thì không: đây là tiêu
//! chuẩn cho người ngoài triển khai, và nó phải khớp với bản kê khai
//! (`spec_version`, `capabilities`, `entry`) vốn đã dùng tiếng Anh.
//!
//! # Ví dụ
//!
//! ```json
//! {
//!   "kind": "group",
//!   "flow": "column",
//!   "gap": "medium",
//!   "children": [
//!     { "kind": "text", "content": "Xin chào", "emphasis": "title" },
//!     { "kind": "button", "label": "Gửi", "action": "gui", "tone": "primary" }
//!   ]
//! }
//! ```

use serde::{Deserialize, Serialize};

use crate::{Alt, Emphasis, Flow, Gap, Node, Tone, UiError};

/// Trần kích thước tệp giao diện, tính bằng byte.
///
/// Trần cây (`MAX_NODES`, `MAX_DEPTH`) chỉ chặn được SAU khi đã giải mã xong —
/// mà giải mã một tệp JSON 500 MB thì đã ngốn hết bộ nhớ trước đó rồi. Chặn
/// theo byte là chặn ở bước sớm nhất có thể.
pub const MAX_UI_BYTES: usize = 1024 * 1024;

/// Một nút giao diện ở dạng trên dây.
///
/// Kiểu này CỐ Ý là dữ liệu trần, không có bất biến nào. Mọi phép kiểm nằm ở
/// [`TryFrom<UiNode> for Node`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum UiNode {
    Text {
        content: String,
        #[serde(default)]
        emphasis: Emphasis,
    },
    Button {
        label: String,
        action: String,
        #[serde(default)]
        tone: Tone,
    },
    Field {
        label: String,
        #[serde(default)]
        value: String,
        #[serde(default)]
        secret: bool,
    },
    Image {
        source: String,
        /// ⚠️ KHÔNG có `#[serde(default)]`. Thiếu trường này là LỖI, không phải
        /// mặc định thành trang trí — "quên viết mô tả ảnh" phải bị chặn chứ
        /// không được lặng lẽ biến thành "ảnh này không cần mô tả".
        alt: UiAlt,
    },
    Toggle {
        label: String,
        /// Mặc định TẮT. Không phải chuyện tiện tay: một công tắc quyền mặc định
        /// bật là câu hỏi tự trả lời hộ người dùng.
        #[serde(default)]
        on: bool,
        action: String,
    },
    Group {
        #[serde(default)]
        flow: Flow,
        #[serde(default)]
        gap: Gap,
        #[serde(default)]
        children: Vec<UiNode>,
    },
}

/// Mô tả ảnh ở dạng trên dây.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum UiAlt {
    Text {
        text: String,
    },
    /// Phải khai ra miệng. Không có đường mặc định nào dẫn tới đây.
    Decorative,
}

impl From<UiAlt> for Alt {
    fn from(a: UiAlt) -> Self {
        match a {
            UiAlt::Text { text } => Self::Text(text),
            UiAlt::Decorative => Self::Decorative,
        }
    }
}

impl TryFrom<UiNode> for Node {
    type Error = UiError;

    /// Dựng lại cây **qua đúng các hàm dựng có kiểm tra**.
    ///
    /// Không có đường tắt nào ở đây. `Node::text`, `Node::button`, `Node::image`
    /// và `Node::child` chạy y hệt như khi cây được viết tay trong mã Rust, nên
    /// một cây đến từ đĩa không hưởng bất kỳ ưu ái nào.
    fn try_from(u: UiNode) -> Result<Self, Self::Error> {
        match u {
            UiNode::Text { content, emphasis } => Self::text_with(content, emphasis),
            UiNode::Button {
                label,
                action,
                tone,
            } => Self::button(label, &action, tone),
            UiNode::Field {
                label,
                value,
                secret,
            } => Self::field(label, value, secret),
            UiNode::Toggle { label, on, action } => Self::toggle(label, on, &action),
            UiNode::Image { source, alt } => Self::image(source, alt.into()),
            UiNode::Group {
                flow,
                gap,
                children,
            } => {
                let mut g = Self::group(flow, gap);
                for c in children {
                    // `child` kiểm trần độ sâu và trần số nút ở TỪNG lần thêm,
                    // nên một cây khổng lồ bị chặn giữa chừng chứ không phải sau
                    // khi đã dựng xong toàn bộ.
                    g = g.child(Self::try_from(c)?)?;
                }
                Ok(g)
            }
        }
    }
}

/// Đọc một cây giao diện từ byte trong gói.
///
/// # Errors
/// Quá trần kích thước, JSON hỏng, hoặc cây vi phạm một phép kiểm nào đó.
pub fn decode(byte: &[u8]) -> Result<Node, DecodeError> {
    if byte.len() > MAX_UI_BYTES {
        return Err(DecodeError::TooLarge(byte.len()));
    }
    let tho: UiNode = serde_json::from_slice(byte).map_err(|e| DecodeError::Json(e.to_string()))?;
    Node::try_from(tho).map_err(|e| DecodeError::Tree(e.to_string()))
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("tệp giao diện {0} byte, vượt trần {MAX_UI_BYTES}")]
    TooLarge(usize),

    #[error("tệp giao diện không phải JSON hợp lệ: {0}")]
    Json(String),

    #[error("cây giao diện không hợp lệ: {0}")]
    Tree(String),
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay, không nuốt lỗi"
)]
mod kiem_thu {
    use super::*;
    use crate::{MAX_DEPTH, MAX_NODES, NodeKind, Role};

    const MAU: &str = r#"{
      "kind": "group", "flow": "column", "gap": "medium",
      "children": [
        { "kind": "text", "content": "Ví TCC", "emphasis": "title" },
        { "kind": "field", "label": "Mật khẩu", "secret": true },
        { "kind": "image", "source": "anh/logo.png",
          "alt": { "kind": "text", "text": "Biểu trưng" } },
        { "kind": "button", "label": "Gửi tiền", "action": "gui-tien", "tone": "danger" }
      ]
    }"#;

    #[test]
    fn doc_duoc_cay_binh_thuong() {
        let n = decode(MAU.as_bytes()).unwrap();
        assert_eq!(n.children().len(), 4);
        assert!(matches!(n.kind(), NodeKind::Group { .. }));
        // Trợ năng vẫn dựng được như cây viết tay.
        let a = n.accessibility_tree();
        assert!(
            a.children
                .iter()
                .any(|c| c.role == Role::Button { destructive: true })
        );
    }

    /// Công tắc thiếu `on` phải mặc định TẮT.
    ///
    /// Mặc định bật là câu hỏi tự trả lời hộ người dùng — và ở hộp thoại hỏi
    /// quyền thì nó tự trả lời "đồng ý".
    #[test]
    fn cong_tac_thieu_trang_thai_thi_mac_dinh_tat() {
        let n = decode(r#"{"kind":"toggle","label":"Quyền mạng","action":"q-mang"}"#.as_bytes())
            .unwrap();
        assert_eq!(
            n.accessibility_tree().role,
            crate::Role::Switch { on: false },
            "công tắc thiếu trạng thái lại mặc định BẬT"
        );
    }

    #[test]
    fn qua_json_roi_ve_thi_khong_doi() {
        let tho: UiNode = serde_json::from_str(MAU).unwrap();
        let lai: UiNode = serde_json::from_str(&serde_json::to_string(&tho).unwrap()).unwrap();
        assert_eq!(tho, lai);
    }

    // ---- ⚠️ Giải mã KHÔNG được đi vòng qua hàm dựng ----
    //
    // Bốn phép thử dưới đây là lý do tồn tại của cả tệp này. Mỗi cái tương ứng
    // một phép kiểm mà `#[derive(Deserialize)]` thẳng lên `Node` sẽ bỏ qua.

    /// Ký tự đảo chiều chữ dựng LÚC CHẠY, không viết thẳng vào mã.
    ///
    /// Không phải để cho đẹp: `rustc` TỪ CHỐI biên dịch tệp nguồn có ký tự đảo
    /// chiều chữ (phòng thủ thêm sau vụ "Trojan Source"). Tức là trình biên dịch
    /// đang cưỡng chế đúng cái luật mà `check_display_text` cưỡng chế lúc chạy —
    /// một sự trùng khớp đáng để ghi lại.
    #[test]
    fn nhan_nut_giat_gan_tu_json_van_bi_chan() {
        let ac = format!(
            r#"{{"kind":"button","label":"Hu{}r","action":"x"}}"#,
            '\u{202e}'
        );
        assert!(
            matches!(decode(ac.as_bytes()), Err(DecodeError::Tree(_))),
            "ký tự đảo chiều chữ lọt qua đường JSON"
        );
    }

    #[test]
    fn anh_tro_ra_mang_tu_json_van_bi_chan() {
        let ac = r#"{"kind":"image","source":"https://theo-doi.example/1.png",
                     "alt":{"kind":"decorative"}}"#;
        assert!(matches!(decode(ac.as_bytes()), Err(DecodeError::Tree(_))));
    }

    #[test]
    fn ma_hanh_dong_bay_tu_json_van_bi_chan() {
        let ac = r#"{"kind":"button","label":"A","action":"CHO PHEP TAT CA"}"#;
        assert!(matches!(decode(ac.as_bytes()), Err(DecodeError::Tree(_))));
    }

    #[test]
    fn cay_qua_nhieu_nut_tu_json_van_bi_chan() {
        let con = r#"{"kind":"text","content":"x"},"#.repeat(MAX_NODES + 10);
        let ac = format!(
            r#"{{"kind":"group","children":[{}]}}"#,
            con.trim_end_matches(',')
        );
        assert!(
            matches!(decode(ac.as_bytes()), Err(DecodeError::Tree(_))),
            "cây vượt trần số nút lọt qua đường JSON"
        );
    }

    /// Lồng sâu bị chặn — nhưng có thể bị chặn ở **hai chỗ khác nhau**, và cả
    /// hai đều chấp nhận được: `serde_json` có trần đệ quy riêng, chạm trước thì
    /// ra lỗi JSON, không chạm thì `child` chặn.
    #[test]
    fn cay_qua_sau_tu_json_van_bi_chan() {
        let n = MAX_DEPTH + 10;
        let ac = format!(
            "{}{}{}",
            r#"{"kind":"group","children":["#.repeat(n),
            r#"{"kind":"text","content":"đáy"}"#,
            "]}".repeat(n)
        );
        assert!(
            decode(ac.as_bytes()).is_err(),
            "cây lồng {n} tầng lọt qua đường JSON"
        );
    }

    // ---- Ràng buộc của chính dạng trên dây ----

    /// ⚠️ Thiếu mô tả ảnh phải là LỖI, không được mặc định thành trang trí.
    ///
    /// Mặc định lặng lẽ ở đây nghĩa là mọi ảnh người ta quên mô tả sẽ biến mất
    /// khỏi trình đọc màn hình mà không ai biết.
    #[test]
    fn anh_thieu_mo_ta_thi_bao_loi_chu_khong_mac_dinh() {
        let thieu = r#"{"kind":"image","source":"a.png"}"#;
        assert!(
            matches!(decode(thieu.as_bytes()), Err(DecodeError::Json(_))),
            "ảnh thiếu mô tả bị nuốt thành trang trí"
        );
    }

    /// Trường lạ phải bị từ chối: nó gần như luôn là gõ sai tên, và im lặng bỏ
    /// qua nghĩa là người viết tưởng đã đặt được một thuộc tính mà thật ra không.
    #[test]
    fn truong_la_thi_tu_choi() {
        let la = r#"{"kind":"text","content":"x","mau":"do"}"#;
        assert!(matches!(decode(la.as_bytes()), Err(DecodeError::Json(_))));
    }

    #[test]
    fn tep_qua_lon_bi_chan_truoc_khi_giai_ma() {
        let to = vec![b' '; MAX_UI_BYTES + 1];
        assert!(matches!(decode(&to), Err(DecodeError::TooLarge(_))));
    }

    #[test]
    fn thieu_truong_bat_buoc_thi_bao_ro() {
        for x in [
            r#"{"kind":"button","label":"A"}"#, // thiếu action
            r#"{"kind":"text"}"#,               // thiếu content
            r#"{"kind":"khong-co-loai-nay"}"#,  // loại lạ
        ] {
            assert!(
                matches!(decode(x.as_bytes()), Err(DecodeError::Json(_))),
                "{x}"
            );
        }
    }
}
