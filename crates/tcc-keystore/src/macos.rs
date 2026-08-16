//! Keychain của macOS — bản cài đặt thật đầu tiên.
//!
//! # Đọc `docs/vi-thiet-ke.md` §1 trước
//!
//! Secure Enclave **không** giữ được khoá TCC (chỉ P-256). Nên tệp này bảo vệ
//! khoá **lúc nằm yên**, và khoá vẫn phải vào bộ nhớ tiến trình lúc ký. Đừng
//! đọc nó thành nhiều hơn thế.
//!
//! # Ba thứ được đặt, và vì sao
//!
//! | Đặt gì | Chặn được gì |
//! |---|---|
//! | `USER_PRESENCE` | Ký lén khi người dùng không có mặt — hệ điều hành hỏi Touch ID hoặc mật khẩu **cho từng lần lấy khoá** |
//! | `set_access_synchronized(Some(false))` | Khoá theo iCloud Keychain sang máy khác |
//! | Không ghi đè | Ghi đè khoá ví là mất tiền vĩnh viễn, không phải mất tiện lợi |
//!
//! Mục đầu là mục đáng giá nhất của cả kho khoá, và là mục dễ quên nhất khi
//! cài đặt — `set_generic_password` trơn không có nó.

use security_framework::passwords::{
    AccessControlOptions, PasswordOptions, delete_generic_password, generic_password,
    set_generic_password_options,
};

use crate::{Keystore, KeystoreError, Purpose, SecretKey};

/// Tên dịch vụ trong Keychain. Đổi nó là mọi khoá cũ thành vô hình.
const SERVICE: &str = "com.tcc.browser.wallet";

/// Kho khoá thật, dựa trên Keychain.
#[derive(Debug, Default)]
pub struct MacKeychain;

impl MacKeychain {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    fn tuy_chon(name: &str, hoi_nguoi_dung: bool) -> PasswordOptions {
        let mut o = PasswordOptions::new_generic_password(SERVICE, name);
        // KHÔNG đồng bộ lên iCloud. Khoá ví rời khỏi máy là rời khỏi tầm bảo vệ
        // của cả danh sách kiểm soát truy cập lẫn FileVault.
        o.set_access_synchronized(Some(false));
        if hoi_nguoi_dung {
            // `USER_PRESENCE`: hệ điều hành tự hỏi Touch ID / mật khẩu. Không
            // dùng `BIOMETRY_ANY` vì máy không có cảm biến sẽ không mở được
            // khoá — mà "không mở được ví" là hỏng nặng hơn "hỏi mật khẩu".
            o.set_access_control_options(AccessControlOptions::USER_PRESENCE);
        }
        o
    }
}

impl Keystore for MacKeychain {
    fn store(&mut self, name: &str, key: SecretKey) -> Result<(), KeystoreError> {
        // Kiểm TRƯỚC bằng đường không cần xác thực. `set_generic_password` của
        // Keychain ghi đè im lặng nếu mục đã tồn tại — và ghi đè khoá ví là
        // mất tiền vĩnh viễn, nên phải tự chặn chứ không trông vào hệ điều hành.
        if self.contains(name) {
            return Err(KeystoreError::AlreadyExists(name.to_owned()));
        }
        set_generic_password_options(key.expose(), Self::tuy_chon(name, true))
            .map_err(|e| KeystoreError::Os(e.to_string()))
    }

    fn unlock(&self, name: &str, purpose: &Purpose) -> Result<SecretKey, KeystoreError> {
        // ⚠️ HẠN CHẾ ĐÃ BIẾT: `purpose.prompt` KHÔNG tới được hộp thoại của hệ
        // điều hành qua API này. macOS hiện câu của riêng nó ("… muốn dùng
        // thông tin đăng nhập"), nên lý do phải được hiện ở màn hình CỦA TA
        // ngay trước khi gọi hàm này — đó là việc của `transaction_screen`.
        //
        // Giữ tham số `purpose` là cố ý: nó ép bên gọi phải NGHĨ ra lý do, và
        // khi nào API cho phép truyền xuống thì không phải đổi chữ ký hàm.
        let _ = purpose;
        let mut o = Self::tuy_chon(name, true);
        o.set_access_control_options(AccessControlOptions::USER_PRESENCE);
        match generic_password(o) {
            Ok(b) => Ok(SecretKey::new(b)),
            Err(e) => {
                let loi = phan_loai(e, name);
                // ⚠️ "Không tìm thấy" ở đây có HAI nghĩa, và chúng khác nhau
                // với người dùng. Đặt `USER_PRESENCE` lúc đọc là một BỘ LỌC:
                // truy vấn chỉ thấy mục đã được CẤT kèm bảo vệ ấy. Một khoá
                // cất bằng `security add-generic-password` vẫn nằm đó và vẫn
                // đọc được bằng truy vấn thường.
                //
                // Nên trước khi nói "không có ví", hỏi lại bằng đường không
                // điều kiện. Có thật thì đó là chuyện khác hẳn — và là chuyện
                // người dùng phải biết.
                if matches!(loi, KeystoreError::NotFound(_)) && self.contains(name) {
                    return Err(KeystoreError::UnprotectedKey(name.to_owned()));
                }
                Err(loi)
            }
        }
    }

    fn contains(&self, name: &str) -> bool {
        // KHÔNG đặt `USER_PRESENCE` ở đây: hỏi Touch ID chỉ để biết "đã có ví
        // chưa" là cách nhanh nhất dạy người dùng chạm bừa mọi hộp thoại.
        generic_password(Self::tuy_chon(name, false)).is_ok()
    }

    fn delete(&mut self, name: &str) -> Result<(), KeystoreError> {
        delete_generic_password(SERVICE, name).map_err(|e| phan_loai(e, name))
    }
}

/// Phân biệt "không có khoá" với "người dùng từ chối" với "hệ điều hành hỏng".
///
/// Gộp cả ba thành một lỗi là giao diện không nói được điều đúng: "chưa có ví"
/// và "bạn vừa bấm huỷ" là hai câu khác nhau với người dùng.
fn phan_loai(e: security_framework::base::Error, name: &str) -> KeystoreError {
    // errSecItemNotFound = -25300; errSecUserCanceled = -128.
    match e.code() {
        -25300 => KeystoreError::NotFound(name.to_owned()),
        -128 => KeystoreError::UserRefused,
        _ => KeystoreError::Os(e.to_string()),
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay"
)]
mod kiem_thu {
    use super::*;

    /// **Khoá cất KHÔNG kèm bảo vệ thì `unlock` phải nói ĐÚNG chuyện ấy.**
    ///
    /// Phát hiện ngày 17/08/2026 khi chạy thật: `security add-generic-password`
    /// cất được khoá, `contains` thấy nó, mà `unlock` báo "không có khoá nào".
    /// Vì đặt `USER_PRESENCE` lúc đọc là một BỘ LỌC, không phải một yêu cầu.
    ///
    /// Phép thử cất một mục KHÔNG kèm bảo vệ rồi đòi `unlock` trả
    /// `UnprotectedKey`. Nó chạm Keychain thật nhưng **không hỏi người dùng**:
    /// mục không có điều khiển truy cập thì đọc không cần xác thực.
    #[test]
    fn khoa_khong_duoc_bao_ve_thi_noi_dung_chuyen_ay() {
        let ten = "tcc-kiem-thu-khoa-khong-bao-ve";
        let k = MacKeychain::new();
        // Dọn trước, phòng lần chạy hỏng để lại.
        let _ = delete_generic_password(SERVICE, ten);

        // Cất KHÔNG kèm `USER_PRESENCE` — đúng thứ `security` CLI làm.
        set_generic_password_options(
            b"12345678901234567890123456789012",
            MacKeychain::tuy_chon(ten, false),
        )
        .expect("cất được mục không kèm bảo vệ");

        let loi = k
            .unlock(ten, &Purpose::new("kiểm thử"))
            .expect_err("phải từ chối");
        assert!(
            matches!(loi, KeystoreError::UnprotectedKey(_)),
            "báo sai loại: {loi}"
        );

        let _ = delete_generic_password(SERVICE, ten);
    }

    /// Đọc một khoá KHÔNG tồn tại phải ra `NotFound`, không ra `Os(...)`.
    ///
    /// Phép thử này chạm Keychain thật nhưng KHÔNG hỏi người dùng, vì mục
    /// không tồn tại thì hệ điều hành trả về trước khi cần xác thực.
    #[test]
    fn khoa_khong_ton_tai_ra_dung_loai_loi() {
        let k = MacKeychain::new();
        assert!(!k.contains("tcc-kiem-thu-khong-bao-gio-ton-tai"));
    }
}
