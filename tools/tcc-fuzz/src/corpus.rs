//! Tập hạt giống: lấy từ VECTOR KIỂM ĐỊNH và gói ví dụ, không tự bịa.
//!
//! Đột biến từ dữ liệu hợp lệ thật đi sâu hơn hẳn đột biến từ chuỗi rỗng: một
//! bản kê khai ngẫu nhiên gần như luôn hỏng ngay ở dấu ngoặc đầu tiên, còn một
//! bản kê khai hợp lệ bị lật một bit thì đi qua được bộ đọc JSON rồi mới va vào
//! các phép kiểm — chính là chỗ đáng soi.

/// Nhúng lúc BIÊN DỊCH chứ không đọc lúc chạy: bộ fuzz phải chạy được từ bất kỳ
/// thư mục nào, và không được im lặng bỏ qua hạt giống khi đường dẫn sai.
const VECTORS_MANIFEST: &str = include_str!("../../../conformance/vectors/manifest.json");
const VECTORS_UI: &str = include_str!("../../../conformance/vectors/ui.json");
const EXAMPLE_MANIFEST: &str = include_str!("../../../examples/hello-tcc/manifest.json");
const EXAMPLE_UI: &str = include_str!("../../../examples/hello-tcc/content/ui.json");

const EXAMPLE_SIGNATURE: &str = include_str!("../../../examples/hello-tcc/signature.hex");

/// Hạt giống chia theo LOẠI. Trộn chung thì mục tiêu chữ ký sẽ ăn toàn JSON —
/// mà JSON thì trượt ngay ở phép kiểm độ dài, không bao giờ chạm tới mã mật mã.
pub struct Corpus {
    /// Bản kê khai và cây giao diện.
    pub text: Vec<Vec<u8>>,
    /// Chữ ký lai thật, 3373 byte.
    pub signature: Vec<Vec<u8>>,
    /// Khoá công khai lai thật, 1984 byte.
    pub public_key: Vec<Vec<u8>>,
    /// Khoá bí mật lai, 64 byte.
    pub secret_key: Vec<Vec<u8>>,
}

/// Chuỗi hex → byte. Viết tay để không kéo thêm phụ thuộc vào một công cụ kiểm.
#[must_use]
pub fn unhex(s: &str) -> Vec<u8> {
    let t: Vec<u8> = s.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    t.chunks_exact(2)
        .filter_map(|c| {
            let hi = char::from(c[0]).to_digit(16)?;
            let lo = char::from(c[1]).to_digit(16)?;
            u8::try_from(hi * 16 + lo).ok()
        })
        .collect()
}

/// Thông điệp được ký chính là BYTE THÔ của `manifest.json`, không phải cấu
/// trúc đã giải mã. Đó là một câu của tiêu chuẩn, và nếu ai đổi nó thì mục tiêu
/// fuzz này sẽ ngừng kiểm được thứ nó định kiểm.
#[must_use]
pub fn signed_message() -> Vec<u8> {
    EXAMPLE_MANIFEST.as_bytes().to_vec()
}

/// Khoá công khai và chữ ký THẬT của gói ví dụ.
#[must_use]
pub fn real_pair() -> (Vec<u8>, Vec<u8>) {
    let khoa = serde_json::from_str::<serde_json::Value>(EXAMPLE_MANIFEST)
        .ok()
        .and_then(|v| v["publisher"].as_str().map(unhex))
        .unwrap_or_default();
    (khoa, unhex(EXAMPLE_SIGNATURE))
}

/// Số hạt giống văn bản TỐI THIỂU.
///
/// Bộ nạp này đọc khoá của tệp vector. Khi tập vector đổi khoá từ `truong_hop`
/// sang `cases`, bộ nạp im lặng tụt từ 55 hạt giống xuống 7 — và bộ fuzz vẫn
/// báo "ĐẠT" suốt nhiều lượt chạy trong khi hầu như không có gì để đột biến.
///
/// Một bộ fuzz mất hạt giống không kêu lên; nó chỉ ngừng tìm ra thứ gì. Nên
/// đây là cái chốt: thiếu hạt giống là HỎNG, không phải là chạy nhanh hơn.
const TOI_THIEU_VAN_BAN: usize = 40;

/// Rút mọi mẫu từ vector rồi trả về dưới dạng byte.
///
/// # Panics
/// Khi số hạt giống văn bản tụt dưới [`TOI_THIEU_VAN_BAN`].
#[must_use]
pub fn load() -> Corpus {
    let (khoa, chu_ky) = real_pair();
    let mut ra: Vec<Vec<u8>> = vec![
        EXAMPLE_MANIFEST.as_bytes().to_vec(),
        EXAMPLE_UI.as_bytes().to_vec(),
    ];

    for (van, khoa) in [(VECTORS_MANIFEST, "manifest"), (VECTORS_UI, "tree")] {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(van) else {
            continue;
        };
        if let Some(cac) = v["cases"].as_array() {
            for t in cac {
                let muc = &t[khoa];
                if !muc.is_null()
                    && let Ok(s) = serde_json::to_vec(muc)
                {
                    ra.push(s);
                }
            }
        }
    }

    // Vài hình dạng thô mà vector không có: rỗng, chỉ dấu ngoặc, lồng sâu.
    ra.push(Vec::new());
    ra.push(b"{}".to_vec());
    ra.push(b"[]".to_vec());
    ra.push(br#"{"kind":"group","children":[]}"#.to_vec());
    let mut sau = String::new();
    for _ in 0..64 {
        sau.push_str(r#"{"kind":"group","children":["#);
    }
    sau.push_str(&"]}".repeat(64));
    ra.push(sau.into_bytes());

    assert!(
        ra.len() >= TOI_THIEU_VAN_BAN,
        "chỉ nạp được {} hạt giống văn bản, cần ít nhất {TOI_THIEU_VAN_BAN} — \
         tập vector đã đổi khoá mà bộ nạp chưa theo?",
        ra.len()
    );

    Corpus {
        text: ra,
        // Chữ ký cụt và chữ ký đảo hai nửa: bố cục byte là một phần của tiêu
        // chuẩn, nên đảo nửa phải bị từ chối chứ không được hoảng loạn.
        signature: vec![
            chu_ky.clone(),
            chu_ky.iter().copied().rev().collect(),
            chu_ky[..64].to_vec(),
            Vec::new(),
        ],
        public_key: vec![khoa.clone(), khoa[..32].to_vec(), Vec::new()],
        // Hạt giống khoá bí mật: một khoá hợp lệ, một khoá cụt, và rỗng.
        secret_key: vec![(0..64u8).collect(), vec![0u8; 32], Vec::new()],
    }
}
