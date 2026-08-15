//! Giao dịch chuỗi TCC — và phép kiểm chặn KÝ MÙ.
//!
//! # Vấn đề crate này sinh ra để giải
//!
//! Ví web hiện gọi `tcc_buildUnsignedTransfer` rồi **ký thẳng chuỗi
//! `signing_message_hex` do máy chủ RPC trả về**. Thông điệp ấy là một **băm 32
//! byte**, nên ví không có cách nào biết nó ứng với giao dịch nào.
//!
//! > Người dùng đọc *"gửi 5 TCC cho X"* trên màn hình, rồi ký một chuỗi băm do
//! > máy chủ đưa. Một RPC bị chiếm trả về băm của giao dịch khác thì ví vẫn ký,
//! > và màn hình vẫn hiện đúng thứ người dùng vừa gõ.
//!
//! May là máy chủ **cũng** trả về `unsigned_tx_base64`. Nên trình duyệt giải mã
//! được nó, **tự tính lại** thông điệp ký, so với thứ được đưa, và chỉ ký khi
//! hai bên khớp. Rồi hiện ra các trường **đã giải mã** — không phải hiện lại
//! thứ người dùng vừa gõ, vì thứ ấy chứng minh được gì đâu.
//!
//! # Crate này ĐỌC, không ký
//!
//! Không có khoá nào đi qua đây. Nó giải mã và tính băm; việc ký nằm ở
//! `tcc-crypto`, việc giữ khoá nằm ở `tcc-keystore`. Chia thế để phần dễ sai
//! nhất — bố cục byte — kiểm được mà không cần khoá thật.

#![forbid(unsafe_code)]

use core::fmt;

/// Địa chỉ: 32 byte, hiện ra dạng `0x` + 64 hex.
///
/// `DECISIONS-IRREVERSIBLE.md` D3 của chuỗi: `Address = BLAKE3(pubkey)`, 32 byte
/// thô. D4: hiển thị dạng hex, bech32m để sau.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Address(pub [u8; 32]);

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x")?;
        for b in self.0 {
            write!(f, "{b:02x}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ChainError {
    #[error("dữ liệu cụt: cần {can} byte nữa ở vị trí {tai}")]
    TooShort { tai: usize, can: usize },

    #[error("loại payload {0} không phải giao dịch chuyển tiền")]
    NotATransfer(u32),

    #[error("còn {0} byte thừa sau khi giải mã xong — đây không phải giao dịch ta hiểu")]
    TrailingBytes(usize),

    #[error("chuỗi ghi nhớ không phải UTF-8 hợp lệ")]
    BadMemo,

    #[error(
        "THÔNG ĐIỆP KÝ KHÔNG KHỚP giao dịch — máy chủ đưa {duoc_dua}, tự tính ra {tu_tinh}. \
         KHÔNG ký."
    )]
    SigningMessageMismatch { duoc_dua: String, tu_tinh: String },
}

/// Giao dịch chuyển tiền, đã giải mã từ `unsigned_tx_base64`.
///
/// Chỉ có **các trường được ký**. `public_key` và `signature` nằm cuối cấu trúc
/// trên dây nhưng không vào thông điệp ký, nên chúng không thuộc về đây — đưa
/// vào chỉ tạo ấn tượng sai rằng chúng được bảo vệ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transfer {
    pub nonce: u128,
    pub from: Address,
    pub to: Address,
    /// Đơn vị nhỏ nhất, 18 chữ số thập phân — như wei.
    pub amount: u128,
    pub gas_price: u128,
    pub gas_limit: u128,
    /// Dấu thời gian của chuỗi. Xem ghi chú về endianness ở `signing_message`.
    pub timestamp: i64,
    /// Chuỗi ghi nhớ, tức `PayloadOption::Commit`.
    pub memo: String,
}

/// Chỉ số biến thể của `PayloadOption::Commit` trong `enum` của chuỗi.
///
/// `bincode` 1.x mã hoá biến thể enum bằng `u32` little-endian theo THỨ TỰ KHAI
/// BÁO. `Commit` khai đầu tiên nên là 0. Thêm một biến thể vào TRƯỚC nó ở phía
/// chuỗi là đổi con số này — và đó là thay đổi phá vỡ, im lặng.
const PAYLOAD_COMMIT: u32 = 0;

impl Transfer {
    /// Thông điệp ký, tính lại **độc lập** với máy chủ.
    ///
    /// Công thức chép từ `dilithium3/src/lib.rs::signing_message` của chuỗi:
    ///
    /// ```text
    /// BLAKE3( nonce(BE) ‖ from(32) ‖ to(32) ‖ amount(BE)
    ///         ‖ gas_price(BE) ‖ gas_limit(BE) ‖ timestamp(LE) ‖ BLAKE3(memo) )
    /// ```
    ///
    /// ⚠️ **`timestamp` là LITTLE-endian trong khi mọi trường khác BIG-endian.**
    /// Đây không phải lỗi gõ của tôi — chuỗi làm đúng thế. Nó chạy được vì hai
    /// bên cùng làm, nhưng nó đúng là loại chi tiết làm vỡ bản cài đặt thứ ba,
    /// cùng lớp với bẫy giao diện FIPS 204 mà dự án này đã dẫm một lần.
    ///
    /// ⚠️ **`chain_id` KHÔNG có trong đây.** Chuỗi chưa đưa nó vào (D6, trạng
    /// thái `CẦN-VERIFY`). Nghĩa là một giao dịch đã ký về nguyên tắc phát lại
    /// được sang mạng TCC khác cùng địa chỉ và nonce. Khi chuỗi sửa, **hàm này
    /// phải sửa theo cùng lúc**, nếu không trình duyệt sẽ tính ra băm khác và
    /// từ chối ký mọi giao dịch — hỏng về phía an toàn, nhưng vẫn là hỏng.
    #[must_use]
    pub fn signing_message(&self) -> [u8; 32] {
        let mut d = Vec::with_capacity(160);
        d.extend_from_slice(&self.nonce.to_be_bytes());
        d.extend_from_slice(&self.from.0);
        d.extend_from_slice(&self.to.0);
        d.extend_from_slice(&self.amount.to_be_bytes());
        d.extend_from_slice(&self.gas_price.to_be_bytes());
        d.extend_from_slice(&self.gas_limit.to_be_bytes());
        d.extend_from_slice(&self.timestamp.to_le_bytes());
        d.extend_from_slice(blake3::hash(self.memo.as_bytes()).as_bytes());
        *blake3::hash(&d).as_bytes()
    }

    /// Giải mã `unsigned_tx_base64` đã bỏ base64.
    ///
    /// Bố cục là `bincode` 1.x mặc định: số nguyên cố định độ dài, **little-
    /// endian**, `Vec` và `String` có tiền tố độ dài `u64`, biến thể enum là
    /// `u32`.
    ///
    /// # Errors
    /// Dữ liệu cụt, thừa byte, payload không phải chuyển tiền, hoặc memo hỏng.
    pub fn decode(bytes: &[u8]) -> Result<Self, ChainError> {
        let mut r = Doc::new(bytes);
        let nonce = r.u128()?;
        let from = Address(r.mang32()?);
        let to = Address(r.mang32()?);
        let amount = r.u128()?;
        let gas_price = r.u128()?;
        let gas_limit = r.u128()?;
        let timestamp = r.i64()?;

        let bien_the = r.u32()?;
        if bien_the != PAYLOAD_COMMIT {
            return Err(ChainError::NotATransfer(bien_the));
        }
        let memo = r.chuoi()?;

        // `public_key` và `signature` đứng sau; giao dịch CHƯA ký nên cả hai
        // rỗng. Đọc cho hết để phát hiện byte thừa — dữ liệu ta không hiểu là
        // dữ liệu ta không được phép bỏ qua.
        let _pub = r.byte_co_do_dai()?;
        let _sig = r.byte_co_do_dai()?;
        let con_lai = r.con_lai();
        if con_lai != 0 {
            return Err(ChainError::TrailingBytes(con_lai));
        }

        Ok(Self {
            nonce,
            from,
            to,
            amount,
            gas_price,
            gas_limit,
            timestamp,
            memo,
        })
    }
}

/// Kiểm rằng thông điệp máy chủ đưa ĐÚNG là của giao dịch này.
///
/// Đây là hàm mà cả crate tồn tại vì nó. Không có nó, ký là ký mù.
///
/// # Errors
/// Băm tự tính khác băm được đưa.
pub fn check_signing_message(tx: &Transfer, duoc_dua: &[u8; 32]) -> Result<(), ChainError> {
    let tu_tinh = tx.signing_message();
    if tu_tinh == *duoc_dua {
        return Ok(());
    }
    Err(ChainError::SigningMessageMismatch {
        duoc_dua: hex32(duoc_dua),
        tu_tinh: hex32(&tu_tinh),
    })
}

fn hex32(b: &[u8; 32]) -> String {
    use core::fmt::Write as _;
    let mut ra = String::with_capacity(64);
    for x in b {
        let _ = write!(ra, "{x:02x}");
    }
    ra
}

/// Bộ đọc byte, không bao giờ đọc quá cuối.
struct Doc<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> Doc<'a> {
    const fn new(b: &'a [u8]) -> Self {
        Self { b, i: 0 }
    }

    fn lay(&mut self, n: usize) -> Result<&'a [u8], ChainError> {
        let het = self.i.checked_add(n).ok_or(ChainError::TooShort {
            tai: self.i,
            can: n,
        })?;
        if het > self.b.len() {
            return Err(ChainError::TooShort {
                tai: self.i,
                can: het - self.b.len(),
            });
        }
        let ra = &self.b[self.i..het];
        self.i = het;
        Ok(ra)
    }

    fn u32(&mut self) -> Result<u32, ChainError> {
        let x: [u8; 4] = self.lay(4)?.try_into().unwrap_or([0; 4]);
        Ok(u32::from_le_bytes(x))
    }

    fn u64(&mut self) -> Result<u64, ChainError> {
        let x: [u8; 8] = self.lay(8)?.try_into().unwrap_or([0; 8]);
        Ok(u64::from_le_bytes(x))
    }

    fn i64(&mut self) -> Result<i64, ChainError> {
        let x: [u8; 8] = self.lay(8)?.try_into().unwrap_or([0; 8]);
        Ok(i64::from_le_bytes(x))
    }

    fn u128(&mut self) -> Result<u128, ChainError> {
        let x: [u8; 16] = self.lay(16)?.try_into().unwrap_or([0; 16]);
        Ok(u128::from_le_bytes(x))
    }

    fn mang32(&mut self) -> Result<[u8; 32], ChainError> {
        Ok(self.lay(32)?.try_into().unwrap_or([0; 32]))
    }

    fn byte_co_do_dai(&mut self) -> Result<&'a [u8], ChainError> {
        let n = self.u64()?;
        let n = usize::try_from(n).map_err(|_| ChainError::TooShort {
            tai: self.i,
            can: usize::MAX,
        })?;
        self.lay(n)
    }

    fn chuoi(&mut self) -> Result<String, ChainError> {
        let b = self.byte_co_do_dai()?;
        String::from_utf8(b.to_vec()).map_err(|_| ChainError::BadMemo)
    }

    const fn con_lai(&self) -> usize {
        self.b.len() - self.i
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "kiểm thử: hỏng thì phải nổ ngay")]
mod kiem_thu {
    use super::*;

    fn mau() -> Transfer {
        Transfer {
            nonce: 7,
            from: Address([0x11; 32]),
            to: Address([0x22; 32]),
            amount: 5_000_000_000_000_000_000, // 5 TCC
            gas_price: 1,
            gas_limit: 21_000,
            timestamp: 1_760_000_000,
            memo: "chào".to_owned(),
        }
    }

    #[test]
    fn thong_diep_ky_tat_dinh() {
        assert_eq!(mau().signing_message(), mau().signing_message());
    }

    /// Đổi BẤT KỲ trường nào cũng phải đổi thông điệp ký.
    ///
    /// Một trường không vào băm là một trường máy chủ sửa được mà không ai biết
    /// — đúng cái lỗ này sinh ra để chặn.
    #[test]
    fn moi_truong_deu_vao_thong_diep_ky() {
        let goc = mau().signing_message();
        let mut t = mau();
        t.nonce += 1;
        assert_ne!(t.signing_message(), goc, "nonce KHÔNG vào thông điệp ký");
        let mut t = mau();
        t.to = Address([0x33; 32]);
        assert_ne!(
            t.signing_message(),
            goc,
            "NGƯỜI NHẬN không vào thông điệp ký"
        );
        let mut t = mau();
        t.amount += 1;
        assert_ne!(t.signing_message(), goc, "SỐ TIỀN không vào thông điệp ký");
        let mut t = mau();
        t.gas_price += 1;
        assert_ne!(t.signing_message(), goc, "gas_price không vào");
        let mut t = mau();
        t.gas_limit += 1;
        assert_ne!(t.signing_message(), goc, "gas_limit không vào");
        let mut t = mau();
        t.timestamp += 1;
        assert_ne!(t.signing_message(), goc, "timestamp không vào");
        let mut t = mau();
        t.memo.push('!');
        assert_ne!(t.signing_message(), goc, "memo không vào");
        let mut t = mau();
        t.from = Address([0x44; 32]);
        assert_ne!(t.signing_message(), goc, "from không vào");
    }

    #[test]
    fn khop_thi_dat_lech_thi_tu_choi() {
        let t = mau();
        let dung = t.signing_message();
        assert!(check_signing_message(&t, &dung).is_ok());

        let mut sai = dung;
        sai[0] ^= 1;
        let loi = check_signing_message(&t, &sai).unwrap_err();
        assert!(
            matches!(loi, ChainError::SigningMessageMismatch { .. }),
            "{loi}"
        );
    }

    /// Đòn thật: máy chủ đổi NGƯỜI NHẬN nhưng vẫn đưa băm của giao dịch cũ.
    ///
    /// Ví hiện tại sẽ ký. Phép kiểm này phải từ chối.
    #[test]
    fn may_chu_doi_nguoi_nhan_thi_bi_bat() {
        let that = mau();
        let mut ke_gian = mau();
        ke_gian.to = Address([0xEE; 32]);

        // Máy chủ đưa băm của giao dịch NGƯỜI DÙNG NGHĨ mình đang ký,
        // nhưng gói tin lại là của kẻ gian.
        let loi = check_signing_message(&ke_gian, &that.signing_message()).unwrap_err();
        assert!(matches!(loi, ChainError::SigningMessageMismatch { .. }));
    }

    #[test]
    fn du_lieu_cut_thi_bao_loi_chu_khong_hoang_loan() {
        for n in 0..80 {
            let _ = Transfer::decode(&vec![0u8; n]);
        }
    }

    #[test]
    fn byte_thua_bi_tu_choi() {
        // 16+32+32+16+16+16+8 = 136 byte trường cố định, rồi enum + memo rỗng
        // + pubkey rỗng + sig rỗng, rồi một byte thừa.
        let mut b = vec![0u8; 136];
        b.extend_from_slice(&0u32.to_le_bytes()); // Commit
        b.extend_from_slice(&0u64.to_le_bytes()); // memo rỗng
        b.extend_from_slice(&0u64.to_le_bytes()); // pubkey rỗng
        b.extend_from_slice(&0u64.to_le_bytes()); // signature rỗng
        assert!(
            Transfer::decode(&b).is_ok(),
            "bố cục cơ bản phải giải mã được"
        );
        b.push(0);
        assert_eq!(Transfer::decode(&b), Err(ChainError::TrailingBytes(1)));
    }

    #[test]
    fn payload_khong_phai_chuyen_tien_thi_tu_choi() {
        let mut b = vec![0u8; 136];
        b.extend_from_slice(&3u32.to_le_bytes()); // ContractUpgrade chẳng hạn
        assert_eq!(Transfer::decode(&b), Err(ChainError::NotATransfer(3)));
    }
}
