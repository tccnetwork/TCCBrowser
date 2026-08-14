//! Sinh nhiễu TẤT ĐỊNH và các phép đột biến.
//!
//! Tất định là bắt buộc, không phải tiện tay: một lần hỏng chỉ hữu ích khi tái
//! hiện được. Cùng hạt giống, cùng số vòng thì ra đúng cùng chuỗi đầu vào — kể
//! cả trên máy khác.

/// Xorshift64*, đủ tốt cho việc chọn đột biến và không kéo thêm phụ thuộc nào.
///
/// KHÔNG dùng cho mật mã, và không có chỗ nào trong kho này dùng nó cho mật mã.
pub struct Rng(u64);

impl Rng {
    #[must_use]
    pub const fn new(hat: u64) -> Self {
        // Hạt 0 làm xorshift đứng yên mãi mãi ở 0.
        Self(if hat == 0 { 0x9E37_79B9_7F4A_7C15 } else { hat })
    }

    pub const fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            usize::try_from(self.next_u64() % n as u64).unwrap_or(0)
        }
    }
}

/// Vài byte đặc biệt hay làm vỡ bộ phân tích, trộn vào cho nhanh chạm.
///
/// Không phải may rủi: đây là những thứ ĐÃ từng là lỗ trong dự án này — ký tự
/// đảo chiều (L3), dấu kết hợp (L10), dấu `@` giả mạo userinfo (L9), `..` thoát
/// thư mục, và dấu hai chấm của ổ đĩa Windows.
const POISON: &[&[u8]] = &[
    b"\xe2\x80\xae", // U+202E đảo chiều chữ
    b"\xcc\x81",     // U+0301 dấu sắc kết hợp
    b"..",
    b"/",
    b"\\",
    b":",
    b"@",
    b"\x00",
    b"\r",
    b"\n",
    b"\"",
    b"{",
    b"}",
    b"[",
    b"]",
    b"-1",
    b"1e999",
    b"18446744073709551616",
    b"null",
    b"x-acme-la",
];

/// Đột biến một mẫu hạt giống thành một đầu vào mới.
#[must_use]
pub fn mutate(nhieu: &mut Rng, goc: &[u8]) -> Vec<u8> {
    let mut v = goc.to_vec();
    // Vài phép một lượt: một phép đơn lẻ hiếm khi đi sâu được.
    let so_phep = 1 + nhieu.below(4);
    for _ in 0..so_phep {
        match nhieu.below(7) {
            // lật một bit
            0 if !v.is_empty() => {
                let i = nhieu.below(v.len());
                let bit = nhieu.below(8);
                if let Some(b) = v.get_mut(i) {
                    *b ^= 1u8 << bit;
                }
            }
            // chèn một mảnh độc
            1 => {
                let m = POISON[nhieu.below(POISON.len())];
                let i = nhieu.below(v.len() + 1);
                let mut moi = v[..i].to_vec();
                moi.extend_from_slice(m);
                moi.extend_from_slice(&v[i..]);
                v = moi;
            }
            // xoá một đoạn
            2 if v.len() > 2 => {
                let i = nhieu.below(v.len());
                let n = 1 + nhieu.below((v.len() - i).min(16));
                v.drain(i..i + n);
            }
            // cắt cụt — bộ phân tích hay hỏng ở đầu vào thiếu đuôi
            3 if !v.is_empty() => {
                let n = nhieu.below(v.len());
                v.truncate(n);
            }
            // nhân bản một đoạn: sinh ra khoá JSON trùng, quyền xin trùng
            4 if !v.is_empty() => {
                let i = nhieu.below(v.len());
                let n = 1 + nhieu.below((v.len() - i).min(64));
                let doan = v[i..i + n].to_vec();
                let j = nhieu.below(v.len() + 1);
                let mut moi = v[..j].to_vec();
                moi.extend_from_slice(&doan);
                moi.extend_from_slice(&v[j..]);
                v = moi;
            }
            // chồng dấu: đúng đòn L10, và phải chồng ĐỦ NHIỀU mới chạm trần
            5 if !v.is_empty() => {
                let i = nhieu.below(v.len());
                let lan = 1 + nhieu.below(40);
                let mut moi = v[..i].to_vec();
                for _ in 0..lan {
                    moi.extend_from_slice(b"\xcc\x81");
                }
                moi.extend_from_slice(&v[i..]);
                v = moi;
            }
            // đổi một byte thành byte ngẫu nhiên
            _ if !v.is_empty() => {
                let i = nhieu.below(v.len());
                if let Some(b) = v.get_mut(i) {
                    *b = u8::try_from(nhieu.next_u64() & 0xff).unwrap_or(0);
                }
            }
            _ => {}
        }
    }
    // Trần: bộ phân tích đã có trần riêng, ta không cần nuôi nó tệp 100 MB.
    v.truncate(128 * 1024);
    v
}
