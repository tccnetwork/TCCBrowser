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
echo "--- Luật 13: định danh CÔNG KHAI không được mang tên tiếng Việt ---"
# ARCHITECTURE §7 nói định danh viết tiếng Anh, chú thích viết tiếng Việt. Luật
# đó đã TRÔI suốt nhiều tháng — vì nó là luật duy nhất không có máy canh. Đây là
# máy canh.
#
# Ranh giới cố ý đặt ở `pub`: đó là bề mặt người viết bản cài đặt thứ hai đọc.
# Tên hàm kiểm thử, biến cục bộ và tên ví dụ đối kháng vẫn tiếng Việt — chúng là
# LẬP LUẬN, không phải giao diện, và SECURITY.md trích dẫn tên phép thử làm bằng
# chứng nên đổi tên là làm hỏng đúng thứ nó ghi lại.
GOC_VIET='(chu|cau|vai|dau|mat|sac|thai|tra|luu|xoa|them|ky|bo|dung|mo|tao|vong|thoat|hoi|quyet|dinh|choi|cho|phep|bat|tat|khoa|nho|hanh|man|hinh|khoi|tan|cong|bang|kieu|thu|ghi|danh|phuc|cua|hop|quet|goi|kiem|bam|nut|loi|mang|nap|gui|tep|duong|noi|ten|nguoi|lieu|tro|nang|lietke|quen|van|tay|tinh|trang)'
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
  ma_code=$(grep -rhoE '"[a-z][a-z0-9-]+"' crates/*/src/lib.rs crates/*/src/tree.rs 2>/dev/null | tr -d '"' | sort -u)
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
CONG_DEMO="cb0689bc08e4905c98a8276b2646ee7337f0e8b185c4223825be4cc1c16b9b81f7a3d1dfc6f69acb3473e76dfc1305128d14f537e993a7ef8f3aee9482584027619c439213659b55b79c4b3e6781ebf5ef9d753fc233fb7740b9171b94f6fe436b069870605b29fb621ac6723e3b7e66e8a336c90254f6d9a23fd83aa3f9f0c6c38d7414eeb23227306a207434e7f5780283a01e46f9642c7581d49892b4a60b659c3573f7f094150fd7541590925af6d33a481810edfb73265098bd615a7da16da63bb60111b6061e23a10a8d0dc621cfbc069ab9bb77978e836cff0aeeabf6a72d0a0d0f8b2a5b9aacc8b6718c178b8082a7c37c6bd4bd14da8477d0627e5c002bd649f5b1926645fcfbbd1e1a3c63dd2f538f8047e0e3eb07128957e809f475e7b77abb5044b84ab32b129b1382d8cf616f6f1931a2e2750f2f1527746293bc3391b10e326ad9c5972fad25caf5c676639c926247e6e3ba181c0b2cdacd252b63d6b2f798118d9e7696596baaf4b873e777b8a17a2a8f8238d2223c5c2ff6890b7e05f6a790397e36a78b1e4405974516b1cf80af9aa1965a8fbeba48561255aa4ffcb3b558436b4b5a18bad173619422f9df066da81d2c787523b66bec1c4d99231b986baff4f8c7fa00119bc93426aeaa5a85261f0609f235b29ad1ab4c6873ed6ff0062aedc2e00283b4794f9a99f8d3632fc52e37d02c828e57b52e69c6a8d5360c84c737fb71ca48b6982ce6530c3c378b3bd9035e5f9f37b2bb239f829ac4462504e8db15ea1043e4c5a8986d188b48fdd47b379b9b67a5b541fef725457b8a9df4f5d4416adf7d5bbf18d261e8e47c0cecbc3f39c7f2eee4b57fb74db29448f27e0108a56b68643ccff4709af90ccde72e1203df05ecd644e2137b2d72a0934a978bd41cc0bacf7eeb8e84cbab07a89a284d21f37908a6ca58fc2719bd11a658e2f667b02c5b9cef6505c89f7aac954185ddca578539f9274554cbfb88bc4d9b593b28669075131268be9703161cd12c58d8aad929e1c30339cecdf022f45487100d4e16e887647657728b3def18079bc642d6ab040d94b07c29f2e8731594a939b2031ac36f31445bfd8684d33e97577b880f017e3a31d74cedd7287486b28027dab3f2a899e1f0ec546e1d7a2d4329e1b2a2a5dde217c4af8d4777de82bae6530d1347e54737bc5dddb381dc365ad432fc3ed3259894c246f64ee936b9b4dfcec185591c10f4b8b3cf7ce53c06bebcbde95bd0e1e6c5762e19289218d1429e15441a9e6d22af420e5f80746d6104d88b3404977f10f8c88d05843679230aa59acf60426493c366c97e07033e5ce428def2a444459d96a482387e5e89f37767bbdc7ca91b96179fcf51f834b380979166be685045861cd944a00c4ccad6e6be4a3ea310b4098333d27de05316b25f6ab2bf78a46d338cd90e5256acace451ac15013c0bbc2f8b27532b4bc67202e99a01d8b307148b4f554692b32570b0afc80a65150837f3fc525387a81dde986d2f8cd2c0a313732a06ca9f1eda3e7c44f16e9574201c96760822050a090e538bbb4214dfc2430dc01d35b8ed2588897267e65b962e169b309d4c83efd64e8cbf18da5960c9a1cd431052f770f6eeed3622567251e39998fbc55440f548f92fe665b9191c32dac59327a1ae318a677e1dfef9e6b787cea2f60a384145fdfec904a86fb28cd8ea01d6ee6f06dd0957aed3940e1ae653260a63b1411deff446c8d68442ce3b6e9b9e8a5f461a75849b8bc4451b75028e383b4e9190fcaab5c7dc66646db879b1bb4803d3d2b17c3013c0b3a64a415e5583c4679ef9cb5eca15b9dd05bd927a0de6857ef5136c39f66b56df4f7c9e18e36eec3f003f9b3bd7a32eebb736ecf1cbeb711e0fe2553c4cd89c3f2f73b78e8f53fd72657f653afbe4bebc06b3a27faa64f7626960b4f7cab94b138d39aa5118cd1ca749a7a4dc30af59b4b491b96133a5f639e3161dacb3de5f90e7bd4b5369aeef4414083e9c340936c323f135bac9eff5b3fbc9fd162d9ca3a1a4b2effb3db9edad0192c936d7effb4e80ce049f8f7877d8896e181436205f93e808f982cb78fab4730e2c836e64328e2b1f84c5ea014fc6f0a85a5f635d3294e8414ceaf4d89ac866f8d0f7446778cb8df98d87b60ae7395a2e1d413404590a9760fa82bf936e11eefd78fa6f6b4a47d34ffb0718f26bfc61f03891aa474813e6b345bb1defefd2b925543276a5839cd23bcce605b4754dd26c253d1c13feca36bb26edb2b8238c1ff1e3a4d35f9bdb8de4a6c1886f274e0521f080b61ea7a247a992d1ac5eb54ffaa6f0fa008f5a0421ea0141f4791009cfae708fb4bbb28e7c5f99fd69b835313c82874a53a34f830e357fe5f09a4c8ab19d84c161e91d236ea5780545f4beed0f5050716a7cfb163af40564f8739e0843e8d8a73b7d8954f7850073b62d3e92c33567efeceaa8d492eb3211fb267424ba178c04276478a2fc51a3f33f0d109cdc6049a9c766d3f3a4a5ff99769a8b46819da3bcd7113054fa5966b52cfc319ef386b01e6f193154e16fe4b4610e17ba1d7e15dc0a28d25a5de4ff757b54b989a6668e9c7b5b486bae6ceddb90b154502dd020d2120c1ae6499f8e6355f85c2acb997e1b2bfd2adac0e17c9dbc60ccf9f177479339ad1ade36e10ed25139525c40f150ecb0bca645975d5d396951c733657ceee99f15abb16a8ce2563853020815666ffcb78ee018890715527590763da3114712fe1b52db7474ffca395f3dac6b1b64e8ea4e6fa6e690a1e7139372887a16afaa"
lac=$(grep -rl "$CONG_DEMO" --include=manifest.json . 2>/dev/null | grep -v '^./examples/' || true)
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
for v in canonical signature acvp-mldsa65 manifest ui capability; do
  [ -f "conformance/vectors/$v.json" ] || thieu="$thieu $v"
done
if [ -n "$thieu" ]; then
  bao "thiếu nhóm vector:$thieu"
else
  dat "đủ sáu nhóm vector"
fi

echo
if [ "$loi" = "0" ]; then
  echo "════ ĐẠT: $loi vi phạm ════"
else
  echo "════ HỎNG: $loi vi phạm ════"
fi
exit "$loi"
