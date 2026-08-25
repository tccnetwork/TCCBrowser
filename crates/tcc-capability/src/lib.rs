//! Mô hình quyền năng — trái tim kiến trúc bảo mật.
//!
//! # Nguyên tắc: quyền năng KHÔNG TỒN TẠI cho tới khi được cấp
//!
//! Cách thường thấy ở trình duyệt cũ:
//!
//! ```text
//! API luôn có sẵn  +  cờ permission = false  →  kiểm tra lúc gọi
//! ```
//!
//! Cách của TCC:
//!
//! ```text
//! Không có giá trị nào để gọi  →  cho tới khi người dùng cấp
//! ```
//!
//! Khác biệt không phải chữ nghĩa. Ở cách cũ, **quên một lần kiểm là thủng** —
//! và mã có hàng trăm chỗ gọi. Ở cách này, muốn gọi mạng thì phải **cầm được một
//! `NetworkCapability`**, mà kiểu đó có trường riêng tư nên **không dựng được từ
//! ngoài crate này**. Quên kiểm là chuyện không xảy ra được, vì không có gì để
//! quên: không có quyền thì không có giá trị, không có giá trị thì không biên
//! dịch nổi.
//!
//! Trình biên dịch làm việc canh gác, không phải người soi mã.
//!
//! # Bốn tính chất tiêu chuẩn đòi
//!
//! | Tính chất | Cách đạt |
//! |---|---|
//! | Tường minh | Phải cầm được giá trị mới gọi được |
//! | Có phạm vi | Phạm vi nằm trong chính giá trị, kiểm ở mỗi lần dùng |
//! | Thu hồi được | Cờ chung; thu hồi là mọi bản sao chết theo tức thì |
//! | Ghi vết được | Mỗi lần dùng tăng bộ đếm, đọc ra được |

use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

use tcc_spec::{AppId, CapabilityRequest, Scope};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CapabilityError {
    #[error("quyền năng đã bị thu hồi")]
    Revoked,

    #[error("ngoài phạm vi: {0}")]
    OutOfScope(String),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GrantError {
    /// Cùng một quyền được khai hai lần.
    ///
    /// `tcc-spec` đã chặn ở tầng định dạng, nhưng `grant` là hàm CÔNG KHAI và
    /// người gọi có thể quên gọi `validate_shape` trước. Chặn ở cả hai tầng:
    /// một tầng quên thì tầng kia vẫn giữ.
    #[error("quyền \"{0}\" được khai nhiều lần — mục sau sẽ lặng lẽ đè mục trước")]
    Duplicate(String),
}

/// Trạng thái sống/chết dùng chung cho mọi quyền năng của MỘT ứng dụng.
///
/// Thu hồi phải giết được cả những bản sao ứng dụng đang giữ trong tay — nên
/// trạng thái nằm ở đây, không nằm trong từng bản sao.
#[derive(Debug)]
struct Life {
    alive: AtomicBool,
    uses: AtomicU64,
}

impl Life {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            alive: AtomicBool::new(true),
            uses: AtomicU64::new(0),
        })
    }

    fn touch(&self) -> Result<(), CapabilityError> {
        if self.alive.load(Ordering::SeqCst) {
            self.uses.fetch_add(1, Ordering::Relaxed);
            Ok(())
        } else {
            Err(CapabilityError::Revoked)
        }
    }
}

/// Quyền gọi mạng, giới hạn ở đúng những tên máy chủ đã được duyệt.
///
/// Không dựng được từ ngoài crate — trường riêng tư:
///
/// ```compile_fail
/// use tcc_capability::NetworkCapability;
/// // Không có cách nào dựng: mọi trường đều riêng tư.
/// let _gia_mao = NetworkCapability { hosts: vec![], life: todo!() };
/// ```
#[derive(Debug, Clone)]
pub struct NetworkCapability {
    hosts: Arc<[String]>,
    life: Arc<Life>,
}

impl NetworkCapability {
    /// Xin phép gọi tới một máy chủ.
    ///
    /// # KHỚP TÊN MIỀN CHÍNH XÁC — không khớp tên miền con
    ///
    /// Được cấp `shop.tcc-coin.com` thì KHÔNG gọi được `evil.shop.tcc-coin.com`,
    /// và cũng không gọi được `shop.tcc-coin.com.gia-mao.net`.
    ///
    /// Đây đúng bài học đã trả giá ở kho mật khẩu của TCCBrowser v1: khớp mờ theo
    /// hậu tố là đưa quyền cho kẻ gian, vì tên miền của chúng chỉ cần *trông
    /// giống*.
    ///
    /// # Errors
    /// Quyền đã bị thu hồi, hoặc máy chủ không nằm trong danh sách đã duyệt.
    pub fn allow(&self, host: &str) -> Result<(), CapabilityError> {
        self.life.touch()?;
        // Chuẩn hoá dấu chấm cuối: "shop.tcc-coin.com." và "shop.tcc-coin.com"
        // là CÙNG một máy chủ theo chuẩn DNS. Không chuẩn hoá thì kết quả vẫn an
        // toàn (từ chối), nhưng từ chối sai làm người sau tưởng có lỗi ở chỗ khác.
        let host = chuan_hoa(host);
        if self.hosts.iter().any(|h| h == &host) {
            Ok(())
        } else {
            Err(CapabilityError::OutOfScope(format!(
                "{host} không nằm trong danh sách đã duyệt"
            )))
        }
    }

    #[must_use]
    pub fn hosts(&self) -> &[String] {
        &self.hosts
    }
}

/// Quyền lưu trữ, giới hạn bằng hạn mức byte.
#[derive(Debug, Clone)]
pub struct StorageCapability {
    quota_bytes: u64,
    life: Arc<Life>,
}

impl StorageCapability {
    /// # Errors
    /// Quyền đã bị thu hồi, hoặc vượt hạn mức.
    pub fn allow_write(&self, bytes: u64) -> Result<(), CapabilityError> {
        self.life.touch()?;
        if bytes <= self.quota_bytes {
            Ok(())
        } else {
            Err(CapabilityError::OutOfScope(format!(
                "ghi {bytes} byte, hạn mức {}",
                self.quota_bytes
            )))
        }
    }
}

/// Quyền chạm tới ví.
///
/// Tách "đọc địa chỉ" khỏi "xin chữ ký" là CỐ Ý: phần lớn ứng dụng chỉ cần biết
/// địa chỉ để hiển thị, và không có lý do gì cho chúng quyền xin chữ ký.
#[derive(Debug, Clone)]
pub struct WalletCapability {
    may_request_signature: bool,
    life: Arc<Life>,
}

impl WalletCapability {
    /// # Errors
    /// Quyền đã bị thu hồi.
    pub fn allow_read_address(&self) -> Result<(), CapabilityError> {
        self.life.touch()
    }

    /// Xin CHỮ KÝ. Vẫn phải có màn xác nhận riêng của người dùng cho từng giao
    /// dịch — quyền năng này chỉ cho phép *hỏi*, không cho phép *ký*.
    ///
    /// # Errors
    /// Quyền đã bị thu hồi, hoặc ứng dụng không được cấp quyền xin chữ ký.
    pub fn allow_request_signature(&self) -> Result<(), CapabilityError> {
        self.life.touch()?;
        if self.may_request_signature {
            Ok(())
        } else {
            Err(CapabilityError::OutOfScope(
                "ứng dụng chỉ được cấp quyền đọc địa chỉ, không được xin chữ ký".to_string(),
            ))
        }
    }
}

/// Tập quyền năng trao cho một ứng dụng đang chạy.
///
/// Mặc định RỖNG. Không có đường nào dựng ra tập "đầy đủ".
#[derive(Debug, Clone)]
pub struct CapabilitySet {
    app: AppId,
    network: Option<NetworkCapability>,
    storage: Option<StorageCapability>,
    wallet: Option<WalletCapability>,
    life: Arc<Life>,
}

impl CapabilitySet {
    #[must_use]
    pub fn app(&self) -> &AppId {
        &self.app
    }

    /// Trả `None` khi ứng dụng không được cấp — KHÔNG trả về một quyền đã tắt.
    /// Khác biệt này chính là điều khiến "quên kiểm" trở thành không thể.
    #[must_use]
    pub fn network(&self) -> Option<&NetworkCapability> {
        self.network.as_ref()
    }

    #[must_use]
    pub fn storage(&self) -> Option<&StorageCapability> {
        self.storage.as_ref()
    }

    #[must_use]
    pub fn wallet(&self) -> Option<&WalletCapability> {
        self.wallet.as_ref()
    }

    /// Thu hồi TẤT CẢ, tức thì. Mọi bản sao ứng dụng đang giữ cũng chết theo, vì
    /// trạng thái sống nằm chung chứ không nằm trong từng bản sao.
    pub fn revoke_all(&self) {
        self.life.alive.store(false, Ordering::SeqCst);
    }

    /// Số lần quyền năng được dùng — dành cho nhật ký và màn hình kiểm tra.
    #[must_use]
    pub fn use_count(&self) -> u64 {
        self.life.uses.load(Ordering::Relaxed)
    }
}

/// Chuẩn hoá tên máy chủ trước khi so khớp: chữ thường, bỏ dấu chấm cuối.
///
/// KHÔNG làm gì hơn thế. Cụ thể là KHÔNG cắt tên miền con, KHÔNG khớp hậu tố —
/// đó chính là bài học đã trả giá ở kho mật khẩu v1.
fn chuan_hoa(host: &str) -> String {
    host.trim_end_matches('.').to_ascii_lowercase()
}

/// Quyết định của người dùng cho MỘT mục xin quyền.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Deny,
}

/// Dựng tập quyền năng từ những gì bản kê khai xin và người dùng duyệt.
///
/// Đây là **lối vào duy nhất** để một quyền năng ra đời. Không có hàm dựng công
/// khai nào khác trong crate này.
///
/// Luật: quyền nào ứng dụng KHÔNG xin trong bản kê khai thì không bao giờ được
/// cấp, kể cả khi hàm gọi truyền vào `Allow`. Bản kê khai là trần, ý người dùng
/// chỉ hạ xuống được chứ không nâng lên.
///
/// # Errors
/// Cùng một quyền được khai nhiều hơn một lần.
pub fn grant(
    app: AppId,
    requested: &[CapabilityRequest],
    mut decide: impl FnMut(&CapabilityRequest) -> Decision,
) -> Result<CapabilitySet, GrantError> {
    // Chặn khai trùng TRƯỚC khi hỏi người dùng: hỏi rồi mới báo lỗi là làm phiền
    // vô ích, và tệ hơn là người dùng đã bấm duyệt cho một mục sắp bị bỏ đi.
    let mut da_thay: Vec<&str> = Vec::new();
    for r in requested {
        if da_thay.contains(&r.name.as_str()) {
            return Err(GrantError::Duplicate(r.name.clone()));
        }
        da_thay.push(&r.name);
    }

    let life = Life::new();
    let mut set = CapabilitySet {
        app,
        network: None,
        storage: None,
        wallet: None,
        life: Arc::clone(&life),
    };

    for req in requested {
        if decide(req) != Decision::Allow {
            continue;
        }
        match &req.scope {
            Scope::Network { hosts } => {
                let lower: Vec<String> = hosts.iter().map(|h| chuan_hoa(h)).collect();
                set.network = Some(NetworkCapability {
                    hosts: lower.into(),
                    life: Arc::clone(&life),
                });
            }
            Scope::Storage { quota_bytes } => {
                set.storage = Some(StorageCapability {
                    quota_bytes: *quota_bytes,
                    life: Arc::clone(&life),
                });
            }
            Scope::Wallet {
                may_request_signature,
            } => {
                set.wallet = Some(WalletCapability {
                    may_request_signature: *may_request_signature,
                    life: Arc::clone(&life),
                });
            }
        }
    }
    Ok(set)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "kiểm thử: hỏng thì phải nổ ngay")]
mod kiem_thu {
    use super::*;

    fn app() -> AppId {
        AppId::parse("com.tcc.hello").unwrap()
    }

    fn xin_mang(hosts: &[&str]) -> CapabilityRequest {
        CapabilityRequest {
            name: "network".to_string(),
            scope: Scope::Network {
                hosts: hosts.iter().map(|s| (*s).to_string()).collect(),
            },
            reason: "tải danh sách sản phẩm".to_string(),
        }
    }

    fn cho_het(_: &CapabilityRequest) -> Decision {
        Decision::Allow
    }

    #[test]
    fn khong_xin_gi_thi_khong_co_gi() {
        let s = grant(app(), &[], cho_het).unwrap();
        assert!(s.network().is_none());
        assert!(s.storage().is_none());
        assert!(s.wallet().is_none());
    }

    /// ⚠️ Người dùng từ chối → `None`, KHÔNG phải một quyền đã tắt.
    /// Đây là khác biệt cốt lõi so với mô hình cờ permission.
    #[test]
    fn nguoi_dung_tu_choi_thi_khong_co_gia_tri_nao() {
        let s = grant(app(), &[xin_mang(&["shop.tcc-coin.com"])], |_| {
            Decision::Deny
        })
        .unwrap();
        assert!(
            s.network().is_none(),
            "từ chối phải cho ra None chứ không phải quyền tắt"
        );
    }

    #[test]
    fn duoc_cap_thi_goi_dung_may_chu_da_duyet() {
        let s = grant(app(), &[xin_mang(&["shop.tcc-coin.com"])], cho_het).unwrap();
        let n = s.network().unwrap();
        assert!(n.allow("shop.tcc-coin.com").is_ok());
        assert!(
            n.allow("SHOP.TCC-COIN.COM").is_ok(),
            "tên miền không phân biệt hoa thường"
        );
    }

    /// Bài học đã trả giá ở kho mật khẩu v1: KHỚP CHÍNH XÁC, không khớp hậu tố.
    #[test]
    fn khong_khop_ten_mien_con_va_khong_khop_hau_to() {
        let s = grant(app(), &[xin_mang(&["shop.tcc-coin.com"])], cho_het).unwrap();
        let n = s.network().unwrap();
        for gia_mao in [
            "evil.shop.tcc-coin.com",        // tên miền con
            "shop.tcc-coin.com.gia-mao.net", // hậu tố giả
            "tcc-coin.com",                  // tên miền cha
            "shop.tcc-coin.co",              // gần giống
        ] {
            assert!(
                n.allow(gia_mao).is_err(),
                "phải chặn {gia_mao} — chỉ khớp chính xác"
            );
        }
    }

    /// ⚠️ Thu hồi phải giết cả bản sao ứng dụng đang cầm trong tay.
    #[test]
    fn thu_hoi_giet_ca_ban_sao_dang_cam() {
        let s = grant(app(), &[xin_mang(&["shop.tcc-coin.com"])], cho_het).unwrap();
        let ban_sao = s.network().unwrap().clone();
        assert!(ban_sao.allow("shop.tcc-coin.com").is_ok());

        s.revoke_all();

        assert_eq!(
            ban_sao.allow("shop.tcc-coin.com"),
            Err(CapabilityError::Revoked),
            "bản sao cũ vẫn dùng được sau khi thu hồi — thu hồi không có tác dụng"
        );
    }

    /// **Thu hồi phải giết MỌI lối vào, của MỌI quyền năng.**
    ///
    /// Phép thử B4 gốc chỉ đi qua quyền MẠNG. Ngày 25/08/2026 hai lỗ lộ ra,
    /// và chúng lộ ra theo hai cách khác nhau — đáng ghi cả hai:
    ///
    /// * `cargo-mutants` thay thân `allow_read_address` bằng `Ok(())` và không
    ///   phép thử nào đỏ. Hàm ấy chỉ làm ĐÚNG MỘT việc là hỏi quyền còn sống
    ///   không, nên "thay cả thân vẫn xanh" nghĩa là việc duy nhất ấy chưa từng
    ///   được kiểm.
    /// * `allow_write` thì công cụ KHÔNG tìm ra, vì thân nó còn phép kiểm hạn
    ///   mức nên `-> Ok(())` bị bắt. Phải tự tay gỡ đúng dòng `life.touch()?`
    ///   mới thấy: mọi phép thử vẫn xanh. Danh mục đột biến của công cụ là hữu
    ///   hạn, và im lặng của nó không phải bằng chứng.
    ///
    /// Nên phép thử này đi qua TẤT CẢ, không phải một đường mẫu: thêm một
    /// quyền năng mới mà quên thu hồi thì chỗ này là nơi phải sửa.
    #[test]
    fn thu_hoi_giet_moi_loi_vao_cua_moi_quyen() {
        let xin_vi = CapabilityRequest {
            name: "wallet".to_string(),
            scope: Scope::Wallet {
                may_request_signature: true,
            },
            reason: "ký giao dịch".to_string(),
        };
        let xin_kho = CapabilityRequest {
            name: "storage".to_string(),
            scope: Scope::Storage { quota_bytes: 1000 },
            reason: "lưu nháp".to_string(),
        };
        let s = grant(
            app(),
            &[xin_mang(&["shop.tcc-coin.com"]), xin_vi, xin_kho],
            cho_het,
        )
        .unwrap();

        // Bản sao CẦM TRÊN TAY từ trước lúc thu hồi — đó mới là điều B4 nói.
        let mang = s.network().unwrap().clone();
        let vi = s.wallet().unwrap().clone();
        let kho = s.storage().unwrap().clone();

        assert!(mang.allow("shop.tcc-coin.com").is_ok());
        assert!(vi.allow_read_address().is_ok());
        assert!(vi.allow_request_signature().is_ok());
        assert!(kho.allow_write(10).is_ok());

        s.revoke_all();

        let sau: [(&str, Result<(), CapabilityError>); 4] = [
            ("mạng", mang.allow("shop.tcc-coin.com")),
            ("ví: đọc địa chỉ", vi.allow_read_address()),
            ("ví: xin chữ ký", vi.allow_request_signature()),
            ("kho: ghi", kho.allow_write(10)),
        ];
        for (ten, ket) in sau {
            assert_eq!(
                ket,
                Err(CapabilityError::Revoked),
                "{ten} vẫn dùng được sau khi thu hồi"
            );
        }
    }

    #[test]
    fn vi_chi_doc_thi_khong_xin_duoc_chu_ky() {
        let xin = CapabilityRequest {
            name: "wallet".to_string(),
            scope: Scope::Wallet {
                may_request_signature: false,
            },
            reason: "hiện số dư".to_string(),
        };
        let s = grant(app(), &[xin], cho_het).unwrap();
        let w = s.wallet().unwrap();
        assert!(w.allow_read_address().is_ok());
        assert!(
            w.allow_request_signature().is_err(),
            "quyền chỉ-đọc mà xin được chữ ký là thủng"
        );
    }

    #[test]
    fn luu_tru_qua_han_muc_thi_chan() {
        let xin = CapabilityRequest {
            name: "storage".to_string(),
            scope: Scope::Storage { quota_bytes: 1000 },
            reason: "lưu bản nháp".to_string(),
        };
        let s = grant(app(), &[xin], cho_het).unwrap();
        let st = s.storage().unwrap();
        assert!(st.allow_write(1000).is_ok());
        assert!(st.allow_write(1001).is_err());
    }

    /// Cấp từng phần: xin ba quyền, người dùng chỉ duyệt một.
    #[test]
    fn duyet_tung_phan() {
        let xin = vec![
            xin_mang(&["shop.tcc-coin.com"]),
            CapabilityRequest {
                name: "wallet".to_string(),
                scope: Scope::Wallet {
                    may_request_signature: true,
                },
                reason: "thanh toán".to_string(),
            },
        ];
        let s = grant(app(), &xin, |r| {
            if r.name == "network" {
                Decision::Allow
            } else {
                Decision::Deny
            }
        })
        .unwrap();
        assert!(s.network().is_some());
        assert!(s.wallet().is_none(), "ví bị từ chối mà vẫn có");
    }

    /// ⚠️ LỖ 6. `tcc-spec` đã chặn khai trùng, nhưng `grant` là hàm CÔNG KHAI —
    /// người gọi có thể quên `validate_shape`. Không chặn ở đây thì mục thứ hai
    /// lặng lẽ đè mục thứ nhất, và cái được cấp là cái người duyệt không đọc.
    #[test]
    fn grant_tu_choi_khai_trung_du_ben_goi_quen_kiem() {
        let xin = vec![
            xin_mang(&["lanh.tcc-coin.com"]),
            xin_mang(&["xau.example.com"]),
        ];
        assert_eq!(
            grant(app(), &xin, cho_het).unwrap_err(),
            GrantError::Duplicate("network".to_string()),
            "grant phải tự chặn, không dựa vào bên gọi đã kiểm hay chưa"
        );
    }

    /// Dấu chấm cuối là cùng một máy chủ theo chuẩn DNS. Không chuẩn hoá thì kết
    /// quả vẫn AN TOÀN (từ chối), nhưng từ chối sai làm người sau đi tìm lỗi nhầm chỗ.
    #[test]
    fn dau_cham_cuoi_ten_mien_van_khop() {
        let s = grant(app(), &[xin_mang(&["shop.tcc-coin.com"])], cho_het).unwrap();
        let n = s.network().unwrap();
        assert!(n.allow("shop.tcc-coin.com.").is_ok());
    }

    /// Chuẩn hoá KHÔNG được nới lỏng việc khớp: vẫn phải chặn tên miền con.
    #[test]
    fn chuan_hoa_khong_lam_hong_luat_khop_chinh_xac() {
        let s = grant(app(), &[xin_mang(&["shop.tcc-coin.com"])], cho_het).unwrap();
        let n = s.network().unwrap();
        for gia_mao in [
            "evil.shop.tcc-coin.com.",
            "shop.tcc-coin.com.gia-mao.net.",
            ".shop.tcc-coin.com",
        ] {
            assert!(n.allow(gia_mao).is_err(), "phải chặn {gia_mao}");
        }
    }

    #[test]
    fn dem_duoc_so_lan_dung_de_ghi_vet() {
        let s = grant(app(), &[xin_mang(&["a.tcc-coin.com"])], cho_het).unwrap();
        let n = s.network().unwrap();
        assert_eq!(s.use_count(), 0);
        let _ = n.allow("a.tcc-coin.com");
        let _ = n.allow("b.tcc-coin.com"); // ngoài phạm vi nhưng vẫn tính là một lần dùng
        assert_eq!(s.use_count(), 2);
    }

    /// **Danh sách máy chủ đã duyệt phải đọc ra ĐÚNG cái đã cấp.**
    ///
    /// `hosts()` là `pub` trên một crate lá và **không một chỗ nào trong kho
    /// gọi nó** — cùng hạng với `FileTree::paths` mà `cargo-mutants` chỉ ra
    /// ngày 25/08/2026: không ai gọi thì không đột biến nào giết được, và API
    /// công khai không ai kiểm là một lời hứa suông. Không xoá, vì người viết
    /// bản cài đặt thứ hai cần đọc được danh sách đã cấp để hiện lên màn hình.
    #[test]
    fn danh_sach_may_chu_doc_ra_dung_cai_da_cap() {
        // Phải đi qua `grant`: `NetworkCapability` CỐ Ý không dựng được từ
        // ngoài — đó chính là bất biến "quyền năng không giả mạo được", và
        // phép thử này không được là chỗ đầu tiên phá nó.
        let s = grant(
            app(),
            &[xin_mang(&["shop.tcc-coin.com", "rpc2.tcc-coin.com"])],
            cho_het,
        )
        .unwrap();
        let q = s.network().unwrap();
        assert_eq!(q.hosts(), ["shop.tcc-coin.com", "rpc2.tcc-coin.com"]);

        // Không xin gì thì không có quyền để mà đọc.
        let rong = grant(app(), &[], cho_het).unwrap();
        assert!(rong.network().is_none(), "chưa xin mà có quyền mạng");
    }
}
