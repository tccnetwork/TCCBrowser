//! Hỏi chuỗi: **địa chỉ này có sở hữu thứ kia không?**
//!
//! # ⚠️ ĐỌC TRƯỚC: cái này KHÔNG bảo vệ nội dung
//!
//! Kế hoạch gọi 3.4 là *"bảo vệ nội dung TCC"*. Sau khi làm, tên ấy nói quá, và
//! đây là chỗ ghi lại vì sao.
//!
//! Nội dung của một gói TCC nằm trên đĩa ở dạng **đọc được**: gói là một thư
//! mục đã ký, không phải một hộp khoá. Ai mở thư mục ra là thấy. Nên một phép
//! kiểm sở hữu chỉ chặn được **màn hình**, không chặn được **tệp**.
//!
//! Chặn hiển thị mà không mã hoá nội dung là một cái khoá treo trên cánh cửa
//! không có tường: nó ngăn người đi ngang, không ngăn ai muốn vào.
//!
//! ## Vì sao không mã hoá luôn cho xong
//!
//! Vì không có chỗ nào cất khoá giải mã cho đúng người:
//!
//! | Cất ở đâu | Hỏng thế nào |
//! |---|---|
//! | Trong gói | Ai cũng đọc được — không mã hoá gì cả |
//! | Trên chuỗi | Chuỗi công khai; cùng chuyện |
//! | Người bán mã hoá cho người mua | Người bán vẫn giữ bản rõ, và bán tiếp cho ai cũng được |
//! | Máy chủ phát khoá sau khi kiểm sở hữu | **Chạy được** — nhưng đó là một DỊCH VỤ, không phải một tính chất của chuỗi |
//!
//! Kết luận thẳng: **bảo vệ nội dung không có máy chủ phát khoá là không làm
//! được.** Ai bảo làm được thì đang bán một cánh cửa không tường.
//!
//! Nên module này làm đúng thứ làm được và gọi đúng tên nó: **kiểm sở hữu**.
//! Nó đủ cho *"chỉ chủ sở hữu mới thấy nút Mở"*, và đúng như kế hoạch tự viết:
//! chặn được sao chép tuỳ tiện, **không** chặn được người có kỹ thuật và có
//! động lực.

use crate::Address;

/// Một thứ được sở hữu trên chuỗi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedToken {
    /// `0x` + hex. Giữ nguyên dạng chuỗi trả về, không diễn giải thành số:
    /// số hoá rồi in lại là chỗ `0x01` và `0x1` thành hai thứ khác nhau.
    pub token_id: String,
    /// Đường dẫn siêu dữ liệu do người phát hành đặt. **Chỉ để hiện**, không
    /// bao giờ để quyết định — nó là chuỗi người khác viết.
    pub uri: String,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum OwnershipError {
    #[error("phản hồi không phải JSON của `tcc_getNftsByOwner`")]
    BadShape,
    /// Máy chủ trả về danh sách của một ĐỊA CHỈ KHÁC.
    ///
    /// Cùng loại đòn với ký mù: hỏi về ví A, máy chủ trả lời về ví B, và người
    /// dùng thấy "bạn sở hữu" mà không biết đó là tài sản của ai.
    #[error("máy chủ trả về danh sách của địa chỉ khác — hỏi {hoi}, nhận {nhan}")]
    WrongOwner { hoi: String, nhan: String },
    /// Máy chủ trả về bộ sưu tập khác.
    #[error("máy chủ trả về bộ sưu tập khác — hỏi {hoi}, nhận {nhan}")]
    WrongCollection { hoi: String, nhan: String },
}

/// Đọc phản hồi `tcc_getNftsByOwner` và **kiểm nó trả lời đúng câu đã hỏi**.
///
/// # Vì sao phải kiểm lại `owner` và `contract`
///
/// Đây là cùng một bài học với chống ký mù: **máy chủ là bên ta đề phòng, không
/// phải bên ta dựa vào**. Nó trả về `owner` và `contract` trong phản hồi, nên
/// so lại được — và không so thì một máy chủ bị chiếm chỉ cần trả danh sách của
/// một ví giàu nào đó là mọi người đều "sở hữu".
///
/// # Errors
/// Phản hồi sai hình dạng, hoặc trả lời về địa chỉ/bộ sưu tập khác.
pub fn read_owned(
    phan_hoi: &serde_json::Value,
    hoi_bo_suu_tap: &str,
    hoi_chu: &Address,
) -> Result<Vec<OwnedToken>, OwnershipError> {
    let nhan_chu = phan_hoi
        .get("owner")
        .and_then(|v| v.as_str())
        .ok_or(OwnershipError::BadShape)?;
    let nhan_bst = phan_hoi
        .get("contract")
        .and_then(|v| v.as_str())
        .ok_or(OwnershipError::BadShape)?;

    let mong_chu = hoi_chu.to_string();
    if !bang_hex(nhan_chu, &mong_chu) {
        return Err(OwnershipError::WrongOwner {
            hoi: mong_chu,
            nhan: nhan_chu.to_owned(),
        });
    }
    if !bang_hex(nhan_bst, hoi_bo_suu_tap) {
        return Err(OwnershipError::WrongCollection {
            hoi: hoi_bo_suu_tap.to_owned(),
            nhan: nhan_bst.to_owned(),
        });
    }

    let ds = phan_hoi
        .get("nfts")
        .and_then(|v| v.as_array())
        .ok_or(OwnershipError::BadShape)?;
    ds.iter()
        .map(|x| {
            Ok(OwnedToken {
                token_id: x
                    .get("token_id")
                    .and_then(|v| v.as_str())
                    .ok_or(OwnershipError::BadShape)?
                    .to_owned(),
                // `uri` thiếu thì coi như rỗng: nó chỉ để hiện, và một mục
                // thiếu mô tả không phải lý do vứt cả phản hồi.
                uri: x
                    .get("uri")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_owned(),
            })
        })
        .collect()
}

/// Có sở hữu đúng mã ấy không.
///
/// So **không phân biệt hoa thường** và bỏ qua tiền tố `0x` — hai cách viết của
/// cùng một mã là cùng một thứ, và bắt người dùng khớp từng ký tự hoa thường là
/// tạo ra một lỗi không ai gỡ được.
#[must_use]
pub fn owns(ds: &[OwnedToken], token_id: &str) -> bool {
    ds.iter().any(|t| bang_hex(&t.token_id, token_id))
}

/// So hai chuỗi hex: bỏ `0x`, không phân biệt hoa thường, và **bỏ số 0 ở đầu**.
///
/// `0x01` và `0x1` là cùng một mã. Không chuẩn hoá thì một bên viết cách này,
/// một bên viết cách kia, và người dùng sở hữu thật lại bị báo là không.
fn bang_hex(a: &str, b: &str) -> bool {
    let gon = |s: &str| {
        s.trim()
            .trim_start_matches("0x")
            .trim_start_matches("0X")
            .trim_start_matches('0')
            .to_ascii_lowercase()
    };
    gon(a) == gon(b)
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay"
)]
mod kiem_thu {
    use super::*;

    const CHU: &str = "0x266346046c9d284e8598a2ed52ac73e31b095da31d16cf1738c96ee3eb5e9a71";
    const BST: &str = "0xaabbccdd";

    fn dia_chi() -> Address {
        let mut b = [0u8; 32];
        for (i, o) in b.iter_mut().enumerate() {
            let h = CHU.strip_prefix("0x").unwrap();
            *o = u8::from_str_radix(&h[i * 2..i * 2 + 2], 16).unwrap();
        }
        Address(b)
    }

    fn phan_hoi(chu: &str, bst: &str, ma: &[&str]) -> serde_json::Value {
        serde_json::json!({
            "contract": bst,
            "owner": chu,
            "count": ma.len(),
            "nfts": ma.iter().map(|m| serde_json::json!({"token_id": m, "uri": "ipfs://x"})).collect::<Vec<_>>(),
        })
    }

    #[test]
    fn doc_duoc_danh_sach() {
        let v = phan_hoi(CHU, BST, &["0x01", "0x02"]);
        let ds = read_owned(&v, BST, &dia_chi()).unwrap();
        assert_eq!(ds.len(), 2);
        assert_eq!(ds[0].token_id, "0x01");
        assert_eq!(ds[0].uri, "ipfs://x");
        assert!(owns(&ds, "0x02"));
        assert!(!owns(&ds, "0x03"));
    }

    /// **Máy chủ trả lời về ví KHÁC thì từ chối.**
    ///
    /// Cùng đòn với ký mù: hỏi ví A, trả lời ví B, người dùng thấy "bạn sở
    /// hữu" mà đó là tài sản của người khác.
    #[test]
    fn tra_loi_ve_vi_khac_thi_tu_choi() {
        let khac = "0x1111111111111111111111111111111111111111111111111111111111111111";
        let v = phan_hoi(khac, BST, &["0x01"]);
        let loi = read_owned(&v, BST, &dia_chi()).unwrap_err();
        assert!(matches!(loi, OwnershipError::WrongOwner { .. }), "{loi}");
    }

    /// Trả lời về bộ sưu tập khác cũng thế.
    #[test]
    fn tra_loi_ve_bo_suu_tap_khac_thi_tu_choi() {
        let v = phan_hoi(CHU, "0xdeadbeef", &["0x01"]);
        let loi = read_owned(&v, BST, &dia_chi()).unwrap_err();
        assert!(
            matches!(loi, OwnershipError::WrongCollection { .. }),
            "{loi}"
        );
    }

    /// `0x01` và `0x1` là cùng một mã — không chuẩn hoá thì chủ thật bị báo là
    /// không sở hữu.
    #[test]
    fn ma_viet_khac_nhau_van_la_mot() {
        let v = phan_hoi(CHU, BST, &["0x0001"]);
        let ds = read_owned(&v, BST, &dia_chi()).unwrap();
        for cach in ["0x1", "0x01", "0X0001", "1", "0x0000001"] {
            assert!(owns(&ds, cach), "không nhận ra {cach}");
        }
        assert!(!owns(&ds, "0x2"));
    }

    /// Địa chỉ viết hoa/thường vẫn là một địa chỉ.
    #[test]
    fn dia_chi_hoa_thuong_van_khop() {
        let v = phan_hoi(&CHU.to_uppercase(), BST, &["0x01"]);
        assert!(read_owned(&v, BST, &dia_chi()).is_ok());
    }

    /// Không sở hữu gì thì trả danh sách rỗng, KHÔNG phải lỗi.
    #[test]
    fn khong_so_huu_gi_khong_phai_loi() {
        let v = phan_hoi(CHU, BST, &[]);
        let ds = read_owned(&v, BST, &dia_chi()).unwrap();
        assert!(ds.is_empty());
        assert!(!owns(&ds, "0x01"));
    }

    /// Phản hồi hỏng thì báo lỗi, không hoảng loạn và không đoán bừa.
    #[test]
    fn phan_hoi_hong_thi_bao_loi() {
        for xau in [
            serde_json::json!({}),
            serde_json::json!({"owner": CHU}),
            serde_json::json!({"owner": CHU, "contract": BST}),
            serde_json::json!({"owner": CHU, "contract": BST, "nfts": "không phải mảng"}),
            serde_json::json!({"owner": CHU, "contract": BST, "nfts": [{"uri": "x"}]}),
        ] {
            assert!(read_owned(&xau, BST, &dia_chi()).is_err(), "{xau}");
        }
    }
}
