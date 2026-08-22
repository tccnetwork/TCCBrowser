#!/usr/bin/env bash
# Cưỡng chế các luật phụ thuộc bằng MÁY, không bằng lời hứa.
#
# Luật viết trong chú thích thì sớm muộn cũng bị vi phạm — thường là vào lúc
# 11 giờ đêm khi ai đó chỉ muốn "cho nó chạy đã". Tệp này chặn việc đó.
#
# Chạy: tools/kiem-luat-phu-thuoc.sh    (và trong CI)

set -uo pipefail
cd "$(dirname "$0")/.." || exit 1

loi=0
bao() { echo "❌ $1"; loi=$((loi + 1)); }
dat() { echo "✅ $1"; }

# Trả về danh sách phụ thuộc nội bộ (tcc-*) của một crate
phu_thuoc() { grep -oE '^tcc-[a-z-]+' "$1/Cargo.toml" 2>/dev/null | sort -u; }

echo "--- Luật 1: tcc-ui KHÔNG được biết bất kỳ bộ dựng nào ---"
# Đây là luật quan trọng nhất dự án. Vi phạm nó là mất đường thoát khỏi WebView:
# ứng dụng sẽ nhìn thấy DOM, và ngày có bộ dựng riêng thì mọi ứng dụng phải viết lại.
if phu_thuoc crates/tcc-ui | grep -q 'tcc-render'; then
  bao "tcc-ui đang phụ thuộc một bộ dựng — giàn giáo đã hoá thành nhà"
else
  dat "tcc-ui sạch, không biết bộ dựng nào"
fi

echo
echo "--- Luật 2: chỉ tcc-shell được phụ thuộc tcc-render-* ---"
# tcc-shell là điểm lắp ráp duy nhất được chọn bộ dựng cụ thể.
vi_pham=0
for c in crates/*/ tools/*/ apps/*/; do
  ten=$(basename "$c")
  [ "$ten" = "tcc-shell" ] && continue
  case "$ten" in tcc-render-*) continue ;; esac
  if phu_thuoc "$c" | grep -q 'tcc-render'; then
    bao "$ten phụ thuộc tcc-render-* — chỉ tcc-shell được phép"
    vi_pham=1
  fi
done
[ "$vi_pham" = "0" ] && dat "chỉ tcc-shell chạm tới bộ dựng"

echo
echo "--- Luật 3: tcc-crypto phải là lá, không phụ thuộc crate nội bộ nào ---"
# Biên giới tin cậy. Mỗi phụ thuộc thêm vào đây là mở rộng bề mặt cần kiểm định.
n=$(phu_thuoc crates/tcc-crypto | grep -c . || true)
if [ "$n" != "0" ]; then
  bao "tcc-crypto phụ thuộc $n crate nội bộ — biên giới tin cậy đang phình ra:"
  phu_thuoc crates/tcc-crypto | sed 's/^/      /'
else
  dat "tcc-crypto vẫn là lá"
fi

echo
echo "--- Luật 4: tcc-spec phải là lá ---"
# Người ngoài tự cài đặt tiêu chuẩn chỉ cần đọc crate này.
n=$(phu_thuoc crates/tcc-spec | grep -c . || true)
if [ "$n" != "0" ]; then
  bao "tcc-spec phụ thuộc crate nội bộ — người ngoài sẽ phải kéo theo cả trình duyệt"
else
  dat "tcc-spec vẫn là lá"
fi

echo
echo "--- Luật 5: tcc-runtime KHÔNG được biết bộ dựng ---"
if phu_thuoc crates/tcc-runtime | grep -q 'tcc-render'; then
  bao "tcc-runtime phụ thuộc bộ dựng — nó chỉ được nói chuyện với tcc-ui"
else
  dat "tcc-runtime sạch"
fi

echo
echo "--- Luật 6: API ứng dụng KHÔNG được lộ ra DOM/HTML/CSS ---"
# Lộ ra là ứng dụng dính chặt vào WebView.
# Bỏ phần "tệp:dòng:" mà grep -rn thêm vào TRƯỚC khi lọc chú thích — nếu không
# thì chính chú thích giải thích luật này lại bị báo là vi phạm.
ro_ri=$(grep -rniE '\b(dom|innerhtml|queryselector|cssstyledeclaration)\b' \
     crates/tcc-ui/src crates/tcc-spec/src 2>/dev/null \
     | sed -E 's/^[^:]+:[0-9]+://' | grep -vE '^\s*(//|\*)' || true)
if [ -n "$ro_ri" ]; then
  bao "tcc-ui hoặc tcc-spec nhắc tới DOM/HTML/CSS ngoài chú thích:"
  printf '%s\n' "$ro_ri" | sed 's/^/      /'
else
  dat "không có rò rỉ DOM/HTML/CSS"
fi

echo
echo "--- Luật 8: chỉ tcc-shell được phụ thuộc tcc-net ---"
# Đường ra khỏi máy phải NHÌN THẤY ĐƯỢC trong cây phụ thuộc. `tcc-runtime` không
# phụ thuộc tcc-net nghĩa là bộ nạp ứng dụng không tự mở socket được — nó chỉ
# gọi qua `trait Mang` tiêm từ ngoài vào.
vi_pham=0
for c in crates/*/ tools/*/ apps/*/; do
  ten=$(basename "$c")
  [ "$ten" = "tcc-shell" ] && continue
  [ "$ten" = "tcc-net" ] && continue
  if phu_thuoc "$c" | grep -q 'tcc-net'; then
    bao "$ten phụ thuộc tcc-net — chỉ tcc-shell được phép mở đường ra ngoài"
    vi_pham=1
  fi
done
[ "$vi_pham" = "0" ] && dat "chỉ tcc-shell chạm tới đường ra ngoài"

echo
echo "--- Luật 11: bản dịch đặc tả không được TRÔI khỏi bản chuẩn ---"
# Bản tiếng Anh là bản CHUẨN; tiếng Việt là bản dịch. Hai bản lệch nhau thì tệ
# hơn chỉ có một bản: người đọc bản dịch cài đặt theo một tiêu chuẩn khác mà
# không ai biết. Kiểm hai thứ đo được: số tệp bằng nhau, và tập MÃ LỖI y hệt.
if [ -d spec/0.1/vi ]; then
  n_en=$(ls spec/0.1/*.md 2>/dev/null | wc -l | tr -d " ")
  n_vi=$(ls spec/0.1/vi/*.md 2>/dev/null | wc -l | tr -d " ")
  if [ "$n_en" != "$n_vi" ]; then
    bao "bản chuẩn có $n_en tệp, bản dịch có $n_vi — một mục đã bị bỏ quên"
  else
    ma_en=$(grep -oE "^\| \`[a-z][a-z0-9-]+\` \|" spec/0.1/06-error-codes.md | tr -d "|\` " | sort)
    ma_vi=$(grep -oE "^\| \`[a-z][a-z0-9-]+\` \|" spec/0.1/vi/06-ma-loi.md | tr -d "|\` " | sort)
    if [ "$ma_en" != "$ma_vi" ]; then
      bao "tập mã lỗi hai bản KHÁC nhau:"
      diff <(printf "%s\n" "$ma_en") <(printf "%s\n" "$ma_vi") | sed "s/^/      /" | head -8
    else
      dat "$n_en tệp mỗi bản, $(printf "%s\n" "$ma_en" | wc -l | tr -d " ") mã lỗi khớp nhau"
    fi
  fi
else
  bao "thiếu bản dịch spec/0.1/vi/"
fi
# Tài liệu chính sách ở tầng spec/ (VERSIONING, GOVERNANCE) cũng phải có bản dịch.
# README.md của spec/ là mục lục hướng vào kho, không phải văn bản tiêu chuẩn, nên
# không đòi bản dịch.
thieu=""
for f in spec/*.md; do
  ten=$(basename "$f")
  [ "$ten" = "README.md" ] && continue
  [ -f "spec/vi/$ten" ] || thieu="$thieu $ten"
done
if [ -n "$thieu" ]; then
  bao "tài liệu chính sách chưa có bản dịch spec/vi/:$thieu"
else
  dat "tài liệu chính sách tầng spec/ đều có bản dịch"
fi

echo
echo "--- Luật 18: chỉ tcc-shell được phụ thuộc tcc-keystore và tcc-chain ---"
# Cùng hình dạng với luật 8, và cùng lý do: ĐỌC `Cargo.toml` LÀ BIẾT NGAY bộ nạp
# ứng dụng không với tới được khoá ví. Nếu `tcc-runtime` phụ thuộc kho khoá thì
# đường từ mã ứng dụng tới khoá bí mật chỉ còn là chuyện ai gọi hàm nào — mà đó
# là thứ phải soi từng dòng mới thấy.
#
# Kho khoá GIẢ cũng nằm trong crate này, nên luật còn chặn luôn việc ai đó tiện
# tay dùng bản giả ở đường chạy thật.
# `tcc-chain` cùng hạng: nó biết bố cục giao dịch của chuỗi, và bộ nạp ứng dụng
# không có việc gì với thứ đó.
lac=$(grep -lE 'tcc-keystore|tcc-chain' crates/*/Cargo.toml apps/*/Cargo.toml tools/*/Cargo.toml 2>/dev/null \
  | grep -v 'crates/tcc-shell/Cargo.toml' | grep -v 'crates/tcc-keystore/Cargo.toml' \
  | grep -v 'crates/tcc-chain/Cargo.toml' || true)
if [ -n "$lac" ]; then
  bao "crate KHÔNG phải tcc-shell mà phụ thuộc tcc-keystore/tcc-chain:"
  printf '%s\n' "$lac" | sed 's/^/      /'
else
  dat "chỉ tcc-shell được với tới kho khoá ví và bố cục giao dịch chuỗi"
fi

echo
echo "--- Luật 20: KHÔNG khai quyền thiết bị trong gói macOS ---"
# `wry` 0.52.1 viết CỨNG `WKPermissionDecision::Grant` cho yêu cầu micro/camera
# của trang web (`wkwebview/class/wry_web_view_ui_delegate.rs:74`) và không cho
# ghi đè. Ở tầng 2 — mở trang web bất kỳ — nghĩa là một trang gọi
# `getUserMedia()` được cấp mà KHÔNG ai hỏi người dùng.
#
# Chắn duy nhất còn lại là tầng hệ điều hành: thiếu chuỗi mô tả mục đích thì
# macOS từ chối, nên lời "Grant" của wry không có gì để cấp. Thêm một dòng
# `NS*UsageDescription` là gỡ chắn ấy — nên luật này canh.
#
# Ngày nào wry cho ghi đè quyết định ấy thì luật này mới nên nới, và phải nới
# kèm một hộp thoại hỏi quyền THẬT, không nới trơn.
# Soi KHAI BÁO thật (`<key>NS…UsageDescription</key>`), không soi mọi chỗ nhắc
# tên. Bản đầu soi chuỗi trơn và nó tự tố cáo chính kịch bản đóng gói — chỗ
# đang GIẢI THÍCH vì sao không được khai.
lo_quyen=$(grep -rlE "<key>NS[A-Za-z]*UsageDescription</key>" tools/ apps/ 2>/dev/null || true)
if [ -n "$lo_quyen" ]; then
  bao "khai quyền thiết bị trong gói macOS:$(printf ' %s' $lo_quyen)"
else
  dat "gói macOS không khai quyền thiết bị nào — wry không tự cấp được micro/camera"
fi

echo
echo "--- Luật 17: số luật ghi trong tài liệu phải khớp số luật THẬT ---"
# Con số này trôi nhiều nhất và im lặng nhất: tài liệu ghi 6, rồi 10, rồi 12,
# trong khi kịch bản đã có 16. Người đọc tin con số, không đếm lại.
#
# Chỉ kiểm SỐ LUẬT ở đây. Số phép thử VÀ số vector đều nằm ở `kiem-so-lieu.sh`,
# chạy trong CI sau bước dựng.
#
# Tôi đã thử canh số vector ngay tại đây bằng cách đếm `cases` trong JSON. Đếm
# ra 122 trong khi bộ kiểm định báo 138 — vì `signature.json` không dùng khoá
# `cases`, và nhóm ACVP chạy thêm một phép ngoài danh sách. Một phép đếm gần
# đúng còn tệ hơn không đếm: nó báo động giả rồi bị người ta tắt đi.
#
# Và quan trọng hơn — số trong VĂN KỂ là sự thật lịch sử ("211 phép
# thử mù hoàn toàn"), sửa nó là bóp méo hồ sơ, nên cả ba luật chỉ soi những
# tài liệu và cụm từ nêu đích danh.
that=$(grep -c '^echo "--- Luật' "$0")
lech=""
for f in README.md SECURITY.md ARCHITECTURE.md CLAUDE.md docs/ke-hoach.md docs/dang-lam-gi.md docs/AUDIT.md; do
  [ -f "$f" ] || continue
  for n in $(grep -ohE '[0-9]+ (luật kiến trúc|luật cứng|architecture rules)' "$f" | grep -oE '^[0-9]+' | sort -u); do
    [ "$n" = "$that" ] || lech="$lech $(basename "$f"):$n"
  done
done
if [ -n "$lech" ]; then
  bao "tài liệu ghi sai số luật (thật là $that):$lech"
else
  dat "$that luật, và mọi tài liệu nhắc tới đều ghi đúng con số"
fi

echo
echo "--- Luật 16: mọi mã lỗi trong đặc tả phải có VECTOR ---"
# Luật 10 kiểm mã có TỒN TẠI trong mã nguồn. Nó không kiểm được mã đó có bao giờ
# NỔ hay không — và bốn mã trong danh sách hoá ra không chạm tới được: bộ đọc
# JSON, phép kiểm hình dạng, hoặc thư viện mật mã chặn trước. Mã không chạm tới
# được là chỗ hai bản cài đặt báo hai mã khác nhau cho cùng một gói.
#
# Viết một vector cho từng mã là cách DUY NHẤT phát hiện ra điều đó.
MIEN="duplicate-path"   # không diễn đạt được bằng JSON: một đối tượng không thể có hai khoá trùng
thieu=$(python3 - <<'PY2'
import json, pathlib, re
van = open("spec/0.1/06-error-codes.md").read()
# Bảng "ba mã đã bỏ" cũng là bảng, nên phải cắt trước khi quét — nếu không thì
# luật này đòi vector cho đúng những mã vừa bị bỏ vì không chạm tới được.
van = van.split("## Three codes were removed")[0]
doc = set(re.findall(r"^\| `([a-z][a-z0-9-]+)` \|", van, re.M))
vec = set()
for p in pathlib.Path("conformance/vectors").glob("*.json"):
    def quet(o):
        if isinstance(o, dict):
            if isinstance(o.get("code"), str):
                vec.add(o["code"])
            for v in o.values():
                quet(v)
        elif isinstance(o, list):
            for v in o:
                quet(v)
    quet(json.load(open(p)))
print(" ".join(sorted(doc - vec - {"duplicate-path"})))
PY2
)
if [ -n "$thieu" ]; then
  bao "mã lỗi trong đặc tả mà KHÔNG vector nào chạm tới:$thieu"
else
  dat "mọi mã lỗi trong đặc tả đều có vector (miễn trừ có ghi lý do: $MIEN)"
fi

echo
echo "--- Luật 15: vector kiểm định phải ĐỌC ĐƯỢC bởi người ngoài ---"
# Bộ vector là thứ DUY NHẤT phân xử được một tuyên bố tuân thủ, và người đọc nó
# là người viết bản cài đặt thứ hai — người không đọc tiếng Việt. Khoá của khung
# kiểm định bằng tiếng Việt là một rào chắn ngay tại cổng vào.
#
# Chỉ soi khoá của KHUNG, KHÔNG soi bên trong dữ liệu thử (`manifest`, `tree`,
# `files`): ở đó `tài-liệu/chào.txt` là một đường dẫn hợp lệ và `"Xin chào"` là
# chính thứ đang được đem ra thử. Dịch chúng đi là mất phép thử.
thieu=""
[ -f conformance/FORMAT.md ] || thieu="$thieu thiếu-conformance/FORMAT.md"
xau=$(python3 - <<'PY2'
import json, pathlib, re

vn = re.compile(r"[àáâãèéêìíòóôõùúỳăđĩũơưạảấầẩẫậắằẳẵặẹẻẽếềểễệỉịọỏốồổỗộớờởỡợụủứừửữựỵỷỹ]")
CU = {"mo_ta","truong_hop","ten","dat","ke_khai","cay","tep","cap","goi","cho_phep",
      "khoa","chu_ky","hat_giong","ghi_chu","luat","nguon","loai","vi_sao","bam",
      "bam_hex","chuan_tac_hex","thuat_toan","neo_ngoai","ky_hop_le","chu_ky_hong",
      "bi_mat_hex","cong_khai_hex","chu_ky_hex","thong_diep_hex","phai_dat"}
# Cây con là DỮ LIỆU THỬ: dừng soi ở đây.
DU_LIEU = {"manifest", "tree", "files"}

def quet(o, ten_tep):
    if isinstance(o, dict):
        for k, v in o.items():
            if k in CU or vn.search(k):
                print(f"{ten_tep}:{k}")
            if k not in DU_LIEU:
                quet(v, ten_tep)
    elif isinstance(o, list):
        for v in o:
            quet(v, ten_tep)

for p in sorted(pathlib.Path("conformance/vectors").glob("*.json")):
    quet(json.load(open(p)), p.name)
PY2
)
[ -n "$xau" ] && thieu="$thieu khoá-tiếng-Việt:$(printf '%s' "$xau" | tr '\n' ' ')"
if [ -n "$thieu" ]; then
  bao "vector không đọc được bởi người ngoài:$thieu"
else
  dat "$(ls conformance/vectors/*.json | wc -l | tr -d ' ') tệp vector dùng khoá tiếng Anh, có FORMAT.md"
fi

echo
echo "--- Luật 14: kho phải có giấy phép, và mọi crate phải khai đúng nó ---"
# Kho công khai KHÔNG có tệp giấy phép thì mặc định pháp lý là "giữ mọi quyền":
# người ngoài đọc được mà không được cài đặt lại. Điều đó mâu thuẫn thẳng với
# GOVERNANCE §3 — cổng ra của tiêu chuẩn là một người ngoài tự dựng gói.
#
# Kiểm cả hai vế, vì mỗi vế hỏng một kiểu: thiếu tệp LICENSE thì GitHub không
# nhận ra giấy phép nào; còn một crate khai lệch (hoặc quay lại "UNLICENSED")
# thì gói xuất bản mang điều khoản khác hẳn thứ kho tuyên bố.
GIAY_PHEP="Apache-2.0"
loi_gp=""
[ -f LICENSE ] || loi_gp="$loi_gp thiếu-tệp-LICENSE"
grep -qF "Version 2.0, January 2004" LICENSE 2>/dev/null || loi_gp="$loi_gp LICENSE-không-phải-Apache-2.0"
grep -qF "3. Grant of Patent License" LICENSE 2>/dev/null || loi_gp="$loi_gp LICENSE-thiếu-điều-khoản-sáng-chế"
lech=$(grep -h '^license' Cargo.toml 2>/dev/null | grep -v "\"$GIAY_PHEP\"" || true)
[ -n "$lech" ] && loi_gp="$loi_gp workspace-khai:$lech"
for f in crates/*/Cargo.toml tools/*/Cargo.toml apps/*/Cargo.toml; do
  grep -q 'license' "$f" || loi_gp="$loi_gp $f-không-khai-giấy-phép"
done
if [ -n "$loi_gp" ]; then
  bao "giấy phép:$loi_gp"
else
  dat "LICENSE là $GIAY_PHEP (có điều khoản sáng chế), $(ls crates/*/Cargo.toml tools/*/Cargo.toml apps/*/Cargo.toml | wc -l | tr -d ' ') crate đều khai giấy phép"
fi

echo
echo "--- Luật 13: định danh CÔNG KHAI không được mang tên tiếng Việt ---"
# ARCHITECTURE §7 nói định danh viết tiếng Anh, chú thích viết tiếng Việt. Luật
# đó đã TRÔI suốt nhiều tháng — vì nó là luật duy nhất không có máy canh. Đây là
# máy canh.
#
# Ranh giới cố ý đặt ở `pub`: đó là bề mặt người viết bản cài đặt thứ hai đọc.
# Tên hàm kiểm thử, biến cục bộ và tên ví dụ đối kháng vẫn tiếng Việt — chúng là
# LẬP LUẬN, không phải giao diện, và SECURITY.md trích dẫn tên phép thử làm bằng
# chứng nên đổi tên là làm hỏng đúng thứ nó ghi lại.
GOC_VIET='(chu|cau|vai|dau|mat|sac|thai|tra|luu|xoa|them|ky|bo|dung|mo|tao|vong|thoat|hoi|quyet|dinh|choi|cho|phep|bat|tat|khoa|nho|hanh|man|hinh|khoi|tan|cong|bang|kieu|thu|ghi|danh|phuc|cua|hop|quet|goi|kiem|bam|nut|loi|mang|nap|gui|tep|duong|noi|ten|nguoi|lieu|tro|nang|lietke|quen|van|tay|tinh|trang|nhieu|moi|tiep|doi|dot|bien|hat|giong|muc|tieu|sau|bot|cay|byte_doc|so|lan|phan|doan)'
vi_pham=$(grep -rhoE 'pub (struct|enum|fn|const|trait|type|mod) [A-Za-z_][A-Za-z0-9_]*' \
            crates/*/src/*.rs apps/*/src/*.rs tools/*/src/*.rs 2>/dev/null \
          | awk '{print $3}' | sort -u \
          | grep -iE "^${GOC_VIET}(_|$)|_${GOC_VIET}(_|$)" || true)
if [ -n "$vi_pham" ]; then
  bao "định danh public mang tên tiếng Việt:"
  printf '%s\n' "$vi_pham" | sed 's/^/      /' | head -12
else
  dat "$(grep -rhoE 'pub (struct|enum|fn|const|trait|type|mod) [A-Za-z_][A-Za-z0-9_]*' crates/*/src/*.rs apps/*/src/*.rs tools/*/src/*.rs 2>/dev/null | wc -l | tr -d ' ') định danh public đều là tiếng Anh"
fi

echo
echo "--- Luật 12: đặc tả KHÔNG được có liên kết chết ---"
# Đặc tả là thứ người ngoài đọc để tự cài đặt. Một liên kết chết ở đó nghĩa là
# một luật trỏ tới hư không — và người đọc không có mã nguồn để đoán bù vào.
chet=$(python3 - <<'PY'
import re, pathlib
for f in sorted(pathlib.Path("spec").rglob("*.md")):
    for m in re.findall(r"\]\(([^)#:]+\.md)\)", f.read_text()):
        if not (f.parent / m).resolve().exists():
            print(f"{f} → {m}")
PY
)
if [ -n "$chet" ]; then
  bao "liên kết chết trong đặc tả:"
  printf '%s\n' "$chet" | sed 's/^/      /' | head -8
else
  dat "$(grep -rhoE '\]\([^)#:]+\.md\)' spec --include='*.md' | wc -l | tr -d ' ') liên kết trong đặc tả đều tới đích"
fi

echo
echo "--- Luật 10: mã lỗi trong đặc tả phải TỒN TẠI trong mã ---"
# Đặc tả nói bản cài đặt "PHẢI dùng đúng các mã này". Một mã viết trong đặc tả mà
# không có trong mã nguồn là một lời hứa không ai giữ — và người ngoài đọc đặc tả
# sẽ cài đặt theo nó rồi không bao giờ khớp bộ kiểm định.
if [ -f spec/0.1/06-error-codes.md ]; then
  ma_doc=$(grep -oE '^\| `[a-z][a-z0-9-]+` \|' spec/0.1/06-error-codes.md | tr -d '|` ')
  ma_code=$(grep -rhoE '"[a-z][a-z0-9-]+"' crates/*/src/*.rs 2>/dev/null | tr -d '"' | sort -u)
  thieu=""
  for m in $ma_doc; do
    printf '%s\n' "$ma_code" | grep -qx "$m" || thieu="$thieu $m"
  done
  if [ -n "$thieu" ]; then
    bao "mã lỗi có trong đặc tả nhưng KHÔNG có trong mã:$thieu"
  else
    dat "$(printf '%s\n' "$ma_doc" | wc -l | tr -d ' ') mã lỗi trong đặc tả đều tồn tại trong mã"
  fi
else
  bao "thiếu spec/0.1/06-error-codes.md"
fi

echo
echo "--- Luật 9: khoá demo KHÔNG được rời khỏi examples/ ---"
# `examples/khoa-vi-du-AI-CUNG-CO.hex` nằm ngay trong kho, ai cũng có. Ký một gói
# thật bằng nó nghĩa là bất kỳ ai cũng giả mạo được nhà phát hành đó — và khi có
# tầng ghim khoá, người dùng ghim nhầm nó là ghim vào một khoá công cộng.
# Đọc khoá demo TỪ CHÍNH gói ví dụ, không chép cứng vào đây.
#
# Hai lý do. Một: chép cứng thì ký lại gói ví dụ là hằng số này trôi, và luật
# lặng lẽ đi tìm một khoá không còn ai dùng. Hai: kịch bản chứa khoá thì chính
# nó bị bắt — bản đầu tôi viết thế và nó tự tố cáo mình.
CONG_DEMO=$(python3 -c "import json;print(json.load(open('examples/hello-tcc/manifest.json'))['publisher'])")
# Quét MỌI tệp, không chỉ manifest.json. Bản đầu chỉ quét bản kê khai, nên khi
# khoá demo được nhúng vào mã nguồn `tcc-cli` thì luật này không thấy gì cả.
#
# NGOẠI LỆ CÓ CHỦ Ý: `tools/tcc-cli/src/khoa-demo-cong-khai.txt` chứa đúng khoá
# ấy để `tcc sign` CẢNH BÁO khi ai đó ký bằng nó. Đó là danh sách chặn, ngược
# hẳn với việc dùng khoá — và nhúng lúc biên dịch từ gói ví dụ để hai bản không
# trôi khỏi nhau. Ai gỡ tệp đó đi là gỡ mất cảnh báo, nên nó phải TỒN TẠI.
# `--exclude-dir` chứ không lọc sau: `target/` là 4,6 GB, đọc hết rồi mới bỏ
# thì luật này treo. Bản đầu tôi viết đúng kiểu đó và nó chạy quá hai phút.
lac=$(grep -rl "$CONG_DEMO" . 2>/dev/null \
  --exclude-dir=.git --exclude-dir=target --exclude-dir=corpus \
  --exclude-dir=seeds --exclude-dir=examples \
  | grep -v '^./tools/tcc-cli/src/khoa-demo-cong-khai.txt' || true)
[ -f tools/tcc-cli/src/khoa-demo-cong-khai.txt ] || lac="$lac THIẾU-tệp-danh-sách-chặn-của-tcc-sign"
if [ -n "$lac" ]; then
  bao "khoá công khai demo xuất hiện ngoài examples/:"
  printf '%s\n' "$lac" | sed 's/^/      /'
else
  dat "khoá demo chỉ nằm trong examples/"
fi

echo
echo "--- Luật 7: mọi nhóm vector phải có mặt và chạy được ---"
# Bộ kiểm định tuân thủ là thứ biến đặc tả thành TIÊU CHUẨN. Thiếu một nhóm
# vector nghĩa là có phần của đặc tả không ai kiểm được — mà không kiểm được thì
# nó là lời hứa, không phải tiêu chuẩn.
thieu=""
for v in canonical signature acvp-mldsa65 manifest ui capability package verify; do
  [ -f "conformance/vectors/$v.json" ] || thieu="$thieu $v"
done
if [ -n "$thieu" ]; then
  bao "thiếu nhóm vector:$thieu"
else
  dat "đủ tám tệp vector, chín nhóm"
fi

echo
echo "--- Luật 23: yêu cầu của 0.1 không được dựa vào tài liệu ngoài 0.1 ---"
# `spec/0.1/README.md` hứa rằng mọi thứ mang tính quy phạm đều nằm trong thư
# mục 0.1, và các liên kết ra ngoài chỉ để tham khảo. Lời hứa ấy phải kiểm được,
# vì nếu mất thì mất theo một cách không ai thấy: `VERSIONING.md` nằm ngoài mọi
# thư mục có phiên bản nên KHÔNG bất biến — một yêu cầu của 0.1 tựa vào nó là
# một yêu cầu sửa được mà không cần tăng phiên bản, đúng cái hỏng `VERSIONING.md`
# §1 sinh ra để chặn.
#
# Kiểm theo ĐOẠN VĂN, không theo dòng: câu quy phạm và liên kết hiếm khi rơi
# đúng một dòng, và kiểm theo dòng thì luật này chỉ bắt được trường hợp dễ nhất.
#
# Miễn trừ đúng MỘT chỗ: liên kết tới `conformance/`. Bộ vector là bộ máy của
# tiêu chuẩn, không phải văn xuôi diễn giải nó — và `spec/0.1/README.md` nêu
# ràng buộc giữ nó khỏi trôi: một vector chỉ được kiểm một yêu cầu đã nêu sẵn
# trong 0.1. Chính luật này bắt ra chỗ ấy, và câu ràng buộc kia là câu trả lời.
ngoai=$(python3 - <<'PY2'
import re, pathlib

MOC = re.compile(r"\*\*(MUST|MUST NOT|SHALL|PHẢI|KHÔNG ĐƯỢC)\*\*")
RA_NGOAI = re.compile(r"\]\((\.\./[^)]*)\)")
xau = []
mien = []
for p in sorted(pathlib.Path("spec/0.1").rglob("*.md")):
    for i, doan in enumerate(p.read_text().split("\n\n")):
        if not MOC.search(doan):
            continue
        for dich in RA_NGOAI.findall(doan):
            if "conformance/" in dich:
                mien.append(f"{p}:đoạn{i + 1}")
                continue
            xau.append(f"{p}:đoạn{i + 1}→{dich}")
# Miễn trừ tự canh chính nó. Kiểm đột biến cho thấy: nới `conformance/` thành
# "mọi liên kết" thì luật vẫn BÁO ĐẠT trong khi một vi phạm thật vừa lọt qua —
# một miễn trừ rộng ra là một luật tắt dần mà không ai thấy.
#
# Canh bằng NGỮ NGHĨA chứ không bằng con số: chỗ duy nhất được viện tới bộ vector
# là mục "Tuân thủ" của README. Đếm cứng thì thêm liên kết vào bản dịch — một
# thay đổi tốt — cũng làm luật đỏ, mà một luật đỏ vì việc tốt là một luật người
# ta học cách bỏ qua.
for m in mien:
    if not m.split(":")[0].endswith("README.md"):
        xau.append(f"MIỄN-TRỪ-TRÔI({m})")
print(" ".join(xau))
PY2
)
if [ -n "$ngoai" ]; then
  bao "đoạn văn vừa nêu yêu cầu vừa tựa vào tài liệu ngoài 0.1:$ngoai"
else
  dat "mọi yêu cầu của 0.1 đều tự đứng trong 0.1"
fi

echo
echo "--- Luật 22: mã lỗi chỉ được GỌI TÊN ở nơi nó được ĐỊNH NGHĨA ---"
# Luật 10 canh bảng mã lỗi: mã nào trong bảng cũng phải có trong mã nguồn. Nó
# KHÔNG canh văn xuôi — và đó là chỗ tôi lọt.
#
# Tôi viết `action-outside-scope` trong một đoạn văn của đặc tả. Mã ấy không tồn
# tại ở đâu cả: không trong bảng, không trong mã nguồn, không trong vector nào.
# Luật 10 nhìn qua nó vì luật 10 chỉ đọc bảng. Người ngoài đọc đặc tả rồi cài
# đặt theo thì sinh ra một mã lỗi không bản nào khác biết — đúng thứ một tiêu
# chuẩn sinh ra để chặn.
#
# Nên: mọi token kiểu-gạch-nối trong dấu nháy ngược khắp `spec/` phải nằm trong
# bảng mã lỗi, hoặc nằm trong danh sách miễn trừ ngay dưới đây. Danh sách này
# CỐ Ý ngắn và cố ý phải sửa bằng tay: thêm một dòng vào đây là một quyết định
# có ý thức, không phải một chỗ trôi.
mien_22=$(cat <<'EOF'
acvp-mldsa65
dilithium-py
hybrid-ed25519-mldsa65-v1
x-acme-autostart
x-acme-tu-chay
EOF
)
# `acvp-mldsa65` là tên nhóm vector; `dilithium-py` là tên thư viện đối chiếu;
# `hybrid-ed25519-mldsa65-v1` là tên bộ thuật toán; hai `x-acme-*` là VÍ DỤ về
# trường lạ mà đặc tả cấm — không phải cơ chế mở rộng, 0.1 không có cơ chế ấy.
# ⚠️ Tra vào bảng của ĐÚNG PHIÊN BẢN chứa tệp, không phải luôn của 0.1.
#
# Bản đầu đọc bảng của 0.1 cho MỌI tệp dưới `spec/`. Nó chạy đúng suốt thời gian
# `spec/` chỉ có một phiên bản, rồi bản nháp 0.2 đặt tên hai mã lỗi mới và luật
# này chặn — tức là **cấm mọi phiên bản sau đặt thêm mã lỗi**, đúng thứ
# `VERSIONING.md` §3 cho phép làm. Một luật đúng với ý định mà sai với hiện thực
# vẫn là luật sai, và cái giá của nó là người ta lách bằng cách đổi tên mã lỗi —
# vừa qua được luật, vừa mất đúng cái luật sinh ra để giữ.
#
# Tệp ngoài mọi thư mục phiên bản (`spec/README.md`, `VERSIONING.md`, …) vẫn tra
# vào 0.1: chúng nói về tiêu chuẩn nói chung, và 0.1 là bản duy nhất đã chốt.
la=$(python3 - "$mien_22" <<'PY2'
import re, sys, pathlib
mien = set(sys.argv[1].split())

def bang_cua(thu_muc):
    """Tập mã lỗi khai trong bảng của một thư mục phiên bản."""
    t = pathlib.Path("spec") / thu_muc / "06-error-codes.md"
    if not t.exists():
        return set()
    return set(re.findall(r"^\| `([a-z][a-z0-9-]+)` \|", t.read_text(), re.M))

goc = bang_cua("0.1")
nho = {}
ra = {}
for p in sorted(pathlib.Path("spec").rglob("*.md")):
    # `spec/<phien-ban>/...` — phần đầu là số phiên bản thì tra bảng của nó.
    phan = p.relative_to("spec").parts
    pb = phan[0] if len(phan) > 1 and re.fullmatch(r"\d+\.\d+", phan[0]) else None
    if pb is None:
        bang = goc
    else:
        # Phiên bản kế thừa mã của 0.1: 0.2 KHÔNG bỏ mã nào của 0.1 (luật thu
        # hồi nằm ở `VERSIONING.md`, không phải ở đây), nên hợp hai tập.
        if pb not in nho:
            nho[pb] = goc | bang_cua(pb)
        bang = nho[pb]
    for tok in re.findall(r"`([a-z][a-z0-9]*(?:-[a-z0-9]+)+)`", p.read_text()):
        if tok not in bang and tok not in mien:
            ra.setdefault(tok, set()).add(str(p))
print(" ".join(f"{k}({','.join(sorted(v))})" for k, v in sorted(ra.items())))
PY2
)
if [ -n "$la" ]; then
  bao "token kiểu mã lỗi trong đặc tả mà KHÔNG có trong bảng mã lỗi:$la"
else
  dat "mọi mã lỗi được gọi tên trong đặc tả đều có trong bảng"
fi

echo
echo "--- Luật 21: cờ nào được CI \`check\` thì cũng phải được CI \`test\` ---"
# `cargo check` biên dịch mà KHÔNG chạy phép thử nào. Mã sau một cờ chỉ được
# `check` thì mọi phép thử của nó là chữ trên giấy: chúng biên dịch được, và
# không lần nào thực thi.
#
# Đã trả giá 18/08/2026: toàn bộ chắn của tầng 2 nằm sau cờ `window`, CI chỉ
# `check`, nên sáu phép thử canh chúng chưa từng chạy ở đâu ngoài máy tôi. Một
# phép thử chưa từng chạy không phải một chắn — nó là niềm tin rằng có một chắn.
ci=".github/workflows/ci.yml"
thieu=""
while read -r goi co; do
  grep -q "cargo test -p $goi --features $co" "$ci" || thieu="$thieu $goi:$co"
done <<EOF
$(grep -oE "cargo check -p [a-z0-9-]+ --features [a-z0-9-]+" "$ci" \
  | awk '{print $4, $6}' | sort -u)
EOF
if [ -n "$thieu" ]; then
  bao "cờ được check mà không được test:$thieu"
else
  dat "mọi cờ được CI check đều được CI test"
fi

echo
if [ "$loi" = "0" ]; then
  echo "════ ĐẠT: $loi vi phạm ════"
else
  echo "════ HỎNG: $loi vi phạm ════"
fi
exit "$loi"
