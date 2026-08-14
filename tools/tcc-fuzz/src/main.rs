//! Bộ fuzz cho các bộ PHÂN TÍCH — đầu vào chưa xác thực đầu tiên mã ta chạm tới.
//!
//! # Vì sao đây là chỗ đáng fuzz nhất
//!
//! Trong `verify_package`, thứ tự là:
//!
//! ```text
//! 0. chặn trần kích thước
//! 1. serde_json::from_slice::<Manifest>   ← đây
//! 2. manifest.validate_shape()            ← và đây
//! 3. so tên bộ ký
//! 4. KIỂM CHỮ KÝ                          ← mãi tới đây mới xác thực
//! ```
//!
//! Bước 1 và 2 **bắt buộc** phải đứng trước bước 4: khoá công khai nằm trong
//! chính bản kê khai, nên không đọc nó ra thì không biết lấy khoá nào mà kiểm.
//! Hệ quả không tránh được: **bộ phân tích chạy trên dữ liệu hoàn toàn chưa xác
//! thực.** Một kẻ tấn công không cần chữ ký hợp lệ, không cần khoá, không cần
//! gì — chỉ cần đưa được một tệp tới chỗ ta.
//!
//! Nên nếu có đúng một chỗ trong kho này đáng fuzz, nó là chỗ này.
//!
//! # Bộ này KHÔNG dẫn hướng theo độ phủ
//!
//! Nó đột biến ngẫu nhiên từ một tập hạt giống, không đo nhánh nào đã chạy qua.
//! libFuzzer/AFL mạnh hơn hẳn ở việc lần vào nhánh sâu. Đổi lại, bộ này chạy
//! trên Rust ổn định, không thêm phụ thuộc, tất định theo hạt giống nên tái
//! hiện được, và **chạy được trong CI**. Ghi ra đây để không ai đọc nhầm nó
//! thành thứ nó không phải.
//!
//! Chạy:
//! ```sh
//! cargo run -p tcc-fuzz                 # 20000 vòng, hạt giống 1
//! cargo run -p tcc-fuzz -- 200000 42    # số vòng, hạt giống
//! ```

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "công cụ dòng lệnh: in ra là việc của nó"
)]

use std::{panic, process::ExitCode};

mod corpus;
mod mutation;

use mutation::Rng;
use tcc_crypto::{HybridEd25519MlDsa, SignatureScheme as _};

/// Một mục tiêu fuzz: tên, và cái để chạy trên một chuỗi byte.
struct Target {
    ten: &'static str,
    chay: fn(&[u8]) -> Result<(), String>,
    /// Hạt giống của RIÊNG mục tiêu này. Trộn chung thì mục tiêu chữ ký chỉ ăn
    /// toàn JSON — mà JSON thì trượt ngay ở phép kiểm độ dài, không bao giờ
    /// chạm tới mã mật mã, tức là đo hư không.
    seeds: fn(&corpus::Corpus) -> &[Vec<u8>],
    /// Số vòng chia cho con số này. Kiểm một chữ ký ML-DSA đắt hơn phân tích
    /// một tệp JSON khoảng ba bậc độ lớn, nên cho cùng ngân sách vòng là biến
    /// cả lượt chạy thành một phép đo tốc độ mật mã. Ngân sách phải theo THỜI
    /// GIAN, không theo số vòng.
    chi_phi: u64,
    /// Đầu vào có ĐỘ DÀI CỐ ĐỊNH không? Nếu có thì đột biến phải giữ nguyên độ
    /// dài, nếu không mọi đầu vào đều chết ở cổng kiểm độ dài.
    co_dinh: bool,
    /// Đầu vào này có đi được vào PHẦN SÂU không (qua bộ đọc JSON, tới các phép
    /// kiểm thật)? Không đo cái này thì "0 lỗi" là một con số vô nghĩa: một bộ
    /// fuzz nảy hết ở dấu ngoặc đầu tiên cũng báo 0 lỗi y hệt một bộ fuzz tốt.
    sau: fn(&[u8]) -> bool,
}

fn deep_manifest(b: &[u8]) -> bool {
    serde_json::from_slice::<tcc_spec::Manifest>(b).is_ok()
}
fn deep_ui_tree(b: &[u8]) -> bool {
    serde_json::from_slice::<tcc_ui::wire::UiNode>(b).is_ok()
}
/// Kiểm CHỮ KÝ với byte chữ ký do fuzz sinh ra.
///
/// Đây là chỗ `ml-dsa` 0.1.1 — thư viện CHƯA có kiểm định độc lập nào được công
/// bố, xem `SECURITY.md` §3.2 — phân tích byte hoàn toàn do kẻ tấn công điều
/// khiển. Nó là mã mật mã của bên thứ ba đứng ngay trên đường đi của dữ liệu
/// chưa xác thực.
///
/// Đòi hỏi nặng hơn "không hoảng loạn": **không một đột biến nào của chữ ký thật
/// được phép kiểm ĐẠT.** Đột biến ra chữ ký hợp lệ nghĩa là giả mạo được.
fn target_signature(byte: &[u8]) -> Result<(), String> {
    let (khoa, chu_ky_that) = corpus::real_pair();
    let thong_diep = corpus::signed_message();
    let ket = HybridEd25519MlDsa.verify(&khoa, &thong_diep, byte);
    if ket.is_ok() && byte != chu_ky_that.as_slice() {
        return Err("một chữ ký KHÁC bản thật lại kiểm ĐẠT — giả mạo được".to_owned());
    }
    // Kiểm hai lần phải ra một kết quả: phán quyết mật mã không được phụ thuộc
    // thứ gì ngoài ba đầu vào.
    if HybridEd25519MlDsa.verify(&khoa, &thong_diep, byte).is_ok() != ket.is_ok() {
        return Err("kiểm hai lần ra HAI phán quyết".to_owned());
    }
    Ok(())
}

/// Kiểm chữ ký với KHOÁ CÔNG KHAI do fuzz sinh ra.
///
/// Khoá công khai đến thẳng từ bản kê khai của kẻ tấn công (`hex::decode` rồi
/// đưa vào thư viện), nên nó cũng là byte chưa xác thực đi vào mã mật mã.
fn target_public_key(byte: &[u8]) -> Result<(), String> {
    let (khoa_that, chu_ky) = corpus::real_pair();
    let thong_diep = corpus::signed_message();
    if HybridEd25519MlDsa
        .verify(byte, &thong_diep, &chu_ky)
        .is_ok()
        && byte != khoa_that.as_slice()
    {
        return Err("một khoá công khai KHÁC lại kiểm ĐẠT cùng chữ ký đó".to_owned());
    }
    Ok(())
}

fn deep_signature(b: &[u8]) -> bool {
    // "Sâu" ở đây nghĩa là qua được phép kiểm độ dài, tức là thật sự chạm vào
    // mã mật mã chứ không bị chặn ở cổng.
    b.len() == 3373
}
fn deep_public_key(b: &[u8]) -> bool {
    b.len() == 1984
}

fn deep_file_tree(b: &[u8]) -> bool {
    core::str::from_utf8(b).is_ok_and(|s| {
        let mut c = tcc_spec::tree::FileTree::new();
        s.split('\n').take(32).any(|p| c.insert(p, vec![0]).is_ok())
    })
}

/// Phân tích bản kê khai — đường đi của dữ liệu CHƯA xác thực.
///
/// Không chỉ đòi "không hoảng loạn". Nếu bản kê khai được NHẬN, nó còn phải
/// **quay vòng được**: viết ra rồi đọc lại phải ra đúng giá trị cũ. Nhận vào một
/// thứ mà chính mình không tái tạo nổi là một dạng hỏng âm thầm — nó nghĩa là
/// có trạng thái sống sót qua bước kiểm mà không ai mô tả được.
fn target_manifest(byte: &[u8]) -> Result<(), String> {
    let Ok(m) = serde_json::from_slice::<tcc_spec::Manifest>(byte) else {
        return Ok(()); // từ chối là kết quả ĐÚNG, không phải lỗi
    };
    if m.validate_shape().is_err() {
        return Ok(());
    }

    let lai =
        serde_json::to_vec(&m).map_err(|e| format!("nhận rồi mà không viết lại được: {e}"))?;
    let m2 = serde_json::from_slice::<tcc_spec::Manifest>(&lai)
        .map_err(|e| format!("viết ra rồi đọc lại không được: {e}"))?;
    if m != m2 {
        return Err("quay vòng ra giá trị KHÁC".to_owned());
    }
    // Đã nhận lần một thì phải nhận lần hai: phép kiểm hình dạng không được
    // phụ thuộc thứ gì ngoài chính giá trị.
    m2.validate_shape()
        .map_err(|e| format!("nhận lần đầu, từ chối lần sau: {e}"))
}

/// Cây giao diện — cũng đến từ gói, cũng chưa xác thực lúc đọc.
///
/// Đòi hỏi ở đây là **viết ra rồi đọc lại không được đổi phán quyết**. Đó chính
/// là lớp lỗi B16/L8: dạng trên dây (`UiNode`) và dạng đã kiểm (`Node`) là hai
/// kiểu khác nhau, và chỗ nguy hiểm nằm ở chỗ nối giữa chúng. Nhận một cây mà
/// tự mình viết ra lại không đọc nổi nghĩa là có trạng thái sống sót qua bước
/// kiểm mà dạng trên dây không mô tả được — hai bản cài đặt sẽ bất đồng.
fn target_ui_tree(byte: &[u8]) -> Result<(), String> {
    let Ok(cay) = tcc_ui::wire::decode(byte) else {
        return Ok(());
    };
    let Ok(day) = serde_json::from_slice::<tcc_ui::wire::UiNode>(byte) else {
        return Err("`decode` NHẬN nhưng dạng trên dây lại không đọc được".to_owned());
    };
    let lai = serde_json::to_vec(&day).map_err(|e| format!("không viết lại được: {e}"))?;
    match tcc_ui::wire::decode(&lai) {
        Ok(cay2) if cay2 == cay => Ok(()),
        Ok(_) => Err("quay vòng ra cây KHÁC".to_owned()),
        Err(e) => Err(format!("viết ra rồi đọc lại bị TỪ CHỐI: {e}")),
    }
}

/// Đường dẫn tệp trong gói — nơi `..`, ổ đĩa Windows và trùng hoa thường bị chặn.
///
/// Thêm một đòi hỏi nữa: **dạng chuẩn tắc phải tất định**. Cùng một cây dựng
/// hai lần phải cho cùng một chuỗi byte, nếu không thì hai bản cài đặt băm ra
/// hai kết quả và chữ ký của bên này bên kia không kiểm được.
fn target_file_tree(byte: &[u8]) -> Result<(), String> {
    let Ok(s) = core::str::from_utf8(byte) else {
        return Ok(());
    };
    let mut cay = tcc_spec::tree::FileTree::new();
    for (i, phan) in s.split('\n').take(32).enumerate() {
        let _ = cay.insert(phan, vec![u8::try_from(i & 0xff).unwrap_or(0)]);
    }
    let a = cay.canonical_bytes();
    let b = cay.canonical_bytes();
    if a == b {
        Ok(())
    } else {
        Err("dạng chuẩn tắc KHÔNG tất định".to_owned())
    }
}

const TARGETS: &[Target] = &[
    Target {
        ten: "ke-khai",
        chay: target_manifest,
        co_dinh: false,
        chi_phi: 1,
        sau: deep_manifest,
        seeds: |k| &k.text,
    },
    Target {
        ten: "cay-giao-dien",
        chay: target_ui_tree,
        co_dinh: false,
        chi_phi: 1,
        sau: deep_ui_tree,
        seeds: |k| &k.text,
    },
    Target {
        ten: "cay-tep",
        chay: target_file_tree,
        co_dinh: false,
        chi_phi: 1,
        sau: deep_file_tree,
        seeds: |k| &k.text,
    },
    Target {
        ten: "chu-ky",
        chay: target_signature,
        co_dinh: true,
        chi_phi: 20,
        sau: deep_signature,
        seeds: |k| &k.signature,
    },
    Target {
        ten: "khoa-cong-khai",
        chay: target_public_key,
        co_dinh: true,
        chi_phi: 20,
        sau: deep_public_key,
        seeds: |k| &k.public_key,
    },
];

fn main() -> ExitCode {
    let mut arg = std::env::args().skip(1);
    let so_vong: u64 = arg.next().and_then(|s| s.parse().ok()).unwrap_or(20_000);
    let hat: u64 = arg.next().and_then(|s| s.parse().ok()).unwrap_or(1);

    // Bộ fuzz cố ý nuốt tiếng hoảng loạn: nếu không, dòng in ra lẫn với hàng
    // nghìn vòng chạy và không ai tìm được đầu vào gây ra nó.
    let cu = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    let kho = corpus::load();
    println!(
        "hạt giống: {} văn bản + {} chữ ký + {} khoá · {so_vong} vòng/mục tiêu · gieo {hat}",
        kho.text.len(),
        kho.signature.len(),
        kho.public_key.len()
    );

    let mut hong = 0usize;
    for mt in TARGETS {
        let mut nhieu = Rng::new(hat);
        let mut so_nhan = 0usize;
        let mut so_sau = 0usize;
        let vong_muc_tieu = (so_vong / mt.chi_phi).max(1);
        for _ in 0..vong_muc_tieu {
            let hg = (mt.seeds)(&kho);
            let goc = &hg[usize::try_from(nhieu.next_u64() % 1024).unwrap_or(0) % hg.len()];
            let dau_vao = if mt.co_dinh {
                mutation::mutate_fixed(&mut nhieu, goc)
            } else {
                mutation::mutate(&mut nhieu, goc)
            };

            let ket = panic::catch_unwind(panic::AssertUnwindSafe(|| (mt.chay)(&dau_vao)));
            let loi = match ket {
                Err(_) => Some("HOẢNG LOẠN".to_owned()),
                Ok(Err(e)) => Some(e),
                Ok(Ok(())) => None,
            };
            if let Some(e) = loi {
                hong += 1;
                println!("\n✗ [{}] {e}", mt.ten);
                println!(
                    "   đầu vào ({} byte): {:?}",
                    dau_vao.len(),
                    Excerpt(&dau_vao)
                );
                if hong >= 5 {
                    break;
                }
            } else if !dau_vao.is_empty() {
                so_nhan += 1;
                if (mt.sau)(&dau_vao) {
                    so_sau += 1;
                }
            }
        }
        let ty_le = if so_nhan == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss, reason = "chỉ để in ra một tỷ lệ")]
            {
                so_sau as f64 * 100.0 / so_nhan as f64
            }
        };
        println!(
            "  {:<14} {so_nhan} vòng · {so_sau} vào được phần sâu ({ty_le:.1}%)",
            mt.ten
        );
        if so_sau == 0 {
            println!("     ⚠️ KHÔNG đầu vào nào đi qua bộ đọc — bộ fuzz này đang đo hư không");
        }
    }

    panic::set_hook(cu);

    if hong == 0 {
        println!("\n✓ ĐẠT — không mục tiêu nào hoảng loạn hay tự mâu thuẫn.");
        ExitCode::SUCCESS
    } else {
        println!("\n✗ {hong} lần hỏng. Chạy lại với đúng số vòng và hạt giống để tái hiện.");
        ExitCode::FAILURE
    }
}

/// In gọn đầu vào: đủ để tái hiện, không tràn màn hình.
struct Excerpt<'a>(&'a [u8]);
impl core::fmt::Debug for Excerpt<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let n = self.0.len().min(200);
        write!(f, "{}", String::from_utf8_lossy(&self.0[..n]))?;
        if self.0.len() > n {
            write!(f, "…(còn {} byte)", self.0.len() - n)?;
        }
        Ok(())
    }
}
