# Auditing this repository

Written for someone who has been handed this and asked to find what is wrong
with it. It tells you where the weak parts are rather than where the strong ones
are, because you can find the strong ones yourself and the point of a review is
the rest.

## Start here, in this order

| | |
|---|---|
| 1 | [`../SECURITY.md`](../SECURITY.md) §3 — **what has never been examined.** Read this before anything else; it is the list of places where nobody has looked |
| 2 | [`../spec/GOVERNANCE.md`](../spec/GOVERNANCE.md) §1 — who wrote this and who has reviewed it. The answer is one party for all of it |
| 3 | [`../SECURITY.md`](../SECURITY.md) §1 — the 40 invariants, each naming the test that holds it |
| 4 | [`../spec/0.1/`](../spec/0.1/) — the standard. English is normative |
| 5 | [`../ARCHITECTURE.md`](../ARCHITECTURE.md) — layering, and the 18 machine-enforced rules |

## The wallet is on another branch

`main` does **not** contain the wallet. `crates/tcc-chain`, `crates/tcc-keystore`,
key derivation, 24-word recovery phrases, the web-wallet import path, the
transaction-confirmation screen and the real macOS Keychain all live on
`giai-doan-3.1`, and `SECURITY.md` on `main` has no invariants covering them.

That means a review of `main` alone skips the riskiest part of the project. This
was raised by an outside reviewer on 2026-08-16 (finding F2) and is stated here
rather than fixed by merging, because merging an unaudited wallet into the branch
people are told to read would trade one problem for a worse one.

```bash
git checkout giai-doan-3.1     # the wallet, and the parts most worth attacking
```

## Reproduce every claim in about five minutes

The two Python cross-checks need two packages that are **not** in any
requirements file, and without them both commands fail with a missing-import
error rather than a useful message (finding F5):

```bash
pip install dilithium-py blake3
```

```bash
cargo test --workspace                              # 293 tests
cargo test --workspace --features tcc-shell/window  # 296, three need a window
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
tools/kiem-luat-phu-thuoc.sh                        # 18 architecture rules
cargo run -p tcc-conformance                        # 138 conformance vectors, eight groups
cargo run -p tcc-cli -- verify examples/hello-tcc
cargo run -p tcc-fuzz --release                     # byte-mutation fuzzer
python3 conformance/doi-chieu-doc-lap.py            # cross-check vs dilithium-py
python3 conformance/dung-goi-doc-lap.py             # package built in Python, verified by Rust
cargo audit                                         # 0 vulnerabilities, 14 known warnings
tools/kiem-so-lieu.sh                               # the two numbers above must be real
```

The commands above run on **macOS, Linux and Windows** in CI. Windows was added
last and broke immediately — not the code, the checkout: git rewrites LF to CRLF
there by default and a signature covers raw bytes, so every signature in the
repository died. `.gitattributes` fixes it here, and `../spec/0.1/01-package.md`
records the general form, which is that any transport normalising text destroys
a package.

These next ones drive a real WebKit view. On macOS they use WKWebView; on Linux,
install `libwebkit2gtk-4.1-dev` and prefix with `xvfb-run -a`. CI runs both, but
the Linux run cannot fail the build — WebKitGTK under a virtual display does not
reliably get a window handle:

```bash
cargo run -p tcc-shell --features window --example kiem-khoi-tan-cong
cargo run -p tcc-shell --features window --example kiem-khoi-tan-cong chi-csp
cargo run -p tcc-shell --features window --example kiem-man-hinh-ung-dung -- examples/hello-tcc
```

If any of these disagree with what the documents claim, the documents are wrong
and that is itself a finding.

## Where I would attack it

Ranked by where a real flaw is most likely, not by how much code there is.

**1. The unauthenticated parse.** `verify_package` must decode the manifest at
steps 1–2 and only verifies the signature at step 4, because the public key
lives inside the manifest. There is no way to reorder it. Every byte of
`manifest.json` reaches `serde_json` and `validate_shape` with nothing
authenticated. Both fuzzers point here; neither is proof.

**2. `ml-dsa` 0.1.1.** No published independent audit. It is behind a trait so
it can be swapped, and the signature is hybrid so Ed25519 still stands if it
fails — but the library parses attacker-controlled bytes.

**3. The interface layer as a whole.** It renders app-supplied text into a
WebKit document. Three layers stop escape and each is tested alone
(`SECURITY.md` B7), but this is the largest surface by far.

**4. Anything the tests assert about *appearance*.** Twice now a rule was
declared, tested, green, and invisible on screen: `Tone::Danger` drawn
identically to everything else (B31), and the wallet capability that must look
different from every other one and did not. The accessibility tree proves a
blind user can hear it; it proves nothing about what a sighted user sees.

**5. The permission dialog.** It is drawn through WebKit, in a separate process
from the app, and the single-event-loop architecture means the two never exist
at once (§3.1b). That guarantee is architectural, not enforced — anyone moving
to a multi-window loop breaks it silently.

## Known weak, already written down

Do not spend time rediscovering these; they are in `SECURITY.md` §3 with the
reasoning:

- No key registry. **A signature proves integrity, never identity.** Packages
  are self-signed.
- No independent security audit — that is what you are doing.
- `verify` leaks by timing which half of a signature failed (35 µs against
  221 µs). Left in deliberately, with the condition for revisiting stated.
- No constant-time proof for signing; the measurement is a screen, not a proof.
- No container format. A package is a directory.
- No package updates, no navigation, no WASM.

## The two things this project most needs from you

**A second implementation.** Everything else is documentation. Until an
implementation written by someone else passes
[`../conformance/`](../conformance/), "conformant to TCC 0.1" means "agrees with
this one implementation" and nothing more. The vectors are plain JSON and
[`../conformance/FORMAT.md`](../conformance/FORMAT.md) is the schema.

**A reading of [`../spec/0.1/`](../spec/0.1/) by someone who has not read the
source.** Build a package directory from the specification text alone. If you
need to ask a question, or to look at the code, the specification has failed and
the question is the finding. Auditing the clauses this way already produced four
classes of defect that 237 passing tests were blind to — including four error
codes that could never fire and a requirement the standard gave no means to
satisfy (`../spec/README.md` records them).

## How to report

Not a public issue. Use GitHub's private vulnerability reporting on this
repository, or the TCC IT department.

A report that the **design** is wrong is worth more here than a report that the
code disagrees with the design. The code has 238 tests watching it. The design
has had one pair of eyes.
