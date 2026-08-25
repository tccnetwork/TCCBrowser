//! Bố cục gói trên đĩa, và cách đọc một thư mục thành `FileTree`.
//!
//! Đóng gói KHÔNG phải việc của `tcc-manifest` — crate đó chỉ nhận byte và cây
//! tệp, nên kiểm thử được mà không đụng đĩa. Mọi thứ liên quan hệ thống tệp nằm
//! ở đây.
//!
//! # Vì sao ở `tcc-runtime` chứ không ở `tcc-cli`
//!
//! Ban đầu tệp này nằm trong `tools/tcc-cli`. Nhưng thư viện không dùng lại được
//! mã trong một crate nhị phân, nên khi `tcc-shell` cần nạp gói từ đĩa thì chỉ
//! có hai lựa chọn: chép lại, hoặc chuyển lên thư viện. Chép lại nghĩa là hai
//! bản đọc gói trôi dạt khỏi nhau — mà `tcc verify` nói "đạt" còn trình duyệt
//! đọc ra thứ khác thì đó đúng là loại lỗi tệ nhất của một tiêu chuẩn.
//!
//! # Bố cục
//!
//! ```text
//! ung-dung/
//! ├── manifest.json    ← chữ ký ký lên ĐÚNG byte của tệp này
//! ├── signature.hex    ← chữ ký lai, dạng hex
//! └── content/         ← mọi thứ trong này đi vào content_hash
//! ```

use std::{fs, io, path::Path};

use tcc_spec::FileTree;

pub const MANIFEST_FILE: &str = "manifest.json";
pub const SIGNATURE_FILE: &str = "signature.hex";
pub const CONTENT_DIR: &str = "content";

/// Trần tổng kích thước nội dung. Không có trần thì `tcc verify` trên một thư
/// mục khổng lồ sẽ ngốn hết bộ nhớ — `canonical_bytes` dựng tất cả trong RAM.
pub const MAX_CONTENT_BYTES: u64 = 256 * 1024 * 1024;

// Chốt con số, lúc DỰNG. `256 * 1024 * 1024` đổi một dấu `*` thành `+` vẫn biên
// dịch, vẫn chạy, chỉ ra một trần khác hẳn — và không phép thử nào nhìn tới một
// biểu thức hằng.
const _: () = assert!(MAX_CONTENT_BYTES == 268_435_456);

#[derive(Debug)]
pub enum PackageError {
    Io(String),
    Tree(String),
    TooLarge {
        total: u64,
    },
    Symlink(String),
    MissingFile(String),
    /// `signature.hex` có nhưng KHÔNG đúng 6746 chữ số hex.
    ///
    /// Tách khỏi `not-hex`: "sai định dạng" và "đúng định dạng nhưng sai độ
    /// dài" là hai chuyện khác nhau với người dựng gói — một bên là gõ nhầm,
    /// một bên là ký bằng lược đồ khác.
    BadSignatureLength(usize),
    /// Không phải hex chữ thường.
    ///
    /// Tách khỏi `Io`: `Io` là "đĩa hỏng", còn đây là "tệp có nhưng viết sai".
    /// Gộp lại thì mã lỗi ra `bad-path`, mà đặc tả nói `not-hex` — và một mã
    /// sai là bộ kiểm định của người ngoài không khớp được.
    KhongPhaiHex(String),
}

impl PackageError {
    /// Mã lỗi ỔN ĐỊNH, thuộc về TIÊU CHUẨN — `spec/0.1/06-error-codes.md`.
    #[must_use]
    pub const fn ma(&self) -> &'static str {
        match self {
            Self::MissingFile(_) => "missing-file",
            Self::KhongPhaiHex(_) => "not-hex",
            Self::BadSignatureLength(_) => "bad-signature-length",
            Self::Symlink(_) => "symlink",
            Self::TooLarge { .. } => "package-too-large",
            Self::Io(_) | Self::Tree(_) => "bad-path",
        }
    }
}

impl std::fmt::Display for PackageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "lỗi đọc/ghi: {e}"),
            Self::Tree(e) => write!(f, "cây tệp không hợp lệ: {e}"),
            Self::TooLarge { total } => write!(
                f,
                "nội dung {total} byte, vượt trần {MAX_CONTENT_BYTES} byte"
            ),
            Self::Symlink(p) => write!(
                f,
                "\"{p}\" là liên kết mềm — gói TCC không nhận liên kết mềm vì nó có \
                 thể trỏ ra ngoài thư mục gói, và cái được ký sẽ khác cái được chạy"
            ),
            Self::MissingFile(p) => write!(f, "thiếu \"{p}\""),
            Self::KhongPhaiHex(p) => write!(f, "{p} không phải hex chữ thường"),
            Self::BadSignatureLength(n) => {
                write!(
                    f,
                    "signature.hex có {n} chữ số hex, cần đúng {SIGNATURE_HEX_LEN}"
                )
            }
        }
    }
}

impl From<io::Error> for PackageError {
    fn from(e: io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

/// Đọc thư mục `content/` thành một `FileTree`.
///
/// # Errors
/// Lỗi đọc đĩa, có liên kết mềm, đường dẫn vi phạm ràng buộc, hoặc quá trần.
pub fn read_content(goc: &Path) -> Result<FileTree, PackageError> {
    let thu_muc = goc.join(CONTENT_DIR);
    if !thu_muc.is_dir() {
        return Err(PackageError::MissingFile(CONTENT_DIR.to_string()));
    }
    let mut cay = FileTree::new();
    let mut tong: u64 = 0;
    di_sau(&thu_muc, &thu_muc, &mut cay, &mut tong, MAX_CONTENT_BYTES)?;
    Ok(cay)
}

/// Trần truyền VÀO chứ không đọc hằng ở đây.
///
/// Dựng 256 MiB tệp trong một phép thử là không làm được, nên trần cứng ở đây
/// nghĩa là trần ấy KHÔNG bao giờ được kiểm — và kiểm đột biến 26/08/2026 xác
/// nhận đúng thế: đổi `>` thành `>=`, đổi `+=` thành `*=`, cả năm đột biến ở
/// chỗ này đều sống. Một trần chặn tràn bộ nhớ trên dữ liệu CHƯA xác thực mà
/// chưa ai kiểm thì nó là một lời hứa, không phải một chắn.
fn di_sau(
    goc: &Path,
    hien: &Path,
    cay: &mut FileTree,
    tong: &mut u64,
    tran: u64,
) -> Result<(), PackageError> {
    let mut muc: Vec<_> = fs::read_dir(hien)?.collect::<Result<_, _>>()?;
    // Sắp xếp để thứ tự duyệt xác định. `FileTree` tự sắp lại khi băm, nhưng
    // duyệt có thứ tự làm thông báo lỗi ổn định giữa các lần chạy.
    muc.sort_by_key(std::fs::DirEntry::path);

    for m in muc {
        let duong = m.path();
        // `symlink_metadata` KHÔNG đi theo liên kết — phải dùng nó để phát hiện
        // được liên kết mềm. `metadata` thường sẽ đi theo và ta không thấy gì.
        let meta = fs::symlink_metadata(&duong)?;

        if meta.file_type().is_symlink() {
            return Err(PackageError::Symlink(duong.display().to_string()));
        }
        if meta.is_dir() {
            di_sau(goc, &duong, cay, tong, tran)?;
            continue;
        }

        *tong += meta.len();
        if *tong > tran {
            return Err(PackageError::TooLarge { total: *tong });
        }

        let tuong_doi = duong
            .strip_prefix(goc)
            .map_err(|e| PackageError::Io(e.to_string()))?
            .to_string_lossy()
            // Trên Windows `Path` dùng `\`; dạng chuẩn tắc chỉ nhận `/`.
            .replace('\\', "/");

        let noi_dung = fs::read(&duong)?;
        cay.insert(&tuong_doi, noi_dung)
            .map_err(|e| PackageError::Tree(e.to_string()))?;
    }
    Ok(())
}

/// Đọc bản kê khai dưới dạng BYTE THÔ.
///
/// Trả byte chứ không trả cấu trúc đã giải mã — chữ ký ký lên đúng chuỗi byte
/// này, nên đọc rồi tuần tự hoá lại là làm hỏng chữ ký.
///
/// # Errors
/// Thiếu tệp hoặc lỗi đọc.
pub fn read_manifest(goc: &Path) -> Result<Vec<u8>, PackageError> {
    let p = goc.join(MANIFEST_FILE);
    if !p.is_file() {
        return Err(PackageError::MissingFile(MANIFEST_FILE.to_string()));
    }
    Ok(fs::read(p)?)
}

/// # Errors
/// Thiếu tệp, lỗi đọc, hoặc nội dung không phải hex.
pub fn read_signature(goc: &Path) -> Result<Vec<u8>, PackageError> {
    let p = goc.join(SIGNATURE_FILE);
    if !p.is_file() {
        return Err(PackageError::MissingFile(SIGNATURE_FILE.to_string()));
    }
    let s = fs::read_to_string(p)?;
    read_signature_hex(&s)
}

/// Số chữ số hex của một chữ ký lai: `2 × 3373`.
pub const SIGNATURE_HEX_LEN: usize = 6746;

/// Đọc nội dung `signature.hex` theo ĐÚNG luật của tiêu chuẩn.
///
/// # Vì sao chặt đến thế
///
/// Bản cũ (tới 18/08/2026) gọi `s.trim()` rồi `hex::decode`, tức là nhận cả
/// khoảng trắng đầu dòng, cả tab, cả **chữ hoa**. Đặc tả thì nói *"lowercase
/// hex"* — nên hai bên lệch nhau, và một gói bản này nhận thì bản khác từ chối.
///
/// Không chuẩn hoá chữ hoa thành chữ thường mà **từ chối**, cùng lý do bản kê
/// khai cấm trường lạ: hai cách viết của một giá trị là hai thứ phải so, mà
/// chính phép so ấy là thứ chữ ký bảo vệ.
///
/// # Errors
/// Không phải hex chữ thường, hoặc sai độ dài.
pub fn read_signature_hex(noi_dung: &str) -> Result<Vec<u8>, PackageError> {
    // ĐÚNG một dấu xuống dòng ở cuối, không hơn. `trim` nhận cả khoảng trắng
    // đầu và mọi thứ ở cuối — rộng hơn hẳn thứ đặc tả cho phép.
    let than = noi_dung
        .strip_suffix("\r\n")
        .or_else(|| noi_dung.strip_suffix('\n'))
        .unwrap_or(noi_dung);

    if than.len() != SIGNATURE_HEX_LEN {
        return Err(PackageError::BadSignatureLength(than.len()));
    }
    if !than
        .bytes()
        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(PackageError::KhongPhaiHex(SIGNATURE_FILE.to_owned()));
    }
    hex::decode(than).map_err(|_| PackageError::KhongPhaiHex(SIGNATURE_FILE.to_owned()))
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay"
)]
mod kiem_thu_chu_ky_hex {
    use super::*;

    fn hop_le() -> String {
        "ab".repeat(SIGNATURE_HEX_LEN / 2)
    }

    #[test]
    fn dung_do_dai_va_chu_thuong_thi_nhan() {
        assert!(read_signature_hex(&hop_le()).is_ok());
        assert!(read_signature_hex(&format!("{}\n", hop_le())).is_ok());
        assert!(read_signature_hex(&format!("{}\r\n", hop_le())).is_ok());
    }

    /// **Chữ HOA bị từ chối**, không được chuẩn hoá lặng lẽ.
    ///
    /// Hai cách viết của một giá trị là hai thứ phải so, mà chính phép so ấy là
    /// thứ chữ ký bảo vệ.
    #[test]
    fn chu_hoa_bi_tu_choi() {
        let hoa = hop_le().to_uppercase();
        assert!(read_signature_hex(&hoa).is_err());
        // Lẫn lộn cũng thế.
        let mut lan = hop_le();
        lan.replace_range(0..1, "A");
        assert!(read_signature_hex(&lan).is_err());
    }

    /// Khoảng trắng thừa bị từ chối — `trim()` cũ nhận hết.
    #[test]
    fn khoang_trang_thua_bi_tu_choi() {
        for xau in [
            format!(" {}", hop_le()),
            format!("{}  ", hop_le()),
            format!("{}\n\n", hop_le()),
            format!("\t{}", hop_le()),
            format!("{}\n ", hop_le()),
        ] {
            assert!(read_signature_hex(&xau).is_err(), "nhận nhầm {xau:?}");
        }
    }

    /// Sai độ dài ra mã RIÊNG, không lẫn với "không phải hex".
    ///
    /// Với người dựng gói, "gõ nhầm" và "ký bằng lược đồ khác" là hai việc.
    #[test]
    fn sai_do_dai_ra_ma_rieng() {
        let ngan = "ab".repeat(100);
        let e = read_signature_hex(&ngan).unwrap_err();
        assert_eq!(e.ma(), "bad-signature-length", "{e}");

        let dai = format!("{}ab", hop_le());
        assert_eq!(
            read_signature_hex(&dai).unwrap_err().ma(),
            "bad-signature-length"
        );
    }

    /// Thiếu tệp ra `missing-file`.
    #[test]
    fn thieu_tep_ra_dung_ma() {
        let e = PackageError::MissingFile("signature.hex".to_owned());
        assert_eq!(e.ma(), "missing-file");
    }

    /// **Trần nội dung phải chặn ĐÚNG ở mép, và cộng dồn qua thư mục con.**
    ///
    /// Kiểm đột biến 26/08/2026: cả năm đột biến ở chỗ này đều SỐNG — `>` thành
    /// `>=`, `+=` thành `*=`, và cả hai dấu `*` trong `256 * 1024 * 1024`. Trần
    /// ấy chặn tràn bộ nhớ khi đọc dữ liệu CHƯA xác thực; chưa ai kiểm thì nó
    /// là một lời hứa, không phải một chắn.
    #[test]
    fn tran_noi_dung_chan_dung_o_mep() {
        let tam = std::env::temp_dir().join("tcc-kiem-tran-noi-dung");
        let _ = std::fs::remove_dir_all(&tam);
        std::fs::create_dir_all(tam.join("con")).unwrap();
        // 10 byte ở gốc, 10 byte trong thư mục con — tổng 20.
        std::fs::write(tam.join("a.txt"), vec![b'x'; 10]).unwrap();
        std::fs::write(tam.join("con").join("b.txt"), vec![b'y'; 10]).unwrap();

        let doc = |tran: u64| {
            let mut cay = FileTree::new();
            let mut tong = 0u64;
            di_sau(&tam, &tam, &mut cay, &mut tong, tran).map(|()| tong)
        };

        // Đúng bằng trần thì QUA — `>` chứ không phải `>=`.
        assert_eq!(doc(20).ok(), Some(20), "đúng bằng trần mà bị chặn");
        // Hơn một byte thì chối.
        assert!(
            matches!(doc(19), Err(PackageError::TooLarge { .. })),
            "vượt trần mà không bị chặn"
        );
        // Và phải CỘNG DỒN qua thư mục con: trần 15 lớn hơn từng tệp một, nhưng
        // nhỏ hơn tổng. `+=` thành `*=` hay quên cộng thì chỗ này lọt.
        assert!(
            matches!(doc(15), Err(PackageError::TooLarge { .. })),
            "không cộng dồn qua thư mục con — mỗi tệp lọt riêng lẻ"
        );

        let _ = std::fs::remove_dir_all(&tam);
    }
}
