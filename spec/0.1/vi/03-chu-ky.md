# 03 — Chữ ký

Bộ ký duy nhất của phiên bản 0.1: **`hybrid-ed25519-mldsa65-v1`**.

## Vì sao LAI, không chỉ hậu lượng tử

Mật mã hậu lượng tử còn trẻ. SIKE — một ứng viên vào tới vòng bốn của NIST — bị
phá trong **khoảng một giờ trên một lõi CPU** năm 2022, bằng toán học cổ điển.

Chữ ký lai nghĩa là kẻ tấn công phải phá **CẢ HAI**: Ed25519 (đã đứng vững hàng
chục năm, nhưng máy tính lượng tử đủ lớn sẽ hạ) **và** ML-DSA-65 (chống lượng tử,
nhưng còn mới). Một trong hai gãy thì chữ ký vẫn đứng.

## Bố cục byte — LÀ MỘT PHẦN CỦA TIÊU CHUẨN

Không phải chi tiết cài đặt. Đảo thứ tự hai nửa là một gói khác hẳn.

| | Bố cục | Tổng |
|---|---|---|
| Khoá bí mật | `[ hạt giống Ed25519: 32B ][ hạt giống ML-DSA-65: 32B ]` | **64 B** |
| Khoá công khai | `[ Ed25519: 32B ][ ML-DSA-65: 1952B ]` | **1984 B** |
| Chữ ký | `[ Ed25519: 64B ][ ML-DSA-65: 3309B ]` | **3373 B** |

`publisher` trong bản kê khai là khoá công khai viết hex chữ thường → **3968 ký tự**.

**Khoá bí mật giữ dạng HẠT GIỐNG**, không phải khoá đã bung. Ngắn hơn nhiều, và
bung lại được bất cứ lúc nào theo FIPS 204. Lưu ý: đây là chỗ khác với ACVP của
NIST, vốn xuất khoá bí mật ML-DSA ở dạng đã bung 4032 byte. Cả hai đều hợp
FIPS 204 — chỉ là hai cách biểu diễn.

## Ký

```text
ký(khoá_bí_mật, thông_điệp) =
      Ed25519.Sign( hạt_giống[0..32],  thông_điệp )      64B
   ‖  ML-DSA.Sign(  hạt_giống[32..64], thông_điệp )    3309B
```

Thông điệp là **đúng chuỗi byte của `manifest.json`** như nó nằm trên đĩa. Đọc
JSON rồi tuần tự hoá lại là làm hỏng chữ ký — khoảng trắng và thứ tự khoá đều đổi.

### ⚠️ Giao diện FIPS 204: NGOÀI, ngữ cảnh RỖNG

FIPS 204 có **hai** giao diện ký:

| | Ký cái gì |
|---|---|
| **Ngoài** (`ML-DSA.Sign`) | `M' = 0x00 ‖ len(ctx) ‖ ctx ‖ M` |
| **Trong** (`ML-DSA.Sign_internal`) | thẳng `M` |

**Tiêu chuẩn TCC dùng giao diện NGOÀI với `ctx` RỖNG.** Nên thông điệp thật sự
được ký là `0x00 0x00 ‖ manifest_bytes`.

Đây là câu quan trọng nhất trang này. Một bản cài đặt dùng nhầm giao diện sẽ sinh
ra chữ ký mà bên kia **không kiểm được** — mà cả hai bên đều "đúng FIPS 204". Bẫy
interop im lặng nhất của cả tiêu chuẩn.

Đo được bằng vector sigVer của NIST: nhóm `external` khớp, nhóm `internal` lệch.

### Ký là TẤT ĐỊNH

Cả hai nửa. Ed25519 tất định theo RFC 8032; ML-DSA dùng biến thể tất định của
FIPS 204 (`rnd = 0^256`).

Nghĩa là **cùng khoá + cùng thông điệp luôn cho cùng chữ ký từng byte**. Vector
`signature` dựa vào tính chất này, và nó cũng làm việc dựng lại gói tái lập được.

## Kiểm

```text
kiểm(khoá_công_khai, thông_điệp, chữ_ký):
    Ed25519.Verify( kc[0..32],   thông_điệp, ck[0..64]    )  PHẢI đạt
    ML-DSA.Verify(  kc[32..1984], thông_điệp, ck[64..3373] )  PHẢI đạt
```

**CẢ HAI PHẢI đạt.** Không có đường tắt trả `Ok` sớm khi một nửa đạt — làm thế
là chữ ký lai mất sạch ý nghĩa và ta chỉ còn Ed25519.

Độ dài sai (thiếu hoặc thừa byte) là **lỗi**, không phải "cắt bớt rồi kiểm".

## Kiểm gói

Chữ ký ký lên bản kê khai. Nội dung buộc vào bản kê khai qua `content_hash`. Nên
kiểm gói là **hai** bước, và thiếu bước nào cũng thủng:

```text
1. kiểm(publisher, manifest_bytes, signature)      → bản kê khai không bị sửa
2. content_hash == băm(dạng_chuẩn_tắc(content/))   → ruột không bị thay
```

Chỉ bước 1 thì thay cả thư mục `content/` mà chữ ký vẫn đạt.

## Mốc ngoài

Bản cài đặt **NÊN** tự kiểm bằng mốc ngoài, đừng chỉ so với chính mình:

| Nửa | Mốc |
|---|---|
| Ed25519 | RFC 8032 §7.1 TEST 1: hạt giống `9d61b1…7f60` → khoá công khai `d75a98…511a` |
| ML-DSA-65 | NIST ACVP `ML-DSA-keyGen-FIPS204`, bộ tham số ML-DSA-65 |

Vector ACVP đã trích sẵn 25 ca vào `conformance/vectors/acvp-mldsa65.json`.

Chiều **ký** thì ACVP không neo được (nó xuất khoá đã bung, xem trên). Bản cài
đặt **NÊN** đối chiếu với một bản cài đặt FIPS 204 độc lập thứ hai — và **PHẢI**
kiểm bản thứ hai đó bằng vector NIST trước, nếu không nó chỉ là ý kiến thứ hai:
hai bản cùng sai theo một kiểu vẫn khớp nhau.
