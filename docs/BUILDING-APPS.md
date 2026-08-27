# Building apps for TCC Browser

> **Status: 0.1.** Small on purpose. Read [What you cannot do](#what-you-cannot-do-in-01)
> before you plan anything — that section is the honest one, and it will save you
> more time than the rest of this page.
>
> The normative text is [`../spec/0.1/`](../spec/0.1/). Where this guide and the
> specification disagree, **the specification is right and this page is a bug**.

## What a TCC app is

A **directory**. Not an archive, not a bundle, not a program.

```
my-app/
├── manifest.json      what the app is, and everything it may do
├── content/
│   └── ui.json        the screen, as data
└── signature.hex      written by `tcc sign`
```

Three properties follow from that, and they are the whole design:

1. **An app carries no code.** No script, no bytecode, no HTML. `ui.json` is a
   declarative component tree. The browser decides how to draw it.
2. **Everything an app may do is written in the signed manifest.** A behaviour
   that is not declared cannot run — not "is blocked", *cannot run*: there is no
   path from a screen to an effect except through the manifest.
3. **A capability does not exist until the user grants it**, one item at a time.
   There is no "allow all" button, and that is deliberate.

## Five minutes, start to running

```bash
cargo run -p tcc-cli -- new my-app --id com.example.my-app
cargo run -p tcc-cli -- check my-app                 # no key needed — see below
cargo run -p tcc-cli -- key --ra my-key.hex          # keep this file; it IS your identity
cargo run -p tcc-cli -- sign my-app --khoa my-key.hex
cargo run -p tcc-cli -- verify my-app                # prints exactly what your app asks for
cargo build -p tcc-browser --features window
./target/debug/tcc-browser my-app
```

### The edit loop: `check` while you work, `sign` once at the end

**Editing `manifest.json` or anything under `content/` invalidates the
signature.** The browser will refuse the package until you re-sign it. That is
the first thing every newcomer trips over, and it is working as intended — a
signature covers bytes, so changing a byte breaks it.

So do not sign on every edit. `check` validates the manifest and the component
tree **without a key and without signing**:

```bash
$EDITOR my-app/content/ui.json
cargo run -p tcc-cli -- check my-app     # instant, read-only
# … repeat until it passes …
cargo run -p tcc-cli -- sign my-app --khoa my-key.hex
```

`check` reports the standard's **error codes**, so a failure is something you
can look up rather than guess at:

```
✗ [bad-app-id] mã ứng dụng "khong-co-dau-cham" sai định dạng: cần ít nhất hai đoạn
```

Look `bad-app-id` up in [`../spec/0.1/06-error-codes.md`](../spec/0.1/06-error-codes.md).
The prose around a code may be reworded at any time; **the code will not** —
that is what other implementations compare against.

`check` also cross-checks the **screen against the manifest** — the mistake
almost everyone makes first:

```
⚠ 1 nút/công tắc mang mã hành động KHÔNG có trong bản kê khai:
    • delete-all
```

Both directions are **warnings, never errors**, and the exit code stays 0. An
undeclared action is legal — the starter kit ships one on purpose, so you can
watch it be refused — and a declared action no screen uses is legal too, since
another screen may use it. Turning either into an error would be telling you to
fix something that is not broken.

**`check` validates what you wrote; `verify` validates what you shipped.** Edit
one byte in a signed package and `verify` fails with `[content-hash-mismatch]`
while `check` still passes — that is the division of labour, not a bug. Every
`tcc` command reports the standard's codes this way, so a failure anywhere is
something you can look up.

It deliberately does **not** check two things, and says so on every run: the
signature, and `content_hash` — both are produced by `sign`. On a package fresh
out of `new`, those two fields are empty; `check` substitutes placeholders and
tells you it did, so you can validate everything you actually wrote.

`verify` is worth running before every release. It prints the capabilities and
the effects in plain language — if that list surprises you, your users will be
surprised too, and they are the ones being asked to say yes.

**A worked example lives in [`../examples/khoi-dau/`](../examples/khoi-dau/)** —
every component kind in 0.1, rendered, plus one action that is deliberately
*not* declared so you can watch the browser refuse it.

```bash
./target/debug/tcc-browser examples/khoi-dau
```

## The component vocabulary — all of it

Six kinds. There is no seventh.

| `kind` | Required | Optional | Notes |
|---|---|---|---|
| `text` | `content` | `emphasis`: `title` · `normal` · `subtle` · `warning` | A paragraph. Newlines allowed |
| `button` | `label`, `action` | `tone`: `neutral` · `primary` · `danger` | |
| `field` | `label` | `value` | Text input. A masked field is **rejected** — see below |
| `toggle` | `label`, `action` | `on` (default **`false`**) | A switch |
| `image` | `source`, **`alt`** | | `source` is a path inside `content/` |
| `group` | | `flow`: `row` · `column` · `gap`: `none`/`small`/`medium`/`large` · `children` | The only kind that takes children |

`alt` on an image is **required**, not encouraged. A decorative image says so
explicitly (`{"kind": "decorative"}`); there is no way to leave it out and no
default that silently invents a description.

## Capabilities

Declared in the manifest, granted by the user, revocable at any time.

| `name` | Scope field | |
|---|---|---|
| `network` | `hosts`: array of host names | No scheme, no port, no path |
| `storage` | `quota_bytes`: `0 … 2⁵³−1` | `0` is legal: "asks, writes nothing" |
| `wallet` | `may_request_signature`: boolean | `false` means read the address only |

Every capability needs a `reason` written for a human. It is shown to the user
verbatim, so write it as a sentence you would be willing to defend, not as a
category name.

**Host matching has no wildcards and no suffix rule.** `tcc-coin.com` does
**not** match `shop.tcc-coin.com`, and `evil-tcc-coin.com` does not match
either — suffix matching is a hole, so the standard does not have it. List every
host you need.

## Actions and effects

An action is an `id` plus an `effect`. Buttons and toggles reference the `id`.

In 0.1 there is **exactly one effect kind**:

| `kind` | Fields | Meaning |
|---|---|---|
| `fetch` | `host`, `path` | Ask one host for one path |

That is not an oversight, and it is also not a promise that it will stay this
small. It is what has been specified, tested against conformance vectors, and
implemented. Anything else is not yet a standard.

An action id used in `ui.json` but absent from the manifest **does nothing**,
and the browser says so on screen. Try it in the starter kit: the button labelled
*Delete everything* is not declared. That refusal is the mechanism that stops a
tampered screen from inventing behaviour.

## What will get your package rejected

These are checks, not style advice. Each one returns a stable error code — see
[`../spec/0.1/06-error-codes.md`](../spec/0.1/06-error-codes.md).

- **Strings that can lie about themselves.** Control characters, bidirectional
  overrides, and zero-width characters are rejected in every string the user
  sees. Stacked combining marks are capped: eight per character. A name that can
  redraw the screen around it is not a name.
- **Labels are not paragraphs.** `button.label`, `field.label`, `toggle.label`
  and an image's text `alt` reject newlines; `text.content` and `field.value`
  allow them.
- **A package cannot draw a masked input.** The row of dots is a shape users are
  taught to trust, so only the browser frame may draw it. If your app needs a
  PIN, it does not get to ask for one.
- **Unknown manifest fields are rejected**, never ignored. A field the standard
  does not define is an error, because a field silently ignored by one
  implementation and honoured by another is how a standard splits in two.
- **`spec_version` must be exactly `"0.1"`.** Never guessed, never
  "close enough".
- **Duplicate action ids**, duplicate capabilities, and a `scope.kind` that does
  not match its `name` are all errors.

## What you cannot do in 0.1

Read this before designing. None of it is a limitation you can work around;
every item is a decision with a reason, written down.

| | |
|---|---|
| **Run code** | There is no script engine, no bytecode, no plugin. If your idea needs a loop, 0.1 cannot express it |
| **Anything but `fetch`** | One effect kind. No writes, no local files, no timers, no background work |
| **Store data** | The `storage` capability exists and is granted, but 0.1 defines no effect that writes. It is a prompt with nothing behind it yet |
| **Draw your own widgets** | Six component kinds. No custom drawing, no canvas, no colours of your choosing |
| **Prove who you are** | 0.1 has **no key registry**. A signature proves the package was not modified. It does not prove authorship, and no screen will ever say "verified publisher" |
| **Ship an archive** | 0.1 defines no packaging format. A package is a directory; distribution is not solved |
| **Browse the web** | This is not a web browser. There is no web engine in the process, by design |

## About signing

`tcc key` writes a private key to a file. That file **is** your publisher
identity: whoever holds it can sign packages that verify as yours. It is not
protected by the OS keystore, it is not encrypted, and losing it is
unrecoverable in 0.1 because there is no registry to revoke against.

**Never sign anything real with the demo key** in `examples/`. It is published
in this repository so anyone can reproduce the examples — which also means
anyone can forge a package signed by it. The starter kit says so on its own
first screen.

## When you find something wrong

The specification is young and was written by one party. That is stated openly
in [`../spec/GOVERNANCE.md`](../spec/GOVERNANCE.md) §1, and it is the strongest
reason to check our work rather than trust it.

- A rule that seems arbitrary is either wrong or under-explained — both are
  worth reporting.
- A conformance vector that disagrees with the prose is a defect in one of them.
- If the browser and this guide disagree with `spec/0.1/`, **the specification
  wins** and the other two are bugs.

Changing the standard goes through the process in
[`../spec/GOVERNANCE.md`](../spec/GOVERNANCE.md) §4 — a written proposal, not a
pull request that quietly edits a table.
