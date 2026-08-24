//! Nhập ví **trong cửa sổ trình duyệt** — ba màn hình nối nhau.
//!
//! Sau cả hai cờ `import-web-wallet` và `os-keystore`.
//!
//! ```text
//! chọn ví  →  nhập PIN  →  xong (và câu về bản cũ)
//! ```
//!
//! # Vì sao ba màn hình chứ không một
//!
//! Màn chọn **không cần PIN**: người dùng thấy mình có mấy ví, nhãn gì, địa chỉ
//! nào, rồi mới quyết định gõ PIN cho cái nào. Bắt gõ PIN trước khi biết mình
//! đang mở cái gì là dạy người ta gõ PIN vào mọi ô hỏi PIN.
//!
//! # Mã PIN sống đúng một lượt, và không đi đâu khác
//!
//! Nó về từ WebView (chỉ màn hình của khung đọc được ô nhập —
//! `KICH_BAN_KHUNG`), dùng một lần để giải mã, rồi hết. Không ghi ra đĩa, không
//! vào nhật ký: `DialogAnswer` che giá trị ngay trong `Debug`.
//!
//! # Cất khoá có thể HỎNG, và đó là đúng
//!
//! `wallet_store::open` từ chối chạy khi không có kho khoá thật của hệ điều
//! hành — xem `docs/vi-thiet-ke.md` §19. Màn hình này KHÔNG được lùi về một chỗ
//! cất yếu hơn để "cho xong": người dùng thấy ví không nhập được thì đi tìm
//! cách khác, còn ví nhập được mà bảo vệ kém hơn họ tưởng thì họ không thấy gì,
//! cho tới lúc mất tiền.

use std::{cell::RefCell, rc::Rc};

use tcc_chain::{
    import::{ImportError, read_export},
    wallet::WalletSecret,
};
use tcc_keystore::SecretKey;
use tcc_render_raster::window::{self as bo_dung_cua_so, Next, Screen};

use crate::{
    import_screen, recovery_screen,
    text::{Language, TextKey, label},
    wallet_store,
};

#[derive(Debug, thiserror::Error)]
pub enum WalletFlowError {
    #[error("không đọc được tệp: {0}")]
    File(String),
    #[error(transparent)]
    Import(#[from] ImportError),
    #[error("cửa sổ hỏng: {0}")]
    Window(String),
    #[error("người dùng đã huỷ")]
    Cancelled,
    #[error("không cất được vào kho khoá: {0}")]
    Store(String),
}

/// Chạy cả ba màn hình **trong một cửa sổ**. Trả về địa chỉ ví đã nhập.
///
/// # Vì sao một cửa sổ chứ không ba
///
/// `tao` chỉ cho dựng một vòng lặp sự kiện mỗi tiến trình; mở cửa sổ thứ hai
/// làm nó hoảng loạn và nhìn từ ngoài chỉ thấy **treo**. Đã trả giá ngày
/// 17/08/2026 — xem `tcc_render_raster::window::open_sequence`.
///
/// # Errors
/// Tệp hỏng, người dùng huỷ, sai PIN, hoặc kho khoá từ chối.
pub fn import_from_file(
    duong_dan: &std::path::Path,
    ngon_ngu: Language,
) -> Result<String, WalletFlowError> {
    let noi_dung = std::fs::read_to_string(duong_dan)
        .map_err(|e| WalletFlowError::File(format!("{}: {e}", duong_dan.display())))?;
    let ds = read_export(&noi_dung)?;

    // Trạng thái của luồng sống trong bao đóng, và **kết quả** đi ra qua đây.
    let ket_qua: Rc<RefCell<Result<String, WalletFlowError>>> =
        Rc::new(RefCell::new(Err(WalletFlowError::Cancelled)));
    let ra = Rc::clone(&ket_qua);
    let nhan_pin = label(TextKey::NhapPinNhan, ngon_ngu).to_owned();
    let mut dia_chi_dang_chon: Option<String> = None;

    let man_dau = man_hinh(
        &import_screen::build_choice(&ds, ngon_ngu).map_err(|e| e.to_string())?,
        label(TextKey::NhapTieuDe, ngon_ngu),
        ngon_ngu,
    );

    bo_dung_cua_so::open_sequence(man_dau, move |t| {
        // Đóng cửa sổ không phải một câu trả lời — `open_sequence` đã chắn, nên
        // tới đây `action` luôn có. Vẫn viết ra: một `unwrap_or("")` ở đây rẻ
        // hơn một `unwrap` sẽ nổ nếu chỗ chắn kia đổi.
        let hanh_dong = t.action.as_deref().unwrap_or_default();

        let pin_go = t.fields.get(&nhan_pin).cloned().unwrap_or_default();
        let buoc = import_step(hanh_dong, dia_chi_dang_chon.as_deref(), &pin_go);

        // ── Màn 1 → 2: đã chọn ví, hỏi PIN ──
        if let ImportStep::HoiPin(dia_chi) = &buoc {
            dia_chi_dang_chon = Some(dia_chi.clone());
            let Ok(cay) = import_screen::build_pin(dia_chi, ngon_ngu) else {
                return Next::Done;
            };
            return Next::Show(Box::new(man_hinh(
                &cay,
                label(TextKey::NhapPinTieuDe, ngon_ngu),
                ngon_ngu,
            )));
        }

        // ── Màn 2 → 3: mở khoá rồi cất ──
        if let ImportStep::MoKhoa { dia_chi, pin } = buoc {
            let Some(vi) = ds.iter().find(|v| v.address == dia_chi) else {
                return Next::Done;
            };
            let da_nhap = match vi.unlock(&pin) {
                Ok(x) => x,
                Err(e) => {
                    let cau = label(import_screen::error_text(&e), ngon_ngu).to_owned();
                    *ra.borrow_mut() = Err(WalletFlowError::Import(e));
                    let Ok(cay) = recovery_screen::build_failure(&cau, ngon_ngu) else {
                        return Next::Done;
                    };
                    return Next::Show(Box::new(man_hinh(
                        &cay,
                        label(TextKey::HongTieuDe, ngon_ngu),
                        ngon_ngu,
                    )));
                }
            };
            // `pin` hết vai trò ở đây. Không ghi, không trả về, không vào nhật ký.
            drop(pin);

            // Cất vào kho khoá THẬT. Hỏng thì hỏng, KHÔNG lùi về chỗ yếu hơn.
            match cat_khoa(&dia_chi, &da_nhap) {
                Ok(()) => {}
                Err(e) => {
                    let cau = e.to_string();
                    *ra.borrow_mut() = Err(e);
                    let Ok(cay) = recovery_screen::build_failure(&cau, ngon_ngu) else {
                        return Next::Done;
                    };
                    return Next::Show(Box::new(man_hinh(
                        &cay,
                        label(TextKey::HongTieuDe, ngon_ngu),
                        ngon_ngu,
                    )));
                }
            }

            *ra.borrow_mut() = Ok(dia_chi);
            let Ok(cay) = import_screen::build_done(&da_nhap, ngon_ngu) else {
                return Next::Done;
            };
            return Next::Show(Box::new(man_hinh(
                &cay,
                label(TextKey::NhapXongTieuDe, ngon_ngu),
                ngon_ngu,
            )));
        }

        Next::Done
    })
    .map_err(WalletFlowError::Window)?;

    Rc::try_unwrap(ket_qua)
        .map_err(|_| WalletFlowError::Cancelled)?
        .into_inner()
}

fn cat_khoa(
    dia_chi: &str,
    da_nhap: &tcc_chain::import::ImportedWallet,
) -> Result<(), WalletFlowError> {
    let mut kho = wallet_store::open().map_err(|e| WalletFlowError::Store(e.to_string()))?;
    let ten = wallet_store::key_name(dia_chi);
    kho.store(&ten, SecretKey::new(da_nhap.secret.expose_seed().to_vec()))
        .map_err(|e| WalletFlowError::Store(e.to_string()))
}

/// Gói một cây thành màn hình cho bộ dựng ra pixel.
///
/// Không có danh sách hành động "được phép" như bên kia: ở đó tài liệu là HTML
/// và một trang chạy được kịch bản nên phải chặn hành động lạ từ ngoài danh
/// sách. Ở đây cây chính là thứ được vẽ, và một mã hành động không có trong cây
/// thì không có nút nào phát ra nó — chặn lại là chặn một đường không tồn tại.
fn man_hinh(cay: &tcc_ui::Node, tieu_de: &str, ngon_ngu: Language) -> Screen {
    Screen {
        tree: cay.clone(),
        title: tieu_de.to_owned(),
        text: crate::text::raster_text(ngon_ngu),
    }
}

impl From<String> for WalletFlowError {
    fn from(e: String) -> Self {
        Self::Window(e)
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay"
)]
mod kiem_thu {

    /// **Luồng NHẬP VÍ: mọi nhánh điều phối, không cần cửa sổ.**
    ///
    /// Ca đáng nhất là *bấm mở khoá khi CHƯA chọn ví nào*. Trong bao đóng cũ nó
    /// trả `Next::Done` — nhìn từ ngoài y hệt "người dùng bấm huỷ", nên một
    /// nhánh sai ở đó im lặng hoàn toàn. Đúng hình dạng lỗi 17/08/2026, tái diễn
    /// ở lần cổng sang `open_sequence`.
    #[test]
    fn moi_nhanh_cua_luong_nhap_vi() {
        const DIA_CHI: &str = "0xabc";
        let chon = import_screen::choose_id(DIA_CHI);

        // Chọn một ví → hỏi PIN cho ĐÚNG ví ấy.
        assert_eq!(
            import_step(&chon, None, ""),
            ImportStep::HoiPin(DIA_CHI.to_owned())
        );

        // Mở khoá khi đã chọn → mang theo đúng địa chỉ và mã PIN.
        assert_eq!(
            import_step(import_screen::ACTION_UNLOCK, Some(DIA_CHI), "1234"),
            ImportStep::MoKhoa {
                dia_chi: DIA_CHI.to_owned(),
                pin: "1234".to_owned()
            }
        );

        // ⚠️ Mở khoá khi CHƯA chọn ví → DỪNG, không đoán lấy ví đầu tiên.
        assert_eq!(
            import_step(import_screen::ACTION_UNLOCK, None, "1234"),
            ImportStep::Dung,
            "bấm mở khoá khi chưa chọn ví lại mở một ví nào đó"
        );

        // Khoảng trắng quanh mã PIN bị CẮT — bộ gõ của hệ điều hành thêm được,
        // và "PIN có dấu cách ở cuối thì sai" là một hành vi, không phải chi tiết.
        assert_eq!(
            import_step(import_screen::ACTION_UNLOCK, Some(DIA_CHI), "  1234\n"),
            ImportStep::MoKhoa {
                dia_chi: DIA_CHI.to_owned(),
                pin: "1234".to_owned()
            }
        );

        // Mã lạ → dừng, không rơi vào nhánh nào khác.
        assert_eq!(
            import_step("ma-bia-ra", Some(DIA_CHI), "1234"),
            ImportStep::Dung
        );
    }

    use super::*;

    /// Tệp hỏng thì báo lỗi **trước khi mở cửa sổ nào**.
    ///
    /// Mở cửa sổ rồi mới báo là bắt người dùng nhìn một màn hình trống rồi mới
    /// biết mình chọn nhầm tệp.
    #[test]
    fn tep_hong_thi_bao_truoc_khi_mo_cua_so() {
        let tam = std::env::temp_dir().join("tcc-vi-hong.json");
        std::fs::write(&tam, "{ không phải json").unwrap();
        let loi = import_from_file(&tam, Language::Vi).unwrap_err();
        assert!(matches!(loi, WalletFlowError::Import(_)), "{loi}");
        let _ = std::fs::remove_file(&tam);
    }

    /// Không có tệp thì cũng thế.
    #[test]
    fn khong_co_tep_thi_bao_ngay() {
        let loi = import_from_file(std::path::Path::new("/khong/ton/tai.json"), Language::Vi)
            .unwrap_err();
        assert!(matches!(loi, WalletFlowError::File(_)), "{loi}");
    }

    /// Nhãn ô PIN dùng để tìm giá trị PHẢI khớp nhãn màn hình vẽ ra.
    ///
    /// Hai chỗ cùng đọc `TextKey::NhapPinNhan`, nhưng nếu ai đó đổi một bên
    /// sang chuỗi viết thẳng thì luồng này lặng lẽ không tìm thấy PIN — và
    /// người dùng thấy "đã huỷ" sau khi vừa gõ đúng mã.
    #[test]
    fn nhan_o_pin_khop_giua_man_hinh_va_luong() {
        for ngon_ngu in [Language::En, Language::Vi] {
            let cay = import_screen::build_pin("0xabc", ngon_ngu).unwrap();
            let s = crate::do_cay::chu(&cay);
            let nhan = label(TextKey::NhapPinNhan, ngon_ngu);
            // Luồng tra ô nhập THEO NHÃN (`t.fields.get(&nhan_pin)`), nên nhãn
            // trên màn hình và nhãn trong luồng phải là một chuỗi. Lệch một chữ
            // là mã PIN người dùng gõ không bao giờ tới nơi, và màn hình vẫn
            // trông bình thường.
            assert!(
                s.contains(&format!("] {nhan}:")),
                "màn hình không mang nhãn {nhan:?} mà luồng đang tìm:\n{s}"
            );
        }
    }
}

/// Bước tiếp theo của luồng NHẬP VÍ — **hàm THUẦN, kiểm được không cần cửa sổ**.
///
/// # Vì sao tách, lần thứ hai
///
/// `phrase_step` ngay dưới đã tách vì đúng lý do này, sau lỗi 17/08/2026. Rồi
/// ngày 23/08 luồng nhập ví được cổng sang `open_sequence`, và **quyết định mới
/// lại nằm trong một bao đóng chạy giữa vòng lặp sự kiện** — không cách nào
/// kiểm, và một nhánh sai ở đó trông y hệt "người dùng bấm huỷ". Bài học cũ,
/// mất lần nữa ở lần viết lại.
///
/// Cắt khoảng trắng của mã PIN nằm TRONG đây chứ không ở chỗ gọi: nó là một
/// quyết định — bộ gõ của hệ điều hành thêm được khoảng trắng, và "PIN có dấu
/// cách ở cuối thì sai" là một hành vi phải kiểm được, không phải một chi tiết.
#[derive(Debug, PartialEq, Eq)]
pub enum ImportStep {
    /// Đã chọn ví — hỏi mã PIN cho địa chỉ này.
    HoiPin(String),
    /// Mở khoá ví đang chọn bằng mã PIN đã cắt khoảng trắng.
    MoKhoa { dia_chi: String, pin: String },
    /// Không nhận ra, hoặc bấm mở khoá khi CHƯA chọn ví nào — dừng.
    Dung,
}

/// Quyết định bước tiếp theo của luồng nhập ví.
#[must_use]
pub fn import_step(hanh_dong: &str, dang_chon: Option<&str>, pin: &str) -> ImportStep {
    if let Some(dia_chi) = import_screen::address_to_import(hanh_dong) {
        return ImportStep::HoiPin(dia_chi.to_owned());
    }
    if hanh_dong == import_screen::ACTION_UNLOCK {
        // Chưa chọn ví mà bấm mở khoá: DỪNG, không đoán lấy ví đầu tiên. Đoán ở
        // đây là mở nhầm ví của người dùng.
        return dang_chon.map_or(ImportStep::Dung, |dia_chi| ImportStep::MoKhoa {
            dia_chi: dia_chi.to_owned(),
            pin: pin.trim().to_owned(),
        });
    }
    ImportStep::Dung
}

/// Bước tiếp theo của luồng cụm từ — **hàm THUẦN, kiểm được không cần cửa sổ**.
///
/// Tách ra vì lỗi ngày 17/08/2026: gõ sai cụm từ thì cửa sổ **đóng luôn** thay
/// vì hiện lại màn nhập kèm câu báo lỗi. Toàn bộ quyết định nằm trong một bao
/// đóng chạy giữa vòng lặp sự kiện, nên không có cách nào kiểm — và một nhánh
/// sai ở đó trông y hệt "người dùng bấm huỷ".
#[derive(Debug, PartialEq, Eq)]
pub enum PhraseStep {
    /// Gõ sai — hiện LẠI màn nhập kèm câu báo lỗi.
    ShowError,
    /// Quay lại màn gõ để sửa. **Khác hẳn `Cancel`** — xem `ACTION_BACK`.
    Back,
    /// Đọc được — hiện màn xác nhận địa chỉ này.
    Confirm(String),
    /// Bấm lưu.
    Save,
    /// Huỷ, hoặc mã lạ.
    Cancel,
}

/// Quyết định bước tiếp theo từ một câu trả lời.
#[must_use]
pub fn phrase_step(hanh_dong: &str, go: &str) -> PhraseStep {
    if hanh_dong == recovery_screen::ACTION_CONTINUE {
        return recovery_screen::read_phrase(go).map_or(PhraseStep::ShowError, |khoa| {
            PhraseStep::Confirm(khoa.address().to_string())
        });
    }
    if hanh_dong == recovery_screen::ACTION_SAVE {
        return PhraseStep::Save;
    }
    if hanh_dong == recovery_screen::ACTION_BACK {
        return PhraseStep::Back;
    }
    PhraseStep::Cancel
}

/// Khôi phục ví bằng **cụm từ gõ thẳng**, không cần tệp nào.
///
/// ```text
/// gõ 24 chữ  →  xem địa chỉ nó mở ra  →  lưu
/// ```
///
/// Màn xác nhận địa chỉ **không phải thủ tục**: tổng kiểm BIP39 bắt được phần
/// lớn lỗi gõ nhưng không bắt hết, và khi ấy thứ duy nhất phân biệt hai ví là
/// địa chỉ — xem `recovery_screen`.
///
/// # Errors
/// Người dùng huỷ, hoặc kho khoá từ chối.
pub fn restore_from_phrase(ngon_ngu: Language) -> Result<String, WalletFlowError> {
    let ket_qua: Rc<RefCell<Result<String, WalletFlowError>>> =
        Rc::new(RefCell::new(Err(WalletFlowError::Cancelled)));
    let ra = Rc::clone(&ket_qua);
    let nhan_o = label(TextKey::CumTuNhan, ngon_ngu).to_owned();
    // Ví đang chờ xác nhận. Giữ trong bao đóng, không đi đâu khác.
    let mut dang_cho: Option<WalletSecret> = None;

    let man_dau = man_hinh(
        &recovery_screen::build_entry(None, ngon_ngu).map_err(|e| e.to_string())?,
        label(TextKey::CumTuTieuDe, ngon_ngu),
        ngon_ngu,
    );

    bo_dung_cua_so::open_sequence(man_dau, move |t| {
        let hanh_dong = t.action.as_deref().unwrap_or_default();
        if hanh_dong == recovery_screen::ACTION_CONTINUE {
            let go = t.fields.get(&nhan_o).cloned().unwrap_or_default();
            // Dùng CHÍNH hàm thuần đã kiểm được, thay vì lặp lại logic ở đây.
            // Lặp lại là để hai bên trôi dạt, và lúc đó phép thử xanh trong khi
            // cửa sổ làm một việc khác.
            let (cay, tieu_de) = if let PhraseStep::Confirm(dia_chi) = phrase_step(hanh_dong, &go) {
                let Ok(khoa) = recovery_screen::read_phrase(&go) else {
                    return Next::Done;
                };
                dang_cho = Some(khoa);
                let Ok(c) = recovery_screen::build_confirm(&dia_chi, ngon_ngu) else {
                    return Next::Done;
                };
                (c, label(TextKey::CumTuXacNhanTieuDe, ngon_ngu))
            } else {
                // Gõ lại NGAY TRÊN màn ấy, không đá về từ đầu — 24 chữ mà phải
                // gõ lại từ đầu vì một lỗi chính tả là cách chắc chắn để người
                // dùng đi dán từ chỗ khác.
                let cau = label(TextKey::CumTuLoiKhongHopLe, ngon_ngu);
                let Ok(c) = recovery_screen::build_entry(Some(cau), ngon_ngu) else {
                    return Next::Done;
                };
                (c, label(TextKey::CumTuTieuDe, ngon_ngu))
            };
            return Next::Show(Box::new(man_hinh(&cay, tieu_de, ngon_ngu)));
        }

        // Quay lại màn gõ — giữ cửa sổ, cho sửa. KHÔNG phải huỷ.
        if hanh_dong == recovery_screen::ACTION_BACK {
            dang_cho = None;
            let Ok(cay) = recovery_screen::build_entry(None, ngon_ngu) else {
                return Next::Done;
            };
            return Next::Show(Box::new(man_hinh(
                &cay,
                label(TextKey::CumTuTieuDe, ngon_ngu),
                ngon_ngu,
            )));
        }

        if hanh_dong == recovery_screen::ACTION_SAVE {
            let Some(khoa) = dang_cho.take() else {
                return Next::Done;
            };
            let dia_chi = khoa.address().to_string();
            match cat_hat_giong(&dia_chi, &khoa) {
                Ok(()) => {
                    *ra.borrow_mut() = Ok(dia_chi);
                    return Next::Done;
                }
                Err(e) => {
                    // ⚠️ Báo TRONG cửa sổ, không chỉ ra `stderr`. Người dùng
                    // trình duyệt không nhìn terminal; cửa sổ đóng im lặng thì
                    // họ chỉ thấy "gõ xong rồi ứng dụng tắt".
                    let cau = e.to_string();
                    *ra.borrow_mut() = Err(e);
                    let Ok(cay) = recovery_screen::build_failure(&cau, ngon_ngu) else {
                        return Next::Done;
                    };
                    return Next::Show(Box::new(man_hinh(
                        &cay,
                        label(TextKey::HongTieuDe, ngon_ngu),
                        ngon_ngu,
                    )));
                }
            }
        }

        Next::Done
    })
    .map_err(WalletFlowError::Window)?;

    Rc::try_unwrap(ket_qua)
        .map_err(|_| WalletFlowError::Cancelled)?
        .into_inner()
}

fn cat_hat_giong(dia_chi: &str, khoa: &WalletSecret) -> Result<(), WalletFlowError> {
    let mut kho = wallet_store::open().map_err(|e| WalletFlowError::Store(e.to_string()))?;
    let ten = wallet_store::key_name(dia_chi);
    kho.store(&ten, SecretKey::new(khoa.expose_seed().to_vec()))
        .map_err(|e| WalletFlowError::Store(e.to_string()))
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "kiểm thử: hỏng thì phải nổ ngay"
)]
mod kiem_thu_cum_tu {
    use super::*;

    const ABANDON: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
    const DIA_CHI: &str = "0x11b22b300e195c44c910d71cdb1515c4617e852393cde5e80c860906b8a2d549";

    /// **Gõ SAI thì hiện lại màn nhập, KHÔNG đóng cửa sổ.**
    ///
    /// Lỗi ngày 17/08/2026: người dùng gõ sai một chữ và cả ứng dụng tắt. Gõ
    /// lại 24 chữ đã đủ bực; mất luôn cửa sổ thì lần sau người ta đi dán từ chỗ
    /// khác — mà "chỗ khác" thường là một ô nhập trên web.
    #[test]
    fn go_sai_thi_hien_lai_chu_khong_dong() {
        for sai in ["", "chưa gõ gì", &ABANDON.replace(" art", " arm"), "0x1234"] {
            assert_eq!(
                phrase_step(recovery_screen::ACTION_CONTINUE, sai),
                PhraseStep::ShowError,
                "gõ {sai:?} không ra màn lỗi"
            );
        }
    }

    /// Gõ đúng thì sang màn xác nhận, kèm ĐÚNG địa chỉ.
    #[test]
    fn go_dung_thi_sang_man_xac_nhan() {
        assert_eq!(
            phrase_step(recovery_screen::ACTION_CONTINUE, ABANDON),
            PhraseStep::Confirm(DIA_CHI.to_owned())
        );
    }

    /// **Ở màn xác nhận, "không phải ví này" phải QUAY LẠI, không đóng.**
    ///
    /// Lỗi ngày 17/08/2026: nút ấy dùng chung mã với "huỷ", nên thấy địa chỉ
    /// sai rồi bấm là mất cả 24 chữ vừa gõ. Cả điểm của màn xác nhận là **sửa
    /// được** — bắt gõ lại từ đầu là cách chắc chắn khiến lần sau người ta đi
    /// dán từ chỗ khác, mà chỗ khác thường là một ô nhập trên web.
    #[test]
    fn quay_lai_khac_han_huy() {
        assert_eq!(
            phrase_step(recovery_screen::ACTION_BACK, ABANDON),
            PhraseStep::Back
        );
        assert_ne!(recovery_screen::ACTION_BACK, recovery_screen::ACTION_CANCEL);
    }

    /// Màn xác nhận KHÔNG được mang nút huỷ — chỉ "lưu" và "quay lại sửa".
    #[test]
    fn man_xac_nhan_khong_co_nut_huy() {
        let cay = recovery_screen::build_confirm(DIA_CHI, Language::Vi).unwrap();
        let ma: Vec<String> = cay
            .action_ids()
            .iter()
            .map(|a| a.as_str().to_owned())
            .collect();
        assert!(ma.contains(&recovery_screen::ACTION_SAVE.to_owned()));
        assert!(ma.contains(&recovery_screen::ACTION_BACK.to_owned()));
        assert!(
            !ma.contains(&recovery_screen::ACTION_CANCEL.to_owned()),
            "màn xác nhận vẫn còn nút huỷ — bấm là mất 24 chữ vừa gõ"
        );
    }

    /// Màn báo hỏng cũng phải cho quay lại sửa.
    #[test]
    fn man_hong_cho_quay_lai() {
        let cay = recovery_screen::build_failure("hỏng gì đó", Language::Vi).unwrap();
        let ma: Vec<String> = cay
            .action_ids()
            .iter()
            .map(|a| a.as_str().to_owned())
            .collect();
        assert!(
            ma.contains(&recovery_screen::ACTION_BACK.to_owned()),
            "hỏng xong chỉ còn nước đóng cửa sổ"
        );
    }

    /// Bấm lưu là lưu; bấm huỷ hoặc mã lạ là dừng.
    #[test]
    fn cac_nhanh_con_lai() {
        assert_eq!(
            phrase_step(recovery_screen::ACTION_SAVE, ""),
            PhraseStep::Save
        );
        assert_eq!(
            phrase_step(recovery_screen::ACTION_CANCEL, ABANDON),
            PhraseStep::Cancel
        );
        assert_eq!(phrase_step("ma-bia-ra", ABANDON), PhraseStep::Cancel);
    }
}
