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

pub mod mnemonic;
pub mod wallet;

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

    #[error("phiên bản giao dịch {0} — crate này chỉ hiểu v{1}")]
    UnsupportedVersion(u32, u32),

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
    pub version: u32,
    /// Mạng nào. **Có** trong thông điệp ký, nên một giao dịch của testnet
    /// không phát lại được sang mainnet.
    pub chain_id: u64,
    pub from: Address,
    pub to: Address,
    pub nonce: u64,
    /// Đơn vị nhỏ nhất, 18 chữ số thập phân — như wei.
    pub amount: u128,
    pub gas_price: u64,
    pub gas_limit: u64,
    pub timestamp: i64,
    /// Chiều cao khối mà giao dịch hết hạn. Cũng nằm trong thông điệp ký, nên
    /// máy chủ không kéo dài hạn của một giao dịch đã ký được.
    pub expires_at: i64,
    pub memo: String,
}

/// Bộ tách miền của thông điệp ký v1 — `src/tx/signing.rs` của chuỗi.
///
/// Có nó thì một chuỗi byte ký cho TCC không bao giờ trùng nghĩa với chuỗi byte
/// ký cho giao thức khác. Đây là thứ gói TCC **chưa có** ở tầng chữ ký gói, và
/// là một ý đáng mượn.
const DOMAIN_V1: &[u8] = b"tcc/v1/tx";

/// Biến thể `Transfer` trên DÂY (thứ tự khai báo enum).
const WIRE_TRANSFER: u32 = 0;

/// Nhãn `Transfer` khi BĂM payload — `Payload::TAG_TRANSFER` của chuỗi.
///
/// ⚠️ Khác con số trên dây: dây dùng 0, băm dùng 1. Hai không gian số riêng
/// biệt, và lẫn chúng là tính ra băm sai mà vẫn "chạy".
const TAG_TRANSFER: u8 = 0x01;

/// Phiên bản giao dịch v1 — bố cục thông điệp ký mà crate này hiểu.
///
/// Chuỗi đã có **v2** với bộ tách miền riêng `"tcc/v2/tx"`, bỏ `nonce` và
/// `gas_price`, thêm `recent_blockhash` và `priority_fee`. Ta chỉ hiểu v1, nên
/// gặp phiên bản khác thì **từ chối** — tính băm theo bố cục sai rồi so sánh là
/// một phép kiểm luôn trượt, còn ký theo nó thì tệ hơn nhiều.
pub const TX_VERSION_V1: u32 = 1;

impl Transfer {
    /// Thông điệp ký, tính lại **độc lập** với máy chủ.
    ///
    /// Chép từ `src/tx/signing.rs::signing_message` của chuỗi — tệp tự ghi
    /// *"single source of truth"*. Mọi số little-endian:
    ///
    /// ```text
    /// BLAKE3( "tcc/v1/tx" ‖ version(u32) ‖ chain_id(u64) ‖ from(32) ‖ to(32)
    ///         ‖ nonce(u64) ‖ amount(u128) ‖ gas_price(u64) ‖ gas_limit(u64)
    ///         ‖ timestamp(i64) ‖ expires_at(i64) ‖ BLAKE3(0x01 ‖ memo) )
    /// ```
    ///
    /// Neo bằng một giao dịch THẬT lấy từ testnet — xem `khop_giao_dich_that`.
    /// Không có mốc ấy thì đây chỉ là cách tôi ĐỌC mã của chuỗi, và đọc sai thì
    /// trình duyệt từ chối mọi giao dịch hợp lệ.
    #[must_use]
    pub fn signing_message(&self) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(DOMAIN_V1);
        h.update(&self.version.to_le_bytes());
        h.update(&self.chain_id.to_le_bytes());
        h.update(&self.from.0);
        h.update(&self.to.0);
        h.update(&self.nonce.to_le_bytes());
        h.update(&self.amount.to_le_bytes());
        h.update(&self.gas_price.to_le_bytes());
        h.update(&self.gas_limit.to_le_bytes());
        h.update(&self.timestamp.to_le_bytes());
        h.update(&self.expires_at.to_le_bytes());
        h.update(&payload_hash(&self.memo));
        *h.finalize().as_bytes()
    }

    /// Giải mã `unsigned_tx_hex` / `unsigned_tx_base64` đã bỏ base64.
    ///
    /// Bố cục dây, mọi số little-endian:
    ///
    /// ```text
    /// version(u32) chain_id(u64) from(32) to(32) nonce(u64) amount(u128)
    /// gas_price(u64) gas_limit(u64) timestamp(i64) expires_at(i64)
    /// payload: bien_the(u32) do_dai(u64) byte
    /// ```
    ///
    /// # Errors
    /// Dữ liệu cụt, thừa byte, payload không phải chuyển tiền, hoặc memo hỏng.
    pub fn decode(bytes: &[u8]) -> Result<Self, ChainError> {
        let mut r = Doc::new(bytes);
        let version = r.u32()?;
        if version != TX_VERSION_V1 {
            return Err(ChainError::UnsupportedVersion(version, TX_VERSION_V1));
        }
        let chain_id = r.u64()?;
        let from = Address(r.mang32()?);
        let to = Address(r.mang32()?);
        let nonce = r.u64()?;
        let amount = r.u128()?;
        let gas_price = r.u64()?;
        let gas_limit = r.u64()?;
        let timestamp = r.i64()?;
        let expires_at = r.i64()?;

        let bien_the = r.u32()?;
        if bien_the != WIRE_TRANSFER {
            return Err(ChainError::NotATransfer(bien_the));
        }
        let memo = r.chuoi()?;

        let con_lai = r.con_lai();
        if con_lai != 0 {
            return Err(ChainError::TrailingBytes(con_lai));
        }

        Ok(Self {
            version,
            chain_id,
            from,
            to,
            nonce,
            amount,
            gas_price,
            gas_limit,
            timestamp,
            expires_at,
            memo,
        })
    }
}

/// Băm payload chuyển tiền: `BLAKE3(TAG_TRANSFER ‖ memo)`.
fn payload_hash(memo: &str) -> [u8; 32] {
    let mut v = Vec::with_capacity(1 + memo.len());
    v.push(TAG_TRANSFER);
    v.extend_from_slice(memo.as_bytes());
    *blake3::hash(&v).as_bytes()
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

    /// Giao dịch THẬT lấy từ RPC testnet ngày 15/08/2026 (chain 91338).
    ///
    /// Đây là MỐC NGOÀI của cả crate. Không có nó, mọi thứ ở đây chỉ là cách
    /// tôi ĐỌC mã của chuỗi — và bản đầu tôi đọc nhầm tệp, dựng theo SDK WASM
    /// cũ với bố cục khác hẳn. Phép thử này bắt được điều đó ngay.
    const TX_HEX: &str = "01000000ca640100000000001111111111111111111111111111111111111111111111111111111111111111222222222222222222222222222222222222222222222222222222222222222200000000000000000000f444829163450000000000000000c40051160b00000008520000000000000000000000000000b67a0200000000000000000004000000000000006368616f";
    const SIGNING_MESSAGE_HEX: &str =
        "05d1f926a92678bea9a8d1aae6a7ef86ae295d7a5811301a065e388da97f5b8a";

    fn tu_hex(h: &str) -> Vec<u8> {
        (0..h.len() / 2)
            .map(|i| u8::from_str_radix(&h[i * 2..i * 2 + 2], 16).unwrap())
            .collect()
    }

    fn mau_that() -> Transfer {
        Transfer::decode(&tu_hex(TX_HEX)).unwrap()
    }

    /// Giải mã một giao dịch thật, rồi tính lại đúng băm máy chủ đã đưa.
    #[test]
    fn khop_giao_dich_that() {
        let t = mau_that();
        assert_eq!(t.version, TX_VERSION_V1);
        assert_eq!(t.chain_id, 91338, "testnet");
        assert_eq!(t.amount, 5_000_000_000_000_000_000, "5 TCC");
        assert_eq!(t.gas_limit, 21_000);
        assert_eq!(t.expires_at, 162_486);
        assert_eq!(t.memo, "chao");

        let cho: [u8; 32] = tu_hex(SIGNING_MESSAGE_HEX).try_into().unwrap();
        assert_eq!(
            t.signing_message(),
            cho,
            "tính lại KHÔNG ra băm mà máy chủ đưa"
        );
        assert!(check_signing_message(&t, &cho).is_ok());
    }

    /// Đổi BẤT KỲ trường nào cũng phải đổi thông điệp ký.
    ///
    /// Một trường không vào băm là một trường máy chủ sửa được mà không ai biết.
    #[test]
    fn moi_truong_deu_vao_thong_diep_ky() {
        let goc = mau_that().signing_message();
        macro_rules! doi {
            ($ten:literal, $sua:expr) => {{
                let mut t = mau_that();
                #[allow(clippy::redundant_closure_call)]
                ($sua)(&mut t);
                assert_ne!(t.signing_message(), goc, concat!($ten, " KHÔNG vào băm"));
            }};
        }
        doi!("version", |t: &mut Transfer| t.version += 1);
        doi!("chain_id", |t: &mut Transfer| t.chain_id += 1);
        doi!("from", |t: &mut Transfer| t.from = Address([9; 32]));
        doi!("NGƯỜI NHẬN", |t: &mut Transfer| t.to =
            Address([9; 32]));
        doi!("nonce", |t: &mut Transfer| t.nonce += 1);
        doi!("SỐ TIỀN", |t: &mut Transfer| t.amount += 1);
        doi!("gas_price", |t: &mut Transfer| t.gas_price += 1);
        doi!("gas_limit", |t: &mut Transfer| t.gas_limit += 1);
        doi!("timestamp", |t: &mut Transfer| t.timestamp += 1);
        doi!("expires_at", |t: &mut Transfer| t.expires_at += 1);
        doi!("memo", |t: &mut Transfer| t.memo.push('!'));
    }

    /// Đòn thật: máy chủ đưa băm của giao dịch NGƯỜI DÙNG NGHĨ mình đang ký,
    /// còn gói tin lại là của kẻ gian. Ví web hôm nay sẽ ký. Đây phải từ chối.
    #[test]
    fn may_chu_doi_nguoi_nhan_thi_bi_bat() {
        let that = mau_that();
        let mut ke_gian = mau_that();
        ke_gian.to = Address([0xEE; 32]);
        let loi = check_signing_message(&ke_gian, &that.signing_message()).unwrap_err();
        assert!(
            matches!(loi, ChainError::SigningMessageMismatch { .. }),
            "{loi}"
        );
    }

    /// Cùng địa chỉ, cùng nonce, KHÁC mạng → khác băm. Không phát lại được.
    #[test]
    fn giao_dich_testnet_khong_phat_lai_duoc_sang_mainnet() {
        let testnet = mau_that();
        let mut mainnet = mau_that();
        mainnet.chain_id = 1;
        assert_ne!(testnet.signing_message(), mainnet.signing_message());
    }

    #[test]
    fn du_lieu_cut_thi_bao_loi_chu_khong_hoang_loan() {
        let b = tu_hex(TX_HEX);
        for n in 0..b.len() {
            let _ = Transfer::decode(&b[..n]);
        }
    }

    #[test]
    fn byte_thua_bi_tu_choi() {
        let mut b = tu_hex(TX_HEX);
        b.push(0);
        assert_eq!(Transfer::decode(&b), Err(ChainError::TrailingBytes(1)));
    }

    /// Phiên bản lạ thì TỪ CHỐI, không đoán bố cục.
    #[test]
    fn phien_ban_la_thi_tu_choi() {
        let mut b = tu_hex(TX_HEX);
        b[0] = 2; // v2 — bố cục khác hẳn
        assert_eq!(
            Transfer::decode(&b),
            Err(ChainError::UnsupportedVersion(2, TX_VERSION_V1))
        );
    }

    #[test]
    fn payload_khong_phai_chuyen_tien_thi_tu_choi() {
        let mut b = tu_hex(TX_HEX);
        b[132] = 3; // đổi biến thể payload trên dây
        assert_eq!(Transfer::decode(&b), Err(ChainError::NotATransfer(3)));
    }
}
