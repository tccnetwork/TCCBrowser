# 06 — Mã lỗi

## Vì sao cần mã, khi đã có thông báo

Thông báo lỗi là **văn xuôi cho người đọc**. Nó được phép sửa cho dễ hiểu hơn bất
cứ lúc nào, và nó được phép dịch. Không có gì so khớp được với nó.

Mã lỗi thì **ổn định**. Bộ kiểm định tuân thủ và mọi bản triển khai so khớp bằng
nó. **Đổi một mã là đổi tiêu chuẩn**, phải tăng phiên bản.

## Luật

Bản cài đặt tuân thủ **PHẢI** dùng đúng các mã dưới đây khi từ chối.

Lỗi bọc lỗi **PHẢI** trả mã của **nguyên nhân gốc**. Một bản kê khai hỏng vì ký
tự giả mạo phải ra `unsafe-display-string` — trả một mã chung như `spec` thì
không nói lên điều gì và bộ kiểm định không so khớp được.

## Danh sách

### Gói và đường dẫn

| Mã | Khi nào |
|---|---|
| `empty-path` | Đường dẫn rỗng |
| `bad-path` | Vi phạm ràng buộc đường dẫn ở [01](01-goi.md) |
| `duplicate-path` | Hai tệp cùng đường dẫn |
| `case-collision` | Hai tệp chỉ khác hoa thường |

### Bản kê khai

| Mã | Khi nào |
|---|---|
| `bad-json` | JSON hỏng, hoặc khoá trùng lặp |
| `manifest-too-large` | Vượt 64 KiB |
| `unsupported-spec-version` | `spec_version` khác `"0.1"` |
| `bad-app-id` | `id` vi phạm ràng buộc ở [02](02-ban-ke-khai.md) |
| `unsafe-display-string` | Chuỗi hiện ra người dùng chứa ký tự cấm, hoặc quá nhiều dấu chồng |
| `not-hex` | Trường phải là hex mà không phải |
| `bad-hex-length` | Chuỗi hex sai độ dài |
| `publisher-not-hex` | `publisher` không phải hex |
| `scheme-mismatch` | `scheme` không phải bộ ký đang dùng |
| `content-hash-mismatch` | Băm nội dung không khớp bản kê khai |
| `bad-entry` | `entry` là đường dẫn không hợp lệ |
| `missing-entry` | `entry` không có trong gói |

### Quyền năng

| Mã | Khi nào |
|---|---|
| `unknown-capability` | `name` không thuộc `network`/`storage`/`wallet` |
| `duplicate-capability` | Xin trùng một quyền |
| `missing-reason` | `reason` rỗng hoặc toàn khoảng trắng |
| `bad-scope` | `scope` sai kiểu, rỗng, có ký tự đại diện, hoặc tên máy chủ sai hình dạng |
| `non-ascii-host` | Tên máy chủ ngoài ASCII |

### Hành vi

| Mã | Khi nào |
|---|---|
| `bad-action-id` | Mã hành động vi phạm ràng buộc |
| `duplicate-action` | Khai trùng một hành động |
| `action-host-not-granted` | Hành vi gọi máy chủ không nằm trong quyền đã xin |

### Giao diện

| Mã | Khi nào |
|---|---|
| `external-image` | `source` của ảnh trỏ ra mạng |
| `text-too-long` | Chuỗi vượt 4 096 ký tự |
| `too-deep` | Cây vượt 64 tầng |
| `too-many-nodes` | Cây vượt 10 000 nút |
| `not-a-container` | Nút lá nhận nút con |

### Mật mã

| Mã | Khi nào |
|---|---|
| `bad-signature` | Một trong hai nửa chữ ký không hợp lệ |
| `bad-length` | Khoá hoặc chữ ký sai độ dài |
| `bad-key` | Khoá không dùng được |

## Điều mã lỗi KHÔNG nói

Mã cho biết **vì sao từ chối**, không cho biết **nửa nào của chữ ký hỏng**, cũng
không cho biết **tệp nào có trong gói**. Bản cài đặt **KHÔNG NÊN** trả thông tin
chi tiết hơn mức cần cho người viết ứng dụng sửa lỗi — với một gói đang bị dò,
mỗi chi tiết là một manh mối.
