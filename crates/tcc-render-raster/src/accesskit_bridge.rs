//! Cầu nối tới **AccessKit** — đưa cây trợ năng ra hệ điều hành thật.
//!
//! Sau cờ `accesskit`. Đây là mục 4.4 của kế hoạch.
//!
//! # Vì sao đây là chỗ dễ mất thông tin nhất
//!
//! `tcc-ui` mang hai thứ mà AccessKit **không có vai trò riêng** để nói:
//!
//! | Ta có | AccessKit có |
//! |---|---|
//! | `Button { destructive: true }` | chỉ `Role::Button` |
//! | `TextInput { secret: true }` | có `Role::PasswordInput` ✅ |
//! | `Switch { on }` | `Role::Switch` + `toggled` ✅ |
//!
//! Ô mật khẩu và công tắc dịch được thẳng. **Nút không hoàn tác thì không** —
//! và nếu bỏ qua, người dùng trình đọc màn hình nghe *"Xoá dữ liệu, nút"* y hệt
//! *"Huỷ, nút"*. Cùng một câu cho một việc xoá sạch và một việc không làm gì.
//!
//! Nên nó đi vào `description`, câu mà `SECURITY.md` đã chốt và `text.rs` đã
//! dịch sẵn: cùng chuỗi người dùng WebView nghe, không phải một chuỗi mới bịa ra
//! ở đây. Hai bộ dựng nói hai câu khác nhau cho cùng một nút là đúng thứ phép
//! kiểm chéo sinh ra để chặn.
//!
//! # `Group` mang nhãn `None` là CÓ CHỦ Ý
//!
//! Nhóm và ảnh trang trí là hai thứ trình đọc màn hình phải **đi xuyên qua**.
//! Gán nhãn cho chúng là bắt người dùng nghe thêm một câu vô nghĩa ở mỗi tầng.

use accesskit::{Node as AkNode, NodeId, Role as AkRole, Toggled, Tree, TreeId, TreeUpdate};
use tcc_ui::{AccessNode, Role};

/// Chuỗi bộ dựng cần. **Tiêm từ ngoài vào**, y như `tcc-render-webview`.
///
/// Bản đầu tôi viết cứng một câu tiếng Việt ngay trong tệp này. Sai hai lần:
/// giao diện mặc định TIẾNG ANH, và WebView đã nhận câu ấy làm tham số do
/// `tcc-shell` dịch. Hai bộ dựng đọc hai câu khác nhau cho cùng một nút là đúng
/// thứ phép kiểm chéo sinh ra để chặn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessText {
    /// Câu mô tả hành động mất mát.
    pub cau_mat_mat: String,
}

impl Default for AccessText {
    /// Mặc định TIẾNG ANH — đúng mặc định của cả giao diện, và đúng mặc định
    /// `RendererText` của bộ dựng WebView.
    fn default() -> Self {
        Self {
            cau_mat_mat: "This cannot be undone.".to_owned(),
        }
    }
}

/// Đổi cây trợ năng của ta thành một `TreeUpdate` của AccessKit.
///
/// Trả về cả cây, sẵn sàng đẩy cho adapter của nền tảng.
#[must_use]
pub fn to_accesskit(goc: &AccessNode, chu: &AccessText) -> TreeUpdate {
    let mut nodes = Vec::new();
    let mut dem = 0u64;
    let id_goc = them(goc, chu, &mut nodes, &mut dem);
    TreeUpdate {
        nodes,
        // `TreeId::ROOT` — cây gốc của cửa sổ. Ta chỉ có một cây; ngày có
        // nhiều thì mỗi cây một id, và chỗ này phải đổi.
        tree_id: TreeId::ROOT,
        tree: Some(Tree::new(id_goc)),
        focus: id_goc,
    }
}

fn them(n: &AccessNode, chu: &AccessText, ra: &mut Vec<(NodeId, AkNode)>, dem: &mut u64) -> NodeId {
    let id = NodeId(*dem);
    *dem += 1;

    let mut nut = AkNode::new(match n.role {
        Role::Text => AkRole::Label,
        Role::Button { .. } => AkRole::Button,
        // AccessKit CÓ vai trò riêng cho ô mật khẩu. Dùng nó chứ không dùng
        // `TextInput` rồi tự che: hệ điều hành đọc `PasswordInput` khác hẳn —
        // nó không đọc to từng ký tự.
        Role::TextInput { secret: true } => AkRole::PasswordInput,
        Role::TextInput { secret: false } => AkRole::TextInput,
        Role::Switch { .. } => AkRole::Switch,
        Role::Image => AkRole::Image,
        Role::Group => AkRole::GenericContainer,
    });

    if let Some(nhan) = &n.label {
        nut.set_label(nhan.clone());
    }
    // Nhóm và ảnh trang trí KHÔNG có nhãn — trình đọc màn hình đi xuyên qua.

    if let Role::Switch { on } = n.role {
        nut.set_toggled(Toggled::from(on));
    }
    if let Role::Button { destructive: true } = n.role {
        // AccessKit không có vai trò cho "không hoàn tác". Không nói ra thì
        // "Xoá dữ liệu, nút" nghe y hệt "Huỷ, nút".
        nut.set_description(chu.cau_mat_mat.clone());
    }

    let con: Vec<NodeId> = n.children.iter().map(|c| them(c, chu, ra, dem)).collect();
    nut.set_children(con);

    ra.push((id, nut));
    id
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay"
)]
mod kiem_thu {
    use super::*;

    fn cay(role: Role, nhan: Option<&str>) -> AccessNode {
        AccessNode {
            role,
            label: nhan.map(str::to_owned),
            children: Vec::new(),
        }
    }

    fn mot(n: &AccessNode) -> AkNode {
        let u = to_accesskit(n, &AccessText::default());
        u.nodes.into_iter().next().unwrap().1
    }

    /// **Ô bí mật phải ra `PasswordInput`, không ra `TextInput`.**
    ///
    /// Khác biệt không phải hình thức: `TextInput` làm trình đọc màn hình đọc
    /// TO từng ký tự mật khẩu. Đây đúng là bất biến B32 đã ghi trong
    /// `SECURITY.md`, giờ áp cho nền tảng thứ hai.
    #[test]
    fn o_bi_mat_ra_vai_tro_mat_khau() {
        let n = mot(&cay(Role::TextInput { secret: true }, Some("Mã PIN")));
        assert_eq!(n.role(), AkRole::PasswordInput);

        let thuong = mot(&cay(Role::TextInput { secret: false }, Some("Tìm")));
        assert_eq!(thuong.role(), AkRole::TextInput);
    }

    /// **Nút không hoàn tác phải NÓI RA điều đó.**
    ///
    /// AccessKit không có vai trò riêng, nên nếu không đẩy vào `description`
    /// thì "Xoá dữ liệu, nút" nghe y hệt "Huỷ, nút".
    #[test]
    fn nut_khong_hoan_tac_noi_ra_dieu_do() {
        let nguy = mot(&cay(Role::Button { destructive: true }, Some("Xoá hết")));
        assert_eq!(nguy.role(), AkRole::Button);
        assert_eq!(
            nguy.description(),
            Some(AccessText::default().cau_mat_mat.as_str())
        );

        let thuong = mot(&cay(Role::Button { destructive: false }, Some("Huỷ")));
        assert_eq!(thuong.description(), None, "nút thường bị gán câu mất mát");
    }

    /// Công tắc phải mang trạng thái bật/tắt, không chỉ mang nhãn.
    #[test]
    fn cong_tac_mang_trang_thai() {
        assert_eq!(
            mot(&cay(Role::Switch { on: true }, Some("Mạng"))).toggled(),
            Some(Toggled::True)
        );
        assert_eq!(
            mot(&cay(Role::Switch { on: false }, Some("Mạng"))).toggled(),
            Some(Toggled::False)
        );
    }

    /// Nhóm KHÔNG có nhãn — trình đọc màn hình đi xuyên qua.
    #[test]
    fn nhom_khong_mang_nhan() {
        let n = mot(&cay(Role::Group, None));
        assert_eq!(n.role(), AkRole::GenericContainer);
        assert_eq!(n.label(), None);
    }

    /// Cây con phải sang đủ, không rụng tầng nào.
    #[test]
    fn cay_con_sang_du() {
        let goc = AccessNode {
            role: Role::Group,
            label: None,
            children: vec![
                cay(Role::Text, Some("một")),
                AccessNode {
                    role: Role::Group,
                    label: None,
                    children: vec![cay(Role::Text, Some("hai"))],
                },
            ],
        };
        let u = to_accesskit(&goc, &AccessText::default());
        assert_eq!(u.nodes.len(), 4, "rụng mất tầng: {:?}", u.nodes.len());
        let nhan: Vec<_> = u
            .nodes
            .iter()
            .filter_map(|(_, n)| n.label().map(std::string::ToString::to_string))
            .collect();
        assert_eq!(nhan, vec!["một", "hai"]);
    }

    /// Câu mất mát đi từ ngoài vào, không viết cứng trong crate này.
    #[test]
    fn cau_mat_mat_tiem_tu_ngoai() {
        let chu = AccessText {
            cau_mat_mat: "Hành động này không hoàn tác được.".to_owned(),
        };
        let u = to_accesskit(&cay(Role::Button { destructive: true }, Some("Xoá")), &chu);
        let n = &u.nodes.first().unwrap().1;
        assert_eq!(n.description(), Some(chu.cau_mat_mat.as_str()));
    }

    /// Mỗi nút một `NodeId` riêng — trùng id là AccessKit dựng sai cây.
    #[test]
    fn moi_nut_mot_id_rieng() {
        let goc = AccessNode {
            role: Role::Group,
            label: None,
            children: vec![cay(Role::Text, Some("a")), cay(Role::Text, Some("b"))],
        };
        let u = to_accesskit(&goc, &AccessText::default());
        let mut id: Vec<_> = u.nodes.iter().map(|(i, _)| i.0).collect();
        id.sort_unstable();
        id.dedup();
        assert_eq!(id.len(), u.nodes.len(), "có NodeId trùng nhau");
    }
}
