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
| B7 | App-supplied text **cannot escape** the renderer's document | `chu_cua_ung_dung_khong_thoat_ra_duoc_tai_lieu` + the `kiem-khoi-tan-cong` example |
| B8 | The accessibility tree the renderer publishes **matches** the source tree | `check_accessibility_parity` + `nhan_khac_chu_hien_ra_thi_bao_loi` |
| B9 | The interface **cannot express** a button missing a role or a label | Types: `Alt::Decorative` must be stated out loud |
| B10 | Apps **cannot set colours** — only declare intent | `Tone` is a closed enum with no colour field |
| B11 | Skipping signature verification **does not compile** | `grant_verified` accepts only `VerifiedApp` |
| B12 | Every unclear path ends in **DENY** | `moi_duong_khong_ro_rang_deu_ra_tu_choi` |
| B13 | **The renderer wires events; the app never does** | `script-src 'none'` + the renderer's init script |
| B14 | Phantom actions **go nowhere** | Allowlist + `kiem-bam-nut ma` |
| B15 | Apps **ship no markup**, only a declarative tree | `entry` is `ui.json`; `tcc-ui` exposes no web concepts |
| B16 | Decoding from disk **cannot bypass the constructors** | `UiNode` is a separate type; `TryFrom` rebuilds through the checking constructors |
| B17 | Permission is asked **item by item**, every toggle default OFF | `moi_cong_tac_mac_dinh_tat`, `bam_cho_phep_ma_khong_bat_gi_thi_khong_cap_quyen_nao` |
| B18 | A phantom toggle **discards the whole message**, it does not filter | `doc_tra_loi` + `kiem-bam-nut ct-ma` |
| B19 | **The first decision wins** — no overwriting | An `o.is_none()` check in the IPC receiver |
| B20 | Button behaviour is declared in the **signed manifest**, not in `ui.json` | `Manifest::actions` sits inside the signature's scope |
| B21 | An action **cannot request** a capability that was never asked for | `kiem_hanh_vi` + `hanh_vi_goi_may_chu_chua_xin_quyen_thi_tu_choi` |
| B22 | **Not one packet leaves the machine** before the grant | `chua_cap_quyen_thi_khong_goi_ra_ngoai_mot_lan_nao` |
| B23 | **Redirects are NOT followed** — that is a capability escape | `max_redirects(0)` + `moi_chuyen_huong_deu_bi_tu_choi` |
| B24 | HTTPS only, with a timeout and a size ceiling | `tcc-net` |
| B25 | The path out to the network is **visible in the dependency tree** | Rule 8: only `tcc-shell` depends on `tcc-net` |
| B26 | A remembered permission is bound to the **signer's key**, not just the app id | `doi_khoa_nguoi_ky_thi_phai_hoi_lai` |
| B27 | A remembered permission is bound to the **scope**, not just the capability name | `noi_rong_pham_vi_thi_phai_hoi_lai` |
| B28 | A corrupt permission store → **ask again**, never fall back to allow | `tep_hong_hoac_phien_ban_la_thi_hoi_lai` |
| B29 | Signing key changed → **warn**, and the warning comes BEFORE the permission list | `doi_khoa_ky_thi_canh_bao_hien_ra_truoc_danh_sach_quyen` |
| B30 | Control labels are **visible to people who look**, not only `aria-label` | `nhan_cong_tac_hien_ra_cho_nguoi_nhin_thay`, `nhan_o_nhap_hien_ra_cho_nguoi_nhin_thay` |
| B31 | A destructive tone is **drawn differently**, not merely declared | `sac_thai_mat_mat_duoc_ve_khac_di` |
| B32 | Input fields carry **NO ARIA role** — ARIA overrides native semantics | `moi_nut_deu_mang_vai_tro_ro_rang` |
| B33 | The "destructive" signal **reaches the OS accessibility axis** | `nut_mat_mat_mang_cau_canh_bao` |
| B34 | Package files are served over a **custom protocol**, paths through `check_path` | `duong_dan_di_ra_ngoai_goi_bi_chan` |
| B35 | **Images only**, by an extension ALLOWLIST — **no SVG** | `chi_phuc_vu_anh`, `svg_khong_duoc_phuc_vu` |
| B36 | The permission dialog **serves no app file at all** | `open(..., \|_\| None, ...)` |
| B37 | Decisions **never read the description** stored on disk | `quyet_dinh_khong_doc_phan_mo_ta` |
| B38 | A destructive button **does not stretch the full width** | `nut_khong_gian_kin_be_ngang` |
| B39 | **Machine markers are separate from human text** — text translates, markers never change | `doi_chu_sang_ngon_ngu_khac_khong_lam_mat_dau_hieu_may` |
| B40 | A manifest field the standard does not define is **rejected** | Conformance: three `truong la` vectors, mutation-tested in both directions |

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

### 3.5 No wallet code touches a real private key yet

As planned. The hard gate:

> **No transaction reaches mainnet before an independent security audit.**

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
cargo run -p tcc-conformance                 # 104 cases
cargo run -p tcc-conformance -- --chi-tiet
```

---

## 4. Reproducing everything

```bash
cargo test --workspace                              # 234 tests
cargo test --workspace --features tcc-shell/window  # 237 — three more that need a window
cargo run -p tcc-conformance                        # 135 conformance vectors
python3 conformance/doi-chieu-doc-lap.py <vectors>  # dilithium-py cross-check
cargo clippy --workspace --all-targets -- -D warnings
tools/kiem-luat-phu-thuoc.sh                        # 17 architecture rules
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
