# 04 — Capabilities

## Principle

**A capability does not exist until it is granted.** An app has no default
permissions. There is no such thing as a "harmless default permission".

An implementation **SHOULD** enforce this through its **type system**, not through
discipline: with no grant there is no value to hold, and code that reads the
network does not compile. "Forgetting a check" then becomes impossible.

## Declared in the manifest

```json
{
  "name": "network",
  "scope": { "kind": "network", "hosts": ["shop.tcc-coin.com"] },
  "reason": "Load the product list"
}
```

| Field | |
|---|---|
| `name` | MUST be one of `network`, `storage`, `wallet`. Otherwise → `unknown-capability` |
| `scope` | The scope; its `kind` MUST match `name` |
| `reason` | A human-readable reason, MUST be non-empty, subject to the string check in [02](02-manifest.md) |

**Each capability may be declared ONCE.** A repeat → `duplicate-capability`.

⚠️ This is not about tidiness. An app declares `network: [safe.com]` for whoever
reviews the package, then declares `network: [evil.com]` further down — the
granting layer takes the later entry, and what gets granted is the second one. An
implementation **MUST** block this at **both the manifest layer and the granting
layer**; neither should trust the other to have done it.

## Three scope kinds

### `network`

```json
{ "kind": "network", "hosts": ["shop.tcc-coin.com", "cdn.tcc-coin.com"] }
```

`hosts` **MUST** be non-empty — a network capability must name its servers.

`hosts` **MUST NOT** contain `*`. A wildcard turns a finite scope into an infinite
one, and gives the user nothing to evaluate.

Each host **MUST** be a valid domain name:

- ASCII (use punycode for internationalised names) — `non-ascii-host`
- 1–253 characters, each label 1–63
- Only **letters, digits, `-`**; a label must not start or end with `-`
- MAY have exactly one trailing `.` (absolute form)

⚠️ **Checking "is ASCII" is NOT ENOUGH.** The string
`shop.tcc-coin.com:8080@evil.example` is ASCII, non-empty, wildcard-free — it
passes all of that. But build a URL from it and everything before `@` becomes
**userinfo**, while the real host is `evil.example`. The permission dialog shows
that whole string, and a hurried reader sees "shop.tcc-coin.com".

**The general rule:** a string about to enter another syntax (URL, path, command)
must be validated against **that target syntax**, not against "does it contain
odd characters".

### `storage`

```json
{ "kind": "storage", "quota_bytes": 1048576 }
```

### `wallet`

```json
{ "kind": "wallet", "may_request_signature": true }
```

`may_request_signature` is the **only capability that can move the user's money**.
An implementation **MUST** make it visually distinct from every other capability,
and **MUST** say in plain language that it moves money — not a generic word like
"wallet".

## Host matching rules

When an app calls a host, an implementation **MUST** match **exactly**.

| Granted | Called | |
|---|---|---|
| `shop.tcc-coin.com` | `shop.tcc-coin.com` | ✅ |
| `shop.tcc-coin.com` | `SHOP.TCC-COIN.COM` | ✅ domain names are case-insensitive |
| `shop.tcc-coin.com` | `shop.tcc-coin.com.` | ✅ a trailing dot is the same host |
| `tcc-coin.com` | `shop.tcc-coin.com` | ❌ **subdomains do NOT match** |
| `shop.tcc-coin.com` | `tcc-coin.com` | ❌ |
| `tcc-coin.com` | `evil-tcc-coin.com` | ❌ **suffix matching is a hole** |
| `shop.tcc-coin.com` | `shop.tcc-coin.com.evil.example` | ❌ |

Normalise before comparing: strip a trailing dot, lowercase. Then compare for
**equality**, not "ends with".

## Revocation

Revocation **MUST** take effect **immediately**, including for capability handles
the app is already holding. An implementation **MUST NOT** let a stale handle keep
working after revocation.

## Asking the user

An implementation **MUST** ask **item by item**, not once for the whole package.

- Each item's default **MUST** be **not granted**.
- Consenting without enabling any item grants **nothing**.
- Enabling one capability **MUST NOT** imply another.
- Closing the window, any failure, or any unclear path → **DENY**.

The dialog **MUST** show: the app name · the **specific scope** (name the servers;
never just "network access") · the app's **reason verbatim** · and a warning that
the signature does not prove identity.

The two decision buttons **SHOULD** carry equal visual weight. Making the consent
button more prominent than the refusal button pushes the user one way — at exactly
the most dangerous moment.
