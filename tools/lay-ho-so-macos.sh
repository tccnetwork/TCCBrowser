#!/usr/bin/env bash
# Tải HỒ SƠ CẤP PHÉP macOS về từ App Store Connect.
#
# # Vì sao có tệp này
#
# Ngày 22/08/2026 tôi dựng hồ sơ ấy bằng tay, để nó trong thư mục tạm, và máy
# khởi động lại là mất. Một bước "đã làm xong" mà không lặp lại được thì nó chưa
# xong — nó chỉ đang chờ lần dọn dẹp tiếp theo.
#
# Hồ sơ KHÔNG phải bí mật (nó không chứa khoá riêng nào), nhưng cũng không nằm
# trong kho: nó gắn với một tài khoản Apple cụ thể và hết hạn sau một năm. Lấy
# về khi cần đúng hơn là ghim vào lịch sử git.
#
# # Cần gì
#
#   ASC_KEY_ID     mã khoá App Store Connect
#   ASC_ISSUER_ID  mã phát hành
#   ~/.appstoreconnect/private_keys/AuthKey_$ASC_KEY_ID.p8
#
# Ba thứ ấy KHÔNG viết vào tệp này: mã tài khoản của công ty không thuộc về lịch
# sử một kho mã nguồn mở.
#
#   ASC_KEY_ID=… ASC_ISSUER_ID=… tools/lay-ho-so-macos.sh [tên-hồ-sơ]
set -uo pipefail
cd "$(dirname "$0")/.."

TEN="${1:-TCC Browser Mac Development}"
RA="${TCC_HO_SO_RA:-target/release/bundle/tcc.provisionprofile}"

if [ -z "${ASC_KEY_ID:-}" ] || [ -z "${ASC_ISSUER_ID:-}" ]; then
  echo "❌ cần ASC_KEY_ID và ASC_ISSUER_ID — xem chú thích đầu tệp"
  exit 1
fi
KHOA="$HOME/.appstoreconnect/private_keys/AuthKey_$ASC_KEY_ID.p8"
[ -f "$KHOA" ] || { echo "❌ không thấy khoá riêng: $KHOA"; exit 1; }

mkdir -p "$(dirname "$RA")"
node -e '
const fs=require("fs"),crypto=require("crypto"),https=require("https");
const K=process.env.ASC_KEY_ID, I=process.env.ASC_ISSUER_ID, TEN=process.argv[1], RA=process.argv[2];
const key=fs.readFileSync(process.env.HOME+"/.appstoreconnect/private_keys/AuthKey_"+K+".p8");
const b=o=>Buffer.from(JSON.stringify(o)).toString("base64url"), n=Math.floor(Date.now()/1e3);
const h=b({alg:"ES256",kid:K,typ:"JWT"}), p=b({iss:I,iat:n,exp:n+900,aud:"appstoreconnect-v1"});
const jwt=h+"."+p+"."+crypto.sign("sha256",Buffer.from(h+"."+p),{key,dsaEncoding:"ieee-p1363"}).toString("base64url");
https.get("https://api.appstoreconnect.apple.com/v1/profiles?limit=200",{headers:{Authorization:"Bearer "+jwt}},r=>{
  let d="";r.on("data",c=>d+=c);r.on("end",()=>{
    const j=JSON.parse(d);
    const ho=(j.data||[]).find(x=>x.attributes.name===TEN);
    if(!ho){console.error("❌ không thấy hồ sơ tên \""+TEN+"\". Có: "+(j.data||[]).map(x=>x.attributes.name).join(", "));process.exit(1);}
    if(ho.attributes.profileState!=="ACTIVE"){console.error("❌ hồ sơ ở trạng thái "+ho.attributes.profileState);process.exit(1);}
    fs.writeFileSync(RA,Buffer.from(ho.attributes.profileContent,"base64"));
    console.log("✅ "+RA+"  ("+ho.attributes.profileType+", hết hạn "+(ho.attributes.expirationDate||"").slice(0,10)+")");
  });}).on("error",e=>{console.error("❌ "+e.message);process.exit(1);});
' "$TEN" "$RA" || exit 1
