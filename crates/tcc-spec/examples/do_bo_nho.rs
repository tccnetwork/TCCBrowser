//! Đo bộ nhớ đỉnh của hai đường băm. Chạy TAY, không nằm trong CI.
//!
//! Mỗi đường phải chạy trong MỘT TIẾN TRÌNH RIÊNG: bộ cấp phát không trả bộ nhớ
//! về hệ điều hành ngay sau `drop`, nên đo hai đường trong cùng một tiến trình
//! cho ra con số của đường tốn nhiều hơn. Bản đầu tôi đo chung và suýt kết luận
//! nhầm rằng bản theo luồng không tiết kiệm gì.
//!
//! ```sh
//! cargo run -p tcc-spec --release --example do_bo_nho -- mot-lan 64
//! cargo run -p tcc-spec --release --example do_bo_nho -- theo-luong 64
//! ```

#![allow(
    clippy::expect_used,
    clippy::print_stdout,
    reason = "công cụ đo chạy tay: hỏng thì nổ ngay, và nó phải in ra"
)]

fn main() {
    let mut a = std::env::args().skip(1);
    let cach = a.next().unwrap_or_else(|| "theo-luong".to_owned());
    let mb: usize = a.next().and_then(|s| s.parse().ok()).unwrap_or(64);

    let mut cay = tcc_spec::tree::FileTree::new();
    for i in 0..mb {
        cay.insert(&format!("f{i:04}.bin"), vec![0u8; 1024 * 1024])
            .expect("chèn tệp hỏng");
    }
    let nen = rss();

    let n = if cach == "mot-lan" {
        cay.canonical_bytes().len() as u64
    } else {
        let mut n: u64 = 0;
        cay.for_each_canonical_chunk(|c| n += c.len() as u64);
        n
    };

    println!("  cách {cach:<11} nội dung {mb} MiB · {n} byte đi qua");
    println!(
        "  RSS: {nen} MiB (chỉ có cây) → {} MiB (sau khi băm)",
        rss()
    );
}

fn rss() -> u64 {
    std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .ok()
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .trim()
                .parse::<u64>()
                .ok()
        })
        .unwrap_or(0)
        / 1024
}
