//! Kiểu dữ liệu của TIÊU CHUẨN TCC — và không gì khác.
//!
//! VIỆC CỦA CRATE NÀY: định nghĩa hình dạng dữ liệu mà `spec/` mô tả bằng lời,
//! cùng các ràng buộc định dạng do chính tiêu chuẩn đặt ra (dạng mã ứng dụng,
//! độ dài chuỗi hex, tên quyền năng hợp lệ).
//!
//! LUẬT: crate này KHÔNG phụ thuộc crate nào khác trong workspace và KHÔNG chứa
//! mật mã. Nó là lá của cây phụ thuộc — người ngoài muốn tự cài đặt tiêu chuẩn
//! TCC chỉ cần đọc crate này, không phải đọc cả trình duyệt.
//!
//! Kiểm chữ ký nằm ở `tcc-manifest`, không nằm đây.

pub mod tree;
pub use tree::{FileTree, TreeError};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Phiên bản tiêu chuẩn mà crate này cài đặt.
pub const SPEC_VERSION: &str = "0.1";

/// SHA-384 → 48 byte → 96 ký tự hex.
pub const CONTENT_HASH_HEX_LEN: usize = 96;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SpecError {
    #[error("mã ứng dụng \"{0}\" sai định dạng: {1}")]
    BadAppId(String, &'static str),

    #[error("phiên bản tiêu chuẩn \"{0}\" không hỗ trợ — bản này hiểu {SPEC_VERSION}")]
    UnsupportedSpecVersion(String),

    #[error("trường {field} phải là {expected} ký tự hex, nhận {actual}")]
    BadHexLength {
        field: &'static str,
        expected: usize,
        actual: usize,
    },

    #[error("trường {field} không phải hex hợp lệ")]
    NotHex { field: &'static str },

    #[error("quyền năng \"{0}\" không có trong tiêu chuẩn {SPEC_VERSION}")]
    UnknownCapability(String),

    #[error(
        "quyền năng \"{name}\" thiếu lý do — tiêu chuẩn bắt buộc phải giải thích cho người dùng"
    )]
    MissingReason { name: String },

    #[error("quyền năng \"{name}\" khai phạm vi sai kiểu: {why}")]
    BadScope { name: String, why: &'static str },

    #[error("xin quyền \"{0}\" nhiều hơn một lần — mỗi quyền chỉ được khai một mục")]
    DuplicateCapability(String),

    #[error("trường {field} chứa ký tự {why} — chuỗi này hiện ra cho người dùng")]
    UnsafeDisplayString {
        field: &'static str,
        why: &'static str,
    },

    #[error("tên máy chủ \"{0}\" phải viết dạng ASCII (punycode nếu là tên miền quốc tế)")]
    NonAsciiHost(String),

    #[error("điểm vào \"{0}\" không có trong nội dung gói")]
    MissingEntry(String),

    #[error("điểm vào không hợp lệ: {0}")]
    BadEntry(String),

    #[error(
        "mã hành động \"{0}\" không hợp lệ — chỉ chữ thường ASCII, chữ số, dấu gạch ngang và chấm"
    )]
    BadActionId(String),

    #[error("hành động \"{0}\" khai nhiều hơn một lần")]
    DuplicateAction(String),

    #[error(
        "hành động \"{action}\" muốn gọi \"{host}\" nhưng bản kê khai KHÔNG xin quyền mạng tới \
         máy chủ đó — nút này bấm vào sẽ không bao giờ chạy được"
    )]
    ActionHostNotGranted { action: String, host: String },
}

impl SpecError {
    /// Mã lỗi ỔN ĐỊNH, thuộc về TIÊU CHUẨN.
    ///
    /// Thông báo lỗi là văn xuôi tiếng Việt cho người đọc; nó được phép sửa cho
    /// dễ hiểu hơn bất cứ lúc nào. Mã này thì KHÔNG — bộ kiểm định tuân thủ và
    /// mọi bản triển khai bằng ngôn ngữ khác so khớp bằng nó. **Đổi một mã là
    /// đổi tiêu chuẩn**, phải tăng phiên bản đặc tả.
    #[must_use]
    pub const fn ma(&self) -> &'static str {
        match self {
            Self::BadAppId { .. } => "bad-app-id",
            Self::UnsupportedSpecVersion { .. } => "unsupported-spec-version",
            Self::BadHexLength { .. } => "bad-hex-length",
            Self::NotHex { .. } => "not-hex",
            Self::UnknownCapability { .. } => "unknown-capability",
            Self::MissingReason { .. } => "missing-reason",
            Self::BadScope { .. } => "bad-scope",
            Self::DuplicateCapability { .. } => "duplicate-capability",
            Self::UnsafeDisplayString { .. } => "unsafe-display-string",
            Self::NonAsciiHost { .. } => "non-ascii-host",
            Self::MissingEntry { .. } => "missing-entry",
            Self::BadEntry { .. } => "bad-entry",
            Self::BadActionId { .. } => "bad-action-id",
            Self::DuplicateAction { .. } => "duplicate-action",
            Self::ActionHostNotGranted { .. } => "action-host-not-granted",
        }
    }
}

/// Số dấu kết hợp tối đa cho phép chồng lên MỘT chữ cái.
///
/// # ⚠️ LỖ L10 — chồng dấu che mất cảnh báo (13/08/2026)
///
/// Dấu kết hợp không có giới hạn tự nhiên: nhồi 500 dấu sắc lên một chữ thì bộ
/// dựng vẽ ra một vệt dọc trùm lên phần màn hình bên trên. Trong hộp thoại hỏi
/// quyền, phần bên trên chính là **câu cảnh báo danh tính** — cái người dùng
/// phải đọc trước khi bấm.
///
/// Không cấm hẳn dấu kết hợp được: tiếng Việt sống bằng nó. Nên phải đặt TRẦN.
///
/// # Chọn số 8 thế nào
///
/// | | Số dấu tối đa trên một chữ |
/// |---|---|
/// | Tiếng Việt (`ỡ` = o + móc + ngã) | 2 |
/// | Thái, Devanagari — cụm nặng nhất | ~4–6 |
/// | **Trần ở đây** | **8** |
/// | UAX #15 cho trao đổi dữ liệu | 30 |
///
/// UAX #15 nới tới 30 vì nó lo việc TRAO ĐỔI dữ liệu; ta lo việc HIỂN THỊ trên
/// một màn hình quyết định bảo mật, nên chặt hơn. 8 vẫn cao hơn mọi ngôn ngữ
/// thật, mà 8 dấu chồng thì chưa đủ che một dòng chữ.
///
/// Đây là một đánh đổi có thể sai với một chữ viết tôi chưa biết. Nếu có, sửa
/// con số này chứ đừng gỡ phép kiểm.
pub const MAX_COMBINING_MARKS: usize = 8;

/// Chuỗi này hiện ra ở dạng nào — quyết định `\n` có được phép không.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextKind {
    /// Nhãn MỘT DÒNG: nút bấm, tiêu đề, tên ứng dụng, mục hộp thoại hỏi quyền.
    /// Xuống dòng ở đây làm vỡ bố cục một dòng, nên cấm.
    Label,
    /// Đoạn văn nhiều dòng: cho `\n`, vẫn cấm mọi thứ nguy hiểm còn lại.
    /// KHÔNG cho `\r` và `\t` — canh chữ là việc của bộ dựng, không phải của
    /// ứng dụng, và `\r` một mình có thể xoá dòng trên vài thiết bị đầu cuối.
    Paragraph,
}

/// Ký tự cấm trong mọi chuỗi HIỆN RA CHO NGƯỜI DÙNG.
///
/// Đây không phải chuyện thẩm mỹ mà là chống giả mạo giao diện. Ký tự đảo chiều
/// chữ (U+202E) làm "app-evil.exe" hiển thị thành "app-exe.live"; ký tự rộng
/// bằng không làm hai tên khác nhau trông y hệt; ký tự điều khiển làm vỡ bố cục
/// hộp thoại hỏi quyền — đúng cái hộp mà người dùng phải đọc để quyết định.
///
/// Hàm này công khai để `tcc-ui` dùng LẠI, không chép lại. Nhãn nút bấm cần đúng
/// phép kiểm này: một nút ghi "Huỷ" mà nhét U+202E vào có thể hiện ra thành
/// "Đồng ý", và người dùng bấm cái họ đọc chứ không bấm cái mã nói.
///
/// # Errors
/// Chuỗi chứa ký tự điều khiển, đảo chiều chữ, rộng bằng không, hoặc rỗng.
pub fn check_display_text(
    field: &'static str,
    value: &str,
    kind: TextKind,
) -> Result<(), SpecError> {
    let text = |why| Err(SpecError::UnsafeDisplayString { field, why });
    let mut lien_tiep = 0usize;
    for c in value.chars() {
        // Đếm dấu kết hợp LIÊN TIẾP. Gặp chữ cái là đếm lại từ đầu — ta chặn
        // chồng dấu lên MỘT chữ, không chặn cả câu nhiều dấu.
        if matches!(
            unicode_general_category::get_general_category(c),
            unicode_general_category::GeneralCategory::NonspacingMark
                | unicode_general_category::GeneralCategory::SpacingMark
                | unicode_general_category::GeneralCategory::EnclosingMark
        ) {
            lien_tiep += 1;
            if lien_tiep > MAX_COMBINING_MARKS {
                return text("quá nhiều dấu chồng lên một chữ");
            }
        } else {
            lien_tiep = 0;
        }

        // Đoạn văn được xuống dòng; nhãn thì không. Nhánh có điều kiện phải đứng
        // trước nhánh cấm bên dưới, nếu không `\n` bị chặn trước khi xét kiểu.
        if c == '\n' && kind == TextKind::Paragraph {
            continue;
        }
        // THỨ TỰ QUAN TRỌNG: nhánh cụ thể phải đứng TRƯỚC nhánh dải rộng.
        // Bản đầu tôi viết ngược lại, và `\r` (0x0D) rơi vào dải điều khiển nên
        // nhánh "xuống dòng hoặc tab" không bao giờ chạy tới. Kiểm thử không lộ
        // ra vì tôi chỉ thử `\n` và `\u{0}`. Clippy bắt được (unreachable pattern).
        match c {
            // Xuống dòng và tab: vỡ bố cục hộp thoại một dòng
            '\n' | '\r' | '\t' => return text("xuống dòng hoặc tab"),
            // Điều khiển C0/C1 còn lại
            '\u{0}'..='\u{1f}' | '\u{7f}'..='\u{9f}' => {
                return text("điều khiển");
            }
            // Đảo chiều chữ hai chiều — vũ khí giả mạo tên tệp kinh điển
            '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}' | '\u{200e}' | '\u{200f}' => {
                return text("đảo chiều chữ");
            }
            // Rộng bằng không: hai chuỗi khác nhau trông giống hệt
            '\u{200b}'..='\u{200d}' | '\u{feff}' | '\u{2060}' => return text("rộng bằng không"),
            _ => {}
        }
    }
    if value.trim().is_empty() {
        return text("rỗng hoặc toàn khoảng trắng");
    }
    Ok(())
}

/// Kiểm một nhãn MỘT DÒNG. Đây là mức chặt nhất và là mặc định trong bản kê khai
/// — mọi trường của bản kê khai đều hiện ra ở dạng một dòng.
fn check_display_safe(field: &'static str, value: &str) -> Result<(), SpecError> {
    check_display_text(field, value, TextKind::Label)
}

/// Mã một hành động: nối một nút bấm trên màn hình với một hành vi khai trong
/// bản kê khai.
///
/// ⚠️ `#[serde(transparent)]` — nên giải mã KHÔNG đi qua `parse`. Đó là lỗ L8 đã
/// dẫm với `AppId`. Ở đây bù bằng cách `Manifest::validate_shape` kiểm lại mọi
/// mã hành động, và có phép thử chốt.
///
/// Không hiện ra cho người dùng, nên không cần kiểm giả mạo hiển thị. Nhưng vẫn
/// buộc ASCII hẹp: mã hành động đi vào nhật ký và vào biên giới giữa bộ dựng và
/// ứng dụng, mà chuỗi tuỳ ý ở biên giới là chỗ sinh lỗi phân tích.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ActionId(String);

impl ActionId {
    /// # Errors
    /// Rỗng, hoặc có ký tự ngoài `a-z0-9-.`.
    pub fn parse(s: &str) -> Result<Self, SpecError> {
        let hop_le = !s.is_empty()
            && s.len() <= 128
            && s.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.');
        if hop_le {
            Ok(Self(s.to_owned()))
        } else {
            Err(SpecError::BadActionId(s.to_owned()))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Kiểm HÌNH DẠNG một tên máy chủ.
///
/// # ⚠️ LỖ L9 — đòn giả mạo userinfo (13/08/2026)
///
/// Trước đây chỗ này chỉ kiểm ASCII, không rỗng, không có ký tự đại diện. Một
/// bản kê khai khai được:
///
/// ```text
/// hosts: ["shop.tcc-coin.com:8080@evil.example"]
/// ```
///
/// Chuỗi đó là ASCII, không rỗng, không có `*` — qua hết. Nhưng khi dựng địa
/// chỉ, `shop.tcc-coin.com:8080` thành phần **userinfo** còn máy chủ thật là
/// `evil.example`. Hộp thoại hỏi quyền hiện nguyên chuỗi, và người đọc lướt
/// thấy "shop.tcc-coin.com".
///
/// Lỗ này chỉ lộ ra khi tôi ngồi hỏi "tên máy chủ đi thẳng vào việc dựng địa chỉ
/// — nó đã được kiểm hình dạng chưa?" trước khi viết máy khách HTTP. Không phép
/// thử nào chạm tới, vì mọi phép thử đều dùng tên máy chủ hợp lệ.
///
/// # Errors
/// Không phải một tên miền hợp lệ.
pub fn check_host(host: &str) -> Result<(), &'static str> {
    if host.is_empty() || host.len() > 253 {
        return Err("tên máy chủ phải dài 1–253 ký tự");
    }
    // Bỏ đúng MỘT dấu chấm cuối (dạng tên miền tuyệt đối), rồi mới xét từng đoạn.
    let body = host.strip_suffix('.').unwrap_or(host);
    if body.is_empty() {
        return Err("tên máy chủ chỉ có dấu chấm");
    }
    for doan in body.split('.') {
        if doan.is_empty() {
            return Err("có đoạn rỗng — hai dấu chấm liền nhau hoặc bắt đầu bằng dấu chấm");
        }
        if doan.len() > 63 {
            return Err("một đoạn dài quá 63 ký tự");
        }
        if doan.starts_with('-') || doan.ends_with('-') {
            return Err("đoạn không được bắt đầu hoặc kết thúc bằng dấu gạch ngang");
        }
        // CHỈ chữ, số, gạch ngang. Đây là chỗ chặn `@`, `:`, `/`, `?`, `#` —
        // mọi ký tự biến một tên máy chủ thành một địa chỉ trỏ đi nơi khác.
        if !doan.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
            return Err(
                "chỉ cho chữ cái, chữ số và dấu gạch ngang (dạng punycode nếu là tên miền quốc tế)",
            );
        }
    }
    Ok(())
}

/// Hành vi của một nút bấm, khai trong BẢN KÊ KHAI.
///
/// # Vì sao ở bản kê khai chứ không ở `ui.json`
///
/// Ba lý do, lý do nào cũng đủ:
///
/// 1. **Chữ ký bao trùm bản kê khai.** Hành vi là thứ nguy hiểm nhất một ứng
///    dụng khai — nó phải nằm trong phạm vi chữ ký, không được sửa sau khi ký.
/// 2. **Hiện cho người dùng được.** Bản kê khai là thứ hộp thoại hỏi quyền đọc.
///    Đặt hành vi ở đây thì về sau hiện được "nút này gọi shop.tcc-coin.com".
/// 3. **Giữ tầng giao diện sạch.** Khai ở `ui.json` nghĩa là `tcc-ui` phải biết
///    tới mạng — và crate đó không được biết gì ngoài giao diện.
///
/// Cây giao diện chỉ mang một `ActionId`. Nối mã đó với hành vi là việc của
/// tiêu chuẩn, không phải của giao diện.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Effect {
    /// Gọi một máy chủ. Máy chủ này **phải** nằm trong quyền mạng đã xin.
    Fetch {
        host: String,
        /// Đường dẫn trên máy chủ, luôn bắt đầu bằng `/`.
        path: String,
    },
}

/// Một hành động: mã + hành vi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    pub id: ActionId,
    pub effect: Effect,
}

/// Mã định danh ứng dụng, kiểu tên miền ngược: `com.tcc.hello`.
///
/// Ràng buộc chặt là CÓ CHỦ ĐÍCH: mã lỏng lẻo mở đường cho mã trông na ná nhau
/// để giả mạo (`com.tcc.vi` với `com.tcc.ví`). Chỉ cho chữ thường ASCII, chữ số
/// và dấu chấm.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AppId(String);

impl AppId {
    /// # Errors
    /// Sai định dạng theo các ràng buộc ghi trong tiêu chuẩn.
    pub fn parse(s: &str) -> Result<Self, SpecError> {
        let text = |why| SpecError::BadAppId(s.to_string(), why);

        if s.is_empty() || s.len() > 128 {
            return Err(text("dài 1–128 ký tự"));
        }
        let doan: Vec<&str> = s.split('.').collect();
        if doan.len() < 2 {
            return Err(text("cần ít nhất hai đoạn, ví dụ com.tcc.hello"));
        }
        for d in &doan {
            if d.is_empty() {
                return Err(text("có đoạn rỗng (hai dấu chấm liền nhau?)"));
            }
            if !d.starts_with(|c: char| c.is_ascii_lowercase()) {
                return Err(text("mỗi đoạn phải bắt đầu bằng chữ thường ASCII"));
            }
            if !d
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            {
                return Err(text("chỉ cho phép a-z, 0-9 và dấu gạch ngang"));
            }
        }
        Ok(Self(s.to_string()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Tên các quyền năng mà tiêu chuẩn 0.1 biết.
///
/// Danh sách này CỐ Ý NGẮN. Thêm quyền năng là mở rộng bề mặt tấn công vĩnh
/// viễn — mỗi cái thêm vào phải kèm một mục trong `spec/` và một phép kiểm trong
/// bộ kiểm định tuân thủ.
pub const KNOWN_CAPABILITIES: &[&str] = &["network", "storage", "wallet"];

/// Phạm vi của một quyền năng. Quyền năng KHÔNG có phạm vi là quyền năng vô hạn,
/// nên tiêu chuẩn không cho phép điều đó.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Scope {
    /// Chỉ được gọi đúng những tên máy chủ này. Không có ký tự đại diện.
    Network { hosts: Vec<String> },
    /// Hạn mức lưu trữ, tính bằng byte.
    Storage { quota_bytes: u64 },
    /// Ví: chỉ đọc địa chỉ, hay được xin chữ ký.
    Wallet { may_request_signature: bool },
}

impl Scope {
    #[must_use]
    pub fn capability_name(&self) -> &'static str {
        match self {
            Self::Network { .. } => "network",
            Self::Storage { .. } => "storage",
            Self::Wallet { .. } => "wallet",
        }
    }
}

/// Một mục xin quyền trong bản kê khai.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityRequest {
    pub name: String,
    pub scope: Scope,
    /// Lý do, hiện NGUYÊN VĂN cho người dùng lúc hỏi quyền.
    ///
    /// Tiêu chuẩn bắt buộc trường này. Ép lập trình viên viết ra lý do là cách rẻ
    /// nhất để người dùng có cái mà cân nhắc — hộp thoại "Ứng dụng xin quyền
    /// mạng" thì không ai đọc, "Để tải danh sách sản phẩm từ shop.tcc-coin.com"
    /// thì có.
    pub reason: String,
}

/// Trần của `quota_bytes`, theo `spec/0.1/04-capabilities.md`.
///
/// 2^53−1 chứ không phải 2^64−1: số trong JSON là `double` ở phần lớn bộ đọc.
pub const MAX_QUOTA_BYTES: u64 = 9_007_199_254_740_991;

impl CapabilityRequest {
    /// # Errors
    /// Tên lạ, thiếu lý do, hoặc phạm vi không khớp với tên quyền.
    pub fn validate(&self) -> Result<(), SpecError> {
        if !KNOWN_CAPABILITIES.contains(&self.name.as_str()) {
            return Err(SpecError::UnknownCapability(self.name.clone()));
        }
        if self.reason.trim().is_empty() {
            return Err(SpecError::MissingReason {
                name: self.name.clone(),
            });
        }
        // Lý do HIỆN NGUYÊN VĂN trong hộp hỏi quyền — đúng chỗ kẻ gian muốn chèn
        // ký tự đảo chiều để câu chữ đọc ra nghĩa khác.
        check_display_safe("reason", &self.reason)?;
        if self.scope.capability_name() != self.name {
            return Err(SpecError::BadScope {
                name: self.name.clone(),
                why: "phạm vi khai một loại, tên khai loại khác",
            });
        }
        // Trần `quota_bytes`: 2^53−1, KHÔNG phải 2^64−1.
        //
        // `04-capabilities.md` §"Scope fields, exactly" ghi rõ `0 ≤ n ≤ 2^53−1`,
        // và ghi cả lý do: số trong JSON là `double` ở phần lớn bộ đọc, nên một
        // giá trị không đi qua nổi bộ đọc mà còn nguyên là một giá trị hai bản
        // cài đặt sẽ bất đồng.
        //
        // ⚠️ Trần ấy có tài liệu, có lập luận, và tới 27/08/2026 KHÔNG được
        // cưỡng chế: kiểu là `u64` nên `2^53` lọt thẳng. Nó vô hình suốt vì
        // KHÔNG vector nào từng chạm `quota_bytes` — thêm vector là lộ ra ngay.
        // Đúng thứ bộ kiểm định tuân thủ sinh ra để bắt.
        if let Scope::Storage { quota_bytes } = &self.scope
            && *quota_bytes > MAX_QUOTA_BYTES
        {
            return Err(SpecError::BadScope {
                name: self.name.clone(),
                why: "hạn mức vượt 2^53−1 — số ấy không qua nổi bộ đọc JSON mà còn nguyên",
            });
        }
        if let Scope::Network { hosts } = &self.scope {
            if hosts.is_empty() {
                return Err(SpecError::BadScope {
                    name: self.name.clone(),
                    why: "danh sách máy chủ rỗng — quyền mạng phải nêu đích danh",
                });
            }
            if hosts.iter().any(|h| h.contains('*')) {
                return Err(SpecError::BadScope {
                    name: self.name.clone(),
                    why: "không cho ký tự đại diện: * biến phạm vi thành vô hạn",
                });
            }
            for h in hosts {
                // Tên miền Unicode có vô số cặp trông y hệt nhau (chữ "a" Latin
                // và "а" Kirin). So sánh chuỗi không phân biệt được, mà người
                // dùng nhìn cũng không. Bắt buộc dạng ASCII/punycode để việc so
                // khớp là so đúng thứ trình phân giải tên miền sẽ dùng.
                if !h.is_ascii() {
                    return Err(SpecError::NonAsciiHost(h.clone()));
                }
                if let Err(why) = check_host(h) {
                    return Err(SpecError::BadScope {
                        name: self.name.clone(),
                        why,
                    });
                }
                if h.trim().is_empty() {
                    return Err(SpecError::BadScope {
                        name: self.name.clone(),
                        why: "có tên máy chủ rỗng",
                    });
                }
            }
        }
        Ok(())
    }
}

/// Bản kê khai ứng dụng TCC.
///
/// # Chữ ký bao trùm cái gì
///
/// Chữ ký ký lên **đúng chuỗi byte của tệp `manifest.json`**, không phải lên cấu
/// trúc sau khi giải mã. Xem `tcc-manifest` để biết vì sao.
///
/// `content_hash` là mắt xích nối bản kê khai với nội dung:
///
/// ```text
/// chữ ký ──ký──▶ byte của manifest.json ──chứa──▶ content_hash ──băm──▶ nội dung
/// ```
///
/// Sửa ở bất cứ mắt nào cũng đứt chuỗi.
/// ## Vì sao `deny_unknown_fields`
///
/// Chữ ký phủ lên **toàn bộ byte** của `manifest.json`. Một trường mà không luật
/// nào của tiêu chuẩn nhìn tới là một kênh mang ý nghĩa NGOÀI tiêu chuẩn: cùng
/// một gói đã ký, bản cài đặt hiểu `x-acme-tu-run_loop` thì làm một đằng, bản không
/// hiểu thì làm một nẻo. Đó đúng là cách thời tiền tố `-webkit-` phá vỡ tính
/// liên thông của web.
///
/// Đóng trường lạ để **thứ được ký == thứ được kiểm**. Muốn thêm trường thì mở
/// phiên bản mới, đó là lý do `spec_version` phải khớp chính xác.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// Phiên bản tiêu chuẩn mà gói này viết theo.
    pub spec_version: String,
    pub id: AppId,
    pub name: String,
    pub version: String,
    /// Khoá công khai của người phát hành, dạng hex.
    pub publisher: String,
    /// Tên bộ ký, ví dụ `hybrid-ed25519-mldsa65-v1`.
    pub scheme: String,
    /// Blake3-384 của dạng chuẩn tắc cây tệp, dạng hex.
    pub content_hash: String,
    /// Tệp trong `content/` mà runtime nạp đầu tiên.
    ///
    /// Bắt buộc phải có, và phải TỒN TẠI trong cây tệp — kiểm ở
    /// `validate_against_content`. Không có trường này thì runtime phải đoán
    /// (`index.html`? `main.wasm`?), và đoán là chỗ sinh ra hành vi khác nhau
    /// giữa các bản cài đặt tiêu chuẩn.
    pub entry: String,
    #[serde(default)]
    pub capabilities: Vec<CapabilityRequest>,

    /// Hành vi của từng nút bấm. Vắng mặt nghĩa là ứng dụng chỉ hiện thông tin,
    /// không làm gì — và đó là mặc định đúng.
    #[serde(default)]
    pub actions: Vec<Action>,
}

impl Manifest {
    /// Hành vi phải NẰM TRONG quyền năng đã xin.
    ///
    /// ⚠️ Đây là phép kiểm quan trọng nhất của hành vi. Không có nó, ứng dụng
    /// khai được một nút gọi `ke-gian.example` trong khi chỉ xin quyền mạng tới
    /// `shop.tcc-coin.com`. Lúc chạy thì quyền năng vẫn chặn — nhưng người dùng
    /// đã bấm, không thấy gì xảy ra, và không ai biết vì sao.
    ///
    /// Bắt ở đây nghĩa là `tcc verify` báo cho người viết ứng dụng, lúc họ còn
    /// ngồi trước máy.
    fn kiem_hanh_vi(&self, a: &Action) -> Result<(), SpecError> {
        match &a.effect {
            Effect::Fetch { host, path } => {
                if !host.is_ascii() {
                    return Err(SpecError::NonAsciiHost(host.clone()));
                }
                if let Err(why) = check_host(host) {
                    return Err(SpecError::BadScope {
                        name: a.id.as_str().to_owned(),
                        why,
                    });
                }
                if !path.starts_with('/') {
                    return Err(SpecError::BadScope {
                        name: a.id.as_str().to_owned(),
                        why: "đường dẫn phải bắt đầu bằng /",
                    });
                }
                let duoc = self.capabilities.iter().any(|c| match &c.scope {
                    Scope::Network { hosts } => hosts
                        .iter()
                        .any(|h| h.eq_ignore_ascii_case(host.trim_end_matches('.'))),
                    _ => false,
                });
                if !duoc {
                    return Err(SpecError::ActionHostNotGranted {
                        action: a.id.as_str().to_owned(),
                        host: host.clone(),
                    });
                }
                Ok(())
            }
        }
    }

    /// Kiểm các ràng buộc ĐỊNH DẠNG. Không kiểm mật mã — đó là việc của
    /// `tcc-manifest`, và tách ra để crate này không cần phụ thuộc mật mã.
    ///
    /// # Errors
    /// Bất kỳ ràng buộc nào trong tiêu chuẩn bị vi phạm.
    pub fn validate_shape(&self) -> Result<(), SpecError> {
        // ⚠️ LỖ L8, do BỘ KIỂM ĐỊNH TUÂN THỦ tìm ra (13/08/2026).
        //
        // `AppId` khai `#[serde(transparent)]`, nên giải mã từ JSON lấy thẳng
        // chuỗi và **không đi qua `AppId::parse`**. Kiểm thử đơn vị không bao
        // giờ chạm tới vì chúng luôn dựng `AppId` bằng `parse`. Hậu quả: gói
        // ship được `id: "hello"` (thiếu đoạn) hoặc `id: "com.TCC.hello"` — mà
        // mã ứng dụng khác hoa thường là HAI danh tính trông y hệt nhau, đúng
        // cái mà `AppId::parse` sinh ra để chặn.
        //
        // Kiểm lại ở đây chứ không viết `Deserialize` riêng: mọi trường hiện ra
        // người dùng (`name`, `version`, `entry`) đều theo lối "giải mã rồi
        // `validate_shape`", và `verify_package` luôn gọi hàm này.
        AppId::parse(self.id.as_str())?;

        if self.spec_version != SPEC_VERSION {
            return Err(SpecError::UnsupportedSpecVersion(self.spec_version.clone()));
        }
        check_hex("content_hash", &self.content_hash, CONTENT_HASH_HEX_LEN)?;
        // Khoá công khai: chỉ kiểm là hex, KHÔNG kiểm độ dài — độ dài phụ thuộc
        // bộ ký, mà bộ ký thì thay được. Kiểm độ dài là việc của tcc-crypto.
        if hex::decode(&self.publisher).is_err() {
            return Err(SpecError::NotHex { field: "publisher" });
        }
        // Tên ứng dụng hiện trong hộp hỏi quyền và trên thanh tiêu đề.
        check_display_safe("name", &self.name)?;
        check_display_safe("version", &self.version)?;
        // Điểm vào là một đường dẫn trong gói → chịu đúng ràng buộc của cây tệp.
        tree::check_path_public(&self.entry).map_err(|e| SpecError::BadEntry(e.to_string()))?;

        // Mã hành động: `#[serde(transparent)]` không gọi `parse` (lỗ L8), nên
        // kiểm lại ở đây.
        let mut da_thay_hd: Vec<&str> = Vec::new();
        for a in &self.actions {
            ActionId::parse(a.id.as_str())?;
            if da_thay_hd.contains(&a.id.as_str()) {
                return Err(SpecError::DuplicateAction(a.id.as_str().to_owned()));
            }
            da_thay_hd.push(a.id.as_str());
            self.kiem_hanh_vi(a)?;
        }

        // LỖ ĐÃ SỬA: xin trùng một quyền hai lần thì bên cấp quyền lấy mục sau
        // đè mục trước. Ứng dụng có thể khai `network: [lanh.com]` cho người
        // duyệt đọc, rồi khai thêm `network: [xau.com]` ở dưới — và cái được cấp
        // là cái thứ hai. Chặn thẳng ở đây thay vì hy vọng bên cấp xử lý đúng.
        let mut da_thay: Vec<&str> = Vec::new();
        for c in &self.capabilities {
            if da_thay.contains(&c.name.as_str()) {
                return Err(SpecError::DuplicateCapability(c.name.clone()));
            }
            da_thay.push(&c.name);
            c.validate()?;
        }
        Ok(())
    }
}

impl Manifest {
    /// Kiểm những ràng buộc CẦN tới nội dung gói.
    ///
    /// Tách khỏi `validate_shape` vì `validate_shape` chỉ nhìn bản kê khai, còn
    /// hàm này cần cây tệp — và bên gọi có thể chưa có cây tệp ở thời điểm đó.
    ///
    /// # Errors
    /// Điểm vào không tồn tại trong gói.
    pub fn validate_against_content(&self, content: &FileTree) -> Result<(), SpecError> {
        if content.get(&self.entry).is_none() {
            return Err(SpecError::MissingEntry(self.entry.clone()));
        }
        Ok(())
    }
}

fn check_hex(field: &'static str, value: &str, expected: usize) -> Result<(), SpecError> {
    if value.len() != expected {
        return Err(SpecError::BadHexLength {
            field,
            expected,
            actual: value.len(),
        });
    }
    if hex::decode(value).is_err() {
        return Err(SpecError::NotHex { field });
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "kiểm thử: hỏng thì phải nổ ngay")]
mod kiem_thu {
    /// **Mọi mã lỗi của `SpecError` được GHIM ở đây.**
    ///
    /// Kiểm đột biến 26/08/2026: `SpecError::ma` trả `""` hoặc `"xyzzy"` mà
    /// `cargo test -p tcc-spec` vẫn xanh. Bộ kiểm định tuân thủ CÓ canh — nó so
    /// `e.ma()` với vector — nhưng đó là một nhị phân khác, ngoài tầm trọng tài.
    ///
    /// Bất đối xứng ngay trong cùng hòm: `TreeError::ma` CÓ phép thử ghim chuỗi
    /// (`tree.rs`), `SpecError::ma` thì không. Hai loại mã lỗi cùng giá trị với
    /// người cài đặt tiêu chuẩn, canh bằng hai mức khác nhau, không vì lý do gì.
    ///
    /// ⚠️ Chú thích trên `ma()` viết: **"Đổi một mã là đổi tiêu chuẩn"**. Bảng
    /// dưới đây là chỗ điều đó trở thành thứ máy chặn được — đổi một chuỗi thì
    /// phép thử đỏ ngay ở `cargo test`, không đợi một nhị phân riêng.
    #[test]
    fn moi_ma_loi_cua_spec_deu_duoc_ghim() {
        use super::SpecError as E;
        let bang: &[(E, &str)] = &[
            (E::BadAppId(String::new(), ""), "bad-app-id"),
            (
                E::UnsupportedSpecVersion(String::new()),
                "unsupported-spec-version",
            ),
            (
                E::BadHexLength {
                    field: "",
                    expected: 0,
                    actual: 0,
                },
                "bad-hex-length",
            ),
            (E::NotHex { field: "" }, "not-hex"),
            (E::UnknownCapability(String::new()), "unknown-capability"),
            (
                E::MissingReason {
                    name: String::new(),
                },
                "missing-reason",
            ),
            (
                E::BadScope {
                    name: String::new(),
                    why: "",
                },
                "bad-scope",
            ),
            (
                E::DuplicateCapability(String::new()),
                "duplicate-capability",
            ),
            (
                E::UnsafeDisplayString { field: "", why: "" },
                "unsafe-display-string",
            ),
            (E::NonAsciiHost(String::new()), "non-ascii-host"),
            (E::MissingEntry(String::new()), "missing-entry"),
            (E::BadEntry(String::new()), "bad-entry"),
            (E::BadActionId(String::new()), "bad-action-id"),
            (E::DuplicateAction(String::new()), "duplicate-action"),
            (
                E::ActionHostNotGranted {
                    action: String::new(),
                    host: String::new(),
                },
                "action-host-not-granted",
            ),
        ];
        for (loi, cho) in bang {
            assert_eq!(loi.ma(), *cho, "mã lỗi lệch — ĐỔI MÃ LÀ ĐỔI TIÊU CHUẨN");
        }
        // Và không mã nào được rỗng hay trùng nhau.
        let mut da_thay = std::collections::BTreeSet::new();
        for (loi, _) in bang {
            let m = loi.ma();
            assert!(!m.is_empty(), "mã rỗng");
            assert!(da_thay.insert(m), "hai biến thể dùng chung mã {m}");
        }
    }

    use super::*;

    #[test]
    fn ma_ung_dung_hop_le() {
        assert!(AppId::parse("com.tcc.hello").is_ok());
        assert!(AppId::parse("vn.thuanan.bao-gia").is_ok());
    }

    #[test]
    fn ma_ung_dung_sai_thi_tu_choi() {
        for xau in [
            "",              // rỗng
            "hello",         // một đoạn
            "com..hello",    // đoạn rỗng
            "Com.Tcc",       // chữ hoa
            "com.tcc.hé",    // ngoài ASCII
            "com.9tcc",      // đoạn bắt đầu bằng số
            "com.tcc hello", // khoảng trắng
        ] {
            assert!(AppId::parse(xau).is_err(), "phải từ chối: {xau:?}");
        }
    }

    fn ke_khai_mau() -> Manifest {
        Manifest {
            spec_version: SPEC_VERSION.to_string(),
            id: AppId::parse("com.tcc.hello").unwrap(),
            name: "Hello TCC".to_string(),
            version: "1.0.0".to_string(),
            publisher: "aa".repeat(32),
            scheme: "hybrid-ed25519-mldsa65-v1".to_string(),
            content_hash: "ab".repeat(48),
            entry: "index.html".to_string(),
            capabilities: vec![],
            actions: vec![],
        }
    }

    #[test]
    fn ke_khai_toi_thieu_thi_dat() {
        assert!(ke_khai_mau().validate_shape().is_ok());
    }

    #[test]
    fn phien_ban_tieu_chuan_la_thi_tu_choi() {
        let mut m = ke_khai_mau();
        m.spec_version = "9.9".to_string();
        assert!(matches!(
            m.validate_shape(),
            Err(SpecError::UnsupportedSpecVersion(_))
        ));
    }

    #[test]
    fn bam_noi_dung_sai_do_dai_thi_tu_choi() {
        let mut m = ke_khai_mau();
        m.content_hash = "ab".repeat(32); // SHA-256 chứ không phải SHA-384
        assert!(matches!(
            m.validate_shape(),
            Err(SpecError::BadHexLength { .. })
        ));
    }

    /// Quyền mạng có ký tự đại diện là quyền mạng vô hạn. Tiêu chuẩn cấm.
    #[test]
    fn quyen_mang_khong_cho_ky_tu_dai_dien() {
        let mut m = ke_khai_mau();
        m.capabilities = vec![CapabilityRequest {
            name: "network".to_string(),
            scope: Scope::Network {
                hosts: vec!["*.tcc-coin.com".to_string()],
            },
            reason: "tải dữ liệu".to_string(),
        }];
        assert!(matches!(
            m.validate_shape(),
            Err(SpecError::BadScope { .. })
        ));
    }

    #[test]
    fn quyen_mang_khong_cho_danh_sach_rong() {
        let mut m = ke_khai_mau();
        m.capabilities = vec![CapabilityRequest {
            name: "network".to_string(),
            scope: Scope::Network { hosts: vec![] },
            reason: "tải dữ liệu".to_string(),
        }];
        assert!(matches!(
            m.validate_shape(),
            Err(SpecError::BadScope { .. })
        ));
    }

    /// Lý do là bắt buộc — đây là thứ người dùng đọc khi quyết định cấp quyền.
    #[test]
    fn thieu_ly_do_thi_tu_choi() {
        let mut m = ke_khai_mau();
        m.capabilities = vec![CapabilityRequest {
            name: "storage".to_string(),
            scope: Scope::Storage {
                quota_bytes: 1_000_000,
            },
            reason: "   ".to_string(),
        }];
        assert!(matches!(
            m.validate_shape(),
            Err(SpecError::MissingReason { .. })
        ));
    }

    /// Khai tên một loại, phạm vi loại khác — dấu hiệu gói bị sửa tay.
    #[test]
    fn ten_va_pham_vi_lech_nhau_thi_tu_choi() {
        let mut m = ke_khai_mau();
        m.capabilities = vec![CapabilityRequest {
            name: "wallet".to_string(),
            scope: Scope::Storage { quota_bytes: 1 },
            reason: "x".to_string(),
        }];
        assert!(matches!(
            m.validate_shape(),
            Err(SpecError::BadScope { .. })
        ));
    }

    #[test]
    fn quyen_nang_la_thi_tu_choi() {
        let mut m = ke_khai_mau();
        m.capabilities = vec![CapabilityRequest {
            name: "filesystem".to_string(),
            scope: Scope::Storage { quota_bytes: 1 },
            reason: "x".to_string(),
        }];
        assert!(matches!(
            m.validate_shape(),
            Err(SpecError::UnknownCapability(_))
        ));
    }

    #[test]
    fn diem_vao_phai_ton_tai_trong_goi() {
        let m = ke_khai_mau(); // entry = "index.html"
        let mut cay = FileTree::new();
        cay.insert("khac.html", b"x".to_vec()).unwrap();
        assert!(matches!(
            m.validate_against_content(&cay),
            Err(SpecError::MissingEntry(_))
        ));

        let mut dung = FileTree::new();
        dung.insert("index.html", b"x".to_vec()).unwrap();
        assert!(m.validate_against_content(&dung).is_ok());
    }

    /// Điểm vào chịu ĐÚNG bộ luật đường dẫn của cây tệp — không có bộ luật thứ hai.
    #[test]
    fn diem_vao_khong_duoc_thoat_ra_ngoai_goi() {
        for xau in ["../ngoai.html", "/etc/passwd", "a\\b.html", ""] {
            let mut m = ke_khai_mau();
            m.entry = xau.to_string();
            assert!(
                m.validate_shape().is_err(),
                "điểm vào phải bị chặn: {xau:?}"
            );
        }
    }

    // ─────────── Phép thử cho các lỗ tìm ra khi tự soi lại (13/08/2026) ───────────

    /// ⚠️ LỖ 1. Xin `network` hai lần: mục đầu vô hại cho người duyệt đọc, mục
    /// sau mới là mục thật được cấp. Bên cấp quyền lấy cái sau đè cái trước.
    #[test]
    fn xin_trung_mot_quyen_hai_lan_thi_tu_choi() {
        let mut m = ke_khai_mau();
        m.capabilities = vec![
            CapabilityRequest {
                name: "network".to_string(),
                scope: Scope::Network {
                    hosts: vec!["lanh.tcc-coin.com".to_string()],
                },
                reason: "tải dữ liệu".to_string(),
            },
            CapabilityRequest {
                name: "network".to_string(),
                scope: Scope::Network {
                    hosts: vec!["xau.example.com".to_string()],
                },
                reason: "tải dữ liệu".to_string(),
            },
        ];
        assert!(
            matches!(m.validate_shape(), Err(SpecError::DuplicateCapability(_))),
            "khai trùng quyền phải bị từ chối, nếu không mục sau lặng lẽ đè mục trước"
        );
    }

    /// ⚠️ LỖ 3. Ký tự đảo chiều chữ trong tên ứng dụng — vũ khí giả mạo kinh
    /// điển. Tên này hiện ngay trong hộp hỏi quyền.
    #[test]
    fn ten_ung_dung_co_ky_tu_dao_chieu_thi_tu_choi() {
        let mut m = ke_khai_mau();
        m.name = "Vi TCC\u{202e}gnp.exe".to_string();
        assert!(matches!(
            m.validate_shape(),
            Err(SpecError::UnsafeDisplayString { field: "name", .. })
        ));
    }

    /// Lý do cũng hiện nguyên văn cho người dùng → cùng mức bảo vệ.
    #[test]
    fn ly_do_co_ky_tu_nguy_hiem_thi_tu_choi() {
        for xau in [
            "Doc du lieu\u{202e}gnohk", // đảo chiều
            "Doc\ndu lieu",             // xuống dòng, vỡ bố cục hộp thoại
            "Doc\u{200b}du lieu",       // rộng bằng không
            "Doc\u{0}du lieu",          // ký tự điều khiển
            "Doc\rdu lieu",             // \r — lỗi phân loại clippy bắt được, đã sửa
            "Doc\tdu lieu",             // tab
        ] {
            let mut m = ke_khai_mau();
            m.capabilities = vec![CapabilityRequest {
                name: "storage".to_string(),
                scope: Scope::Storage { quota_bytes: 1 },
                reason: xau.to_string(),
            }];
            assert!(
                matches!(
                    m.validate_shape(),
                    Err(SpecError::UnsafeDisplayString { .. })
                ),
                "phải từ chối lý do chứa ký tự nguy hiểm: {xau:?}"
            );
        }
    }

    /// ⚠️ LỖ 4. Tên miền Unicode trông y hệt tên miền thật.
    /// Chữ "а" Kirin (U+0430) và "a" Latin nhìn không phân biệt được.
    #[test]
    fn ten_may_chu_ngoai_ascii_thi_tu_choi() {
        let mut m = ke_khai_mau();
        m.capabilities = vec![CapabilityRequest {
            name: "network".to_string(),
            scope: Scope::Network {
                hosts: vec!["sh\u{43e}p.tcc-coin.com".to_string()], // "о" Kirin
            },
            reason: "tải dữ liệu".to_string(),
        }];
        assert!(matches!(
            m.validate_shape(),
            Err(SpecError::NonAsciiHost(_))
        ));
    }

    /// Chuỗi tiếng Việt có dấu là ASCII mở rộng — PHẢI cho qua, đây là ngôn ngữ
    /// chính của người dùng. Chỉ cấm ký tự vô hình và ký tự đảo chiều.
    #[test]
    fn tieng_viet_co_dau_van_dung_duoc() {
        let mut m = ke_khai_mau();
        m.name = "Ví TCC — Thanh toán".to_string();
        m.capabilities = vec![CapabilityRequest {
            name: "storage".to_string(),
            scope: Scope::Storage { quota_bytes: 1 },
            reason: "Lưu bản nháp đơn hàng của bạn".to_string(),
        }];
        assert!(
            m.validate_shape().is_ok(),
            "tiếng Việt có dấu bị chặn nhầm — người dùng chính là người Việt"
        );
    }

    /// Bản kê khai đi qua JSON rồi quay lại phải y nguyên — nếu không, hai bản
    /// cài đặt tiêu chuẩn sẽ hiểu khác nhau.
    #[test]
    fn qua_json_roi_ve_thi_khong_doi() {
        let m = ke_khai_mau();
        let s = serde_json::to_string(&m).unwrap();
        let lai: Manifest = serde_json::from_str(&s).unwrap();
        assert_eq!(m, lai);
    }

    /// ⚠️ LỖ L8 — bộ kiểm định tuân thủ tìm ra, kiểm thử đơn vị thì không.
    ///
    /// Vì sao kiểm thử đơn vị mù: chúng luôn dựng `AppId` bằng `AppId::parse`,
    /// nên không bao giờ đi qua đường giải mã JSON. Còn `#[serde(transparent)]`
    /// thì lấy thẳng chuỗi, không gọi `parse`. Chỉ có bộ kiểm định — vốn nạp
    /// bản kê khai từ JSON như người dùng thật — mới chạm tới.
    #[test]
    fn ma_ung_dung_bay_tu_json_van_bi_chan() {
        for id in [
            "hello",         // thiếu đoạn
            "com.TCC.hello", // chữ hoa: hai danh tính trông y hệt nhau
            "com..hello",    // đoạn rỗng
            "com.tcc.ví",    // ngoài ASCII
            "",              // rỗng
        ] {
            let s = format!(
                r#"{{"spec_version":"0.1","id":"{id}","name":"A","version":"1",
"publisher":"{}","scheme":"hybrid-ed25519-mldsa65-v1","content_hash":"{}",
"entry":"ui.json","capabilities":[]}}"#,
                "aa".repeat(1992),
                "bb".repeat(48)
            );
            let m: Manifest = serde_json::from_str(&s)
                .unwrap_or_else(|e| panic!("\"{id}\" không giải mã được: {e}"));
            assert!(
                matches!(m.validate_shape(), Err(SpecError::BadAppId { .. })),
                "mã ứng dụng \"{id}\" lọt qua đường JSON"
            );
        }
    }

    // ---- Chồng dấu (lỗ L10) ----

    /// ⚠️ Chồng dấu vô hạn lên một chữ vẽ ra một vệt dọc trùm lên phần màn hình
    /// bên trên — mà trong hộp thoại hỏi quyền, phần bên trên là CÂU CẢNH BÁO.
    #[test]
    fn chong_dau_qua_nhieu_thi_tu_choi() {
        for n in [MAX_COMBINING_MARKS + 1, 20, 500] {
            let s = format!("Huỷ{}", "\u{301}".repeat(n));
            assert!(
                check_display_text("x", &s, TextKind::Label).is_err(),
                "{n} dấu chồng lọt qua"
            );
        }
    }

    /// ⚠️ Phép thử ĐỐI TRỌNG: cấm quá tay là hỏng tiếng Việt.
    ///
    /// Đây mới là phép thử khó — chặn Zalgo thì dễ, chặn mà không giết tiếng
    /// Việt mới khó. Chữ nặng nhất của tiếng Việt cần 2 dấu.
    #[test]
    fn tieng_viet_dang_tach_van_qua() {
        for (ten, s) in [
            ("ế = e + mũ + sắc", "Ti\u{65}\u{302}\u{301}ng"),
            ("ệ = e + mũ + nặng", "Vi\u{65}\u{323}\u{302}t"),
            ("ỡ = o + móc + ngã", "n\u{6f}\u{31b}\u{303}"),
            (
                "cả câu dạng tách",
                "Ti\u{65}\u{302}\u{301}ng Vi\u{65}\u{323}\u{302}t c\u{6f}\u{301} d\u{61}\u{301}u",
            ),
        ] {
            assert!(
                check_display_text("x", s, TextKind::Label).is_ok(),
                "{ten} bị chặn oan: {:?}",
                check_display_text("x", s, TextKind::Label)
            );
        }
    }

    /// Trần tính theo dấu LIÊN TIẾP, không theo cả chuỗi: một câu dài toàn chữ
    /// có dấu là chuyện bình thường của tiếng Việt.
    #[test]
    fn cau_dai_nhieu_dau_van_qua() {
        let cau = "Ti\u{65}\u{302}\u{301}ng ".repeat(200);
        assert!(check_display_text("x", &cau, TextKind::Label).is_ok());
    }

    // ---- Hình dạng tên máy chủ (lỗ L9) ----

    /// ⚠️ Đòn giả mạo userinfo: `shop.tcc-coin.com:8080@evil.example`.
    ///
    /// Chuỗi này là ASCII, không rỗng, không có ký tự đại diện — qua hết những
    /// phép kiểm cũ. Nhưng khi dựng địa chỉ, phần trước `@` thành userinfo và
    /// máy chủ THẬT là `evil.example`. Người đọc lướt hộp thoại hỏi quyền thấy
    /// "shop.tcc-coin.com".
    #[test]
    fn ten_may_chu_gia_mao_userinfo_bi_chan() {
        for h in [
            "shop.tcc-coin.com:8080@evil.example",
            "user@evil.example",
            "shop.tcc-coin.com/@evil.example",
            "shop.tcc-coin.com#@evil.example",
            "shop.tcc-coin.com?x=@evil.example",
        ] {
            assert!(check_host(h).is_err(), "\"{h}\" lọt qua");
        }
    }

    #[test]
    fn ten_may_chu_hong_dinh_dang_bi_chan() {
        for h in [
            "",                       // rỗng
            ".",                      // chỉ dấu chấm
            ".tcc-coin.com",          // bắt đầu bằng dấu chấm
            "shop..tcc-coin.com",     // hai chấm liền
            "-shop.tcc-coin.com",     // đoạn bắt đầu bằng gạch ngang
            "shop-.tcc-coin.com",     // đoạn kết thúc bằng gạch ngang
            "shop.tcc-coin.com/../x", // đường dẫn lẫn vào
            "shop.tcc coin.com",      // khoảng trắng
            "127.0.0.1:8080",         // cổng
        ] {
            assert!(check_host(h).is_err(), "\"{h}\" lọt qua");
        }
    }

    /// **Ranh giới độ dài phải ĐÚNG như đặc tả ghi — bao gồm cả hai đầu.**
    ///
    /// `spec/0.1/04-capabilities.md:52` viết "1–253 characters, each label
    /// 1–63"; `05-interface.md:284` viết "1–128 characters". Dải BAO GỒM hai
    /// đầu, nên đúng 253, đúng 63, đúng 128 phải ĐƯỢC NHẬN.
    ///
    /// Vì sao có phép thử này: `cargo-mutants` ngày 25/08/2026 đổi `>` thành
    /// `>=` ở cả ba chỗ và **không phép thử nào đỏ**. Phép thử cũ chỉ thử tên
    /// ngắn hợp lệ và tên hỏng-hình-dạng; không cái nào đứng ở mép. Một bản cài
    /// đặt thứ hai đọc đặc tả sẽ nhận 253, bản này sẽ chối — hai bên bất đồng ở
    /// đúng chỗ tiêu chuẩn nói rõ nhất.
    #[test]
    fn ranh_gioi_do_dai_dung_nhu_dac_ta_ghi() {
        // 63 + 1 + 63 + 1 + 63 + 1 + 61 = 253
        let host_253 = format!("{a}.{a}.{a}.{b}", a = "a".repeat(63), b = "a".repeat(61));
        assert_eq!(host_253.len(), 253);
        assert!(check_host(&host_253).is_ok(), "253 ký tự phải được nhận");
        assert!(
            check_host(&format!("{host_253}a")).is_err(),
            "254 ký tự phải bị chối"
        );

        let nhan_63 = format!("{}.com", "a".repeat(63));
        assert!(check_host(&nhan_63).is_ok(), "nhãn 63 phải được nhận");
        assert!(
            check_host(&format!("{}.com", "a".repeat(64))).is_err(),
            "nhãn 64 phải bị chối"
        );

        let id_128 = format!("com.{}", "a".repeat(124));
        assert_eq!(id_128.len(), 128);
        assert!(
            AppId::parse(&id_128).is_ok(),
            "mã ứng dụng 128 phải được nhận"
        );
        assert!(
            AppId::parse(&format!("com.{}", "a".repeat(125))).is_err(),
            "mã ứng dụng 129 phải bị chối"
        );

        // "cần ít nhất hai đoạn" — nên đúng HAI đoạn là hợp lệ.
        assert!(AppId::parse("com.hello").is_ok(), "hai đoạn phải được nhận");
        assert!(AppId::parse("hello").is_err(), "một đoạn phải bị chối");
    }

    /// **Đúng `MAX_COMBINING_MARKS` dấu là được; hơn một dấu là không.**
    ///
    /// Phép thử cũ chỉ thử `MAX + 1`, `20`, `500` — toàn quá ngưỡng. Đổi `>`
    /// thành `>=` thì mọi phép thử ấy VẪN đỏ đúng như cũ, nên đột biến sống.
    #[test]
    fn dung_nguong_dau_chong_thi_van_qua() {
        let vua_du = format!("a{}", "\u{0301}".repeat(MAX_COMBINING_MARKS));
        assert!(
            check_display_text("x", &vua_du, TextKind::Label).is_ok(),
            "đúng {MAX_COMBINING_MARKS} dấu bị chặn oan"
        );
        let qua = format!("a{}", "\u{0301}".repeat(MAX_COMBINING_MARKS + 1));
        assert!(check_display_text("x", &qua, TextKind::Label).is_err());
    }

    /// **Xuống dòng và tab bị chối VỚI LÝ DO CỦA CHÍNH CHÚNG.**
    ///
    /// Xoá nhánh `'\n' | '\r' | '\t'` thì ba ký tự ấy rơi xuống dải điều
    /// khiển C0 và **vẫn bị chối** — bất biến còn nguyên, nên `cargo-mutants`
    /// báo đột biến ấy sống mà không phép thử nào đỏ. Nhưng chú thích ngay tại
    /// chỗ đó ghi rằng nhánh này ĐÃ TỪNG bị vô hiệu hoá trong im lặng một lần
    /// (thứ tự nhánh sai, `\r` không bao giờ tới nơi). Ghim lý do lại là lớp
    /// phòng thủ thứ hai cho đúng cái đã hỏng một lần rồi.
    #[test]
    fn xuong_dong_va_tab_bi_choi_voi_ly_do_rieng() {
        for c in ['\n', '\r', '\t'] {
            let loi = check_display_text("x", &format!("a{c}b"), TextKind::Label).unwrap_err();
            assert!(
                loi.to_string().contains("xuống dòng hoặc tab"),
                "{c:?} bị chối bằng lý do khác: {loi}"
            );
        }
        // Đoạn văn thì `\n` được phép — nhánh có điều kiện phải đứng trước.
        assert!(check_display_text("x", "a\nb", TextKind::Paragraph).is_ok());
    }

    #[test]
    fn ten_may_chu_hop_le_van_qua() {
        for h in [
            "shop.tcc-coin.com",
            "shop.tcc-coin.com.", // dạng tuyệt đối, một dấu chấm cuối
            "localhost",          // không có dấu chấm — hợp lệ, dùng khi phát triển
            "a.b.c.d.e.f",
            "xn--th-e0a.com", // punycode cho tên miền quốc tế
            "127.0.0.1",
        ] {
            assert!(
                check_host(h).is_ok(),
                "\"{h}\" bị chặn oan: {:?}",
                check_host(h)
            );
        }
    }

    /// Tên máy chủ hỏng phải bị chặn qua CẢ HAI đường: quyền năng và hành vi.
    /// Chặn một đường thì đường kia vẫn dựng được địa chỉ trỏ đi nơi khác.
    #[test]
    fn ten_may_chu_gia_mao_bi_chan_o_ca_quyen_lan_hanh_vi() {
        const AC: &str = "shop.tcc-coin.com:8080@evil.example";

        let qua_quyen = format!(
            r#"[{{"name":"network","scope":{{"kind":"network","hosts":["{AC}"]}},"reason":"x"}}]"#
        );
        let m = ke_khai_hanh_vi(&qua_quyen, "[]").unwrap();
        assert!(matches!(
            m.validate_shape(),
            Err(SpecError::BadScope { .. })
        ));

        let qua_hanh_vi =
            format!(r#"[{{"id":"x","effect":{{"kind":"fetch","host":"{AC}","path":"/a"}}}}]"#);
        let m = ke_khai_hanh_vi(QUYEN_SHOP, &qua_hanh_vi).unwrap();
        assert!(
            m.validate_shape().is_err(),
            "hành vi dùng tên máy chủ giả mạo mà lọt"
        );
    }

    // ---- Hành vi của nút bấm ----

    fn ke_khai_hanh_vi(quyen: &str, hanh_dong: &str) -> Result<Manifest, serde_json::Error> {
        serde_json::from_str(&format!(
            r#"{{"spec_version":"0.1","id":"com.tcc.a","name":"A","version":"1",
"publisher":"{}","scheme":"hybrid-ed25519-mldsa65-v1","content_hash":"{}",
"entry":"ui.json","capabilities":{quyen},"actions":{hanh_dong}}}"#,
            "aa".repeat(1992),
            "bb".repeat(48)
        ))
    }

    const QUYEN_SHOP: &str = r#"[{"name":"network",
        "scope":{"kind":"network","hosts":["shop.tcc-coin.com"]},
        "reason":"tải hàng"}]"#;

    #[test]
    fn hanh_vi_trong_pham_vi_quyen_thi_dat() {
        let m = ke_khai_hanh_vi(
            QUYEN_SHOP,
            r#"[{"id":"tai-hang","effect":{"kind":"fetch","host":"shop.tcc-coin.com","path":"/ds"}}]"#,
        )
        .unwrap();
        assert!(m.validate_shape().is_ok(), "{:?}", m.validate_shape());
    }

    /// ⚠️ Phép thử quan trọng nhất của hành vi.
    ///
    /// Không có nó, ứng dụng khai được một nút gọi máy chủ của kẻ gian trong khi
    /// chỉ xin quyền tới máy chủ lành. Lúc chạy quyền năng vẫn chặn — nhưng
    /// người dùng đã bấm, không thấy gì xảy ra, và không ai biết vì sao.
    #[test]
    fn hanh_vi_goi_may_chu_chua_xin_quyen_thi_tu_choi() {
        let m = ke_khai_hanh_vi(
            QUYEN_SHOP,
            r#"[{"id":"tai-hang","effect":{"kind":"fetch","host":"ke-gian.example","path":"/x"}}]"#,
        )
        .unwrap();
        assert!(matches!(
            m.validate_shape(),
            Err(SpecError::ActionHostNotGranted { .. })
        ));
    }

    /// Khai hành vi mà KHÔNG xin quyền mạng nào cũng phải bị chặn.
    #[test]
    fn hanh_vi_khi_khong_xin_quyen_mang_nao_thi_tu_choi() {
        let m = ke_khai_hanh_vi(
            "[]",
            r#"[{"id":"tai-hang","effect":{"kind":"fetch","host":"shop.tcc-coin.com","path":"/x"}}]"#,
        )
        .unwrap();
        assert!(matches!(
            m.validate_shape(),
            Err(SpecError::ActionHostNotGranted { .. })
        ));
    }

    /// Tên miền con KHÔNG được coi là nằm trong phạm vi — khớp phải chính xác,
    /// đúng như luật của `tcc-capability`. Lệch hai bên là lỗ.
    #[test]
    fn ten_mien_con_khong_duoc_coi_la_trong_pham_vi() {
        let m = ke_khai_hanh_vi(
            QUYEN_SHOP,
            r#"[{"id":"x","effect":{"kind":"fetch","host":"api.shop.tcc-coin.com","path":"/x"}}]"#,
        )
        .unwrap();
        assert!(matches!(
            m.validate_shape(),
            Err(SpecError::ActionHostNotGranted { .. })
        ));
    }

    #[test]
    fn ma_hanh_dong_bay_tu_json_van_bi_chan() {
        let m = ke_khai_hanh_vi(
            QUYEN_SHOP,
            r#"[{"id":"TAI HANG","effect":{"kind":"fetch","host":"shop.tcc-coin.com","path":"/x"}}]"#,
        )
        .unwrap();
        assert!(matches!(m.validate_shape(), Err(SpecError::BadActionId(_))));
    }

    #[test]
    fn khai_trung_mot_hanh_dong_thi_tu_choi() {
        let hai = r#"[
            {"id":"x","effect":{"kind":"fetch","host":"shop.tcc-coin.com","path":"/a"}},
            {"id":"x","effect":{"kind":"fetch","host":"shop.tcc-coin.com","path":"/b"}}]"#;
        let m = ke_khai_hanh_vi(QUYEN_SHOP, hai).unwrap();
        assert!(matches!(
            m.validate_shape(),
            Err(SpecError::DuplicateAction(_))
        ));
    }

    #[test]
    fn khong_khai_hanh_vi_nao_van_hop_le() {
        let m = ke_khai_hanh_vi(QUYEN_SHOP, "[]").unwrap();
        assert!(m.validate_shape().is_ok());
        assert!(m.actions.is_empty());
    }

    // ---- Hai mức kiểm chuỗi hiện ra người dùng ----

    #[test]
    fn doan_van_duoc_xuong_dong_nhan_thi_khong() {
        let v = "Dòng một\nDòng hai";
        assert!(check_display_text("x", v, TextKind::Paragraph).is_ok());
        assert!(
            check_display_text("x", v, TextKind::Label).is_err(),
            "nhãn một dòng mà cho xuống dòng thì bố cục hộp thoại vỡ"
        );
    }

    /// Nới `\n` cho đoạn văn KHÔNG được kéo theo `\r` và `\t`.
    #[test]
    fn doan_van_van_cam_cr_va_tab() {
        for (c, ten) in [('\r', "CR"), ('\t', "tab")] {
            assert!(
                check_display_text("x", &format!("a{c}b"), TextKind::Paragraph).is_err(),
                "{ten} lọt vào đoạn văn"
            );
        }
    }

    /// Đây là phép thử giữ cho việc nới lỏng không mở toang: mọi ký tự giả mạo
    /// vẫn phải bị chặn ở CẢ HAI mức.
    #[test]
    fn ky_tu_gia_mao_bi_chan_o_ca_hai_muc() {
        for c in ['\u{202e}', '\u{200b}', '\u{feff}', '\u{0}', '\u{7f}'] {
            for kind in [TextKind::Label, TextKind::Paragraph] {
                assert!(
                    check_display_text("x", &format!("a{c}b"), kind).is_err(),
                    "U+{:04X} lọt qua mức {kind:?}",
                    c as u32
                );
            }
        }
    }
}
