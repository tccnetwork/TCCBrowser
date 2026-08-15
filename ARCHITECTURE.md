# TCC Browser — Architecture

> How the pieces fit together and **why they are divided this way**. The standard
> itself lives in [`spec/`](spec/), the implementation plan in
> [`docs/ke-hoach.md`](docs/ke-hoach.md), and the original v0.1 draft is kept for
> reference in [`docs/dac-ta-goc-v0.1.md`](docs/dac-ta-goc-v0.1.md). Those two
> `docs/` files are internal working notes and are in Vietnamese.

---

## 1. Three tiers of content

The browser opens three kinds of thing, and **deliberately** refuses to pretend
they are the same kind of thing.

```
┌──────────────────────────────────────────────────────────────┐
│                        TCC BROWSER                           │
│                                                              │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐  │
│  │  TIER 1        │  │  TIER 2        │  │  TIER 3        │  │
│  │  TCC apps      │  │  Modern web    │  │  Escape hatch  │  │
│  ├────────────────┤  ├────────────────┤  ├────────────────┤  │
│  │ Declarative    │  │ HTML/CSS/JS    │  │ Hand off to    │  │
│  │ Capabilities   │  │ to published   │  │ the OS browser │  │
│  │ Wallet,identity│  │ standards      │  │                │  │
│  │ PQ signatures  │  │                │  │ Netflix, DRM,  │  │
│  │                │  │ "TCC Ready"    │  │ sites too old  │  │
│  │ ← THIS IS THE  │  │                │  │ to care about  │  │
│  │   PRODUCT      │  │                │  │                │  │
│  └────────────────┘  └────────────────┘  └────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

**Tier 3 is what makes the whole strategy viable.** Without it we are forced to
chase Chromium forever — a race nobody wins. With it, we are allowed to say "we
don't run that page" and the user still gets their work done.

---

## 2. The dependency tree

Arrows read "depends on". Their direction **must never be reversed**.

```
                    ┌──────────────┐
                    │ tcc-browser  │  the application (thin)
                    └──────┬───────┘
                           │
                    ┌──────▼───────┐
                    │  tcc-shell   │  ★ ASSEMBLY POINT
                    │              │    the ONLY place a renderer is chosen
                    └──┬────────┬──┘
              ┌────────┘        └────────┐
              │                          │
      ┌───────▼────────┐      ┌──────────▼─────────┐
      │  tcc-runtime   │      │ tcc-render-webview │  scaffolding
      └───┬────┬───┬───┘      └──────────┬─────────┘
          │    │   │                     │
          │    │   └─────────┐  ┌────────┘
          │    │             ▼  ▼
          │    │        ┌─────────┐
          │    │        │ tcc-ui  │  ABSTRACT component API
          │    │        └────┬────┘
          │    │             │
   ┌──────▼──┐ │        ┌────▼─────────┐
   │tcc-mani-│ │        │              │
   │  fest   │ │        │              │
   └──┬───┬──┘ │        │              │
      │   │    │        │              │
      │   │  ┌─▼────────▼──┐           │
      │   └─►│ tcc-capabi- │           │
      │      │    lity     │           │
      │      └──────┬──────┘           │
      │             │                  │
   ┌──▼──────┐   ┌──▼──────────────────▼──┐
   │tcc-crypto│   │       tcc-spec        │
   └─────────┘   └───────────────────────┘
      ★ LEAF          ★ LEAF
   trust boundary   the standard's
                     data types
```

`tcc-net` hangs off `tcc-shell` alone — see rule 8 below.

### Why the seams are here

Crates are not divided by **topic**. They are divided by **trust boundaries and
replacement boundaries**:

| Crate | Split out because |
|---|---|
| `tcc-crypto` | **Trust boundary.** It needs an independent audit, so it must carry the fewest dependencies and be readable on its own. |
| `tcc-spec` | **Outsiders must be able to read it.** Anyone implementing the TCC standard needs this crate and nothing else — not the whole browser. |
| `tcc-ui` ⟷ `tcc-render-*` | **Replacement boundary.** WebView today, GPU tomorrow. Apps must never be able to tell. |
| `tcc-net` | **The way out of the machine, made visible.** Only `tcc-shell` may depend on it, so reading `Cargo.toml` proves the app loader cannot open a socket. |
| `tcc-shell` | **Assembly point.** Exactly one place knows which concrete renderer exists. |

The original draft proposed **25 crates**. There are **9**, because creating empty
crates is not modularity — it is empty directories. Split further only for a
**real reason**: a new trust boundary, or something that must be replaceable.

---

## 3. Hard rules — machine-enforced

Run `tools/kiem-luat-phu-thuoc.sh`. CI runs it **before compiling**.

| # | Rule | Why |
|---|---|---|
| 1 | `tcc-ui` depends on no renderer | Lose this and the escape route from WebView is gone |
| 2 | Only `tcc-shell` depends on `tcc-render-*` | Keep the assembly point singular |
| 3 | `tcc-crypto` is a leaf | A trust boundary must not swell |
| 4 | `tcc-spec` is a leaf | Outsiders can implement the standard |
| 5 | `tcc-runtime` knows no renderer | It speaks only through `tcc-ui` |
| 6 | No DOM/HTML/CSS in the app-facing API | Leak it and apps are welded to WebView |
| 7 | Every conformance vector group is present and runnable | A missing group is a part of the standard nobody can check |
| 8 | Only `tcc-shell` depends on `tcc-net` | The path off the machine stays visible in the dependency tree |
| 9 | The demo key never leaves `examples/` | Anyone can sign with it; a real package signed by it is forgeable by everyone |
| 10 | Every error code in the specification exists in the source | A code that exists only on paper is a promise nobody keeps |
| 11 | The translation does not drift from the normative text | A skewed translation is worse than none — its readers implement a different standard without knowing |
| 12 | The specification contains no dead links | Outsiders reading it have no source code to guess from |
| 13 | No public identifier carries a Vietnamese name | The API surface is what a second implementer reads; see §7 |
| 17 | The rule count stated in the documentation matches the real one | It drifted from 6 to 10 to 12 while the script had more. A reader trusts the number instead of counting |
| 16 | Every error code in the specification has a conformance vector | Rule 10 proves a code exists in the source; only a vector proves it can ever fire. Four could not |
| 15 | Conformance vectors use English keys and `conformance/FORMAT.md` exists | The vectors are the only thing that can settle a conformance claim, and their reader does not speak Vietnamese |
| 14 | The repository has an Apache-2.0 LICENSE and every crate declares it | A public repository with no licence means "all rights reserved" — outsiders may read but not implement, which contradicts the whole point |

> **A rule written in a comment gets violated eventually** — usually at 11pm by
> somebody who just wants it to run. So they are enforced by a machine.

---

## 4. The escape route from WebView

This is the **most important** architectural decision in the project.

```
   PHASE 1                            PHASE 4
   (borrowed scaffolding)             (standing on its own)

   TCC app                            TCC app
        │                                  │
        ▼                                  ▼
   ┌─────────┐                        ┌─────────┐
   │ tcc-ui  │ ◄── the app knows ───► │ tcc-ui  │
   └────┬────┘     only this layer    └────┬────┘
        │                                  │
        ▼                                  ▼
  ┌───────────┐                     ┌────────────┐
  │ WebView   │                     │ GPU render │
  │ (WKWebView│                     │  (wgpu)    │
  │  WebView2)│                     └────────────┘
  └───────────┘
                    APPS CHANGE NOT ONE LINE
```

**The trap to avoid:** if TCC apps were written directly in HTML/CSS/JS running
inside a WebView, then on the day a native renderer exists **every app has to be
rewritten** — and at that point nobody dares drop WebView. The scaffolding becomes
the building.

Rules 1, 2, 5 and 6 exist **solely** to keep the right-hand box reachable.

The seam is no longer only a plan: `tcc-render-webview` is built and exercised
on **WebKitGTK under Linux** as well as WKWebView on macOS, in CI. The renderer
being replaceable was an argument until something else actually ran it.

A side benefit: the constraint forces `tcc-ui` to be designed properly from the
start. If it is abstract enough to run on two genuinely different renderers, it
was designed correctly.

---

## 5. The thin slice — the path a TCC app travels

This is what Phase 1 has to make work end to end.

```
  package directory
         │
         │  ┌─────────────────────────────────────────┐
         ├─►│ 1. Verify signature      [tcc-crypto]   │
         │  │    Ed25519 + ML-DSA (HYBRID)            │
         │  │    Fails → stop, say exactly where      │
         │  └─────────────────────────────────────────┘
         │
         │  ┌─────────────────────────────────────────┐
         ├─►│ 2. Read the manifest     [tcc-manifest] │
         │  │    who signed? what is requested?       │
         │  └─────────────────────────────────────────┘
         │
         │  ┌─────────────────────────────────────────┐
         ├─►│ 3. Build capability set [tcc-capability]│
         │  │    NOTHING pre-granted. The user rules. │
         │  └─────────────────────────────────────────┘
         │
         │  ┌─────────────────────────────────────────┐
         ├─►│ 4. Run                   [tcc-runtime]  │
         │  │    the app reaches only what was granted│
         │  └─────────────────────────────────────────┘
         │
         │  ┌─────────────────────────────────────────┐
         └─►│ 5. Draw the interface    [tcc-ui]       │
            │    → WebView (phase 1)                  │
            └─────────────────────────────────────────┘
```

Getting this path to run **is having a browser** — it opens something, and that
something is a thing Chrome cannot open.

---

## 6. Cryptography: HYBRID, not pure post-quantum

```
   SIGNATURES                     KEY EXCHANGE
   ┌────────────┐                 ┌────────────┐
   │  Ed25519   │ classical       │   X25519   │ classical
   │     +      │                 │     +      │
   │   ML-DSA   │ post-quantum    │   ML-KEM   │ post-quantum
   │ (FIPS 204) │                 │ (FIPS 203) │
   └────────────┘                 └────────────┘
     Safe while EITHER one still stands
```

**Why hybrid rather than pure post-quantum:** in 2022 **SIKE** — a finalist in the
NIST competition — was broken **on a single CPU core in about an hour**.
Post-quantum algorithms are too young to be trusted alone.

**What is urgent and what is not:**

| | Urgency | Why |
|---|---|---|
| Key exchange | **URGENT** | An attacker records traffic today and decrypts it later. Today's secrets leak in the future. |
| Signatures | Not urgent | Nobody forges a 2026 signature retroactively in 2040 |
| Symmetric (AES-256, SHA-384) | **No change needed** | Grover halves the effective bits — AES-256 still gives 128, still ample |

Effort spent replacing AES is wasted. Only **asymmetric** cryptography falls to
Shor.

---

## 7. Conventions

**Identifiers in code: English. Comments and internal docs: Vietnamese.**

Because `spec/` is a standard for outsiders to read and implement, type names,
function names and crate names must be English. The team maintaining this reads
Vietnamese, so comments are written in Vietnamese — the same convention as v1.

The boundary is `pub`, and **rule 13 enforces it by machine**. All 216 public
identifiers are English; module filenames follow.

What deliberately stays Vietnamese: comments, test function names, local
variables, and the adversarial example binaries. Those are **reasoning**, not
interface — and `SECURITY.md` cites test names as evidence, so renaming them
would damage the very thing they document.

> **This rule drifted for months** before rule 13 existed, and rule 13 caught
> three identifiers a manual sweep had already missed. It was the only convention
> in this document with no machine watching it, and it was the only one that
> drifted. That is not a coincidence, and it is the argument for every other rule
> in §3.

**Comments explain WHY, not WHAT.** The code already states what it does. What it
cannot state is why this approach was chosen, and which approach was tried first
and failed.
