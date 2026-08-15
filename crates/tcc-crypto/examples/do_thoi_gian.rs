//! Đo thời gian `verify` và `sign` — đóng mục `SECURITY.md` §3.4.
//!
//! Chạy: `cargo run -p tcc-crypto --release --example do_thoi_gian`
//!
//! # Công cụ này chứng minh được gì, và KHÔNG chứng minh được gì
//!
//! Nó đo trên một máy tính đang chạy hệ điều hành đầy đủ, có turbo, có điều
//! tiết nhiệt, có tiến trình khác. Một khác biệt NHỎ ở đây không kết luận được
//! gì cả — nhiễu che mất. Một khác biệt LỚN thì kết luận được, và đó chính là
//! thứ đáng tìm: kênh biên đáng lo là kênh đo được từ xa, mà từ xa thì chỉ
//! khác biệt lớn mới sống sót qua đường truyền.
//!
//! Nói cách khác: đây là phép thử SÀNG, không phải chứng minh thời gian hằng.
//! Chứng minh thật cần đo trên phần cứng yên tĩnh, hoặc đếm lệnh bằng công cụ
//! phân tích tĩnh. Ghi ra để không ai đọc kết quả này thành thứ nó không phải.

#![allow(
    clippy::expect_used,
    clippy::print_stdout,
    clippy::cast_precision_loss,
    reason = "công cụ đo chạy tay: hỏng thì nổ ngay, và nó phải in số ra"
)]

use std::time::{Duration, Instant};

use tcc_crypto::{HybridEd25519MlDsa, SignatureScheme as _};

const LAN: usize = 300;

fn do_lap(mut f: impl FnMut()) -> (Duration, Duration) {
    // Chạy nóng trước: lần đầu luôn đắt vì bộ nhớ đệm còn lạnh.
    for _ in 0..20 {
        f();
    }
    let mut mau: Vec<Duration> = Vec::with_capacity(LAN);
    for _ in 0..LAN {
        let t = Instant::now();
        f();
        mau.push(t.elapsed());
    }
    mau.sort_unstable();
    (mau[LAN / 2], mau[LAN / 20]) // trung vị, và bách phân vị 5 (ít nhiễu nhất)
}

fn main() {
    let bo_ky = HybridEd25519MlDsa;
    let bi_mat: Vec<u8> = (0..64u8).collect();
    let cong_khai = HybridEd25519MlDsa::public_from_secret(&bi_mat).expect("suy khoá hỏng");
    let thong_diep = b"TCC timing probe";
    let chu_ky = bo_ky.sign(&bi_mat, thong_diep).expect("ký hỏng");

    let mut hong_ed = chu_ky.clone();
    hong_ed[0] ^= 1; // lật một bit trong nửa cổ điển
    let mut hong_pq = chu_ky.clone();
    hong_pq[100] ^= 1; // lật một bit trong nửa hậu lượng tử

    println!("── kiểm chữ ký ({LAN} lần mỗi loại, in trung vị / bách phân vị 5)\n");

    let (dung, dung5) = do_lap(|| {
        let _ = bo_ky.verify(&cong_khai, thong_diep, &chu_ky);
    });
    let (ed, ed5) = do_lap(|| {
        let _ = bo_ky.verify(&cong_khai, thong_diep, &hong_ed);
    });
    let (pq, pq5) = do_lap(|| {
        let _ = bo_ky.verify(&cong_khai, thong_diep, &hong_pq);
    });

    println!("  chữ ký ĐÚNG              {dung:>12.2?}   {dung5:>12.2?}");
    println!("  nửa CỔ ĐIỂN hỏng         {ed:>12.2?}   {ed5:>12.2?}");
    println!("  nửa HẬU LƯỢNG TỬ hỏng    {pq:>12.2?}   {pq5:>12.2?}");

    let ty = ed.as_secs_f64() / pq.as_secs_f64();
    println!("\n  cổ điển hỏng / hậu lượng tử hỏng = {ty:.2}×");
    if (0.5..2.0).contains(&ty) {
        println!("  hai loại hỏng tốn thời gian tương đương — không đọc ra được nửa nào hỏng.");
    } else {
        println!(
            "  ⚠️ KHÁC BIỆT LỚN: đo thời gian là biết NỬA NÀO hỏng.\n     \
             `verify` dừng sớm ở nửa cổ điển, nên nửa hậu lượng tử không chạy."
        );
    }

    println!("\n── ký, hai khoá bí mật khác nhau\n");
    let khoa_khong: Vec<u8> = vec![0u8; 64];
    let khoa_ff: Vec<u8> = vec![0xffu8; 64];
    let (k0, _) = do_lap(|| {
        let _ = bo_ky.sign(&khoa_khong, thong_diep);
    });
    let (kf, _) = do_lap(|| {
        let _ = bo_ky.sign(&khoa_ff, thong_diep);
    });
    println!("  khoá toàn 0x00           {k0:>12.2?}");
    println!("  khoá toàn 0xff           {kf:>12.2?}");
    let ty2 = k0.as_secs_f64() / kf.as_secs_f64();
    println!("  tỷ lệ = {ty2:.3}×  (gần 1.0 là dấu hiệu tốt, KHÔNG phải bằng chứng)");
}
