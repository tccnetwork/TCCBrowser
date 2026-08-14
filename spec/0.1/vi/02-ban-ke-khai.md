# 02 — Bản kê khai

`manifest.json` — JSON, UTF-8, **tối đa 64 KiB**.

Trần này kiểm ở **bước 0**, trước cả khi phân tích JSON: không có nó thì một tệp
hàng trăm MB được phân tích xong xuôi trước khi ta kịp kiểm chữ ký.

Khoá JSON **KHÔNG ĐƯỢC** trùng lặp. Bản cài đặt **PHẢI** từ chối, không được lấy
khoá cuối — nếu không, công cụ hiển thị và bên kiểm chữ ký có thể thấy hai giá
trị khác nhau.

## Ví dụ đầy đủ

```json
{
  "spec_version": "0.1",
  "id": "com.tcc.vi-du.hello",
  "name": "Xin chào TCC",
  "version": "0.1.0",
  "publisher": "<3968 ký tự hex>",
  "scheme": "hybrid-ed25519-mldsa65-v1",
  "content_hash": "<96 ký tự hex>",
  "entry": "ui.json",
  "capabilities": [
    {
      "name": "network",
      "scope": { "kind": "network", "hosts": ["example.com"] },
      "reason": "Tải một trang mẫu"
    }
  ],
  "actions": [
    { "id": "tai-trang", "effect": { "kind": "fetch", "host": "example.com", "path": "/" } }
  ]
}
```

## Các trường

| Trường | Kiểu | Bắt buộc | |
|---|---|---|---|
| `spec_version` | chuỗi | PHẢI | Đúng `"0.1"`. Khác là từ chối, KHÔNG đoán. |
| `id` | chuỗi | PHẢI | Mã ứng dụng, xem dưới |
| `name` | chuỗi | PHẢI | Tên hiện cho người dùng |
| `version` | chuỗi | PHẢI | Phiên bản ứng dụng; tiêu chuẩn không quy định dạng |
| `publisher` | chuỗi | PHẢI | Khoá công khai, hex chữ thường |
| `scheme` | chuỗi | PHẢI | Đúng `"hybrid-ed25519-mldsa65-v1"` |
| `content_hash` | chuỗi | PHẢI | 96 ký tự hex, xem [01](01-goi.md) |
| `entry` | chuỗi | PHẢI | Đường dẫn trong `content/` |
| `capabilities` | mảng | PHẢI | Có thể rỗng |
| `actions` | mảng | không | Vắng mặt = ứng dụng chỉ hiện thông tin |

### Trường lạ

Bản cài đặt **PHẢI** từ chối bản kê khai có bất kỳ trường nào tiêu chuẩn này
không định nghĩa — ở gốc, trong mục xin quyền, và trong phạm vi quyền. Mã lỗi
`bad-json`.

Lý do không phải là gọn gàng. **Chữ ký phủ lên từng byte của `manifest.json`, kể
cả byte mà không luật nào của tiêu chuẩn đọc tới.** Một trường không ai kiểm là
một kênh mang ý nghĩa NGOÀI tiêu chuẩn: cùng một gói đã ký, bản cài đặt hiểu
`x-acme-tu-chay` làm một đằng, bản không hiểu làm một nẻo. Đó đúng là cách các
tiền tố riêng của hãng phá vỡ tính liên thông của web — và chữ ký còn làm nó tệ
hơn, vì hành vi lệch nhau lại trông như chính chủ.

Trong phạm vi quyền thì còn nặng hơn. Một phạm vi ghi
`{"kind":"network","hosts":["a.com"],"ports":[443]}` cấp cổng 443 trên bản cài
đặt biết `ports`, và cấp **mọi cổng** trên bản bỏ qua nó. Im lặng bỏ một trường
chỉ có thể NỚI quyền ra, không bao giờ thu hẹp lại.

Nên: thứ được ký đúng bằng thứ được kiểm. Muốn thêm trường thì phải có
`spec_version` mới — đó là lý do `spec_version` phải khớp chính xác, không bao
giờ đoán. Xem [`VERSIONING.md`](../../VERSIONING.md).

## `id` — mã ứng dụng

Kiểu tên miền ngược. **PHẢI** thoả:

- 1–128 ký tự
- Ít nhất **hai đoạn** ngăn bằng `.` (nên `hello` không hợp lệ, `com.tcc.hello` hợp lệ)
- Đoạn không rỗng
- Chỉ **chữ thường ASCII, chữ số, và `.`**

**Vì sao chặt thế:** mã lỏng lẻo mở đường cho mã trông na ná nhau. `com.tcc.vi`
và `com.TCC.vi` là **hai danh tính trông y hệt nhau** với người đọc lướt, mà kho
quyền lại coi là hai ứng dụng khác nhau — hoặc tệ hơn, coi là một.

⚠️ **Cạm bẫy đã dẫm:** nhiều thư viện JSON cho phép giải mã thẳng vào một kiểu
bọc chuỗi mà **không gọi hàm kiểm**. Bản cài đặt **PHẢI** kiểm `id` ở bước kiểm
hình dạng, không tin vào tầng giải mã. Mã lỗi: `bad-app-id`.

## Chuỗi hiện ra người dùng

`name`, `version`, và `reason` của từng quyền hiện lên màn hình quyết định bảo
mật. Chúng **PHẢI** qua phép kiểm sau — **KHÔNG ĐƯỢC** chứa:

| Loại | Ví dụ | Vì sao |
|---|---|---|
| Xuống dòng, tab | `\n` `\r` `\t` | vỡ bố cục hộp thoại một dòng |
| Điều khiển C0/C1 | `U+0000`–`U+001F`, `U+007F`–`U+009F` | |
| Đảo chiều chữ hai chiều | `U+202A`–`U+202E`, `U+2066`–`U+2069`, `U+200E`, `U+200F` | `"app-evil.exe"` hiện thành `"app-exe.live"` |
| Rộng bằng không | `U+200B`–`U+200D`, `U+FEFF`, `U+2060` | hai chuỗi khác nhau trông y hệt |
| Rỗng hoặc toàn khoảng trắng | | |

**Dấu kết hợp:** tối đa **8 dấu liên tiếp** trên một chữ cái.

Không cấm hẳn được — tiếng Việt sống bằng dấu kết hợp (`ỡ` = `o` + móc + ngã = 2
dấu). Nhưng không có trần thì 500 dấu chồng lên một chữ vẽ ra một **vệt dọc trùm
lên phần màn hình bên trên** — mà trong hộp thoại hỏi quyền, phần bên trên chính
là câu cảnh báo người dùng phải đọc.

| | Dấu tối đa trên một chữ |
|---|---|
| Tiếng Việt | 2 |
| Thái, Devanagari — cụm nặng nhất | ~4–6 |
| **Trần của tiêu chuẩn** | **8** |
| UAX #15 cho trao đổi dữ liệu | 30 |

UAX #15 nới tới 30 vì nó lo việc **trao đổi**; ta lo việc **hiển thị** trên màn
hình quyết định bảo mật.

Mã lỗi cho mọi trường hợp trên: `unsafe-display-string`.

## `entry`

Đường dẫn trong `content/`, chịu đúng ràng buộc ở [01](01-goi.md). Tệp **PHẢI**
tồn tại thật trong gói — kiểm ngay sau chữ ký, trước khi làm phiền người dùng.

Nội dung của nó là cây giao diện, xem [05](05-giao-dien.md).

Mã lỗi: `bad-entry` (đường dẫn hỏng), `missing-entry` (không có tệp đó).
