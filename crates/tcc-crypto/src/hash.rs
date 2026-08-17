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

/// Bối cảnh tách miền của **vân tay người ký**.
///
/// Có nó thì vân tay không bao giờ trùng với băm nội dung gói hay địa chỉ ví,
/// dù cả ba đều là BLAKE3 trên vài chục byte. Ba mục đích khác nhau đi qua một
/// hàm băm là chỗ dễ lẫn nhất — dự án này đã ghi lại bài học ấy ở
/// `spec/0.1/03-signature.md`.
pub const PUBLISHER_FINGERPRINT_CONTEXT: &str = "tcc/v1/publisher-fingerprint";

/// **Vân tay của khoá người ký**, để người dùng SO BẰNG MẮT.
///
/// # Vì sao phải là một BĂM, không phải một lát cắt của khoá
///
/// Bản đầu (tới 18/08/2026) lấy 10 ký tự đầu và 10 ký tự cuối của khoá thô.
/// Nó không sai về mặt chi phí — muốn khớp cả hai đầu thì phải mò 80 bit — mà
/// sai về mặt **phạm vi**: nó không phủ khúc giữa. Hai khoá trùng hai đầu và
/// khác ruột hiện ra y hệt nhau, và kẻ dựng ra cặp ấy chỉ cần đụng vào phần
/// không ai nhìn.
///
/// Một băm phủ **toàn bộ** khoá: đổi một bit ở bất kỳ đâu là đổi vân tay.
///
/// # Vì sao hiện ĐỦ, không cắt ngắn
///
/// Cùng luật với địa chỉ ví: cắt ngắn là lỗ dò trùng đầu-đuôi. Vân tay này
/// ngắn sẵn (32 byte), nên không có lý do gì để cắt thêm.
///
/// Trả về hex chữ thường, 64 ký tự. Bên hiện ra màn hình tự chia nhóm cho dễ
/// đọc — chia nhóm là việc của giao diện, không phải của phép băm.
#[must_use]
pub fn publisher_fingerprint_hex(public_key_hex: &str) -> String {
    let mut h = blake3::Hasher::new();
    h.update(PUBLISHER_FINGERPRINT_CONTEXT.as_bytes());
    // Băm CHUỖI HEX như người dùng thấy nó trong bản kê khai, không băm byte đã
    // giải mã: bản kê khai là thứ ký, và một khoá viết hoa/viết thường khác
    // nhau là hai chuỗi khác nhau ở đó.
    h.update(public_key_hex.as_bytes());
    hex::encode(h.finalize().as_bytes())
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay"
)]
mod kiem_thu_van_tay {
    use super::*;

    /// **Đổi một ký tự ở GIỮA khoá là đổi vân tay.**
    ///
    /// Đây đúng là thứ bản cũ không bắt được: nó chỉ nhìn hai đầu.
    #[test]
    fn doi_giua_khoa_la_doi_van_tay() {
        let khoa = "ab".repeat(1984);
        let mut giua = khoa.clone();
        let n = giua.len() / 2;
        giua.replace_range(n..=n, "c");

        assert_ne!(
            publisher_fingerprint_hex(&khoa),
            publisher_fingerprint_hex(&giua),
            "đổi ruột khoá mà vân tay không đổi"
        );
        // Hai đầu vẫn y hệt nhau — bản cũ sẽ hiện cùng một vân tay.
        assert_eq!(khoa[..10], giua[..10]);
        assert_eq!(khoa[khoa.len() - 10..], giua[giua.len() - 10..]);
    }

    /// Vân tay dài đúng 64 ký tự hex, chữ thường, không cắt.
    #[test]
    fn van_tay_du_64_ky_tu_chu_thuong() {
        let v = publisher_fingerprint_hex("00ff");
        assert_eq!(v.len(), 64);
        assert!(
            v.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase())
        );
        assert!(!v.contains('…'));
    }

    /// **Tách miền**: cùng dữ liệu, khác mục đích, khác kết quả.
    ///
    /// Bản đầu của phép thử này so `publisher_fingerprint_hex(d)` với
    /// `content_hash_hex(d)` bằng `assert_ne!` — và nó **vô nghĩa**: một bên 32
    /// byte, một bên 48, nên hai chuỗi hex không bao giờ bằng nhau dù có bối
    /// cảnh hay không. Gỡ bối cảnh đi phép thử vẫn xanh.
    ///
    /// Phép so đúng: 32 byte đầu của XOF-48 **chính là** BLAKE3-256 chuẩn (ghi
    /// ngay đầu tệp này). Nên thiếu bối cảnh thì vân tay sẽ là TIỀN TỐ của băm
    /// nội dung. Đòi nó KHÔNG phải tiền tố mới là đòi đúng thứ cần đòi.
    #[test]
    fn tach_mien_khoi_bam_noi_dung() {
        let d = "cùng một chuỗi";
        let van_tay = publisher_fingerprint_hex(d);
        let noi_dung = content_hash_hex(d.as_bytes());
        assert_eq!(van_tay.len(), 64);
        assert_eq!(noi_dung.len(), CONTENT_HASH_LEN * 2);
        assert!(
            !noi_dung.starts_with(&van_tay),
            "vân tay là tiền tố của băm nội dung — bối cảnh tách miền đã mất"
        );
    }

    /// Tất định: cùng khoá, cùng vân tay, mọi lúc.
    #[test]
    fn cung_khoa_cung_van_tay() {
        let k = "deadbeef".repeat(400);
        assert_eq!(publisher_fingerprint_hex(&k), publisher_fingerprint_hex(&k));
    }
}
