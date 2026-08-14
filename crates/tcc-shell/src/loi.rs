//! Chuỗi hiện ra người dùng, song ngữ, mặc định TIẾNG ANH.
//!
//! # Vì sao khoá là ENUM chứ không phải chuỗi
//!
//! Bản v1 (Electron) dùng khoá chuỗi: `t('mk.thieuKho')`. Thiếu một bản dịch thì
//! `t()` trả về chính cái khoá, và ta nhìn màn hình mới biết. Ở đây khoá là
//! `enum`, nên `match` phải phủ hết mọi nhánh — **thiếu một bản dịch là KHÔNG
//! BIÊN DỊCH ĐƯỢC**. Gõ sai tên khoá cũng thế.
//!
//! Cái giá: thêm một chuỗi phải sửa hai chỗ (enum và bảng dịch). Đổi lại không
//! bao giờ có chuỗi thiếu lọt ra bản phát hành.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NgonNgu {
    /// Mặc định — trình duyệt phát cả ra ngoài Việt Nam.
    #[default]
    En,
    Vi,
}

/// Mọi chuỗi hiện ra người dùng trong khung trình duyệt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Khoa {
    QuyenTieuDe,
    QuyenNutChoPhep,
    QuyenNutTuChoi,
    QuyenKhongXinGi,
    /// ⚠️ Câu quan trọng nhất trong cả giao diện — xem [`nhan`].
    QuyenCanhBaoDanhTinh,
    QuyenMang,
    QuyenLuuTru,
    QuyenVi,
    ViDuocXinChuKy,
    ViChiDocDiaChi,
    NguonKhongRo,
    /// ⚠️ Cùng mã ứng dụng nhưng khoá ký đã đổi.
    DoiKhoaKy,
    DoiKhoaKyGiaiThich,
    KhoaCu,
    QuanLyTieuDe,
    QuanLyTrong,
    QuanLyDaChoPhep,
    QuanLyDaTuChoi,
    QuanLyNutQuen,
    QuanLyGiaiThichQuen,
    QuanLyNutDong,
    /// Câu đọc lên cho hành động không hoàn tác được.
    CauMatMat,
    /// Chuỗi thay tên vai trò "nút" cho hành động không hoàn tác được.
    VaiTroMatMat,
    /// Tiêu đề cửa sổ màn hình quản lý quyền.
    QuanLyTieuDeCuaSo,
}

/// Bản dịch.
///
/// # Câu cảnh báo danh tính
///
/// [`Khoa::QuyenCanhBaoDanhTinh`] là câu quan trọng nhất ở đây. Chữ ký hợp lệ
/// chứng minh gói KHÔNG BỊ SỬA — nó không chứng minh người ký là ai, vì bất kỳ
/// ai cũng tự sinh khoá được. Chừng nào chưa có sổ đăng ký khoá, giao diện
/// **không bao giờ** được viết "nhà phát hành đã xác minh". Có phép thử chốt.
#[must_use]
pub const fn nhan(k: Khoa, n: NgonNgu) -> &'static str {
    match (k, n) {
        (Khoa::QuyenTieuDe, NgonNgu::En) => "This app is asking for permission",
        (Khoa::QuyenTieuDe, NgonNgu::Vi) => "Ứng dụng này đang xin quyền",

        (Khoa::QuyenNutChoPhep, NgonNgu::En) => "Allow",
        (Khoa::QuyenNutChoPhep, NgonNgu::Vi) => "Cho phép",

        (Khoa::QuyenNutTuChoi, NgonNgu::En) => "Deny",
        (Khoa::QuyenNutTuChoi, NgonNgu::Vi) => "Từ chối",

        (Khoa::QuyenKhongXinGi, NgonNgu::En) => "This app asks for no permissions.",
        (Khoa::QuyenKhongXinGi, NgonNgu::Vi) => "Ứng dụng này không xin quyền nào.",

        (Khoa::QuyenCanhBaoDanhTinh, NgonNgu::En) => {
            "The signature proves this package was not modified. \
             It does NOT prove who signed it — anyone can generate a key."
        }
        (Khoa::QuyenCanhBaoDanhTinh, NgonNgu::Vi) => {
            "Chữ ký chứng minh gói này không bị sửa. \
             Nó KHÔNG chứng minh người ký là ai — bất kỳ ai cũng tự sinh khoá được."
        }

        (Khoa::QuyenMang, NgonNgu::En) => "Connect to these servers",
        (Khoa::QuyenMang, NgonNgu::Vi) => "Kết nối tới các máy chủ này",

        (Khoa::QuyenLuuTru, NgonNgu::En) => "Store data on this device",
        (Khoa::QuyenLuuTru, NgonNgu::Vi) => "Lưu dữ liệu trên máy này",

        (Khoa::QuyenVi, NgonNgu::En) => "Access your TCC wallet",
        (Khoa::QuyenVi, NgonNgu::Vi) => "Truy cập ví TCC của bạn",

        (Khoa::ViDuocXinChuKy, NgonNgu::En) => {
            "Can ask you to sign transactions — this moves money"
        }
        (Khoa::ViDuocXinChuKy, NgonNgu::Vi) => {
            "Được phép xin bạn ký giao dịch — việc này chuyển tiền"
        }

        (Khoa::ViChiDocDiaChi, NgonNgu::En) => "Read your wallet address only",
        (Khoa::ViChiDocDiaChi, NgonNgu::Vi) => "Chỉ đọc địa chỉ ví của bạn",

        (Khoa::NguonKhongRo, NgonNgu::En) => "Unknown publisher",
        (Khoa::NguonKhongRo, NgonNgu::Vi) => "Không rõ nhà phát hành",

        // Câu này nêu một SỰ THẬT QUAN SÁT ĐƯỢC, không phải phán quyết. Ta không
        // biết ai đúng ai sai — có thể nhà phát hành đổi khoá hợp lệ, có thể là
        // gói giả mạo. Viết "ứng dụng này giả mạo" là nói điều ta không biết.
        (Khoa::DoiKhoaKy, NgonNgu::En) => "This app was previously signed with a DIFFERENT key",
        (Khoa::DoiKhoaKy, NgonNgu::Vi) => "Ứng dụng này trước đây được ký bằng một khoá KHÁC",

        (Khoa::DoiKhoaKyGiaiThich, NgonNgu::En) => {
            "That can mean the publisher rotated their key — or that this is a different \
             app pretending to be the one you trusted. Every permission you granted before \
             has been cleared."
        }
        (Khoa::DoiKhoaKyGiaiThich, NgonNgu::Vi) => {
            "Có thể nhà phát hành đã đổi khoá — cũng có thể đây là một ứng dụng khác \
             mạo danh ứng dụng bạn từng tin. Mọi quyền bạn đã cấp trước đây đã bị xoá."
        }

        (Khoa::KhoaCu, NgonNgu::En) => "Key used before",
        (Khoa::KhoaCu, NgonNgu::Vi) => "Khoá dùng lần trước",

        (Khoa::QuanLyTieuDe, NgonNgu::En) => "Permissions you have answered",
        (Khoa::QuanLyTieuDe, NgonNgu::Vi) => "Những quyền bạn đã trả lời",

        (Khoa::QuanLyTrong, NgonNgu::En) => "No app has asked you for anything yet.",
        (Khoa::QuanLyTrong, NgonNgu::Vi) => "Chưa ứng dụng nào hỏi bạn điều gì.",

        (Khoa::QuanLyDaChoPhep, NgonNgu::En) => "ALLOWED",
        (Khoa::QuanLyDaChoPhep, NgonNgu::Vi) => "ĐÃ CHO PHÉP",

        (Khoa::QuanLyDaTuChoi, NgonNgu::En) => "denied",
        (Khoa::QuanLyDaTuChoi, NgonNgu::Vi) => "đã từ chối",

        (Khoa::QuanLyNutQuen, NgonNgu::En) => "Forget this app",
        (Khoa::QuanLyNutQuen, NgonNgu::Vi) => "Quên ứng dụng này",

        (Khoa::QuanLyGiaiThichQuen, NgonNgu::En) => {
            "Forgetting removes every answer you gave this app. It will ask again \
             next time — it does not ask on your behalf."
        }
        (Khoa::QuanLyGiaiThichQuen, NgonNgu::Vi) => {
            "Quên là xoá mọi câu trả lời bạn đã cho ứng dụng này. Lần sau nó sẽ hỏi \
             lại — chứ không tự trả lời thay bạn."
        }

        (Khoa::QuanLyNutDong, NgonNgu::En) => "Close",
        (Khoa::QuanLyNutDong, NgonNgu::Vi) => "Đóng",

        (Khoa::CauMatMat, NgonNgu::En) => "This cannot be undone.",
        (Khoa::CauMatMat, NgonNgu::Vi) => "Hành động này không hoàn tác được.",

        // `aria-roledescription` THAY THẾ tên vai trò, nên chuỗi phải tự nhắc
        // đây là một nút — nếu không người dùng mất thông tin đó.
        (Khoa::VaiTroMatMat, NgonNgu::En) => "button — this cannot be undone",
        (Khoa::VaiTroMatMat, NgonNgu::Vi) => "nút — hành động này không hoàn tác được",

        (Khoa::QuanLyTieuDeCuaSo, NgonNgu::En) => "TCC — permissions granted",
        (Khoa::QuanLyTieuDeCuaSo, NgonNgu::Vi) => "TCC — quyền đã cấp",
    }
}

/// Chữ mà bộ dựng cần, đã dịch sẵn.
///
/// Bộ dựng không biết ngôn ngữ — bảng dịch nằm ở đây, và hàm này là cửa duy
/// nhất đưa chữ xuống. Thêm một chuỗi cho bộ dựng thì thêm ở `Khoa` rồi thêm
/// vào đây, không bao giờ viết thẳng vào bộ dựng.
#[must_use]
pub fn chu_bo_dung(n: NgonNgu) -> tcc_render_webview::danh_dau::ChuBoDung {
    tcc_render_webview::danh_dau::ChuBoDung {
        cau_mat_mat: nhan(Khoa::CauMatMat, n).to_owned(),
        vai_tro_mat_mat: nhan(Khoa::VaiTroMatMat, n).to_owned(),
    }
}

#[cfg(test)]
mod kiem_thu {
    use super::*;

    const MOI_KHOA: &[Khoa] = &[
        Khoa::QuyenTieuDe,
        Khoa::QuyenNutChoPhep,
        Khoa::QuyenNutTuChoi,
        Khoa::QuyenKhongXinGi,
        Khoa::QuyenCanhBaoDanhTinh,
        Khoa::QuyenMang,
        Khoa::QuyenLuuTru,
        Khoa::QuyenVi,
        Khoa::ViDuocXinChuKy,
        Khoa::ViChiDocDiaChi,
        Khoa::NguonKhongRo,
        Khoa::DoiKhoaKy,
        Khoa::DoiKhoaKyGiaiThich,
        Khoa::KhoaCu,
        Khoa::QuanLyTieuDe,
        Khoa::QuanLyTrong,
        Khoa::QuanLyDaChoPhep,
        Khoa::QuanLyDaTuChoi,
        Khoa::QuanLyNutQuen,
        Khoa::QuanLyGiaiThichQuen,
        Khoa::QuanLyNutDong,
        Khoa::CauMatMat,
        Khoa::VaiTroMatMat,
        Khoa::QuanLyTieuDeCuaSo,
    ];

    /// Không chuỗi nào được rỗng, và hai ngôn ngữ không được trùng nhau —
    /// trùng nhau gần như luôn nghĩa là quên dịch một bên.
    #[test]
    fn hai_ngon_ngu_deu_co_chu_va_khac_nhau() {
        for &k in MOI_KHOA {
            let en = nhan(k, NgonNgu::En);
            let vi = nhan(k, NgonNgu::Vi);
            assert!(!en.trim().is_empty(), "{k:?} thiếu bản tiếng Anh");
            assert!(!vi.trim().is_empty(), "{k:?} thiếu bản tiếng Việt");
            assert_ne!(
                en, vi,
                "{k:?} hai bản y hệt nhau — nhiều khả năng quên dịch"
            );
        }
    }

    /// Mọi chuỗi phải qua được phép kiểm hiển thị của `tcc-ui`, nếu không thì
    /// dựng hộp thoại sẽ hỏng lúc chạy chứ không hỏng lúc kiểm thử.
    #[test]
    fn moi_chuoi_deu_dung_duoc_trong_giao_dien() {
        for &k in MOI_KHOA {
            for n in [NgonNgu::En, NgonNgu::Vi] {
                assert!(
                    tcc_ui::Node::text(nhan(k, n)).is_ok(),
                    "{k:?}/{n:?} không dùng được làm chữ trên giao diện"
                );
            }
        }
    }

    /// ⚠️ LUẬT CỨNG CỦA v2: giao diện KHÔNG BAO GIỜ nói "đã xác minh" khi mới
    /// chỉ kiểm được chữ ký.
    ///
    /// Phép thử này quét toàn bộ bảng dịch. Ai thêm một chuỗi có chữ đó vào sẽ
    /// bị chặn ngay, kể cả khi họ chưa đọc `SECURITY.md`.
    #[test]
    fn khong_chuoi_nao_noi_da_xac_minh_nha_phat_hanh() {
        const CAM: &[&str] = &[
            "verified publisher",
            "publisher verified",
            "trusted publisher",
            "nhà phát hành đã xác minh",
            "đã xác minh nhà phát hành",
            "nhà phát hành tin cậy",
            // Cảnh báo đổi khoá cũng không được thành phán quyết: ta không biết
            // ai đúng ai sai, chỉ biết khoá đã đổi.
            "giả mạo",
            "lừa đảo",
            "is fake",
            "is malicious",
        ];
        for &k in MOI_KHOA {
            for n in [NgonNgu::En, NgonNgu::Vi] {
                let s = nhan(k, n).to_lowercase();
                for c in CAM {
                    assert!(
                        !s.contains(c),
                        "{k:?}/{n:?} nói \"{c}\" — chữ ký KHÔNG chứng minh danh tính"
                    );
                }
            }
        }
    }

    /// Câu cảnh báo phải nói rõ CẢ HAI vế: chứng minh cái gì, và không chứng
    /// minh cái gì. Chỉ nói vế đầu là hiểu ngược.
    #[test]
    fn canh_bao_danh_tinh_noi_du_hai_ve() {
        let en = nhan(Khoa::QuyenCanhBaoDanhTinh, NgonNgu::En);
        assert!(en.contains("not modified"), "thiếu vế chứng minh: {en}");
        assert!(en.contains("does NOT prove who"), "thiếu vế phủ định: {en}");

        let vi = nhan(Khoa::QuyenCanhBaoDanhTinh, NgonNgu::Vi);
        assert!(vi.contains("không bị sửa"), "thiếu vế chứng minh: {vi}");
        assert!(vi.contains("KHÔNG chứng minh"), "thiếu vế phủ định: {vi}");
    }
}
