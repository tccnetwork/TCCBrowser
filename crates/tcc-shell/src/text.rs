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
pub enum Language {
    /// Mặc định — trình duyệt phát cả ra ngoài Việt Nam.
    #[default]
    En,
    Vi,
}

/// Mọi chuỗi hiện ra người dùng trong khung trình duyệt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextKey {
    QuyenTieuDe,
    QuyenNutChoPhep,
    QuyenNutTuChoi,
    QuyenKhongXinGi,
    /// ⚠️ Câu quan trọng nhất trong cả giao diện — xem [`label`].
    QuyenCanhBaoDanhTinh,
    QuyenMang,
    QuyenLuuTru,
    QuyenVi,
    // ── màn xác nhận giao dịch ──
    GdTieuDe,
    GdChuyenTien,
    GdSoTien,
    GdNguoiNhan,
    GdPhi,
    GdMang,
    GdThuTu,
    GdGhiNho,
    GdNutKy,
    GdNutHuy,
    // ── màn nhập ví cũ từ ví web ──
    NhapTieuDe,
    NhapGiaiThich,
    NhapTrong,
    NhapNhan,
    NhapCoCumTu,
    NhapKhongCumTu,
    NhapNutChon,
    NhapNutHuy,
    NhapPinTieuDe,
    NhapPinNhan,
    /// ⚠️ Câu nói VÌ SAO đang hỏi PIN — xem `import_screen::build_pin`.
    NhapPinViSao,
    NhapPinChiDungMotLan,
    NhapNutMoKhoa,
    NhapXongTieuDe,
    /// ⚠️ Câu bắt buộc sau khi nhập xong — xem `import_screen`.
    NhapBanCuVanCon,
    NhapBanCuLamGi,
    NhapCumTuDaMangSang,
    NhapNutXong,
    NhapLoiSaiPin,
    NhapLoiKhoaCu,
    NhapLoiLechDiaChi,
    NhapLoiDocKhongDuoc,
    // ── màn gõ thẳng cụm từ khôi phục ──
    CumTuTieuDe,
    CumTuNhan,
    CumTuGiaiThich,
    /// ⚠️ Câu cảnh báo người xung quanh đọc được — xem `recovery_screen`.
    CumTuAiNhinCungDoc,
    CumTuNutTiep,
    CumTuXacNhanTieuDe,
    CumTuDayLaVi,
    CumTuKiemKyTruocKhiCat,
    CumTuNutCat,
    CumTuLoiKhongHopLe,
    // ── màn báo hỏng, hiện TRONG cửa sổ ──
    HongTieuDe,
    HongKhongCatDuoc,
    HongNutDong,
    /// Nút quay lại màn gõ — KHÔNG phải nút huỷ.
    CumTuNutSuaLai,
    /// ⚠️ Bản dựng chưa ký nên hệ điều hành không cho cất khoá.
    HongChuaKyGoi,
    HongKhongPhaiLoiCuaBan,
    // ── phiên thử: khoá chỉ sống trong bộ nhớ ──
    PhienTieuDe,
    /// ⚠️ Khoá không được cất — nói TRƯỚC khi người dùng gõ.
    PhienKhongCatDau,
    PhienDongLaMat,
    // ── màn ĐÃ GỬI ──
    XongTieuDe,
    XongMaGiaoDich,
    XongConCho,
    XongNutDong,
    // ── tầng 3: mở bằng trình duyệt hệ thống ──
    RaNgoaiTieuDe,
    /// ⚠️ Câu nói rõ ra khỏi TCC — xem `external_link`.
    RaNgoaiRoiKhoiTcc,
    RaNgoaiKhongConCheChan,
    RaNgoaiNutMo,
    RaNgoaiNutHuy,
    RaNgoaiKhongPhaiWeb,
    // ── tầng 2: thanh địa chỉ ──
    WebNhanDiaChi,
    WebNutDi,
    /// Nói thẳng rằng phiên này KHÔNG giữ gì lại.
    WebKhongGiuGi,
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
    /// Tiêu đề cửa sổ hộp thoại hỏi quyền — **của TRÌNH DUYỆT**.
    HoiQuyenTieuDeCuaSo,
}

/// Bản dịch.
///
/// # Câu cảnh báo danh tính
///
/// [`TextKey::QuyenCanhBaoDanhTinh`] là câu quan trọng nhất ở đây. Chữ ký hợp lệ
/// chứng minh gói KHÔNG BỊ SỬA — nó không chứng minh người ký là ai, vì bất kỳ
/// ai cũng tự sinh khoá được. Chừng nào chưa có sổ đăng ký khoá, giao diện
/// **không bao giờ** được viết "nhà phát hành đã xác minh". Có phép thử chốt.
#[must_use]
#[allow(
    clippy::too_many_lines,
    reason = "bảng dịch là MỘT `match` có chủ ý: cắt nhỏ ra thì trình biên dịch \
              không còn ép phủ hết mọi nhánh nữa, mà đó là cả lý do khoá là enum"
)]
#[allow(
    clippy::match_same_arms,
    reason = "hai chuỗi khác nhau tình cờ trùng bản tiếng Anh hôm nay \
              (\"Cancel\") — gộp arm lại là ngày một trong hai đổi lời thì \
              chuỗi kia đổi theo mà không ai để ý"
)]
pub const fn label(k: TextKey, n: Language) -> &'static str {
    match (k, n) {
        (TextKey::QuyenTieuDe, Language::En) => "This app is asking for permission",
        (TextKey::QuyenTieuDe, Language::Vi) => "Ứng dụng này đang xin quyền",

        (TextKey::QuyenNutChoPhep, Language::En) => "Allow",
        (TextKey::QuyenNutChoPhep, Language::Vi) => "Cho phép",

        (TextKey::QuyenNutTuChoi, Language::En) => "Deny",
        (TextKey::QuyenNutTuChoi, Language::Vi) => "Từ chối",

        (TextKey::QuyenKhongXinGi, Language::En) => "This app asks for no permissions.",
        (TextKey::QuyenKhongXinGi, Language::Vi) => "Ứng dụng này không xin quyền nào.",

        (TextKey::QuyenCanhBaoDanhTinh, Language::En) => {
            "The signature proves this package was not modified. \
             It does NOT prove who signed it — anyone can generate a key."
        }
        (TextKey::QuyenCanhBaoDanhTinh, Language::Vi) => {
            "Chữ ký chứng minh gói này không bị sửa. \
             Nó KHÔNG chứng minh người ký là ai — bất kỳ ai cũng tự sinh khoá được."
        }

        (TextKey::QuyenMang, Language::En) => "Connect to these servers",
        (TextKey::QuyenMang, Language::Vi) => "Kết nối tới các máy chủ này",

        (TextKey::QuyenLuuTru, Language::En) => "Store data on this device",
        (TextKey::QuyenLuuTru, Language::Vi) => "Lưu dữ liệu trên máy này",

        (TextKey::QuyenVi, Language::En) => "Access your TCC wallet",
        (TextKey::GdTieuDe, Language::En) => "Confirm this transaction",
        (TextKey::GdTieuDe, Language::Vi) => "Xác nhận giao dịch này",
        (TextKey::GdChuyenTien, Language::En) => "Signing this moves money — it cannot be undone",
        (TextKey::GdChuyenTien, Language::Vi) => "Ký cái này là chuyển tiền — không hoàn tác được",
        (TextKey::GdSoTien, Language::En) => "Amount",
        (TextKey::GdSoTien, Language::Vi) => "Số tiền",
        (TextKey::GdNguoiNhan, Language::En) => "To this address",
        (TextKey::GdNguoiNhan, Language::Vi) => "Gửi tới địa chỉ",
        (TextKey::GdPhi, Language::En) => "Maximum fee",
        (TextKey::GdPhi, Language::Vi) => "Phí tối đa",
        (TextKey::GdMang, Language::En) => "Network",
        (TextKey::GdMang, Language::Vi) => "Mạng",
        (TextKey::GdThuTu, Language::En) => "Sequence number",
        (TextKey::GdThuTu, Language::Vi) => "Số thứ tự",
        (TextKey::GdGhiNho, Language::En) => "Memo",
        (TextKey::GdGhiNho, Language::Vi) => "Ghi nhớ",
        (TextKey::GdNutKy, Language::En) => "Sign and send",
        (TextKey::GdNutKy, Language::Vi) => "Ký và gửi",
        (TextKey::GdNutHuy, Language::En) => "Cancel",
        (TextKey::GdNutHuy, Language::Vi) => "Huỷ",

        (TextKey::NhapTieuDe, Language::En) => "Import a wallet from the web wallet",
        (TextKey::NhapTieuDe, Language::Vi) => "Nhập ví từ ví web",
        (TextKey::NhapGiaiThich, Language::En) => {
            "These wallets were found. Pick one, then enter the PIN you use on the website."
        }
        (TextKey::NhapGiaiThich, Language::Vi) => {
            "Tìm thấy những ví này. Chọn một ví, rồi nhập mã PIN bạn dùng ở trang web."
        }
        (TextKey::NhapTrong, Language::En) => "No wallet found in that file.",
        (TextKey::NhapTrong, Language::Vi) => "Không tìm thấy ví nào trong tệp đó.",
        (TextKey::NhapNhan, Language::En) => "Label",
        (TextKey::NhapNhan, Language::Vi) => "Nhãn",
        (TextKey::NhapCoCumTu, Language::En) => "Recovery phrase included",
        (TextKey::NhapCoCumTu, Language::Vi) => "Có kèm cụm từ khôi phục",
        (TextKey::NhapKhongCumTu, Language::En) => "No recovery phrase stored",
        (TextKey::NhapKhongCumTu, Language::Vi) => "Không có cụm từ khôi phục",
        (TextKey::NhapNutChon, Language::En) => "Import this wallet",
        (TextKey::NhapNutChon, Language::Vi) => "Nhập ví này",
        (TextKey::NhapNutHuy, Language::En) => "Cancel",
        (TextKey::NhapNutHuy, Language::Vi) => "Huỷ",
        (TextKey::NhapPinTieuDe, Language::En) => "Enter the PIN for this wallet",
        (TextKey::NhapPinTieuDe, Language::Vi) => "Nhập mã PIN của ví này",
        (TextKey::NhapPinNhan, Language::En) => "PIN",
        (TextKey::NhapPinNhan, Language::Vi) => "Mã PIN",
        // Hỏi bí mật mà không nói vì sao là dạy người dùng gõ bí mật vào bất
        // kỳ ô nào hỏi. Câu này phải đứng NGAY TRÊN ô nhập.
        (TextKey::NhapPinViSao, Language::En) => {
            "This is the PIN you use on the website. It unlocks the copy of the key stored there."
        }
        (TextKey::NhapPinViSao, Language::Vi) => {
            "Đây là mã PIN bạn dùng ở trang web. Nó mở bản khoá đang cất ở đó."
        }
        (TextKey::NhapPinChiDungMotLan, Language::En) => {
            "It is used once, here, and is not stored."
        }
        (TextKey::NhapPinChiDungMotLan, Language::Vi) => {
            "Mã này chỉ dùng một lần ở đây, và không được lưu lại."
        }
        (TextKey::NhapNutMoKhoa, Language::En) => "Unlock",
        (TextKey::NhapNutMoKhoa, Language::Vi) => "Mở khoá",
        (TextKey::NhapXongTieuDe, Language::En) => "Wallet imported",
        (TextKey::NhapXongTieuDe, Language::Vi) => "Đã nhập ví",
        // ⚠️ Câu này KHÔNG được bỏ đi và KHÔNG được viết nhẹ hơn. Người dùng
        // tưởng đã dọn sạch trong khi bản yếu vẫn nằm ở trang web là tình
        // huống xấu nhất: mất cảnh giác mà rủi ro không giảm.
        (TextKey::NhapBanCuVanCon, Language::En) => {
            "The website still has its own copy of this wallet, still locked with the same PIN."
        }
        (TextKey::NhapBanCuVanCon, Language::Vi) => {
            "Trang web vẫn giữ một bản của ví này, vẫn khoá bằng đúng mã PIN cũ."
        }
        (TextKey::NhapBanCuLamGi, Language::En) => {
            "Nothing here touched it. Remove it on the website itself when you are sure this copy works."
        }
        (TextKey::NhapBanCuLamGi, Language::Vi) => {
            "Ở đây không đụng vào bản ấy. Khi nào chắc bản này chạy được thì tự xoá nó ngay trên trang web."
        }
        (TextKey::NhapCumTuDaMangSang, Language::En) => "Your recovery phrase came across too.",
        (TextKey::NhapCumTuDaMangSang, Language::Vi) => "Cụm từ khôi phục cũng đã mang sang.",
        (TextKey::NhapNutXong, Language::En) => "Done",
        (TextKey::NhapNutXong, Language::Vi) => "Xong",
        (TextKey::NhapLoiSaiPin, Language::En) => "Wrong PIN, or the data is damaged.",
        (TextKey::NhapLoiSaiPin, Language::Vi) => "Sai PIN, hoặc dữ liệu đã hỏng.",
        (TextKey::NhapLoiKhoaCu, Language::En) => {
            "This wallet is older than the ML-DSA change and cannot be imported here."
        }
        (TextKey::NhapLoiKhoaCu, Language::Vi) => {
            "Ví này cũ hơn bản ML-DSA, không nhập được ở đây."
        }
        (TextKey::NhapLoiLechDiaChi, Language::En) => {
            "The key does not match the address in that record. Nothing was imported."
        }
        (TextKey::NhapLoiLechDiaChi, Language::Vi) => {
            "Khoá không khớp địa chỉ ghi trong bản ghi ấy. Không nhập gì cả."
        }
        (TextKey::NhapLoiDocKhongDuoc, Language::En) => "That file is not a wallet export.",
        (TextKey::NhapLoiDocKhongDuoc, Language::Vi) => "Tệp đó không phải bản kết xuất ví.",
        (TextKey::CumTuTieuDe, Language::En) => "Restore a wallet from its recovery phrase",
        (TextKey::CumTuTieuDe, Language::Vi) => "Khôi phục ví từ cụm từ khôi phục",
        (TextKey::CumTuNhan, Language::En) => "Recovery phrase",
        (TextKey::CumTuNhan, Language::Vi) => "Cụm từ khôi phục",
        (TextKey::CumTuGiaiThich, Language::En) => {
            "The 24 words, separated by spaces. A 64-character seed in hexadecimal works too."
        }
        (TextKey::CumTuGiaiThich, Language::Vi) => {
            "24 chữ, cách nhau bằng dấu cách. Một hạt giống 64 ký tự hex cũng được."
        }
        // ⚠️ Chữ KHÔNG bị che, cố ý — xem `recovery_screen`. Nên phải nói ra.
        (TextKey::CumTuAiNhinCungDoc, Language::En) => {
            "These words are shown as you type, so anyone who can see this screen can take your wallet."
        }
        (TextKey::CumTuAiNhinCungDoc, Language::Vi) => {
            "Chữ hiện ra khi bạn gõ, nên ai nhìn được màn hình này là lấy được ví của bạn."
        }
        (TextKey::CumTuNutTiep, Language::En) => "Continue",
        (TextKey::CumTuNutTiep, Language::Vi) => "Tiếp tục",
        (TextKey::CumTuXacNhanTieuDe, Language::En) => "Is this the right wallet?",
        (TextKey::CumTuXacNhanTieuDe, Language::Vi) => "Đúng ví này chứ?",
        (TextKey::CumTuDayLaVi, Language::En) => "These words open this wallet",
        (TextKey::CumTuDayLaVi, Language::Vi) => "Cụm từ này mở ra ví",
        (TextKey::CumTuKiemKyTruocKhiCat, Language::En) => {
            "Check the address before saving. One mistyped word opens a different wallet that is just as valid."
        }
        (TextKey::CumTuKiemKyTruocKhiCat, Language::Vi) => {
            "Đối chiếu địa chỉ trước khi lưu. Gõ nhầm một chữ là ra một ví khác, cũng hợp lệ y như thế."
        }
        (TextKey::CumTuNutCat, Language::En) => "Save this wallet",
        (TextKey::CumTuNutCat, Language::Vi) => "Lưu ví này",
        (TextKey::CumTuLoiKhongHopLe, Language::En) => {
            "Not a valid recovery phrase: 24 words, or 64 hexadecimal characters."
        }
        (TextKey::CumTuLoiKhongHopLe, Language::Vi) => {
            "Không phải cụm từ khôi phục hợp lệ: cần 24 chữ, hoặc 64 ký tự hex."
        }
        (TextKey::HongTieuDe, Language::En) => "That did not work",
        (TextKey::HongTieuDe, Language::Vi) => "Việc này không xong",
        (TextKey::HongKhongCatDuoc, Language::En) => {
            "Nothing was saved. Your wallet is unchanged, and the words you typed were not stored anywhere."
        }
        (TextKey::HongKhongCatDuoc, Language::Vi) => {
            "Không có gì được lưu. Ví của bạn nguyên như cũ, và cụm từ bạn vừa gõ không được cất ở đâu cả."
        }
        (TextKey::HongNutDong, Language::En) => "Close",
        (TextKey::HongNutDong, Language::Vi) => "Đóng",
        (TextKey::CumTuNutSuaLai, Language::En) => "Go back and fix it",
        (TextKey::CumTuNutSuaLai, Language::Vi) => "Quay lại sửa",
        // Câu của hệ điều hành ("A required entitlement isn't present") nói với
        // LẬP TRÌNH VIÊN. Người dùng đọc nó chỉ tưởng mình vừa làm sai gì đó.
        (TextKey::HongChuaKyGoi, Language::En) => {
            "This build is not signed yet, so macOS will not let it store a key in the Keychain."
        }
        (TextKey::HongChuaKyGoi, Language::Vi) => {
            "Bản dựng này chưa được ký, nên macOS không cho nó cất khoá vào Keychain."
        }
        (TextKey::HongKhongPhaiLoiCuaBan, Language::En) => {
            "Nothing you typed was wrong. The wallet cannot be saved until the app is signed."
        }
        (TextKey::HongKhongPhaiLoiCuaBan, Language::Vi) => {
            "Không phải bạn gõ sai. Ví chưa lưu được cho tới khi ứng dụng được ký."
        }
        (TextKey::PhienTieuDe, Language::En) => "Try the wallet for this session only",
        (TextKey::PhienTieuDe, Language::Vi) => "Dùng thử ví, chỉ trong phiên này",
        (TextKey::PhienKhongCatDau, Language::En) => {
            "The key stays in memory and is never saved. Nothing is written to the Keychain or to disk."
        }
        (TextKey::PhienKhongCatDau, Language::Vi) => {
            "Khoá chỉ nằm trong bộ nhớ, không được cất. Không ghi vào Keychain, không ghi ra đĩa."
        }
        (TextKey::PhienDongLaMat, Language::En) => {
            "Close this window and the key is gone. You will have to type the phrase again."
        }
        (TextKey::PhienDongLaMat, Language::Vi) => {
            "Đóng cửa sổ là mất khoá. Muốn dùng lại thì phải gõ lại cụm từ."
        }
        (TextKey::XongTieuDe, Language::En) => "Sent",
        (TextKey::XongTieuDe, Language::Vi) => "Đã gửi",
        (TextKey::XongMaGiaoDich, Language::En) => "Transaction id",
        (TextKey::XongMaGiaoDich, Language::Vi) => "Mã giao dịch",
        // Nhận != đã ghi vào khối. Nói rõ, vì "đã gửi" dễ bị đọc thành "xong rồi".
        (TextKey::XongConCho, Language::En) => {
            "The network accepted it. It is not in a block yet — check the id above to see when it lands."
        }
        (TextKey::XongConCho, Language::Vi) => {
            "Mạng đã nhận. Nó CHƯA vào khối — tra mã ở trên để biết khi nào lên."
        }
        (TextKey::XongNutDong, Language::En) => "Close",
        (TextKey::XongNutDong, Language::Vi) => "Đóng",
        (TextKey::RaNgoaiTieuDe, Language::En) => "Open in your system browser?",
        (TextKey::RaNgoaiTieuDe, Language::Vi) => "Mở bằng trình duyệt hệ thống?",
        // "Không giấu, không xin lỗi" — docs/ke-hoach.md, tầng 3.
        (TextKey::RaNgoaiRoiKhoiTcc, Language::En) => {
            "This leaves TCC Browser. The page opens in your normal browser, as an ordinary web page."
        }
        (TextKey::RaNgoaiRoiKhoiTcc, Language::Vi) => {
            "Việc này ra khỏi TCC Browser. Trang sẽ mở trong trình duyệt thường của bạn, như một trang web bình thường."
        }
        (TextKey::RaNgoaiKhongConCheChan, Language::En) => {
            "Nothing here protects you there: no capability gate, no signature, no permission prompt."
        }
        (TextKey::RaNgoaiKhongConCheChan, Language::Vi) => {
            "Ở đó không còn thứ gì của TCC che chắn: không cổng quyền năng, không chữ ký, không hỏi quyền."
        }
        (TextKey::RaNgoaiNutMo, Language::En) => "Open it",
        (TextKey::RaNgoaiNutMo, Language::Vi) => "Mở ra",
        (TextKey::RaNgoaiNutHuy, Language::En) => "Stay here",
        (TextKey::RaNgoaiNutHuy, Language::Vi) => "Ở lại đây",
        (TextKey::RaNgoaiKhongPhaiWeb, Language::En) => {
            "Only http and https links can be opened. Anything else is refused."
        }
        (TextKey::RaNgoaiKhongPhaiWeb, Language::Vi) => {
            "Chỉ mở được liên kết http và https. Thứ khác thì từ chối."
        }
        (TextKey::WebNhanDiaChi, Language::En) => "Address",
        (TextKey::WebNhanDiaChi, Language::Vi) => "Địa chỉ",
        (TextKey::WebKhongGiuGi, Language::En) => {
            "Nothing is kept: cookies and logins vanish when this window closes"
        }
        (TextKey::WebKhongGiuGi, Language::Vi) => {
            "Không giữ gì: cookie và đăng nhập mất khi đóng cửa sổ này"
        }
        (TextKey::WebNutDi, Language::En) => "Go",
        (TextKey::WebNutDi, Language::Vi) => "Đi",
        (TextKey::QuyenVi, Language::Vi) => "Truy cập ví TCC của bạn",

        (TextKey::ViDuocXinChuKy, Language::En) => {
            "Can ask you to sign transactions — this moves money"
        }
        (TextKey::ViDuocXinChuKy, Language::Vi) => {
            "Được phép xin bạn ký giao dịch — việc này chuyển tiền"
        }

        (TextKey::ViChiDocDiaChi, Language::En) => "Read your wallet address only",
        (TextKey::ViChiDocDiaChi, Language::Vi) => "Chỉ đọc địa chỉ ví của bạn",

        (TextKey::NguonKhongRo, Language::En) => "Unknown publisher",
        (TextKey::NguonKhongRo, Language::Vi) => "Không rõ nhà phát hành",

        // Câu này nêu một SỰ THẬT QUAN SÁT ĐƯỢC, không phải phán quyết. Ta không
        // biết ai đúng ai sai — có thể nhà phát hành đổi khoá hợp lệ, có thể là
        // gói giả mạo. Viết "ứng dụng này giả mạo" là nói điều ta không biết.
        (TextKey::DoiKhoaKy, Language::En) => "This app was previously signed with a DIFFERENT key",
        (TextKey::DoiKhoaKy, Language::Vi) => "Ứng dụng này trước đây được ký bằng một khoá KHÁC",

        (TextKey::DoiKhoaKyGiaiThich, Language::En) => {
            "That can mean the publisher rotated their key — or that this is a different \
             app pretending to be the one you trusted. Every permission you granted before \
             has been cleared."
        }
        (TextKey::DoiKhoaKyGiaiThich, Language::Vi) => {
            "Có thể nhà phát hành đã đổi khoá — cũng có thể đây là một ứng dụng khác \
             mạo danh ứng dụng bạn từng tin. Mọi quyền bạn đã cấp trước đây đã bị xoá."
        }

        (TextKey::KhoaCu, Language::En) => "Key used before",
        (TextKey::KhoaCu, Language::Vi) => "Khoá dùng lần trước",

        (TextKey::QuanLyTieuDe, Language::En) => "Permissions you have answered",
        (TextKey::QuanLyTieuDe, Language::Vi) => "Những quyền bạn đã trả lời",

        (TextKey::QuanLyTrong, Language::En) => "No app has asked you for anything yet.",
        (TextKey::QuanLyTrong, Language::Vi) => "Chưa ứng dụng nào hỏi bạn điều gì.",

        (TextKey::QuanLyDaChoPhep, Language::En) => "ALLOWED",
        (TextKey::QuanLyDaChoPhep, Language::Vi) => "ĐÃ CHO PHÉP",

        (TextKey::QuanLyDaTuChoi, Language::En) => "denied",
        (TextKey::QuanLyDaTuChoi, Language::Vi) => "đã từ chối",

        (TextKey::QuanLyNutQuen, Language::En) => "Forget this app",
        (TextKey::QuanLyNutQuen, Language::Vi) => "Quên ứng dụng này",

        (TextKey::QuanLyGiaiThichQuen, Language::En) => {
            "Forgetting removes every answer you gave this app. It will ask again \
             next time — it does not ask on your behalf."
        }
        (TextKey::QuanLyGiaiThichQuen, Language::Vi) => {
            "Quên là xoá mọi câu trả lời bạn đã cho ứng dụng này. Lần sau nó sẽ hỏi \
             lại — chứ không tự trả lời thay bạn."
        }

        (TextKey::QuanLyNutDong, Language::En) => "Close",
        (TextKey::QuanLyNutDong, Language::Vi) => "Đóng",

        (TextKey::CauMatMat, Language::En) => "This cannot be undone.",
        (TextKey::CauMatMat, Language::Vi) => "Hành động này không hoàn tác được.",

        // `aria-roledescription` THAY THẾ tên vai trò, nên chuỗi phải tự nhắc
        // đây là một nút — nếu không người dùng mất thông tin đó.
        (TextKey::VaiTroMatMat, Language::En) => "button — this cannot be undone",
        (TextKey::VaiTroMatMat, Language::Vi) => "nút — hành động này không hoàn tác được",

        (TextKey::HoiQuyenTieuDeCuaSo, Language::En) => "TCC — permission request",
        (TextKey::HoiQuyenTieuDeCuaSo, Language::Vi) => "TCC — hỏi quyền",
        (TextKey::QuanLyTieuDeCuaSo, Language::En) => "TCC — permissions granted",
        (TextKey::QuanLyTieuDeCuaSo, Language::Vi) => "TCC — quyền đã cấp",
    }
}

/// Chữ mà bộ dựng cần, đã dịch sẵn.
///
/// Bộ dựng không biết ngôn ngữ — bảng dịch nằm ở đây, và hàm này là cửa duy
/// nhất đưa chữ xuống. Thêm một chuỗi cho bộ dựng thì thêm ở `TextKey` rồi thêm
/// vào đây, không bao giờ viết thẳng vào bộ dựng.
#[must_use]
pub fn renderer_text(n: Language) -> tcc_render_webview::markup::RendererText {
    tcc_render_webview::markup::RendererText {
        cau_mat_mat: label(TextKey::CauMatMat, n).to_owned(),
        vai_tro_mat_mat: label(TextKey::VaiTroMatMat, n).to_owned(),
    }
}

#[cfg(test)]
mod kiem_thu {
    use super::*;

    const MOI_KHOA: &[TextKey] = &[
        TextKey::QuyenTieuDe,
        TextKey::QuyenNutChoPhep,
        TextKey::QuyenNutTuChoi,
        TextKey::QuyenKhongXinGi,
        TextKey::QuyenCanhBaoDanhTinh,
        TextKey::QuyenMang,
        TextKey::QuyenLuuTru,
        TextKey::QuyenVi,
        TextKey::ViDuocXinChuKy,
        TextKey::ViChiDocDiaChi,
        TextKey::NguonKhongRo,
        TextKey::DoiKhoaKy,
        TextKey::DoiKhoaKyGiaiThich,
        TextKey::KhoaCu,
        TextKey::QuanLyTieuDe,
        TextKey::QuanLyTrong,
        TextKey::QuanLyDaChoPhep,
        TextKey::QuanLyDaTuChoi,
        TextKey::QuanLyNutQuen,
        TextKey::QuanLyGiaiThichQuen,
        TextKey::QuanLyNutDong,
        TextKey::CauMatMat,
        TextKey::VaiTroMatMat,
        TextKey::QuanLyTieuDeCuaSo,
        TextKey::HoiQuyenTieuDeCuaSo,
    ];

    /// Không chuỗi nào được rỗng, và hai ngôn ngữ không được trùng nhau —
    /// trùng nhau gần như luôn nghĩa là quên dịch một bên.
    #[test]
    fn hai_ngon_ngu_deu_co_chu_va_khac_nhau() {
        for &k in MOI_KHOA {
            let en = label(k, Language::En);
            let vi = label(k, Language::Vi);
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
            for n in [Language::En, Language::Vi] {
                assert!(
                    tcc_ui::Node::text(label(k, n)).is_ok(),
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
            for n in [Language::En, Language::Vi] {
                let s = label(k, n).to_lowercase();
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
        let en = label(TextKey::QuyenCanhBaoDanhTinh, Language::En);
        assert!(en.contains("not modified"), "thiếu vế chứng minh: {en}");
        assert!(en.contains("does NOT prove who"), "thiếu vế phủ định: {en}");

        let vi = label(TextKey::QuyenCanhBaoDanhTinh, Language::Vi);
        assert!(vi.contains("không bị sửa"), "thiếu vế chứng minh: {vi}");
        assert!(vi.contains("KHÔNG chứng minh"), "thiếu vế phủ định: {vi}");
    }
}
