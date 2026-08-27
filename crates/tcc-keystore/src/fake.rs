//! Kho khoá GIẢ — trong bộ nhớ, không chạm hệ điều hành.
//!
//! Tồn tại để phần logic của ví kiểm được trên cả ba hệ điều hành mà không cần
//! Keychain, và để phép thử chạy được trong CI nơi không có ai chạm Touch ID.
//!
//! ⚠️ Nó **không** giả vờ an toàn. Khoá nằm trong một `HashMap`. Đừng dùng nó
//! ở đâu ngoài phép thử — và luật kiến trúc sẽ chặn nếu ai thử.

use std::collections::HashMap;

use crate::{Keystore, KeystoreError, Purpose, SecretKey};

/// Kho khoá trong bộ nhớ, có ghi lại từng lần bị hỏi.
#[derive(Default)]
pub struct FakeKeystore {
    keys: HashMap<String, Vec<u8>>,
    /// Người dùng có đồng ý không. Đặt `false` để kiểm nhánh từ chối.
    pub user_approves: bool,
    /// Mọi lý do đã hiện ra, theo thứ tự.
    ///
    /// Phép thử đòi được rằng lý do ĐẾN ĐƯỢC người dùng, chứ không chỉ được
    /// truyền vào rồi rơi mất — đó là khác biệt giữa hỏi và giả vờ hỏi.
    pub prompts: std::cell::RefCell<Vec<String>>,
}

impl FakeKeystore {
    #[must_use]
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            user_approves: true,
            prompts: std::cell::RefCell::new(Vec::new()),
        }
    }

    /// Người dùng sẽ từ chối mọi lần hỏi.
    #[must_use]
    pub fn refusing(mut self) -> Self {
        self.user_approves = false;
        self
    }
}

impl Keystore for FakeKeystore {
    fn store(&mut self, name: &str, key: SecretKey) -> Result<(), KeystoreError> {
        if self.keys.contains_key(name) {
            return Err(KeystoreError::AlreadyExists(name.to_owned()));
        }
        self.keys.insert(name.to_owned(), key.expose().to_vec());
        Ok(())
    }

    fn unlock(&self, name: &str, purpose: &Purpose) -> Result<SecretKey, KeystoreError> {
        // Ghi lại TRƯỚC khi quyết định: một bản cài đặt hỏi rồi mới kiểm có
        // khoá hay không sẽ làm phiền người dùng vì một khoá không tồn tại.
        self.prompts.borrow_mut().push(purpose.prompt.clone());
        if !self.user_approves {
            return Err(KeystoreError::UserRefused);
        }
        self.keys
            .get(name)
            .map(|b| SecretKey::new(b.clone()))
            .ok_or_else(|| KeystoreError::NotFound(name.to_owned()))
    }

    fn contains(&self, name: &str) -> bool {
        self.keys.contains_key(name)
    }

    fn delete(&mut self, name: &str) -> Result<(), KeystoreError> {
        self.keys
            .remove(name)
            .map(|_| ())
            .ok_or_else(|| KeystoreError::NotFound(name.to_owned()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "kiểm thử: hỏng thì phải nổ ngay")]
mod kiem_thu_gia {
    use super::*;

    /// **Kho khoá GIẢ phải xoá thật, và `refusing()` phải thật sự từ chối.**
    ///
    /// Kiểm đột biến 27/08/2026: `delete → Ok(())` và `refusing → default` đều
    /// sống. Nghe như chuyện của riêng bộ thử — nhưng kho giả này là nền cho
    /// phép thử của các hòm KHÁC. Một `delete` báo thành công mà không xoá gì
    /// làm mọi phép thử "thu hồi rồi thì không dùng được nữa" xanh giả.
    #[test]
    fn kho_gia_xoa_that_va_tu_choi_that() {
        let mut k = FakeKeystore::new();
        k.store("a", SecretKey::new(vec![1u8; 32])).unwrap();
        assert!(k.contains("a"));

        k.delete("a").expect("xoá được");
        assert!(!k.contains("a"), "`delete` báo thành công mà khoá vẫn còn");
        assert!(
            k.delete("a").is_err(),
            "xoá thứ không có phải BÁO LỖI, không được im lặng thành công"
        );

        // `refusing()` là bộ CHỈNH, không phải bộ dựng lại: nó phải GIỮ khoá đã
        // cất. `#[derive(Default)]` cho `user_approves: false` — đúng bằng thứ
        // `refusing()` đặt — nên đột biến `refusing → Default::default()` vẫn
        // "từ chối" và chỉ khác ở chỗ nó VỨT SẠCH khoá. Kiểm "có từ chối không"
        // thôi thì không phân biệt được; phải kiểm cả "khoá còn không".
        let mut tu_choi = FakeKeystore::new();
        tu_choi.store("b", SecretKey::new(vec![2u8; 32])).unwrap();
        let tu_choi = tu_choi.refusing();
        assert!(
            tu_choi.contains("b"),
            "`refusing()` làm mất khoá đã cất — nó là bộ chỉnh, không phải bộ \
             dựng lại"
        );
        assert!(
            tu_choi.unlock("b", &Purpose::new("kiểm thử")).is_err(),
            "`refusing()` phải làm `unlock` hỏng — nếu nó chỉ trả bản mặc định \
             thì mọi phép thử 'người dùng từ chối' đang kiểm nhầm nhánh"
        );
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "kiểm thử: hỏng thì phải nổ ngay")]
mod kiem_thu {
    use super::*;

    #[test]
    fn cat_roi_lay_ra_duoc() {
        let mut k = FakeKeystore::new();
        k.store("vi", SecretKey::new(vec![7; 64])).unwrap();
        let ra = k.unlock("vi", &Purpose::new("ký thử")).unwrap();
        assert_eq!(ra.expose(), &[7u8; 64]);
    }

    /// Ghi đè khoá ví là MẤT TIỀN, không phải mất tiện lợi.
    #[test]
    fn khong_ghi_de_khoa_cu() {
        let mut k = FakeKeystore::new();
        k.store("vi", SecretKey::new(vec![1; 64])).unwrap();
        let loi = k.store("vi", SecretKey::new(vec![2; 64])).unwrap_err();
        assert_eq!(loi, KeystoreError::AlreadyExists("vi".to_owned()));
        // Và khoá CŨ phải còn nguyên.
        let ra = k.unlock("vi", &Purpose::new("kiểm")).unwrap();
        assert_eq!(ra.expose()[0], 1, "khoá cũ bị đè mất");
    }

    #[test]
    fn nguoi_dung_tu_choi_thi_khong_lay_duoc_khoa() {
        let mut k = FakeKeystore::new();
        k.store("vi", SecretKey::new(vec![9; 64])).unwrap();
        let k = FakeKeystore {
            keys: k.keys,
            user_approves: false,
            prompts: std::cell::RefCell::new(Vec::new()),
        };
        assert_eq!(
            k.unlock("vi", &Purpose::new("ký")).unwrap_err(),
            KeystoreError::UserRefused
        );
    }

    /// Lý do phải ĐẾN ĐƯỢC người dùng, không chỉ được truyền vào rồi rơi mất.
    #[test]
    fn ly_do_hien_ra_cho_nguoi_dung() {
        let mut k = FakeKeystore::new();
        k.store("vi", SecretKey::new(vec![3; 64])).unwrap();
        let _ = k.unlock("vi", &Purpose::new("Ký giao dịch gửi 5 TCC"));
        assert_eq!(k.prompts.borrow().len(), 1, "không hỏi người dùng lần nào");
        assert!(k.prompts.borrow()[0].contains("5 TCC"));
    }

    /// `contains` KHÔNG được làm phiền người dùng.
    ///
    /// Bắt chạm Touch ID chỉ để biết "bạn đã có ví chưa" là cách nhanh nhất
    /// dạy người dùng chạm bừa mỗi khi thấy hộp thoại.
    #[test]
    fn contains_khong_hoi_nguoi_dung() {
        let mut k = FakeKeystore::new();
        k.store("vi", SecretKey::new(vec![4; 64])).unwrap();
        assert!(k.contains("vi"));
        assert!(!k.contains("khong-co"));
        assert!(k.prompts.borrow().is_empty(), "contains đã hỏi người dùng");
    }
}
