//! Nạp và chạy một ứng dụng TCC.
//!
//! VIỆC CỦA CRATE NÀY: ghép lát cắt lại — kiểm chữ ký, hỏi người dùng, dựng tập
//! quyền năng, rồi trao cho ứng dụng đúng những gì nó được cấp.
//!
//! LUẬT: crate này KHÔNG biết bộ dựng nào đang chạy. Nó không nhắc tới WebView,
//! không nhắc tới GPU. Có CI kiểm (`tools/kiem-luat-phu-thuoc.sh`, luật 5).
//!
//! # Thứ tự các bước là một tính chất BẢO MẬT
//!
//! ```text
//! 1. Kiểm chữ ký          ← chưa qua bước này thì KHÔNG tin gì trong bản kê khai
//! 2. Kiểm điểm vào tồn tại
//! 3. Hỏi người dùng
//! 4. Dựng quyền năng
//! ```
//!
//! Hỏi người dùng TRƯỚC khi kiểm chữ ký là sai nghiêm trọng: hộp thoại sẽ hiện
//! tên và lý do lấy từ một bản kê khai **chưa được xác thực** — tức kẻ gian viết
//! gì thì người dùng đọc nấy. Phép thử `khong_hoi_nguoi_dung_khi_chu_ky_hong`
//! chốt lại điều này.

pub mod goi;

use std::path::Path;

use tcc_capability::{CapabilitySet, Decision, GrantError, grant};
use tcc_crypto::SignatureScheme;
use tcc_manifest::{ManifestError, VerifiedApp, verify_package};
use tcc_spec::{CapabilityRequest, Effect, FileTree, SpecError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("gói không hợp lệ: {0}")]
    Package(#[from] ManifestError),

    #[error("bản kê khai không khớp nội dung: {0}")]
    Spec(#[from] SpecError),

    #[error("cấp quyền thất bại: {0}")]
    Grant(#[from] GrantError),

    #[error("không đọc được gói: {0}")]
    Goi(String),
}

/// Đường ra ngoài, tiêm từ bên ngoài vào.
///
/// `tcc-runtime` KHÔNG tự mở socket. Hai lý do: kiểm thử được mà không đụng mạng
/// thật, và mọi đường ra khỏi máy đều nhìn thấy được ngay tại chỗ gọi — không có
/// lối đi ngầm nào chôn trong thư viện.
pub trait Mang {
    /// # Errors
    /// Tuỳ bản cài đặt.
    fn get(&self, host: &str, path: &str) -> Result<Vec<u8>, String>;
}

#[derive(Debug, Error)]
pub enum ActionError {
    #[error("bản kê khai không khai hành động \"{0}\"")]
    KhongCo(String),

    #[error("chưa được cấp quyền \"{0}\" — người dùng không bật công tắc này")]
    ChuaCapQuyen(String),

    #[error("quyền năng từ chối: {0}")]
    Quyen(#[from] tcc_capability::CapabilityError),

    #[error("mạng lỗi: {0}")]
    Mang(String),
}

/// Một ứng dụng đã nạp xong: chữ ký đã kiểm, quyền năng đã cấp.
///
/// Chỉ dựng được qua `load`, nên cầm được nó nghĩa là mọi bước đã qua.
#[derive(Debug)]
pub struct LoadedApp {
    app: VerifiedApp,
    caps: CapabilitySet,
    content: FileTree,
}

impl LoadedApp {
    #[must_use]
    pub fn manifest(&self) -> &tcc_spec::Manifest {
        self.app.manifest()
    }

    /// Quyền năng ứng dụng THẬT SỰ có. Có thể ít hơn nhiều so với những gì nó xin.
    #[must_use]
    pub fn capabilities(&self) -> &CapabilitySet {
        &self.caps
    }

    /// Nội dung tệp điểm vào.
    ///
    /// Không trả `Option`: `load` đã kiểm điểm vào tồn tại, nên tới được đây là
    /// chắc chắn có. Kiểu dữ liệu mang theo bằng chứng đó.
    #[must_use]
    pub fn entry_content(&self) -> &[u8] {
        self.content
            .get(&self.app.manifest().entry)
            .unwrap_or_else(|| unreachable!("load đã kiểm điểm vào tồn tại"))
    }

    /// Bản sao cây tệp đã ký, để trao cho trình phục vụ của bộ dựng.
    ///
    /// Bản SAO chứ không phải tham chiếu: trình phục vụ sống trong vòng lặp sự
    /// kiện của cửa sổ, lâu hơn lời gọi này. Cây tệp đã qua kiểm chữ ký nên bản
    /// sao cũng vậy — không có đường nào nhét nội dung chưa ký vào đây.
    #[must_use]
    pub fn ban_sao_noi_dung(&self) -> FileTree {
        self.content.clone()
    }

    /// Đọc một tệp bất kỳ trong gói. Ứng dụng chỉ thấy được nội dung ĐÃ KÝ —
    /// không có đường nào ra hệ thống tệp thật.
    #[must_use]
    pub fn read(&self, path: &str) -> Option<&[u8]> {
        self.content.get(path)
    }

    /// Chạy một hành động người dùng vừa bấm.
    ///
    /// # Thứ tự là một tính chất BẢO MẬT
    ///
    /// ```text
    /// 1. Tra hành động trong bản kê khai ĐÃ KÝ   ← không có thì không chạy gì
    /// 2. Hỏi QUYỀN NĂNG                          ← chưa cấp thì dừng ở đây
    /// 3. Mới gọi ra ngoài
    /// ```
    ///
    /// Bước 2 phải đứng trước bước 3. Gọi trước rồi kiểm sau nghĩa là gói tin
    /// đã rời khỏi máy — mà với một máy chủ theo dõi thì chỉ cần gói tin đến
    /// nơi là đủ, nội dung trả về không quan trọng.
    ///
    /// `mang` được TIÊM VÀO chứ không nằm trong crate này: `tcc-runtime` không
    /// mở socket, nên nó kiểm thử được mà không đụng mạng thật, và đường ra
    /// ngoài luôn nhìn thấy được ở chỗ gọi.
    ///
    /// # Errors
    /// Không có hành động đó, chưa được cấp quyền, máy chủ ngoài phạm vi, quyền
    /// đã bị thu hồi, hoặc mạng lỗi.
    pub fn thuc_hien(&self, id: &str, mang: &dyn Mang) -> Result<Vec<u8>, ActionError> {
        // 1. Hành động phải có trong bản kê khai ĐÃ KÝ. Mã đến từ cú bấm trên
        //    màn hình; không tra ở đây thì trang tự bịa ra hành động được.
        let a = self
            .app
            .manifest()
            .actions
            .iter()
            .find(|a| a.id.as_str() == id)
            .ok_or_else(|| ActionError::KhongCo(id.to_owned()))?;

        match &a.effect {
            Effect::Fetch { host, path } => {
                // 2. Quyền năng TRƯỚC. `network()` trả `None` khi người dùng
                //    không bật công tắc cho quyền này.
                let n = self
                    .caps
                    .network()
                    .ok_or_else(|| ActionError::ChuaCapQuyen("network".to_owned()))?;
                n.allow(host)?;

                // 3. Giờ mới ra ngoài.
                mang.get(host, path).map_err(ActionError::Mang)
            }
        }
    }

    /// Thu hồi mọi quyền năng, tức thì. Dùng khi người dùng đóng ứng dụng hoặc
    /// bấm "ngắt quyền" trong trình duyệt.
    pub fn revoke_all(&self) {
        self.caps.revoke_all();
    }
}

/// Nạp một gói ứng dụng.
///
/// `decide` được gọi MỘT LẦN cho mỗi quyền ứng dụng xin, và **chỉ sau khi chữ ký
/// đã hợp lệ**. Trong trình duyệt thật nó sẽ dựng hộp thoại; trong kiểm thử nó là
/// một hàm đơn giản.
///
/// # Errors
/// Chữ ký hỏng, điểm vào không tồn tại, hoặc bản kê khai xin trùng quyền.
pub fn load(
    manifest_bytes: &[u8],
    signature: &[u8],
    content: FileTree,
    scheme: &dyn SignatureScheme,
    decide: impl FnMut(&CapabilityRequest) -> Decision,
) -> Result<LoadedApp, RuntimeError> {
    let app = verify(manifest_bytes, signature, &content, scheme)?;
    grant_verified(app, content, decide)
}

/// BƯỚC 1: kiểm chữ ký và điểm vào. **Chưa hỏi người dùng.**
///
/// Tách riêng vì giao diện cần bản kê khai để VẼ hộp thoại hỏi quyền, mà bản kê
/// khai chỉ đáng tin sau bước này. Gộp một bước thì `decide` chỉ nhận được từng
/// mục quyền lẻ và không có cách nào dựng nổi một hộp thoại nói rõ ứng dụng tên
/// gì — mà tên ứng dụng chính là thứ người dùng dựa vào để quyết định.
///
/// # Errors
/// Chữ ký hỏng, hoặc điểm vào không có trong nội dung.
pub fn verify(
    manifest_bytes: &[u8],
    signature: &[u8],
    content: &FileTree,
    scheme: &dyn SignatureScheme,
) -> Result<VerifiedApp, RuntimeError> {
    // 1. Chữ ký TRƯỚC — chưa qua thì không có gì trong bản kê khai đáng tin
    let app = verify_package(manifest_bytes, signature, content, scheme)?;
    // 2. Điểm vào phải tồn tại, kiểm trước khi làm phiền người dùng
    app.manifest().validate_against_content(content)?;
    Ok(app)
}

/// BƯỚC 2: hỏi người dùng, rồi cấp quyền.
///
/// Chỉ nhận `VerifiedApp`. Đó KHÔNG phải chuyện tiện tay: `VerifiedApp` chỉ dựng
/// được từ `verify_package`, nên kiểu dữ liệu tự nó chặn đường gọi bước 2 mà bỏ
/// qua bước 1. Không cần ai nhớ luật, trình biên dịch nhớ hộ.
///
/// # Errors
/// Bản kê khai xin trùng quyền.
pub fn grant_verified(
    app: VerifiedApp,
    content: FileTree,
    decide: impl FnMut(&CapabilityRequest) -> Decision,
) -> Result<LoadedApp, RuntimeError> {
    let caps = grant(
        app.manifest().id.clone(),
        &app.manifest().capabilities,
        decide,
    )?;
    Ok(LoadedApp { app, caps, content })
}

/// Như [`verify`] nhưng đọc từ thư mục trên đĩa.
///
/// Trả kèm `FileTree` vì bước 2 cần nó, và đọc lại đĩa lần nữa là vừa chậm vừa
/// mở ra khe hở: tệp có thể đổi giữa hai lần đọc.
///
/// # Errors
/// Thiếu tệp, liên kết mềm, gói quá lớn, hoặc chữ ký hỏng.
pub fn verify_from_dir(
    duong_dan: &Path,
    scheme: &dyn SignatureScheme,
) -> Result<(VerifiedApp, FileTree), RuntimeError> {
    let loi = |e: goi::LoiGoi| RuntimeError::Goi(e.to_string());
    let ke_khai = goi::doc_ke_khai(duong_dan).map_err(loi)?;
    let chu_ky = goi::doc_chu_ky(duong_dan).map_err(loi)?;
    let noi_dung = goi::doc_noi_dung(duong_dan).map_err(loi)?;
    let app = verify(&ke_khai, &chu_ky, &noi_dung, scheme)?;
    Ok((app, noi_dung))
}

/// Nạp một gói TCC **từ thư mục trên đĩa**.
///
/// Thứ tự vẫn y như [`load`] — đọc đĩa xong là quay về đúng đường ống đó, không
/// có lối tắt nào. Cụ thể: **đọc xong KHÔNG có nghĩa là tin**. Ba tệp đọc lên
/// mới chỉ là byte; chữ ký kiểm ở `load`, và chỉ sau đó bản kê khai mới đáng tin
/// để đem hỏi người dùng.
///
/// # Errors
/// Thiếu tệp, có liên kết mềm, gói quá lớn, chữ ký hỏng, thiếu điểm vào, hoặc
/// bản kê khai xin trùng quyền.
pub fn load_from_dir(
    duong_dan: &Path,
    scheme: &dyn SignatureScheme,
    decide: impl FnMut(&CapabilityRequest) -> Decision,
) -> Result<LoadedApp, RuntimeError> {
    let (app, noi_dung) = verify_from_dir(duong_dan, scheme)?;
    grant_verified(app, noi_dung, decide)
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay, không nuốt lỗi"
)]
mod kiem_thu {
    use super::*;
    use std::cell::Cell;
    use tcc_crypto::{HybridEd25519MlDsa, content_hash_hex};

    struct Goi {
        manifest: Vec<u8>,
        signature: Vec<u8>,
        content: FileTree,
    }

    fn tao_goi(quyen_json: &str) -> Goi {
        let khoa = HybridEd25519MlDsa::generate();
        let mut cay = FileTree::new();
        cay.insert("index.html", b"<h1>Xin chao</h1>".to_vec())
            .unwrap();
        cay.insert("anh/logo.png", b"PNG".to_vec()).unwrap();

        let manifest = format!(
            r#"{{"spec_version":"0.1","id":"com.tcc.hello","name":"Xin chào",
"version":"1.0.0","publisher":"{}","scheme":"hybrid-ed25519-mldsa65-v1",
"content_hash":"{}","entry":"index.html","capabilities":{quyen_json}}}"#,
            hex::encode(&khoa.public),
            content_hash_hex(&cay.canonical_bytes())
        )
        .into_bytes();
        let signature = HybridEd25519MlDsa.sign(&khoa.secret, &manifest).unwrap();
        Goi {
            manifest,
            signature,
            content: cay,
        }
    }

    fn nap(
        g: Goi,
        d: impl FnMut(&CapabilityRequest) -> Decision,
    ) -> Result<LoadedApp, RuntimeError> {
        load(&g.manifest, &g.signature, g.content, &HybridEd25519MlDsa, d)
    }

    const XIN_MANG: &str = r#"[{"name":"network",
        "scope":{"kind":"network","hosts":["shop.tcc-coin.com"]},
        "reason":"tải danh sách sản phẩm"}]"#;

    #[test]
    fn nap_goi_lanh_lan_thi_dat() {
        let app = nap(tao_goi("[]"), |_| Decision::Allow).unwrap();
        assert_eq!(app.manifest().name, "Xin chào");
        assert_eq!(app.entry_content(), b"<h1>Xin chao</h1>");
    }

    #[test]
    fn doc_duoc_tep_khac_trong_goi() {
        let app = nap(tao_goi("[]"), |_| Decision::Allow).unwrap();
        assert_eq!(app.read("anh/logo.png"), Some(&b"PNG"[..]));
        assert_eq!(app.read("khong-co.txt"), None);
    }

    /// ⚠️ PHÉP THỬ QUAN TRỌNG NHẤT TỆP NÀY.
    ///
    /// Hộp thoại hỏi quyền hiện TÊN và LÝ DO lấy từ bản kê khai. Nếu ta hỏi
    /// trước khi kiểm chữ ký, thì kẻ gian viết gì người dùng đọc nấy — và người
    /// dùng bấm "cho phép" dựa trên chữ của kẻ gian.
    #[test]
    fn khong_hoi_nguoi_dung_khi_chu_ky_hong() {
        let mut g = tao_goi(XIN_MANG);
        g.signature[0] ^= 0xFF;

        let da_hoi = Cell::new(false);
        let ket = nap(g, |_| {
            da_hoi.set(true);
            Decision::Allow
        });

        assert!(ket.is_err(), "chữ ký hỏng mà vẫn nạp");
        assert!(
            !da_hoi.get(),
            "ĐÃ HỎI NGƯỜI DÙNG khi chữ ký chưa hợp lệ — hộp thoại hiện chữ của kẻ gian"
        );
    }

    /// Cũng không được làm phiền người dùng khi gói hỏng vì lý do khác.
    #[test]
    fn khong_hoi_nguoi_dung_khi_thieu_diem_vao() {
        let khoa = HybridEd25519MlDsa::generate();
        let mut cay = FileTree::new();
        cay.insert("khac.html", b"x".to_vec()).unwrap();
        let manifest = format!(
            r#"{{"spec_version":"0.1","id":"com.tcc.a","name":"A","version":"1",
"publisher":"{}","scheme":"hybrid-ed25519-mldsa65-v1","content_hash":"{}",
"entry":"index.html","capabilities":{XIN_MANG}}}"#,
            hex::encode(&khoa.public),
            content_hash_hex(&cay.canonical_bytes())
        )
        .into_bytes();
        let sig = HybridEd25519MlDsa.sign(&khoa.secret, &manifest).unwrap();

        let da_hoi = Cell::new(false);
        let ket = load(&manifest, &sig, cay, &HybridEd25519MlDsa, |_| {
            da_hoi.set(true);
            Decision::Allow
        });
        assert!(matches!(ket, Err(RuntimeError::Spec(_))));
        assert!(!da_hoi.get(), "hỏi người dùng về một gói hỏng");
    }

    #[test]
    fn nguoi_dung_tu_choi_thi_ung_dung_khong_co_quyen() {
        let app = nap(tao_goi(XIN_MANG), |_| Decision::Deny).unwrap();
        assert!(
            app.capabilities().network().is_none(),
            "từ chối mà ứng dụng vẫn có quyền mạng"
        );
    }

    #[test]
    fn duoc_cap_thi_chi_goi_duoc_may_chu_da_duyet() {
        let app = nap(tao_goi(XIN_MANG), |_| Decision::Allow).unwrap();
        let n = app.capabilities().network().unwrap();
        assert!(n.allow("shop.tcc-coin.com").is_ok());
        assert!(n.allow("evil.shop.tcc-coin.com").is_err());
    }

    /// Lý do đưa cho người quyết định phải là lý do THẬT trong bản kê khai —
    /// đây là thứ hộp thoại sẽ hiện lên.
    #[test]
    fn ly_do_dua_cho_nguoi_dung_lay_tu_ban_ke_khai() {
        let thay = Cell::new(String::new());
        let _ = nap(tao_goi(XIN_MANG), |r| {
            thay.set(r.reason.clone());
            Decision::Deny
        });
        assert_eq!(thay.take(), "tải danh sách sản phẩm");
    }

    // ---- Chạy hành động ----

    /// Mạng GIẢ, ghi lại mọi lần bị gọi.
    ///
    /// Ghi lại chứ không chỉ trả dữ liệu: phép thử quan trọng nhất ở đây là
    /// "KHÔNG gọi ra ngoài khi chưa có quyền", mà muốn khẳng định điều đó thì
    /// phải đếm được số lần gọi.
    struct MangGia {
        da_goi: std::cell::RefCell<Vec<String>>,
    }

    impl MangGia {
        fn moi() -> Self {
            Self {
                da_goi: std::cell::RefCell::new(Vec::new()),
            }
        }
        fn so_lan(&self) -> usize {
            self.da_goi.borrow().len()
        }
    }

    impl Mang for MangGia {
        fn get(&self, host: &str, path: &str) -> Result<Vec<u8>, String> {
            self.da_goi.borrow_mut().push(format!("{host}{path}"));
            Ok(b"[]".to_vec())
        }
    }

    const XIN_SHOP: &str = r#"[{"name":"network",
        "scope":{"kind":"network","hosts":["shop.tcc-coin.com"]},
        "reason":"tải hàng"}]"#;

    const HD_SHOP: &str =
        r#"[{"id":"tai-hang","effect":{"kind":"fetch","host":"shop.tcc-coin.com","path":"/ds"}}]"#;

    fn goi_co_hanh_vi(quyen: &str, hanh_dong: &str) -> Goi {
        let khoa = HybridEd25519MlDsa::generate();
        let mut cay = FileTree::new();
        cay.insert("ui.json", br#"{"kind":"text","content":"x"}"#.to_vec())
            .unwrap();
        let manifest = format!(
            r#"{{"spec_version":"0.1","id":"com.tcc.hello","name":"A","version":"1",
"publisher":"{}","scheme":"hybrid-ed25519-mldsa65-v1","content_hash":"{}",
"entry":"ui.json","capabilities":{quyen},"actions":{hanh_dong}}}"#,
            hex::encode(&khoa.public),
            content_hash_hex(&cay.canonical_bytes())
        )
        .into_bytes();
        let signature = HybridEd25519MlDsa.sign(&khoa.secret, &manifest).unwrap();
        Goi {
            manifest,
            signature,
            content: cay,
        }
    }

    #[test]
    fn duoc_cap_quyen_thi_hanh_dong_chay_duoc() {
        let app = nap(goi_co_hanh_vi(XIN_SHOP, HD_SHOP), |_| Decision::Allow).unwrap();
        let m = MangGia::moi();
        assert_eq!(app.thuc_hien("tai-hang", &m).unwrap(), b"[]");
        assert_eq!(m.da_goi.borrow().as_slice(), ["shop.tcc-coin.com/ds"]);
    }

    /// ⚠️ PHÉP THỬ QUAN TRỌNG NHẤT CỦA HÀNH VI.
    ///
    /// Không chỉ kiểm "trả về lỗi" mà kiểm **KHÔNG MỘT GÓI TIN NÀO rời khỏi
    /// máy**. Kiểm quyền sau khi gọi thì gói tin đã đến nơi rồi — mà với một máy
    /// chủ theo dõi, chỉ cần gói tin đến là đủ.
    #[test]
    fn chua_cap_quyen_thi_khong_goi_ra_ngoai_mot_lan_nao() {
        let app = nap(goi_co_hanh_vi(XIN_SHOP, HD_SHOP), |_| Decision::Deny).unwrap();
        let m = MangGia::moi();
        let ket = app.thuc_hien("tai-hang", &m);
        assert!(matches!(ket, Err(ActionError::ChuaCapQuyen(_))));
        assert_eq!(
            m.so_lan(),
            0,
            "đã gọi ra ngoài dù chưa được cấp quyền — gói tin đã rời khỏi máy"
        );
    }

    #[test]
    fn thu_hoi_roi_thi_hanh_dong_khong_chay_va_khong_goi_ra_ngoai() {
        let app = nap(goi_co_hanh_vi(XIN_SHOP, HD_SHOP), |_| Decision::Allow).unwrap();
        app.revoke_all();
        let m = MangGia::moi();
        assert!(app.thuc_hien("tai-hang", &m).is_err());
        assert_eq!(m.so_lan(), 0, "thu hồi rồi mà vẫn gọi ra ngoài");
    }

    /// Mã hành động không có trong bản kê khai ĐÃ KÝ thì không chạy gì.
    ///
    /// Mã đến từ cú bấm trên màn hình. Không tra lại ở đây thì một trang bị
    /// chiếm quyền tự bịa ra hành động được.
    #[test]
    fn hanh_dong_khong_khai_thi_khong_chay_va_khong_goi_ra_ngoai() {
        let app = nap(goi_co_hanh_vi(XIN_SHOP, HD_SHOP), |_| Decision::Allow).unwrap();
        let m = MangGia::moi();
        assert!(matches!(
            app.thuc_hien("hanh-dong-ma", &m),
            Err(ActionError::KhongCo(_))
        ));
        assert_eq!(m.so_lan(), 0);
    }

    // ---- Nạp từ đĩa ----

    /// Dựng một gói THẬT trên đĩa rồi nạp qua `load_from_dir`.
    ///
    /// Phép thử này bắt được loại lỗi mà kiểm thử trong bộ nhớ không bao giờ
    /// thấy: dạng chuẩn tắc của cây tệp phụ thuộc đường dẫn, mà đường dẫn đọc
    /// từ đĩa lên có thể khác đường dẫn ta tự gõ (dấu gạch chéo ngược trên
    /// Windows, thứ tự duyệt thư mục, tệp ẩn).
    #[test]
    fn nap_duoc_goi_that_tren_dia() {
        let thu_muc = std::env::temp_dir().join(format!("tcc-nap-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&thu_muc);
        std::fs::create_dir_all(thu_muc.join("content/anh")).unwrap();
        std::fs::write(thu_muc.join("content/index.html"), b"<h1>Xin chao</h1>").unwrap();
        std::fs::write(thu_muc.join("content/anh/logo.png"), b"PNG").unwrap();

        let khoa = HybridEd25519MlDsa::generate();
        let cay = goi::doc_noi_dung(&thu_muc).unwrap();
        let ke_khai = format!(
            r#"{{"spec_version":"0.1","id":"com.tcc.dia","name":"Tren dia",
"version":"1.0.0","publisher":"{}","scheme":"hybrid-ed25519-mldsa65-v1",
"content_hash":"{}","entry":"index.html","capabilities":[]}}"#,
            hex::encode(&khoa.public),
            content_hash_hex(&cay.canonical_bytes())
        );
        std::fs::write(thu_muc.join("manifest.json"), &ke_khai).unwrap();
        let sig = HybridEd25519MlDsa
            .sign(&khoa.secret, ke_khai.as_bytes())
            .unwrap();
        std::fs::write(thu_muc.join("signature.hex"), hex::encode(&sig)).unwrap();

        let app = load_from_dir(&thu_muc, &HybridEd25519MlDsa, |_| Decision::Allow)
            .expect("gói lành lặn trên đĩa mà nạp không được");
        assert_eq!(app.entry_content(), b"<h1>Xin chao</h1>");
        assert_eq!(app.read("anh/logo.png"), Some(&b"PNG"[..]));

        // ⚠️ Sửa một byte trong nội dung SAU khi ký thì phải hỏng.
        std::fs::write(thu_muc.join("content/index.html"), b"<h1>Ma doc</h1>").unwrap();
        assert!(
            load_from_dir(&thu_muc, &HybridEd25519MlDsa, |_| Decision::Allow).is_err(),
            "đổi nội dung trên đĩa mà vẫn nạp được"
        );

        let _ = std::fs::remove_dir_all(&thu_muc);
    }

    #[test]
    fn thieu_tep_thi_bao_ro_thieu_cai_gi() {
        let trong = std::env::temp_dir().join(format!("tcc-trong-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&trong);
        std::fs::create_dir_all(&trong).unwrap();
        let ket = load_from_dir(&trong, &HybridEd25519MlDsa, |_| Decision::Allow);
        let e = ket.unwrap_err().to_string();
        assert!(
            e.contains("manifest.json"),
            "lỗi không nêu tên tệp thiếu: {e}"
        );
        let _ = std::fs::remove_dir_all(&trong);
    }

    #[test]
    fn thu_hoi_thi_ung_dung_mat_quyen_ngay() {
        let app = nap(tao_goi(XIN_MANG), |_| Decision::Allow).unwrap();
        let n = app.capabilities().network().unwrap().clone();
        assert!(n.allow("shop.tcc-coin.com").is_ok());

        app.revoke_all();

        assert!(
            n.allow("shop.tcc-coin.com").is_err(),
            "thu hồi rồi mà bản sao cũ vẫn dùng được"
        );
    }
}
