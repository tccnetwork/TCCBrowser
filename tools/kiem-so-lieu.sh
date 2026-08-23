#!/usr/bin/env bash
# Số phép thử và số vector ghi trong tài liệu phải khớp số THẬT.
#
# Vì sao không nằm trong `kiem-luat-phu-thuoc.sh`: kịch bản ấy chạy TRƯỚC bước
# dựng, nên nó không biết hai con số này. Đếm mò thì sai — bản thử đếm `cases`
# trong JSON ra 122 trong khi bộ kiểm định báo 138.
#
# Người soát độc lập bắt được đúng chỗ này ngày 16/08/2026 (phát hiện F3):
# SECURITY.md ghi 234 phép thử / 135 vector trong khi thật là 238 / 136, còn
# docs/AUDIT.md thì đúng. Đây là loại trôi mà chính dự án đã chứng minh nhiều
# lần: cái gì không có máy canh thì trôi.
set -uo pipefail
cd "$(dirname "$0")/.."
loi=0
bao() { echo "❌ $*"; loi=$((loi + 1)); }
dat() { echo "✅ $*"; }

that_test=$(cargo test --workspace 2>&1 \
  | grep -E '^test result: ok' | sed 's/test result: ok. //;s/ passed.*//' | paste -sd+ - | bc)
that_vec=$(cargo run -q -p tcc-conformance 2>&1 \
  | grep -E '^TỔNG' | awk '{print $2}')

# Chỉ soi con số đứng SAU chính cái lệnh sinh ra nó:
#
#     cargo test --workspace        # 290 tests
#     cargo run -p tcc-conformance  # 138 conformance vectors
#
# Cố ý hẹp. Bản đầu quét mọi cụm "N phép thử" trong tài liệu và nó đòi sửa
# "211 phép thử mù hoàn toàn" — một sự thật LỊCH SỬ, sửa đi là bóp méo hồ sơ.
# Con số duy nhất phải đúng-ngay-hôm-nay là con số người soát chạy lệnh để đối
# chiếu, và nó luôn nằm ngay sau lệnh ấy.
kiem() {  # $1 = lệnh, $2 = số thật, $3 = tên để in
  local lech=""
  for f in README.md SECURITY.md ARCHITECTURE.md CLAUDE.md docs/*.md; do
    [ -f "$f" ] || continue
    # `--features` bị loại: đó là một lệnh KHÁC, số của nó khác ba đơn vị.
    for n in $(grep -F "$1" "$f" | grep -v -- '--features' | grep -oE '# *[0-9]+' | grep -oE '[0-9]+' | sort -u); do
      [ "$n" = "$2" ] || lech="$lech $(basename "$f"):$n"
    done
  done
  if [ -n "$lech" ]; then
    bao "tài liệu ghi sai $3 (thật là $2):$lech"
  else
    dat "$2 $3, mọi tài liệu nhắc tới đều ghi đúng"
  fi
}

kiem 'cargo test --workspace ' "$that_test" "phép thử"
kiem 'cargo run -p tcc-conformance' "$that_vec" "vector"

# ── FORMAT.md: mỗi tệp vector một mục, và con số ở tiêu đề phải ĐÚNG ──────────
#
# `FORMAT.md` là thứ DUY NHẤT người cài đặt bên ngoài đọc để dùng được bộ vector.
# Số sai ở đó là số sai ngay cửa vào.
#
# Nó đã trôi, và trôi lâu: đo ngày 23/08/2026 thì **4 trong 7** con số sai
# (`signature` 15 vs 9, `acvp` 26 vs 25, `manifest` 31 vs 34, `ui` 17 vs 27), và
# hai tệp — `package.json`, `verify.json` — **không có mục nào cả**. Không phép
# canh nào kêu, vì phép canh ở trên cố ý chỉ soi con số đứng sau một LỆNH.
#
# Đếm mọi mảng ở mức ngoài cùng trừ `notes` (mảng ấy là văn xuôi). Một tệp có
# nhiều mảng ca — `package.json` có ba — thì tiêu đề ghi TỔNG.
lech_format=$(python3 - <<'PY2'
import json, re, pathlib
s = open("conformance/FORMAT.md").read()
ghi = {t: int(n) for t, n in re.findall(r"^## `([a-z0-9-]+\.json)` — (\d+) cases", s, re.M)}
ra = []
for p in sorted(pathlib.Path("conformance/vectors").glob("*.json")):
    d = json.load(open(p))
    that = sum(len(v) for k, v in d.items() if isinstance(v, list) and k != "notes")
    if p.name not in ghi:
        ra.append(f"{p.name}:KHÔNG-CÓ-MỤC")
    elif ghi[p.name] != that:
        ra.append(f"{p.name}:ghi-{ghi[p.name]}-thật-{that}")
for ten in ghi:
    if not (pathlib.Path("conformance/vectors") / ten).exists():
        ra.append(f"{ten}:MỤC-THỪA")
print(" ".join(ra))
PY2
)
if [ -n "$lech_format" ]; then
  bao "conformance/FORMAT.md lệch với vectors/:$lech_format"
else
  dat "FORMAT.md mô tả đủ mọi tệp vector, và mọi con số đều đúng"
fi


# ── Mọi PHÉP THỬ mà SECURITY.md trích dẫn phải TỒN TẠI ───────────────────────
#
# §1 của `SECURITY.md` là bảng "bất biến → phép thử giữ nó". Cấu trúc ấy chỉ có
# giá trị khi phép thử được trích còn tồn tại.
#
# Ngày 23/08/2026 xoá một bộ dựng, và **13 trong 40** bất biến trỏ vào phép thử
# đã đi cùng nó. Một trong số đó — B31, "sắc thái mất mát phải được VẼ khác" —
# đồng thời trở thành SAI: bộ dựng mới đọc `Tone` chỉ để đặt cờ trợ năng. Tài
# liệu vẫn khẳng định bất biến ấy được giữ, và trỏ vào một hàm không còn.
#
# Cổng này không kiểm được bất biến còn ĐÚNG hay không — không cổng nào kiểm
# được. Nó kiểm điều duy nhất kiểm được bằng máy: bằng chứng được trích có tồn
# tại hay không. Một bất biến mất bằng chứng phải nói ra là mất, chứ không im.
thieu_thu=$(python3 - <<'PY2'
import re, subprocess, pathlib
# CHỈ soi các dòng của bảng bất biến (`| B…`). Quét cả tệp thì bắt nhầm tên
# mô-đun và thuộc tính serde — `deny_unknown_fields` không phải một phép thử.
# Bảng ấy đúng là chỗ "bất biến → bằng chứng", nên nó là chỗ duy nhất mà một
# tên trong nháy ngược HỨA rằng có một phép thử mang tên đó.
dong = [l for l in open("SECURITY.md") if re.match(r"\| B\d+ \|", l)]
ten = set(re.findall(r"`([a-z][a-z0-9_]{12,})`", "".join(dong)))
# Soi mọi ĐỊNH DANH trong mã nguồn, không riêng `fn`. Một dòng bất biến được
# phép trích một hàm, một biến hay một bảng làm bằng chứng CẤU TRÚC — B14 trích
# `bang_hanh_dong`, và đó là một trích dẫn hợp lệ.
#
# Cổng này không hứa "cái được trích là một phép thử". Nó hứa điều hẹp hơn và
# vẫn đủ: **cái được trích còn TỒN TẠI**. Đó đúng là thứ vỡ khi xoá một crate —
# mười ba dòng trỏ vào hư không, và một trong số đó che một bất biến đã thành sai.
ma = subprocess.run(
    ["grep", "-rhoE", r"\b[a-z][a-z0-9_]{12,}\b", "--include=*.rs", "crates", "tools", "apps"],
    capture_output=True, text=True).stdout
that = set(ma.split())
print(" ".join(sorted(t for t in ten if t not in that)))
PY2
)
if [ -n "$thieu_thu" ]; then
  bao "SECURITY.md trích thứ KHÔNG TỒN TẠI trong mã nguồn:$thieu_thu"
else
  dat "mọi bằng chứng SECURITY.md trích dẫn đều tồn tại trong mã nguồn"
fi

# ── Mọi lệnh `cargo` trong tài liệu phải trỏ tới gói và cờ CÓ THẬT ────────────
#
# `docs/AUDIT.md` bảo người soát độc lập chạy một danh sách lệnh để tự đối chiếu
# mọi khẳng định. Ngày 23/08/2026 nó vẫn bảo họ chạy
# `cargo test -p tcc-render-webview --features window` — một crate đã bị xoá.
# Người soát gõ vào, nhận lỗi, và điều họ học được là tài liệu không đáng tin.
#
# Kiểm TĨNH, không chạy: gói có trong workspace không, và cờ có tồn tại cho gói
# ấy không. Rẻ, và bắt đúng hạng lỗi đã xảy ra.
lech_lenh=$(python3 - <<'PY2'
import json, re, subprocess, pathlib
md = json.loads(subprocess.run(
    ["cargo", "metadata", "--no-deps", "--format-version", "1"],
    capture_output=True, text=True).stdout)
goi = {p["name"]: set(p["features"]) for p in md["packages"]}
ra = []
tep = ["README.md", "SECURITY.md", "ARCHITECTURE.md", "CLAUDE.md"]
tep += [str(x) for x in pathlib.Path("docs").glob("*.md")]
for f in tep:
    if not pathlib.Path(f).exists():
        continue
    for l in re.findall(r"^(cargo [^\n#]*)", open(f).read(), re.M):
        m = re.search(r"-p (\S+)", l)
        c = re.search(r"--features (\S+)", l)
        if m and m.group(1) not in goi:
            ra.append(f"{pathlib.Path(f).name}:gói-{m.group(1)}")
        elif m and c:
            for co in c.group(1).split(","):
                if co not in goi[m.group(1)]:
                    ra.append(f"{pathlib.Path(f).name}:{m.group(1)}-thiếu-cờ-{co}")
print(" ".join(sorted(set(ra))))
PY2
)
if [ -n "$lech_lenh" ]; then
  bao "tài liệu bảo chạy lệnh trỏ tới gói/cờ KHÔNG TỒN TẠI:$lech_lenh"
else
  # Dấu nháy ngược trong chuỗi kép là THAY THẾ LỆNH, không phải chữ. Bản đầu
  # viết "mọi lệnh `cargo` …" và vỏ lệnh chạy `cargo` rồi dán trang trợ giúp của
  # nó vào giữa câu báo ĐẠT.
  dat "mọi lệnh cargo trong tài liệu đều trỏ tới gói và cờ có thật"
fi

[ "$loi" = 0 ] && echo "════ ĐẠT ════" || echo "════ HỎNG: $loi ════"
exit "$loi"
