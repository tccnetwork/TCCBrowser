# 01 — Gói

## Bố cục trên đĩa

```text
<thư-mục-gói>/
├── manifest.json      ← bản kê khai
├── signature.hex      ← chữ ký, dạng hex chữ thường, có thể có xuống dòng cuối
└── content/           ← MỌI thứ trong đây đi vào băm nội dung
```

Ba mục này **PHẢI** có. Thiếu bất kỳ mục nào là gói không hợp lệ.

Mọi thứ **ngoài** ba mục đó **KHÔNG ĐƯỢC** đi vào chữ ký, và bản cài đặt **KHÔNG
ĐƯỢC** đọc chúng khi chạy ứng dụng.

## Bản 0.1 KHÔNG có định dạng đóng gói

Thứ mục trên định nghĩa là một **thư mục**. Bản 0.1 không định nghĩa dạng nén
nào, không định nghĩa tệp gói một-tệp nào, và không có định dạng `.tccapp` nào.
Một gói là một thư mục bố trí như trên, hết.

Nói to điều này vì cụm "gói `.tccapp`" xuất hiện trong tài liệu dự án, mà nó gọi
tên một thứ chưa tồn tại. Một bản cài đặt vẫn tuân thủ mà không đọc một tệp nén
nào.

**Vì sao không định nghĩa ở đây.** Luật 1 của [`spec/README.md`](../../README.md):
tiêu chuẩn rút ra từ mã đã chạy, không viết trước. Chưa có định dạng đóng gói
nào được cài đặt, nên viết ra bây giờ là đoán.

**Định dạng đóng gói tương lai phải trả lời được những điều sau**, để người làm
nó không bắt đầu từ con số không:

| Vấn đề | Vì sao nó không phải chuyện hình thức |
|---|---|
| Nó được phân tích **TRƯỚC** khi kiểm chữ ký | Bản kê khai và chữ ký đều nằm bên trong, nên bộ phân tích đóng gói hứng dữ liệu hoàn toàn chưa xác thực — đúng vị trí `serde_json` đang đứng hôm nay, nhưng bề mặt tấn công lớn hơn nhiều |
| Thoát thư mục lúc giải nén | Lỗi kinh điển của mọi định dạng nén: một mục tên `../../etc/passwd`. Luật đường dẫn bên dưới phải áp cho mục trong tệp nén, và phải áp **trước** khi ghi bất cứ thứ gì ra đĩa |
| Tỷ lệ nén | Tệp nén mời gọi bom nén. Trần 256 MiB bên dưới là trần của nội dung ĐÃ GIẢI NÉN, và phải cưỡng chế trong lúc giải nén chứ không phải sau |
| Mục trùng tên | Định dạng nén thường cho phép hai mục cùng tên. Bên đọc lấy cái đầu, bên kia lấy cái cuối — một chữ ký, hai gói |
| Mục ngoài ba tên đã biết | Phải TỪ CHỐI, không được bỏ qua, đúng lý lẽ về trường lạ ở [02](02-ban-ke-khai.md) |
| Không được bắt buộc giải nén | Kiểm chữ ký phải làm được bằng cách đọc thẳng, không ghi gì ra đĩa |

Chừng nào chưa có, "dựng một gói hợp lệ" nghĩa là dựng đúng thư mục tả ở trên.

## Đường dẫn trong `content/`

Đường dẫn tính **tương đối từ `content/`**, dùng `/` làm dấu phân cách trên mọi
hệ điều hành.

Một đường dẫn hợp lệ **PHẢI** thoả hết:

| Luật | Vì sao |
|---|---|
| Không rỗng, tối đa **1024** ký tự | |
| KHÔNG chứa `..` | đi ra ngoài gói |
| KHÔNG bắt đầu bằng `/` | đường dẫn tuyệt đối |
| KHÔNG chứa `\` | Windows coi nó là dấu phân cách |
| KHÔNG chứa `:` | Windows coi nó là tên ổ đĩa / luồng dữ liệu phụ |
| KHÔNG chứa `//` | hai cách viết cho cùng một tệp |
| KHÔNG kết thúc bằng `/` | đó là thư mục, không phải tệp |
| KHÔNG phải `.` | |
| KHÔNG chứa ký tự điều khiển | |

**Trùng lặp:** hai tệp **KHÔNG ĐƯỢC** có cùng đường dẫn, kể cả khi chỉ **khác
hoa thường**. macOS và Windows coi `Logo.png` và `logo.png` là một tệp — cùng
một chữ ký sẽ cho hai kết quả khác nhau trên hai máy. Mã lỗi: `case-collision`.

**Liên kết mềm:** `content/` **KHÔNG ĐƯỢC** chứa liên kết mềm. Cái được ký là
liên kết, cái được đọc là tệp đích — hai thứ khác nhau.

**Trần kích thước:** tổng nội dung tối đa **256 MiB**. Không có trần thì dựng
dạng chuẩn tắc sẽ ngốn hết bộ nhớ trước khi kiểm được gì.

## Dạng chuẩn tắc

Đây là chuỗi byte mà băm nội dung tính lên. Hai bản cài đặt **PHẢI** dựng ra
**cùng một chuỗi byte** — lệch một byte là chữ ký của bên này bên kia không kiểm
được, mà cả hai đều tưởng mình đúng.

Sắp mọi tệp theo **thứ tự byte tăng dần của đường dẫn** (so byte thô của UTF-8,
KHÔNG dùng thứ tự theo ngôn ngữ). Rồi với mỗi tệp, nối vào:

```text
u64 độ dài đường dẫn (big-endian, 8 byte)
đường dẫn        (UTF-8, không có byte kết thúc)
u64 độ dài nội dung (big-endian, 8 byte)
nội dung         (byte thô)
```

### Vì sao PHẢI có tiền tố độ dài

Không có tiền tố thì hai cây khác nhau cho ra cùng chuỗi byte:

| Cây | Nối trơn |
|---|---|
| `{"ab": "c"}` | `abc` |
| `{"a": "bc"}` | `abc` |

Hai gói khác nhau, cùng một băm, cùng một chữ ký. Kẻ gian đổi ruột mà chữ ký vẫn
đạt. Tiền tố độ dài là thứ duy nhất chặn nó.

### Cây rỗng

Cây không có tệp nào cho ra chuỗi byte **rỗng**. Băm của nó là băm của chuỗi
rỗng — xem vector `canonical`, trường hợp "cây rỗng".

## Băm nội dung

**BLAKE3** ở chế độ XOF, lấy **48 byte đầu**, viết **hex chữ thường** (96 ký tự).

```text
content_hash = hex_thường( BLAKE3_XOF( dạng_chuẩn_tắc )[0..48] )
```

Kiểm chứng nhanh: cây rỗng **PHẢI** cho
`af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262…`
(32 byte đầu là KAT công khai của BLAKE3 cho chuỗi rỗng).

**Vì sao BLAKE3 chứ không SHA-2:** để đồng bộ với chuỗi TCC, vốn đã dùng BLAKE3.
Hai hàm băm trong một hệ sinh thái là hai chỗ để sai.

**48 byte chứ không 32:** dư mức an toàn trước Grover, và khớp độ dài SHA-384 nên
đổi hàm băm sau này không phải đổi độ dài trường.

## Kiểm gói — THỨ TỰ LÀ MỘT TÍNH CHẤT BẢO MẬT

```text
1. Đọc ba mục          → mới chỉ là byte, chưa tin gì
2. Kiểm CHỮ KÝ         → chưa qua bước này thì KHÔNG có gì trong bản kê khai đáng tin
3. Kiểm điểm vào tồn tại
4. Hỏi người dùng      → tên và lý do hiện lên đã được chữ ký bảo chứng
5. Cấp quyền
```

Bước 4 **KHÔNG ĐƯỢC** đứng trước bước 2. Hỏi trước khi kiểm nghĩa là hộp thoại
hiện tên và lý do lấy từ một bản kê khai **chưa xác thực** — kẻ gian viết gì
người dùng đọc nấy.
