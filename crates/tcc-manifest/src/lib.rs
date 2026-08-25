//! Đọc, kiểm và xác thực gói ứng dụng TCC.
//!
//! # Chữ ký ký lên CÁI GÌ — quyết định tiêu chuẩn quan trọng nhất ở đây
//!
//! Chữ ký ký lên **đúng chuỗi byte của tệp `manifest.json` như nó nằm trên
//! đĩa**, KHÔNG phải lên cấu trúc sau khi giải mã rồi tuần tự hoá lại.
//!
//! Vì sao — đây là chỗ nhiều tiêu chuẩn đã chết:
//!
//! Nếu ký lên *cấu trúc*, thì mỗi bản cài đặt phải tuần tự hoá ra **đúng cùng
//! một chuỗi byte** mới khớp chữ ký. Nhưng thứ tự khoá, khoảng trắng, cách viết
//! số thực, cách thoát Unicode — mỗi thư viện JSON làm một kiểu. Muốn khớp thì
//! phải có "dạng chuẩn tắc", và dạng chuẩn tắc là một đặc tả phụ đầy cạm bẫy mà
//! ai cũng cài đặt sai một góc.
//!
//! Ký thẳng lên byte thì bài toán đó **biến mất**: bên kiểm ký lên đúng thứ nó
//! nhận được. Cái giá là không thể sửa bản kê khai rồi giữ nguyên chữ ký — mà ta
//! không cần điều đó bao giờ.
//!
//! # Chuỗi tin cậy
//!
//! ```text
//! chữ ký ──ký──▶ byte manifest.json ──chứa──▶ content_hash ──băm──▶ dạng
//!                                                                   chuẩn tắc
//!                                                                   của cây tệp
//! ```
//!
//! Sửa ở mắt nào cũng đứt chuỗi.
//!
//! # ⚠️ Chữ ký chứng minh TOÀN VẸN, không chứng minh DANH TÍNH
//!
//! Khoá công khai của người phát hành nằm ngay trong bản kê khai, tức gói này
//! **tự ký**. Nó chứng minh gói không bị sửa bởi người không có khoá — nhưng
//! **bất kỳ ai cũng tự sinh được một cặp khoá rồi ký gói của mình**.
//!
//! Muốn biết "khoá này có đúng là của nhà phát hành X không" thì cần một tầng
//! nữa: sổ đăng ký, hoặc tin-lần-đầu rồi ghim khoá lại. Tầng đó CHƯA CÓ ở phiên
//! bản 0.1. Đừng để giao diện nói "đã xác minh nhà phát hành" khi mới chỉ kiểm
//! được chữ ký.

use tcc_crypto::{CryptoError, SignatureScheme};

/// Trần kích thước `manifest.json`.
///
/// Bản kê khai thật chỉ vài kilobyte. Không có trần thì một gói độc hại gửi tệp
/// hàng trăm megabyte và ta phân tích JSON hết chỗ đó TRƯỚC khi kiểm được chữ ký
/// — tức tiêu tài nguyên cho dữ liệu chưa hề được xác thực. Trần này phải kiểm
/// TRƯỚC MỌI THỨ KHÁC.
pub const MAX_MANIFEST_BYTES: usize = 64 * 1024;
use tcc_spec::{FileTree, Manifest, SpecError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("manifest.json không phải JSON hợp lệ: {0}")]
    BadJson(String),

    #[error("bản kê khai sai tiêu chuẩn: {0}")]
    Spec(#[from] SpecError),

    #[error("chữ ký không hợp lệ: {0}")]
    Signature(#[from] CryptoError),

    #[error("khoá công khai trong bản kê khai không phải hex")]
    PublisherNotHex,

    #[error("bản kê khai khai bộ ký \"{declared}\" nhưng đang kiểm bằng \"{available}\"")]
    SchemeMismatch {
        declared: String,
        available: &'static str,
    },

    #[error("nội dung không khớp bản kê khai: chờ băm {expected}, tính ra {actual}")]
    ContentHashMismatch { expected: String, actual: String },

    #[error("manifest.json quá lớn: {actual} byte, trần {MAX_MANIFEST_BYTES}")]
    ManifestTooLarge { actual: usize },
}

impl ManifestError {
    /// Mã lỗi ỔN ĐỊNH, thuộc về TIÊU CHUẨN.
    ///
    /// ⚠️ Nhánh bọc lỗi trả mã của NGUYÊN NHÂN GỐC, không phải mã lớp vỏ. Một
    /// bản kê khai hỏng vì ký tự giả mạo phải ra `unsafe-display-string`, chứ
    /// `spec` thì không nói lên điều gì và bộ kiểm định không so khớp được.
    #[must_use]
    pub const fn ma(&self) -> &'static str {
        match self {
            Self::BadJson { .. } => "bad-json",
            Self::Spec(e) => e.ma(),
            Self::Signature(e) => e.ma(),
            Self::PublisherNotHex { .. } => "publisher-not-hex",
            Self::SchemeMismatch { .. } => "scheme-mismatch",
            Self::ContentHashMismatch { .. } => "content-hash-mismatch",
            Self::ManifestTooLarge { .. } => "manifest-too-large",
        }
    }
}

/// Kết quả xác thực. Chỉ dựng được qua `verify_package`, nên cầm được nó nghĩa
/// là mọi phép kiểm đã qua — kiểu dữ liệu tự nó là bằng chứng.
#[derive(Debug, Clone)]
pub struct VerifiedApp {
    manifest: Manifest,
}

impl VerifiedApp {
    #[must_use]
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }
}

/// Xác thực một gói ứng dụng.
///
/// Nhận byte thô chứ không nhận đường dẫn: cách đóng gói (thư mục, zip, tải qua
/// mạng) là việc của tầng trên, còn hàm này kiểm thuần tuý và kiểm thử được mà
/// không cần đụng đĩa.
///
/// Thứ tự kiểm là CÓ CHỦ ĐÍCH — rẻ trước, đắt sau, và không tin gì trong bản kê
/// khai cho tới khi chữ ký qua được:
///
/// 0. **Chặn trần kích thước** — trước khi tiêu tài nguyên cho dữ liệu chưa xác thực
/// 1. Phân tích JSON
/// 2. Kiểm định dạng theo tiêu chuẩn
/// 3. Kiểm tên bộ ký khớp
/// 4. **Kiểm chữ ký trên byte thô**
/// 5. Băm nội dung, so với bản kê khai
///
/// # Errors
/// Bất kỳ bước nào ở trên thất bại.
pub fn verify_package(
    manifest_bytes: &[u8],
    signature: &[u8],
    content: &FileTree,
    scheme: &dyn SignatureScheme,
) -> Result<VerifiedApp, ManifestError> {
    // 0 — chặn trước khi tiêu bất kỳ tài nguyên nào cho dữ liệu chưa xác thực
    if manifest_bytes.len() > MAX_MANIFEST_BYTES {
        return Err(ManifestError::ManifestTooLarge {
            actual: manifest_bytes.len(),
        });
    }

    // 1 & 2
    let manifest: Manifest = serde_json::from_slice(manifest_bytes)
        .map_err(|e| ManifestError::BadJson(e.to_string()))?;
    manifest.validate_shape()?;

    // 3 — kiểm trước khi tốn công mật mã, và báo lỗi dễ hiểu hơn "chữ ký sai"
    if manifest.scheme != scheme.name() {
        return Err(ManifestError::SchemeMismatch {
            declared: manifest.scheme.clone(),
            available: scheme.name(),
        });
    }

    // 4 — ký lên BYTE THÔ, không phải lên cấu trúc đã giải mã
    let publisher = hex::decode(&manifest.publisher).map_err(|_| ManifestError::PublisherNotHex)?;
    scheme.verify(&publisher, manifest_bytes, signature)?;

    // 5 — tới đây bản kê khai đã đáng tin; giờ mới so nội dung với nó.
    // Băm lên DẠNG CHUẨN TẮC của cây tệp, không phải lên từng tệp rời: mọi
    // trường đều có độ dài ghi trước nên hai cây khác nhau không thể ra cùng một
    // chuỗi byte. Xem `tcc_spec::tree`.
    // Băm theo LUỒNG: không dựng bản sao đầy đủ của nội dung chỉ để băm nó.
    // Với trần 256 MiB, bản sao ấy là nửa gigabyte cho một gói chưa xác thực.
    let mut h = tcc_crypto::hash::ContentHasher::new();
    content.for_each_canonical_chunk(|c| h.update(c));
    let actual = h.finish_hex();
    if actual != manifest.content_hash {
        return Err(ManifestError::ContentHashMismatch {
            expected: manifest.content_hash.clone(),
            actual,
        });
    }

    Ok(VerifiedApp { manifest })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "kiểm thử: hỏng thì phải nổ ngay")]
mod kiem_thu {
    use super::*;
    // Chỉ phép thử cần hàm này; đường chạy thật băm theo LUỒNG.
    use tcc_crypto::HybridEd25519MlDsa;
    use tcc_crypto::content_hash_hex;
    use tcc_crypto::hybrid::KeyPair;

    struct Goi {
        manifest: Vec<u8>,
        signature: Vec<u8>,
        content: FileTree,
    }

    fn cay(noi_dung: &[u8]) -> FileTree {
        let mut t = FileTree::new();
        t.insert("app.wasm", noi_dung.to_vec()).unwrap();
        t
    }

    fn tao_goi(noi_dung: &[u8]) -> (Goi, KeyPair) {
        let khoa = HybridEd25519MlDsa::generate();
        let cay_tep = cay(noi_dung);
        let manifest = format!(
            r#"{{
  "spec_version": "0.1",
  "id": "com.tcc.hello",
  "name": "Hello TCC",
  "version": "1.0.0",
  "publisher": "{}",
  "scheme": "hybrid-ed25519-mldsa65-v1",
  "content_hash": "{}",
  "entry": "app.wasm",
  "capabilities": []
}}"#,
            hex::encode(&khoa.public),
            content_hash_hex(&cay_tep.canonical_bytes())
        )
        .into_bytes();
        let signature = HybridEd25519MlDsa.sign(&khoa.secret, &manifest).unwrap();
        (
            Goi {
                manifest,
                signature,
                content: cay_tep,
            },
            khoa,
        )
    }

    fn kiem(g: &Goi) -> Result<VerifiedApp, ManifestError> {
        verify_package(&g.manifest, &g.signature, &g.content, &HybridEd25519MlDsa)
    }

    #[test]
    fn goi_lanh_lan_thi_dat() {
        let (g, _) = tao_goi(b"noi dung ung dung");
        let app = kiem(&g).unwrap();
        assert_eq!(app.manifest().id.as_str(), "com.tcc.hello");
    }

    /// ⚠️ PHÉP THỬ TRUNG TÂM: đổi nội dung nhưng giữ nguyên bản kê khai và chữ ký.
    ///
    /// Đây là đòn hiển nhiên nhất — lấy một gói đã ký của người khác, thay ruột.
    /// Nếu chữ ký không bao trùm được nội dung qua `content_hash` thì đòn này lọt.
    #[test]
    fn thay_ruot_giu_chu_ky_thi_hong() {
        let (mut g, _) = tao_goi(b"ung dung lanh");
        g.content = cay(b"ung dung doc hai");
        assert!(matches!(
            kiem(&g),
            Err(ManifestError::ContentHashMismatch { .. })
        ));
    }

    /// Sửa bản kê khai (xin thêm quyền) mà giữ chữ ký cũ.
    #[test]
    fn sua_ban_ke_khai_giu_chu_ky_thi_hong() {
        let (mut g, _) = tao_goi(b"noi dung");
        let s = String::from_utf8(g.manifest.clone()).unwrap();
        g.manifest = s
            .replace("\"capabilities\": []", "\"capabilities\":[]")
            .into_bytes();
        // Chỉ bỏ một dấu cách — nội dung ngữ nghĩa y hệt, byte thì khác.
        // Ký lên BYTE nên phải hỏng. Đây chính là cái giá đã chấp nhận, và là
        // bằng chứng ta đang ký byte chứ không ký cấu trúc.
        assert!(matches!(kiem(&g), Err(ManifestError::Signature(_))));
    }

    /// Ký bằng khoá của mình rồi khai khoá công khai của người khác.
    #[test]
    fn khoa_cong_khai_gia_thi_hong() {
        let (g, _) = tao_goi(b"noi dung");
        let ke_gian = HybridEd25519MlDsa::generate();
        let s = String::from_utf8(g.manifest.clone()).unwrap();
        let sua = s.split("\"publisher\": \"").next().unwrap().to_string()
            + "\"publisher\": \""
            + &hex::encode(&ke_gian.public)
            + "\","
            + s.split_once("\",\n  \"scheme\"").unwrap().1;
        let g2 = Goi {
            manifest: sua.into_bytes(),
            signature: g.signature.clone(),
            content: g.content.clone(),
        };
        assert!(kiem(&g2).is_err());
    }

    #[test]
    fn json_hong_thi_bao_ro() {
        let g = Goi {
            manifest: b"{khong phai json".to_vec(),
            signature: vec![0; 3373],
            content: FileTree::new(),
        };
        assert!(matches!(kiem(&g), Err(ManifestError::BadJson(_))));
    }

    #[test]
    fn khai_sai_ten_bo_ky_thi_bao_ro_chu_khong_bao_chu_ky_sai() {
        let (g, _) = tao_goi(b"noi dung");
        let s = String::from_utf8(g.manifest).unwrap();
        let g2 = Goi {
            manifest: s
                .replace("hybrid-ed25519-mldsa65-v1", "rsa-2048")
                .into_bytes(),
            signature: g.signature,
            content: g.content,
        };
        assert!(matches!(
            kiem(&g2),
            Err(ManifestError::SchemeMismatch { .. })
        ));
    }

    /// ⚠️ LỖ 2. Không có trần thì ta phân tích JSON hàng trăm megabyte của dữ
    /// liệu CHƯA HỀ ĐƯỢC XÁC THỰC. Trần phải chặn TRƯỚC bước phân tích.
    #[test]
    fn manifest_qua_lon_thi_chan_truoc_khi_phan_tich() {
        let g = Goi {
            manifest: vec![b'{'; MAX_MANIFEST_BYTES + 1],
            signature: vec![0; 3373],
            content: FileTree::new(),
        };
        assert!(
            matches!(kiem(&g), Err(ManifestError::ManifestTooLarge { .. })),
            "phải chặn bằng trần kích thước, không phải bằng lỗi JSON"
        );
    }

    /// ⚠️ LỖ 5 — HOÁ RA KHÔNG PHẢI LỖ, nhưng ta đang DỰA VÀO hành vi này.
    ///
    /// Nếu bộ phân tích JSON chấp nhận khoá trùng và lấy giá trị cuối, thì kẻ
    /// gian ghi `publisher` hai lần: công cụ hiển thị cho người duyệt đọc giá trị
    /// đầu, còn bên kiểm chữ ký dùng giá trị cuối. serde từ chối thẳng — phép thử
    /// này ghim hành vi đó lại, để ngày ai đó đổi thư viện JSON thì nó gãy ngay.
    #[test]
    fn khoa_json_trung_lap_bi_tu_choi() {
        let (g, _) = tao_goi(b"noi dung");
        let s = String::from_utf8(g.manifest).unwrap();
        let trung = s.replace("\"scheme\":", "\"publisher\": \"deadbeef\",\n  \"scheme\":");
        let g2 = Goi {
            manifest: trung.into_bytes(),
            signature: g.signature,
            content: g.content,
        };
        match kiem(&g2) {
            Err(ManifestError::BadJson(e)) => {
                assert!(e.contains("duplicate"), "phải nói rõ là khoá trùng: {e}");
            }
            khac => panic!("khoá trùng phải bị từ chối ở bước JSON, nhận: {khac:?}"),
        }
    }

    /// ⚠️ Đòn "hai cây khác nhau, cùng một băm" — đòn mà dạng chuẩn tắc sinh ra
    /// để chặn. Ký gói có tệp "ab" nội dung "c", rồi tráo sang tệp "a" nội dung
    /// "bc". Không ghi độ dài trước thì cả hai đều ra chuỗi "abc" và chữ ký nhận
    /// nhầm.
    #[test]
    fn khong_trao_duoc_cay_khac_ma_giu_chu_ky() {
        let khoa = HybridEd25519MlDsa::generate();
        let mut cay_that = FileTree::new();
        cay_that.insert("ab", b"c".to_vec()).unwrap();

        let manifest = format!(
            r#"{{"spec_version":"0.1","id":"com.tcc.hello","name":"Hello",
"version":"1.0.0","publisher":"{}","scheme":"hybrid-ed25519-mldsa65-v1",
"content_hash":"{}","entry":"ab","capabilities":[]}}"#,
            hex::encode(&khoa.public),
            content_hash_hex(&cay_that.canonical_bytes())
        )
        .into_bytes();
        let chu_ky = HybridEd25519MlDsa.sign(&khoa.secret, &manifest).unwrap();

        // Cây tráo: cùng "nội dung nối lại" nhưng ranh giới tên/nội dung dịch đi
        let mut cay_trao = FileTree::new();
        cay_trao.insert("a", b"bc".to_vec()).unwrap();

        let ket = verify_package(&manifest, &chu_ky, &cay_trao, &HybridEd25519MlDsa);
        assert!(
            matches!(ket, Err(ManifestError::ContentHashMismatch { .. })),
            "tráo được cây khác mà chữ ký vẫn hợp lệ — dạng chuẩn tắc bị nhập nhằng"
        );
    }

    /// Độ dài băm bên tcc-crypto và hằng số bên tcc-spec phải khớp nhau.
    /// Hai crate không phụ thuộc nhau (đều là lá), nên chốt ở đây.
    #[test]
    fn do_dai_bam_hai_crate_khop_nhau() {
        assert_eq!(content_hash_hex(b"x").len(), tcc_spec::CONTENT_HASH_HEX_LEN);
    }

    /// **Đúng 64 KiB phải ĐƯỢC NHẬN; hơn một byte thì không.**
    ///
    /// `spec/0.1/02-manifest.md:3` ghi "at most **64 KiB**" — dải bao gồm đầu
    /// trên. `cargo-mutants` đổi `>` thành `>=` mà không phép thử nào đỏ: phép
    /// thử cũ chỉ thử bản kê khai nhỏ hoặc lớn hẳn, không cái nào đứng ở mép.
    /// Cùng hạng với sáu ranh giới đã vá trong `tcc-spec`.
    #[test]
    fn dung_tran_bản_ke_khai_thi_van_qua() {
        // Đệm cho tổng byte đúng bằng trần, rồi thêm một byte.
        let dem = |tong: usize| {
            let g = Goi {
                manifest: vec![b'{'; tong],
                signature: vec![0; 3373],
                content: FileTree::new(),
            };
            kiem(&g)
        };
        assert!(
            !matches!(
                dem(MAX_MANIFEST_BYTES),
                Err(ManifestError::ManifestTooLarge { .. })
            ),
            "đúng {MAX_MANIFEST_BYTES} byte bị chặn bằng trần — đặc tả nói được nhận"
        );
        assert!(
            matches!(
                dem(MAX_MANIFEST_BYTES + 1),
                Err(ManifestError::ManifestTooLarge { .. })
            ),
            "hơn trần một byte mà không bị chặn"
        );
    }

    /// **`publisher-not-hex` KHÔNG gói nào tạo ra được — ghim lời ấy lại.**
    ///
    /// `spec/0.1/06-error-codes.md` gỡ mã này khỏi tiêu chuẩn với lý lẽ: phép
    /// kiểm hình dạng chối một `publisher` không phải hex bằng `not-hex`, và nó
    /// chạy TRƯỚC. Lời ấy đúng chừng nào thứ tự trong `verify_package` còn giữ
    /// nguyên — đổi chỗ hai bước là bản này bắn ra một mã tiêu chuẩn nói là
    /// không tồn tại.
    ///
    /// Ngày 25/08/2026 đúng chuyện ấy đã xảy ra với `bad-key`: lý lẽ khi gỡ nó
    /// dựa trên giả định về thư viện Ed25519, giả định sai, và không phép thử
    /// nào canh. Đây là phép thử đáng lẽ phải có từ đầu, cho mã còn lại.
    #[test]
    fn goi_co_publisher_khong_phai_hex_ra_ma_not_hex() {
        let mut g = tao_goi(b"x");
        let mut ke: serde_json::Value = serde_json::from_slice(&g.0.manifest).unwrap();
        ke["publisher"] = serde_json::Value::String("khong-phai-hex".to_owned());
        g.0.manifest = serde_json::to_vec(&ke).unwrap();

        let ma = kiem(&g.0).unwrap_err().ma();
        assert_eq!(
            ma, "not-hex",
            "gói tạo ra được mã `{ma}` — nếu là `publisher-not-hex` thì đặc tả \
             đang nói sai, y như `bad-key`"
        );
    }
}
