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

/// Tên mục ĐÁNH DẤU đi kèm một khoá.
///
/// Ký tự `|` không nằm trong tên khoá nào (tên dẫn từ địa chỉ ví, chỉ `0-9a-f`),
/// nên một mục đánh dấu không bao giờ đụng tên một khoá thật.
fn dau_moc(name: &str) -> String {
    format!("{name}|co")
}

/// Kho khoá thật, dựa trên Keychain.
#[derive(Debug, Default)]
pub struct MacKeychain;

impl MacKeychain {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Có MỤC KHOÁ tên này không — hỏi thẳng mục khoá, không qua mục đánh dấu.
    ///
    /// Chỉ dùng ở nhánh dự phòng của [`Self::unlock`], và chỉ khi truy vấn kèm
    /// `USER_PRESENCE` đã trả về "không thấy". Nghĩa là tới đây chắc chắn không
    /// có mục nào được bảo vệ khớp tên — nên lời hỏi này không đụng ACL nào và
    /// không dựng hộp thoại nào.
    ///
    /// KHÔNG gộp vào `contains`: `contains` là câu hỏi của giao diện ("đã có ví
    /// chưa") và phải im lặng tuyệt đối; đây là câu hỏi của phần chẩn đoán
    /// ("có phải khoá nằm đó mà không được bảo vệ không"). Gộp hai câu ấy làm
    /// một chính là chỗ bản trước hỏng.
    fn co_muc_khoa(name: &str) -> bool {
        generic_password(Self::tuy_chon(name, false)).is_ok()
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
            .map_err(|e| KeystoreError::Os(e.to_string()))?;
        // Mục đánh dấu ghi SAU khoá: hỏng ở giữa thì còn khoá mà mất dấu — giao
        // diện nói "chưa có ví" trong khi khoá vẫn an toàn trong Keychain. Ghi
        // ngược lại thì có dấu mà không có khoá, và giao diện mời người dùng mở
        // một cái ví không tồn tại.
        set_generic_password_options(&[1u8], Self::tuy_chon(&dau_moc(name), false))
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
                if matches!(loi, KeystoreError::NotFound(_)) && Self::co_muc_khoa(name) {
                    return Err(KeystoreError::UnprotectedKey(name.to_owned()));
                }
                Err(loi)
            }
        }
    }

    fn contains(&self, name: &str) -> bool {
        // ⚠️ Đọc MỤC ĐÁNH DẤU, không đụng tới mục giữ khoá.
        //
        // Hai bản trước đều sai, và sai theo hai kiểu khác nhau:
        //
        // 1. Gọi `generic_password(...)` với `USER_PRESENCE` tắt, tin rằng
        //    không xin quyền thì không bị hỏi. Cờ ấy chỉ nói về TRUY VẤN; danh
        //    sách kiểm soát truy cập nằm trên CHÍNH MỤC đã cất.
        // 2. Hỏi THUỘC TÍNH thay vì dữ liệu. Vẫn hỏi Touch ID — macOS phải
        //    thoả ACL mới xét được mục có khớp hay không.
        //
        // Đo được 22/08/2026, in từng bước: `contains` TRƯỚC khi cất trả `false`
        // và im lặng; SAU khi cất thì treo, và `coreautha` dựng hộp thoại.
        //
        // `kSecUseAuthenticationUIFail` — cách chuẩn để hỏi "có tồn tại không"
        // mà không hiện hộp thoại — thì `security-framework` 3.7 KHÔNG phơi ra;
        // thứ nó phơi ra là `skip_authenticated_items`, làm điều ngược lại: bỏ
        // qua mục cần xác thực, tức là báo "chưa có ví" trong khi có.
        //
        // Nên: cất khoá thì cất kèm một MỤC ĐÁNH DẤU không khoá. Nó không giữ
        // bí mật nào — nội dung là một byte — và nó trả lời đúng câu `contains`
        // hỏi: "đã có ví chưa". Thêm một `unsafe` thứ hai chỉ để đỡ một hộp
        // thoại là đánh đổi tồi hơn hẳn.
        generic_password(Self::tuy_chon(&dau_moc(name), false)).is_ok()
    }

    fn delete(&mut self, name: &str) -> Result<(), KeystoreError> {
        // Xoá dấu TRƯỚC: nếu xoá khoá hỏng giữa chừng thì thà mất dấu còn hơn
        // để lại một cái dấu trỏ vào chỗ trống.
        let _ = delete_generic_password(SERVICE, &dau_moc(name));
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
    /// `UnprotectedKey`.
    ///
    /// ⚠️ **Chú thích cũ ở đây nói SAI, và không ai kiểm lời ấy.** Nó viết:
    /// "chạm Keychain thật nhưng không hỏi người dùng — mục không có điều khiển
    /// truy cập thì đọc không cần xác thực". Ngày 26/08/2026 phép thử này làm
    /// treo cổng `kiem-theo-co.sh` **hơn bốn mươi phút**: macOS bật hộp thoại
    /// "…muốn dùng thông tin mật trong chuỗi khoá của bạn" và ngồi chờ một cú
    /// bấm. Lý do không phải điều khiển truy cập mà là **danh sách ứng dụng**
    /// của mục: nhị phân phép thử dựng lại là một chương trình khác đối với
    /// Keychain, nên nó phải xin phép.
    ///
    /// Cùng khuôn với `bad-key` (SECURITY.md §3.18): một lời giải thích nghe
    /// hợp lý, chưa ai kiểm, và thực tế bác bỏ.
    ///
    /// Nên nó thành `#[ignore]`. Một phép thử đòi người bấm chuột mà nằm trong
    /// cổng tự động thì hoặc treo cổng, hoặc dạy người ta bỏ qua cổng.
    ///
    /// ⚠️ `#[ignore]` chứ KHÔNG phải "trả về sớm khi thiếu biến môi trường".
    /// Trả về sớm là một phép thử XANH GIẢ: dòng kết quả ghi "ok, 10 passed" y
    /// hệt lúc nó chạy thật. `#[ignore]` thì cargo ĐẾM RA — "9 passed; 1
    /// ignored" — nên người đọc thấy ngay là có thứ chưa chạy.
    ///
    /// Chạy có chủ đích:
    /// `cargo test -p tcc-keystore --features os-keystore -- --ignored`
    ///
    /// ⚠️ Lời giải ĐÚNG chưa làm: dựng một **Keychain tạm** có mật khẩu biết
    /// trước rồi mở sẵn, thay vì dùng chuỗi khoá đăng nhập. Việc ấy đòi
    /// `MacKeychain` nhận vào một chuỗi khoá thay vì luôn dùng mặc định — xem
    /// SECURITY.md §3.28.
    #[test]
    #[ignore = "hỏi người dùng qua hộp thoại Keychain — chạy bằng `-- --ignored`"]
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

    /// **Ba mã trạng thái, ba câu KHÁC NHAU với người dùng.**
    ///
    /// Chú thích của `phan_loai` nói rõ vì sao nó tồn tại: "chưa có ví" và
    /// "bạn vừa bấm huỷ" là hai câu khác nhau. Nhưng tới 26/08/2026 không phép
    /// thử nào đọc kết quả của nó — kiểm đột biến xoá hẳn nhánh `-128` mà mọi
    /// phép thử vẫn xanh, tức là một người bấm HUỶ sẽ được báo "hệ điều hành
    /// hỏng", và họ sẽ đi tìm lỗi ở máy mình.
    ///
    /// Hàm này THUẦN nên kiểm được không cần Keychain thật — khác hẳn phần còn
    /// lại của tệp này, xem ghi chú ở SECURITY.md §3.28.
    #[test]
    fn ba_ma_trang_thai_ra_ba_loi_khac_nhau() {
        use security_framework::base::Error;

        assert!(
            matches!(
                phan_loai(Error::from_code(-25300), "vi"),
                KeystoreError::NotFound(_)
            ),
            "errSecItemNotFound phải thành 'chưa có ví'"
        );
        assert!(
            matches!(
                phan_loai(Error::from_code(-128), "vi"),
                KeystoreError::UserRefused
            ),
            "errSecUserCanceled phải thành 'người dùng từ chối', không phải lỗi hệ điều hành"
        );
        assert!(
            matches!(phan_loai(Error::from_code(-1), "vi"), KeystoreError::Os(_)),
            "mã lạ phải thành lỗi hệ điều hành, không được nuốt thành 'chưa có ví'"
        );
    }
}
