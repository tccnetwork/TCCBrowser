//! Đường dẫn trong gói, và tính TẤT ĐỊNH của dạng chuẩn tắc.
//!
//! Hai đường dựng dạng chuẩn tắc phải cho cùng chuỗi byte: lệch một byte là băm
//! khác, là chữ ký của bên này bên kia không kiểm được.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|byte: &[u8]| {
    let Ok(s) = core::str::from_utf8(byte) else {
        return;
    };
    let mut cay = tcc_spec::tree::FileTree::new();
    for (i, phan) in s.split('\n').take(32).enumerate() {
        let _ = cay.insert(phan, vec![u8::try_from(i & 0xff).unwrap_or(0)]);
    }
    let mot_lan = cay.canonical_bytes();
    let mut theo_luong = Vec::new();
    cay.for_each_canonical_chunk(|c| theo_luong.extend_from_slice(c));
    assert_eq!(mot_lan, theo_luong, "hai đường dựng dạng chuẩn tắc KHÁC nhau");
});
