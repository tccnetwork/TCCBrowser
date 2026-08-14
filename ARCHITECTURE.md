# TCC Browser — Kiến trúc

> Trình duyệt thế hệ mới cho hệ TCC. Tài liệu này mô tả **cách các phần ghép lại**
> và **vì sao chia như vậy**. Đặc tả tiêu chuẩn nằm ở `spec/`, kế hoạch triển khai
> ở `docs/ke-hoach.md`, đặc tả gốc v0.1 giữ ở `docs/dac-ta-goc-v0.1.md`.

---

## 1. Ba tầng nội dung

Trình duyệt mở ba loại thứ, và **cố ý** không giả vờ rằng cả ba như nhau.

```
┌──────────────────────────────────────────────────────────────┐
│                        TCC BROWSER                           │
│                                                              │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐  │
│  │  TẦNG 1        │  │  TẦNG 2        │  │  TẦNG 3        │  │
│  │  Ứng dụng TCC  │  │  Web hiện đại  │  │  Lối thoát     │  │
│  ├────────────────┤  ├────────────────┤  ├────────────────┤  │
│  │ WASM           │  │ HTML/CSS/JS    │  │ Mở bằng trình  │  │
│  │ Quyền năng     │  │ theo chuẩn đã  │  │ duyệt hệ điều  │  │
│  │ Ví, danh tính  │  │ công bố        │  │ hành           │  │
│  │ Ký hậu lượng tử│  │                │  │                │  │
│  │                │  │ Nhãn TCC Ready │  │ Netflix, trang │  │
│  │ ← ĐÂY LÀ THỨ   │  │                │  │ cần DRM, trang │  │
│  │   TA BÁN       │  │                │  │ quá cũ         │  │
│  └────────────────┘  └────────────────┘  └────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

**Tầng 3 là thứ khiến cả chiến lược khả thi.** Không có nó, ta bị buộc phải đuổi
theo Chromium mãi mãi — và cuộc đua đó không ai thắng được. Có nó, ta được phép
nói "trang này chúng tôi không chạy" mà người dùng vẫn làm được việc.

---

## 2. Cây phụ thuộc

Mũi tên đọc là "phụ thuộc vào". Chiều mũi tên **không bao giờ được đảo**.

```
                    ┌──────────────┐
                    │ tcc-browser  │  ứng dụng (mỏng)
                    └──────┬───────┘
                           │
                    ┌──────▼───────┐
                    │  tcc-shell   │  ★ ĐIỂM LẮP RÁP
                    │              │    nơi DUY NHẤT chọn bộ dựng
                    └──┬────────┬──┘
              ┌────────┘        └────────┐
              │                          │
      ┌───────▼────────┐      ┌──────────▼─────────┐
      │  tcc-runtime   │      │ tcc-render-webview │  giàn giáo
      └───┬────┬───┬───┘      └──────────┬─────────┘
          │    │   │                     │
          │    │   └─────────┐  ┌────────┘
          │    │             ▼  ▼
          │    │        ┌─────────┐
          │    │        │ tcc-ui  │  API component TRỪU TƯỢNG
          │    │        └────┬────┘
          │    │             │
   ┌──────▼──┐ │        ┌────▼─────────┐
   │tcc-mani-│ │        │              │
   │  fest   │ │        │              │
   └──┬───┬──┘ │        │              │
      │   │    │        │              │
      │   │  ┌─▼────────▼──┐           │
      │   └─►│ tcc-capabi- │           │
      │      │    lity     │           │
      │      └──────┬──────┘           │
      │             │                  │
   ┌──▼──────┐   ┌──▼──────────────────▼──┐
   │tcc-crypto│   │       tcc-spec        │
   └─────────┘   └───────────────────────┘
      ★ LÁ            ★ LÁ
   biên giới       kiểu dữ liệu
   tin cậy         của tiêu chuẩn
```

### Vì sao chia đúng chỗ này

Crate không chia theo **chủ đề**, mà theo **biên giới tin cậy và biên giới thay thế**:

| Crate | Chia ra vì |
|---|---|
| `tcc-crypto` | **Biên giới tin cậy.** Cần kiểm định độc lập, nên phải ít phụ thuộc nhất và đọc được một mình. |
| `tcc-spec` | **Người ngoài phải đọc được.** Ai muốn tự cài đặt tiêu chuẩn TCC chỉ cần crate này, không phải kéo cả trình duyệt. |
| `tcc-ui` ⟷ `tcc-render-*` | **Biên giới thay thế.** Hôm nay dựng bằng WebView, mai bằng GPU. Ứng dụng không được biết. |
| `tcc-shell` | **Điểm lắp ráp.** Chỉ một nơi biết bộ dựng cụ thể là nơi nào. |

Đặc tả gốc đề xuất **25 crate**. Ở đây có **8**, vì tạo crate rỗng không phải là
tính mô-đun — nó chỉ là thư mục rỗng. Tách thêm khi có **lý do thật**: một biên
giới tin cậy mới, hoặc một thứ cần thay thế được.

---

## 3. Luật cứng — có máy kiểm

Chạy `tools/kiem-luat-phu-thuoc.sh`, và chạy trong CI.

| # | Luật | Vì sao |
|---|---|---|
| 1 | `tcc-ui` không phụ thuộc bộ dựng nào | Mất luật này là mất đường thoát khỏi WebView |
| 2 | Chỉ `tcc-shell` phụ thuộc `tcc-render-*` | Giữ điểm lắp ráp là một |
| 3 | `tcc-crypto` là lá | Biên giới tin cậy không được phình |
| 4 | `tcc-spec` là lá | Người ngoài cài đặt được tiêu chuẩn |
| 5 | `tcc-runtime` không biết bộ dựng | Chỉ nói chuyện qua `tcc-ui` |
| 6 | Không lộ DOM/HTML/CSS ra API ứng dụng | Lộ ra là ứng dụng dính chặt WebView |

> **Luật viết trong chú thích thì sớm muộn cũng bị vi phạm** — thường vào 11 giờ
> đêm khi ai đó chỉ muốn "cho nó chạy đã". Nên chúng được cưỡng chế bằng máy.

---

## 4. Đường thoát khỏi WebView

Đây là quyết định kiến trúc **quan trọng nhất** của dự án.

```
   GIAI ĐOẠN 1                        GIAI ĐOẠN 4
   (mượn giàn giáo)                   (tự đứng)

   Ứng dụng TCC                       Ứng dụng TCC
        │                                  │
        ▼                                  ▼
   ┌─────────┐                        ┌─────────┐
   │ tcc-ui  │ ◄── ứng dụng chỉ ────► │ tcc-ui  │
   └────┬────┘     biết tầng này      └────┬────┘
        │                                  │
        ▼                                  ▼
  ┌───────────┐                     ┌────────────┐
  │ WebView   │                     │ Bộ dựng GPU│
  │ (WKWebView│                     │  (wgpu)    │
  │  WebView2)│                     └────────────┘
  └───────────┘
                    ỨNG DỤNG KHÔNG SỬA MỘT DÒNG
```

**Cái bẫy phải tránh:** nếu ứng dụng TCC được viết thẳng bằng HTML/CSS/JS chạy
trong WebView, thì ngày có bộ dựng riêng, **mọi ứng dụng phải viết lại** — và lúc
đó không ai dám bỏ WebView nữa. Giàn giáo hoá thành nhà.

Luật 1, 2, 5, 6 tồn tại **chỉ để** giữ cho ô bên phải luôn khả thi.

Lợi ích phụ: ràng buộc này bắt ta thiết kế `tcc-ui` cho tử tế ngay từ đầu. Nếu nó
đủ trừu tượng để chạy trên hai bộ dựng khác hẳn nhau, thì nó đã được thiết kế đúng.

---

## 5. Lát cắt mỏng — đường đi của một ứng dụng TCC

Đây là thứ Giai đoạn 1 phải làm chạy được từ đầu tới cuối.

```
  Gói ứng dụng .tccapp
         │
         │  ┌─────────────────────────────────────────┐
         ├─►│ 1. Kiểm chữ ký          [tcc-crypto]    │
         │  │    Ed25519 + ML-DSA (LAI)               │
         │  │    Hỏng → dừng, báo rõ sai ở đâu        │
         │  └─────────────────────────────────────────┘
         │
         │  ┌─────────────────────────────────────────┐
         ├─►│ 2. Đọc bản kê khai      [tcc-manifest]  │
         │  │    ai ký? xin quyền gì?                 │
         │  └─────────────────────────────────────────┘
         │
         │  ┌─────────────────────────────────────────┐
         ├─►│ 3. Dựng tập quyền năng  [tcc-capability]│
         │  │    KHÔNG cấp sẵn gì. Người dùng duyệt.  │
         │  └─────────────────────────────────────────┘
         │
         │  ┌─────────────────────────────────────────┐
         ├─►│ 4. Chạy                 [tcc-runtime]   │
         │  │    ứng dụng chỉ gọi được đúng thứ đã cấp│
         │  └─────────────────────────────────────────┘
         │
         │  ┌─────────────────────────────────────────┐
         └─►│ 5. Vẽ giao diện         [tcc-ui]        │
            │    → WebView (giai đoạn 1)              │
            └─────────────────────────────────────────┘
```

Chạy được đường này là **đã có trình duyệt** — nó mở được một thứ, và thứ đó là
thứ Chrome không mở được.

---

## 6. Mật mã: LAI, không thuần hậu lượng tử

```
   CHỮ KÝ                         TRAO ĐỔI KHOÁ
   ┌────────────┐                 ┌────────────┐
   │  Ed25519   │ cổ điển         │   X25519   │ cổ điển
   │     +      │                 │     +      │
   │   ML-DSA   │ hậu lượng tử    │   ML-KEM   │ hậu lượng tử
   │ (FIPS 204) │                 │ (FIPS 203) │
   └────────────┘                 └────────────┘
     An toàn nếu MỘT trong hai còn đứng vững
```

**Vì sao lai chứ không thuần hậu lượng tử:** năm 2022, **SIKE** — một ứng viên đã
vào vòng chung kết NIST — bị phá **trên một nhân CPU trong khoảng một giờ**. Thuật
toán hậu lượng tử còn quá trẻ để tin một mình.

**Cái gì gấp, cái gì không:**

| | Mức gấp | Vì sao |
|---|---|---|
| Trao đổi khoá | **GẤP** | Kẻ tấn công thu lưu lượng hôm nay, giải mã sau. Bí mật hôm nay lộ ở tương lai. |
| Chữ ký | Không gấp | Không ai giả mạo ngược được chữ ký 2026 vào năm 2040 |
| Đối xứng (AES-256, SHA-384) | **Không cần đổi** | Grover chỉ làm yếu một nửa số bit — AES-256 còn tương đương 128 bit, vẫn thừa |

Đổ công sức thay AES là lãng phí. Chỉ mã hoá **bất đối xứng** mới bị Shor phá.

---

## 7. Quy ước

**Định danh trong mã: tiếng Anh. Chú thích và tài liệu: tiếng Việt.**

Vì `spec/` là tiêu chuẩn cho người ngoài đọc và cài đặt, nên tên kiểu dữ liệu,
tên hàm, tên crate phải là tiếng Anh. Còn đội ngũ đọc mã là người Việt, nên chú
thích viết tiếng Việt — giống v1.

**Chú thích giải thích VÌ SAO, không giải thích CÁI GÌ.** Mã đã nói nó làm gì rồi.
Thứ mã không nói được là vì sao chọn cách này, và đã thử cách nào rồi hỏng.
