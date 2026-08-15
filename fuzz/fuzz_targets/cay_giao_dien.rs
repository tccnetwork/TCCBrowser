//! Cây giao diện — chỗ nối giữa dạng trên dây và dạng đã kiểm (lỗ B16/L8).
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|byte: &[u8]| {
    let Ok(cay) = tcc_ui::wire::decode(byte) else {
        return;
    };
    let day = serde_json::from_slice::<tcc_ui::wire::UiNode>(byte)
        .expect("`decode` NHẬN nhưng dạng trên dây lại không đọc được");
    let lai = serde_json::to_vec(&day).expect("không viết lại được");
    let cay2 = tcc_ui::wire::decode(&lai).expect("viết ra rồi đọc lại bị TỪ CHỐI");
    assert_eq!(cay, cay2, "quay vòng ra cây KHÁC");
});
