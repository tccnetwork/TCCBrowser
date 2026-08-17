//! **Đo nền tảng web** — bộ máy trên máy này thật sự có những gì.
//!
//! ```text
//! cargo run -p tcc-shell --features window --example do-nen-tang
//! ```
//!
//! # Vì sao đo chứ không liệt kê
//!
//! Mục 5.1 của kế hoạch là *"công bố TCC Modern Baseline — chính xác những gì
//! hỗ trợ"*. Một danh sách viết tay là một **lời hứa**; nó đúng vào ngày viết
//! và trôi ngay hôm sau, mà không ai biết nó đã trôi.
//!
//! Nên nền tảng ở đây là thứ **đo được**: nạp một tài liệu vào bộ máy THẬT, hỏi
//! nó có gì, và in ra bảng. Ba nền chạy ba bộ máy khác nhau — WKWebView,
//! WebKitGTK, WebView2 — nên nền tảng công bố được phải là **phần giao** của cả
//! ba, không phải phần hợp.
//!
//! # Cái này KHÔNG phải tầng 2
//!
//! Tầng 2 là mở trang web ngoài đời, và nó vẫn 0 dòng. Đây chỉ là phép đo trả
//! lời câu **"nếu làm tầng 2 thì đứng được trên cái gì"**.

use std::{process::ExitCode, time::Duration};

use tcc_render_webview::window;

/// Mỗi mục: tên, và một biểu thức JavaScript trả về `true`/`false`.
///
/// Chọn theo thứ **thật sự khác nhau giữa các bộ máy** hoặc **thật sự cần cho
/// tiếng Việt**, không chọn theo thứ nghe kêu.
const MUC: &[(&str, &str)] = &[
    // ── Chữ và tiếng Việt: nhóm quan trọng nhất của dự án này ──
    ("normalize NFC", "'e\\u0301'.normalize('NFC') === 'é'"),
    (
        "Intl.Collator('vi')",
        "typeof Intl !== 'undefined' && new Intl.Collator('vi').compare('à','b') < 0",
    ),
    (
        "Intl.Segmenter",
        "typeof Intl !== 'undefined' && typeof Intl.Segmenter === 'function'",
    ),
    (
        "font-variation-settings",
        "CSS.supports('font-variation-settings','\\'wght\\' 400')",
    ),
    // ── Bố cục ──
    ("CSS grid", "CSS.supports('display','grid')"),
    ("CSS flexbox gap", "CSS.supports('gap','1px')"),
    ("CSS custom properties", "CSS.supports('--a','1')"),
    ("CSS :has()", "CSS.supports('selector(:has(*))')"),
    (
        "container queries",
        "CSS.supports('container-type','inline-size')",
    ),
    // ── Ngôn ngữ ──
    (
        "optional chaining",
        "(() => { try { return eval('({a:1})?.a === 1'); } catch (e) { return false; } })()",
    ),
    ("BigInt", "typeof BigInt === 'function'"),
    ("structuredClone", "typeof structuredClone === 'function'"),
    // ── Nền tảng ──
    ("fetch", "typeof fetch === 'function'"),
    ("WebAssembly", "typeof WebAssembly === 'object'"),
    (
        "IntersectionObserver",
        "typeof IntersectionObserver === 'function'",
    ),
    ("ResizeObserver", "typeof ResizeObserver === 'function'"),
    (
        "crypto.subtle",
        "typeof crypto !== 'undefined' && typeof crypto.subtle === 'object'",
    ),
    // ── Thứ ta CỐ Ý không muốn có trong tầng 2 ──
    //
    // Không phải để khoe: nếu chúng CÓ thì tầng 2 phải tắt chúng đi, và biết
    // trước là phải tắt cái gì.
    ("localStorage", "typeof localStorage === 'object'"),
    ("Notification", "typeof Notification === 'function'"),
    (
        "navigator.geolocation",
        "typeof navigator.geolocation === 'object'",
    ),
];

fn main() -> ExitCode {
    let mut js = String::from("var ra = {};\n");
    for (ten, bieu_thuc) in MUC {
        // Mỗi mục bọc `try`: một bộ máy thiếu hẳn một API làm cả kịch bản chết,
        // và khi ấy ta mất luôn kết quả của những mục sau nó.
        use std::fmt::Write as _;
        let _ = writeln!(
            js,
            "try {{ ra[{ten}] = !!({bieu_thuc}); }} catch (e) {{ ra[{ten}] = false; }}",
            ten = serde_json::Value::from(*ten),
        );
    }
    js.push_str("window.ipc.postMessage(JSON.stringify(ra));\n");

    let tai_lieu = "<!doctype html><meta charset=utf-8><title>đo nền tảng</title><p>đang đo…";
    let kich_ban =
        format!("document.addEventListener('DOMContentLoaded', function () {{\n{js}}});");

    println!("Đo bộ máy web trên nền này…\n");
    let ra = match window::probe_document(tai_lieu, &kich_ban, Duration::from_secs(20)) {
        Ok(Some(s)) => s,
        Ok(None) => {
            eprintln!("✗ bộ máy không trả lời trong 20 giây");
            return ExitCode::FAILURE;
        }
        Err(e) => {
            eprintln!("✗ {e}");
            return ExitCode::FAILURE;
        }
    };

    let Ok(bang) = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&ra) else {
        eprintln!("✗ phản hồi không đọc được: {ra}");
        return ExitCode::FAILURE;
    };

    let mut co = 0;
    for (ten, _) in MUC {
        let dat = bang.get(*ten).and_then(serde_json::Value::as_bool) == Some(true);
        if dat {
            co += 1;
        }
        println!("  {} {ten}", if dat { "✓" } else { "·" });
    }
    println!("\n{co}/{} mục có mặt trên bộ máy này.", MUC.len());
    println!("Nền tảng công bố được là PHẦN GIAO của cả ba nền, không phải bảng này.");
    ExitCode::SUCCESS
}
