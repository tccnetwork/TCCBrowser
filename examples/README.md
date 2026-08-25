# Examples

Two signed packages, committed to the repository. Verifiable immediately, with
nothing to build first:

```sh
cargo run -p tcc-cli -- verify examples/hello-tcc
cargo run -p tcc-cli -- verify examples/vi-du-vi
```

---

## `vi-du-vi` — what a **wallet** request looks like

`hello-tcc` asks for one capability, and it is a network one. That leaves the
most consequential row in the permission dialog — the wallet — with nothing to
show it against. This package asks for `wallet` **with**
`may_request_signature`, so the dialog renders the row that says, in the frame's
own words rather than the app's:

> Can ask you to sign transactions — **this moves money**

Three things are worth looking at rather than reading about:

| Look at | Why |
|---|---|
| The wallet row against the network row | B45: the warning mark has to sit on **that sentence**, not merely somewhere on the screen |
| The "Gửi tiền" button's **double frame** | B31: a single-ink renderer cannot say *destructive* in colour, so it says it in shape — and B51's focus ring is drawn **outside** so the two never merge |
| Pressing "Gửi tiền" | `gui-tien` is **not declared in the signed manifest**. The button exists; the behaviour does not. It is refused, and the frame says so instead of failing quietly |

⚠️ **No mainnet transaction exists here, or anywhere in this project.** The hard
gate holds: nothing real until an independent security audit. The package says
so on its own screen.

---

# `hello-tcc`

A signed TCC package, committed to the repository. Verifiable immediately, with
nothing to build first:

```sh
cargo run -p tcc-cli -- verify examples/hello-tcc
```

## What it demonstrates

| | |
|---|---|
| The entry point is a **declarative tree** (`ui.json`), not HTML | The app states *what is on screen*; the renderer decides *how it is drawn* |
| Images come **from inside the package** | An image fetched from the network is a beacon revealing what the user is looking at |
| A plain field and a **secret** field | The secret one produces a real password input, which modern operating systems keep out of typing suggestions |
| A **destructive** button | "Delete data" must look unmistakably different from an ordinary button — and must *sound* different to a screen reader |
| One **capability** and one **action** | Pressing "Load sample page" travels through the capability gate, not around it |
| Vietnamese diacritics everywhere | This doubles as the combining-mark and IME test |

`example.com` is reserved by IANA for documentation (RFC 2606) — safe to
demonstrate with, and owned by nobody.

## ⚠️ The signing key is a DEMO key. Everyone has it.

`khoa-vi-du-AI-CUNG-CO.hex` sits in this repository. Anyone can sign a package
carrying that same public key.

**Never use it for a real package.** A CI rule enforces this: the demo public key
must not appear in any manifest outside `examples/`
(`tools/kiem-luat-phu-thuoc.sh`, rule 9).

This restates what `SECURITY.md` §3.1 already says: a valid signature proves the
package was **not modified**; it does **not** prove who signed it. The demo key
makes that visible rather than theoretical — its signatures are perfectly valid
and tell you nothing at all.

## Edit, then re-sign

```sh
examples/ky-lai.sh
```

Change `content/` without re-signing and `tcc verify` reports a content-hash
mismatch — which is exactly correct behaviour, and the cheapest possible
demonstration that the hash chain works.
