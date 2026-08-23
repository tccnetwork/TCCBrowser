//! **Bố cục** — biến cây khai báo thành toạ độ tuyệt đối.
//!
//! # Vì sao tách khỏi `lib.rs`, và vì sao dùng `taffy`
//!
//! Bản trước tự viết bố cục: một hàm xếp cột, một hàm xếp hàng có xuống dòng,
//! một hàm xả dòng. Khoảng 113 dòng, và chúng **đúng** — năm phép thử trong
//! `lib.rs` ghim lại điều đó. Vấn đề không phải chúng sai, mà là chúng chỉ biết
//! đúng hai thứ: xếp dọc và xếp ngang. Không bề rộng, không căn lề, không đệm,
//! không vùng cuộn. Mỗi thứ ấy thêm vào là thêm một nhánh nữa vào đúng đoạn mã
//! mà một lỗi trong đó khiến **một nút vô hình vẫn bấm được** (xem `xa_dong`).
//!
//! `taffy` là bộ tính flexbox/grid độc lập, không dính bộ dựng nào — nó nhận
//! kiểu dáng và kích thước rồi trả toạ độ. Đưa phần *xếp* cho nó, giữ lại phần
//! *đo chữ* (cosmic-text) và phần *quyết định an ninh* ở đây.
//!
//! # Ba thứ KHÔNG giao cho `taffy`
//!
//! Vì `taffy` không có khái niệm tương ứng, và cả ba đều là quyết định của dự
//! án chứ không phải của flexbox:
//!
//! 1. **Nhóm lồng trong một hàng thì chiếm trọn dòng của nó.** Ở đây làm bằng
//!    `width: 100%`, không phải bằng một nhánh riêng.
//! 2. **Nút trên cùng một dòng rộng bằng nhau** — và chỉ khi kéo xong vẫn vừa.
//!    Làm SAU khi `taffy` tính xong, theo từng dòng thật, vì "cùng một dòng"
//!    chỉ biết được sau khi biết chỗ nào xuống dòng.
//! 3. **Đo chữ.** `taffy` không biết tiếng Việt có dấu phụ; cosmic-text biết.

use taffy::{
    AlignItems, AvailableSpace, Dimension, FlexDirection, FlexWrap, JustifyContent, NodeId,
    Overflow, Rect, Size, Style, TaffyTree,
    geometry::Point,
    prelude::{auto, length, percent},
};
use tcc_ui::{AccessNode, AlignCross, AlignMain, Extent, Flow, Gap, Node, NodeKind, Role, Sizing};

use crate::{DaDat, O};

/// Một nhóm đã dựng trong cây `taffy`, kèm chỗ tra ngược về ô đã đo.
///
/// `taffy` chỉ trả về toạ độ theo `NodeId`; phần chữ, cỡ, khung nằm trong `O`.
/// Ngữ cảnh của mỗi lá là chỉ số vào `la`.
struct Cay {
    cay: TaffyTree<usize>,
    la: Vec<O>,
}

/// Đổi bề khai của một nhóm thành kích thước `taffy`.
///
/// Trục CHÍNH theo `flow`, nên `main` là chiều ngang với hàng và chiều dọc với
/// cột. Bản đầu quên chỗ này và một nhóm `flow: column, size.main: half` ra nửa
/// bề NGANG — đúng nửa của trục sai.
///
/// `phu_vang` là nghĩa của "trục phụ vắng mặt", và nó KHÁC nhau theo chỗ gọi:
/// với `size` là chiếm trọn bề cha (nghĩa cũ của 0.1), còn với `min`/`max` là
/// không ràng buộc. Dùng chung một mặc định thì một `min` không khai trục phụ
/// hoá thành "tối thiểu bằng trọn bề cha" — và mọi thứ nở ra không ai hiểu vì
/// sao.
fn be_khai(flow: Flow, cha: Option<Flow>, size: Sizing, phu_vang: Dimension) -> Size<Dimension> {
    let doi = |be: Option<Extent>, mac_dinh: Dimension| match be {
        None => mac_dinh,
        // `content` và `none` để `taffy` tự đo. `fill` cũng vậy: nó chia khoảng
        // TRỐNG chứ không phải một phần của cha, tức là `flex-grow`, không phải
        // một bề — phần chia nằm ở `flex_grow` bên dưới.
        Some(Extent::Content | Extent::None | Extent::Fill) => auto(),
        Some(k) => k.ti_le().map_or(mac_dinh, percent),
    };
    // Trục chính không khai thì để `auto`; trục phụ không khai thì chiếm trọn bề
    // cha (nghĩa cũ của 0.1: một nhóm lồng chiếm trọn dòng của nó).
    // Bắt đầu từ hình dạng 0.1 — bề NGANG trọn cha, bề DỌC theo nội dung — rồi
    // mới để lời khai đè lên từng trục. Làm ngược lại (suy mặc định theo trục
    // chính) thì một nhóm `flow: row` không khai gì mất bề ngang, vì trục chính
    // của nó là chiều ngang. Hai phép thử bất biến bắt được đúng chỗ này.
    let mut ra = Size {
        width: phu_vang,
        height: auto(),
    };
    let (chinh, phu) = match flow {
        Flow::Row => (&mut ra.width, &mut ra.height),
        Flow::Column => (&mut ra.height, &mut ra.width),
    };
    match size.main {
        // ⚠️ `fill` KHÔNG phải một bề, và nó KHÔNG nói về trục chính của nút này.
        //
        // Nó là "một phần khoảng TRỐNG trên trục chính của CHA" — tức
        // `flex-grow`. Bản đầu gắn nó vào trục chính của chính nút ấy và để
        // `auto`, nên trong một cha kiểu CỘT nó làm nhóm **co lại theo nội
        // dung**: đo được 23/08/2026, con nhảy từ x=618 về x=12. Đó là làm một
        // việc người viết không hề xin, tệ hơn cả không làm gì.
        //
        // Đúng: chỉ để `auto` trên trục chính CỦA CHA, để `flex_grow` có chỗ
        // nới; trục kia giữ nguyên mặc định.
        Some(Extent::Fill) => {
            let cha_ngang = matches!(cha, Some(Flow::Row));
            if cha_ngang {
                ra.width = auto();
            } else {
                ra.height = auto();
            }
            return ra;
        }
        Some(m) => *chinh = doi(Some(m), auto()),
        None => {}
    }
    if size.cross.is_some() {
        *phu = doi(size.cross, phu_vang);
    }
    ra
}

/// Khe giữa các phần tử, theo pixel.
fn khe_cua(gap: Gap) -> f32 {
    match gap {
        Gap::None => 0.0,
        Gap::Small => 4.0,
        Gap::Medium => 8.0,
        Gap::Large => 12.0,
    }
}

impl Cay {
    /// Dựng cây `taffy` song song với cây khai báo, đo mọi lá trên đường đi.
    ///
    /// `do_la` là bao đóng giữ `&mut RasterRenderer` — bố cục không được biết bộ
    /// dựng nào, nên nó nhận về khả năng *đo* chứ không nhận bộ dựng.
    fn dung(
        &mut self,
        n: &Node,
        rong_toi_da: f32,
        do_la: &mut dyn FnMut(&Node, f32, &mut Vec<AccessNode>) -> O,
        access: &mut Vec<AccessNode>,
        // Hướng xếp của CHA. `None` ở gốc — gốc không có cha, và `fill` ở đó
        // không có khoảng trống nào để chia.
        cha: Option<Flow>,
    ) -> Option<NodeId> {
        let NodeKind::Group {
            flow,
            gap,
            size,
            min,
            max,
            align_main,
            align_cross,
            padding,
            wrap,
            scroll,
        } = n.kind()
        else {
            return self.dung_la(n, rong_toi_da, do_la, access);
        };

        let khe = khe_cua(*gap);
        let mut con_access = Vec::new();
        let mut con = Vec::new();
        for c in n.children() {
            if let Some(id) = self.dung(c, rong_toi_da, do_la, &mut con_access, Some(*flow)) {
                con.push(id);
            }
        }
        let dang = Style {
            flex_direction: match flow {
                Flow::Column => FlexDirection::Column,
                Flow::Row => FlexDirection::Row,
            },
            // Xuống dòng chứ không tràn ra ngoài: một nút bị đẩy khỏi mép là một
            // nút người dùng **không bấm được và không biết là có**.
            // Vắng lời khai thì XUỐNG DÒNG — mặc định của 0.1, không phải của
            // flexbox. Cây 0.1 không có chữ `wrap` nào, và một nút bị đẩy khỏi
            // mép là một nút người dùng **không bấm được và không biết là có**.
            // Ứng dụng 0.2 nói rõ `wrap: false` thì được để tràn: tràn nhìn thấy
            // được, và §9.1 nói tràn không bao giờ bị huỷ.
            flex_wrap: if wrap.unwrap_or(true) {
                FlexWrap::Wrap
            } else {
                FlexWrap::NoWrap
            },
            // Căn giữa theo trục ngang của dòng — một nhãn nhỏ cạnh một tiêu đề
            // lớn mà dính mép trên thì trông như bị treo lơ lửng.
            align_items: Some(match align_cross {
                // Vắng mặt = `Start`, và với HÀNG thì `Start` giữ nghĩa cũ của
                // 0.1: căn giữa theo chiều dọc. Một nhãn nhỏ cạnh một tiêu đề
                // lớn mà dính mép trên thì trông như bị treo lơ lửng, và cây 0.1
                // không có chữ nào để nói khác đi.
                AlignCross::Start => match flow {
                    Flow::Column => AlignItems::FLEX_START,
                    Flow::Row => AlignItems::CENTER,
                },
                AlignCross::End => AlignItems::FLEX_END,
                AlignCross::Center => AlignItems::CENTER,
                AlignCross::Stretch => AlignItems::STRETCH,
            }),
            justify_content: Some(match align_main {
                AlignMain::Start => JustifyContent::FLEX_START,
                AlignMain::End => JustifyContent::FLEX_END,
                AlignMain::Center => JustifyContent::CENTER,
            }),
            padding: {
                let d = khe_cua(*padding);
                Rect {
                    left: length(d),
                    right: length(d),
                    top: length(d),
                    bottom: length(d),
                }
            },
            // Vùng cuộn cắt nội dung; ngoài vùng cuộn thì KHÔNG cắt — §9.1 nói
            // tràn không bao giờ bị huỷ, vì tràn là thứ người viết ứng dụng phải
            // nhìn thấy mới sửa được.
            overflow: if *scroll {
                Point {
                    x: Overflow::Scroll,
                    y: Overflow::Scroll,
                }
            } else {
                Point {
                    x: Overflow::Visible,
                    y: Overflow::Visible,
                }
            },
            gap: Size {
                width: length(khe),
                height: length(khe),
            },
            // Mọi NHÓM chiếm trọn bề ngang cha cho nó. Trong một cột thì đó là
            // lẽ thường; trong một HÀNG thì đây chính là luật "nhóm lồng chiếm
            // trọn dòng", viết bằng kiểu dáng thay vì bằng một nhánh `if`.
            // Vắng lời khai thì nhóm chiếm trọn bề ngang cha — luật §8.1,
            // mặc định để tương thích 0.1, và 0.2 THAY THẾ nó ngay khi ứng dụng
            // khai `size.main`.
            size: be_khai(*flow, cha, *size, percent(1.0)),
            min_size: be_khai(*flow, cha, *min, auto()),
            max_size: be_khai(*flow, cha, *max, auto()),
            // `fill` là "một phần bằng nhau của khoảng TRỐNG", tức `flex-grow`,
            // không phải một bề. Nó nằm ở đây chứ không nằm trong `size`.
            flex_grow: if size.main == Some(Extent::Fill) {
                1.0
            } else {
                0.0
            },
            ..Style::default()
        };
        // Cùng luật với lá: nhóm chỉ vào cây trợ năng khi nút bố cục của nó
        // dựng XONG. Đẩy trước thì một nhóm không xếp được vẫn được đọc lên, và
        // mọi con của nó đi theo.
        let id = self.cay.new_with_children(dang, &con).ok()?;
        access.push(AccessNode {
            role: Role::Group,
            label: None,
            action: None,
            children: con_access,
        });
        Some(id)
    }

    /// Nhánh LÁ của [`Cay::dung`] — tách ra vì hàm kia đã chạm trần độ dài.
    fn dung_la(
        &mut self,
        n: &Node,
        rong_toi_da: f32,
        do_la: &mut dyn FnMut(&Node, f32, &mut Vec<AccessNode>) -> O,
        access: &mut Vec<AccessNode>,
    ) -> Option<NodeId> {
        // ⚠️ Nút trợ năng viết vào chỗ TẠM, chỉ nhập vào cây thật khi nút
        // bố cục dựng XONG.
        //
        // Bản trước đẩy thẳng vào `access` rồi mới dựng nút bố cục. Bước sau
        // hỏng thì nút ấy **có trong cây trợ năng mà không có trong bố cục**:
        // không vẽ ra, chuột không bấm tới, nhưng `bang_hanh_dong_cua` dựng
        // bảng TỪ cây trợ năng nên trục trợ năng vẫn kích hoạt được nó.
        //
        // Cùng hình dạng với F1 (nút vô hình mà `hit_test` vẫn trả về), chỉ
        // soi gương: vô hình với mắt và với chuột, mà trình đọc màn hình vẫn
        // với tới. `check_accessibility_parity` KHÔNG bắt được — nó so cây
        // nguồn với cây công bố, và cả hai đều có nút ấy.
        let mut tam = Vec::new();
        let o = do_la(n, rong_toi_da, &mut tam);
        let dang = Style {
            // Kích thước CỐ ĐỊNH theo số đo. Không giao việc đo cho `taffy`
            // qua hàm đo: đo hai lần với hai bề rộng khác nhau thì một nút
            // có thể đổi số dòng giữa lượt đo và lượt vẽ, và cái vẽ ra
            // không còn khớp cái `hit_test` tin.
            size: Size {
                width: length(o.rong),
                height: length(o.cao),
            },
            ..Style::default()
        };
        self.la.push(o);
        let ngu_canh = self.la.len() - 1;
        let id = self.cay.new_leaf_with_context(dang, ngu_canh).ok()?;
        access.append(&mut tam);
        Some(id)
    }
}

/// Xếp cây `n` vào góc (`trai`, `tren`) trong bề ngang `rong_toi_da`.
///
/// Trả về **chiều cao đã dùng**, và đẩy mọi ô đã đặt vào `ra` theo toạ độ tuyệt
/// đối. Không đặt được thì trả `None` — nơi gọi tự quyết định báo lỗi thế nào.
pub(crate) fn xep(
    n: &Node,
    trai: f32,
    tren: f32,
    rong_toi_da: f32,
    do_la: &mut dyn FnMut(&Node, f32, &mut Vec<AccessNode>) -> O,
    ra: &mut Vec<DaDat>,
    access: &mut Vec<AccessNode>,
) -> Option<f32> {
    let mut cay = Cay {
        cay: TaffyTree::new(),
        la: Vec::new(),
    };
    let goc = cay.dung(n, rong_toi_da, do_la, access, None)?;
    cay.cay
        .compute_layout(
            goc,
            Size {
                width: AvailableSpace::Definite(rong_toi_da),
                height: AvailableSpace::MaxContent,
            },
        )
        .ok()?;

    let mut dat = Vec::new();
    thu_hoach(
        &cay.cay,
        goc,
        trai,
        tren,
        rong_toi_da,
        &mut cay.la,
        &mut dat,
    )?;
    let cao = cay.cay.layout(goc).ok()?.size.height;
    ra.append(&mut dat);
    Some(cao)
}

/// Đi lại cây đã tính, đổi toạ độ tương đối của `taffy` thành tuyệt đối.
fn thu_hoach(
    cay: &TaffyTree<usize>,
    id: NodeId,
    trai: f32,
    tren: f32,
    rong_toi_da: f32,
    la: &mut Vec<O>,
    ra: &mut Vec<DaDat>,
) -> Option<()> {
    let bo_cuc = cay.layout(id).ok()?;
    let x = trai + bo_cuc.location.x;
    let y = tren + bo_cuc.location.y;

    if let Some(&chi_so) = cay.get_node_context(id) {
        // `std::mem::take` chứ không `clone`: mỗi lá xuất hiện đúng một lần
        // trong cây, và lấy hẳn ra thì lần thứ hai lấy nhầm sẽ ra ô rỗng nhìn
        // thấy được, thay vì ra một bản sao im lặng.
        let mut o = std::mem::take(la.get_mut(chi_so)?);
        // Bề rộng cuối do `taffy` chốt, không phải số đo ban đầu: `ve_o` dùng
        // `o.rong` làm chỗ ngắt dòng chữ, nên hai con số này phải là một.
        o.rong = bo_cuc.size.width;
        o.cao = bo_cuc.size.height;
        ra.push(DaDat {
            o,
            trai: x,
            tren: y,
        });
        return Some(());
    }

    let con = cay.children(id).ok()?;
    // Mốc để biết những ô nào là của RIÊNG nhóm này. Kéo nút bằng nhau phải
    // giới hạn trong một nhóm: hai nhóm anh em không bao giờ chung dòng, và gom
    // nhầm chúng lại là xếp hai dòng đè lên nhau.
    let bat_dau = ra.len();
    let toan_la = con.iter().all(|&c| cay.get_node_context(c).is_some());
    for c in con {
        thu_hoach(cay, c, x, y, rong_toi_da, la, ra)?;
    }
    if toan_la {
        rong_bang_nhau(ra.get_mut(bat_dau..)?, rong_toi_da);
    }
    Some(())
}

/// **Nút trên cùng một dòng phải rộng bằng nhau** — và chỉ khi kéo xong vẫn vừa.
///
/// Đây không phải chuyện thẩm mỹ. Màn xác nhận giao dịch cố ý cho hai nút CÙNG
/// sắc thái, vì làm nút "Ký" nổi hơn là đẩy người dùng về một phía đúng lúc nguy
/// hiểm nhất. Nhưng bề rộng cũng đẩy: một nút to hơn hẳn nút kia vẫn là một cái
/// hích, chỉ bằng hình học thay vì bằng màu.
///
/// ⚠️ CHỈ kéo khi kéo xong VẪN VỪA. Kéo vô điều kiện thì một hàng "vừa" bị nới
/// quá lề và những ô sau trôi ra ngoài ảnh: đo được ngày 21/08/2026 là một nút
/// nằm ở 681,8→1008,7 trên ảnh rộng 640 — **không một điểm ảnh nào được vẽ**, mà
/// `hit_test` vẫn trả về nó. Người dùng bấm vào khoảng trắng và một nút họ chưa
/// từng thấy chạy. Không vừa thì thà để bề rộng tự nhiên: một hàng nút không đều
/// đẹp hơn một nút vô hình bấm được.
///
/// Chạy SAU khi `taffy` tính xong, vì "cùng một dòng" chỉ biết được sau khi biết
/// chỗ nào xuống dòng.
fn rong_bang_nhau(dat: &mut [DaDat], rong_toi_da: f32) {
    let mut i = 0;
    while i < dat.len() {
        // Một DÒNG kết thúc ở chỗ mép trái TỤT VỀ — đó là chỗ `taffy` xuống
        // dòng.
        //
        // ⚠️ KHÔNG nhận dòng bằng "cùng mép trên". Hàng căn giữa theo chiều
        // dọc, nên một nút cao và một nút thấp CÙNG DÒNG có mép trên KHÁC NHAU,
        // còn hai ô ở hai dòng khác nhau thì mép trên có thể trùng nhau. Bản
        // đầu làm thế và `khong_bao_gio_co_o_chong_len_nhau` bắt được ngay ở
        // hạt 28: hai dòng bị gom làm một rồi xếp lại thành hàng, ô nọ đè ô kia.
        let mut j = i + 1;
        while j < dat.len() && dat[j].trai > dat[j - 1].trai {
            j += 1;
        }
        let dong = &mut dat[i..j];
        i = j;

        // Chỉ áp cho dòng TOÀN nút. Một nút cạnh một nhãn thì kéo bằng nhau là
        // vô nghĩa.
        if dong.len() < 2 || !dong.iter().all(|d| d.o.co_khung()) {
            continue;
        }
        let rong_nhat = dong.iter().fold(0.0f32, |a, d| a.max(d.o.rong));
        let khe = (dong[1].trai - dong[0].trai - dong[0].o.rong).max(0.0);
        #[expect(clippy::cast_precision_loss, reason = "số ô trong một dòng, luôn nhỏ")]
        let tong = rong_nhat.mul_add(dong.len() as f32, khe * (dong.len() - 1) as f32);
        if tong > rong_toi_da {
            continue;
        }
        // Kéo bằng nhau thì phải đẩy lại chỗ: giữ nguyên mép trái dòng.
        let mut x = dong[0].trai;
        for d in dong.iter_mut() {
            d.o.rong = rong_nhat;
            d.trai = x;
            x += rong_nhat + khe;
        }
    }
}
