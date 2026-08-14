//! Nhớ quyền người dùng đã cấp, giữa các lần mở trình duyệt.
//!
//! # Vì sao cần
//!
//! Hỏi lại mỗi lần chạy là cách nhanh nhất khiến người dùng bấm bừa. Một hộp
//! thoại người ta đọc là một hộp thoại có tác dụng; hộp thoại thứ mười trong
//! ngày thì không.
//!
//! # Ba điều dễ làm sai, và làm sai thì thành lỗ hổng
//!
//! ### 1. Nhớ theo mã ứng dụng là KHÔNG ĐỦ
//!
//! Mã ứng dụng do chính ứng dụng khai. Kẻ gian ship một gói mang `com.tcc.vi`
//! và thừa hưởng mọi quyền người dùng từng cấp cho ví thật. Nên bản ghi kèm
//! **khoá công khai của người ký**; khoá đổi là coi như ứng dụng khác.
//!
//! ### 2. Quyền phải gắn với PHẠM VI, không chỉ với tên quyền
//!
//! Bản 1.0 xin `network: [shop.tcc-coin.com]`, người dùng đồng ý. Bản 1.1 xin
//! `network: [shop.tcc-coin.com, thu-thap.example]`. Nhớ theo tên quyền thì
//! quyền cũ **tự động phủ lên phạm vi mới** — người dùng chưa bao giờ đồng ý
//! với máy chủ thứ hai.
//!
//! Nên bản ghi kèm **vân tay của phạm vi**. Phạm vi đổi một ký tự là hỏi lại.
//!
//! ### 3. Mọi thứ không rõ ràng đều phải HỎI LẠI
//!
//! Tệp thiếu, tệp hỏng, phiên bản lạ, vân tay không khớp — tất cả ra "chưa có
//! câu trả lời", tức là hỏi lại. Không có nhánh nào ngả về "cho phép".
//!
//! # Điều tệp này KHÔNG bảo vệ được
//!
//! Người sửa được tệp này là người đã có quyền vào tài khoản của người dùng —
//! lúc đó họ đọc được cả kho khoá lẫn dữ liệu duyệt web. Tệp ghi quyền 0600 và
//! ghi qua tệp tạm rồi đổi tên, nhưng đó là chống hỏng nửa chừng và chống người
//! dùng khác trên cùng máy, KHÔNG phải chống kẻ đã chiếm được tài khoản.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tcc_capability::Decision;
use tcc_crypto::content_hash_hex;
use tcc_spec::{CapabilityRequest, Manifest, Scope};

/// Phiên bản định dạng tệp. Đọc thấy số khác là **bỏ hết và hỏi lại**, không cố
/// đoán — đoán sai ở đây nghĩa là cấp nhầm quyền.
const PHIEN_BAN: u32 = 2;

#[derive(Debug, Default, Serialize, Deserialize)]
struct Kho {
    phien_ban: u32,
    #[serde(default)]
    ung_dung: BTreeMap<String, MucUngDung>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MucUngDung {
    /// Khoá công khai của người ký, dạng hex. Đổi là coi như ứng dụng khác.
    publisher: String,
    #[serde(default)]
    quyen: BTreeMap<String, MucQuyen>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MucQuyen {
    /// Vân tay của PHẠM VI lúc người dùng trả lời. Phạm vi đổi là hỏi lại.
    ///
    /// **Đây là thứ DUY NHẤT quyết định.** Xem `mo_ta` bên dưới.
    van_tay: String,
    /// `true` = đã cho phép. Từ chối cũng được nhớ, để không hỏi lại mãi.
    cho_phep: bool,
    /// Mô tả phạm vi bằng chữ, **CHỈ ĐỂ HIỆN RA**, không bao giờ để quyết định.
    ///
    /// # Vì sao tách bạch điều này thành một câu riêng
    ///
    /// Màn hình quản lý quyền cần hiện "shop.tcc-coin.com", mà vân tay thì
    /// không đọc ngược ra được. Nên phải lưu thêm chữ.
    ///
    /// Nhưng chữ lưu trên đĩa là chữ **sửa được**. Ai sửa tệp có thể làm màn
    /// hình hiện "shop.tcc-coin.com" trong khi vân tay ứng với một phạm vi khác
    /// hẳn — tức là màn hình quản lý quyền NÓI DỐI, đúng cái màn hình mà lời nói
    /// dối gây hại nhất.
    ///
    /// Ta chấp nhận rủi ro đó vì nó nằm trong mô hình đe doạ đã ghi: người sửa
    /// được tệp này đã chiếm được tài khoản người dùng rồi. Nhưng phải chặn
    /// đường nó ảnh hưởng tới QUYẾT ĐỊNH — nên `tra()` không đọc trường này, và
    /// có phép thử chốt điều đó.
    #[serde(default)]
    mo_ta: String,
}

/// Vân tay của một phạm vi quyền năng.
///
/// # Vì sao không băm thẳng JSON
///
/// JSON không có dạng chuẩn tắc: thứ tự khoá, khoảng trắng, cách viết số đều
/// thay đổi được mà nghĩa không đổi. Băm JSON nghĩa là cùng một phạm vi cho ra
/// hai vân tay khác nhau — và lúc đó người dùng bị hỏi lại vô cớ, rồi ai đó sẽ
/// "sửa" bằng cách nới lỏng phép so.
///
/// Nên dùng đúng lối của `tcc_spec::tree`: **tiền tố độ dài cho mọi trường**.
/// Không có tiền tố thì `["ab","c"]` và `["a","bc"]` cho cùng một chuỗi byte.
#[must_use]
pub fn scope_fingerprint(scope: &Scope) -> String {
    let mut b = Vec::new();
    match scope {
        Scope::Network { hosts } => {
            b.push(1u8);
            // Sắp xếp: `[a,b]` và `[b,a]` là CÙNG một phạm vi, phải cùng vân tay.
            let mut ds: Vec<String> = hosts.iter().map(|h| h.to_ascii_lowercase()).collect();
            ds.sort();
            ds.dedup();
            b.extend_from_slice(&(ds.len() as u64).to_be_bytes());
            for h in ds {
                b.extend_from_slice(&(h.len() as u64).to_be_bytes());
                b.extend_from_slice(h.as_bytes());
            }
        }
        Scope::Storage { quota_bytes } => {
            b.push(2u8);
            b.extend_from_slice(&quota_bytes.to_be_bytes());
        }
        Scope::Wallet {
            may_request_signature,
        } => {
            b.push(3u8);
            b.push(u8::from(*may_request_signature));
        }
    }
    content_hash_hex(&b)
}

/// Ứng dụng này, so với lần trước ta thấy nó.
///
/// # Đây là TIN-LẦN-ĐẦU, không phải chứng minh danh tính
///
/// Nó KHÔNG trả lời "gói này có đúng của nhà phát hành X không" — chưa có tầng
/// nào trả lời được câu đó. Nó trả lời một câu hẹp hơn nhiều nhưng vẫn đáng
/// giá: **"khoá ký lần này có giống lần trước không"**.
///
/// Câu hẹp đó bắt được đúng một tình huống, và là tình huống nguy hiểm nhất:
/// một gói mang mã ứng dụng forget thuộc nhưng ký bằng khoá lạ. Không cảnh báo
/// thì người dùng chỉ thấy hộp thoại hỏi quyền hiện lại như lần đầu, và không
/// có cách nào biết ứng dụng đã đổi tay.
///
/// Chữ hiện lên phải là **sự thật quan sát được**, không phải phán quyết:
/// "trước đây ký bằng khoá khác", chứ không phải "ứng dụng này giả mạo".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignerStatus {
    /// Chưa từng thấy mã ứng dụng này. Bình thường, không cảnh báo gì.
    LanDau,
    /// Đúng khoá đã thấy lần trước.
    KhopKhoaCu,
    /// ⚠️ Cùng mã ứng dụng, KHÁC khoá ký.
    DoiKhoa {
        /// Vân tay khoá cũ, để hiện cho người dùng đối chiếu nếu họ muốn.
        van_tay_cu: String,
    },
}

/// Vân tay ngắn của một khoá công khai, để người dùng đối chiếu.
///
/// Khoá lai dài gần 4000 ký tự hex — hiện cả ra là không ai đọc. Lấy đầu và
/// cuối: đủ để so bằng mắt, và kẻ gian muốn khớp cả hai đầu thì phải phá băm
/// chứ không chỉ mò thêm vài byte.
#[must_use]
pub fn key_fingerprint(hex: &str) -> String {
    if hex.len() <= 20 {
        return hex.to_owned();
    }
    format!("{}…{}", &hex[..10], &hex[hex.len() - 10..])
}

/// Một ứng dụng trong danh sách quản lý quyền.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredEntry {
    pub ma_ung_dung: String,
    pub key_fingerprint: String,
    pub quyen: Vec<AnsweredPermission>,
}

/// Một quyền đã có câu trả lời.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnsweredPermission {
    pub ten: String,
    /// Chữ mô tả phạm vi — **chỉ để hiện**, xem `MucQuyen::mo_ta`.
    pub mo_ta: String,
    pub cho_phep: bool,
}

/// Kho quyền đã nhớ.
#[derive(Debug)]
pub struct PermissionStore {
    duong_dan: PathBuf,
    kho: Kho,
}

impl PermissionStore {
    /// Mở kho. Tệp thiếu hoặc hỏng thì bắt đầu từ kho RỖNG — tức là hỏi lại mọi
    /// thứ. Không có nhánh nào ngả về "cho phép".
    #[must_use]
    pub fn open(duong_dan: &Path) -> Self {
        let kho = std::fs::read(duong_dan)
            .ok()
            .and_then(|b| serde_json::from_slice::<Kho>(&b).ok())
            .filter(|k| k.phien_ban == PHIEN_BAN)
            .unwrap_or(Kho {
                phien_ban: PHIEN_BAN,
                ung_dung: BTreeMap::new(),
            });
        Self {
            duong_dan: duong_dan.to_path_buf(),
            kho,
        }
    }

    /// So khoá ký lần này với lần trước.
    ///
    /// Phải gọi TRƯỚC [`PermissionStore::nho`] — `nho` ghi đè khoá mới lên, và sau đó
    /// không còn gì để so nữa.
    #[must_use]
    pub fn signer_status(&self, m: &Manifest) -> SignerStatus {
        match self.kho.ung_dung.get(m.id.as_str()) {
            None => SignerStatus::LanDau,
            Some(muc) if muc.publisher == m.publisher => SignerStatus::KhopKhoaCu,
            Some(muc) => SignerStatus::DoiKhoa {
                van_tay_cu: key_fingerprint(&muc.publisher),
            },
        }
    }

    /// Tra câu trả lời đã nhớ cho MỘT quyền.
    ///
    /// Trả `None` nghĩa là **chưa có câu trả lời** — phải hỏi người dùng. Đó là
    /// kết quả của mọi trường hợp không rõ ràng: chưa từng hỏi, khoá người ký
    /// đổi, hoặc phạm vi đổi.
    #[must_use]
    pub fn lookup(&self, m: &Manifest, xin: &CapabilityRequest) -> Option<Decision> {
        let muc = self.kho.ung_dung.get(m.id.as_str())?;
        // Khoá người ký đổi → coi như ứng dụng khác, không thừa hưởng gì.
        if muc.publisher != m.publisher {
            return None;
        }
        let q = muc.quyen.get(&xin.name)?;
        // Phạm vi đổi → câu trả lời cũ không áp cho phạm vi mới.
        if q.van_tay != scope_fingerprint(&xin.scope) {
            return None;
        }
        Some(if q.cho_phep {
            Decision::Allow
        } else {
            Decision::Deny
        })
    }

    /// Ghi nhớ câu trả lời cho một quyền.
    /// Mô tả phạm vi bằng chữ, để hiện trên màn hình quản lý quyền.
    fn mo_ta_pham_vi(scope: &Scope) -> String {
        match scope {
            Scope::Network { hosts } => hosts.join(", "),
            Scope::Storage { quota_bytes } => format!("{} KB", quota_bytes / 1024),
            Scope::Wallet {
                may_request_signature,
            } => if *may_request_signature {
                "được xin chữ ký giao dịch"
            } else {
                "chỉ đọc địa chỉ"
            }
            .to_owned(),
        }
    }

    pub fn remember(&mut self, m: &Manifest, xin: &CapabilityRequest, qd: Decision) {
        let muc = self
            .kho
            .ung_dung
            .entry(m.id.as_str().to_owned())
            .or_insert_with(|| MucUngDung {
                publisher: m.publisher.clone(),
                quyen: BTreeMap::new(),
            });
        // Khoá người ký đổi → XOÁ SẠCH mục cũ. Giữ lại là để quyền của người ký
        // cũ rơi vào tay người ký mới.
        if muc.publisher != m.publisher {
            muc.publisher.clone_from(&m.publisher);
            muc.quyen.clear();
        }
        muc.quyen.insert(
            xin.name.clone(),
            MucQuyen {
                van_tay: scope_fingerprint(&xin.scope),
                cho_phep: qd == Decision::Allow,
                mo_ta: Self::mo_ta_pham_vi(&xin.scope),
            },
        );
    }

    /// Liệt kê mọi thứ đã nhớ, để hiện trên màn hình quản lý quyền.
    ///
    /// Sắp theo mã ứng dụng để thứ tự ổn định giữa các lần mở — danh sách nhảy
    /// chỗ mỗi lần mở là cách chắc chắn khiến người dùng bấm nhầm.
    #[must_use]
    pub fn list_all(&self) -> Vec<StoredEntry> {
        self.kho
            .ung_dung
            .iter()
            .map(|(id, muc)| StoredEntry {
                ma_ung_dung: id.clone(),
                key_fingerprint: key_fingerprint(&muc.publisher),
                quyen: muc
                    .quyen
                    .iter()
                    .map(|(ten, q)| AnsweredPermission {
                        ten: ten.clone(),
                        mo_ta: q.mo_ta.clone(),
                        cho_phep: q.cho_phep,
                    })
                    .collect(),
            })
            .collect()
    }

    /// Quên mọi thứ đã nhớ về một ứng dụng.
    pub fn forget(&mut self, id: &str) {
        self.kho.ung_dung.remove(id);
    }

    /// Ghi ra đĩa.
    ///
    /// Qua tệp tạm rồi đổi tên: mất điện giữa chừng thì tệp cũ còn nguyên, chứ
    /// không để lại một tệp cụt mà lần sau đọc thành kho rỗng.
    ///
    /// # Errors
    /// Lỗi ghi đĩa.
    pub fn save(&self) -> std::io::Result<()> {
        let tam = self.duong_dan.with_extension("tam");
        let b = serde_json::to_vec_pretty(&self.kho)?;
        std::fs::write(&tam, &b)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&tam, std::fs::Permissions::from_mode(0o600))?;
        }
        std::fs::rename(&tam, &self.duong_dan)
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay, không nuốt lỗi"
)]
mod kiem_thu {
    use super::*;

    fn ke_khai(id: &str, pubk: &str, hosts: &[&str]) -> Manifest {
        let h = hosts
            .iter()
            .map(|x| format!("\"{x}\""))
            .collect::<Vec<_>>()
            .join(",");
        serde_json::from_str(&format!(
            r#"{{"spec_version":"0.1","id":"{id}","name":"A","version":"1",
"publisher":"{}","scheme":"hybrid-ed25519-mldsa65-v1","content_hash":"{}",
"entry":"ui.json","capabilities":[{{"name":"network",
"scope":{{"kind":"network","hosts":[{h}]}},"reason":"x"}}]}}"#,
            pubk.repeat(1992 / pubk.len().max(1)),
            "bb".repeat(48)
        ))
        .expect("bản kê khai mẫu hỏng")
    }

    fn tam(ten: &str) -> PathBuf {
        std::env::temp_dir().join(format!("tcc-ghinho-{ten}-{}.json", std::process::id()))
    }

    #[test]
    fn tinh_trang_nguoi_ky_ba_truong_hop() {
        let p = tam("tinhtrang");
        let _ = std::fs::remove_file(&p);
        let that = ke_khai("com.tcc.vi", "aa", &["shop.tcc-coin.com"]);
        let gia = ke_khai("com.tcc.vi", "cc", &["shop.tcc-coin.com"]);

        let mut g = PermissionStore::open(&p);
        assert_eq!(g.signer_status(&that), SignerStatus::LanDau);

        g.remember(&that, &that.capabilities[0], Decision::Allow);
        assert_eq!(g.signer_status(&that), SignerStatus::KhopKhoaCu);

        // ⚠️ Trường hợp đáng giá nhất: cùng mã ứng dụng, khác khoá ký.
        match g.signer_status(&gia) {
            SignerStatus::DoiKhoa { van_tay_cu } => {
                assert!(
                    van_tay_cu.contains('…'),
                    "vân tay phải rút gọn: {van_tay_cu}"
                );
            }
            khac => panic!("đổi khoá mà báo {khac:?}"),
        }
        let _ = std::fs::remove_file(&p);
    }

    /// Vân tay lấy CẢ hai đầu. Chỉ lấy một đầu thì kẻ gian mò khoá khớp mười ký
    /// tự đầu là xong — rẻ hơn hẳn so với phải khớp cả hai đầu.
    #[test]
    fn van_tay_khoa_lay_ca_hai_dau() {
        let k = format!("{}{}", "a".repeat(10), "b".repeat(3990));
        let v = key_fingerprint(&k);
        assert!(v.starts_with("aaaaaaaaaa"), "{v}");
        assert!(v.ends_with("bbbbbbbbbb"), "{v}");
        assert!(v.len() < 30, "vân tay dài quá, không ai đọc: {v}");
    }

    #[test]
    fn nho_roi_thi_khong_hoi_lai() {
        let p = tam("nho");
        let _ = std::fs::remove_file(&p);
        let m = ke_khai("com.tcc.a", "aa", &["shop.tcc-coin.com"]);

        let mut g = PermissionStore::open(&p);
        assert_eq!(
            g.lookup(&m, &m.capabilities[0]),
            None,
            "chưa hỏi mà đã có câu trả lời"
        );
        g.remember(&m, &m.capabilities[0], Decision::Allow);
        g.save().unwrap();

        // Mở lại từ đĩa — đây mới là điều cần chứng minh.
        let g2 = PermissionStore::open(&p);
        assert_eq!(g2.lookup(&m, &m.capabilities[0]), Some(Decision::Allow));
        let _ = std::fs::remove_file(&p);
    }

    /// ⚠️ PHÉP THỬ QUAN TRỌNG NHẤT TỆP NÀY.
    ///
    /// Bản 1.0 xin một máy chủ, người dùng đồng ý. Bản 1.1 xin thêm một máy chủ
    /// nữa. Câu trả lời cũ **không được** phủ lên phạm vi mới.
    #[test]
    fn noi_rong_pham_vi_thi_phai_hoi_lai() {
        let p = tam("noirong");
        let _ = std::fs::remove_file(&p);
        let cu = ke_khai("com.tcc.a", "aa", &["shop.tcc-coin.com"]);
        let moi = ke_khai(
            "com.tcc.a",
            "aa",
            &["shop.tcc-coin.com", "thu-thap.example"],
        );

        let mut g = PermissionStore::open(&p);
        g.remember(&cu, &cu.capabilities[0], Decision::Allow);

        assert_eq!(g.lookup(&cu, &cu.capabilities[0]), Some(Decision::Allow));
        assert_eq!(
            g.lookup(&moi, &moi.capabilities[0]),
            None,
            "phạm vi nới rộng mà vẫn dùng câu trả lời cũ — người dùng chưa bao giờ \
             đồng ý với máy chủ thứ hai"
        );
        let _ = std::fs::remove_file(&p);
    }

    /// ⚠️ Khoá người ký đổi = ứng dụng khác. Không thừa hưởng gì.
    #[test]
    fn doi_khoa_nguoi_ky_thi_phai_hoi_lai() {
        let p = tam("doikhoa");
        let _ = std::fs::remove_file(&p);
        let that = ke_khai("com.tcc.vi", "aa", &["shop.tcc-coin.com"]);
        let gia = ke_khai("com.tcc.vi", "cc", &["shop.tcc-coin.com"]);

        let mut g = PermissionStore::open(&p);
        g.remember(&that, &that.capabilities[0], Decision::Allow);
        assert_eq!(
            g.lookup(&gia, &gia.capabilities[0]),
            None,
            "gói ký bằng khoá khác lại thừa hưởng quyền của gói thật"
        );
        let _ = std::fs::remove_file(&p);
    }

    /// ⚠️ Ghi đè bằng khoá mới phải XOÁ SẠCH quyền cũ, không chỉ đè lên.
    ///
    /// Bản đầu tôi viết phép thử này bằng cách cho cả hai khoá xin CÙNG một
    /// quyền — và nó vô dụng: `insert` đè lên rồi, `clear()` không có tác dụng
    /// quan sát được. Kiểm đột biến lộ ra: gỡ `clear()` mà mọi phép thử vẫn
    /// xanh.
    ///
    /// Phải cho hai khoá xin HAI quyền KHÁC nhau. Không xoá thì quyền của người
    /// ký cũ nằm lại và rơi vào tay người ký mới.
    #[test]
    fn khoa_moi_ghi_de_thi_quyen_cu_bi_xoa() {
        let p = tam("ghide");
        let _ = std::fs::remove_file(&p);

        let that = ke_khai("com.tcc.vi", "aa", &["a.tcc-coin.com"]);
        // Cùng mã ứng dụng, khoá KHÁC, và xin một quyền KHÁC hẳn.
        let khac: Manifest = serde_json::from_str(&format!(
            r#"{{"spec_version":"0.1","id":"com.tcc.vi","name":"A","version":"1",
"publisher":"{}","scheme":"hybrid-ed25519-mldsa65-v1","content_hash":"{}",
"entry":"ui.json","capabilities":[{{"name":"wallet",
"scope":{{"kind":"wallet","may_request_signature":true}},"reason":"x"}}]}}"#,
            "cc".repeat(996),
            "bb".repeat(48)
        ))
        .unwrap();

        let mut g = PermissionStore::open(&p);
        g.remember(&that, &that.capabilities[0], Decision::Allow);
        g.remember(&khac, &khac.capabilities[0], Decision::Allow);

        // Quyền mạng của người ký CŨ phải đã biến mất, không rơi sang người ký mới.
        let mang_duoi_khoa_moi = CapabilityRequest {
            name: "network".to_owned(),
            scope: that.capabilities[0].scope.clone(),
            reason: "x".to_owned(),
        };
        assert_eq!(
            g.lookup(&khac, &mang_duoi_khoa_moi),
            None,
            "quyền của người ký cũ rơi vào tay người ký mới"
        );
        // Và người ký cũ quay lại cũng không còn gì.
        assert_eq!(g.lookup(&that, &that.capabilities[0]), None);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn tu_choi_cung_duoc_nho() {
        let p = tam("tuchoi");
        let _ = std::fs::remove_file(&p);
        let m = ke_khai("com.tcc.a", "aa", &["shop.tcc-coin.com"]);
        let mut g = PermissionStore::open(&p);
        g.remember(&m, &m.capabilities[0], Decision::Deny);
        g.save().unwrap();
        assert_eq!(
            PermissionStore::open(&p).lookup(&m, &m.capabilities[0]),
            Some(Decision::Deny)
        );
        let _ = std::fs::remove_file(&p);
    }

    /// ⚠️ PHÉP THỬ CHỐT LỜI HỨA CỦA `mo_ta`.
    ///
    /// Mô tả là chữ lưu trên đĩa, tức là chữ SỬA ĐƯỢC. Nếu quyết định đọc nó thì
    /// sửa tệp là đổi được quyền. Phép thử này sửa mô tả thành một lời nói dối
    /// trắng trợn và đòi quyết định KHÔNG đổi.
    #[test]
    fn quyet_dinh_khong_doc_phan_mo_ta() {
        let p = tam("mota");
        let _ = std::fs::remove_file(&p);
        let m = ke_khai("com.tcc.a", "aa", &["shop.tcc-coin.com"]);

        let mut g = PermissionStore::open(&p);
        g.remember(&m, &m.capabilities[0], Decision::Allow);
        g.save().unwrap();

        // Sửa MÔ TẢ trên đĩa thành một phạm vi hoàn toàn khác, giữ nguyên vân tay.
        let tho = std::fs::read_to_string(&p).unwrap();
        let doi = tho.replace("shop.tcc-coin.com", "ke-gian.example");
        assert_ne!(tho, doi, "phép thử tự hỏng: không tìm thấy mô tả để sửa");
        std::fs::write(&p, &doi).unwrap();

        let g2 = PermissionStore::open(&p);
        // Quyết định vẫn y nguyên: nó chỉ đọc vân tay.
        assert_eq!(g2.lookup(&m, &m.capabilities[0]), Some(Decision::Allow));
        // Và mô tả đúng là đã bị sửa — tức phép thử thật sự chạm tới nó.
        assert_eq!(g2.list_all()[0].quyen[0].mo_ta, "ke-gian.example");
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn liet_ke_du_va_sap_on_dinh() {
        let p = tam("lietke");
        let _ = std::fs::remove_file(&p);
        let mut g = PermissionStore::open(&p);
        for id in ["com.tcc.z", "com.tcc.a", "com.tcc.m"] {
            let m = ke_khai(id, "aa", &["shop.tcc-coin.com"]);
            g.remember(&m, &m.capabilities[0], Decision::Allow);
        }
        let ds = g.list_all();
        let ma: Vec<&str> = ds.iter().map(|x| x.ma_ung_dung.as_str()).collect();
        assert_eq!(
            ma,
            ["com.tcc.a", "com.tcc.m", "com.tcc.z"],
            "danh sách nhảy chỗ giữa các lần mở là cách chắc chắn khiến người dùng bấm nhầm"
        );
        assert_eq!(ds[0].quyen[0].mo_ta, "shop.tcc-coin.com");
        assert!(ds[0].key_fingerprint.contains('…'));
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn quen_thi_hoi_lai() {
        let p = tam("forget");
        let _ = std::fs::remove_file(&p);
        let m = ke_khai("com.tcc.a", "aa", &["shop.tcc-coin.com"]);
        let mut g = PermissionStore::open(&p);
        g.remember(&m, &m.capabilities[0], Decision::Allow);
        g.forget("com.tcc.a");
        assert_eq!(g.lookup(&m, &m.capabilities[0]), None);
        let _ = std::fs::remove_file(&p);
    }

    /// ⚠️ Mọi thứ không rõ ràng đều phải ra "chưa có câu trả lời".
    #[test]
    fn tep_hong_hoac_phien_ban_la_thi_hoi_lai() {
        let m = ke_khai("com.tcc.a", "aa", &["shop.tcc-coin.com"]);
        for (ten, noi) in [
            ("khong-phai-json", "{{{ hỏng"),
            ("phien-ban-la", r#"{"phien_ban":999,"ung_dung":{}}"#),
            ("rong", ""),
        ] {
            let p = tam(ten);
            std::fs::write(&p, noi).unwrap();
            assert_eq!(
                PermissionStore::open(&p).lookup(&m, &m.capabilities[0]),
                None,
                "tệp {ten} lại cho ra một câu trả lời"
            );
            let _ = std::fs::remove_file(&p);
        }
        // Tệp không tồn tại.
        assert_eq!(
            PermissionStore::open(Path::new("/khong/co/that.json")).lookup(&m, &m.capabilities[0]),
            None
        );
    }

    // ---- Vân tay phạm vi ----

    #[test]
    fn cung_pham_vi_khac_thu_tu_van_cung_van_tay() {
        let a = Scope::Network {
            hosts: vec!["b.tcc-coin.com".into(), "a.tcc-coin.com".into()],
        };
        let b = Scope::Network {
            hosts: vec!["a.tcc-coin.com".into(), "B.TCC-COIN.COM".into()],
        };
        assert_eq!(scope_fingerprint(&a), scope_fingerprint(&b));
    }

    /// Không có tiền tố độ dài thì `["ab","c"]` và `["a","bc"]` cho cùng chuỗi
    /// byte — hai phạm vi khác nhau mà cùng vân tay là cấp nhầm quyền.
    #[test]
    fn pham_vi_khac_nhau_thi_van_tay_khac_nhau() {
        let a = Scope::Network {
            hosts: vec!["ab".into(), "c".into()],
        };
        let b = Scope::Network {
            hosts: vec!["a".into(), "bc".into()],
        };
        assert_ne!(scope_fingerprint(&a), scope_fingerprint(&b));
    }

    #[test]
    fn moi_loai_pham_vi_cho_van_tay_khac_nhau() {
        let ds = [
            scope_fingerprint(&Scope::Network { hosts: vec![] }),
            scope_fingerprint(&Scope::Storage { quota_bytes: 0 }),
            scope_fingerprint(&Scope::Wallet {
                may_request_signature: false,
            }),
            scope_fingerprint(&Scope::Wallet {
                may_request_signature: true,
            }),
        ];
        let mut u = ds.to_vec();
        u.sort();
        u.dedup();
        assert_eq!(u.len(), ds.len(), "hai phạm vi khác nhau cho cùng vân tay");
    }
}
