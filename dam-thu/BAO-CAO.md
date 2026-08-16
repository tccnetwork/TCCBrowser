# Giai đoạn 0 — ba câu đâm thử khả thi

> Kế hoạch đòi ba báo cáo ngắn kèm số đo, **không phải mã sản xuất**. Toàn bộ
> `dam-thu/` nằm NGOÀI workspace vì thế: `cosmic-text` kéo theo một cây phụ
> thuộc mà bản dựng thật không được mang, và `cargo test --workspace` không
> phải trả giá cho một thí nghiệm.
>
> Chạy ngày **17/08/2026**, trên **Intel Mac, Iris Plus 645** — kế hoạch đã dặn
> đừng kết luận hiệu năng đồ hoạ trên máy này, nên ở đây **không có phép đo tốc
> độ nào**.

## 0.1 — Chữ tiếng Việt dựng bằng Rust: ✅ ĐẠT

### Câu hỏi bị đặt sai một chút, và sửa lại thì đo được

Kế hoạch hỏi *"chữ dựng bằng **wgpu** có đẹp bằng hệ điều hành không?"*. Nhưng
wgpu không phải chỗ rủi ro: GPU chỉ dán một tấm ảnh chữ **đã rasterize** lên màn
hình. Rủi ro nằm ở hai bước trước đó, cả hai chạy trên CPU:

1. **Shaping** — xếp dấu vào đúng chỗ. `ế` là `e` + mũ + sắc, và sắc phải nằm
   trên mũ, không đè lên nhau.
2. **Rasterize** — vẽ ra pixel đủ tốt ở cỡ chữ người ta đọc cả ngày.

Đâm thử đo hai bước ấy bằng `cosmic-text` (rustybuzz + swash), **không đụng wgpu**.

### Bốn phép đo

| Phép đo | Kết quả |
|---|---|
| `Chào buổi sáng mọi người` → glyph | 20 ký tự → **24 glyph**, **0 `.notdef`** |
| 16 chữ dấu chồng hai tầng → glyph | 16 ký tự → **31 glyph**, **0 `.notdef`** |
| Dạng dựng sẵn `U+1EBF` vs tổ hợp `e+0302+0301` | **0/2560 pixel khác nhau** |
| Mực chữ có tràn khỏi chiều cao dòng không | mực **17 px** trong dòng **21 px** |

**Phép thứ ba là phép quan trọng nhất.** Bộ gõ macOS phát ra dạng dựng sẵn; tệp
văn bản và trang web thường mang dạng tổ hợp. Nếu hai dạng ra hai hình khác nhau
thì cùng một câu trông khác nhau tuỳ nó tới từ đâu — và người dùng không có cách
nào biết vì sao. Chúng ra **hình trùng khít**.

**Phép thứ tư đo lỗi phổ biến nhất của tiếng Việt**: chiều cao dòng lấy theo chữ
Latin không dấu, rồi `ế` với hai tầng dấu chạm trần và bị xén. Nó không lộ ở chữ
thường, chỉ lộ ở đúng những chữ khó nhất — nên nhìn qua thì tưởng ổn. Còn dư
**4 px**.

### Ảnh

`ra/câu-thường.png` và `ra/dấu-chồng.png`, 15 px, nền trắng chữ đen. Cả 16 chữ
dấu chồng đều đủ hai tầng dấu, không dính nhau, không cụt.

### Kết luận cho Giai đoạn 4

Không có rào chắn khả thi nào ở phần chữ. **Nhưng đây chưa phải giấy phép cho
Giai đoạn 4**: đâm thử này không đo hiệu năng (máy sai), không đo trên màn hình
Retina, và không so cạnh nhau với CoreText ở mức pixel. Nó chỉ trả lời được câu
*"có gì hỏng tới mức phải bỏ hướng này không"* — và câu trả lời là **không**.

---

## 0.2 — Sandbox WASM có đủ nhanh không: ⊘ CÂU HỎI ĐÃ TAN

Không đo, và không nên đo: **kiến trúc đã bỏ WASM khỏi mô hình ứng dụng.**

Quyết định kiến trúc số 1 nói *"ứng dụng KHÔNG mang mã; điểm vào là cây
component khai báo"*. Toàn workspace **không có một dòng nào phụ thuộc
`wasmtime`** — chỉ còn vài chuỗi `"app.wasm"` trong dữ liệu thử của các phép
kiểm đường dẫn.

Câu 0.2 sinh ra để bảo vệ *"mô hình WASM-first"*, và mô hình ấy đã bị thay. Đo
tốc độ một sandbox không ai dùng là làm ra một con số không quyết định gì.

⚠️ **Nhưng đừng ghi nó là "đạt".** Nếu ngày nào đó ứng dụng được phép mang mã
trở lại — và áp lực ấy sẽ tới — thì câu này sống lại nguyên vẹn, và phải đo
trước khi mở cửa, không phải sau.

---

## 0.3 — Cô lập tiến trình trên macOS: ❌ CHƯA ĐO

Chưa làm. Và nó vừa trở nên đắt hơn: §19 của `docs/vi-thiet-ke.md` cho thấy mọi
thứ dính tới chữ ký mã và quyền trên macOS đều đòi hồ sơ cấp phép, mà thiếu nó
thì tiến trình **treo im lặng** chứ không báo lỗi.

Cô lập tiến trình ví là đúng loại việc ấy. Nên 0.3 nên làm **sau** khi có hồ sơ
cấp phép, không phải trước — làm trước là đo một hệ thống đang bị chặn ở tầng
khác và tưởng mình đang đo cái mình định đo.
