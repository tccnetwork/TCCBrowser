#!/usr/bin/env bash
# Đếm số chỗ DÙNG `unsafe` trong mã THẬT SỰ ĐƯỢC BIÊN DỊCH vào nhị phân sản phẩm.
#
# Cách đo: dựng `tcc-shell` vào một thư mục đích SẠCH, rồi đọc tệp `.d` (dep-info)
# rustc sinh ra — chúng liệt kê đúng từng tệp `.rs` đã được đọc. Không đoán `cfg`,
# không đoán nền.
#
# Vì sao phải thế, và đây là lần thứ hai phép đo này sai:
#
#   • Bản 1 đếm theo `cargo metadata`: đồ thị đầy đủ chứa cả phụ thuộc của nền
#     không bao giờ dựng ở đây, nên `r-efi` — ràng buộc firmware UEFI — đứng thứ
#     ba trong danh sách "nặng nhất" của một trình duyệt để bàn.
#   • Bản 2 lọc theo nền, nhưng vẫn đếm CẢ THƯ MỤC `src` của mỗi crate. `blake3`
#     ra 227, trong khi trên máy này nó chỉ biên dịch `portable` + bốn tệp `ffi_*`;
#     `rust_sse2`/`rust_sse41`/`rust_avx2`/`neon`/`wasm32_simd` đều nằm sau `cfg`
#     và KHÔNG được dựng. Lọc crate mà không lọc tệp thì vẫn phồng.
#   • Và bản 2 đo cho `aarch64-apple-darwin` trong khi toolchain ở đây là
#     `x86_64-apple-darwin`. Đo nhầm nền mà không hay biết.
#
# Đếm CHỖ DÙNG chứ không đếm chữ: đếm chữ thì `#![forbid(unsafe_code)]` cũng bị
# tính, và `ttf-parser` — crate CẤM unsafe — ra 2 thay vì 0.
#
# Con số vẫn là CẬN TRÊN theo dòng: hai khối `unsafe` trên một dòng tính một.
set -euo pipefail
cd "$(dirname "$0")/.."

DICH=${CARGO_TARGET_DIR:-$(mktemp -d /Volumes/DATA/.tmp/dem-unsafe-XXXXXX 2>/dev/null || mktemp -d)}
NEN=$(rustc -vV | sed -n 's/^host: //p')
don_dep() { [ -n "${GIU:-}" ] || rm -rf "$DICH"; }
trap don_dep EXIT

echo "── nền: $NEN · thư mục đích sạch: $DICH"
CARGO_TARGET_DIR="$DICH" cargo check -p tcc-shell --quiet 2>&1 | tail -3

python3 - "$DICH" "$NEN" <<'PY'
import pathlib, re, sys, collections, json, subprocess
dich, nen = pathlib.Path(sys.argv[1]), sys.argv[2]
deps = dich/"debug"/"deps"
if not deps.is_dir():
    print(f"❌ không có {deps} — bản dựng chưa chạy, KHÔNG phải 'không có unsafe'")
    sys.exit(1)
tep = set()
for d in deps.glob("*.d"):
    for t in d.read_text(errors="replace").replace(":", " ").split():
        if t.endswith(".rs"):
            tep.add(pathlib.Path(t))
if not tep:
    print("❌ tệp .d không liệt kê nguồn nào — bộ đọc hỏng, KHÔNG phải cây sạch")
    sys.exit(1)

# Crate nào THẬT SỰ nằm trong nhị phân: đi từ `tcc-shell` theo cạnh `normal`,
# KHÔNG đi xuyên qua crate proc-macro. `syn`, `quote`, `proc-macro2` chạy lúc
# dựng rồi biến mất; gộp chúng vào là nói quá về thứ người dùng chạy.
md = json.loads(subprocess.run(["cargo", "metadata", "--format-version", "1"],
                               capture_output=True, text=True).stdout)
goi = {p["id"]: p for p in md["packages"]}
pm = {i for i, p in goi.items()
      if any("proc-macro" in t["kind"] for t in p["targets"])}
nut = {n["id"]: n for n in md["resolve"]["nodes"]}
goc = next(i for i in goi if goi[i]["name"] == "tcc-shell")
tham, hang = set(), [goc]
while hang:
    i = hang.pop()
    if i in tham:
        continue
    tham.add(i)
    if i in pm:
        continue
    for d in nut[i]["deps"]:
        if any(k["kind"] is None for k in d["dep_kinds"]):
            hang.append(d["pkg"])
trong_nhi_phan = {goi[i]["name"] for i in tham}

DUNG = re.compile(r'\bunsafe\s*(\{|fn\b|impl\b|trait\b|extern\b)')
theo_crate = collections.Counter()
so_tep = collections.Counter()
for f in sorted(tep):
    if not f.is_file():
        continue
    # Tên crate: thư mục dạng `<ten>-<phienban>` trong registry; mã của ta thì
    # lấy tên thư mục crate.
    ten = None
    for p in f.parts:
        m = re.fullmatch(r"(.+)-\d+\.\d+\.\d+.*", p)
        if m:
            ten = m.group(1)
    if ten is None:
        parts = f.parts
        ten = parts[parts.index("crates")+1] if "crates" in parts else "(khác)"
    n = 0
    for x in (y.strip() for y in f.read_text(errors="replace").splitlines()):
        if x.startswith("//") or x.startswith("*"):
            continue
        if DUNG.search(x):
            n += 1
    theo_crate[ten] += n
    so_tep[ten] += 1

ngoai = {k: v for k, v in theo_crate.items()
         if not k.startswith("tcc-") and k in trong_nhi_phan}
dung_thoi = {k: v for k, v in theo_crate.items()
             if not k.startswith("tcc-") and k not in trong_nhi_phan}
ta = {k: v for k, v in theo_crate.items() if k.startswith("tcc-")}
print(f"{nen}: {len(tep)} tệp .rs được biên dịch")
print(f"   TRONG NHỊ PHÂN: {len(ngoai)} crate ngoài · {sum(ngoai.values())} chỗ "
      f"dùng unsafe · {sum(1 for v in ngoai.values() if v == 0)} crate sạch")
print(f"   CHỈ LÚC DỰNG (proc-macro và cây của nó): {len(dung_thoi)} crate · "
      f"{sum(dung_thoi.values())} chỗ — không đi vào nhị phân")
print(f"   MÃ CỦA TA: {sum(ta.values())} chỗ")
print("── mười crate nặng nhất trong nhị phân:")
for k, v in sorted(ngoai.items(), key=lambda x: -x[1])[:10]:
    print(f"  {v:5d}  {k}  ({so_tep[k]} tệp)")
duong = ("blake3", "ttf-parser", "cosmic-text", "fontdb", "swash", "libm",
         "core_maths", "ed25519-dalek", "tiny-skia", "softbuffer", "rustybuzz")
print("── đường từ byte gói tới màn hình:")
for k in duong:
    if k in theo_crate:
        print(f"  {theo_crate[k]:5d}  {k}  ({so_tep[k]} tệp)")
PY
