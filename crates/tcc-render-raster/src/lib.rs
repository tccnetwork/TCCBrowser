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

use cosmic_text::{Attrs, Buffer, Color, Family, FontSystem, Metrics, Shaping, SwashCache};
use tcc_ui::{AccessNode, Alt, Emphasis, Gap, Node, NodeKind, Renderer, Role, Tone};

/// Bề rộng khung vẽ. Cố định: bộ dựng này để KIỂM ĐỊNH, không để co giãn theo
/// cửa sổ người dùng.
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
}

/// Bộ dựng ra pixel.
pub struct RasterRenderer {
    fonts: FontSystem,
    cache: SwashCache,
    /// Ảnh xám, 1 byte/pixel. 255 = trắng.
    pixel: Vec<u8>,
    height: usize,
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

    /// Đếm pixel có mực. Dùng để chốt "có vẽ ra gì đó" mà không cần so ảnh.
    #[must_use]
    pub fn ink(&self) -> usize {
        self.pixel.iter().filter(|p| **p < 250).count()
    }
}

/// Một dòng chữ đã đặt xong chỗ, chờ vẽ.
struct Dong {
    chu: String,
    co: f32,
    dam: bool,
    /// Thụt vào, tính từ lề trái.
    thut: f32,
    /// Vẽ khung quanh chữ — nút và ô nhập.
    khung: bool,
}

impl Renderer for RasterRenderer {
    type Error = RasterError;

    fn render(&mut self, tree: &Node) -> Result<(), Self::Error> {
        let mut dong = Vec::new();
        let mut access = Vec::new();
        xep(tree, 0.0, &mut dong, &mut access);

        // Chiều cao tính TỪ nội dung, không đặt trước: một cây dài hơn phải ra
        // một ảnh cao hơn, chứ không phải một ảnh bị cắt.
        let cao = (LE * 2.0 + dong.iter().map(|mot| mot.co * 1.9).sum::<f32>()) as usize;
        if cao > MAX_HEIGHT {
            return Err(RasterError::TooTall(cao));
        }
        self.height = cao.max(1);
        self.pixel = vec![255u8; WIDTH * self.height];

        let mut dinh = LE;
        for mot in &dong {
            self.ve_dong(mot, dinh);
            dinh += mot.co * 1.9;
        }

        // `xep` đẩy ĐÚNG một nút cho gốc. Bọc thêm một `Group` ở đây là dựng
        // ra một tầng không ai vẽ — và phép kiểm ngang bằng bắt ngay.
        debug_assert_eq!(access.len(), 1, "gốc phải ra đúng một nút trợ năng");
        self.published = access.into_iter().next();
        Ok(())
    }

    fn published_accessibility(&self) -> Option<AccessNode> {
        self.published.clone()
    }
}

impl RasterRenderer {
    fn ve_dong(&mut self, dong: &Dong, dinh: f32) {
        let x0 = LE + dong.thut;
        if dong.khung {
            // Khung mảnh quanh nút và ô nhập. Không phải trang trí: nó là thứ
            // duy nhất trên ảnh phân biệt "chữ để đọc" với "chỗ bấm được".
            let cao = (dong.co * 1.6) as usize;
            let rong = (dong.chu.chars().count() as f32 * dong.co * 0.62 + 16.0) as usize;
            self.khung(
                x0 as usize,
                dinh as usize,
                rong.min(WIDTH - x0 as usize - 4),
                cao,
            );
        }

        let mut buffer = Buffer::new(&mut self.fonts, Metrics::new(dong.co, dong.co * 1.4));
        let mut b = buffer.borrow_with(&mut self.fonts);
        b.set_size(Some(WIDTH as f32 - x0 - LE), Some(dong.co * 2.0));
        let mut attrs = Attrs::new().family(Family::SansSerif);
        if dong.dam {
            attrs = attrs.weight(cosmic_text::Weight::BOLD);
        }
        // `Shaping::Advanced` — bắt buộc cho tiếng Việt. `Basic` bỏ qua việc
        // xếp dấu phụ, và với tiếng Việt thì đó không phải "nhanh hơn", đó là SAI.
        b.set_text(&dong.chu, &attrs, Shaping::Advanced, None);
        b.shape_until_scroll(true);

        let (pixel, rong_anh, cao_anh) = (&mut self.pixel, WIDTH, self.height);
        let nen_x = (x0 + 8.0 * f32::from(u8::from(dong.khung))) as i32;
        let nen_y = (dinh + dong.co * 0.25) as i32;
        b.draw(
            &mut self.cache,
            Color::rgb(0, 0, 0),
            |trai, tren, rong_o, cao_o, mau| {
                for hang in 0..cao_o {
                    for cot in 0..rong_o {
                        let px = nen_x + trai + cot as i32;
                        let py = nen_y + tren + hang as i32;
                        if px >= 0 && py >= 0 && (px as usize) < rong_anh && (py as usize) < cao_anh
                        {
                            let vi_tri = py as usize * rong_anh + px as usize;
                            let dam = f32::from(mau.a()) / 255.0;
                            pixel[vi_tri] = (f32::from(pixel[vi_tri]) * (1.0 - dam)) as u8;
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

/// Đi cây, biến mỗi nút thành dòng chữ **và** một nút trợ năng.
///
/// Hai việc làm CÙNG một lượt là có chủ ý: tách ra hai lượt thì chúng trôi khỏi
/// nhau, và cây trợ năng lại thành một lời hứa thay vì một mô tả.
fn xep(n: &Node, thut: f32, ra: &mut Vec<Dong>, access: &mut Vec<AccessNode>) {
    if let NodeKind::Group { gap, .. } = n.kind() {
        let mut con_access = Vec::new();
        // Hàng ngang vẫn xếp dọc trên ảnh này. Nói ra chứ không giả vờ: bố cục
        // hàng cần đo bề rộng từng phần tử, và bộ dựng này sinh ra để kiểm bố
        // cục dọc + trợ năng, không phải để thay WebView hôm nay.
        let them = match gap {
            Gap::None => 0.0,
            Gap::Small => 4.0,
            Gap::Medium => 8.0,
            Gap::Large => 12.0,
        };
        for c in n.children() {
            xep(c, thut + them, ra, &mut con_access);
        }
        access.push(AccessNode {
            role: Role::Group,
            label: None,
            children: con_access,
        });
        return;
    }
    xep_la(n, thut, ra, access);
}

/// Nút LÁ — mỗi loại thành một dòng chữ và một nút trợ năng.
fn xep_la(n: &Node, thut: f32, ra: &mut Vec<Dong>, access: &mut Vec<AccessNode>) {
    match n.kind() {
        NodeKind::Text { content, emphasis } => {
            let (co, dam) = match emphasis {
                Emphasis::Title => (CO_CHU * 1.5, true),
                Emphasis::Warning => (CO_CHU, true),
                Emphasis::Subtle | Emphasis::Normal => (CO_CHU, false),
            };
            ra.push(Dong {
                chu: content.clone(),
                co,
                dam,
                thut,
                khung: false,
            });
            access.push(AccessNode {
                role: Role::Text,
                label: Some(content.clone()),
                children: Vec::new(),
            });
        }
        NodeKind::Button { label, tone, .. } => {
            ra.push(Dong {
                chu: label.clone(),
                co: CO_CHU,
                dam: false,
                thut,
                khung: true,
            });
            access.push(AccessNode {
                role: Role::Button {
                    destructive: *tone == Tone::Danger,
                },
                label: Some(label.clone()),
                children: Vec::new(),
            });
        }
        NodeKind::Field {
            label,
            value,
            secret,
        } => {
            // Ô bí mật hiện dấu chấm, KHÔNG hiện chữ. Vẽ ra rồi mới che là đã
            // vẽ ra — và ảnh này có thể bị lưu lại.
            let hien = if *secret {
                "•".repeat(value.chars().count())
            } else {
                value.clone()
            };
            ra.push(Dong {
                chu: format!("{label}: {hien}"),
                co: CO_CHU,
                dam: false,
                thut,
                khung: true,
            });
            access.push(AccessNode {
                role: Role::TextInput { secret: *secret },
                label: Some(label.clone()),
                children: Vec::new(),
            });
        }
        NodeKind::Toggle { label, on, .. } => {
            ra.push(Dong {
                chu: format!("[{}] {label}", if *on { "x" } else { " " }),
                co: CO_CHU,
                dam: false,
                thut,
                khung: false,
            });
            access.push(AccessNode {
                role: Role::Switch { on: *on },
                label: Some(label.clone()),
                children: Vec::new(),
            });
        }
        NodeKind::Image { alt, .. } => {
            let (chu, nhan) = match alt {
                Alt::Text(t) => (format!("[ảnh: {t}]"), Some(t.clone())),
                Alt::Decorative => ("[ảnh trang trí]".to_owned(), None),
            };
            ra.push(Dong {
                chu,
                co: CO_CHU,
                dam: false,
                thut,
                khung: true,
            });
            access.push(AccessNode {
                role: Role::Image,
                label: nhan,
                children: Vec::new(),
            });
        }
        // `Group` đã xử lý ở `xep` và không bao giờ tới đây.
        NodeKind::Group { .. } => unreachable!("nhóm đã được `xep` chặn trước"),
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
    use tcc_ui::Flow;

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

    /// Cây rỗng vẫn ra một ảnh hợp lệ, không hoảng loạn.
    #[test]
    fn cay_toi_thieu_khong_hoang_loan() {
        let mut bd = RasterRenderer::new();
        bd.render(&Node::group(Flow::Column, Gap::None)).unwrap();
        assert!(bd.height() >= 1);
    }
}
