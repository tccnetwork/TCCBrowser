//! Bản kê khai — đầu vào CHƯA XÁC THỰC đầu tiên mã ta chạm tới.
//!
//! Bộ fuzz tự viết ở `tools/tcc-fuzz` đột biến mù. Bộ này đo được nhánh nào đã
//! chạy qua, nên nó lần được vào những chỗ mà đột biến ngẫu nhiên gần như không
//! bao giờ tới. Hai bộ không thay nhau: bộ kia chạy mọi lần đẩy, bộ này chạy
//! theo lịch vì nó cần nightly và cần thời gian.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|byte: &[u8]| {
    let Ok(m) = serde_json::from_slice::<tcc_spec::Manifest>(byte) else {
        return;
    };
    if m.validate_shape().is_err() {
        return;
    }
    // Nhận rồi thì phải QUAY VÒNG được: viết ra, đọc lại, bằng nhau, vẫn hợp lệ.
    let lai = serde_json::to_vec(&m).expect("nhận rồi mà không viết lại được");
    let m2 = serde_json::from_slice::<tcc_spec::Manifest>(&lai)
        .expect("viết ra rồi đọc lại không được");
    assert_eq!(m, m2, "quay vòng ra giá trị KHÁC");
    m2.validate_shape().expect("nhận lần đầu, từ chối lần sau");
});
