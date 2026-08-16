//! **Đâm thử 0.1** — chữ tiếng Việt dựng bằng Rust có dùng được không?
//!
//! ```text
//! cargo run --release
//! ```
//!
//! # Câu hỏi thật, và nó KHÔNG phải về GPU
//!
//! Kế hoạch viết câu 0.1 là *"chữ dựng bằng wgpu có đẹp bằng hệ điều hành
//! không?"*. Đọc kỹ thì wgpu không phải chỗ rủi ro: GPU chỉ dán một tấm ảnh
//! chữ đã rasterize lên màn hình, và việc ấy không có gì để hỏng theo kiểu
//! "chữ xấu".
//!
//! Rủi ro nằm ở **hai bước trước đó**, và cả hai đều chạy trên CPU:
//!
//! 1. **Shaping** — xếp dấu tiếng Việt vào đúng chỗ. `ế` là `e` + dấu mũ + dấu
//!    sắc, và dấu sắc phải nằm TRÊN dấu mũ, hơi lệch phải, không đè lên nhau.
//! 2. **Rasterize** — vẽ ra pixel với hinting và khử răng cưa đủ tốt ở cỡ chữ
//!    người ta đọc cả ngày.
//!
//! Nên đâm thử này đo hai bước ấy, **không đụng tới wgpu**. Nếu chúng hỏng thì
//! GPU nhanh đến đâu cũng vô nghĩa; nếu chúng tốt thì phần GPU là việc kỹ thuật
//! thường, không phải câu hỏi khả thi.
//!
//! ⚠️ Máy đo là **Intel Mac, Iris Plus 645**. Kế hoạch đã dặn đừng kết luận
//! hiệu năng đồ hoạ trên máy này — nên ở đây **không đo tốc độ**, chỉ đo tính
//! đúng và chất lượng hình.

use cosmic_text::{Attrs, Buffer, Color, Family, FontSystem, Metrics, Shaping, SwashCache};

/// Câu đã dùng ở cổng gõ tiếng Việt của Giai đoạn 1 — giữ nguyên để so được.
const CAU: &str = "Chào buổi sáng mọi người";

/// Chữ khó nhất: **dấu chồng hai tầng**. Mỗi chữ ở đây là nguyên âm + dấu phụ
/// (mũ/móc/trăng) + dấu thanh. Đây là chỗ bộ shaping hỏng nếu nó có hỏng.
const DAU_CHONG: &str = "ế ề ể ễ ệ ố ồ ổ ỗ ộ ữ ự ợ ằ ẵ ặ";

/// Cỡ chữ người ta đọc cả ngày. Đo ở cỡ to thì mọi bộ dựng đều đẹp.
const CO_CHU: f32 = 15.0;

fn main() {
    let mut fonts = FontSystem::new();
    let mut cache = SwashCache::new();

    println!("=== ĐÂM THỬ 0.1 — chữ tiếng Việt ===");
    println!();

    let mut hong = 0;
    for (ten, chu) in [("câu thường", CAU), ("dấu chồng", DAU_CHONG)] {
        hong += do_mot_dong(&mut fonts, &mut cache, ten, chu);
    }

    // Phép so quyết định: dạng DỰNG SẴN và dạng TỔ HỢP phải ra cùng hình.
    //
    // Bộ gõ macOS phát ra dạng dựng sẵn (`ế` = U+1EBF, một mã điểm). Trang web
    // và tệp văn bản thường mang dạng tổ hợp (`e` + U+0302 + U+0301). Nếu hai
    // dạng ra hai hình khác nhau thì cùng một câu trông khác nhau tuỳ nó tới từ
    // đâu — và người dùng không có cách nào biết vì sao.
    hong += so_hai_dang(&mut fonts, &mut cache);

    // Lỗi kinh điển nhất của tiếng Việt: dấu bị CẮT CỤT ở mép trên vì chiều cao
    // dòng tính theo chữ Latin không dấu. Nhìn ảnh thì khó thấy — phải đo.
    hong += do_dau_bi_cat(&mut fonts, &mut cache);

    println!();
    if hong == 0 {
        println!("✅ 0.1 ĐẠT ở phần shaping — xem PNG để xét chất lượng hình.");
    } else {
        println!("❌ 0.1 HỎNG: {hong} phép đo không đạt.");
    }
    println!();
    println!("Ảnh: dam-thu/chu-viet/ra/*.png — mở lên xem mới kết luận được về HÌNH.");
    println!("Số ở trên chỉ chứng minh shaping đúng, không chứng minh chữ đẹp.");
}

fn do_mot_dong(
    fonts: &mut FontSystem,
    cache: &mut SwashCache,
    ten: &str,
    chu: &str,
) -> usize {
    let mut buffer = Buffer::new(fonts, Metrics::new(CO_CHU, CO_CHU * 1.4));
    let mut b = buffer.borrow_with(fonts);
    b.set_size(Some(600.0), Some(80.0));
    // `Shaping::Advanced` — bắt buộc. `Basic` bỏ qua việc xếp dấu phụ, và với
    // tiếng Việt thì đó không phải "nhanh hơn một chút", đó là SAI.
    b.set_text(chu, &Attrs::new().family(Family::SansSerif), Shaping::Advanced, None);
    b.shape_until_scroll(true);

    let mut so_glyph = 0;
    let mut thieu_font = 0;
    for run in b.layout_runs() {
        for g in run.glyphs {
            so_glyph += 1;
            // `glyph_id == 0` là `.notdef` — ô vuông rỗng. Một cái thôi cũng đủ
            // để kết luận font dự phòng không phủ hết tiếng Việt.
            if g.glyph_id == 0 {
                thieu_font += 1;
            }
        }
    }

    let so_ky_tu = chu.chars().filter(|c| !c.is_whitespace()).count();
    println!("── {ten} ──");
    println!("  ký tự (bỏ khoảng trắng): {so_ky_tu}");
    println!("  glyph shaping ra       : {so_glyph}");
    println!("  glyph .notdef          : {thieu_font}");

    let mut hong = 0;
    if thieu_font > 0 {
        println!("  ❌ có {thieu_font} ô vuông rỗng — font dự phòng không phủ tiếng Việt");
        hong += 1;
    }

    ve_png(fonts, cache, chu, &format!("{}.png", ten.replace(' ', "-")));
    hong
}

/// Dạng dựng sẵn và dạng tổ hợp phải ra **cùng một hình**.
fn so_hai_dang(fonts: &mut FontSystem, cache: &mut SwashCache) -> usize {
    // "ế" dựng sẵn (U+1EBF) vs tổ hợp (e + U+0302 + U+0301).
    let dung_san = "ế";
    let to_hop = "e\u{0302}\u{0301}";
    assert_ne!(dung_san, to_hop, "hai chuỗi phải khác nhau ở mức byte");

    let a = anh(fonts, cache, dung_san);
    let b = anh(fonts, cache, to_hop);

    println!("── dựng sẵn vs tổ hợp ──");
    println!("  U+1EBF          : {} byte, {} mã điểm", dung_san.len(), dung_san.chars().count());
    println!("  e + U+0302+0301 : {} byte, {} mã điểm", to_hop.len(), to_hop.chars().count());

    let khac = a.iter().zip(b.iter()).filter(|(x, y)| x != y).count();
    let ti_le = if a.is_empty() {
        1.0
    } else {
        khac as f64 / a.len() as f64
    };
    println!("  pixel khác nhau : {khac}/{} ({:.2}%)", a.len(), ti_le * 100.0);

    if a.len() != b.len() {
        println!("  ❌ hai ảnh khác kích thước — không so được, đã là dấu hiệu xấu");
        return 1;
    }
    // Ngưỡng 1%: khử răng cưa có thể lệch vài pixel biên, nhưng hình phải TRÙNG.
    if ti_le > 0.01 {
        println!("  ❌ hai dạng ra HAI HÌNH khác nhau — cùng một câu trông khác nhau");
        println!("     tuỳ nó tới từ bộ gõ hay từ tệp văn bản");
        return 1;
    }
    println!("  ✅ hai dạng ra cùng một hình");
    0
}

/// **Dấu có bị cắt cụt ở mép trên không?**
///
/// Đây là lỗi phổ biến nhất khi dựng tiếng Việt: chiều cao dòng lấy theo chữ
/// Latin không dấu, rồi `ế` với hai tầng dấu chạm trần và bị xén. Nó không lộ
/// ra ở chữ thường, chỉ lộ ở đúng những chữ khó nhất — nên nhìn qua thì tưởng
/// ổn.
///
/// Đo bằng cách vẽ vào một khung CAO GẤP BA rồi hỏi: mực chữ có tràn lên trên
/// vạch đỉnh dòng mà `Metrics` hứa không.
fn do_dau_bi_cat(fonts: &mut FontSystem, cache: &mut SwashCache) -> usize {
    const W: usize = 400;
    const H: usize = 150;
    const DEM: f32 = 50.0; // vẽ tụt xuống, để chỗ trống bên trên mà đo

    let mut tren_cung = i32::MAX;
    let mut duoi_cung = i32::MIN;

    let mut buffer = Buffer::new(fonts, Metrics::new(CO_CHU, CO_CHU * 1.4));
    let mut b = buffer.borrow_with(fonts);
    b.set_size(Some(W as f32), Some(H as f32));
    b.set_text(DAU_CHONG, &Attrs::new().family(Family::SansSerif), Shaping::Advanced, None);
    b.shape_until_scroll(true);
    b.draw(cache, Color::rgb(0, 0, 0), |_x, y, _w, h, mau| {
        if mau.a() == 0 {
            return;
        }
        tren_cung = tren_cung.min(y);
        duoi_cung = duoi_cung.max(y + h as i32);
    });

    let cao_dong = CO_CHU * 1.4;
    let cao_muc = duoi_cung - tren_cung;

    println!("── dấu có bị cắt không ──");
    println!("  chiều cao dòng Metrics hứa : {cao_dong:.1} px");
    println!("  chiều cao mực chữ thật     : {cao_muc} px (từ {tren_cung} tới {duoi_cung})");
    let _ = DEM;

    if tren_cung < 0 {
        println!("  ❌ mực chữ TRÀN LÊN TRÊN mép khung {} px — dấu sẽ bị xén", -tren_cung);
        return 1;
    }
    if f64::from(cao_muc) > f64::from(cao_dong) {
        println!(
            "  ⚠️ mực chữ CAO HƠN chiều cao dòng {:.1} px — hai dòng liền nhau sẽ chồng dấu",
            f64::from(cao_muc) - f64::from(cao_dong)
        );
        return 1;
    }
    println!("  ✅ mực chữ nằm gọn trong chiều cao dòng");
    0
}

/// Rasterize thành ảnh xám, trả về mảng pixel.
fn anh(fonts: &mut FontSystem, cache: &mut SwashCache, chu: &str) -> Vec<u8> {
    const W: usize = 64;
    const H: usize = 40;
    let mut pixel = vec![0u8; W * H];

    let mut buffer = Buffer::new(fonts, Metrics::new(CO_CHU * 2.0, CO_CHU * 2.4));
    let mut b = buffer.borrow_with(fonts);
    b.set_size(Some(W as f32), Some(H as f32));
    b.set_text(chu, &Attrs::new().family(Family::SansSerif), Shaping::Advanced, None);
    b.shape_until_scroll(true);
    b.draw(cache, Color::rgb(255, 255, 255), |x, y, w, h, mau| {
        for dy in 0..h {
            for dx in 0..w {
                let px = x + dx as i32;
                let py = y + dy as i32;
                if px >= 0 && py >= 0 && (px as usize) < W && (py as usize) < H {
                    let i = py as usize * W + px as usize;
                    // Lấy kênh alpha làm độ đậm — đủ để so hai hình.
                    pixel[i] = pixel[i].max(mau.a());
                }
            }
        }
    });
    pixel
}

fn ve_png(fonts: &mut FontSystem, cache: &mut SwashCache, chu: &str, ten_tep: &str) {
    const W: usize = 620;
    const H: usize = 60;
    // Nền trắng, chữ đen: đúng cách người ta đọc, và là cách lộ khử răng cưa xấu
    // rõ nhất.
    let mut pixel = vec![255u8; W * H];

    let mut buffer = Buffer::new(fonts, Metrics::new(CO_CHU, CO_CHU * 1.4));
    let mut b = buffer.borrow_with(fonts);
    b.set_size(Some(W as f32), Some(H as f32));
    b.set_text(chu, &Attrs::new().family(Family::SansSerif), Shaping::Advanced, None);
    b.shape_until_scroll(true);
    b.draw(cache, Color::rgb(0, 0, 0), |x, y, w, h, mau| {
        for dy in 0..h {
            for dx in 0..w {
                let px = x + dx as i32;
                let py = y + dy as i32;
                if px >= 0 && py >= 0 && (px as usize) < W && (py as usize) < H {
                    let i = py as usize * W + px as usize;
                    let a = f32::from(mau.a()) / 255.0;
                    pixel[i] = (f32::from(pixel[i]) * (1.0 - a)) as u8;
                }
            }
        }
    });

    let _ = std::fs::create_dir_all("ra");
    let Ok(tep) = std::fs::File::create(format!("ra/{ten_tep}")) else {
        eprintln!("không ghi được ra/{ten_tep}");
        return;
    };
    let mut enc = png::Encoder::new(std::io::BufWriter::new(tep), W as u32, H as u32);
    enc.set_color(png::ColorType::Grayscale);
    enc.set_depth(png::BitDepth::Eight);
    if let Ok(mut w) = enc.write_header() {
        let _ = w.write_image_data(&pixel);
    }
}
