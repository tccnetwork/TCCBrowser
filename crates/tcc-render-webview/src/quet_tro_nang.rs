//! Đọc NGƯỢC đánh dấu đã sinh ra, dựng lại cây trợ năng.
//!
//! # Vì sao phải đọc ngược thay vì trả lại cây gốc
//!
//! `Renderer::published_accessibility` dễ cài đặt gian: cứ trả
//! `tree.accessibility_tree()` là phép kiểm định luôn đạt, kể cả khi bộ dựng vẽ
//! ra một màn hình hoàn toàn khác. Đó là con dấu cao su, không phải phép kiểm.
//!
//! Nên tệp này dựng lại cây trợ năng TỪ CHÍNH CHUỖI ĐÁNH DẤU sắp nạp vào
//! WebView. Quên `aria-label` ở một loại nút, đặt sai vai trò, nuốt mất một nút
//! con — phép kiểm định bắt được hết, vì hai cây đi bằng hai đường khác nhau.
//!
//! # Giới hạn đã biết
//!
//! Bộ quét này CHỈ đọc được đánh dấu do `danh_dau.rs` sinh ra: tập thẻ đóng,
//! luôn cân đối, và mọi chuỗi của ứng dụng đã thoát ký tự. Nó KHÔNG phải trình
//! phân tích tài liệu web đa dụng và đừng dùng cho việc đó. Cụ thể nó dựa vào
//! việc `>` trong dữ liệu người dùng đã thành `&gt;` — có phép thử chốt.

use tcc_ui::{AccessNode, Role};

use crate::danh_dau::DAU_MAT_MAT;

#[derive(Debug, PartialEq, Eq)]
pub enum QuetLoi {
    /// Thẻ mở mà không có thẻ đóng, hoặc ngược lại.
    TheLech(String),
    /// Thẻ không nằm trong tập thẻ tiêu chuẩn TCC.
    TheLa(String),
    /// Không có nút gốc, hoặc có nhiều hơn một.
    KhongDungMotGoc(usize),
    /// Nút bắt buộc có nhãn mà thiếu `aria-label`.
    ThieuNhan(String),
    /// ⚠️ Chữ hiện trên màn hình KHÁC chữ trình đọc màn hình đọc lên.
    /// Đây là đúng loại lừa dối mà cả tầng trợ năng sinh ra để chặn.
    NhanLechNoiDung { nhan: String, noi_dung: String },
    /// Đánh dấu cụt giữa chừng.
    Cut,
}

impl std::fmt::Display for QuetLoi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TheLech(t) => write!(f, "thẻ <{t}> không cân đối"),
            Self::TheLa(t) => write!(f, "thẻ <{t}> không thuộc tập thẻ tiêu chuẩn TCC"),
            Self::KhongDungMotGoc(n) => write!(f, "cần đúng một nút gốc, đếm được {n}"),
            Self::ThieuNhan(t) => write!(f, "<{t}> thiếu aria-label"),
            Self::NhanLechNoiDung { nhan, noi_dung } => write!(
                f,
                "màn hình hiện \"{noi_dung}\" nhưng trình đọc màn hình đọc \"{nhan}\""
            ),
            Self::Cut => write!(f, "đánh dấu cụt giữa chừng"),
        }
    }
}

impl std::error::Error for QuetLoi {}

/// Giải mã ngược các thực thể mà `danh_dau::thoat` đã tạo ra.
fn giai_ma(s: &str) -> String {
    const BANG: &[(&str, char)] = &[
        ("&amp;", '&'),
        ("&lt;", '<'),
        ("&gt;", '>'),
        ("&quot;", '"'),
        ("&#39;", '\''),
    ];
    let mut ra = String::with_capacity(s.len());
    let mut con_lai = s;
    while !con_lai.is_empty() {
        if let Some((t, c)) = BANG.iter().find(|(t, _)| con_lai.starts_with(t)) {
            ra.push(*c);
            con_lai = &con_lai[t.len()..];
            continue;
        }
        let Some(c) = con_lai.chars().next() else {
            break;
        };
        ra.push(c);
        con_lai = &con_lai[c.len_utf8()..];
    }
    ra
}

struct The {
    ten: String,
    dong: bool,
    thuoc_tinh: Vec<(String, String)>,
}

impl The {
    fn lay(&self, ten: &str) -> Option<&str> {
        self.thuoc_tinh
            .iter()
            .find(|(k, _)| k == ten)
            .map(|(_, v)| v.as_str())
    }
}

/// Tách phần trong `<…>` thành tên thẻ và danh sách thuộc tính.
fn doc_the(ben_trong: &str) -> The {
    let dong = ben_trong.starts_with('/');
    let than = ben_trong.trim_start_matches('/').trim_end_matches('/');
    let mut it = than.trim().splitn(2, char::is_whitespace);
    let ten = it.next().unwrap_or("").to_ascii_lowercase();
    let phan_con = it.next().unwrap_or("");

    let mut thuoc_tinh = Vec::new();
    let mut con_lai = phan_con;
    while let Some(vt_bang) = con_lai.find('=') {
        let ten_tt = con_lai[..vt_bang].trim().to_ascii_lowercase();
        let sau = &con_lai[vt_bang + 1..];
        // Giá trị luôn nằm trong nháy kép: `danh_dau` không sinh dạng nào khác.
        let Some(mo) = sau.find('"') else { break };
        let Some(dai) = sau[mo + 1..].find('"') else {
            break;
        };
        let gia_tri = giai_ma(&sau[mo + 1..mo + 1 + dai]);
        if !ten_tt.is_empty() {
            thuoc_tinh.push((ten_tt, gia_tri));
        }
        con_lai = &sau[mo + 1 + dai + 1..];
    }
    The {
        ten,
        dong,
        thuoc_tinh,
    }
}

/// Thẻ không có thẻ đóng.
fn la_the_rong(ten: &str) -> bool {
    matches!(ten, "input" | "img")
}

/// Thẻ chỉ để BỌC, không phải một nút.
///
/// `<label>` tồn tại để chữ hiện ra cạnh ô đánh dấu. Nó không mang vai trò nào
/// và không sinh ra nút trợ năng — nút là cái `<input>` bên trong.
fn la_the_boc(ten: &str) -> bool {
    ten == "label"
}

/// Dựng lại cây trợ năng từ chuỗi đánh dấu.
///
/// # Errors
/// Thẻ lệch, thẻ lạ, thiếu nhãn, hoặc nhãn khác nội dung hiện ra.
pub fn quet(danh_dau: &str) -> Result<AccessNode, QuetLoi> {
    // Mỗi phần tử: (nút đang dựng, tên thẻ, chữ gom được bên trong)
    // Phần tử: (nút đang dựng, tên thẻ, chữ gom được, có phải thẻ bọc không)
    let mut ngan_xep: Vec<(AccessNode, String, String, bool)> = Vec::new();
    let mut xong: Vec<AccessNode> = Vec::new();
    let mut con_lai = danh_dau;

    while let Some(vt) = con_lai.find('<') {
        // Chữ nằm giữa hai thẻ thuộc về nút đang mở.
        let chu = &con_lai[..vt];
        if let Some((_, _, gom, _)) = ngan_xep.last_mut() {
            gom.push_str(&giai_ma(chu));
        }
        let sau = &con_lai[vt + 1..];
        let Some(het) = sau.find('>') else {
            return Err(QuetLoi::Cut);
        };
        let the = doc_the(&sau[..het]);
        con_lai = &sau[het + 1..];

        if the.dong {
            let Some((nut, ten_mo, gom, boc)) = ngan_xep.pop() else {
                return Err(QuetLoi::TheLech(the.ten));
            };
            if ten_mo != the.ten {
                return Err(QuetLoi::TheLech(the.ten));
            }
            if boc {
                // Thẻ bọc: chữ của nó phải khớp nhãn của phần tử bên trong. Đây
                // chính là phép kiểm "chữ hiện ra = chữ đọc lên", áp cho công
                // tắc — chỗ mà một ô vuông không nhãn từng lọt qua.
                for con in nut.children {
                    kiem_nhan_khop_noi_dung("label", &con, &gom)?;
                    dat_vao(&mut ngan_xep, &mut xong, con);
                }
            } else {
                kiem_nhan_khop_noi_dung(&ten_mo, &nut, &gom)?;
                dat_vao(&mut ngan_xep, &mut xong, nut);
            }
            continue;
        }

        if la_the_boc(&the.ten) {
            ngan_xep.push((
                AccessNode {
                    role: Role::Group,
                    label: None,
                    children: Vec::new(),
                },
                the.ten,
                String::new(),
                true,
            ));
            continue;
        }

        let nut = dung_nut(&the)?;
        if la_the_rong(&the.ten) {
            dat_vao(&mut ngan_xep, &mut xong, nut);
        } else {
            ngan_xep.push((nut, the.ten, String::new(), false));
        }
    }

    if !ngan_xep.is_empty() {
        return Err(QuetLoi::TheLech(
            ngan_xep.pop().map_or_else(String::new, |(_, t, _, _)| t),
        ));
    }
    if xong.len() == 1 {
        xong.pop().ok_or(QuetLoi::KhongDungMotGoc(0))
    } else {
        Err(QuetLoi::KhongDungMotGoc(xong.len()))
    }
}

fn dat_vao(
    ngan_xep: &mut [(AccessNode, String, String, bool)],
    xong: &mut Vec<AccessNode>,
    nut: AccessNode,
) {
    if let Some((cha, _, _, _)) = ngan_xep.last_mut() {
        cha.children.push(nut);
    } else {
        xong.push(nut);
    }
}

/// ⚠️ Chữ hiện ra và nhãn đọc lên phải TRÙNG NHAU.
///
/// Lệch nhau là dạng lừa dối tệ nhất trong giao diện: một nút hiện chữ "Huỷ"
/// nhưng đọc lên là "Xác nhận" thì người dùng trình đọc màn hình bấm nhầm mà
/// không có cách nào biết. Chỉ kiểm với thẻ có chữ bên trong.
fn kiem_nhan_khop_noi_dung(ten: &str, nut: &AccessNode, gom: &str) -> Result<(), QuetLoi> {
    if !matches!(ten, "p" | "button" | "label") {
        return Ok(());
    }
    let noi_dung = gom.trim();
    let nhan = nut.label.as_deref().unwrap_or_default();
    if noi_dung == nhan {
        Ok(())
    } else {
        Err(QuetLoi::NhanLechNoiDung {
            nhan: nhan.to_owned(),
            noi_dung: noi_dung.to_owned(),
        })
    }
}

fn dung_nut(the: &The) -> Result<AccessNode, QuetLoi> {
    let nhan = the.lay("aria-label").map(str::to_owned);
    let can_nhan = |n: Option<String>| n.ok_or_else(|| QuetLoi::ThieuNhan(the.ten.clone()));

    let (role, label) = match the.ten.as_str() {
        "p" => (Role::Text, Some(can_nhan(nhan)?)),
        "button" => (
            Role::Button {
                // Nhận theo DẤU HIỆU MÁY, không theo chữ. So chữ thì câu cảnh
                // báo không dịch được — xem `DAU_MAT_MAT`.
                destructive: the.lay("data-sac-thai") == Some(DAU_MAT_MAT),
            },
            Some(can_nhan(nhan)?),
        ),
        // Phân nhánh theo VAI TRÒ trước, rồi mới tới kiểu thẻ. Công tắc mang
        // `role="switch"` — đó là ARIA NÂNG CẤP (ô đánh dấu → công tắc), dùng
        // đúng chỗ. Ô nhập thì KHÔNG mang role, vì ARIA ở đó là hạ cấp.
        "input" if the.lay("role") == Some("switch") => (
            Role::Switch {
                on: the.lay("aria-checked") == Some("true"),
            },
            Some(can_nhan(nhan)?),
        ),
        "input" => (
            Role::TextInput {
                secret: the.lay("type") == Some("password"),
            },
            Some(can_nhan(nhan)?),
        ),
        // Ảnh trang trí CỐ Ý không có nhãn — đó là tín hiệu bảo trình đọc màn
        // hình đi qua, nên ở đây `None` là hợp lệ chứ không phải thiếu sót.
        "img" => (Role::Image, nhan),
        "div" => (Role::Group, nhan),
        khac => return Err(QuetLoi::TheLa(khac.to_owned())),
    };

    Ok(AccessNode {
        role,
        label,
        children: Vec::new(),
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "kiểm thử: hỏng thì phải nổ ngay")]
mod kiem_thu {
    use super::*;

    // ⚠️ Đánh dấu trong các phép thử dưới đây VIẾT TAY, cố ý không gọi
    // `danh_dau::than`. Nếu bộ quét lấy đầu vào từ chính bộ sinh thì hai bên
    // cùng sai theo một kiểu mà không ai bắt được — đó là điều phải tránh.

    #[test]
    fn quet_duoc_cay_long_nhau() {
        let m = "<div role=\"group\"><p aria-label=\"xin chào\">xin chào</p>\
                 <button aria-label=\"Gửi\">Gửi</button></div>";
        let a = quet(m).unwrap();
        assert_eq!(a.role, Role::Group);
        assert_eq!(a.children.len(), 2);
        assert_eq!(a.children[0].role, Role::Text);
        assert_eq!(a.children[1].role, Role::Button { destructive: false });
    }

    #[test]
    fn nut_mat_mat_doc_ra_dung() {
        let m = format!("<button aria-label=\"Xoá\" data-sac-thai=\"{DAU_MAT_MAT}\">Xoá</button>");
        assert_eq!(quet(&m).unwrap().role, Role::Button { destructive: true });
    }

    #[test]
    fn cong_tac_doc_ra_dung_ca_hai_trang_thai() {
        let bat = "<input type=\"checkbox\" role=\"switch\" aria-checked=\"true\" \
                   aria-label=\"Quyền mạng\">";
        assert_eq!(quet(bat).unwrap().role, Role::Switch { on: true });

        let tat = "<input type=\"checkbox\" role=\"switch\" aria-checked=\"false\" \
                   aria-label=\"Quyền mạng\">";
        assert_eq!(quet(tat).unwrap().role, Role::Switch { on: false });
    }

    /// Công tắc và ô nhập cùng là thẻ `input` — đọc nhầm nhau là trình đọc màn
    /// hình mô tả sai hoàn toàn thứ người dùng đang chạm vào.
    #[test]
    fn cong_tac_khong_bi_doc_nham_thanh_o_nhap() {
        let ct = "<input type=\"checkbox\" role=\"switch\" aria-checked=\"false\" \
                  aria-label=\"X\">";
        assert!(matches!(quet(ct).unwrap().role, Role::Switch { .. }));

        let on = "<input type=\"text\" role=\"textbox\" aria-label=\"X\" value=\"\">";
        assert!(matches!(quet(on).unwrap().role, Role::TextInput { .. }));
    }

    #[test]
    fn o_nhap_bi_mat_doc_ra_dung() {
        let m = "<input type=\"password\" aria-label=\"Mật khẩu\" value=\"x\">";
        assert_eq!(quet(m).unwrap().role, Role::TextInput { secret: true });

        let m2 = "<input type=\"text\" aria-label=\"Tên\" value=\"\">";
        assert_eq!(quet(m2).unwrap().role, Role::TextInput { secret: false });
    }

    #[test]
    fn anh_trang_tri_khong_nhan_anh_thuong_co_nhan() {
        let tt = quet("<img src=\"a.png\" alt=\"\" role=\"presentation\">").unwrap();
        assert_eq!(tt.label, None);

        let co = quet("<img src=\"a.png\" alt=\"Biểu đồ\" aria-label=\"Biểu đồ\">").unwrap();
        assert_eq!(co.label.as_deref(), Some("Biểu đồ"));
    }

    /// ⚠️ Phép thử quan trọng nhất tệp này.
    #[test]
    fn nhan_khac_chu_hien_ra_thi_bao_loi() {
        let m = "<button aria-label=\"Xác nhận\">Huỷ</button>";
        assert!(matches!(quet(m), Err(QuetLoi::NhanLechNoiDung { .. })));
    }

    #[test]
    fn thieu_nhan_thi_bao_loi() {
        assert!(matches!(
            quet("<p>không nhãn</p>"),
            Err(QuetLoi::ThieuNhan(_))
        ));
    }

    #[test]
    fn the_lech_hoac_la_thi_bao_loi() {
        assert!(matches!(
            quet("<div role=\"group\">"),
            Err(QuetLoi::TheLech(_))
        ));
        assert!(matches!(quet("</div>"), Err(QuetLoi::TheLech(_))));
        assert!(matches!(
            quet("<script>xấu()</script>"),
            Err(QuetLoi::TheLa(_))
        ));
    }

    #[test]
    fn hai_goc_hoac_khong_goc_deu_bao_loi() {
        let hai = "<p aria-label=\"a\">a</p><p aria-label=\"b\">b</p>";
        assert!(matches!(quet(hai), Err(QuetLoi::KhongDungMotGoc(2))));
        assert!(matches!(quet(""), Err(QuetLoi::KhongDungMotGoc(0))));
    }

    #[test]
    fn danh_dau_cut_thi_bao_loi() {
        assert!(matches!(quet("<p aria-label=\"a\""), Err(QuetLoi::Cut)));
    }

    // ---- Giải mã thực thể ----

    #[test]
    fn giai_ma_dung_moi_thuc_the() {
        assert_eq!(giai_ma("&lt;b&gt;&amp;&quot;&#39;"), "<b>&\"'");
    }

    /// `&amp;lt;` phải ra `&lt;` chứ KHÔNG được ra `<` — giải mã hai lần là mở
    /// lại đúng cái lỗ mà thoát ký tự vừa bịt.
    #[test]
    fn khong_giai_ma_hai_lan() {
        assert_eq!(giai_ma("&amp;lt;"), "&lt;");
    }

    #[test]
    fn giai_ma_giu_nguyen_tieng_viet_va_emoji() {
        assert_eq!(giai_ma("Chào bạn 🎉"), "Chào bạn 🎉");
    }
}
