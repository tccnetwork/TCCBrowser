//! `tcc` — công cụ dòng lệnh cho người viết ứng dụng TCC.
//!
//! Đây là thứ lập trình viên ngoài chạm vào ĐẦU TIÊN. Nó dở thì không ai theo
//! tiêu chuẩn, dù đặc tả có hay tới đâu. Nên hai luật cho mọi thông báo ở đây:
//!
//! 1. **Lỗi phải nói SAI Ở ĐÂU và SỬA THẾ NÀO.** "Gói không hợp lệ" là vô dụng.
//! 2. **`verify` phải in ra quyền năng ứng dụng xin.** Người kiểm gói cần thấy
//!    nó đòi những gì, không chỉ thấy chữ "hợp lệ".

// Đọc gói từ đĩa nằm ở thư viện, không ở đây — `tcc-shell` dùng chung đúng mã
// đó, nên `tcc verify` và trình duyệt không thể hiểu khác nhau.
use tcc_runtime::package;

use std::{
    fs,
    io::Write as _,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::{Parser, Subcommand};
use tcc_crypto::{HybridEd25519MlDsa, SignatureScheme};
use tcc_manifest::verify_package;
use tcc_spec::{AppId, SPEC_VERSION};

/// Tên tệp điểm vào mặc định.
///
/// `.json` chứ không `.html`: ứng dụng TCC khai BÁO có gì trên màn hình, bộ dựng
/// quyết định vẽ ra sao. Xem `tcc_ui::wire`.
const TEP_GIAO_DIEN: &str = "ui.json";

#[derive(Parser)]
#[command(name = "tcc", about = "Công cụ đóng gói và ký ứng dụng TCC", version)]
struct Cli {
    #[command(subcommand)]
    lenh: Lenh,
}

#[derive(Subcommand)]
enum Lenh {
    /// Tạo khung một ứng dụng mới
    New {
        /// Thư mục sẽ tạo
        duong_dan: PathBuf,
        /// Mã ứng dụng, kiểu tên miền ngược
        #[arg(long, default_value = "com.tcc.hello")]
        id: String,
    },
    /// Sinh cặp khoá ký
    Key {
        /// Tệp sẽ ghi khoá bí mật
        #[arg(long, default_value = "tcc-key.hex")]
        ra: PathBuf,
    },
    /// Ký một gói: tính băm nội dung, cập nhật bản kê khai, ghi chữ ký
    Sign {
        /// Thư mục gói
        duong_dan: PathBuf,
        /// Tệp khoá bí mật
        #[arg(long)]
        khoa: PathBuf,
    },
    /// Kiểm một gói và in ra những gì nó xin
    Verify {
        /// Thư mục gói
        duong_dan: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let ket = match cli.lenh {
        Lenh::New { duong_dan, id } => lenh_new(&duong_dan, &id),
        Lenh::Key { ra } => lenh_key(&ra),
        Lenh::Sign { duong_dan, khoa } => lenh_sign(&duong_dan, &khoa),
        Lenh::Verify { duong_dan } => lenh_verify(&duong_dan),
    };
    match ket {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("✗ {e}");
            ExitCode::FAILURE
        }
    }
}

fn lenh_new(duong_dan: &Path, id: &str) -> Result<(), String> {
    // Kiểm mã ứng dụng TRƯỚC khi tạo thư mục — tạo rồi mới báo lỗi thì để lại rác.
    let app_id = AppId::parse(id).map_err(|e| e.to_string())?;

    if duong_dan.exists() {
        return Err(format!(
            "\"{}\" đã tồn tại — chọn tên khác hoặc xoá thư mục cũ",
            duong_dan.display()
        ));
    }
    let noi_dung = duong_dan.join(package::CONTENT_DIR);
    fs::create_dir_all(&noi_dung).map_err(|e| e.to_string())?;

    // Điểm vào là CÂY KHAI BÁO, không phải thẻ đánh dấu.
    //
    // Từng có lúc chỗ này sinh ra `index.html`. Nó chạy được, và nó phá luật
    // trung tâm của cả dự án: ứng dụng ship HTML nghĩa là ngày có bộ dựng riêng,
    // mọi ứng dụng phải viết lại — và lúc đó không ai dám bỏ WebView nữa.
    fs::write(
        noi_dung.join(TEP_GIAO_DIEN),
        r#"{
  "kind": "group",
  "flow": "column",
  "gap": "medium",
  "children": [
    { "kind": "text", "content": "Xin chào từ TCC", "emphasis": "title" },
    { "kind": "text", "content": "Sửa tệp này để đổi giao diện." }
  ]
}
"#,
    )
    .map_err(|e| e.to_string())?;

    // `publisher` và `content_hash` để rỗng: `tcc sign` sẽ điền. Ghi số giả ở đây
    // thì người dùng dễ tưởng gói đã ký rồi.
    let ke_khai = format!(
        r#"{{
  "spec_version": "{SPEC_VERSION}",
  "id": "{}",
  "name": "Ứng dụng TCC mới",
  "version": "0.1.0",
  "publisher": "",
  "scheme": "{}",
  "content_hash": "",
  "entry": "{TEP_GIAO_DIEN}",
  "capabilities": []
}}
"#,
        app_id.as_str(),
        HybridEd25519MlDsa.name()
    );
    fs::write(duong_dan.join(package::MANIFEST_FILE), ke_khai).map_err(|e| e.to_string())?;

    println!("✓ Đã tạo {}", duong_dan.display());
    println!();
    println!("Bước tiếp theo:");
    println!("  tcc key --ra tcc-key.hex");
    println!("  tcc sign {} --khoa tcc-key.hex", duong_dan.display());
    println!("  tcc verify {}", duong_dan.display());
    Ok(())
}

fn lenh_key(ra: &Path) -> Result<(), String> {
    let khoa = HybridEd25519MlDsa::generate();

    // Mở tệp với quyền 0600 NGAY LÚC TẠO, không ghi xong rồi mới sửa quyền.
    //
    // Bản đầu làm hai bước: `fs::write` rồi `set_permissions`. Giữa hai bước
    // đó, tệp chứa khoá bí mật nằm trên đĩa với quyền mặc định của umask —
    // thường là 0644, tức là mọi tài khoản trên máy đọc được. Cửa sổ ấy ngắn,
    // nhưng nó là cửa sổ vào tệp NHẠY CẢM NHẤT trong cả hệ thống, và đóng nó
    // không tốn gì.
    //
    // `create_new` cũng thay luôn phép kiểm `exists()` trước đó: kiểm rồi ghi
    // là hai bước, và giữa hai bước ấy tệp có thể xuất hiện. Đây là một bước.
    let mut mo = fs::OpenOptions::new();
    mo.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        mo.mode(0o600);
    }
    let mut tep = mo.open(ra).map_err(|e| {
        if e.kind() == std::io::ErrorKind::AlreadyExists {
            format!(
                "\"{}\" đã tồn tại — KHÔNG ghi đè, vì ghi đè khoá là mất vĩnh viễn \
                 quyền cập nhật mọi ứng dụng đã ký bằng khoá cũ",
                ra.display()
            )
        } else {
            e.to_string()
        }
    })?;
    tep.write_all(hex::encode(&khoa.secret).as_bytes())
        .map_err(|e| e.to_string())?;

    // In VÂN TAY chứ không in cả khoá công khai: khoá lai dài hơn 2000 ký tự hex,
    // đổ hết ra là ngập màn hình. Người dùng cũng không cần chép nó nữa —
    // `tcc sign` tự suy khoá công khai từ khoá bí mật.
    let cong = hex::encode(&khoa.public);
    println!("✓ Đã ghi khoá bí mật vào {}", ra.display());
    println!(
        "  Vân tay khoá công khai: {}…{}",
        &cong[..16],
        &cong[cong.len() - 16..]
    );
    println!();
    #[cfg(not(unix))]
    println!("⚠ Trên Windows tệp này KHÔNG được đặt quyền hạn chế — tự lo lấy.");
    println!("⚠ Mất tệp này là mất quyền cập nhật mọi ứng dụng đã ký bằng nó.");
    println!("  Không có cách khôi phục. Sao lưu ra chỗ an toàn, đừng đưa lên kho mã.");
    Ok(())
}

/// Khoá công khai của khoá demo trong `examples/`.
///
/// Nhúng lúc BIÊN DỊCH từ chính tệp khoá ấy, không chép tay: chép tay là hai
/// bản sao trôi khỏi nhau, và bản trôi thì cảnh báo im lặng ngừng hoạt động.
const KHOA_DEMO_CONG_KHAI: &str = {
    // Suy ra lúc chạy sẽ tốn một lần sinh khoá cho mỗi lệnh `sign`; giá trị này
    // nằm sẵn trong bản kê khai của gói ví dụ, và luật 9 đã chốt nó không được
    // xuất hiện ở đâu khác.
    include_str!("khoa-demo-cong-khai.txt")
};

fn lenh_sign(duong_dan: &Path, tep_khoa: &Path) -> Result<(), String> {
    let hex_khoa = fs::read_to_string(tep_khoa)
        .map_err(|e| format!("không đọc được tệp khoá \"{}\": {e}", tep_khoa.display()))?;
    let bi_mat =
        hex::decode(hex_khoa.trim()).map_err(|e| format!("tệp khoá không phải hex: {e}"))?;

    let cay = package::read_content(duong_dan).map_err(|e| e.to_string())?;
    let mut h = tcc_crypto::hash::ContentHasher::new();
    cay.for_each_canonical_chunk(|c| h.update(c));
    let bam = h.finish_hex();

    // Đọc bản kê khai, điền publisher + content_hash, ghi lại, RỒI mới ký lên
    // đúng byte vừa ghi. Thứ tự này quan trọng: ký trước rồi sửa tệp là chữ ký chết.
    let goc = package::read_manifest(duong_dan).map_err(|e| e.to_string())?;
    let mut ke_khai: serde_json::Value =
        serde_json::from_slice(&goc).map_err(|e| format!("manifest.json hỏng: {e}"))?;

    // Suy khoá công khai TỪ khoá bí mật thay vì bắt người dùng dán tay. Dán tay
    // là dán nhầm, và dán nhầm thì gói ký xong không ai kiểm được.
    let khoa_cong = HybridEd25519MlDsa::public_from_secret(&bi_mat)
        .map_err(|e| format!("tệp khoá không dùng được: {e}"))?;

    // Cảnh báo NGAY LÚC KÝ nếu đang dùng khoá demo.
    //
    // Luật 9 trong `tools/kiem-luat-phu-thuoc.sh` đã chặn khoá công khai demo
    // xuất hiện trong bản kê khai ngoài `examples/` — nhưng luật đó chỉ chạy
    // TRONG kho này. Người ngoài tải kho về, thấy một tệp khoá nằm sẵn, ký bằng
    // nó, rồi phát hành: không có gì cản, và không có gì nói cho họ biết rằng
    // mọi người trên thế giới cũng có đúng khoá ấy.
    //
    // Chỗ đúng để nói là ở đây, lúc họ đang gõ lệnh.
    if hex::encode(&khoa_cong) == KHOA_DEMO_CONG_KHAI {
        eprintln!("⚠ ĐANG KÝ BẰNG KHOÁ DEMO — khoá này nằm công khai trong kho mã.");
        eprintln!("  Bất kỳ ai cũng ký được gói mang cùng danh tính nhà phát hành.");
        eprintln!("  Chỉ dùng để thử. Gói thật thì sinh khoá riêng: tcc key --ra khoa.hex");
        eprintln!();
    }

    ke_khai["publisher"] = serde_json::Value::String(hex::encode(&khoa_cong));
    ke_khai["content_hash"] = serde_json::Value::String(bam.clone());
    let moi = serde_json::to_vec_pretty(&ke_khai).map_err(|e| e.to_string())?;
    fs::write(duong_dan.join(package::MANIFEST_FILE), &moi).map_err(|e| e.to_string())?;

    let chu_ky = HybridEd25519MlDsa
        .sign(&bi_mat, &moi)
        .map_err(|e| format!("ký thất bại: {e}"))?;
    fs::write(
        duong_dan.join(package::SIGNATURE_FILE),
        hex::encode(&chu_ky),
    )
    .map_err(|e| e.to_string())?;

    println!("✓ Đã ký {}", duong_dan.display());
    println!("  {} tệp nội dung", cay.len());
    println!("  băm nội dung: {}…", &bam[..16]);
    Ok(())
}

fn lenh_verify(duong_dan: &Path) -> Result<(), String> {
    let ke_khai = package::read_manifest(duong_dan).map_err(|e| e.to_string())?;
    let chu_ky = package::read_signature(duong_dan).map_err(|e| e.to_string())?;
    let cay = package::read_content(duong_dan).map_err(|e| e.to_string())?;

    let app =
        verify_package(&ke_khai, &chu_ky, &cay, &HybridEd25519MlDsa).map_err(|e| e.to_string())?;
    let m = app.manifest();
    // Chữ ký hợp lệ chưa đủ: điểm vào phải tồn tại thật. Không kiểm ở đây thì
    // runtime nạp xong mới chết, và người dùng gặp lỗi ở chỗ khó lần ra hơn.
    m.validate_against_content(&cay)
        .map_err(|e| e.to_string())?;

    // Chữ ký hợp lệ mà cây giao diện hỏng thì gói vẫn không chạy được. Bắt ở
    // đây, lúc người viết ứng dụng còn ngồi trước máy — chứ không phải lúc
    // người dùng cuối mở gói ra và thấy một cửa sổ trống.
    let cay_giao_dien = tcc_ui::wire::decode(
        cay.get(&m.entry)
            .ok_or_else(|| format!("điểm vào \"{}\" không có trong gói", m.entry))?,
    )
    .map_err(|e| format!("điểm vào \"{}\" không dùng được: {e}", m.entry))?;

    println!("✓ Chữ ký hợp lệ");
    println!();
    println!("  Ứng dụng : {} ({})", m.name, m.id.as_str());
    println!("  Phiên bản: {}", m.version);
    println!(
        "  Ký bởi   : {}…",
        &m.publisher[..32.min(m.publisher.len())]
    );
    println!("  Nội dung : {} tệp", cay.len());
    println!(
        "  Điểm vào : {} — {} nút, sâu {} tầng",
        m.entry,
        cay_giao_dien.node_count(),
        cay_giao_dien.depth()
    );
    println!();

    if m.capabilities.is_empty() {
        println!("  Không xin quyền năng nào.");
    } else {
        println!("  Xin {} quyền năng:", m.capabilities.len());
        for c in &m.capabilities {
            println!("    • {} — {}", c.name, c.reason);
            match &c.scope {
                tcc_spec::Scope::Network { hosts } => {
                    println!("      chỉ tới: {}", hosts.join(", "));
                }
                tcc_spec::Scope::Storage { quota_bytes } => {
                    println!("      hạn mức: {quota_bytes} byte");
                }
                tcc_spec::Scope::Wallet {
                    may_request_signature,
                } => {
                    println!(
                        "      {}",
                        if *may_request_signature {
                            "ĐƯỢC xin chữ ký giao dịch"
                        } else {
                            "chỉ đọc địa chỉ"
                        }
                    );
                }
            }
        }
    }

    println!();
    if m.actions.is_empty() {
        println!("  Không nút nào có hành vi — ứng dụng chỉ hiện thông tin.");
    } else {
        println!("  {} nút có hành vi:", m.actions.len());
        for a in &m.actions {
            match &a.effect {
                tcc_spec::Effect::Fetch { host, path } => {
                    println!("    • {} → gọi {host}{path}", a.id.as_str());
                }
            }
        }
        println!("    (mọi máy chủ trên đây đã được kiểm là nằm trong quyền đã xin)");
    }

    println!();
    println!("⚠ Chữ ký hợp lệ chứng minh gói KHÔNG BỊ SỬA.");
    println!("  Nó KHÔNG chứng minh người ký là ai — bất kỳ ai cũng tự sinh khoá được.");
    Ok(())
}
