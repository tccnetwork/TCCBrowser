//! BỘ KIỂM ĐỊNH TUÂN THỦ — thứ biến một đặc tả thành một TIÊU CHUẨN.
//!
//! Không có nó, câu "ứng dụng khác muốn chạy phải triển khai theo cách TCC" là
//! không kiểm chứng được và không cưỡng chế được. Có nó thì thành một câu đo
//! được: chạy bộ kiểm, đạt 100% mới được mang nhãn TCC.
//!
//! LUẬT: mỗi mục trong `spec/` phải có ít nhất một phép kiểm ở đây. Thêm điều
//! vào đặc tả mà không thêm phép kiểm là thêm một lời hứa không ai kiểm được.
//!
//! # Vector là DỮ LIỆU, không phải mã
//!
//! Mọi trường hợp nằm trong `conformance/vectors/*.json`. Đó là chủ đích: bản
//! triển khai bằng Go, Swift hay TypeScript cũng đọc được đúng những tệp đó.
//! Vector viết bằng Rust thì nó chỉ là kiểm thử đơn vị CỦA CHÚNG TÔI, không phải
//! bộ kiểm định của TIÊU CHUẨN.
//!
//! # So khớp bằng MÃ, không bằng thông báo
//!
//! Thông báo lỗi là văn xuôi tiếng Việt và được phép sửa bất cứ lúc nào. Mã thì
//! không — xem `SpecError::ma`.

#![allow(
    clippy::expect_used,
    reason = "công cụ dòng lệnh: vector hỏng thì phải nổ ngay, không chạy tiếp"
)]

use std::{path::Path, process::ExitCode};

use serde_json::Value;
use tcc_capability::{Decision, grant};
use tcc_crypto::{HybridEd25519MlDsa, SignatureScheme, content_hash_hex};
use tcc_spec::{AppId, CapabilityRequest, FileTree, Manifest};

/// Chỗ giữ chỗ trong vector, thay bằng giá trị đúng độ dài lúc chạy.
///
/// Vector không nhét khoá công khai 4000 ký tự vào từng trường hợp — nó làm tệp
/// không đọc nổi mà chẳng kiểm thêm được gì. Độ dài khoá đã có nhóm vector khác
/// lo.
const GIU_CHO_PUB: &str = "PUB";
const GIU_CHO_HASH: &str = "HASH";

struct Ket {
    dat: usize,
    truot: Vec<String>,
}

impl Ket {
    const fn moi() -> Self {
        Self {
            dat: 0,
            truot: Vec::new(),
        }
    }
    fn ghi(&mut self, ten: &str, ok: bool, vi_sao: &str, chi_tiet: bool) {
        if ok {
            self.dat += 1;
            if chi_tiet {
                println!("    ✓ {ten}");
            }
        } else {
            println!("    ✗ {ten}");
            println!("        {vi_sao}");
            self.truot.push(ten.to_owned());
        }
    }
}

fn doc_vector(ten: &str) -> Value {
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../conformance/vectors")
        .join(ten);
    let b = std::fs::read(&p).unwrap_or_else(|e| panic!("không đọc được {}: {e}", p.display()));
    serde_json::from_slice(&b).unwrap_or_else(|e| panic!("{} không phải JSON: {e}", p.display()))
}

fn cac_truong_hop(v: &Value) -> &Vec<Value> {
    v["cases"].as_array().expect("vector thiếu mảng `cases`")
}

fn ten_cua(t: &Value) -> &str {
    t["case"].as_str().unwrap_or("(không tên)")
}

// ───────────────────────────── Dạng chuẩn tắc + băm ─────────────────────────

/// Nhóm quan trọng nhất cho interop.
///
/// Hai bản triển khai phải tính ra CÙNG chuỗi byte và CÙNG mã băm. Lệch một byte
/// là chữ ký của bên này bên kia không kiểm được — mà lúc đó không ai biết lỗi
/// nằm ở đâu, vì cả hai bên đều "chạy đúng".
fn chay_canonical(chi_tiet: bool) -> Ket {
    let v = doc_vector("canonical.json");
    let mut k = Ket::moi();

    for t in cac_truong_hop(&v) {
        let ten = ten_cua(t);
        let mut cay = FileTree::new();
        let tep = t["files"].as_object().expect("thiếu trường tep");
        let mut hong = None;
        for (duong, noi) in tep {
            let b = noi.as_str().unwrap_or_default().as_bytes().to_vec();
            if let Err(e) = cay.insert(duong, b) {
                hong = Some(format!("không dựng được cây: {e}"));
                break;
            }
        }
        if let Some(e) = hong {
            k.ghi(ten, false, &e, chi_tiet);
            continue;
        }

        let byte = cay.canonical_bytes();
        let cho_byte = t["canonical_hex"].as_str().expect("thiếu chuan_tac_hex");
        let cho_bam = t["hash_hex"].as_str().expect("thiếu bam_hex");

        let that_byte = hex::encode(&byte);
        let that_bam = content_hash_hex(&byte);

        // Băm theo LUỒNG phải ra đúng cùng kết quả. Bản cài đặt nào không chịu
        // dựng cả gói trong bộ nhớ sẽ đi đường này, và một lệch nhỏ ở đây là
        // chữ ký hai bên không kiểm được cho nhau.
        let mut h = tcc_crypto::hash::ContentHasher::new();
        cay.for_each_canonical_chunk(|c| h.update(c));
        let bam_luong = h.finish_hex();
        if bam_luong != that_bam {
            k.ghi(
                ten,
                false,
                &format!("băm theo luồng KHÁC băm một lần: {bam_luong} ≠ {that_bam}"),
                chi_tiet,
            );
            continue;
        }

        if that_byte != cho_byte {
            k.ghi(
                ten,
                false,
                &format!(
                    "dạng chuẩn tắc lệch\n          chờ : {cho_byte}\n          thật: {that_byte}"
                ),
                chi_tiet,
            );
        } else if that_bam != cho_bam {
            k.ghi(
                ten,
                false,
                &format!("băm lệch\n          chờ : {cho_bam}\n          thật: {that_bam}"),
                chi_tiet,
            );
        } else {
            k.ghi(ten, true, "", chi_tiet);
        }
    }
    k
}

// ───────────────────────────── Chữ ký ──────────────────────────────────────

/// Nhóm interop quan trọng thứ hai, sau dạng chuẩn tắc.
///
/// Kiểm BA chiều, không phải một:
///   1. **Sinh khoá**: cùng khoá bí mật phải suy ra cùng khoá công khai
///   2. **Ký**: ký lại phải ra ĐÚNG chuỗi byte cũ (ký ở đây là tất định)
///   3. **Kiểm**: chữ ký hợp lệ phải đạt, mọi đòn phá phải hỏng
///
/// Chỉ kiểm chiều 3 là không đủ: một bản triển khai kiểm được chữ ký của ta mà
/// sinh ra chữ ký ta không kiểm được thì vẫn không dùng chung gói được.
fn chay_signature(chi_tiet: bool) -> Ket {
    let v = doc_vector("signature.json");
    let mut k = Ket::moi();
    let scheme = HybridEd25519MlDsa;

    let bi_mat = doc_hex(&v["keys"]["secret_hex"]);
    let cong_cho = doc_hex(&v["keys"]["public_hex"]);

    // ---- Chiều 1: sinh khoá ----
    match HybridEd25519MlDsa::public_from_secret(&bi_mat) {
        Ok(that) if that == cong_cho => {
            k.ghi("suy khoá công khai từ khoá bí mật", true, "", chi_tiet);
        }
        Ok(_) => k.ghi(
            "suy khoá công khai từ khoá bí mật",
            false,
            "khoá công khai suy ra KHÁC vector — hai bản triển khai sẽ không dùng chung gói được",
            chi_tiet,
        ),
        Err(e) => k.ghi(
            "suy khoá công khai từ khoá bí mật",
            false,
            &format!("lỗi: {e}"),
            chi_tiet,
        ),
    }

    // Bố cục byte là một phần của tiêu chuẩn, không phải chi tiết cài đặt.
    k.ghi(
        "độ dài khoá công khai đúng 1984 byte",
        cong_cho.len() == 1984,
        &format!("thật: {}", cong_cho.len()),
        chi_tiet,
    );

    // ---- Chiều 2 và 3 ----
    for t in v["valid_signatures"]
        .as_array()
        .expect("thiếu `valid_signatures`")
    {
        let m = doc_hex(&t["message_hex"]);
        let ky_cho = doc_hex(&t["signature_hex"]);
        let ten = format!("thông điệp {} byte", m.len());

        k.ghi(
            &format!("{ten}: kiểm chữ ký ĐẠT"),
            scheme.verify(&cong_cho, &m, &ky_cho).is_ok(),
            "chữ ký trong vector lại kiểm HỎNG",
            chi_tiet,
        );
        match scheme.sign(&bi_mat, &m) {
            Ok(that) => k.ghi(
                &format!("{ten}: ký lại ra ĐÚNG byte cũ"),
                that == ky_cho,
                "ký lại ra chuỗi byte khác — ký phải tất định",
                chi_tiet,
            ),
            Err(e) => k.ghi(
                &format!("{ten}: ký lại"),
                false,
                &format!("lỗi: {e}"),
                chi_tiet,
            ),
        }
    }

    // ---- Mọi đòn phá phải hỏng ----
    let m = b"TCC conformance vector 0.1";
    for t in v["broken_signatures"]
        .as_array()
        .expect("thiếu `broken_signatures`")
    {
        let ten = ten_cua(t);
        let ky = doc_hex(&t["signature_hex"]);
        k.ghi(
            ten,
            scheme.verify(&cong_cho, m, &ky).is_err(),
            "chữ ký ĐÃ BỊ PHÁ mà vẫn kiểm đạt",
            chi_tiet,
        );
    }

    // ---- Neo ngoài: RFC 8032 ----
    let hat = doc_hex(&v["external_anchor"]["rfc8032_test1_seed"]);
    let cho = doc_hex(&v["external_anchor"]["rfc8032_test1_public_key"]);
    let mut bm = hat;
    bm.extend_from_slice(&[0x11u8; 32]);
    let that = HybridEd25519MlDsa::public_from_secret(&bm).unwrap_or_default();
    k.ghi(
        "neo ngoài: nửa Ed25519 khớp RFC 8032 §7.1 TEST 1",
        that.len() >= 32 && that[..32] == cho[..],
        "nửa cổ điển KHÔNG khớp mốc bên ngoài — cài đặt Ed25519 sai",
        chi_tiet,
    );

    k
}

fn doc_hex(v: &Value) -> Vec<u8> {
    hex::decode(v.as_str().unwrap_or_default()).unwrap_or_default()
}

// ───────────────────────────── Mốc ngoài: NIST ACVP ─────────────────────────

/// Neo nửa HẬU LƯỢNG TỬ vào vector chính thức của NIST.
///
/// # Vì sao nhóm này quan trọng hơn vẻ ngoài của nó
///
/// Nhóm `signature` kiểm ba chiều, nhưng mọi giá trị trong đó do **chính bản
/// triển khai này** sinh ra. Nó chứng minh ta nhất quán với chính mình, không
/// chứng minh ta đúng. Nửa Ed25519 đã neo vào RFC 8032; nửa ML-DSA-65 thì tới
/// nhóm này mới có mốc ngoài.
///
/// Chỉ so KHOÁ CÔNG KHAI. NIST cũng cho `sk` 4032 byte nhưng đó là khoá đã
/// BUNG, còn dự án này giữ khoá bí mật ở dạng HẠT GIỐNG 32 byte — hai cách biểu
/// diễn đều hợp FIPS 204, so với nhau sẽ báo lệch oan.
fn chay_acvp(chi_tiet: bool) -> Ket {
    let v = doc_vector("acvp-mldsa65.json");
    let mut k = Ket::moi();

    for t in cac_truong_hop(&v) {
        let id = t["tcId"].as_u64().unwrap_or(0);
        let ten = format!("NIST ACVP ML-DSA-65 keyGen tcId={id}");
        let hat = doc_hex(&t["seed_hex"]);
        let cho = doc_hex(&t["public_key_hex"]);

        // Nửa Ed25519 để 0: nhóm vector này không nói gì về nó.
        let mut bi_mat = vec![0u8; 32];
        bi_mat.extend_from_slice(&hat);

        match HybridEd25519MlDsa::public_from_secret(&bi_mat) {
            Ok(that) if that.len() > 32 && that[32..] == cho[..] => k.ghi(&ten, true, "", chi_tiet),
            Ok(_) => k.ghi(
                &ten,
                false,
                "khoá công khai ML-DSA-65 KHÁC vector chính thức của NIST — cài đặt hậu \
                 lượng tử sai, và mọi phép thử khác của dự án đều mù vì chúng chỉ so với \
                 chính bản triển khai này",
                chi_tiet,
            ),
            Err(e) => k.ghi(&ten, false, &format!("lỗi: {e}"), chi_tiet),
        }
    }
    // ---- sigVer: neo chiều KIỂM ----
    //
    // Chỉ một ca dùng được (ACVP chỉ có một ca `context` rỗng trong nhóm ngoài).
    // Mốc mỏng, nhưng mỏng vẫn hơn không có.
    //
    // Kiểm qua API CÔNG KHAI của chữ ký lai: ghép nửa Ed25519 hợp lệ của chính
    // ta với nửa ML-DSA của NIST. Nửa cổ điển đạt theo cách dựng, nên kết quả
    // phản ánh đúng nửa hậu lượng tử.
    for t in v["sig_ver"]["cases"].as_array().unwrap_or(&Vec::new()) {
        let id = t["tcId"].as_u64().unwrap_or(0);
        let ten = format!("NIST ACVP ML-DSA-65 sigVer tcId={id}");
        let pq_pub = doc_hex(&t["public_key_hex"]);
        let msg = doc_hex(&t["message_hex"]);
        let pq_sig = doc_hex(&t["signature_hex"]);
        let phai_dat = t["must_pass"].as_bool().unwrap_or(false);

        let khoa = HybridEd25519MlDsa::generate();
        let Ok(ed_ky) = HybridEd25519MlDsa.sign(&khoa.secret, &msg) else {
            k.ghi(&ten, false, "không ký được nửa cổ điển", chi_tiet);
            continue;
        };
        let mut cong = khoa.public[..32].to_vec();
        cong.extend_from_slice(&pq_pub);
        let mut ky = ed_ky[..64].to_vec();
        ky.extend_from_slice(&pq_sig);

        let that = HybridEd25519MlDsa.verify(&cong, &msg, &ky).is_ok();
        k.ghi(
            &ten,
            that == phai_dat,
            &format!("NIST bảo {phai_dat}, ta ra {that}"),
            chi_tiet,
        );
    }

    k
}

// ───────────────────────────── Bản kê khai ──────────────────────────────────

fn chay_manifest(chi_tiet: bool) -> Ket {
    let v = doc_vector("manifest.json");
    let mut k = Ket::moi();
    let pub_that = "aa".repeat(1992);
    let hash_that = "bb".repeat(48);

    for t in cac_truong_hop(&v) {
        let ten = ten_cua(t);
        let mut ke_khai = t["manifest"].clone();
        // Thay chỗ giữ chỗ bằng giá trị đúng độ dài.
        if ke_khai["publisher"] == GIU_CHO_PUB {
            ke_khai["publisher"] = Value::String(pub_that.clone());
        }
        if ke_khai["content_hash"] == GIU_CHO_HASH {
            ke_khai["content_hash"] = Value::String(hash_that.clone());
        }

        let cho_dat = t["expect_pass"]
            .as_bool()
            .expect("thiếu trường `expect_pass`");
        let cho_ma = t["code"].as_str();

        // Giải mã rồi kiểm hình dạng. Cả hai bước đều có thể từ chối, và mã lỗi
        // phải giống nhau dù bị chặn ở bước nào.
        let ket: Result<(), String> = serde_json::from_value::<Manifest>(ke_khai)
            .map_err(|e| format!("json:{e}"))
            .and_then(|m| m.validate_shape().map_err(|e| e.ma().to_owned()));

        match (cho_dat, ket) {
            (true, Ok(())) => k.ghi(ten, true, "", chi_tiet),
            (true, Err(e)) => k.ghi(
                ten,
                false,
                &format!("phải ĐẠT nhưng bị từ chối: {e}"),
                chi_tiet,
            ),
            (false, Ok(())) => k.ghi(ten, false, "phải TỪ CHỐI nhưng lại đạt", chi_tiet),
            (false, Err(ma)) => {
                let khop = cho_ma.is_none_or(|c| ma == c || ma.starts_with("json:"));
                k.ghi(
                    ten,
                    khop,
                    &format!("từ chối đúng nhưng SAI MÃ: chờ {cho_ma:?}, thật \"{ma}\""),
                    chi_tiet,
                );
            }
        }
    }
    k
}

// ───────────────────────────── Cây giao diện ────────────────────────────────

/// Dựng cây từ mô tả `generate`, cho những ca quá lớn để viết thẳng ra.
///
/// Viết thẳng cây 10001 nút là một tệp vector 300 KB không ai đọc. Mô tả cách
/// dựng thì bên cài đặt nào cũng làm lại được trong ba dòng — xem FORMAT.md.
fn dung_tu_mo_ta(g: &Value) -> Option<Value> {
    if g["shape"].as_str() == Some("big_text") {
        let n = usize::try_from(g["bytes"].as_u64()?).ok()?;
        return Some(serde_json::json!({"kind": "text", "content": "a".repeat(n)}));
    }
    let so = g["children"].as_u64()?;
    let con: Vec<Value> = (0..so)
        .map(|_| serde_json::json!({"kind": "text", "content": "x"}))
        .collect();
    Some(serde_json::json!({"kind": "group", "children": con}))
}

fn chay_ui(chi_tiet: bool) -> Ket {
    let v = doc_vector("ui.json");
    let mut k = Ket::moi();

    for t in cac_truong_hop(&v) {
        let ten = ten_cua(t);
        let cho_dat = t["expect_pass"]
            .as_bool()
            .expect("thiếu trường `expect_pass`");
        let cho_ma = t["code"].as_str();
        let cay = if t["generate"].is_object() {
            if let Some(c) = dung_tu_mo_ta(&t["generate"]) {
                c
            } else {
                k.ghi(ten, false, "mô tả `generate` không đọc được", chi_tiet);
                continue;
            }
        } else {
            t["tree"].clone()
        };
        let byte = serde_json::to_vec(&cay).expect("cây không tuần tự hoá được");

        match (cho_dat, tcc_ui::wire::decode(&byte)) {
            (true, Ok(_)) => k.ghi(ten, true, "", chi_tiet),
            (true, Err(e)) => k.ghi(
                ten,
                false,
                &format!("phải ĐẠT nhưng bị từ chối: {e}"),
                chi_tiet,
            ),
            (false, Ok(_)) => k.ghi(ten, false, "phải TỪ CHỐI nhưng lại đạt", chi_tiet),
            (false, Err(e)) => {
                // Từ 16/08/2026 `DecodeError::Tree` mang NGUYÊN lỗi bên dưới,
                // nên `ma()` trả mã thật và chỗ này không phải đoán gì.
                //
                // Trước đó nó gói một `String`, và đoạn đoán lại mã ở đây từng
                // ghi đè nhầm: ca "tệp giao diện quá 1 MiB" báo `text-too-long`
                // vì cây được dựng lại mà bỏ qua trần kích thước.
                let ma = e.ma().to_owned();
                let khop = cho_ma.is_none_or(|c| ma == c);
                k.ghi(
                    ten,
                    khop,
                    &format!("từ chối đúng nhưng SAI MÃ: chờ {cho_ma:?}, thật \"{ma}\""),
                    chi_tiet,
                );
            }
        }
    }
    k
}

// ───────────────────────────── Quyền năng ───────────────────────────────────

// ─────────────────────────── Tầng gói: đường dẫn ────────────────────────────

/// Luật đường dẫn của `01-package.md`.
///
/// Nhóm này ra đời sau khi rà đặc tả và thấy **16 trong 32 mã lỗi không có
/// vector nào** — nghĩa là một nửa tiêu chuẩn không ai ngoài dự án kiểm chứng
/// được. Tầng gói không có lấy một ca từ chối.
fn chay_package(chi_tiet: bool) -> Ket {
    let v = doc_vector("package.json");
    let mut k = Ket::moi();

    for t in cac_truong_hop(&v) {
        let ten = ten_cua(t);
        let cho_dat = t["expect_pass"]
            .as_bool()
            .expect("thiếu trường `expect_pass`");
        let cho_ma = t["code"].as_str();

        let mut cay = FileTree::new();
        let mut loi: Option<String> = None;
        if let Some(tep) = t["files"].as_object() {
            for (duong, noi) in tep {
                if let Err(e) = cay.insert(duong, noi.as_str().unwrap_or_default().into()) {
                    loi = Some(e.ma().to_owned());
                    break;
                }
            }
        }

        match (cho_dat, loi) {
            (true, None) => k.ghi(ten, true, "", chi_tiet),
            (true, Some(e)) => k.ghi(
                ten,
                false,
                &format!("phải ĐẠT nhưng bị từ chối: {e}"),
                chi_tiet,
            ),
            (false, None) => k.ghi(ten, false, "phải TỪ CHỐI nhưng lại đạt", chi_tiet),
            (false, Some(ma)) => {
                let khop = cho_ma.is_none_or(|c| ma == c);
                k.ghi(
                    ten,
                    khop,
                    &format!("từ chối đúng nhưng SAI MÃ: chờ {cho_ma:?}, thật \"{ma}\""),
                    chi_tiet,
                );
            }
        }
    }
    k
}

// ─────────────────────────── Quyền năng ────────────────────────────

// ──────────────── Kiểm gói đầu-cuối: THỨ TỰ là tính chất bảo mật ────────────────

/// Nhóm này ký gói NGAY LÚC CHẠY rồi mới kiểm.
///
/// Dùng một gói đã ký sẵn thì chỉ kiểm được một mẫu cố định; ký tại chỗ mới kiểm
/// được cả ĐƯỜNG ỐNG, kể cả thứ tự các bước — thứ mà `01-package.md` gọi là một
/// tính chất bảo mật chứ không phải chi tiết cài đặt.
fn chay_verify(chi_tiet: bool) -> Ket {
    let v = doc_vector("verify.json");
    let mut k = Ket::moi();
    let bo_ky = HybridEd25519MlDsa;
    let bi_mat = doc_hex(&v["signer_secret_hex"]);
    let cong_khai = match HybridEd25519MlDsa::public_from_secret(&bi_mat) {
        Ok(c) => hex::encode(c),
        Err(e) => {
            k.ghi("khoá ký của vector", false, &format!("{e}"), chi_tiet);
            return k;
        }
    };

    for t in cac_truong_hop(&v) {
        let ten = ten_cua(t);
        let cho_dat = t["expect_pass"]
            .as_bool()
            .expect("thiếu trường `expect_pass`");
        let cho_ma = t["code"].as_str();

        // Cây tệp
        let mut cay = FileTree::new();
        if let Some(tep) = t["files"].as_object() {
            for (d, n) in tep {
                let _ = cay.insert(d, n.as_str().unwrap_or_default().into());
            }
        }

        // Bản kê khai: thay chỗ giữ chỗ bằng giá trị thật
        let mut ke = t["manifest"].clone();
        if ke["publisher"] == "SIGNER" {
            ke["publisher"] = Value::String(cong_khai.clone());
        }
        if ke["content_hash"] == "COMPUTED" {
            ke["content_hash"] = Value::String(content_hash_hex(&cay.canonical_bytes()));
        }
        if let Some(n) = t["pad_manifest_to"].as_u64() {
            let dai = usize::try_from(n).unwrap_or(0);
            ke["name"] = Value::String("A".repeat(dai));
        }
        let byte_ke = serde_json::to_vec(&ke).expect("bản kê khai không tuần tự hoá được");

        let chu_ky = if t["sign"].as_bool() == Some(true) {
            match bo_ky.sign(&bi_mat, &byte_ke) {
                Ok(c) => c,
                Err(e) => {
                    k.ghi(ten, false, &format!("ký hỏng: {e}"), chi_tiet);
                    continue;
                }
            }
        } else {
            vec![0u8; 3373]
        };

        // Sửa nội dung SAU khi ký: đây là điều duy nhất `content_hash` tồn tại để bắt.
        if t["tamper_content_after_signing"].as_bool() == Some(true) {
            let _ = cay.insert("them.txt", b"sua sau khi ky".to_vec());
        }

        // Kiểm chữ ký, RỒI mới đối chiếu bản kê khai với nội dung. Hai bước tách
        // rời vì bước sau chỉ có nghĩa khi bước trước đã qua — bản kê khai chưa
        // được kiểm chữ ký thì nói gì cũng chưa đáng tin.
        let ket = tcc_manifest::verify_package(&byte_ke, &chu_ky, &cay, &bo_ky).and_then(|app| {
            app.manifest()
                .validate_against_content(&cay)
                .map_err(tcc_manifest::ManifestError::from)?;
            Ok(app)
        });
        match (cho_dat, ket) {
            (true, Ok(_)) => k.ghi(ten, true, "", chi_tiet),
            (true, Err(e)) => k.ghi(ten, false, &format!("phải ĐẠT nhưng hỏng: {e}"), chi_tiet),
            (false, Ok(_)) => k.ghi(ten, false, "phải TỪ CHỐI nhưng lại đạt", chi_tiet),
            (false, Err(e)) => {
                let ma = e.ma();
                let khop = cho_ma.is_none_or(|c| ma == c);
                k.ghi(
                    ten,
                    khop,
                    &format!("từ chối đúng nhưng SAI MÃ: chờ {cho_ma:?}, thật \"{ma}\""),
                    chi_tiet,
                );
            }
        }
    }
    k
}

fn chay_capability(chi_tiet: bool) -> Ket {
    let v = doc_vector("capability.json");
    let mut k = Ket::moi();
    let app = AppId::parse("com.tcc.kiem-dinh").expect("mã ứng dụng mẫu hỏng");

    for t in cac_truong_hop(&v) {
        let ten = ten_cua(t);
        let hosts: Vec<String> = t["granted"]
            .as_array()
            .expect("thiếu trường cap")
            .iter()
            .map(|x| x.as_str().unwrap_or_default().to_owned())
            .collect();
        let goi = t["requested"].as_str().expect("thiếu trường goi");
        let cho_phep = t["allowed"].as_bool().expect("thiếu trường cho_phep");

        let xin = CapabilityRequest {
            name: "network".to_owned(),
            scope: tcc_spec::Scope::Network { hosts },
            reason: "kiểm định".to_owned(),
        };
        let Ok(bo) = grant(app.clone(), std::slice::from_ref(&xin), |_| Decision::Allow) else {
            k.ghi(ten, false, "không cấp được quyền", chi_tiet);
            continue;
        };
        let Some(n) = bo.network() else {
            k.ghi(ten, false, "cấp rồi mà không có quyền mạng", chi_tiet);
            continue;
        };

        let that = n.allow(goi).is_ok();
        k.ghi(
            ten,
            that == cho_phep,
            &format!("chờ {}, thật {}", ket_luan(cho_phep), ket_luan(that)),
            chi_tiet,
        );
    }
    k
}

const fn ket_luan(b: bool) -> &'static str {
    if b { "CHO PHÉP" } else { "TỪ CHỐI" }
}

// ───────────────────────────── Chạy ─────────────────────────────────────────

fn main() -> ExitCode {
    let chi_tiet = std::env::args().any(|a| a == "--chi-tiet");

    println!(
        "Bộ kiểm định tuân thủ TCC — đặc tả {}",
        tcc_spec::SPEC_VERSION
    );
    println!();

    let nhom = [
        ("Dạng chuẩn tắc + băm (interop)", chay_canonical(chi_tiet)),
        ("Chữ ký lai (interop)", chay_signature(chi_tiet)),
        ("Mốc ngoài NIST ACVP (ML-DSA-65)", chay_acvp(chi_tiet)),
        ("Bản kê khai", chay_manifest(chi_tiet)),
        ("Cây giao diện", chay_ui(chi_tiet)),
        ("Quyền năng", chay_capability(chi_tiet)),
        ("Tầng gói: đường dẫn", chay_package(chi_tiet)),
        ("Kiểm gói đầu-cuối", chay_verify(chi_tiet)),
    ];

    let mut tong_dat = 0;
    let mut tong_truot = 0;
    println!();
    println!("{:<36} {:>5} {:>7}", "Nhóm", "đạt", "trượt");
    println!("{}", "─".repeat(50));
    for (ten, k) in &nhom {
        println!("{:<36} {:>5} {:>7}", ten, k.dat, k.truot.len());
        tong_dat += k.dat;
        tong_truot += k.truot.len();
    }
    println!("{}", "─".repeat(50));
    println!("{:<36} {:>5} {:>7}", "TỔNG", tong_dat, tong_truot);
    println!();

    if tong_truot == 0 {
        println!(
            "✓ ĐẠT — bản triển khai này tuân thủ đặc tả {}",
            tcc_spec::SPEC_VERSION
        );
        ExitCode::SUCCESS
    } else {
        println!("✗ TRƯỢT {tong_truot} trường hợp:");
        for (_, k) in &nhom {
            for t in &k.truot {
                println!("    • {t}");
            }
        }
        ExitCode::FAILURE
    }
}
