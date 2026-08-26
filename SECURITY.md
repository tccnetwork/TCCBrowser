# Security — TCC Browser

> Written for **auditors**. It records three things: the invariants that must
> hold, the holes found and fixed, and **what has never been examined**. The last
> section matters most — a security document that lists only achievements is a
> misleading document.

Updated: 2026-08-14 · Scope: `tcc-crypto`, `tcc-spec`, `tcc-manifest`,
`tcc-capability`, `tcc-ui`, `tcc-render-webview`, `tcc-runtime`, `tcc-shell`

---

## 1. Invariants — break one and the architecture is broken

| # | Invariant | Held by |
|---|---|---|
| B1 | A hybrid signature is valid only when **BOTH** halves are | `gia_mao_nua_co_dien`, `gia_mao_nua_hau_luong_tu` |
| B2 | The signature covers **both the manifest and the content** | `thay_ruot_giu_chu_ky_thi_hong` |
| B3 | A capability **does not exist** until granted | Private fields + a `compile_fail` doctest |
| B4 | Revocation kills **every copy already handed out** | `thu_hoi_giet_ca_ban_sao_dang_cam` |
| B5 | Hostnames match **exactly** — no suffix matching | `khong_khop_ten_mien_con_va_khong_khop_hau_to` |
| B6 | Strings shown to the user contain **no spoofing characters** | `ten_ung_dung_co_ky_tu_dao_chieu_thi_tu_choi` |
| B7 | ~~App-supplied text **cannot escape** the renderer's document~~ | **RETIRED 2026-08-23** — there is no document. Text is a node's content and is drawn as glyphs; nothing parses it. Both tests died with the web engine (§3.7) |
| B8 | The accessibility tree the renderer publishes **matches** the source tree | `check_accessibility_parity`, run against `RasterRenderer` in five screen modules |
| B9 | The interface **cannot express** a button missing a role or a label | Types: `Alt::Decorative` must be stated out loud |
| B10 | Apps **cannot set colours** — only declare intent | `Tone` is a closed enum with no colour field |
| B11 | Skipping signature verification **does not compile** | `grant_verified` accepts only `VerifiedApp` |
| B12 | Every unclear path ends in **DENY** | `moi_duong_khong_ro_rang_deu_ra_tu_choi` |
| B13 | **The renderer wires events; the app never does** | Structural: a package ships a `ui.json` tree and no code. The frame hit-tests its own drawing; there is nothing an app could wire an event to. Citation rewritten 2026-08-23 — the old one named a content-security policy and an init script, both of which went with the web engine |
| B14 | Phantom actions **go nowhere** | Structural: both input paths look the action up in tables built **from the drawn tree** — `hit_test` for the mouse, `bang_hanh_dong` for the accessibility axis. An id in neither table resolves to nothing. See §3.9 for what happens when those tables are rebuilt |
| B15 | Apps **ship no markup**, only a declarative tree | `entry` is `ui.json`; `tcc-ui` exposes no web concepts |
| B16 | Decoding from disk **cannot bypass the constructors** | `UiNode` is a separate type; `TryFrom` rebuilds through the checking constructors |
| B17 | Permission is asked **item by item**, every toggle default OFF | `moi_cong_tac_mac_dinh_tat`, `bam_cho_phep_ma_khong_bat_gi_thi_khong_cap_quyen_nao` |
| B18 | ~~A phantom toggle **discards the whole message**~~ | **RETIRED 2026-08-23** — there is no message. Toggle state is held by the frame and changed only by hit-testing the drawn tree, so there is no channel a phantom toggle could arrive on. The intent survives in B14 |
| B19 | **The first decision wins** — no overwriting | `quyet_dinh_dau_tien_thang`. ⚠️ Held on the accessibility path only until 2026-08-23; the mouse path had no guard at all — see §3.10 |
| B20 | Button behaviour is declared in the **signed manifest**, not in `ui.json` | `Manifest::actions` sits inside the signature's scope |
| B21 | An action **cannot request** a capability that was never asked for | `kiem_hanh_vi` + `hanh_vi_goi_may_chu_chua_xin_quyen_thi_tu_choi` |
| B22 | **Not one packet leaves the machine** before the grant | `chua_cap_quyen_thi_khong_goi_ra_ngoai_mot_lan_nao` |
| B23 | **Redirects are NOT followed** — that is a capability escape | `max_redirects(0)` + `moi_chuyen_huong_deu_bi_tu_choi` |
| B24 | HTTPS only, with a timeout and a size ceiling | `dia_chi_luon_la_https`, `chi_2xx_moi_dat`, `moi_chuyen_huong_deu_bi_tu_choi`, `chi_loi_vuot_tran_moi_bao_la_qua_lon`, `cau_bao_qua_lon_mang_tran_cua_kenh_ay`, `MAX_BYTES`/`MAX_WAIT` (build-time `const` assertion) — three of these added 2026-08-25, see §3.16 |
| B25 | The path out to the network is **visible in the dependency tree** | Rule 8: only `tcc-shell` depends on `tcc-net` |
| B26 | A remembered permission is bound to the **signer's key**, not just the app id | `doi_khoa_nguoi_ky_thi_phai_hoi_lai` |
| B27 | A remembered permission is bound to the **scope**, not just the capability name | `noi_rong_pham_vi_thi_phai_hoi_lai` |
| B28 | A corrupt permission store → **ask again**, never fall back to allow | `tep_hong_hoac_phien_ban_la_thi_hoi_lai` |
| B29 | Signing key changed → **warn**, and the warning comes BEFORE the permission list | `doi_khoa_ky_thi_canh_bao_hien_ra_truoc_danh_sach_quyen` |
| B30 | Control labels are **visible to people who look**, not only to the accessibility tree | `nhan_dieu_khien_duoc_ve_ra_cho_nguoi_nhin` — rewritten 2026-08-23 |
| B31 | A destructive tone is **drawn differently**, not merely declared | `nut_mat_mat_duoc_ve_khac_nut_thuong` — ⚠️ **this invariant was BROKEN from 2026-08-23 until it was re-tested the same day**, see below |
| B32 | ~~Input fields carry **NO ARIA role**~~ | **RETIRED 2026-08-23** — ARIA is a markup concept and there is no markup. The intent survives in B8: roles come from `tcc-ui`, and the renderer may not invent or override them |
| B33 | The "destructive" signal **reaches the OS accessibility axis** | `nut_khong_hoan_tac_noi_ra_dieu_do`, in crates/tcc-render-raster/src/accesskit_bridge.rs — the citation was wrong, the evidence was there |
| B34 | ~~Package files are served over a **custom protocol**~~ | **RETIRED 2026-08-23** — the file server existed to feed a web engine. Nothing serves package bytes to a renderer now; the renderer reads the signed tree directly |
| B35 | ~~**Images only**, by an extension ALLOWLIST — **no SVG**~~ | **RETIRED 2026-08-23** — same server, same removal. ⚠️ The reason it existed has NOT gone away: SVG is a document format that can carry script. Any future path that hands package bytes to something that parses them must re-derive this rule |
| B36 | The permission dialog **serves no app file at all** | Structural, and now trivially so: **nothing** serves package bytes to a renderer. The dialog is a `tcc-ui` tree built by the frame from the manifest |
| B37 | Decisions **never read the description** stored on disk | `quyet_dinh_khong_doc_phan_mo_ta` |
| B38 | A destructive button **does not stretch the full width** | `nut_mat_mat_khong_gian_het_be_ngang` — rewritten 2026-08-23 |
| B39 | **Machine markers are separate from human text** — text translates, markers never change | `doi_ngon_ngu_khong_lam_doi_dau_hieu_may` — rewritten 2026-08-23 |
| B40 | A manifest field the standard does not define is **rejected** | Conformance: three `truong la` vectors, mutation-tested in both directions |
| B41 | Every action **announced** to the accessibility axis is also **clickable**, and vice versa | `hanh_dong_doc_len_duoc_thi_cung_bam_duoc` — added 2026-08-23, see §3.11 |
| B42 | A **text field** is reachable from the accessibility axis, and `Focus` **never** activates a button | `o_nhap_vao_duoc_bang_de_tro_nang_chon_duoc`, `tieu_diem_khong_kich_hoat_nut` — added 2026-08-23, see §3.12 |
| B43 | Text the **frame** holds passes the same display-string checks as text from disk | `chu_do_khung_giu_khong_lach_duoc_phep_kiem`, `phim_go_khong_hop_le_bi_tu_choi_ngay` — added 2026-08-23, see §3.13 |
| B44 | The process creates **exactly one** event loop, whatever screens it shows | `vong_lap_su_kien_dung_mot_lan_cho_ca_tien_trinh` — added 2026-08-24, see §3.14 |
| B45 | The sentence that carries a **safety fact** is the sentence that carries the warning mark — not merely some sentence on the same screen | `man_hong_noi_ro_khong_luu_gi`, `quyen_vi_ky_duoc_hien_khac_han_quyen_khac`, `cau_chuyen_tien_duoc_ve_khac_di` — added 2026-08-25, see §3.15 |
| B46 | The bounds the specification states as **inclusive** are the bounds the code enforces, at the exact edge | `ranh_gioi_do_dai_dung_nhu_dac_ta_ghi`, `dung_nguong_dau_chong_thi_van_qua`, `xuong_dong_va_tab_bi_choi_voi_ly_do_rieng` — added 2026-08-25, see §3.17 |
| B47 | The implementation emits **only** error codes the specification defines — checked in **both** directions | `khoa_khong_phai_diem_van_ra_bad_signature`; rule 10 and rule 10b in `tools/kiem-luat-phu-thuoc.sh` — added 2026-08-25, see §3.18 |
| B48 | A signature verifies **only** at its exact length — no trailing bytes, no truncation | `chu_ky_thua_hay_thieu_mot_byte_deu_bi_choi`, vector `them mot byte thua` — added 2026-08-25, see §3.19 |
| B49 | **Every** capability entry point — not a representative one — refuses after `revoke_all` | `thu_hoi_giet_moi_loi_vao_cua_moi_quyen` — added 2026-08-25, see §3.20 |
| B50 | The window is **operable by keyboard**: `Tab` reaches every interactive target and wraps, `Enter` never activates from inside a field, `Enter` on a toggle does not close the screen | `tab_di_vong_hai_dau`, `enter_trong_o_nhap_khong_kich_hoat_gi`, `enter_tren_cong_tac_khong_dong_man`, `tieu_diem_va_o_dang_chon_luon_khop` — added 2026-08-25, see §3.21 |
| B51 | Focus is **visible**, and its mark is distinct from the destructive mark of B31 | `vien_tieu_diem_them_muc_that` (measures ink, not the tree) — added 2026-08-25, see §3.21 |
| B52 | Hover, focus and destructive each have a **distinct shape** — a single-ink renderer cannot say them in colour | `re_chuot_thay_duoc_va_khac_tieu_diem`, `re_chuot_khong_doi_tieu_diem` — added 2026-08-25, see §3.21 |
| B53 | The **product build with a window always carries the accessibility bridge** — it is not a separate flag someone can forget | rule 24 in `tools/kiem-luat-phu-thuoc.sh` — added 2026-08-25, see §3.23 |
| B54 | Text editing works **at the caret**, and cuts on character boundaries — never on bytes | `chen_xoa_cat_theo_chu_khong_theo_byte`, `xoa_mot_lan_di_het_mot_chu_co_dau`, `con_tro_ve_dung_cho_trong_chuoi` — added 2026-08-26, see §3.24 |
| B55 | Clearing a screen's state is bound to **one named variant** — a screen that merely updates keeps what the user typed, a new screen never inherits it | `xoa_trang_thai_gan_voi_dung_mot_bien_the` — added 2026-08-26, see §3.25 |
| B56 | The **content size cap** holds at its exact edge and accumulates across subdirectories | `tran_noi_dung_chan_dung_o_mep`; the constant itself is a build-time `const` assertion — added 2026-08-26, see §3.26 |
| B57 | A code the specification **withdrew** stays unreachable from a package, and a test proves it | `goi_co_publisher_khong_phai_hex_ra_ma_not_hex`, `goi_co_con_tren_nut_la_ra_ma_bad_json`; rule 10b — added 2026-08-26, see §3.26 |
| B58 | A `Debug` that exists to **redact** is tested — it hides the secret **and** still says something | `debug_cua_kieu_giu_khoa_khong_lo_khoa`, `debug_khoa_cong_khai_va_chu_ky_noi_duoc_do_dai` — added 2026-08-26, see §3.27 |
| B59 | "No key yet", "you pressed cancel" and "the OS failed" stay **three different answers** | `ba_ma_trang_thai_ra_ba_loi_khac_nhau` — added 2026-08-26, see §3.28 |
| B60 | Layout fractions are **the fractions they name**, and every one lands in (0, 1] | `phan_so_cua_be_dung_bang_phan_so_ay`, `be_bi_cam_thi_bi_choi_voi_ma_bad_layout` — added 2026-08-26, see §3.29 |
| B61 | A JSON-RPC reply carrying `error: null` is **success**, and one carrying a real error is **not** — decided by a pure function | `doc_phan_hoi_phan_biet_ba_truong_hop` — added 2026-08-26, see §3.29 |
| B62 | A build never **asks** for a capability it cannot grant, and never honours a stored answer for one | `ban_dung_khong_co_vi_thi_khong_hoi_ve_vi`, `cap_duoc_noi_dung_su_that_ve_ban_dung` — added 2026-08-26, see §3.30 |

**B40 — what is signed must be exactly what is checked (2026-08-14).**

The specification said an implementation *SHOULD* reject unknown fields; the code
silently ignored them. `Manifest`, `CapabilityRequest` and `Scope` all lacked
`deny_unknown_fields` — while the interface tree had it from the start.

This is not tidiness. **The signature covers every byte of `manifest.json`,
including bytes no rule of the standard reads.** A field nobody validates is a
channel carrying meaning outside the standard: the same signed package means one
thing on an implementation that understands `x-acme-autostart` and another on one
that does not. Vendor prefixes broke interoperability on the web exactly this
way, and a signature makes it worse — the divergent behaviour arrives looking
authentic.

Inside a capability scope it is worse still. A scope reading
`{"kind":"network","hosts":["a.com"],"ports":[443]}` grants port 443 on an
implementation that knows `ports` and **every port** on one that ignores it.
Dropping a field silently can only ever widen a permission, never narrow one.

The standard-level consequence is recorded in [`spec/VERSIONING.md`](spec/VERSIONING.md)
§3: because unknown fields are rejected, **every added field is a breaking
change**. "It's only additive" is never an argument here.

**B39 — why one warning sentence resisted translation for several sessions.**

The accessibility scanner recognised "destructive action" by comparing
`aria-description` against the exact string `"Hành động không hoàn tác được."`.
That string was simultaneously **human-readable text** and **a machine marker**.

The consequence: translating it into English blinded the scanner, turned the
accessibility check red, and drained `Tone::Danger` of all meaning. So it stayed
hard-coded in Vietnamese — inside an interface that defaults to English.

**Fusing two roles into one string always locks it down like that.** They are now
separate:

| | |
|---|---|
| Machine marker | `data-sac-thai="mat-mat"` — never changes, never displayed |
| Human text | injected down from `tcc-shell`, freely translatable |

The renderer **does not know the language and should not** — the translation
table lives in `tcc-shell`, and `text::renderer_text()` is the only door text comes
through. The same approach as `trait Network` and the file server: anything
context-dependent is injected from outside.

The renderer defaults to **English**, matching the interface default. Measured on
the real accessibility axis: `"Xoá dữ liệu, button — this cannot be undone"` —
the button label stays Vietnamese because it is the **app's** text, and
translating it would be speaking on the app's behalf.

**B37 — the permission management screen, and a lie that had to be walled off.**

"Grant" without "review and revoke" is half a permission system. But that screen
needs to display `shop.tcc-coin.com`, while the store keeps only a **fingerprint**
of the scope — and a fingerprint does not read backwards. So a description string
must also be stored.

Text on disk is **editable text**. Anyone who can edit the file can make the
screen display "shop.tcc-coin.com" while the fingerprint corresponds to an
entirely different scope — meaning the permission management screen **lies**, on
exactly the screen where a lie does the most damage.

The display risk is accepted (it sits inside the documented threat model: someone
who can edit the file has already taken the account). What must be walled off is
its influence on the **DECISION**: `tra()` reads only the fingerprint, never the
description. The test rewrites the on-disk description to a completely different
scope and demands the decision **not change**.

**B38 — a destructive button the size of the screen is a trap.**

In a vertical box, children stretch to full width by default. The "Forget this
app" button filled the screen — easy to hit by accident, and it erases what the
user decided. Same root as the stretched 8×8 image bug: **a minimal stylesheet
still has default behaviour, and default behaviour still has to be considered.**

**B34/B35/B36 — the `tcc-goi:` server, a NEW path into package content.**

Before it, images inside a package **never appeared**: `ui.json` declares relative
paths, and a document loaded with `with_html` has no base URL to resolve them
against. Fixing that meant adding a server that receives **URLs the page asks
for** — that is, opening a new attack surface. Three rules:

| Rule | Blocks |
|---|---|
| Paths go through the standard's own `check_path` | `../` escaping the package |
| Only files PRESENT in the signed tree are served | content outside the signature's scope |
| Content type from an extension ALLOWLIST | forcing the browser to treat a file as HTML |

Three details easy to miss, each with a test:

1. **Percent-decode BEFORE checking.** `%2e%2e%2f` is `../` in disguise. Without
   decoding, `check_path` cannot see it — but the browser can.
2. **Strip query and fragment before checking.** Otherwise the string checked
   differs from the string looked up.
3. **No SVG in the allowlist.** SVG runs script and embeds external resources —
   it is a document, not a picture.

**B36**: the permission dialog is a screen belonging to **the browser**, so it
passes `|_| None` — the app cannot place a single byte inside it. Allowing that
would open a path to drawing over the warning itself.

Denials return an **empty 404** with no message: explaining would tell the page
what does and does not exist inside the package.

**B32/B33 — inspecting the REAL accessibility tree found two more bugs (2026-08-13).**

Once Accessibility permission was granted, the tree VoiceOver actually sees became
inspectable. Two findings, and **the first was caused by my own earlier patch**:

| # | Bug | Cause |
|---|---|---|
| 1 | Password fields exposed as a plain `AXTextField` | I added `role="textbox"` to satisfy the "every control carries an explicit role" invariant — **ARIA overrides native semantics** and downgraded `AXSecureTextField` |
| 2 | The destructive button carried no signal at all | `aria-description` **does not reach** the macOS accessibility axis |

Bug 1 was the expensive lesson: an invariant I added to increase safety **broke
the very thing it was meant to protect**. The first rule of ARIA — *don't use
ARIA when the native element already says the right thing* — I knew it and
violated it anyway, chasing a number I could count.

Fixed invariant: **input fields must carry NO `role`**; every other kind does.
Toggles keep `role="switch"` because that is ARIA **upgrading** a checkbox into a
switch — the correct use.

Bug 2 was fixed with `title` (→ AXHelp) plus `aria-roledescription`
(→ AXRoleDescription). Note that `aria-roledescription` **replaces** the role
name, so its string must state that this is a button — otherwise the user loses
that information.

Measured after the fix, this is what VoiceOver announces:

```
"Gõ thử tiếng Việt, text field"
"Ô bí mật (chữ phải bị che), secure text field"
"Tải trang mẫu, button"
"Xoá dữ liệu, nút — hành động không hoàn tác được"
```

**B30/B31 — three bugs 211 tests missed and one screenshot caught.**

On 2026-08-13, once Screen Recording was granted, the window could be captured
for the first time. The image exposed three things:

| # | Bug | Why the tests were blind |
|---|---|---|
| 1 | The permission toggle was an **empty square**; its label existed only in `aria-label` | The accessibility tree HAD the label, so the accessibility check passed cleanly |
| 2 | Input fields the same — the same class of bug | As above |
| 3 | A `Tone::Danger` button looked **identical** to an ordinary one | The test checked that the `data-sac-thai` attribute was present, not that it did anything |

Bug 1 was the worst: in the permission dialog, the toggle **is** the decision
control. A sighted user saw a blank square with no text and could not tell what
they were switching on — while a screen reader heard everything. The whole
capability layer was meaningless to anyone looking at it.

Bug 3 broke B10: apps declare intent and the renderer decides appearance — but
the renderer **had no stylesheet**, so every declared intent was drawn
identically. A minimal `BANG_KIEU` was added: every declarable intent must have a
visible manifestation.

**The general lesson**: checking the accessibility tree proves *a blind user can
hear it*. It does **not** prove *a sighted user can see it*. Two different
claims, and I had assumed the first contained the second.

**B29 — trust-on-first-use key pinning, and its limits.**

It does **not** answer "is this package really from publisher X" — no layer here
answers that yet. It answers a narrower question: **"is this the same signing key
as last time?"** That narrow question catches exactly one situation, and it is the
most dangerous one: a package bearing a familiar app id signed with an unfamiliar
key.

Previously the user saw the dialog reappear **as if for the first time**, with no
way to know the app had changed hands.

Two details worth stating:

**The text must be an observable fact, not a verdict.** "This app was previously
signed with a DIFFERENT key" — not "this app is fake". We do not know who is
right: the publisher may have rotated keys legitimately. The test
`khong_chuoi_nao_noi_da_xac_minh_nha_phat_hanh` now also forbids "giả mạo",
"lừa đảo", "is fake" and "is malicious".

**Position is part of the warning.** It sits immediately after the app name,
BEFORE the permission list. Put it at the end and the user has already read the
list with their hand on the button. The mutation "move the warning to the bottom"
is caught.

**B26/B27 — two ways to get "remember this permission" wrong.**

Asking on every launch is the fastest way to train users to click blindly, so it
must be remembered. But remembering it wrongly is worse than not remembering:

| Keyed by | What an attacker does |
|---|---|
| App id alone | Ship a package claiming `com.tcc.vi` → inherit every permission of the real wallet |
| Capability name alone | v1.0 requests `[shop]` and is approved; v1.1 requests `[shop, harvest]` → the old grant **covers the new scope by itself** |

So a record carries **both the signer's public key and a fingerprint of the
scope**. Change either and `tra()` returns `None`, meaning ask again.

The scope fingerprint uses **length prefixes**, following `tcc_spec::tree` —
without them `["ab","c"]` and `["a","bc"]` produce the same bytes, and two
different scopes sharing a fingerprint is a misgranted permission. A test pins it.

**What the permission store CANNOT protect against.** Anyone who can edit this
file already has access to the user's account — at which point they can read the
keystore and the browsing data too. The file is written 0600 and through a
temporary file then renamed, but that guards against half-written files and
against other users on the same machine, **not** against someone who has taken
the account. Stated explicitly so nobody assumes it is stronger than it is.

**B23 — a redirect is a CAPABILITY ESCAPE.**

The capability permits calling `shop.tcc-coin.com`. That server returns
`302 → attacker.example`. Any client that follows it has just let the app **reach
a server that was never granted** — while the capability gate in `tcc-runtime`
has already closed behind it with no way to know.

Blocked by two deliberate layers: `max_redirects(0)` in the configuration, **and**
our own code rejecting every 3xx status. The second layer is testable without a
real server, so it never rots. Verified against a real one: `http.badssl.com`
returns 301 and is refused exactly as designed.

**B25 — why `tcc-net` is its OWN crate.**

So that reading `Cargo.toml` is enough to see the app loader cannot open a socket:
`tcc-runtime` does not depend on `tcc-net`, it only calls through an injected
`trait Network`. Rule 8 in CI pins this.

The `mang` feature flag is separate, so a **network-free** build of the browser is
possible — useful during a security review: run that build and no packet can
leave the machine no matter what bug the code contains.

**Choosing `ureq` + rustls**: measured at 22 crates against `reqwest`'s 86 (which
drags in an async runtime). Rustls rather than OpenSSL — consistent with
`unsafe_code = deny`, and it avoids an entire history of vulnerabilities.

**B20 — why behaviour lives in the manifest, not in `ui.json`.**

Three reasons, each sufficient on its own: the signature covers the manifest, so
behaviour cannot be edited after signing; the manifest is what the permission
dialog reads, so it can later show "this button calls shop.tcc-coin.com"; and
declaring it in `ui.json` would mean `tcc-ui` has to know about the network —
and that crate must know nothing beyond the interface.

**B21 — catching INCONSISTENCY, not just violation.**

An app can declare a button calling `attacker.example` while requesting
capabilities only for `shop.tcc-coin.com`. At runtime the capability still blocks
it — but the user has already clicked, nothing happened, and **nobody knows why**.
Blocking it in `validate_shape` means `tcc verify` tells the app author while they
are still sitting at their machine. The host-matching rule here must be
**identical** to `tcc-capability`'s (exact, no subdomain matching) — a mismatch
between the two is a hole, and a test pins it.

**B22 — asserting "it was denied" is NOT ENOUGH.**

The test does not merely check that the function returns an error; it counts how
many times the network path was invoked and demands that number be **0**. Check
the permission after the call and the packet has already arrived — and for a
tracking server, arrival is the entire point; the response is irrelevant. The
mutation "call first, check second" is caught by exactly this test.

The network path is **injected from outside** (`trait Network`): `tcc-runtime` opens
no socket, so it is testable without touching a real network, and every path off
the machine is visible right at the call site — no hidden route buried in a
library.

**B17 — item-by-item consent, changed 2026-08-13.**

Previously a single "Allow" button granted **everything** the app requested. Now
each capability has its own toggle, default OFF, and `Allow` requires **two**
conditions: the allow button pressed **and** that specific capability's toggle
switched on. Pressing "Allow" with nothing switched on grants nothing.

Adding the `Toggle` component kind was **a change to the standard**. Exactly as
documented, `NodeKind` is not marked `#[non_exhaustive]`, so the renderer **does
not compile** until it handles the new kind. That was the price recorded up
front, now paid for the first time — and it behaved exactly as designed.

**B19 — a weakness exposed by MUTATION TESTING.**

The mutation: remove the `role === 'switch'` guard from the event-wiring script so
a toggle sends its message the moment it is pressed. The tests stayed GREEN —
because two clicks landed in the same tick and the later message **overwrote** the
earlier one, so the host still received the correct `cho-phep`.

Being overwritable is a real weakness, not merely a gap in a test: once a decision
is settled nobody may amend it, including the page itself. Fixed with
`o.is_none()` — the first decision wins. After the fix, the mutation is caught.

**B15 — a contradiction at the STANDARD level, found 2026-08-13.**

`tcc new` used to generate `entry: "index.html"`. It worked, and it violated the
central rule of the entire project: an app shipping HTML means that on the day a
GPU renderer exists, **every app has to be rewritten** — and at that point nobody
dares drop WebView. The scaffolding becomes the building. No test caught it
because nothing was "broken"; it surfaced while comparing the template file
against the written rule.

Fixed: the entry point is a **declarative component tree** (`ui.json`). The app
states *what is on screen*, the renderer decides *how it is drawn*.

**B16 deserves its own note — a trap nearly walked into.**

Putting `#[derive(Deserialize)]` straight onto `Node` compiles fine and **punches
through the entire validation layer**: `Node` keeps all its fields private
precisely so every node is born through a checking constructor, whereas direct
deserialization writes into the fields. Everything skipped: the depth ceiling, the
node-count ceiling, spoofing-character filtering, action-id constraints, the ban
on network-sourced images. An attacker need mount no attack at all — they ship a
JSON file.

So there are two types: `UiNode` is plain data for decoding, and `TryFrom` rebuilds
**through exactly those constructors**. Six tests pin it: sensational labels,
network images, malformed action ids, exceeding the node ceiling, exceeding the
depth ceiling, missing image descriptions — all rejected when arriving from JSON
just as when written by hand in Rust.

One detail worth recording: the bidi-character test must build its string **at
runtime**, because `rustc` **refuses to compile** a source file containing that
character (a defence added after "Trojan Source"). The compiler is enforcing the
same rule we enforce at runtime.

**B12 deserves its own note.** `hoi_quyen` does NOT return a `Result` — deliberately.
A `Result` is room for someone to write `.unwrap_or(Allow)`. Window closed, window
broken, dialog fails to build, unknown action id — all produce `Deny`. There is
exactly ONE path to `Allow`: the user pressing the right button. The decision
logic is split into a pure `quyet_dinh()` so it is testable without a screen, and
the test enumerates the near misses too (`"cho-phep "`, `"Cho-Phep"`,
`"cho-phep-tat-ca"`) — every one must produce `Deny`.

**B13 deserves its own note.** An app declares only an `ActionId`; it has no line
of script and may not have one. The event-wiring script runs at the initialization
stage, so it is the **renderer's** script — the app has no way to inject into it.
That makes "the app runs code when the user clicks a button" something that cannot
happen.

**B14, and a hole in the test itself.** The allowlist accepts only actions that
actually exist on the tree. But the original test sent only VALID actions, so when
the allowlist was removed as an experiment **every test stayed green** — loosening
a filter is the mutation class that valid-input-only tests never touch. Closed
with the `kiem-bam-nut ma` mode: send a fabricated id directly and require that
nothing arrives.

**B7 is checked on a second WebKit implementation, best-effort.** WKWebView on
macOS enforces it strictly; WebKitGTK on Linux under a virtual display runs the
same three examples on every push but cannot fail the build. One engine proves a
defence against one engine, and a defence resting on a WKWebView quirk would
look identical to a real one until something else ran it — so the second engine
is worth having even unenforced.

Why unenforced: all three passed the first time, the flag came off, and the next
run failed with *"the underlying handle is not available"* — WebKitGTK under
xvfb does not reliably get a usable window handle. That is a property of the
virtual display, not of the defence. **One green run is not evidence of
reliability**, and treating it as such is how a flaky check gets promoted to a
blocking one.

**B7 deserves its own note — three layers, each tested ALONE.**

| Layer | Blocks | How it is tested alone |
|---|---|---|
| 1. Escaping | `<script>` never becomes a tag | Remove `"` escaping → the test goes red |
| 2. Accessibility scanner | A document that cannot be read back is NOT loaded | Remove `<` escaping → the scanner refuses |
| 3. Content policy | Script that is present still does not run | Load a RAW hostile document, bypassing layers 1–2 |

Layer 3 must be tested separately because in the real pipeline it is **never put
to the test** — layers 1 and 2 stop everything first. A defensive layer that has
never been exercised is a layer nobody knows exists. Run:

```sh
cargo run -p tcc-shell --features window --example kiem-khoi-tan-cong          # full pipeline
cargo run -p tcc-shell --features window --example kiem-khoi-tan-cong chi-csp  # layer 3 only
```

**B8 deserves its own note.** `published_accessibility()` is trivially easy to
implement dishonestly: return `tree.accessibility_tree()` and it always passes.
The real renderer does NOT do that — it rebuilds the tree from **the very markup
about to be loaded into WebView** (`a11y_scan.rs`), so the two trees arrive by
different routes. The scanner also checks that visible text matches the announced
label: a button reading "Cancel" that announces "Confirm" is the exact deception
the accessibility layer exists to prevent.

**B3 deserves its own note.** It is held not by discipline or code review but by
**the compiler**: `NetworkCapability` has all-private fields, so it cannot be
constructed outside its crate. No permission means no value; no value means no
compile. "Forgetting a check once" cannot happen.

---

## 2. Holes found by re-reading, and fixed

The holes below were **not found by testing** — every test was green while they
were present. They surfaced while re-reading the code and asking "what would an
attacker do".

| # | Hole | How it is exploited | Fixed in |
|---|---|---|---|
| L1 | The same capability requested twice | The first entry looks harmless to whoever reviews it; the second is the one actually granted | `tcc-spec` + `tcc-capability`, **blocked at both layers** |
| L2 | No size ceiling on `manifest.json` | Send hundreds of MB and we parse all of it **before** the signature can be checked | `MAX_MANIFEST_BYTES`, checked at **step 0** |
| L3 | `name` / `reason` not filtered | `U+202E` reverses the text so the permission dialog reads as something else | `check_display_safe` |
| L4 | Non-ASCII hostnames | `shоp.tcc-coin.com` with a Cyrillic "о" looks identical to the real one | ASCII/punycode required |
| L6 | `grant()` did not itself block duplicate declarations | A caller forgetting `validate_shape` lets it through | Blocked inside `grant` |
| L7 | Character classification branches in the wrong order | `\r` fell into the wrong range; the later branch was never reached | Specific branches before wide ranges |
| L8 | **App id not validated when decoding JSON** | Ship `id: "com.TCC.hello"` — two identities that look identical | `AppId::parse` inside `validate_shape` |
| L9 | **Hostname shape not validated** | `shop.tcc-coin.com:8080@evil.example` — userinfo spoofing | `check_host` |
| L10 | **Unlimited combining marks on one character** | 500 acute accents draw a vertical streak over the warning | `MAX_COMBINING_MARKS` |

**L10 — stacked marks hiding the warning (2026-08-13).**

Combining marks have no natural limit. A button label of `"Huỷ"` plus 500 acute
accents passed every existing check — no control characters, no bidi override, no
zero-width characters. But the renderer drew it as a **vertical streak covering
the area above**, and in a permission dialog the area above is the identity
warning — the thing the user must read before pressing anything.

Banning combining marks outright is impossible: **Vietnamese lives on them.** So
the ceiling is 8 consecutive marks on one base character:

| | Max marks on one character |
|---|---|
| Vietnamese (`ỡ` = o + horn + tilde) | 2 |
| Thai, Devanagari — heaviest clusters | ~4–6 |
| **Ceiling** | **8** |
| UAX #15, for data interchange | 30 |

UAX #15 allows 30 because it is concerned with **interchanging** data; we are
concerned with **displaying** it on a screen where a security decision is made.

Uses `unicode-general-category` (1 crate, no dependencies) rather than guessing
codepoint ranges — guessing misses cases, and a miss here is a landed attack.

**Mutation-tested in both directions** — this is the correct shape for a ceiling:
loosen it and the Zalgo test goes red; tighten it to 1 and the Vietnamese test
goes red. Blocking Zalgo is easy; blocking it **without killing Vietnamese** is
the hard part.

Found while preparing for Phase 1's "type Vietnamese with diacritics" gate, by
asking: *if combining marks pass, how many can stack?*

**L9 — the userinfo spoofing attack (2026-08-13).**

Previously `hosts` was checked only for ASCII, non-empty, and no wildcards. A
manifest could declare:

```json
"hosts": ["shop.tcc-coin.com:8080@evil.example"]
```

That string passed every existing check. But when a URL is built,
`shop.tcc-coin.com:8080` becomes the **userinfo** component and the REAL host is
`evil.example`. The permission dialog displays the whole string, and a reader
skimming it sees "shop.tcc-coin.com".

**Why no test came near it**: every test used a well-formed hostname. The hole
surfaced only when asking *"the hostname goes straight into URL construction —
has its shape been checked?"* — right before writing the HTTP client. Written in
the other order, the hole would have been living in a build that had networking.

Blocked on **both paths**: `Scope::Network.hosts` and `Effect::Fetch.host`. Block
one and the other still builds a URL pointing somewhere else.

**L8 — found by the conformance suite, not by unit tests (2026-08-13).**

`AppId` declares `#[serde(transparent)]`, so decoding from JSON takes the string
directly and **never passes through `AppId::parse`**. `validate_shape` did not
re-check it either. A package could ship `id: "hello"` (missing a segment) or
`id: "com.TCC.hello"` — and ids differing only in case are two identities that
look identical, exactly what `AppId::parse` exists to prevent.

**Why 34 unit tests were completely blind**: they always built `AppId` via
`AppId::parse`, so not one of them travelled the JSON decoding path. The
conformance suite loads manifests from JSON **like a real user**, so it hit it
immediately.

This is the **same class of hole** as B16 (interface-tree decoding bypassing the
constructors) and as B40. The broader lesson: **wherever a type protects an
invariant with a constructor, ask "does decoding go through that constructor?"** —
and serde's default answer is NO.

**L5 — suspected but NOT a hole.** I suspected duplicate JSON keys might let a
display tool and a signature verifier see two different values. Verified with
running code: `serde` rejects them outright with a `duplicate field` error. No
defensive code was added for a problem that does not exist — but the behaviour is
**pinned by a test**, `khoa_json_trung_lap_bi_tu_choi`, because we now depend on
it and a future change of JSON library must break that test.

**L7 was caught by clippy, not by me.** My tests only tried `\n` and `\u{0}`, so it
never surfaced. This is why CI runs `clippy -D warnings` **before** the test step.

---

## 3. ⚠️ What has NOT been examined — read this section carefully

**A dependency grants camera and microphone to any page, and we cannot stop
it in code.** wry 0.52.1 hardcodes `WKPermissionDecision::Grant` for
`requestMediaCapturePermissionForOrigin` on macOS
(`wkwebview/class/wry_web_view_ui_delegate.rs:74`) and exposes no way to
override that delegate. At tier 2 — arbitrary web pages — a page calling
`getUserMedia()` is granted without anyone being asked, which contradicts
architecture decision 2 as directly as anything in this codebase.

The only remaining barrier is the operating system: with no
`NS*UsageDescription` string in `Info.plist`, macOS refuses the app access to
the camera and microphone, so wry's "Grant" has nothing to grant. That makes
an omission load-bearing, which is a fragile place to stand — someone adding
one line to a plist for an unrelated feature removes it. Architecture rule 20
now watches for exactly that declaration, and the packaging script fails if it
appears.

This is a mitigation, not a fix. The fix is upstream: wry has to let the
embedder answer that request, and then TCC has to answer it with a real
permission dialog. Until then, tier 2 has no per-site camera or microphone
consent, only per-app OS consent, and it is listed here rather than in a
release note.


**Vietnamese input was typed into the real application screen for the first
time on 2026-08-17**, by a person, into `tcc-browser examples/hello-tcc`.
Before that day the application screen was never drawn at all, so every
earlier check of the input method went through the permission dialog or a
diagnostic probe instead. The string was `chào buổi sáng bạn iu`, and the
hard case in it is `ổ` — two stacked marks, circumflex then hook — which
landed correctly, with no orphaned combining marks and the caret at the end.

This still needs a human every time: every simulation pushes an already
composed string into the field, which skips precisely what is being measured.


**The product binary's window path is exercised by hand only.** Until
2026-08-17 `tcc-browser <package>` verified the signature, asked for
permissions, printed three lines and exited — it never called `run_app`, so
no application screen was ever shown. The first run looked right because the
permission dialog appeared; every run after that had a stored answer, showed
nothing, and gave no clue why. Nothing caught it, because opening a window
needs a display and CI has none. `tcc-browser hop-thoai <package>` with
`TCC_KIEM_KHOI=1` is the closest automated substitute and only covers the
permission dialog.


### 3.1 A signature proves INTEGRITY, not IDENTITY

The public key sits inside the manifest — packages are **self-signed**. Anyone can
generate a keypair and sign their own package.

Answering "does this key really belong to publisher X" requires another layer: a
registry, or trust-on-first-use with key pinning. **That layer does not exist in
0.1.**

> **Rule for the interface:** never display "verified publisher" when only the
> signature has been checked. The correct sentence is "valid signature".

This rule now has **an enforcing test**:
`text::kiem_thu::khong_chuoi_nao_noi_da_xac_minh_nha_phat_hanh` scans the whole
translation table for six forbidden phrases in both languages. Anyone adding a
violating string is stopped immediately, even without having read this file. The
permission dialog also always shows "Unknown publisher / Không rõ nhà phát hành"
alongside the full two-part warning.

### 3.1b The permission dialog is drawn through WebKit — MEASURED, 2026-08-14

This debt used to be recorded as a vague worry. It has now been measured, and it
is **much smaller** than the way it was written down:

| Measured | |
|---|---|
| One WebView window | spawns its **own WebKit content process**; closing the window destroys it |
| The permission dialog and the app screen | **never exist at the same time** |

The second is not luck: `tao` runs **one event loop at a time**, and `hoi()` uses
`run_return`, so it returns *after the window has closed*. The app screen opens
only afterwards. The single-event-loop architecture forbids two live windows by
construction.

**⚠️ What would break this guarantee**: opening the dialog as a child window of
the app window, or moving to a multi-window loop. Anyone attempting either must
re-read this section first.

**Residual risk, now narrowed**: both processes are still WebKit. A sandbox escape
from app content could persist and affect a dialog opened later. That is what a
native widget would fix — and only that.

**Why not now**: it belongs to Phase 4 of the plan ("leave WebView"), and Phase 1
is not closed. Doing it now means adding `unsafe` FFI, covering macOS only, and
rebuilding the entire accessibility layer just built — on top of the very thing
scheduled for replacement.

### 3.1c ⚠️ Window title spoofing — found while MEASURING the debt above

An app declares its own `name`, and that name used to be the **entire** title of
its window. An app named `"TCC — granted permissions"` gets a window titled
identically to the browser's own permission management screen — and can then draw
a fake permission list with a fake "Allow" button inside it.

Stopping it from choosing that name is impossible (the name is the app's own
text), but stopping that name from **occupying the whole title** is not. Now:
`com.tcc.vi-du.hello — Xin chào TCC`.

The app id cannot be faked: it sits inside the signature's scope and `AppId::parse`
constrains it to `a-z0-9.` — no spaces, no em dashes, so it **cannot imitate** a
browser title. Tests pin both directions: a spoofing name cannot occupy the front
of the title, and the browser's own title does not look like an app id.

This is **not a complete answer** to title spoofing — no complete answer exists in
software. It blocks the cheapest attack.

**Two places this fix had not reached, found 2026-08-19.**

The permission dialog — the window where the user presses *Allow* — took the
app's declared `name` as its entire title. So the rule protected the app's own
window while leaving the browser's own window open to the same imitation, and
that window is the more valuable one to imitate. Its title is now a browser
string that never contains app-controlled text.

The pixel renderer's first window passed `manifest().name` directly, reopening
the original hole on the new renderer. The cause is worth more than the fix:
`app_window_title` lived inside the module gated behind the WebView feature, so
a renderer that does not pull in WebView could not reach it. **A security rule
kept behind one renderer's feature flag is a rule that holds on one renderer.**
It now lives in `window_title.rs`, which is always compiled, and both renderers
route their windows through the shell rather than passing a title of their own.

The test that pins the dialog title compares strings, and a mutation showed that
is not enough: reverting the call site to `&m.name` leaves the string correct and
the test green. A second test reads the source of the calling function, and that
mutation dies.

**A third instance of the same class, found by looking for more.** The pixel
renderer described a destructive button to VoiceOver using
`AccessText::default()` — English, whatever language the user had chosen — while
the WebView path injects the translated sentence. A Vietnamese VoiceOver user
heard English, and the two renderers described the same button differently,
which is the exact failure the cross-renderer check exists to catch. The
sentence is injected now, and both renderers read it from one key.

The first test for it compared the two functions and the mutation survived,
because hardcoding English at the call site leaves both functions agreeing.
Comparing a function against a function is not a substitute for looking at where
it is used.

### 3.1d Assistive activation is accepted, and refusing it protected nobody

The pixel renderer's accessibility adapter accepts `Action::Click` from the
platform, so a VoiceOver user can press a button. The previous version refused
every action on the reasoning that accepting "press this button" opens a path to
pressing buttons without a mouse, and that on the transaction confirmation
screen this is a path to signing on the user's behalf.

That reasoning compared against a world that does not exist. On macOS an
application must be granted **Accessibility** permission in System Settings
before it can send `AXPress`, and that same permission allows `CGEventPost` —
synthesising a real mouse click, which travels through our ordinary mouse path
and needs no accessibility API at all. Refusing `AXPress` therefore stops no
attacker: they already hold an equivalent path behind the same gate. It stops
only VoiceOver users, who are the people the feature exists for.

**A control that obstructs legitimate users without obstructing an attacker is
not a security control.** It is a barrier, and removing it costs nothing an
attacker was paying.

The requests do not get their own code path. They are queued and drained through
the same state machine a mouse click goes through, so every rule the permission
dialog enforces for the mouse — a toggle does not dismiss, only a button ends the
screen, closing grants nothing — applies unchanged. Only `Action::Click` is
accepted; scroll, focus and set-value are ignored rather than guessed at. A
request whose node id is no longer in the tree is dropped, because the tree is
rebuilt whenever a toggle flips and a stale id would otherwise resolve to a
different button.

**Residual risk, stated rather than closed**: a user who grants Accessibility
permission to a malicious application has given it control over every window on
the machine, not only ours. That is an operating-system trust decision and no
function in this codebase can take it back. It is written here rather than
papered over with an empty handler that looked like a defence.

### 3.2 `ml-dsa` is still at 0.1.1

No independent audit of this library has been published. It was chosen for being
pure Rust (consistent with `unsafe_code = deny`) and for sharing a trait system
with the rest.

Risk is reduced two ways: the signature is **hybrid** — Ed25519 still stands if
ML-DSA fails — and it sits behind a `SignatureScheme` trait so the library can be
swapped without touching the call sites.

### 3.3 ~~Content is ONE blob of bytes~~ — RESOLVED 2026-08-13

`verify_package` now takes a `&FileTree` and hashes the **canonical form** defined
in `tcc_spec::tree`:

```text
for each file, in byte order of its path:
    u64 path length (BE) ‖ path ‖ u64 content length (BE) ‖ content
```

**Writing a length before every field** is what blocks the ambiguity attack: a
file `"ab"` containing `"c"` and a file `"a"` containing `"bc"` both concatenate to
`"abc"` — two entirely different trees, one signature valid for both. The test
`khong_trao_duoc_cay_khac_ma_giu_chu_ky` pins it.

Alongside it, `FileTree::insert` rejects: `..` (directory escape), absolute paths,
`\` (Linux sees one file, Windows sees two levels), colons (Windows drive
letters), control characters, and **names differing only in case** — because on
macOS and Windows `Logo.png` and `logo.png` are the same file, so a package
containing both unpacks differently depending on the operating system: one
signature, two outcomes.

**Streaming, since 2026-08-15.** `canonical_bytes` built the whole package in
memory. Measured: 64 MiB of content cost 128 MiB of RSS, and the content cap is
**256 MiB** — half a gigabyte for a package nobody has authenticated yet.

`FileTree::for_each_canonical_chunk` feeds the hasher without the copy; the same
64 MiB now costs nothing beyond the tree itself — measured at 128 MiB against 0 MiB
of additional RSS, each path in its own process, because an allocator does not
return memory to the operating system promptly enough to measure both in one. `canonical_bytes` stays,
because the conformance suite needs the literal bytes to compare against.

The two paths **must** produce identical bytes — a one-byte difference is a
different hash, which is each side unable to verify the other's signatures. A
test pins the equivalence, and the conformance runner now computes every
`canonical` vector both ways and requires the results to match, so an
implementation that streams is checked rather than assumed.

`tcc-spec` is a leaf crate and may not depend on `tcc-crypto` (rule 3), so the
chunking lives in `tcc-spec` and the hasher in `tcc-crypto` — the layering rule
decided the shape of the fix.

### 3.4 Side channels — measured 2026-08-15, one finding

`crates/tcc-crypto/examples/do_thoi_gian.rs`, 300 iterations each, median:

| Input | Time |
|---|---|
| Valid signature | 223 µs |
| **Classical half wrong** | **35 µs** |
| Post-quantum half wrong | 221 µs |

**Timing tells an observer which half failed.** `verify` checks Ed25519 first and
returns on failure, so the ML-DSA half never runs — a 6× difference, far too
large to hide behind noise.

**Not fixed, and the reasoning matters more than the verdict.** What an attacker
learns is which half of a signature *they themselves supplied* is wrong, which
they already know. Removing the leak means verifying both halves always, making
rejection of garbage 6× more expensive — trading a leak of near-zero value for a
real amplification factor on unauthenticated input. If a use ever arises where
the attacker does **not** already know the answer, this decision must be revisited.

Signing showed 516 µs against 644 µs for an all-zero versus an all-`0xff` secret
key. That ratio is within what this crude harness can attribute to noise, and
**it is not evidence of constant time**: this measurement runs on a laptop with
turbo and thermal scaling. It is a screen for large differences, not a proof.
Establishing constant time needs quiet hardware or instruction-level analysis,
and remains undone.

### 3.4b The signing key file, and two things fixed on 2026-08-15

`tcc key` wrote the secret with `fs::write` and set the mode to 0600 afterwards.
Between those two calls the file sat on disk with the umask default — typically
0644, readable by every account on the machine. The window is short and it opens
onto the most sensitive file in the system. It is now created with
`create_new(true)` and mode 0600 in one call, which also removes the separate
`exists()` check that could race. A test asserts the mode of the file as created
and that a second `tcc key` refuses to overwrite.

`tcc sign` now warns when signing with the demo key. Rule 9 already blocked that
key from appearing in a manifest outside `examples/`, but that rule only runs
**inside this repository**. Someone who clones it, finds a key file sitting
there, signs, and publishes gets no warning at all — and everyone in the world
holds the same key. The right place to say so is while they are typing the
command.

The denylist entry is embedded at compile time from the example package, so the
two cannot drift apart. Rule 9 was widened to scan every file rather than only
manifests — it had been blind to the key appearing in source — and it now reads
the demo key from the example package instead of carrying a hardcoded copy that
would go stale the day the example is re-signed.

### 3.5 No wallet code touches a real private key yet

As planned. The hard gate:

> **No transaction reaches mainnet before an independent security audit.**

### 3.4c A signed package does not survive a helpful transport — found 2026-08-15

The committed example verified on macOS and Linux and **failed on Windows** the
first time CI ran there:

```
✗ chữ ký không hợp lệ: chữ ký Ed25519 không hợp lệ
```

The package was never broken. Git converts LF to CRLF on checkout under Windows
by default, and a signature is computed over the **raw bytes** of
`manifest.json` — 4,682 bytes became 4,713, one per line, and the signature no
longer matched. The content hash over `content/` broke the same way.

Fixed here with `.gitattributes` (`* -text`), and the rule covers every file
rather than only packages, because a rule that must be remembered for each new
signed file is one that gets forgotten.

**The lesson generalises past git, and belongs to implementers rather than to
this repository.** Any channel that normalises text destroys a package: an
archive tool with a text mode, a chat client, an editor that "fixes" line
endings on save, a CI step that runs a formatter. This is recorded in
[`spec/0.1/01-package.md`](spec/0.1/01-package.md) so anyone building
distribution for TCC packages meets it before their users do.

### 3.5b Dependencies, checked against RustSec from 2026-08-15

`cargo audit` runs in CI. It fails the build on a vulnerability and reports
maintenance warnings without failing, because none of them can be fixed from
here.

Current state (re-checked 2026-08-23, after the web engine was removed):
**0 vulnerabilities**, 12 warnings across 452 dependencies. Both numbers fell —
468 → 452 dependencies, 15 → 12 warnings — and the fall is the point of the entry
below about where GTK comes from.

| Warnings | Where they come from |
|---|---|
| 9 GTK crates — `gtk`, `gdk*`, `atk*`, `gtk3-macros` (unmaintained) and `glib` (also unsound) | **`tao`**, the windowing library, on **Linux** only. Until 2026-08-23 this row said `wry`'s WebView backend; `wry` is gone and the row did not empty, because `tao` needs GTK to open a window at all. Removing the web engine did not remove GTK — it removed the *reason to believe* GTK left with it, which is why this was re-measured rather than assumed |
| `proc-macro-error` | Build-time code generation; not in the shipped binary |
| `ttf-parser` (unmaintained) | `cosmic-text` → `fontdb`. **New**, and it arrived with the pixel renderer: text shaping needs a font parser, and that parser now sits between a signed package and the screen |

`fxhash` and `rand 0.7.3` left the tree with the web engine.

### How much `unsafe` is under us, measured (2026-08-25)

§3.5b below traces one crate's chain by hand. That answers *how bad is this
one*; it does not answer *how much is there*. `tools/dem-unsafe.sh` answers the
second, and can be re-run.

It builds `tcc-shell` into a clean target directory and reads the `.d`
dep-info files rustc leaves behind — the exact list of `.rs` files that were
compiled. Nothing is inferred about `cfg` or about platforms.

**Host `x86_64-apple-darwin`, 1647 source files compiled:**

| | Crates | Places using `unsafe` |
|---|---|---|
| Third-party, **in the binary** | 80 (35 with none) | **1597** |
| Third-party, build-time only (proc macros and their tree) | 12 | 148 — never linked |
| Our own code | 10 | **0** |

Heaviest in the binary: `bytemuck` 315, `memchr` 241, `memmap2` 93,
`slotmap` 89, `generic-array` 78, `smallvec` 71, `zmij` 69, `libc` 60,
`swash` 54, `arrayvec` 52.

On the path a signed package's bytes actually travel:

| Crate | Places | What it does there |
|---|---|---|
| `swash` | 54 | Rasterises glyph outlines — **the largest block on this path** |
| `libm` | 45 | Float math under shaping; receives numbers, not font bytes |
| `blake3` | 41 | Hashes the package |
| `fontdb` | 3 | Font enumeration |
| `cosmic-text` | 1 | Layout |
| `ttf-parser`, `ed25519-dalek`, `core_maths` | 0 | — |

**This measurement was wrong twice before it was right, in the same direction
each time: too big, and too confident.**

1. Counting the *word* `unsafe` scores `#![forbid(unsafe_code)]` as unsafe
   code. It gave `ttf-parser` — a crate that forbids it — a 2. Narrowing to
   `unsafe {` / `fn` / `impl` / `trait` / `extern` returns 0, agreeing with the
   by-hand count already in §3.5b.
2. Counting from `cargo metadata`'s full resolve graph includes dependencies
   for platforms never built here: **`r-efi`**, UEFI firmware bindings, ranked
   third-heaviest in a desktop browser.
3. Filtering by platform but still counting each crate's whole `src` gave
   `blake3` **227** — and this document, one draft ago, called it *"the largest
   single block of `unsafe` on the verification path"*. It compiles 11 files
   here, not the SSE/AVX/NEON/wasm variants sitting behind `cfg`, and its real
   figure is **41**. `libc` fell 667 → 60 the same way. The honest total fell
   2829 → 1597. The crate that actually holds the largest block on that path is
   `swash`, which the inflated count had ranked *below* blake3.

   That draft also measured `aarch64-apple-darwin` while the toolchain here is
   `x86_64-apple-darwin`. Wrong platform, no symptom.

⚠️ The number is a per-line **upper bound** — two `unsafe` blocks on one line
count once — and it is a size, not a verdict: it says nothing about whether any
of it is *wrong*. Our own 0 is the default feature set; the one documented
exception (macOS `NSView` → AccessKit) sits behind `accesskit-platform` and is
not in this build.

The script exits non-zero rather than print a total when the build did not run
or the dep-info lists nothing — a smaller number from *"did not read it"* is
the failure this document keeps running into.

### `ttf-parser`, looked at properly (2026-08-23)

The paragraph that stood here flagged this crate for a second look and then did
not take one. Taken now — and split into what was **read** and what is
**recalled**, because a security document that mixes the two teaches its reader
to trust the wrong half.

**Read from the vendored source, the lockfile, and the local advisory database.**
Anyone can re-run these and get the same answer.

| Claim | Where it was read |
|---|---|
| Authors: Caleb Maclennan, Laurenz Stampfl, Yevhenii Reizner, Khaled Hosny | `authors` in the vendored `Cargo.toml` of 0.25.1 |
| Repository is `github.com/harfbuzz/ttf-parser` | `repository`, same file |
| Unmaintained per the author; successor named as `skrifa` | `advisory-db/crates/ttf-parser/RUSTSEC-2026-0192.md`, dated 2026-06-28, citing harfbuzz/ttf-parser#217 |
| `#![forbid(unsafe_code)]` | `src/lib.rs:36`; grepping all of `src` finds **zero** real `unsafe` |
| ~23k lines across 66 files | counted in `src` |
| Reaches us through `cosmic-text` → `fontdb`; we do not depend on it directly | `cargo tree -i ttf-parser` |

**Recalled, NOT verified here.** That Yevhenii Reizner (`RazrFalcon`) is the
*original* author and also wrote `resvg`; that the repository *moved* to the
harfbuzz organisation from somewhere else; that Khaled Hosny maintains HarfBuzz.
The `authors` field records who is named, not who wrote what, nor in what order.
None of this changes the risk; it is separated so nobody cites it as checked.

**The `unsafe` claim, corrected.** An earlier draft of this section said a
parsing bug "gives a panic, not memory corruption". That was stated more
strongly than the evidence. Following the whole chain:

`ttf-parser` (0 `unsafe`, `forbid`) → `core_maths` (0) → **`libm` (57 `unsafe`)**

So: **the parser itself** contains no `unsafe`, and `forbid` cannot be lifted by
an inner `allow`. Underneath it there is `unsafe`, in a float-math library that
receives numbers rather than font bytes. That is a meaningfully better position
than an unsafe-heavy parser — it is not the same as "no unsafe on the path", and
this section should not have said so.

**What still matters.** Fonts come from the operating system, not from a package.
"Unmaintained" means the next bug found stays unfixed, and a panic inside the
draw path takes the window down while a user is reading a transaction they are
about to sign. Availability, not integrity.

**What we cannot do about it.** Switching to `skrifa` is `cosmic-text`'s
decision, not ours: we would have to change text engines to change font parsers.
Worth knowing before an auditor asks why we did not simply swap it.

### 3.5c Dependency licences, and the MSRV claim

Both were claims nobody checked, which is the pattern this document keeps
finding.

**Licences across all 356 dependencies**: no strong copyleft, none undeclared.
The bulk is `MIT OR Apache-2.0`. Four crates are **MPL-2.0** — `cssparser`,
`cssparser-macros`, `dtoa-short`, `selectors`, all reached through `wry` — which
is file-level copyleft: distributing them is fine under Apache-2.0, but a
modified MPL file stays MPL. One crate offers LGPL among its options
(`r-efi`, `MIT OR Apache-2.0 OR LGPL-2.1-or-later`), and the permissive option
is the one taken.

**MSRV**: `Cargo.toml` declared `rust-version = "1.90"` and no job ever built
with it. A dependency raising its own minimum would have broken that quietly for
anyone not on the newest toolchain. Tested and true, and CI now checks it — the
job reads the version out of `Cargo.toml` rather than repeating it, so raising
the minimum in one place cannot leave the check testing the old one.

### 3.6 Fuzzing — exists now, and its limits are worth stating

`tools/tcc-fuzz` fuzzes the three parsers. They are worth fuzzing for a reason
that follows directly from the order inside `verify_package`:

```text
0. size ceiling
1. serde_json::from_slice::<Manifest>   ← here
2. manifest.validate_shape()            ← and here
3. compare scheme name
4. VERIFY THE SIGNATURE                 ← only now is anything authenticated
```

Steps 1 and 2 **must** precede step 4: the public key lives inside the manifest,
so it cannot be read without decoding it first. The unavoidable consequence is
that **the parsers run on entirely unauthenticated input.** An attacker needs no
valid signature, no key, nothing — only a file that reaches us.

It checks five targets, and asks for more than absence of panics:

| Target | Property | Depth |
|---|---|---|
| `Manifest` | Accepted → serialise → parse again must yield an **equal** value that still validates. Accepting something we cannot reproduce means state survived validation that nothing describes. | 3.4% |
| Interface tree | Accepted → the wire form must round-trip **without changing the verdict**. This is exactly the B16/L8 seam between the wire type and the checked type. | 1.9% |
| File tree | The canonical form must be **deterministic** — two implementations hashing differently is two implementations unable to verify each other's signatures. | 7.6% |
| **Signature** | **No mutation of a valid signature may verify.** One that does is a forgery. | 66.7% |
| **Public key** | **No other public key may verify the same signature.** | 50.6% |

The last two matter most. `ml-dsa` 0.1.1 has no published independent audit
(§3.2), and it parses bytes an attacker fully controls: the signature comes from
the package, and the public key comes from the manifest, which is decoded before
anything is authenticated.

**"Depth" is measured because "no findings" is meaningless without it.** The
first version of the signature target reported **0% depth**: a hybrid signature
is exactly 3373 bytes, and every mutation that inserts or deletes a byte dies at
the length check without touching a line of cryptography. The fix was a
length-preserving mutation mode for fixed-size inputs. Without that measurement
the target would have reported "no findings" forever while testing nothing.

**What it is not.** It is not coverage-guided. It mutates from a seed corpus
without measuring which branches were reached, so libFuzzer or AFL would go far
deeper. In exchange it runs on stable Rust with no added dependency, is
deterministic given a seed, and therefore runs in CI on every push with a seed
derived from the commit SHA — new inputs every commit, and any failure still
reproducible from the seed printed in the log.

**Measured depth**, because "no findings" is meaningless without it: roughly
3.4% of inputs get past the JSON decoder into the manifest checks, 1.7% into the
interface tree, 7.8% into the file tree. Two million rounds across eight seeds
found nothing.

**The harness was tested by injecting bugs**, because a fuzzer nobody has seen
catch anything is not evidence:

| Injected | Found? |
|---|---|
| `&self.version[..1]` in `validate_shape` — the classic Rust parser panic on a multi-byte first character | Yes, with a reproducing input |
| Deleting the ML-DSA verification step, so only the Ed25519 half is checked | Yes — both crypto targets reported a different key verifying the same signature |
| A panic on the signature-parsing path | Yes, on the truncated and empty seeds |

`sign` is fuzzed too, with malformed secret keys. That key never comes from an
attacker in normal use — but it comes from **a file on disk** (`tcc sign --khoa`),
and truncated files, wrong-version files and corrupt files are ordinary. The
property is stronger than "does not panic": if signing succeeds, the signature
must **verify against the key derived from that same secret**. Producing a
signature you cannot verify yourself is a silent failure that only shows up on
somebody else's machine.

**The corpus loader silently lost 87% of its seeds.** When the vector files were
renamed from `truong_hop` to `cases`, the loader kept reading the old keys and
dropped from 55 seeds to 7 — and the suite went on reporting PASS for several
runs while having almost nothing to mutate. A fuzzer that loses its corpus does
not complain; it just stops finding things. There is now a minimum-seed assertion,
and it is checked **before** the panic hook that silences target crashes is
installed — the first version asserted after, so it died in complete silence.

**Coverage-guided fuzzing, added 2026-08-15.** `fuzz/` holds three libFuzzer
targets for the same parsers, run nightly by `.github/workflows/fuzz.yml`. Roughly
50,000 executions per second against about 20,000 for the byte-mutation fuzzer,
and it grows its own corpus — 35 seeds became 6,700 inputs in one minute.

**The dictionary is what makes it work, and finding that out was the point.**
Tested by injecting the same `&self.version[..1]` bug:

| Run | Result |
|---|---|
| No dictionary, **2,054,596 executions** | not found |
| With `fuzz/tcc.dict`, 40 seconds | found |

Random byte mutation almost always breaks the JSON, so the whole budget goes
into exploring `serde_json` rather than the rules in this standard. The
dictionary supplies the format's own tokens plus the byte sequences that were
real holes here — bidi override (L3), combining marks (L10), `@` (L9), `..`. The
crashing input it finally produced had `U+200B` at the front of `version`, taken
straight from the dictionary.

Worth stating plainly: **the crude fuzzer found this bug in 20,000 rounds and
libFuzzer missed it in two million.** Coverage guidance is not a substitute for
knowing which byte sequences have hurt you before. The two run side by side —
the cheap one on every push, this one on a schedule.

---

### 3.7 What was REMOVED on 2026-08-23, and what went with it

The first renderer — a web engine embedded through `wry` — was deleted. That is
a deliberate architectural choice, not a security fix, and it removed real
defences along with the thing they defended. Listing them is the point of this
section: a defence that quietly disappears is worse than one that was never
built, because the threat model still reads as if it were there.

**Gone, with their reason:**

| Defence | What it stopped | Why it went |
|---|---|---|
| Markup escaping of app-supplied text | App text becoming markup — the worst possible failure, since script in the renderer's context escapes the whole capability model | There is no markup. Text is a node's content and is drawn as glyphs; nothing parses it |
| Content-Security-Policy on the app document | Script surviving even if escaping failed — the second of three layers | There is no document and no script engine |
| `kiem-khoi-tan-cong` — a hostile manifest driven through a real WebKit view, which then reported what it saw | The only check where a **third party** confirmed our own escaping, rather than our code reading back its own output | The third party was the web engine |
| Tier 2 — opening arbitrary web pages, with its navigation / new-window / download guards | A page reaching beyond its own view | The feature is gone, not the guard |
| The tier-2 address bar, which ran in the frame's own view so a page could not type into it | A page filling in its own address, or answering a permission dialog on the user's behalf | There is no tier 2 and no address to bar |
| `kiem-cum-tu-sai` — a wrong recovery phrase typed by script into a real window | End-to-end proof that the window stays open and re-shows the error rather than yielding a wallet | Scripted typing needs a script engine. `phrase_step`'s own tests remain and cover the decision; what is lost is the confirmation that the **window** behaves that way |

**56 tests went with it** (393 → 337). Tests added since bring the count to 394.

**A leftover found two days later (2026-08-25).** `VerifiedApp::copy_content`
handed out a **clone of the entire signed file tree**, and its own doc comment
said why it existed: *"to give to the renderer's file server"*. That server was
deleted with the web engine. Nothing in the repository called it — which is how
it surfaced, in the same sweep for public API no test can reach that found
`FileTree::paths`. It is now removed; `read(path)` remains and is narrower.

The lesson is not the function. Deleting a subsystem leaves accessors shaped
for it, and an accessor with no caller is not inert — it is a wider surface
than anything still in use, kept alive by nobody noticing.

Two honest observations. First, most of these defended against a class of attack
that no longer has a door: there is no parser between an app's bytes and the
screen, so there is nothing to confuse. That is a real reduction in attack
surface, and it is most of why the change was worth making.

Second, one thing genuinely got weaker: **nothing outside our own code now looks
at what we draw.** The old check could ask WebKit "what do you see"; the pixel
renderer is read back by tests we wrote, against an accessibility tree we also
built. A bug shared between the drawing and the reading is invisible to both.
The closest replacement is a screen reader — a real third party — and no screen
reader has run against this renderer yet (§3.1d). Until one has, that gap stands
open and is not covered by a test.

### 3.14 The product's main path did not run, and my smoke test said it did

Found 2026-08-24 because the user asked to see a window.

`tcc-browser examples/hello-tcc` **aborted**. `tao` allows one event loop per
process; asking for a second does not return an error, it aborts, with a message
that names nothing relevant (`app_state.rs:387: The panic info must exist here`).
The path was: `open_package_raster` opens the permission dialog — loop one — and
then `run_app_raster` opens the app screen — loop two.

The web-engine path had shared one loop. Porting to the pixel renderer split it
into two calls, and **the constraint was written down in this repository two days
earlier**, by me, in the doc comment of the very function that now violated it.

**The worse half.** On 2026-08-23 I ran this exact command as a smoke test,
watched the process stay alive for twelve seconds, and reported it as evidence
that the app worked. It was alive because it was sitting on the permission
dialog — loop one — and had not yet reached loop two. I killed it before it could
crash and called that a pass.

That is the third time in two days that a measurement reported success by not
giving the failure a chance to happen: a probe that never let `wrap` wrap, a
`grep` that silently skipped a file it decided was binary, and this. The pattern
is worth naming: **a check that cannot distinguish "it worked" from "it did not
get that far" is not a check.**

**The first fix was wrong, and its failure was quieter than the bug.** Sharing
one `EventLoop` across two `run_return` calls stops the abort — and on macOS the
second call **returns immediately without delivering a single event**. Measured:
loop one ran its full two seconds, loop two exited in 113 ms with no branch of
ours reached. The app screen appeared and vanished, and the caller got `Ok`,
which reads as "the user closed the window". A silent wrong answer in place of a
loud crash.

**The real fix**: the permission dialog and the app screen are two **screens**,
not two sessions, so they go through one `open_sequence` — which is what that
function exists for. `open_and_run_raster` replaces the `open_package_raster` +
`run_app_raster` pair, and both halves of that pair are **deleted**: leaving them
in place invites the next person to call them in the order that broke.

A second entry into the loop is now a plain error with a sentence, and nested
calls likewise — a programming mistake should say what it was.

**And there is now a check that would have caught it.**
`tools/kiem-khoi-ung-dung.sh` runs the product binary through both screens with
no human present, using two hooks: `TCC_TU_DONG_DONG` (close after N seconds) and
`TCC_TU_DONG_BAM` (press a named action — only one that **exists on the current
screen**). Auto-close alone can never get past the first screen, because closing
a window is not an answer; something has to answer.

The script does not ask "is the process alive" — that is the question whose
answer fooled me. It asks three: exit code zero, no panic in the output, and a
line that is only printed **after the second screen is reached**. The third is
the one that matters; the first two still pass if the program stops at screen
one. Mutation-tested by restoring the two-call sequence: all three fail.

⚠️ `TCC_TU_DONG_BAM` can answer a permission dialog. It presses only actions
present on the screen, and the smoke script deliberately presses **deny** — a
smoke test that grants permissions teaches that granting is the default — but
anyone who sets this variable is giving themselves the power to answer for the
user.

### 3.18 A code the standard says cannot exist, fired by this implementation

`spec/0.1/06-error-codes.md` ends with a section titled *"Three codes were
removed for being unreachable"*, and states of them: **"none of them can be
produced by any package"**. One of the three is `bad-key`, removed with this
reasoning: *Ed25519 libraries commonly validate the point lazily, at
verification rather than at parse, so an undecodable key surfaces as
`bad-signature`.*

That reasoning does not hold for the library this implementation uses.
`ed25519-dalek` 3.0 validates eagerly, inside `VerifyingKey::from_bytes`. A
public key of 32 bytes of `0x7f` — correct length, not a point on the curve —
came back as `bad-key`.

So the implementation emitted a code its own standard declares impossible, for
a package a second implementation would reject as `bad-signature`. The removed-
codes section names that exact outcome as the reason unreachable codes are not
harmless: *"two implementations will report different codes for the same
package, which is exactly what stable codes exist to prevent."*

The mapping now follows the standard: an invalid curve point returns
`bad-signature`. The other six `BadKey` sites were length failures on slices
whose length had already been checked — structurally unreachable — and now
return `bad-length`, which the standard does define. The variant is gone.

**How it surfaced, which matters more than the bug.** `cargo-mutants` flagged
`CryptoError::ma` as unkillable. It was unkillable because no vector covers
`bad-key`; no vector covers `bad-key` because the standard says it cannot
happen. The mutation report did not find the bug — it found the *absence of
evidence*, and the bug was at the end of that thread.

**Measured again afterwards.** Over the same two files —
`tcc-spec/src/lib.rs` and `tcc-spec/src/tree.rs` — the first run left **22
surviving mutants and 3 undetermined**. After the boundary tests and after the
conformance vectors became part of `cargo test`: **98 mutants, 96 caught, 2
unviable, 0 surviving, 0 undetermined.**

⚠️ Getting that number took three runs, and the first two were **not results**:

- Run 2 returned 61 `TIMEOUT` and **zero** `MISSED` — which reads, at a glance,
  like a test suite that catches nothing. The cause was at the end of the
  output: `No space left on device`. Each parallel job copies the whole tree
  with its own `target/`, ~1.8 GB apiece, and `-j 4` filled the disk. Every
  mutant "timed out" because the *build* died.
- Run 3, with disk to spare, still returned 39 `TIMEOUT`. `--timeout-multiplier
  10` had multiplied the time to run **`tcc-spec`'s own tests** (~2 s) and
  applied the resulting 20-second cap to runs of the **whole workspace suite**
  (~70 s). The limit measured one thing and was enforced against another.

Neither run had weak tests, and in both, `TIMEOUT` is *undetermined* — not
survived. This document has recorded the same failure repeatedly in one
direction, where "never reached" looks like **passed**; here it appears
inverted, where "never reached" looks like **caught nothing**. Both are the
same defect: a measurement that cannot distinguish its own failure from the
thing it measures.

**The gate that should have caught it did not exist.** Rule 10 checks one
direction: every code in the specification exists in the source. Nothing
checked the reverse. Rule 10b now does, and found three more codes this
implementation emits that the standard never defines: `symlink`,
`package-too-large`, `bad-scroll` — two of them at the package layer, deciding
whether a package loads at all. Amending a published specification is not an
implementation decision (`spec/GOVERNANCE.md` §4 requires a proposal stating who
breaks and which vectors test it), so those three are written up in
`docs/de-nghi-ma-loi-thieu.md` and **named out loud by the gate on every run**.
A silent exemption list would trade "nobody knew" for "nobody looks any more".

⚠️ Removing `bad-key` from the source then made **rule 10 fail** — it had been
scanning the whole error-code document, including the removed-codes table, and
so demanded the source contain the very code the specification says was
withdrawn. The Python half of rule 16 had cut that table off from the start; the
shell half never did. The bug sat still for as long as the implementation
happened to contradict the specification in the matching direction.

### 3.30 A build with no wallet was still asking about money

The maintainer set the order on 2026-08-26: the wallet stays in the product,
but the **browser is built first and the wallet integrated later**. The
wallet-less build therefore becomes the primary one — and the moment it did, a
defect surfaced that had been invisible while everyone built with the wallet on.

That build still showed the wallet row: *"Access your TCC wallet — can ask you
to sign transactions, **this moves money**"*, with a switch the user could turn
on. A person would weigh a decision about money, grant it, and nothing would be
behind it.

Asking a question you cannot act on is a **dialog that lies**. It is the mirror
of the rule this document keeps stating from the other side: the dialog must
show what the runtime enforces. Here the runtime enforced nothing, because there
was nothing there.

Fixed in **two** places, and the second is the one that matters:

- The dialog no longer renders a switch for a capability this build cannot
  grant; it renders a plain line saying so.
- **The grant path refuses it too.** The dialog is not the only route to a
  decision: `.tcc-quyen.json` written by a wallet-enabled build carries a
  remembered *yes*, and the accessibility axis is another entrance. **An answer
  recorded by a different build is not an answer for this one.**

⚠️ The standard has no way for an implementation to say *"I do not provide this
capability"*. `unknown-capability` means the capability is not in the standard,
which is a different statement. Refusing it as an ordinary denial is the honest
behaviour available today — an app must handle denial anyway — but the gap
belongs on the record next to the three codes in
`docs/de-nghi-ma-loi-thieu.md`.

### 3.29 Four boundaries, one blind spot

Mutating the last two crates — `tcc-ui` and `tcc-net` — closed the sweep: all
nine crates have now been measured at least once. Seventeen survivors, four of
which were real.

**One dropped `!` turned every successful call into a failure.**
`v.get("error").filter(|e| !e.is_null())` decides whether a JSON-RPC reply is an
error. Remove the negation and `"error": null` — the *ordinary* success case —
is treated as a failure, while a genuine error is swallowed. Nothing went red,
because nothing reached it: the whole block sat inside a function that only runs
with a live server. It is now a pure function, `doc_phan_hoi`, and the three
decisions a hostile server can reach are tested without one. Same move as
`dich_loi_doc` and `phan_loai`: **pull the decision out of the I/O, then test the
decision.**

**A "third" could have been three times its parent.** `Extent::ti_le` maps the
0.2 layout vocabulary onto a fraction of the parent. `1.0 / 3.0` mutated to
`1.0 * 3.0` and no test noticed. Both the six values and the invariant that
every fraction lands in (0, 1] are now pinned. Alongside it, `kiem_be` — the
check that refuses an extent on an axis where it is meaningless — could be
replaced wholesale by `Ok(())` with the suite still green.

**And a fourth inclusive boundary.** `MAX_UI_BYTES`: `>` to `>=`, unnoticed.
That makes four of the same shape in two days — host length 253, manifest 64
KiB, package content 256 MiB, interface 1 MiB.

Four instances is not carelessness; it is a **systematic blind spot**. Writing a
test, one naturally picks a value that is clearly wrong and one that is clearly
fine. Nobody naturally picks *exactly the number on the edge* — which is the
only value that distinguishes `>` from `>=`, and the only value the
specification actually names.

Two constants were also pinned to their exact values rather than a range:
`8 * 1024 * 1024` mutated to `8 * 1024 + 1024` stays inside "greater than zero
and at most 64 MiB", so the range assertion waved through a 9 KiB ceiling
wearing an 8 MiB label.

⚠️ Still unproven, and named rather than implied: `HttpNetwork::get` and
`JsonRpc::call` can each be replaced by a stub with the suite green. Both need a
TLS server standing in the test, the same gap §3.16 records for the timeout.

### 3.28 The macOS keystore is mostly untested, and now says so

After mutating `tcc-chain` and `tcc-keystore` properly, the survivors clustered
in one file: `tcc-keystore/src/macos.rs`, the **real** Keychain implementation.
`store`, `unlock`, `contains` and `delete` can each be replaced by a stub and
nothing goes red.

That is not laziness. Testing them means writing into the developer's own login
keychain, or standing up a temporary one; `FakeKeystore` exists precisely so the
rest of the system can be tested without that. The honest statement is that the
code which actually holds a user's wallet key on macOS **is not covered by the
test suite**, and this section exists so that fact has a name rather than an
implied "tested".

One function in that file is pure and now tested: `phan_loai`, which turns an
`OSStatus` into an error. Its doc comment already said why it matters — *"no key
yet" and "you just pressed cancel" are two different sentences to a person* —
and deleting the `-128` arm left every test green. A user who **cancelled** the
Keychain prompt would have been told the operating system failed, and would have
gone looking for a fault on their machine that does not exist.

**The comment on that test asserted the opposite, and nobody checked.** It said,
in as many words, *"it touches the real Keychain but does not ask the user — an
item with no access control needs no authentication to read"*. It does ask. The
reason is not access control but the item's **application list**: a rebuilt test
binary is a different program as far as the Keychain is concerned, so it must
request permission. That is the same shape as `bad-key` in §3.18 — a plausible
explanation, never verified, contradicted by reality.

The test is now `#[ignore]`, run deliberately with `-- --ignored`. It is
**`#[ignore]` and not an early `return` behind an environment variable**,
because an early return is a green lie: the summary reads `ok. 10 passed`
exactly as it would if the test had run. `#[ignore]` makes cargo count it —
`9 passed; 1 ignored` — so a reader sees that something did not run.

⚠️ **This gate could hang forever, and did.** The per-flag runner spent over
forty minutes on `cargo test -p tcc-keystore --features os-keystore`: that test
writes a real Keychain item and calls `unlock`, macOS raised its authorisation
prompt, and the gate sat waiting for a click that was never coming. A gate that
can block indefinitely is a gate people learn to skip, and a skipped gate is not
a gate. Each command now runs under a timeout, and a timeout is reported as
**"timed out"** — never as a failing test, because those are different facts.

### 3.27 A redaction nobody tested, in the type that holds the seed

`WalletSecret` carries the 32-byte wallet seed. It has a hand-written `Debug`
whose doc comment states its whole purpose: *do not print the key to a log, even
if someone calls `{:?}` on a large enclosing struct.*

Nothing read what it printed. Mutating the entire body to `Ok(())` — print
nothing at all — left every test green, which means the redaction had never been
observed by anything.

The mutation that matters is not `Ok(())`. It is someone replacing the
hand-written impl with `#[derive(Debug)]`, at which point the seed flows into
every log line that formats a struct containing it. No tool generates that
mutation, and before this test nothing would have caught it.

The test checks **both directions**, and the second is the one people forget: the
output must not contain the seed, **and** must not be empty. A `Debug` that
prints nothing also passes "does not leak" while destroying what an auditor needs
to read. Mutation-checked three ways — empty output, leaked seed, and the same
on `PublicKey`.

**Two survivors were left alive on purpose.** In `mnemonic.rs`, `|` becomes `^`
in the bit-packing of words to entropy. After `bit << 11` the low eleven bits
are zero and every index is under 2¹¹, so the two operators are **mathematically
identical**. No test can distinguish them and none should try. What can be
pinned is the invariant that makes them identical — the wordlist is exactly 2048
entries — and that had no assertion either. It has one now: change
`BITS_PER_WORD` and the dictionary test fails.

⚠️ **The first measurement of this crate was not a measurement.** Run with the
oracle at `cargo test --workspace`, it reported **45 survivors** in the wallet.
`import.rs` lives entirely behind `import-web-wallet`, a feature that command
does not enable — so the mutated code was never compiled and its tests never
ran, and every mutant in it was recorded as surviving. Re-run with the feature
on, the real figure is **25**.

That is the third time in two days that *"never reached"* has been reported as
*"your tests are weak"* — after the conformance vectors outside the oracle
(§3.17) and a full disk turning every mutant into a timeout. Here the cost of
believing it would have been a day spent patching twenty untouched holes in the
one crate that must not be broken.

### 3.26 The rule I wrote yesterday had yesterday's bug in it

Rule 10b — added 2026-08-25 to ask the reverse of rule 10, *does the
implementation emit any code the specification does not define* — read the
whole of `06-error-codes.md`, including the table titled **"Three codes were
removed for being unreachable"**. So a code the specification explicitly
**withdrew** still counted as defined, and the rule waved it through.

That is the identical defect fixed in rule 10 the previous day, in the same
file. Rule 16's Python half cut that table off from the start; rule 10's shell
half never did (fixed 25/08); rule 10b never did either — and rule 10b was
written by me, hours after fixing rule 10.

With the cut applied it found two more: `not-a-container` and
`publisher-not-hex`. Both **are** constructed in the source, exactly like
`bad-key` was.

The difference is that this time the specification is right, and it now has
evidence rather than an argument:

- `publisher-not-hex` — shape checking rejects a non-hex publisher as `not-hex`
  first. Verified, and mutation-checked: **remove `validate_shape()` and a
  package does emit `publisher-not-hex`.**
- `not-a-container` — a leaf has no `children` field, so a decoder rejects the
  JSON as `bad-json` before any tree rule runs. Verified against a real
  payload.

Both remain in the source because the Rust builder API can reach them; a
**package** cannot. That distinction is the entire content of the
specification's claim, so it is now a test. Rule 10b's exemption for them is not
an amnesty: the gate checks the proving tests still exist, and deleting one
turns it red.

**And the content cap had never been tested at all.** Mutating `tcc-runtime`
left five survivors, all in one place: `MAX_CONTENT_BYTES`, the 256 MiB ceiling
on package content read *before* authentication. `>` to `>=`, `+=` to `*=`, and
both `*` in `256 * 1024 * 1024` — every one survived.

A ceiling cannot be tested by building 256 MiB of files, so it never was. The
limit is now a parameter, which a test can set to twenty bytes, and the test
checks the two things that matter: it holds at the exact edge, and it
**accumulates across subdirectories** — a per-file check would let any number of
files through. The constant's own arithmetic is a build-time `const` assertion,
so changing it fails the build rather than a test.

### 3.25 Pressing a button erased what you had typed

Every screen transition went through one path that reset all frame-held state:
toggles, field contents, focus. For a **new** screen that reset is a security
property, and this document has said so: a toggle left over from the previous
screen is the previous screen answering on the next one's behalf, and a PIN
typed under one label reappearing under the same label elsewhere is a leak.

But the in-app result — press a button, get a line of output — was built as a
*new* screen, because it is the old tree plus one line. So typing into the
field and then pressing any button silently discarded the text.

There are two different operations here and they had one name. They now have
two: `Next::Show` for a new screen, which clears; `Next::Update` for the same
screen with a changed tree, which keeps. `Show` remains the default, and the
distinction is stated at the call site rather than inherited quietly.

The test asserts at source level that the clearing sits behind the condition
and that each variant keeps its meaning, because merging them again fails in
one of two ways — either a button press wipes the user's input, or the previous
screen's toggles answer for the next one. The second is a hole, not an
annoyance.

### 3.24 Text could only be edited at the end

Typing and `Backspace` acted on the end of the string. There was no caret to
move, so a mistake in the middle could only be reached by deleting everything
after it.

The screen where that matters is wallet recovery, which asks for **twenty-four
words**. A typo in the third word meant retyping twenty-one. A person facing
that will paste instead — and pasting a recovery phrase is exactly the habit
this project tells people not to form.

Editing now happens at a caret, with arrows, `Home`, `End` and `Delete`. Three
things are worth stating because they are where this kind of code goes wrong:

- **Cuts are on characters, not bytes.** `ế` is one character and three bytes.
  Splitting on bytes leaves a string that is not valid UTF-8; deleting "a mark"
  to leave `ê` or `e` changes what the user typed. The insert and delete
  functions are pure and tested directly on Vietnamese text.
- **Entering a field puts the caret at the end** of what is already there.
  Starting at the beginning means the next character typed jumps in front of
  everything — with a recovery phrase, that ruins the whole entry.
- **The caret is drawn where it actually is.** A caret in the wrong place is
  worse than none: it points at where the next character will *not* go.

⚠️ The test for this had to be written **three times**, and both failures are
recorded in its doc comment. The first compared ink between a focused field and
a focused button — meaningless, since the two boxes differ in size, so their
focus rings differ in ink; deleting the caret code entirely left it green. The
second looked at a fixed column, and broke the moment the caret could move. The
third looks for the shape: a dark column spanning the box height.

### 3.23 The shipped binary had no accessibility bridge at all

The AccessKit adapter was wired up on 2026-08-19 and this document has been
citing it since. What nobody checked is whether the **product** could switch it
on: it lived behind `tcc-shell/window-tro-nang`, and `tcc-browser` — the crate
that becomes the binary a person runs — exposed no path to that flag. Every
build anyone has ever run had a window and no accessibility tree. A screen
reader pointed at it would have found a rectangle of pixels.

It was not a decision. The flag exists so `cargo test` does not drag in three
platform adapters, and the product simply never got reconnected to it. The
window feature of the application now includes it, and rule 24 fails the build
if anyone separates them again. `cargo test --workspace` does not build that
feature, so the test suite is no heavier.

The lesson repeats the one in §3.17 almost exactly. There, the conformance
vectors were real but invisible to the oracle. Here, the accessibility bridge
was real but absent from the artefact. In both cases the work existed, the
document cited it, and the thing that actually ships did not have it.

⚠️ This makes the bridge **present**. It does not make it **correct** — still
nobody has listened to a screen reader read this window.

### 3.21 The window could not be operated by keyboard at all

Until 2026-08-25 the frame handled exactly two keys: characters, and
`Backspace`. There was no `Tab`, no `Enter`, no focus indicator. Every button
and every field was reachable **only by mouse**.

That is an accessibility failure, but it is also a security one, and the second
is the reason it belongs in this document. §3.12 recorded the mirror of it — a
blind user could not type a PIN — and the fix there went through the
accessibility axis. This one is simpler: a person who cannot use a mouse could
not answer a permission dialog, and a dialog that cannot be answered is a
dialog that gets answered by someone else.

Two rules in the new keyboard path exist for the same reason the mouse rules do:

- **`Enter` inside a text field activates nothing.** On a permission dialog the
  nearest button may be *Allow*; a stray Enter there is an answer the user never
  gave.
- **`Enter` on a toggle flips the toggle and keeps the screen open**, exactly as
  a click does. A keyboard path that closed the dialog on the first toggle would
  answer, on the user's behalf, every item they had not yet read.

The focus ring is drawn **outside** the box with a gap, because destructive
buttons already use a **double frame inside** (B31). Drawing focus inside too
would collapse two distinct signals — *this one is dangerous* and *this one is
selected* — into one shape. This renderer has a single ink channel, so a signal
has to be a shape, never a colour.

Clicking now also sets focus. Before, a click followed by `Tab` jumped back to
the start — mouse and keyboard were looking at different interfaces.

Hovering highlights the target under the cursor — buttons previously gave no
feedback at all. It is a **light fill**, deliberately not another frame: frames
already carry two meanings on this renderer (a frame means *button or field*, a
double frame means *destructive*, B31), and a third meaning on the same shape is
where a user starts guessing. Hover is also kept **separate from keyboard
focus**; merging them would mean moving the mouse silently moves where `Enter`
would land.

⚠️ Still not proven with a real assistive technology. The tests check the frame's
own model of focus; nobody has yet listened to a screen reader read this window.

### 3.22 A window that could not be resized

Layout width was a compile-time constant. Widening the window left the text
wrapping at the old column with white space beside it, because nothing
recomputed. It is now a property of the renderer, clamped to a usable range, and
a resize relays out.

The clamp is not cosmetic: dragging a window nearly shut is an ordinary thing to
do, and it must not become an image of width zero or a division by it.

### 3.20 B4 was proved on one path out of four

`revoke_all` is the emergency stop: B4 says it kills **every copy, including
one already in an app's hands**. Its test walked the **network** capability and
nothing else. On 2026-08-25 the other three entry points turned out to be
unproven, and the two ways they surfaced are worth separating.

`cargo-mutants` replaced the whole body of `WalletCapability::allow_read_address`
with `Ok(())` and no test went red. That function does exactly one thing — ask
whether the capability is still alive — so "the entire body can be replaced and
the suite stays green" means that one thing had never been checked. A revoked
wallet could still read the address, under a `Bn` row that claimed evidence.

`StorageCapability::allow_write` the tool did **not** find. Its body also
enforces a quota, so `-> Ok(())` breaks the quota test and gets caught; the
mutant that mattered was the deletion of the single line `self.life.touch()?;`,
which the tool does not generate for that shape. Removing it by hand left every
test green. **A mutation tool's catalogue is finite, and its silence is not
evidence** — the same sentence this document keeps writing about every other
measurement.

Both are now covered by one test that walks **all four** entry points after
revocation, rather than a representative one. A capability added later without
a revocation path fails there. Each of the four guards was deleted in turn to
confirm the test sees it.

### 3.19 The vectors caught a malleable signature — mine, hours old

Mutation testing `tcc-crypto` on 2026-08-25 left seven survivors, all of them
the same shape: `+` becoming `*` where the *expected length* is computed for an
error message. Reading `take` confirmed the arithmetic feeds only the
`expected:` field — the slicing decision uses `at..at + len` — so behaviour was
unaffected and every wrong-length key was still rejected.

Pinning them was still right, and the reason is worth separating from the rule
that error text may be reworded freely: this is not wording, it is a **number**.
Reporting *"expected 3968 bytes"* when the true figure is 1984 is a false
statement, and whoever reads it goes off to build a key of a length that does
not exist.

Writing that test exposed a worse one. For the **signature**, the expected
length was derived from the input under suspicion —
`pq_sig_len = signature.len().saturating_sub(64)` — so a 10-byte signature was
reported as *"expected 64"* when the real answer is 3373. Replacing it with the
algorithm's own constant made the message truthful and **introduced signature
malleability**: the old derivation had been quietly doing a second job. With
`pq_sig_len` fixed, `take` sliced its 3309 bytes and a **trailing extra byte
was ignored** — the same message now had unlimited valid signatures, and
anything comparing signatures byte-wise (a ledger, a cache, a replay guard) is
defeated.

The conformance vector `them mot byte thua` failed the moment the change
compiled. It failed under **`cargo test --workspace`** — which it could not have
done before that morning, when the vectors ran only under `cargo run`. The
argument for §3.17 was theoretical when it was written; it stopped being
theoretical the same day.

The fix checks the total length with `!=` before slicing, and both directions
are now pinned at the unit level too, so the next person does not need the full
conformance run to learn it. Mutation-checked: `!=` weakened to `<`, and the
check deleted outright — red both times.

### 3.17 The conformance suite was not a test

Running `cargo-mutants` over `tcc-spec` and `tcc-crypto` for the first time on
2026-08-25 returned **34 surviving mutants out of 132**. Reading them split
into two very different findings.

**Six were real gaps, all on the standard's own boundaries.** `>` became `>=`
in the host length check (253), the label check (63), the app-id check (128),
and the combining-mark limit — and nothing went red. `spec/0.1` states those
ranges as *inclusive*: "1–253 characters, each label 1–63". Every existing test
used a comfortably short valid name or a badly *shaped* invalid one; none stood
on the edge. A second implementation reading the specification would accept a
253-character host and this one would have rejected it, disagreeing exactly
where the standard is most explicit. Boundary tests added, each verified by
re-running the mutant that survived.

**The rest exposed a flaw in the method, not the code.** `cargo-mutants` uses
`cargo test` as its oracle, and `tcc-conformance` was a `main.rs` with **zero**
`#[test]` functions. The 154 vectors ran only when someone typed `cargo run`.
They *were* run — by CI, and by the pre-push checklist — but they were invisible
to the oracle, so every mutation that only a vector catches was reported as
surviving. Mutating `SpecError::ma` to return `"xyzzy"` — the machine-readable
error codes the whole standard is compared by — showed **green** under
`cargo test --workspace`.

The suite is now a library plus a thin CLI, with one test per group in
`tests/tuan-thu.rs`. `cargo test --workspace` runs the vectors; the same
mutation is now red. One test per group rather than one for all nine, because a
single combined assertion loses the one fact worth having when it fails: which
group.

⚠️ This is the same failure this document keeps recording, in a new place: a
check that cannot tell *"nobody verified this"* from *"it was verified where I
was not looking"*. The vectors were never weak. The evidence about them was.

### 3.16 B24 claimed three things and proved one

The B24 row read: *HTTPS only, with a timeout and a size ceiling* — three
separate enforceable behaviours. Its entire evidence column was the crate name,
`tcc-net`. Auditing it on 2026-08-25 found HTTPS-only well covered (four pure
tests, no server needed) and the other two covered by **nothing**.

What the audit turned up, in the order it turned up:

**The ceiling really is enforced, and it is off by one.** `ureq`'s
`LimitReader` returns an *error* rather than truncating silently — the
distinction matters, because a silently truncated body is a partial file
treated as complete. But it errors on the read **after** the quota is
exhausted, so a body of exactly `MAX_BYTES` is **rejected**: the ceiling is
strictly-less-than, not less-than-or-equal. That is the safe direction, so the
behaviour stands and the doc comment now says what the code does. Both facts
were read out of `ureq-3.4.0/src/body/limit.rs:22-24`, not recalled — the
distinction §3.5b had to introduce the hard way.

**Both read sites mapped errors wrongly, in opposite directions.**
`lib.rs` wrote `map_err(|_| TooLarge)`: a mid-transfer disconnect, a reset, a
timeout — every read failure was reported to the user as *the file is too
large*, and someone reading that goes looking for a smaller file forever, for a
fault that has nothing to do with size. `rpc.rs` wrote `map_err(Goi)`: the
genuine ceiling hit vanished into a generic *call failed*. The mapping is now
one pure function used by both, and `TooLarge` carries the **channel's own**
limit (8 MiB for packages, 1 MiB for RPC) instead of a hard-coded constant that
lied about one of the two.

`ureq::Error::BodyExceedsLimit` survives the `into_io()` → `From<io::Error>`
round trip (`error.rs:198` and `:216`), which is what makes the distinction
possible at all; that too was read, not assumed.

**The sanity of both constants is now a build-time assertion, not a test.**
Written first as a `#[test]`, clippy pointed out what it really was — an
assertion over constants — and the honest form is `const { assert!(…) }` at
module scope. Setting the ceiling to zero or dropping the timeout now **fails
the build**, which is louder than a red test and impossible to run past.
Mutation-checked: `MAX_WAIT = Duration::ZERO` stops compilation with
`E0080: evaluation panicked`.

⚠️ **What is still not proven.** That the constants are sane is not that
`ureq` honours them. Whether the timeout fires, and whether the ceiling holds
against a real hostile server, needs a TLS server standing in the test — the
HTTPS-only rule means no plain-HTTP fixture can reach this code. The tests say
what they cover and no more.

⚠️ 15 of the 45 `Bn` rows still cite no test. Ten of those are type-level or
structural claims where that is the right answer (`Tone` has no colour field; a
package ships no code) and five are retired. B24 was the row where it was the
wrong answer. The rest have not been audited one by one.

### 3.15 Nine tests asked "is there a warning *somewhere*" (B45, new)

Auditing the ~20 screen tests rewritten during the WebView removal, one of them
— `cau_chuyen_tien_duoc_ve_khac_di`, which exists to prove the sentence *"this
moves money"* stands out from everything around it — turned out to pass when
the warning mark was moved to a **different line entirely**. It asked
`s.contains("[cảnh-báo]")`: *is there a warning mark anywhere in this tree?*
Nine tests asked that same question.

The distance between the two questions is the whole point of the invariant. A
screen full of marks marks nothing; the claim being tested was never "this
screen is alarming", it was "**this sentence** is the alarming one".

`do_cay::co_canh_bao(cay, cau)` asks the narrow question — the line carrying
`cau` also carries the mark — and all nine sites now use it. Mutation-testing
the converted sites moved the mark to a neighbouring line **without removing
it**: red at every site, green under the old wording.

Converting them exposed a real defect behind the weak one. On the recovery
**failure** screen the mark sat on the diagnostic detail (`kho khoá từ chối`),
while the sentence the user has to act on — *nothing was stored, the key is
gone* — was plain body text among four other lines. Its sibling screen marks
the identical class of sentence (`PhienKhongCatDau`, `recovery_screen.rs:106`).
The test named `man_hong_noi_ro_khong_luu_gi` — *"says clearly nothing was
stored"* — had been green for that screen the whole time, because a mark on the
diagnostic answered the question it was actually asking. Both sentences now
carry the mark, and the test names each of them separately.

The general lesson is older than this bug and keeps costing the same way: a
test that cannot distinguish *the thing I care about* from *anything in the
same neighbourhood* is not evidence about the thing I care about.

### 3.13 B16 held for disk and leaked at the keyboard (B43, new)

B16 says decoding from disk **cannot bypass the constructors**. It is true, and
it was answering the wrong question by 2026-08-23.

On the pixel renderer, a text field's content is not in the tree. The frame holds
it (`TrangThai::noi_dung_o`), a keystroke appends to it, and it re-enters the tree
through `with_fields` — which built `NodeKind::Field { value }` **directly**. So
every check `Node::field` performs, including the display-string rules that
B6 rests on, was skipped for anything typed.

The character that matters is a **bidi override**. It makes the text *displayed*
differ from the text *entered* — and this project rejects it everywhere else
precisely because a screen that shows something other than what was signed is the
whole threat model. A field on a transaction screen is the worst place for it.

**Fixed in two layers, deliberately:**

1. `with_fields` now validates, so no path can put unchecked text into a drawn
   tree. On failure it **keeps the previous value** rather than clearing — losing
   what someone typed because of one bad character is its own defect.
2. `Phien::nhan_chu` rejects the keystroke at entry. Without this, the failure
   surfaces at draw time inside `ve_lai_man_hinh`, which has nobody to report to
   and therefore swallows it: the user types, the screen does not change, and
   they type again.

The second layer is the interesting one. Layer 1 alone is *correct* and
*unusable* — the classic shape of a check placed where its failure cannot be
communicated. Mutation-tested separately: removing layer 1 lets the character
into the tree; removing layer 2 lets it into the field.

### 3.12 The mirror of B41: a blind user could not type a PIN

Applying §3.11's rule in the other direction — *what can the mouse do that the
accessibility axis cannot?* — found the sharpest gap of the day.

**Text fields carry no action id.** The 0.1 standard defines no action for a
field, so `AccessNode.action` is `None`, so the field never entered the table the
platform's activation requests are looked up in. The mouse focuses a field
through `hit_test_field`; the accessibility axis had no path at all.

A VoiceOver user could hear *"PIN, secure text field"*, press it, and nothing
would happen — and then typing went nowhere, because `nhan_chu` drops input when
no field is focused. The two screens this lands on are **the PIN prompt and the
24-word recovery phrase**. On this product that means a blind user could not
import or restore a wallet.

**The fix, and the trap inside it.** Fields now enter the table under a separate
target kind, and the queue accepts `Action::Focus` as well as `Action::Click`.
Accepting `Focus` naively would have been much worse than the bug: screen readers
send `Focus` every time the user *moves* to a node, so a button would fire as the
user swept past it — **moving the cursor onto "Allow" would grant the
permission**. `Focus` is therefore accepted only for fields; on a button it is
dropped. Both halves are mutation-tested.

⚠️ **Unverified against a real screen reader.** Which action macOS, Windows and
Orca actually deliver for a text field is taken from AccessKit's role mapping,
not from observation — no screen reader has ever run against this renderer
(§3.1d). The reasoning is written down so the first person with VoiceOver can
check it in a minute rather than re-derive it. If macOS sends something other
than `Focus`, this fix is inert and the gap is still open.

### 3.11 A gap that parity checking cannot see (B41, new)

`check_accessibility_parity` compares the **source** tree with the **published**
tree. Both are trees. Neither is the thing that was actually drawn.

So a node present in both, but absent from **layout**, passes. And that is not a
harmless state: the action table the platform uses is built from the published
tree (`bang_hanh_dong_cua`), so such a node stays activatable over the
accessibility axis while being invisible on screen and unreachable by mouse.

The layout builder made this possible by ordering: it pushed the accessibility
node **before** creating the layout node, so a failure of the second left the
first behind. Now the accessibility node is written to a scratch list and merged
only after the layout node exists — for leaves and for groups alike.

This is the F1 shape reflected. F1 (2026-08-21) was a button drawn outside the
image that `hit_test` still returned: **the mouse could reach what the eye could
not**. This is the mirror: **the screen reader could reach what the mouse could
not**. Neither direction is acceptable, and the general rule is worth stating
plainly because it will come up again:

> Any asymmetry between input paths is a way around whatever the stronger path
> enforces. Three defects this week were exactly that — F3 (the accessibility
> queue outliving a screen), B19 (the guard on one path only), and this one.

B41 now watches it directly: the set of actions in the published tree must equal
the set of actions on drawn boxes. Mutation-tested by dropping one leaf from
layout while still announcing it.

### 3.10 B19 was enforced on one input path and not the other

Same audit, same day, one layer down. `ap_ket_qua` is the single place where a
click becomes an answer, and it is reached from **two** directions: the mouse,
and the platform accessibility axis.

The F3 fix of 2026-08-21 put the "already answered, ignore this" guard at the
**accessibility** call site. The mouse call site had none — and the comment
sitting on the accessibility guard read *"every rule the dialog applies to the
mouse applies here too"*, which states the relationship backwards. The mouse path
was the looser one.

`tao` delivers queued events after `ControlFlow::Exit` is set, so a second press
already in the queue reaches `ap_ket_qua` and overwrote the answer: press
"Refuse", and a queued press of "Allow" replaces it.

The guard now sits **inside `ap_ket_qua`**, where the two paths meet, rather than
being copied to the second site. Copying is how a third path arrives later
without one. It also covers `VeLai`: once a screen has ended, a queued toggle
cannot repaint it either.

Tested by calling `ap_ket_qua` directly — no event loop, no screen reader — so
the test exercises the junction rather than one of its two approaches.

### 3.9 F3 came back, through a door this project opened yesterday

Found 2026-08-23 by asking of each invariant "does the code still *do* this?"
rather than "does a test with that name exist?".

**The bug.** Three facts, each harmless alone:

1. Accessibility node ids are assigned by a counter starting at **0 on every
   tree build** (`to_accesskit_with_actions`). Screen 2's node 5 is not screen
   1's node 5.
2. The queue of pending `AXPress` requests has a push site and a drain site and
   **no clear site**.
3. The F3 guard — `da_bam.is_none()`, which stops a queued request from
   overwriting an answer the user just gave — goes false→true again **at exactly
   the moment the screen changes**, because `ket_man()` has just taken `da_bam`.

Together: a request queued while screen 1 was up, not drained before the swap,
is drained after it and looked up in the **new** table. It runs an action nobody
pressed, on a screen the user has not read yet. In the wallet import flow, screen
2 is the PIN prompt.

**Where it came from.** `open_sequence` — written the day before, to let several
screens share one event loop, because `tao` allows only one per process. Before
that, every screen was its own loop and its own queue, and the bug had no way to
exist. A fix for one problem opened the door for another, and the guard that
would have caught it had been written for the single-screen world.

**The fix** is one `clear()`, placed **before** the action table is rebuilt.
Ordering is the whole fix: between rebuilding the table and clearing the queue,
a stale id is still a valid lookup. Both halves are mutation-tested — remove the
clear, and the test fails; move it after the rebuild, and it fails differently.

Not fixed by making ids globally unique, though that would also work: it would
make a stale request match nothing, which is the safe outcome. It is not done
because AccessKit's adapters diff trees by id, and changing id allocation to
chase this bug would trade a known failure for an unmeasured one.

### 3.8 B31 was BROKEN for a day, and how that happened

On 2026-08-23, self-review found that the pixel renderer read `Tone::Danger`
**only** to set an accessibility flag. A destructive button was drawn with the
same frame, the same text and the same everything as a neutral one. A screen
reader announced "destructive"; a person looking at the screen was told nothing.

B31 exists *because this already happened once*: the first renderer drew
`Tone::Danger` identically, an outside review caught it, and the invariant plus a
test were added. When that renderer was deleted, **the test went with it** — and
the invariant was left in this table pointing at a function that no longer
existed. The property regressed the same day the evidence disappeared, and
nothing connected the two events.

Three things worth taking from it:

1. **Deleting a renderer deletes evidence, not just code.** §3.7 listed the
   defences that went. It did not list the *invariants* that lost their proof —
   thirteen test names in this section pointed at nothing, and B31 was the one
   where the property had also silently become false.
2. **The accessibility tree is not a witness for what a sighted user sees.** It
   was correct throughout — `destructive: true`, the whole time. A test that had
   asked the tree would have stayed green. The replacement test measures **ink**.
3. **The fix had to be shape, not colour.** This renderer has one greyscale
   channel, and the button's text belongs to a signed package and may not be
   edited. A destructive button is now drawn with a **double frame** — visible on
   a monochrome screen and to a colour-blind reader, neither of which a red fill
   would have been.

## 3bis. The conformance suite

`conformance/vectors/*.json` — **data, not code**, so implementations in other
languages read exactly those files. Matching is on **stable error codes**
(`unsafe-display-string`, `bad-app-id`…), never on messages: messages are prose
and may be reworded, codes may not.

| Group | Vectors | Checks |
|---|---|---|
| `canonical` | 7 | Canonical form + hash — **interop** |
| `signature` | 15 | Hybrid signature — **interop**, three directions |
| `acvp-mldsa65` | 26 | **An external NIST anchor** for the post-quantum half |
| `manifest` | 31 | Accepting/rejecting manifests, button behaviour, hostname shape, unknown fields |
| `ui` | 17 | Accepting/rejecting interface trees |
| `capability` | 8 | Network scope matching |

The `canonical` group is generated by an **INDEPENDENT Python implementation**,
not taken from the Rust code — otherwise the vectors would only say "we agree with
ourselves". The empty tree yields `af1349b9f5f9a1a6…`, matching the public BLAKE3
KAT for the empty string, which anchors the Python side to something external.
Rust and Python agree **byte for byte** on all 7 cases.

### The `signature` group checks THREE directions, not one

| Direction | Why it is needed |
|---|---|
| **Key generation** | The same secret key must derive the same public key |
| **Signing** | Re-signing must produce the EXACT same bytes (signing here is deterministic) |
| **Verification** | Valid signatures must pass, six attacks must fail |

Checking only the third is not enough: an implementation that verifies our
signatures but produces signatures we cannot verify **still cannot share packages
with us**.

The six attacks: flip a bit in the Ed25519 half · flip a bit in the ML-DSA half ·
flip the LAST bit · truncate · append a byte · **swap the order of the two halves**
(the byte layout is part of the standard, not an implementation detail).

**Both halves now have an external anchor (2026-08-13).**

| Half | External anchor | Cases |
|---|---|---|
| Ed25519 | RFC 8032 §7.1 TEST 1 | 1 |
| ML-DSA-65 keyGen | **NIST ACVP** (`ML-DSA-keyGen-FIPS204`) | **25 / 25 match** |
| ML-DSA-65 sigVer | **NIST ACVP** (`ML-DSA-sigVer-FIPS204`) | 1 |

**⚠️ A FINDING MORE IMPORTANT THAN THE VECTORS: the FIPS 204 interface.**

FIPS 204 defines **two** signing interfaces. The *external* one computes
`M' = 0x00 ‖ len(ctx) ‖ ctx ‖ M` and signs that; the *internal* one signs `M`
directly.

Running NIST's sigVer vectors through this implementation: the `external` group
matched 1/1, while the `internal` group **disagreed on 3 of 15** — and only on
cases NIST marks as passing. That is exactly the signature of one side using the
external interface.

The conclusion is now **a sentence of the standard** rather than an assumption:
**TCC uses the EXTERNAL interface with an EMPTY context.**

A TCC implementation choosing the wrong interface produces signatures the other
side cannot verify — while **both sides are "FIPS 204 compliant"**. This is the
quietest interoperability trap in the standard, and until that day it was written
down nowhere.

**A whole package built by the other implementation (2026-08-15).**

The vectors check a parser: give it data, see whether it accepts or rejects.
They cannot check the other direction — that a different implementation, in a
different language, can **construct** something this one accepts. An
implementation that rejects everything wrong while producing nothing right still
cannot exchange packages with anybody.

The package it builds is a **full** manifest, not a minimal one: two
capabilities and an action. That is deliberate — capabilities and actions carry
the most rules in the format, and they are where a disagreement between two
implementations costs security rather than merely failing to open. The last two
mutations below are that part: a package whose button points somewhere the
manifest never asked for is refused, and so is one pointing at a subdomain of a
granted host, because matching is exact.

`conformance/dung-goi-doc-lap.py` runs **both directions**. It builds a complete
package from nothing — canonical form, content hash, manifest, hybrid signature,
directory layout — and `tcc verify` accepts it. Then it reads
`examples/hello-tcc`, signed by the Rust, and verifies it in Python: both
signature halves, the content hash, the entry point, in the order
`01-package.md` demands.

The reverse direction matters as much as the forward one and nothing checked it
before. Nothing proved that what `tcc sign` emits is **readable by anybody
else** — and an implementation that produces packages only it can read passes
every test it has.

Ed25519 is written out in that file from RFC 8032 rather than imported, so the
classical half really is two implementations and not two calls into one library.
It was wrong on the first attempt — the point-addition formula returned its
coordinates in the wrong order — and RFC 8032 TEST 1 caught it before anything
else ran.

Mutation-tested in three directions, each corresponding to a decision the
standard makes explicit:

| Mutation | Result |
|---|---|
| Length prefix written after the content instead of before | content-hash mismatch |
| FIPS 204 `ctx` set to `"TCC"` instead of empty | ML-DSA half rejected |
| The two signature halves swapped | Ed25519 half rejected |
| One byte changed in the example's content | Python reports a content-hash mismatch |
| One bit flipped in the example's signature | Python rejects the Ed25519 half |
| An action pointing at a host outside the requested capability | Rust rejects the manifest |
| An action pointing at a **subdomain** of a granted host | Rust rejects it — matching is exact |

The middle one is the quiet interoperability trap of the whole standard, and
this is the first check that would catch a second implementation getting it
wrong.

**Its limits.** I wrote both sides, so it is not the independent implementation
`spec/GOVERNANCE.md` §3 asks for. It catches a disagreement between two readings
of the specification; it cannot catch a place where I misread the specification
the same way twice. `blake3` also comes from a library on both sides, which is
why the `canonical` group anchors it to the published empty-input KAT.

**The SIGNING direction is anchored by CROSS-CHECKING, not by vectors (2026-08-14).**

ACVP's `sigGen` group is unusable here: it supplies secret keys in EXPANDED
4032-byte form, while the `ml-dsa` library loads only a 32-byte SEED. There is no
bridge between them.

So a different route: `dilithium-py` 1.4.0 — a **pure-Python implementation,
written by someone else, sharing not one line of code** with the Rust. Sign the
same messages with the same seed and compare byte for byte.

**A mandatory step first:** the Python side must itself match the NIST vectors
(25/25 keyGen). Without that it is merely a *second opinion* — two implementations
wrong in the same way still agree with each other, and we would believe them.

Result: **byte-for-byte agreement** on all three messages. Stronger than a handful
of isolated vectors, because it agrees on exactly the usage this project has —
external interface, empty context, deterministic.

```sh
python3 conformance/doi-chieu-doc-lap.py <ACVP-vector-directory>
```

**What this script taught back about hybrid signatures.** My first version handed
all six attacks to the Python side to verify, and it **accepted** the "flip one bit
in the Ed25519 half" attack — correctly, because that attack never touches the
ML-DSA half. The hybrid signature still fails, but it fails on the other half. The
script was wrong, not the code. It is also a live demonstration of B1: **breaking
one half does not propagate to the other, and that is precisely why the signature
is hybrid.**

```sh
cargo run -p tcc-conformance                 # 154 conformance vectors
cargo run -p tcc-conformance -- --chi-tiet
```

---

## 4. Reproducing everything

```bash
cargo test --workspace                              # 394 tests
cargo test --workspace --features tcc-shell/window  # 380 — three more that need a window
cargo run -p tcc-conformance                        # 154 conformance vectors
python3 conformance/doi-chieu-doc-lap.py <vectors>  # dilithium-py cross-check
cargo clippy --workspace --all-targets -- -D warnings
tools/kiem-luat-phu-thuoc.sh                        # 24 architecture rules
```

All of them must be clean. `kiem-luat-phu-thuoc.sh` runs **before** compilation in
CI: code that runs but has the wrong architecture is still wrong.

None of the above touches WebKit. The parts that go through the real renderer must
be run separately, on a machine with a screen:

```bash
cargo run -p tcc-shell --features window --example kiem-khoi-tan-cong          # full pipeline
cargo run -p tcc-shell --features window --example kiem-khoi-tan-cong chi-csp  # CSP alone
cargo run -p tcc-shell --features window --example kiem-bam-nut cho-phep       # click → Allow
cargo run -p tcc-shell --features window --example kiem-bam-nut tu-choi        # click → Deny
cargo run -p tcc-shell --features window --example kiem-bam-nut bat            # toggle on → Allow
cargo run -p tcc-shell --features window --example kiem-bam-nut ma             # phantom action discarded
cargo run -p tcc-shell --features window --example kiem-bam-nut ct-ma          # phantom toggle discarded
cargo run -p tcc-shell --features window --example kiem-man-hinh-ung-dung <pkg>  # app screen
cargo run -p tcc-shell --example kiem-hanh-vi <pkg>                              # three-way capability gate
cargo run -p tcc-shell --example kiem-ghi-nho <pkg>                              # permission store on real disk
```

One check needs **a person at the keyboard** and therefore cannot be in CI —
the input method belongs to the operating system, and every way of simulating it
injects a finished string, skipping the composition session that is the thing
being measured:

```bash
cargo run -p tcc-shell --features window --example kiem-go-tieng-viet -- <pkg>
```

It asks WebKit what the field actually holds and prints the codepoints, the
number of standalone combining marks, the deepest mark stack on one character,
and the caret position. Looking at the screen is not enough: a precomposed `ỡ`
and `o` plus two marks render identically while consuming completely different
amounts of the `MAX_COMBINING_MARKS` budget.

These load hostile manifests into real WebKit and then ask WebKit what it sees.
They are **not** part of `cargo test` because on macOS the event loop must run on
the main thread while Rust's test harness runs on worker threads. Do not skip them
just because `cargo test` is green.

---

## 5. Reporting a security issue

Please do not open a public issue. Use GitHub's private vulnerability reporting on
this repository, or contact the TCC IT department directly.

Bear in mind what §3 says about the current state: this is pre-audit software
implementing an unfrozen draft standard, written by a single party. A report that
the design itself is wrong is more valuable here than a report that the code
disagrees with the design.
