//! Băm nội dung — **Blake3**, đầu ra 48 byte (384 bit).
//!
//! # Vì sao Blake3 chứ không phải SHA-384
//!
//! Bản đầu tôi chọn SHA-384. Sau khi khảo sát chuỗi TCC (`tcc_node_v3`) thì đổi,
//! và lý do nằm ngay trong **luật số 1 của crate này: ít phụ thuộc nhất**.
//!
//! Chuỗi TCC đã dùng Blake3 ở khắp nơi — địa chỉ ví chính là
//! `blake3(khoá_công_khai)`. Trình duyệt buộc phải nói chuyện với chuỗi, nên
//! Blake3 sẽ có mặt trong cây phụ thuộc dù ta muốn hay không. Giữ SHA-384 nghĩa
//! là biên giới tin cậy có **HAI** hàm băm thay vì một — hai thứ phải kiểm định,
//! hai thứ phải giải thích, mà không đổi lấy điều gì.
//!
//! Đổi lúc này miễn phí vì chưa gói nào được phát hành. Đổi sau là chia đôi tiêu
//! chuẩn.
//!
//! # Vì sao 384 bit chứ không phải 256
//!
//! Thuật toán Grover làm yếu sức chống va chạm đi một nửa số bit. Blake3-256 còn
//! tương đương 128 bit — vẫn đủ, nhưng 384 bit thì thừa hẳn mà gần như không tốn
//! thêm gì. Với một dự án lấy hậu lượng tử làm định hướng, rộng tay ở đây là rẻ.
//!
//! Blake3 cho đầu ra dài tuỳ ý, và **32 byte đầu của đầu ra dài luôn trùng với
//! Blake3-256 chuẩn** — tính chất đó được dùng làm mỏ neo kiểm chứng bên dưới.

/// Độ dài kết quả: 48 byte.
pub const CONTENT_HASH_LEN: usize = 48;

/// Băm nội dung theo LUỒNG.
///
/// Tồn tại để không phải dựng cả gói trong bộ nhớ chỉ để băm nó — xem
/// `FileTree::for_each_canonical_chunk`.
#[derive(Default)]
pub struct ContentHasher(blake3::Hasher);

impl ContentHasher {
    #[must_use]
    pub fn new() -> Self {
        Self(blake3::Hasher::new())
    }

    pub fn update(&mut self, chunk: &[u8]) {
        self.0.update(chunk);
    }

    /// Kết quả cuối, hex chữ thường, đúng `CONTENT_HASH_LEN` byte.
    #[must_use]
    pub fn finish_hex(&self) -> String {
        let mut ra = [0u8; CONTENT_HASH_LEN];
        self.0.finalize_xof().fill(&mut ra);
        hex::encode(ra)
    }
}

/// Băm nội dung, trả chuỗi hex chữ thường — đúng dạng ghi trong bản kê khai.
#[must_use]
pub fn content_hash_hex(data: &[u8]) -> String {
    let mut ra = [0u8; CONTENT_HASH_LEN];
    let mut h = blake3::Hasher::new();
    h.update(data);
    h.finalize_xof().fill(&mut ra);
    hex::encode(ra)
}

#[cfg(test)]
mod kiem_thu {
    use super::*;

    /// Không phụ thuộc `tcc-spec` (luật số 3: crate này là lá). Chốt con số ở đây;
    /// lệch với bên kia thì phép thử `do_dai_bam_hai_crate_khop_nhau` bên
    /// `tcc-manifest` sẽ bắt.
    const CONTENT_HASH_HEX_LEN: usize = 96;

    #[test]
    fn do_dai_dung_nhu_tieu_chuan() {
        assert_eq!(content_hash_hex(b"").len(), CONTENT_HASH_HEX_LEN);
        assert_eq!(CONTENT_HASH_LEN * 2, CONTENT_HASH_HEX_LEN);
    }

    /// MỎ NEO NGOÀI: 32 byte đầu của đầu ra dài phải trùng Blake3-256 chuẩn của
    /// chuỗi rỗng — giá trị này công bố trong đặc tả Blake3, không phải do ta tự
    /// sinh ra rồi tự chép lại.
    ///
    /// Nếu ai đó vô tình đổi sang hàm băm khác, phép thử này gãy ngay.
    #[test]
    fn khop_gia_tri_chuan_cua_blake3() {
        let ra = content_hash_hex(b"");
        assert_eq!(
            &ra[..64],
            "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
            "32 byte đầu phải trùng Blake3-256 của chuỗi rỗng"
        );
    }

    /// Cùng một hàm băm với chuỗi TCC: địa chỉ ví là `blake3(khoá công khai)`.
    /// Phép thử này ghim lại sự đồng bộ đó.
    #[test]
    fn dong_bo_voi_chuoi_tcc() {
        let ra = content_hash_hex(b"tcc");
        let chuan = blake3::hash(b"tcc");
        assert_eq!(
            &ra[..64],
            hex::encode(chuan.as_bytes()),
            "lệch với Blake3 mà chuỗi TCC dùng cho địa chỉ ví"
        );
    }

    #[test]
    fn doi_mot_byte_thi_bam_doi_han() {
        assert_ne!(content_hash_hex(b"noi dung"), content_hash_hex(b"noi dunh"));
    }
}
