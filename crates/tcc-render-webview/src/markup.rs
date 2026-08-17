//! Dịch cây component sang đánh dấu cho WebView.
//!
//! ⚠️ ĐÂY LÀ TỆP NGUY HIỂM NHẤT CỦA BỘ DỰNG.
//!
//! Cây component chứa chữ do ỨNG DỤNG viết. Nếu chữ đó đi thẳng vào tài liệu mà
//! không thoát ký tự, thì một ứng dụng chỉ cần đặt tên nút là `<script>…` là
//! chạy được mã trong ngữ cảnh của bộ dựng — tức là thoát khỏi toàn bộ mô hình
//! quyền năng mà `tcc-capability` dựng lên. Mọi thứ phía trên thành vô nghĩa.
//!
//! `tcc-ui` đã chặn ký tự giả mạo hiển thị, nhưng thế KHÔNG đủ: chuỗi
//! `<script>alert(1)</script>` không chứa một ký tự cấm nào. Hai phép kiểm khác
//! mục đích, cần cả hai.
//!
//! Ba tầng phòng thủ, xếp chồng:
//!   1. Thoát ký tự mọi chuỗi của ứng dụng (tệp này)
//!   2. Chính sách nội dung chặn mọi kịch bản, kể cả khi tầng 1 thủng
//!   3. Ảnh chỉ lấy từ gói đã ký (`tcc-ui` đã chặn địa chỉ mạng)

use std::fmt::Write as _;

use tcc_ui::{Alt, Emphasis, Flow, Gap, Node, NodeKind, Tone};

/// Dấu hiệu MÁY đọc cho một hành động không hoàn tác được.
///
/// # ⚠️ Dấu hiệu cho MÁY và chữ cho NGƯỜI phải là HAI thứ
///
/// Bản đầu gộp làm một: bộ quét trợ năng so `aria-description` với đúng chuỗi
/// tiếng Việt "Hành động không hoàn tác được." Hệ quả là câu đó **không dịch
/// được** — dịch sang tiếng Anh là bộ quét mù, và mọi phép thử đỏ.
///
/// Gộp hai vai trò vào một chuỗi luôn khoá chặt nó lại như vậy. Nay dấu hiệu
/// máy là thuộc tính này (không bao giờ đổi, không bao giờ hiện ra), còn chữ
/// cho người thì tự do dịch.
pub const MARKER_DESTRUCTIVE: &str = "mat-mat";

/// Chữ mà BỘ DỰNG cần, do tầng trên cấp cho.
///
/// Bộ dựng không biết ngôn ngữ và không nên biết — bảng dịch nằm ở `tcc-shell`.
/// Nó chỉ nhận chữ đã dịch sẵn, đúng lối đã dùng với `trait Network` và trình phục
/// vụ tệp: thứ gì phụ thuộc ngữ cảnh thì tiêm từ ngoài vào.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RendererText {
    /// Câu mô tả hành động mất mát, đọc lên cho trình đọc màn hình.
    pub cau_mat_mat: String,
    /// Chuỗi thay cho tên vai trò "nút".
    ///
    /// `aria-roledescription` THAY THẾ tên vai trò, nên chuỗi này phải tự nhắc
    /// đây là một nút — nếu không người dùng mất thông tin đó.
    pub vai_tro_mat_mat: String,
}

impl Default for RendererText {
    /// Mặc định TIẾNG ANH — đúng mặc định của cả giao diện.
    fn default() -> Self {
        Self {
            cau_mat_mat: "This cannot be undone.".to_owned(),
            vai_tro_mat_mat: "button — this cannot be undone".to_owned(),
        }
    }
}

/// Thoát ký tự cho chuỗi của ứng dụng.
///
/// Thoát cả `'` và `"` vì cùng một hàm dùng cho cả nội dung lẫn giá trị thuộc
/// tính. Một hàm cho cả hai chỗ thì không có chỗ nào bị quên.
#[must_use]
pub fn escape_html(s: &str) -> String {
    let mut ra = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => ra.push_str("&amp;"),
            '<' => ra.push_str("&lt;"),
            '>' => ra.push_str("&gt;"),
            '"' => ra.push_str("&quot;"),
            '\'' => ra.push_str("&#39;"),
            _ => ra.push(c),
        }
    }
    ra
}

const fn ten_huong(f: Flow) -> &'static str {
    match f {
        Flow::Row => "ngang",
        Flow::Column => "doc",
    }
}

const fn ten_cach(g: Gap) -> &'static str {
    match g {
        Gap::None => "khong",
        Gap::Small => "nho",
        Gap::Medium => "vua",
        Gap::Large => "lon",
    }
}

const fn ten_nhan(e: Emphasis) -> &'static str {
    match e {
        Emphasis::Title => "tieu-de",
        Emphasis::Normal => "thuong",
        Emphasis::Subtle => "phu",
        Emphasis::Warning => "canh-bao",
    }
}

const fn ten_sac_thai(t: Tone) -> &'static str {
    match t {
        Tone::Neutral => "thuong",
        Tone::Primary => "chinh",
        Tone::Danger => "mat-mat",
    }
}

/// Dịch một cây thành phần thân tài liệu.
///
/// Mỗi nút sinh ra ĐÚNG MỘT thẻ, không thẻ bọc thừa. Đó là điều kiện để bộ quét
/// trợ năng dựng lại được cây một-đối-một.
#[must_use]
pub fn body(tree: &Node) -> String {
    body_with_text(tree, &RendererText::default())
}

/// Như [`body`] nhưng dùng chữ do tầng trên cấp.
#[must_use]
pub fn body_with_text(tree: &Node, chu: &RendererText) -> String {
    let mut ra = String::new();
    ve(tree, chu, &mut ra);
    ra
}

fn ve(n: &Node, chu: &RendererText, ra: &mut String) {
    match n.kind() {
        NodeKind::Text { content, emphasis } => {
            let c = escape_html(content);
            let _ = write!(
                ra,
                "<p role=\"paragraph\" aria-label=\"{c}\" data-nhan=\"{}\">{c}</p>",
                ten_nhan(*emphasis)
            );
        }
        NodeKind::Button {
            label,
            action,
            tone,
        } => {
            let l = escape_html(label);
            // Mã hành động đã bị `ActionId::parse` ép về ASCII hẹp nên không có
            // gì để thoát — nhưng vẫn thoát, vì phòng thủ không nên phụ thuộc
            // vào một bất biến ở crate khác có thể bị nới ra sau này.
            // `aria-description` KHÔNG lên được trục trợ năng của macOS — đã soi
            // cây thật và nó biến mất. Dùng `title` (ánh xạ sang AXHelp) cộng
            // `aria-roledescription` (ánh xạ sang AXRoleDescription): trình đọc
            // màn hình đọc "Xoá dữ liệu, <câu cảnh báo>" thay vì "…, nút".
            let mo_ta = if *tone == Tone::Danger {
                format!(
                    " title=\"{}\" aria-roledescription=\"{}\" aria-description=\"{}\"",
                    escape_html(&chu.cau_mat_mat),
                    escape_html(&chu.vai_tro_mat_mat),
                    escape_html(&chu.cau_mat_mat)
                )
            } else {
                String::new()
            };
            let _ = write!(
                ra,
                "<button role=\"button\" aria-label=\"{l}\"{mo_ta} \
                 data-hanh-dong=\"{}\" data-sac-thai=\"{}\">{l}</button>",
                escape_html(action.as_str()),
                ten_sac_thai(*tone)
            );
        }
        NodeKind::Field {
            label,
            value,
            secret,
        } => {
            // Ô bí mật ra thẻ nhập kiểu mật khẩu THẬT, không phải ô thường tô
            // chấm bằng bảng kiểu: kiểu thật là thứ khiến hệ điều hành không đưa
            // nội dung vào gợi ý gõ và không chụp vào ảnh màn hình tự động.
            let kieu = if *secret { "password" } else { "text" };
            // ⚠️ Nhãn phải HIỆN RA, cùng lý do với công tắc: `aria-label` chỉ để
            // trình đọc màn hình nghe. Người dùng sáng mắt thấy một ô trống thì
            // không biết ô đó để nhập gì.
            let l = escape_html(label);
            // ⚠️ KHÔNG đặt `role="textbox"`.
            //
            // ARIA ĐÈ LÊN ngữ nghĩa gốc, và ở đây đè xuống thành tệ hơn: thẻ nhập
            // kiểu mật khẩu vốn ra `AXSecureTextField`, nhưng thêm `role=textbox`
            // là nó tụt xuống `AXTextField`. Trình đọc màn hình đọc TO từng ký tự
            // của ô thường, và không đọc của ô bảo mật — nên bản vá "cho rõ ràng"
            // của tôi đã biến ô mật khẩu thành ô đọc-thành-tiếng.
            //
            // Chỉ phát hiện được khi soi cây trợ năng THẬT của hệ điều hành.
            // Luật rút ra: đừng thêm ARIA khi thẻ gốc đã nói đúng.
            let _ = write!(
                ra,
                "<label>{l}<input type=\"{kieu}\" aria-label=\"{l}\" value=\"{}\"></label>",
                escape_html(value)
            );
        }
        NodeKind::Toggle { label, on, action } => {
            // Ô đánh dấu THẬT với `role="switch"`, không phải một ô vuông vẽ
            // bằng bảng kiểu: hệ điều hành chỉ đọc được trạng thái bật/tắt cho
            // trình đọc màn hình khi phần tử thật sự mang vai trò đó.
            //
            // ⚠️ BỌC TRONG `<label>` VÀ VIẾT CHỮ RA. Bản đầu chỉ có `aria-label`,
            // nghĩa là **chỉ trình đọc màn hình nghe thấy** — người dùng sáng mắt
            // thấy một ô vuông trống, không biết mình đang bật cái gì. Hộp thoại
            // hỏi quyền mà nút quyết định không có chữ thì cả tầng quyền năng vô
            // nghĩa.
            //
            // 211 phép thử không bắt được: cây trợ năng CÓ nhãn nên phép kiểm
            // định trợ năng qua. Chỉ khi chụp được cửa sổ mới lộ ra.
            let l = escape_html(label);
            let _ = write!(
                ra,
                "<label><input type=\"checkbox\" role=\"switch\" aria-checked=\"{on}\" \
                 aria-label=\"{l}\" data-hanh-dong=\"{}\"{}>{l}</label>",
                escape_html(action.as_str()),
                if *on { " checked" } else { "" }
            );
        }
        NodeKind::Image { source, alt } => match alt {
            Alt::Text(t) => {
                let _ = write!(
                    ra,
                    "<img src=\"{}\" role=\"img\" alt=\"{}\" aria-label=\"{}\">",
                    escape_html(&crate::package_server::url_for(source)),
                    escape_html(t),
                    escape_html(t)
                );
            }
            // Ảnh trang trí: `alt` rỗng CỘNG `role=presentation`. Chỉ một trong
            // hai là chưa đủ — vài trình đọc màn hình vẫn đọc tên tệp lên.
            //
            // Ảnh CÓ mô tả thì mang `role="img"`. Trước đây nó không mang vai trò
            // nào — thẻ `img` có vai trò ngầm nên trình đọc màn hình vẫn xử lý
            // đúng, nhưng nó phá bất biến "mọi nút đều mang vai trò RÕ RÀNG", và
            // bất biến đó mới là thứ đếm được và kiểm được.
            Alt::Decorative => {
                let _ = write!(
                    ra,
                    "<img src=\"{}\" alt=\"\" role=\"presentation\">",
                    escape_html(&crate::package_server::url_for(source))
                );
            }
        },
        NodeKind::Group { flow, gap } => {
            // ⚠️ Hàng TOÀN NÚT được đánh dấu, để CSS kéo chúng rộng BẰNG NHAU.
            //
            // Không phải thẩm mỹ. Màn xác nhận giao dịch cố ý cho hai nút cùng
            // sắc thái, vì làm nút "Ký" nổi hơn là đẩy người dùng về một phía
            // đúng lúc nguy hiểm nhất. Bề rộng cũng đẩy: "Ký giao dịch này"
            // rộng gấp ba "Huỷ" vẫn là một cái hích, chỉ bằng hình học thay vì
            // bằng màu.
            //
            // Đánh dấu ở đây chứ không viết CSS đoán mò: CSS không hỏi được
            // "hàng này có toàn nút không", mà một nút đứng cạnh nhãn thì kéo
            // giãn ra là vô nghĩa.
            let hang_nut = *flow == Flow::Row
                && n.children().len() > 1
                && n.children()
                    .iter()
                    .all(|c| matches!(c.kind(), NodeKind::Button { .. }));
            let _ = write!(
                ra,
                "<div role=\"group\" data-huong=\"{}\" data-cach=\"{}\"{}>",
                ten_huong(*flow),
                ten_cach(*gap),
                if hang_nut { " data-hang-nut=\"\"" } else { "" }
            );
            for c in n.children() {
                ve(c, chu, ra);
            }
            ra.push_str("</div>");
        }
    }
}

/// Bảng kiểu tối thiểu của BỘ DỰNG.
///
/// # Vì sao bộ dựng phải có bảng kiểu, không phải ứng dụng
///
/// Ứng dụng khai Ý ĐỊNH (`Tone::Danger`, `Gap::Large`); bộ dựng quyết định hình
/// thức. Nhưng khai ý định mà bộ dựng **không vẽ khác đi** thì ý định đó vô
/// nghĩa: nút "Xoá dữ liệu" trông y hệt nút "Tải trang", và cả tầng `Tone` chỉ
/// còn là chú thích trong mã.
///
/// Lỗi này chỉ lộ ra khi chụp được cửa sổ — mọi phép thử đều kiểm thuộc tính
/// `data-sac-thai` có mặt, không kiểm nó có tác dụng gì.
///
/// Bảng kiểu này CỐ Ý tối giản. Nó không phải hệ thống thiết kế; nó chỉ đảm bảo
/// mỗi ý định khai ra đều có một biểu hiện nhìn thấy được.
const BANG_KIEU: &str = "\
body{font:15px/1.55 -apple-system,system-ui,sans-serif;margin:22px;color:#14161c}\
[data-nhan=tieu-de]{font-size:1.5em;font-weight:600;margin:0 0 .3em}\
[data-nhan=phu]{color:#5b6270;font-size:.92em}\
[data-nhan=canh-bao]{color:#8a2b06;background:#fff1e8;border-left:3px solid #d2521a;\
padding:4px 9px;border-radius:4px;font-weight:600}\
[data-huong=doc]{display:flex;flex-direction:column}\
[data-huong=ngang]{display:flex;flex-direction:row;align-items:center}\
[data-cach=nho]{gap:6px}[data-cach=vua]{gap:12px}[data-cach=lon]{gap:20px}\
label{display:flex;align-items:center;gap:8px}\
input[type=text],input[type=password]{flex:1;padding:6px 9px;border:1px solid #c3c7cf;border-radius:6px;font:inherit}\
\
[data-hang-nut]>button{flex:1 1 0;align-self:auto}\
button{align-self:start;padding:7px 15px;border-radius:7px;border:1px solid #c3c7cf;background:#f6f7f9;font:inherit;cursor:pointer}\
[data-sac-thai=chinh]{border-color:#ff8a3d;background:#fff3ea}\
[data-sac-thai=mat-mat]{border-color:#c0392b;color:#c0392b;background:#fdf0ee;font-weight:600}\
\
img{max-width:100%;align-self:start}\
";

/// Tài liệu đầy đủ, kèm chính sách nội dung.
///
/// Chính sách này là TẦNG PHÒNG THỦ THỨ HAI: kể cả khi [`escape_html`] có lỗ, kịch bản
/// vẫn không chạy vì không nguồn nào được phép. `default-src 'none'` chặn hết,
/// rồi mở đúng hai thứ cần: chữ nội tuyến và ảnh từ giao thức của gói.
#[must_use]
pub fn document(tree: &Node) -> String {
    document_with_text(tree, &RendererText::default())
}

/// Như [`document`] nhưng dùng chữ do tầng trên cấp.
#[must_use]
pub fn document_with_text(tree: &Node, chu: &RendererText) -> String {
    format!(
        "<!doctype html><meta charset=\"utf-8\">\
         <meta http-equiv=\"Content-Security-Policy\" content=\"\
         default-src 'none'; img-src tcc-goi:; style-src 'unsafe-inline'; \
         script-src 'none'; object-src 'none'; frame-src 'none'; \
         form-action 'none'; base-uri 'none'\">\
         <style>{BANG_KIEU}</style>\
         <body>{}</body>",
        body_with_text(tree, chu)
    )
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay, không nuốt lỗi"
)]
mod kiem_thu {
    use super::*;

    /// ⚠️ PHÉP THỬ QUAN TRỌNG NHẤT TỆP NÀY.
    ///
    /// Ứng dụng đặt tên nút là mã kịch bản. `tcc-ui` cho qua — đúng, vì chuỗi
    /// này không chứa ký tự giả mạo hiển thị nào. Chặn nằm ở ĐÂY.
    #[test]
    fn chu_cua_ung_dung_khong_thoat_ra_duoc_tai_lieu() {
        const NHAN_AC: &str = "\" onclick=\"doi_vi()";
        let doc = document(
            &Node::group(Flow::Column, Gap::None)
                .child(Node::text("<script>doi_vi()</script>").unwrap())
                .unwrap()
                .child(Node::button(NHAN_AC, "ok", Tone::Neutral).unwrap())
                .unwrap(),
        );

        assert!(!doc.contains("<script>"), "thẻ kịch bản lọt vào tài liệu");
        assert!(
            doc.contains("&lt;script&gt;"),
            "chữ gốc bị mất, không phải bị thoát"
        );

        // ⚠️ ĐỪNG khẳng định `!doc.contains("onclick=")`. Bản đầu tôi viết thế và
        // nó ĐỎ OAN: nhãn đã thoát thành `&quot; onclick=&quot;doi_vi()`, tức
        // chuỗi "onclick=" vẫn có mặt nhưng là CHỮ TRƠ, không phải thuộc tính.
        // Tìm chuỗi con là cách kiểm sai vấn đề.
        //
        // Bằng chứng đúng là ĐỌC NGƯỢC: nếu nhãn phá được ra khỏi giá trị thuộc
        // tính thì bộ quét sẽ đọc ra một cây khác, hoặc một nhãn cụt.
        let a = crate::a11y_scan::scan(&body(&Node::button(NHAN_AC, "ok", Tone::Neutral).unwrap()))
            .expect("nhãn ác làm vỡ đánh dấu");
        assert_eq!(
            a.label.as_deref(),
            Some(NHAN_AC),
            "nhãn không về nguyên vẹn — đã phá được ra khỏi giá trị thuộc tính"
        );
    }

    /// Thoát ký tự phải chạy trên MỌI chuỗi đến từ ứng dụng, không sót loại nút
    /// nào. Phép thử này quét cả năm loại một lượt.
    #[test]
    fn moi_loai_nut_deu_thoat_ky_tu() {
        let doc_ac = "<b>&\"'";
        let cay = Node::group(Flow::Column, Gap::None)
            .child(Node::text(doc_ac).unwrap())
            .unwrap()
            .child(Node::button(doc_ac, "x", Tone::Neutral).unwrap())
            .unwrap()
            .child(Node::field(doc_ac, doc_ac, false).unwrap())
            .unwrap()
            .child(Node::image("a.png", Alt::Text(doc_ac.into())).unwrap())
            .unwrap();
        let ra = body(&cay);
        assert!(
            !ra.contains("<b>"),
            "một loại nút nào đó chưa thoát ký tự:\n{ra}"
        );
    }

    #[test]
    fn chinh_sach_noi_dung_chan_kich_ban() {
        let doc = document(&Node::text("xin chào").unwrap());
        assert!(doc.contains("default-src 'none'"));
        assert!(doc.contains("script-src 'none'"));
        // Ảnh chỉ từ giao thức của gói — không phải từ mạng.
        assert!(doc.contains("img-src tcc-goi:"));
        assert!(!doc.contains("img-src *"));
    }

    #[test]
    fn o_bi_mat_ra_the_nhap_kieu_mat_khau_that() {
        let ra = body(&Node::field("Mật khẩu", "abc", true).unwrap());
        assert!(ra.contains("type=\"password\""), "{ra}");

        let thuong = body(&Node::field("Tên", "abc", false).unwrap());
        assert!(thuong.contains("type=\"text\""), "{thuong}");
    }

    /// ⚠️ Nhãn công tắc phải HIỆN RA, không chỉ nằm trong `aria-label`.
    ///
    /// Bản đầu chỉ có `aria-label` — người dùng sáng mắt thấy một ô vuông trống.
    /// Ở hộp thoại hỏi quyền, đó chính là nút quyết định.
    #[test]
    fn nhan_cong_tac_hien_ra_cho_nguoi_nhin_thay() {
        let ra = body(&Node::toggle("Kết nối tới các máy chủ này", false, "q-mang").unwrap());
        // Chữ phải nằm NGOÀI thuộc tính, tức là trong phần nội dung của thẻ.
        assert!(
            ra.contains(">Kết nối tới các máy chủ này</label>"),
            "nhãn công tắc không hiện ra cho người nhìn thấy:\n{ra}"
        );
    }

    /// ⚠️ Nhãn ô nhập cũng phải HIỆN RA, không chỉ nằm trong `aria-label`.
    #[test]
    fn nhan_o_nhap_hien_ra_cho_nguoi_nhin_thay() {
        let ra = body(&Node::field("Gõ thử tiếng Việt", "", false).unwrap());
        assert!(
            ra.contains("<label>Gõ thử tiếng Việt<input"),
            "nhãn ô nhập không hiện ra:\n{ra}"
        );
    }

    /// ⚠️ Nút KHÔNG được giãn kín bề ngang.
    ///
    /// Trong hộp xếp dọc, phần tử con mặc định giãn hết — và một nút MẤT MÁT to
    /// bằng cả màn hình là một cái bẫy bấm nhầm. Lỗi này lộ ra ở màn hình quản
    /// lý quyền, nơi nút "Quên ứng dụng này" chiếm trọn bề ngang.
    #[test]
    fn nut_khong_gian_kin_be_ngang() {
        let doc = document(&Node::button("Quên", "forget", Tone::Danger).unwrap());
        assert!(
            doc.contains("button{align-self:start"),
            "nút sẽ giãn kín bề ngang trong hộp xếp dọc"
        );
    }

    /// ⚠️ Sắc thái MẤT MÁT phải được VẼ KHÁC ĐI, không chỉ khai ra.
    ///
    /// Khai `Tone::Danger` mà bộ dựng vẽ y hệt nút thường thì cả tầng sắc thái
    /// chỉ là chú thích trong mã. Lỗi này chỉ lộ ra khi chụp được cửa sổ.
    #[test]
    fn sac_thai_mat_mat_duoc_ve_khac_di() {
        let doc = document(&Node::button("Xoá", "xoa", Tone::Danger).unwrap());
        assert!(
            doc.contains("[data-sac-thai=mat-mat]"),
            "bảng kiểu không có luật nào cho sắc thái mất mát"
        );
        // Và luật đó phải thật sự đổi hình thức, không phải một luật rỗng.
        let i = doc.find("[data-sac-thai=mat-mat]").expect("có luật");
        let luat = &doc[i..i + 120];
        assert!(
            luat.contains("color") || luat.contains("border"),
            "luật sắc thái mất mát không đổi gì về hình thức: {luat}"
        );
    }

    /// ⚠️ Tín hiệu "mất mát" phải lên được TRỤC TRỢ NĂNG của hệ điều hành.
    ///
    /// `aria-description` một mình KHÔNG lên được — đã soi cây trợ năng thật của
    /// macOS và nó biến mất. Phải kèm `title` (→ AXHelp) và
    /// `aria-roledescription` (→ AXRoleDescription).
    #[test]
    fn nut_mat_mat_mang_cau_canh_bao() {
        let chu = RendererText::default();
        let ra = body(&Node::button("Xoá ví", "xoa", Tone::Danger).unwrap());
        assert!(ra.contains(&chu.cau_mat_mat), "{ra}");
        assert!(
            ra.contains("title="),
            "thiếu title → AXHelp không có gì:\n{ra}"
        );
        assert!(
            ra.contains("aria-roledescription="),
            "thiếu aria-roledescription:\n{ra}"
        );

        let thuong = body(&Node::button("Xem", "xem", Tone::Neutral).unwrap());
        assert!(!thuong.contains(&chu.cau_mat_mat), "{thuong}");
    }

    /// ⚠️ DẤU HIỆU MÁY và CHỮ CHO NGƯỜI phải tách rời.
    ///
    /// Bản đầu gộp làm một: bộ quét so `aria-description` với đúng chuỗi tiếng
    /// Việt. Hệ quả là câu đó không dịch được — dịch sang tiếng Anh là bộ quét
    /// mù. Đây là phép thử chốt rằng chúng đã tách.
    #[test]
    fn doi_chu_sang_ngon_ngu_khac_khong_lam_mat_dau_hieu_may() {
        let nut = Node::button("Delete", "xoa", Tone::Danger).unwrap();
        let anh = RendererText::default();
        let viet = RendererText {
            cau_mat_mat: "Hành động này không hoàn tác được.".to_owned(),
            vai_tro_mat_mat: "nút — hành động này không hoàn tác được".to_owned(),
        };

        for chu in [&anh, &viet] {
            let ra = body_with_text(&nut, chu);
            // Chữ cho người ĐỔI theo ngôn ngữ…
            assert!(ra.contains(&chu.cau_mat_mat), "{ra}");
            // …còn dấu hiệu máy thì KHÔNG BAO GIỜ đổi.
            assert!(
                ra.contains(&format!("data-sac-thai=\"{MARKER_DESTRUCTIVE}\"")),
                "mất dấu hiệu máy:\n{ra}"
            );
            // Và bộ quét vẫn nhận ra, dù chữ khác hẳn.
            let a = crate::a11y_scan::scan(&ra).expect("quét ngược hỏng");
            assert_eq!(a.role, tcc_ui::Role::Button { destructive: true });
        }
    }

    /// Mặc định của bộ dựng là TIẾNG ANH — đúng mặc định của cả giao diện.
    #[test]
    fn mac_dinh_bo_dung_la_tieng_anh() {
        let c = RendererText::default();
        assert!(
            c.cau_mat_mat.is_ascii(),
            "mặc định phải là tiếng Anh: {c:?}"
        );
        assert!(
            c.vai_tro_mat_mat.contains("button"),
            "chuỗi thay vai trò phải tự nhắc đây là nút: {c:?}"
        );
    }

    #[test]
    fn anh_tro_qua_giao_thuc_cua_goi() {
        let ra = body(&Node::image("anh/logo.png", Alt::Text("Biểu trưng".into())).unwrap());
        assert!(
            ra.contains("src=\"tcc-goi://goi/anh/logo.png\""),
            "ảnh vẫn dùng đường dẫn trần, sẽ không phân giải được:\n{ra}"
        );
        // Và phải khớp chính sách nội dung.
        let doc = document(&Node::image("a.png", Alt::Decorative).unwrap());
        assert!(doc.contains("img-src tcc-goi:"));
        assert!(doc.contains("src=\"tcc-goi://"));
    }

    #[test]
    fn anh_trang_tri_co_ca_alt_rong_lan_vai_tro_trang_tri() {
        let ra = body(&Node::image("anh/vien.png", Alt::Decorative).unwrap());
        assert!(ra.contains("alt=\"\""), "{ra}");
        assert!(ra.contains("role=\"presentation\""), "{ra}");
    }

    /// ⚠️ MỌI nút phải mang thuộc tính vai trò RÕ RÀNG.
    ///
    /// Vai trò ngầm của thẻ là đủ cho trình đọc màn hình, nhưng không đủ cho ta:
    /// bất biến đếm được mới kiểm được. Lỗ này chỉ lộ ra khi dựng ví dụ THẬT có
    /// ảnh kèm mô tả — mọi gói thử trước đó không có nút nào như vậy.
    #[test]
    fn moi_nut_deu_mang_vai_tro_ro_rang() {
        let cay = Node::group(Flow::Column, Gap::None)
            .child(Node::text("chữ").unwrap())
            .unwrap()
            .child(Node::button("nút", "x", Tone::Neutral).unwrap())
            .unwrap()
            .child(Node::field("ô", "", false).unwrap())
            .unwrap()
            .child(Node::toggle("công tắc", false, "ct").unwrap())
            .unwrap()
            .child(Node::image("a.png", Alt::Text("có mô tả".into())).unwrap())
            .unwrap()
            .child(Node::image("b.png", Alt::Decorative).unwrap())
            .unwrap();

        let ra = body(&cay);
        // ⚠️ BẤT BIẾN CŨ SAI, đã sửa.
        //
        // Bản đầu đòi "mỗi nút mang đúng một thuộc tính `role`". Nó khiến tôi
        // thêm `role="textbox"` vào ô nhập — và ARIA đè lên ngữ nghĩa gốc, kéo
        // ô mật khẩu từ `AXSecureTextField` tụt xuống `AXTextField`. Một bất
        // biến làm hỏng đúng thứ nó định bảo vệ.
        //
        // Bất biến đúng: **ô nhập KHÔNG được mang `role`**, mọi loại khác thì có.
        assert!(
            !ra.contains("<input type=\"text\" role=")
                && !ra.contains("<input type=\"password\" role="),
            "ô nhập mang ARIA role — nó ĐÈ LÊN ngữ nghĩa gốc và làm mất tính bảo \
             mật của ô mật khẩu:\n{ra}"
        );
        let so_o_nhap = ra.matches("<input type=\"text\"").count()
            + ra.matches("<input type=\"password\"").count();
        let so_vai_tro = ra.matches("role=\"").count();
        assert_eq!(
            so_vai_tro,
            cay.node_count() - so_o_nhap,
            "mọi nút TRỪ ô nhập phải mang vai trò rõ ràng:\n{ra}"
        );
    }

    #[test]
    fn moi_nut_sinh_dung_mot_the() {
        let cay = Node::group(Flow::Row, Gap::Small)
            .child(Node::text("a").unwrap())
            .unwrap()
            .child(Node::text("b").unwrap())
            .unwrap();
        let ra = body(&cay);
        assert_eq!(ra.matches("<p ").count(), 2);
        assert_eq!(ra.matches("<div ").count(), 1);
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

    /// **Định kiểu của ta KHÔNG được cho phép vẽ đè.**
    ///
    /// Vẽ đè là đòn che chữ: đặt một thứ lên trên câu "việc này chuyển tiền"
    /// thì người dùng xác nhận một thứ họ không đọc được. Ứng dụng không gửi
    /// CSS — nhưng nếu CHÍNH định kiểu của ta mở cửa cho chồng lớp thì một cây
    /// khéo sắp vẫn che được.
    ///
    /// Bộ dựng ra pixel chốt điều này bằng phép đếm ô chồng nhau. WebView thì
    /// không đếm được, nên chốt ở nguồn: những thứ tạo ra chồng lớp phải KHÔNG
    /// có trong định kiểu.
    #[test]
    fn dinh_kieu_khong_mo_cua_cho_ve_de() {
        let doc = document(&Node::text("x").unwrap());
        for cam in [
            "position:absolute",
            "position: absolute",
            "position:fixed",
            "position: fixed",
            "z-index",
            "margin:-",
            "margin: -",
            "margin-top:-",
            "margin-left:-",
            "transform:translate",
        ] {
            assert!(
                !doc.contains(cam),
                "định kiểu chứa {cam:?} — mở cửa cho vẽ đè lên câu cảnh báo"
            );
        }
    }

    /// Và không có `overflow:hidden` nào cắt mất nội dung trong im lặng.
    ///
    /// Cắt im lặng giấu đi phần giao diện người dùng đáng ra phải thấy — và
    /// phần bị giấu có thể là nút "Huỷ".
    #[test]
    fn khong_cat_noi_dung_trong_im_lang() {
        let doc = document(&Node::text("x").unwrap());
        assert!(!doc.contains("overflow:hidden"), "{doc}");
        assert!(!doc.contains("overflow: hidden"));
        assert!(!doc.contains("text-overflow"));
        // `white-space:nowrap` cũng đẩy chữ ra ngoài thay vì xuống dòng.
        assert!(!doc.contains("nowrap"));
    }
}
