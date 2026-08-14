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

/// Rút mọi mẫu từ vector rồi trả về dưới dạng byte.
#[must_use]
pub fn load() -> Vec<Vec<u8>> {
    let mut ra: Vec<Vec<u8>> = vec![
        EXAMPLE_MANIFEST.as_bytes().to_vec(),
        EXAMPLE_UI.as_bytes().to_vec(),
    ];

    for (van, khoa) in [(VECTORS_MANIFEST, "ke_khai"), (VECTORS_UI, "cay")] {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(van) else {
            continue;
        };
        if let Some(cac) = v["truong_hop"].as_array() {
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

    ra
}
