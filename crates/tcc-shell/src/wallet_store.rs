//! Chỗ ví thật được cất — **và chỗ từ chối cất nếu không cất tử tế được**.
//!
//! # Không có đường lùi, và đó là cả nội dung tệp này
//!
//! Nếu máy không có kho khoá của hệ điều hành thì [`open`] trả **lỗi**. Không
//! ghi ra tệp, không mã hoá bằng mật khẩu tự nghĩ ra, không giữ trong bộ nhớ
//! rồi hy vọng.
//!
//! Lý do: ghi khoá ra tệp *"tạm thời cho chạy được"* là đúng thứ trình duyệt
//! này sinh ra để không làm. Ví web buộc phải làm thế vì trang web không có
//! lựa chọn nào khác — trình duyệt thì có, và bỏ lựa chọn ấy đi để "đỡ vướng"
//! là vứt luôn lý do người dùng đổi sang đây.
//!
//! Một tính năng **không chạy** thì người dùng thấy ngay và đi tìm cách khác.
//! Một tính năng chạy mà bảo vệ kém hơn họ tưởng thì họ không thấy gì cả, cho
//! tới lúc mất tiền.
//!
//! # Kho khoá GIẢ không bao giờ đi ra từ đây
//!
//! `tcc_keystore::fake` có sẵn và tiện — và chính vì tiện nên nó nguy hiểm.
//! Nó nằm sau `cfg(test)` của crate ấy, và tệp này không nhắc tới nó ở bất kỳ
//! nhánh nào. Có phép thử chốt rằng trên nền không có kho khoá, [`open`] trả
//! lỗi chứ không trả một thứ chạy được.

use tcc_keystore::{Keystore, Purpose};

/// Câu lỗi này có phải chuyện "gói chưa ký" không?
///
/// macOS trả về `A required entitlement isn't present` khi bản dựng thiếu
/// quyền `keychain-access-groups`. Đó là câu nói với LẬP TRÌNH VIÊN; người dùng
/// đọc nó chỉ tưởng mình vừa gõ sai gì đó.
///
/// Nhận dạng bằng chuỗi vì thư viện không cho mã lỗi riêng cho trường hợp này.
/// Chuỗi đổi thì phép nhận dạng lặng lẽ hỏng — nên bên gọi vẫn phải hiện câu
/// gốc kèm theo, và có phép thử ghim đúng chuỗi đang nhận.
#[must_use]
pub fn is_unsigned_build(loi: &str) -> bool {
    loi.contains("entitlement")
}

/// Vì sao không cất được. Chỉ có một lý do, và nó không sửa được bằng cách thử lại.
#[derive(Debug, thiserror::Error)]
pub enum WalletStoreError {
    /// Nền này chưa có bản cài đặt kho khoá — hoặc cờ `os-keystore` đang tắt.
    #[error("nền này chưa có kho khoá hệ điều hành; ví không dùng được ở đây")]
    NoKeystore,
}

/// Tên mục trong kho khoá, dẫn từ địa chỉ ví.
///
/// Dùng địa chỉ chứ không dùng nhãn người dùng đặt: nhãn đổi được, và đổi nhãn
/// mà mất khoá là một cách hỏng không ai đoán ra.
#[must_use]
pub fn key_name(address: &str) -> String {
    format!("wallet-{address}")
}

/// Lý do hiện cho người dùng trước khi mở khoá ví để ký.
///
/// ⚠️ Chuỗi này **không** tới được hộp thoại của macOS — API không cho truyền
/// xuống. Nó phải được hiện trên màn hình CỦA TA ngay trước khi gọi `unlock`;
/// xem `docs/vi-thiet-ke.md` §10.
#[must_use]
pub fn signing_purpose(address: &str) -> Purpose {
    Purpose::new(format!("Ký một giao dịch bằng ví {address}"))
}

/// Mở kho khoá thật của hệ điều hành.
///
/// # Errors
/// [`WalletStoreError::NoKeystore`] khi nền này chưa có bản cài đặt. Đây **không
/// phải** lỗi tạm thời: thử lại không đổi gì, và không có đường lùi nào cả.
#[cfg(all(feature = "os-keystore", target_os = "macos"))]
pub fn open() -> Result<Box<dyn Keystore>, WalletStoreError> {
    Ok(Box::new(tcc_keystore::macos::MacKeychain::new()))
}

/// Nền chưa có kho khoá — Windows (DPAPI) và Linux còn phải viết.
///
/// # Errors
/// Luôn [`WalletStoreError::NoKeystore`]. Xem ghi chú đầu tệp về việc vì sao ở
/// đây không có đường lùi.
#[cfg(not(all(feature = "os-keystore", target_os = "macos")))]
pub fn open() -> Result<Box<dyn Keystore>, WalletStoreError> {
    Err(WalletStoreError::NoKeystore)
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay"
)]
mod kiem_thu {
    use super::*;

    const DIA_CHI: &str = "0x11b22b300e195c44c910d71cdb1515c4617e852393cde5e80c860906b8a2d549";

    /// Tên mục dẫn từ ĐỊA CHỈ, không dẫn từ nhãn người dùng đặt.
    #[test]
    fn ten_muc_dan_tu_dia_chi() {
        assert!(key_name(DIA_CHI).contains(DIA_CHI));
        assert_ne!(key_name(DIA_CHI), key_name("0xkhac"));
    }

    /// **Câu lỗi "chưa ký gói" phải nhận ra được.**
    ///
    /// Nếu không, người dùng đọc câu của macOS và tưởng mình gõ sai cụm từ.
    #[test]
    fn nhan_ra_duoc_loi_chua_ky_goi() {
        assert!(is_unsigned_build(
            "kho khoá của hệ điều hành báo lỗi: A required entitlement isn't present."
        ));
        assert!(!is_unsigned_build("sai PIN, hoặc dữ liệu đã hỏng"));
        assert!(!is_unsigned_build("không có khoá nào tên \"wallet-0x…\""));
    }

    /// Lý do phải NÓI RA việc gì sắp xảy ra, không phải "cần xác thực".
    #[test]
    fn ly_do_noi_ro_viec_gi() {
        let p = signing_purpose(DIA_CHI);
        assert!(p.prompt.contains(DIA_CHI), "{}", p.prompt);
        assert!(p.prompt.contains("Ký"), "{}", p.prompt);
    }

    /// **Không có kho khoá thì trả LỖI, không trả một thứ chạy được.**
    ///
    /// Phép thử này là cả điểm của tệp. Ngày ai đó thêm một nhánh "tạm thời
    /// ghi ra tệp cho chạy được", nó đỏ.
    #[cfg(not(all(feature = "os-keystore", target_os = "macos")))]
    #[test]
    fn khong_co_kho_khoa_thi_tu_choi_chu_khong_lui() {
        // `unwrap_err` cần `Debug` trên nhánh Ok, mà `dyn Keystore` cố ý không
        // có — một kho khoá in ra được là một kho khoá có thể in nhầm thứ
        // không nên in. Nên khớp bằng `match`, không dùng `unwrap_err`.
        let Err(loi) = open() else {
            panic!("nền không có kho khoá mà `open` vẫn trả về một kho khoá");
        };
        assert!(matches!(loi, WalletStoreError::NoKeystore), "{loi}");
    }

    /// Trên macOS có cờ thì mở được kho khoá THẬT.
    ///
    /// Chỉ mở, **không** đọc khoá: đọc là hệ điều hành hỏi Touch ID, mà máy chạy
    /// kiểm thử thì không có ai chạm vào — cùng lý do cổng gõ tiếng Việt cần
    /// một con người.
    #[cfg(all(feature = "os-keystore", target_os = "macos"))]
    #[test]
    fn tren_macos_mo_duoc_kho_khoa_that() {
        let kho = open().expect("macOS có cờ thì phải mở được");
        assert!(
            !kho.contains(&key_name("0xkhong-bao-gio-ton-tai")),
            "kho khoá báo có một ví chưa từng cất"
        );
    }
}
