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
use tcc_ui::{AccessNode, Alt, Emphasis, Flow, Gap, Node, NodeKind, Renderer, Role, Tone};

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

/// Một ô chữ **đã đo xong**: biết mình rộng bao nhiêu, cao bao nhiêu.
///
/// Đo trước khi đặt là cả điểm của 4.2. Bản 4.1 không đo gì — nó xếp mọi thứ
/// thành một cột, nên `Flow::Row` chỉ là `Flow::Column` đội tên khác.
#[derive(Clone)]
struct O {
    chu: String,
    co: f32,
    dam: bool,
    /// Khung quanh chữ — nút, ô nhập, ảnh.
    khung: bool,
    rong: f32,
    cao: f32,
}

/// Chỗ để đặt: góc trên trái và bề rộng được phép dùng.
///
/// Gom bốn tham số rời thành một kiểu là để **không hoán vị nhầm** — `trai` và
/// `tren` cùng là `f32`, và đổi chỗ chúng thì mã vẫn biên dịch, vẫn chạy, chỉ
/// vẽ sai. Đã suýt vấp đúng thế khi tách `dat_hang`.
#[derive(Clone, Copy)]
struct Cho {
    trai: f32,
    tren: f32,
    rong: f32,
    khe: f32,
}

/// Một ô **đã đặt xong chỗ**, toạ độ tuyệt đối.
struct DaDat {
    o: O,
    trai: f32,
    tren: f32,
}

/// Đệm trong khung, mỗi bên.
const DEM: f32 = 8.0;

impl Renderer for RasterRenderer {
    type Error = RasterError;

    fn render(&mut self, tree: &Node) -> Result<(), Self::Error> {
        let mut access = Vec::new();
        let mut dat = Vec::new();
        let rong_dung = WIDTH as f32 - LE * 2.0;

        // Ba lượt tách bạch: đo → đặt → vẽ. Gộp lượt đo vào lượt vẽ là cách bản
        // 4.1 hỏng — không có kích thước thì không đặt cạnh nhau được gì.
        let cao = self.dat(tree, LE, LE, rong_dung, &mut dat, &mut access);

        let cao_anh = (cao + LE) as usize;
        if cao_anh > MAX_HEIGHT {
            return Err(RasterError::TooTall(cao_anh));
        }
        self.height = cao_anh.max(1);
        self.pixel = vec![255u8; WIDTH * self.height];

        for mot in &dat {
            self.ve_o(mot);
        }

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
    fn do_o(&mut self, chu: &str, co: f32, dam: bool, khung: bool, rong_toi_da: f32) -> O {
        let dem = if khung { DEM * 2.0 } else { 0.0 };
        let cho_chu = (rong_toi_da - dem).max(co);

        let mut buffer = Buffer::new(&mut self.fonts, Metrics::new(co, co * 1.4));
        let mut b = buffer.borrow_with(&mut self.fonts);
        b.set_size(Some(cho_chu), None);
        let mut attrs = Attrs::new().family(Family::SansSerif);
        if dam {
            attrs = attrs.weight(cosmic_text::Weight::BOLD);
        }
        // `Shaping::Advanced` — bắt buộc cho tiếng Việt. `Basic` bỏ qua việc
        // xếp dấu phụ, và với tiếng Việt thì đó không phải "nhanh hơn", đó là SAI.
        b.set_text(chu, &attrs, Shaping::Advanced, None);
        b.shape_until_scroll(false);

        let mut rong_chu: f32 = 0.0;
        let mut so_dong = 0usize;
        for run in b.layout_runs() {
            rong_chu = rong_chu.max(run.line_w);
            so_dong += 1;
        }
        let so_dong = so_dong.max(1);

        O {
            chu: chu.to_owned(),
            co,
            dam,
            khung,
            rong: rong_chu + dem,
            cao: co * 1.4 * so_dong as f32 + if khung { DEM } else { 0.0 },
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
                    children: Vec::new(),
                });
                self.do_o(content, co, dam, false, rong_toi_da)
            }
            NodeKind::Button { label, tone, .. } => {
                access.push(AccessNode {
                    role: Role::Button {
                        destructive: *tone == Tone::Danger,
                    },
                    label: Some(label.clone()),
                    children: Vec::new(),
                });
                self.do_o(label, CO_CHU, false, true, rong_toi_da)
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
                    children: Vec::new(),
                });
                self.do_o(
                    &format!("{label}: {hien}"),
                    CO_CHU,
                    false,
                    true,
                    rong_toi_da,
                )
            }
            NodeKind::Toggle { label, on, .. } => {
                access.push(AccessNode {
                    role: Role::Switch { on: *on },
                    label: Some(label.clone()),
                    children: Vec::new(),
                });
                let chu = format!("[{}] {label}", if *on { "x" } else { " " });
                self.do_o(&chu, CO_CHU, false, false, rong_toi_da)
            }
            NodeKind::Image { alt, .. } => {
                let (chu, nhan) = match alt {
                    Alt::Text(t) => (format!("[ảnh: {t}]"), Some(t.clone())),
                    Alt::Decorative => ("[ảnh trang trí]".to_owned(), None),
                };
                access.push(AccessNode {
                    role: Role::Image,
                    label: nhan,
                    children: Vec::new(),
                });
                self.do_o(&chu, CO_CHU, false, true, rong_toi_da)
            }
            NodeKind::Group { .. } => unreachable!("nhóm đã được `dat` chặn trước"),
        }
    }

    /// Đặt một nút vào chỗ, trả về **chiều cao đã dùng**.
    fn dat(
        &mut self,
        n: &Node,
        trai: f32,
        tren: f32,
        rong_toi_da: f32,
        ra: &mut Vec<DaDat>,
        access: &mut Vec<AccessNode>,
    ) -> f32 {
        let NodeKind::Group { flow, gap, .. } = n.kind() else {
            let o = self.do_la(n, rong_toi_da, access);
            let cao = o.cao;
            ra.push(DaDat { o, trai, tren });
            return cao;
        };

        let khe = match gap {
            Gap::None => 0.0,
            Gap::Small => 4.0,
            Gap::Medium => 8.0,
            Gap::Large => 12.0,
        };
        let cho = Cho {
            trai,
            tren,
            rong: rong_toi_da,
            khe,
        };
        let mut con_access = Vec::new();
        let cao = match flow {
            Flow::Column => self.dat_cot(n, cho, ra, &mut con_access),
            Flow::Row => self.dat_hang(n, cho, ra, &mut con_access),
        };
        access.push(AccessNode {
            role: Role::Group,
            label: None,
            children: con_access,
        });
        cao
    }

    fn dat_cot(
        &mut self,
        n: &Node,
        cho: Cho,
        ra: &mut Vec<DaDat>,
        access: &mut Vec<AccessNode>,
    ) -> f32 {
        let mut y = cho.tren;
        for c in n.children() {
            y += self.dat(c, cho.trai, y, cho.rong, ra, access) + cho.khe;
        }
        (y - cho.tren - cho.khe).max(0.0)
    }

    /// **Bố cục hàng thật** — đặt cạnh nhau, và XUỐNG DÒNG khi hết chỗ.
    ///
    /// Bản 4.1 xếp hàng ngang theo chiều dọc rồi ghi chú "chưa làm". Đây là chỗ
    /// nó được làm: đo từng phần tử trước, rồi mới đặt.
    ///
    /// Xuống dòng chứ không tràn ra ngoài: một nút bị đẩy khỏi mép là một nút
    /// người dùng **không bấm được và không biết là có**.
    fn dat_hang(
        &mut self,
        n: &Node,
        cho: Cho,
        ra: &mut Vec<DaDat>,
        access: &mut Vec<AccessNode>,
    ) -> f32 {
        let (trai, tren, rong_toi_da, khe) = (cho.trai, cho.tren, cho.rong, cho.khe);
        let mut x = trai;
        let mut y = tren;
        let mut cao_hang: f32 = 0.0;
        for c in n.children() {
            // Nhóm lồng trong hàng: đặt nó như một khối chiếm hết phần còn lại.
            if matches!(c.kind(), NodeKind::Group { .. }) {
                let cao = self.dat(c, x, y, rong_toi_da - (x - trai), ra, access);
                y += cao + khe;
                cao_hang = 0.0;
                x = trai;
                continue;
            }
            let o = self.do_la(c, rong_toi_da, access);
            if x > trai && x - trai + o.rong > rong_toi_da {
                x = trai;
                y += cao_hang + khe;
                cao_hang = 0.0;
            }
            cao_hang = cao_hang.max(o.cao);
            let rong = o.rong;
            ra.push(DaDat {
                o,
                trai: x,
                tren: y,
            });
            x += rong + khe;
        }
        (y - tren + cao_hang).max(0.0)
    }

    fn ve_o(&mut self, dat: &DaDat) {
        let o = &dat.o;
        if o.khung {
            self.khung(
                dat.trai as usize,
                dat.tren as usize,
                (o.rong as usize).min(WIDTH.saturating_sub(dat.trai as usize + 2)),
                o.cao as usize,
            );
        }

        let dem = if o.khung { DEM } else { 0.0 };
        let mut buffer = Buffer::new(&mut self.fonts, Metrics::new(o.co, o.co * 1.4));
        let mut b = buffer.borrow_with(&mut self.fonts);
        b.set_size(Some(o.rong - dem * 2.0), None);
        let mut attrs = Attrs::new().family(Family::SansSerif);
        if o.dam {
            attrs = attrs.weight(cosmic_text::Weight::BOLD);
        }
        b.set_text(&o.chu, &attrs, Shaping::Advanced, None);
        b.shape_until_scroll(false);

        let (pixel, rong_anh, cao_anh) = (&mut self.pixel, WIDTH, self.height);
        let nen_x = (dat.trai + dem) as i32;
        let nen_y = (dat.tren + dem * 0.5) as i32;
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
