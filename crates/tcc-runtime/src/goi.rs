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

pub const TEP_KE_KHAI: &str = "manifest.json";
pub const TEP_CHU_KY: &str = "signature.hex";
pub const THU_MUC_NOI_DUNG: &str = "content";

/// Trần tổng kích thước nội dung. Không có trần thì `tcc verify` trên một thư
/// mục khổng lồ sẽ ngốn hết bộ nhớ — `canonical_bytes` dựng tất cả trong RAM.
pub const MAX_CONTENT_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Debug)]
pub enum LoiGoi {
    Io(String),
    Tree(String),
    QuaLon { total: u64 },
    LienKetMem(String),
    ThieuTep(String),
}

impl std::fmt::Display for LoiGoi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "lỗi đọc/ghi: {e}"),
            Self::Tree(e) => write!(f, "cây tệp không hợp lệ: {e}"),
            Self::QuaLon { total } => write!(
                f,
                "nội dung {total} byte, vượt trần {MAX_CONTENT_BYTES} byte"
            ),
            Self::LienKetMem(p) => write!(
                f,
                "\"{p}\" là liên kết mềm — gói TCC không nhận liên kết mềm vì nó có \
                 thể trỏ ra ngoài thư mục gói, và cái được ký sẽ khác cái được chạy"
            ),
            Self::ThieuTep(p) => write!(f, "thiếu \"{p}\""),
        }
    }
}

impl From<io::Error> for LoiGoi {
    fn from(e: io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

/// Đọc thư mục `content/` thành một `FileTree`.
///
/// # Errors
/// Lỗi đọc đĩa, có liên kết mềm, đường dẫn vi phạm ràng buộc, hoặc quá trần.
pub fn doc_noi_dung(goc: &Path) -> Result<FileTree, LoiGoi> {
    let thu_muc = goc.join(THU_MUC_NOI_DUNG);
    if !thu_muc.is_dir() {
        return Err(LoiGoi::ThieuTep(THU_MUC_NOI_DUNG.to_string()));
    }
    let mut cay = FileTree::new();
    let mut tong: u64 = 0;
    di_sau(&thu_muc, &thu_muc, &mut cay, &mut tong)?;
    Ok(cay)
}

fn di_sau(goc: &Path, hien: &Path, cay: &mut FileTree, tong: &mut u64) -> Result<(), LoiGoi> {
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
            return Err(LoiGoi::LienKetMem(duong.display().to_string()));
        }
        if meta.is_dir() {
            di_sau(goc, &duong, cay, tong)?;
            continue;
        }

        *tong += meta.len();
        if *tong > MAX_CONTENT_BYTES {
            return Err(LoiGoi::QuaLon { total: *tong });
        }

        let tuong_doi = duong
            .strip_prefix(goc)
            .map_err(|e| LoiGoi::Io(e.to_string()))?
            .to_string_lossy()
            // Trên Windows `Path` dùng `\`; dạng chuẩn tắc chỉ nhận `/`.
            .replace('\\', "/");

        let noi_dung = fs::read(&duong)?;
        cay.insert(&tuong_doi, noi_dung)
            .map_err(|e| LoiGoi::Tree(e.to_string()))?;
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
pub fn doc_ke_khai(goc: &Path) -> Result<Vec<u8>, LoiGoi> {
    let p = goc.join(TEP_KE_KHAI);
    if !p.is_file() {
        return Err(LoiGoi::ThieuTep(TEP_KE_KHAI.to_string()));
    }
    Ok(fs::read(p)?)
}

/// # Errors
/// Thiếu tệp, lỗi đọc, hoặc nội dung không phải hex.
pub fn doc_chu_ky(goc: &Path) -> Result<Vec<u8>, LoiGoi> {
    let p = goc.join(TEP_CHU_KY);
    if !p.is_file() {
        return Err(LoiGoi::ThieuTep(TEP_CHU_KY.to_string()));
    }
    let s = fs::read_to_string(p)?;
    hex::decode(s.trim()).map_err(|e| LoiGoi::Io(format!("{TEP_CHU_KY} không phải hex: {e}")))
}
