# 05 — Interface

## Apps ship no code

The entry point is a **declarative component tree**: JSON, at most **1 MiB**.

Not HTML. Not script. The app states *what is on screen*; the implementation
decides *how it is drawn*.

**Why:** if apps shipped web markup, then the day a different renderer arrives,
**every app would have to be rewritten** — and at that point nobody dares change
renderers. The scaffolding becomes the building.

## Example

```json
{
  "kind": "group",
  "flow": "column",
  "gap": "large",
  "children": [
    { "kind": "text", "content": "Hello from TCC", "emphasis": "title" },
    { "kind": "image", "source": "img/logo.png",
      "alt": { "kind": "text", "text": "TCC logo" } },
    { "kind": "field", "label": "Password", "secret": true },
    { "kind": "button", "label": "Delete data", "action": "delete", "tone": "danger" }
  ]
}
```

## Six node kinds

| `kind` | Fields | |
|---|---|---|
| `text` | `content` MUST · `emphasis` (`title`/`normal`/`subtle`, default `normal`) | A paragraph; newlines ARE allowed |
| `button` | `label` MUST · `action` MUST · `tone` (`neutral`/`primary`/`danger`, default `neutral`) | |
| `field` | `label` MUST · `value` (default empty) · `secret` (default `false`) | Text input |
| `toggle` | `label` MUST · `action` MUST · `on` (default **`false`**) | Switch |
| `image` | `source` MUST · `alt` **MUST** | An image from the package |
| `group` | `flow` (`row`/`column`, default `column`) · `gap` (`none`/`small`/`medium`/`large`, default `medium`) · `children` | The ONLY kind that takes children |

Unknown fields **MUST** be rejected. They are almost always a typo, and ignoring
them silently means the author believes a property took effect when it did not.

## No pixels, no colours

Apps declare **INTENT** (`tone: "danger"`, `gap: "large"`), never appearance.

This is a security property, not an aesthetic one: if an app could set colours,
the button that wipes a wallet could look exactly like the cancel button.

In exchange, an implementation **MUST** actually render each intent differently.
Declaring `danger` and drawing it identically to a normal button reduces the whole
tone layer to a code comment.

## Accessibility — no opt-out

Every node **MUST** yield an accessibility node with a **role** and a **label**.

An image's `alt` has **no default**. Omitting it is an **error**, not "treat as
decorative" — "forgot to describe the image" must be blocked, never silently
converted into "this image needs no description".

```json
"alt": { "kind": "text", "text": "Price chart" }   // informative
"alt": { "kind": "decorative" }                     // decorative, MUST be stated explicitly
```

An implementation **MUST**:

- Show a **visible label** for `field` and `toggle`. Providing a label only to
  screen readers leaves sighted users looking at an unlabelled box.
- Render `secret: true` as the platform's **real** password input, so the OS masks
  the text and keeps it out of typing suggestions.
- Tell screen readers that `tone: "danger"` is an action that **cannot be undone**,
  while preserving the fact that it is a **button**.

⚠️ **Do not add accessibility annotations where the native element already says the
right thing.** On the web, putting `role="textbox"` on a password input **overrides**
the native semantics and downgrades it from "secure text field" to "text field" —
at which point screen readers read the password aloud, character by character.

## Tree limits

| | |
|---|---|
| Maximum nodes | **10,000** |
| Maximum depth | **32** |
| Maximum string length | **4,096 characters** (characters, NOT bytes) |
| Maximum file size | **1 MiB** |

> **Why 32 and not something larger.** Each level of the tree costs **two**
> levels of JSON nesting — an object and an array. A limit of 64 therefore needs
> 128 levels of JSON, which is exactly where several common parsers stop by
> default: the parser rejects the document before this rule can run, so
> `too-deep` becomes unreachable and a tree at the documented limit is refused.
> Worse for interoperability, an implementation whose parser nests deeper reports
> `too-deep` while a shallower one reports `bad-json` — one package, two error
> codes. A limit belonging to the standard must not depend on the recursion limit
> of whichever JSON library an implementer happened to pick.

Limits **MUST** be enforced **while building**, not after. A hostile app needs only
a loop to produce an enormous tree, and by then it is already in memory.

Count **characters**, not bytes: cutting on bytes lets Vietnamese text use roughly
half as much as English.

## ⚠️ Decoding MUST NOT bypass validation

A tree loaded from disk **MUST** pass **exactly** the same checks as one built in
code: spoofing-character filters, limits, action-id constraints, the ban on remote
images.

Many JSON libraries will decode straight into the destination structure, filling
fields and **bypassing the validation layer entirely**. An attacker then needs no
exploit at all — only a JSON file.

An implementation **SHOULD** use **two separate types**: a plain data type for
decoding, and a rebuild step that goes through the validating constructors.

## An image's `source`

A path **inside the package**, subject to the constraints in [01](01-package.md).

It **MUST NOT** be a network address. A remotely loaded image is a beacon: whoever
owns that server learns who opened which screen, when, and from what address —
while the app never requested a network capability. Error code: `external-image`.

An implementation that serves package files **MUST**:

1. Serve only files **present in the signed tree**
2. Validate the path with the rules in [01](01-package.md), **after** decoding
   `%XX` escapes and stripping the query string — `%2e%2e%2f` is `../` in disguise
3. Choose the content type from an **extension allowlist**, and **MUST NOT** serve
   SVG: it can execute script and embed remote resources — it is a document, not an
   image

## `action` — action ids

Only **lowercase ASCII letters, digits, `-`, `.`**; 1–128 characters. Error code:
`bad-action-id`.

Action ids are never shown to the user. They connect a button to a behaviour
declared in the manifest — see below.

## Button behaviour

Declared in the **manifest**, NOT in the interface tree:

```json
"actions": [
  { "id": "load-items", "effect": { "kind": "fetch", "host": "shop.tcc-coin.com", "path": "/items" } }
]
```

Three reasons, any one of which would suffice:

1. **The signature covers the manifest** — behaviour is the most dangerous thing an
   app declares; it must not be editable after signing.
2. **The permission dialog reads the manifest** — so it can show "this button calls
   shop.tcc-coin.com".
3. **It keeps the interface layer clean** — declaring it in the tree would mean the
   interface layer has to know about networking.

Constraints:

- `id` MUST be a valid action id, and ids **MUST NOT** repeat
- `path` MUST begin with `/`
- ⚠️ `host` **MUST fall within a granted network capability**

That last one is the most important check on behaviour. Without it an app can
declare a button calling `evil.example` while requesting access only to
`shop.tcc-coin.com`. At run time the capability still blocks it — but **the user
has clicked, nothing happened, and nobody knows why**. Error code:
`action-host-not-granted`.

The matching rule here **MUST** be identical to the run-time matching rule
([04](04-capabilities.md)) — a divergence between the two is a hole.

## Running a behaviour

```text
1. Look the action up in the SIGNED manifest   ← not there, nothing runs
2. Ask the CAPABILITY                          ← not granted, stop here
3. Only then reach outside
```

Step 2 **MUST** precede step 3. Calling first and checking after means the packet
**has already left the machine** — and for a tracking server, arrival alone is
enough.

The action id arrives from a click on screen. Without looking it up in the signed
manifest, a compromised page can invent actions.
