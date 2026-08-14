> **Historical draft, in Vietnamese.** Superseded by [`spec/0.1/`](../spec/0.1/),
> which is normative. Kept only so decisions can be traced back to where they
> were first argued.

# TCC Engine
## Modern Rust-Native Internet Runtime & Browser

**Document Version:** 0.1  
**Status:** Architecture Proposal / Research & Development  
**Primary Language:** Rust  
**Target Platforms:** macOS, Linux, Windows  
**Primary Goal:** Build a modern, secure, GPU-first, capability-based Internet/application runtime without inheriting unnecessary legacy browser architecture.

---

# 1. Executive Summary

TCC Engine is a new-generation Internet runtime written primarily in Rust.

The project should NOT attempt to reproduce Chromium, Firefox, WebKit, or another traditional browser engine.

The goal is to explore a different architecture:

> A secure application runtime for the modern Internet, with browser compatibility as one capability rather than the foundation of the entire system.

The engine should prioritize:

- Memory safety
- Security by architecture
- Capability-based permissions
- GPU-first rendering
- WASM-first application execution
- Rust-native components
- Strong process isolation
- Modern networking
- Native wallet capabilities
- Decentralized identity
- Zero-knowledge identity capabilities
- Blockchain-native applications
- Native 2D/3D applications
- Modern reactive UI
- Explicit resource ownership
- Minimal legacy assumptions
- Extensibility
- Deterministic and testable subsystems

The engine should eventually support two application models:

1. **Native TCC Applications**
   - Modern component system
   - Rust/WASM
   - Capability-based security
   - GPU-first rendering
   - Native TCC APIs

2. **Compatibility Web**
   - HTML
   - CSS
   - JavaScript
   - Modern web APIs
   - Existing Internet websites

The compatibility layer must remain isolated from the native runtime.

---

# 2. Core Philosophy

The project must follow these principles.

## 2.1 Do not rebuild the old Web blindly

Do not assume that every historical browser feature must exist in the new engine.

Do not implement legacy behavior merely because old browsers implemented it.

The engine should distinguish between:

- Required modern capabilities
- Compatibility requirements
- Legacy behavior
- Deprecated behavior
- Features that should never become part of the core architecture

---

# 2.2 Security must be architectural

Security must not be implemented as a collection of permission dialogs.

The architecture should make dangerous operations impossible unless an explicit capability exists.

For example:

A web application should NOT have:

```text
filesystem access
private key access
arbitrary network access
OS command execution
raw socket access
camera access
microphone access
```

unless those capabilities have explicitly been granted.

---

# 2.3 Capability-based security

Use capability-oriented design as a core architectural principle.

Instead of:

```text
API exists
+
permission = false
```

prefer:

```text
Capability does not exist
```

until granted.

Example:

```text
Application
    |
    +-- GPU capability
    +-- Storage capability
    +-- Network capability
    +-- Wallet capability
    +-- Identity capability
```

Each capability should be explicit, scoped, auditable, revocable, and origin/application-bound.

---

# 2.4 GPU-first architecture

Rendering must be designed around the GPU from the beginning.

Do not build a CPU-oriented rendering architecture and later bolt on GPU acceleration.

The conceptual pipeline should be:

```text
Application
    ↓
Component Tree
    ↓
Layout Tree
    ↓
Scene Graph
    ↓
Render Graph
    ↓
GPU Command Generation
    ↓
GPU
```

The architecture should support:

- 2D UI
- text
- images
- video
- animations
- 3D
- WebGPU-like workloads
- games
- compositing

---

# 2.5 WASM-first application runtime

WebAssembly should be a first-class execution format.

Rust applications should be able to compile into WASM and run inside a secure sandbox.

The architecture should NOT make JavaScript the central application runtime.

JavaScript should initially be treated as a compatibility technology.

Preferred:

```text
Application
    ↓
WASM
    ↓
Capability Runtime
```

rather than:

```text
Application
    ↓
JavaScript
    ↓
Browser VM
    ↓
Web APIs
```

---

# 2.6 Native applications and compatibility websites are different

The architecture must explicitly distinguish:

```text
Native TCC Application
```

from:

```text
Legacy/Compatibility Website
```

Native applications should use:

```text
WASM
Component API
Capability API
GPU API
TCC APIs
```

Compatibility websites may use:

```text
HTML
CSS
JavaScript
DOM
Web APIs
```

The compatibility environment must not weaken the security of the native runtime.

---

# 3. Project Definition

The project should be organized into several major layers.

```text
TCC Engine
│
├── Platform Layer
│
├── Process/Sandbox Layer
│
├── Runtime Layer
│
├── Capability Layer
│
├── Networking Layer
│
├── Storage Layer
│
├── Graphics Layer
│
├── UI/Component Layer
│
├── WASM Runtime
│
├── Identity Layer
│
├── Wallet Layer
│
├── TCC Blockchain Layer
│
├── Native Application Runtime
│
└── Compatibility Web Layer
```

Each subsystem must have a clear interface.

Avoid creating one giant crate.

---

# 4. Proposed Repository Structure

Use a Cargo workspace.

Recommended initial structure:

```text
tcc-engine/
│
├── Cargo.toml
├── README.md
├── LICENSE
├── ARCHITECTURE.md
├── SECURITY.md
├── CONTRIBUTING.md
│
├── crates/
│   │
│   ├── tcc-core/
│   ├── tcc-platform/
│   ├── tcc-window/
│   │
│   ├── tcc-renderer/
│   ├── tcc-gpu/
│   ├── tcc-compositor/
│   ├── tcc-layout/
│   ├── tcc-text/
│   │
│   ├── tcc-component/
│   ├── tcc-ui/
│   │
│   ├── tcc-runtime/
│   ├── tcc-wasm/
│   ├── tcc-capability/
│   │
│   ├── tcc-network/
│   ├── tcc-http/
│   ├── tcc-quic/
│   ├── tcc-storage/
│   │
│   ├── tcc-identity/
│   ├── tcc-wallet/
│   ├── tcc-crypto/
│   ├── tcc-zkp/
│   │
│   ├── tcc-chain/
│   ├── tcc-rpc/
│   ├── tcc-dapp/
│   │
│   ├── tcc-browser/
│   ├── tcc-shell/
│   │
│   └── tcc-cli/
│
├── apps/
│   ├── tcc-browser/
│   ├── tcc-devtools/
│   └── tcc-example/
│
├── tests/
│   ├── integration/
│   ├── security/
│   ├── rendering/
│   ├── networking/
│   └── wasm/
│
├── examples/
│   ├── hello-world/
│   ├── counter/
│   ├── wallet/
│   └── dapp/
│
└── docs/
    ├── architecture/
    ├── security/
    ├── runtime/
    ├── graphics/
    ├── capabilities/
    └── developer/
```

Claude Code must preserve modularity.

Do NOT place all functionality inside `main.rs`.

---

# 5. Technology Strategy

Preferred technology direction:

## Language

Rust.

Use stable Rust unless a feature genuinely requires nightly.

Avoid unsafe Rust wherever practical.

All `unsafe` code must:

- Be isolated
- Have a safety comment
- Have tests
- Have a documented reason
- Be reviewed separately

---

# 6. Graphics

Use a modern GPU abstraction.

A `wgpu`-style architecture is preferred for the initial implementation.

Do not make the rendering layer directly dependent on one GPU API.

Conceptually:

```text
Renderer
   ↓
GPU abstraction
   ↓
Metal / Vulkan / DX12 / other backend
```

The renderer should be backend-independent.

---

# 7. Windowing

Create a platform abstraction:

```rust
trait WindowPlatform {
    fn create_window(...);
    fn resize(...);
    fn set_title(...);
    fn request_redraw(...);
}
```

Do not spread macOS, Windows, and Linux-specific code throughout the engine.

Platform-specific code belongs inside:

```text
tcc-platform
```

---

# 8. Rendering Architecture

The rendering engine should be retained-mode where practical.

Do not immediately render every UI element independently every frame.

Use:

```text
Component Tree
      ↓
Layout Tree
      ↓
Scene Graph
      ↓
Render Graph
      ↓
GPU
```

The renderer should support:

- Rectangles
- Rounded rectangles
- Borders
- Shadows
- Images
- Text
- Clipping
- Transforms
- Opacity
- Layers
- Animation
- 2D primitives
- 3D surfaces

---

# 9. Scene Graph

Define a scene graph similar conceptually to:

```rust
SceneNode {
    transform,
    opacity,
    clip,
    material,
    children,
}
```

Scene nodes should be immutable or snapshot-based during rendering whenever possible.

This reduces synchronization problems.

---

# 10. Render Graph

Introduce a render graph abstraction.

Example:

```text
Frame
 ├── Background
 ├── Main UI
 │    ├── Navigation
 │    ├── Content
 │    └── Sidebar
 ├── Web/Application Surface
 ├── Overlay
 └── Cursor
```

The renderer should determine efficient GPU execution from this graph.

---

# 11. Component System

Do not create a DOM clone.

Create a modern component tree.

Example conceptual API:

```rust
Card {
    Button {
        label: "Send",
        on_click: send_transaction
    }
}
```

Components should have:

- State
- Properties
- Children
- Lifecycle
- Events
- Layout
- Render behavior

Avoid global mutable state.

---

# 12. Reactive State

Implement a minimal reactive state model.

Conceptually:

```rust
Signal<T>
Computed<T>
Effect
```

Example:

```rust
let balance = signal(100);

let display = computed(|| format!("{}", balance.get()));
```

Changing `balance` should invalidate only the necessary component subtree.

Do not automatically rerender the entire application.

---

# 13. Layout System

The layout engine should initially support:

- Absolute positioning
- Flex-like layout
- Grid-like layout
- Intrinsic sizing
- Min/max constraints
- Alignment
- Padding
- Margin
- Aspect ratio

Do not implement the full CSS layout system initially.

The native layout system should be simpler and more predictable.

---

# 14. Text Engine

Text is a critical subsystem.

It must eventually support:

- Unicode
- UTF-8
- Font fallback
- Shaping
- Bidirectional text
- CJK
- Emoji
- Accessibility

Do not implement text shaping from scratch unless necessary.

Use mature libraries where appropriate.

---

# 15. Image System

Support:

- PNG
- JPEG
- WebP
- AVIF
- SVG or an equivalent vector representation

Decode images outside the main UI thread.

Large images must not block rendering.

---

# 16. Animation

Animation should be timeline-driven and GPU-friendly.

Support:

```text
opacity
transform
position
scale
rotation
color
clip
```

Later support:

```text
physics
spring
particle systems
3D animation
```

---

# 17. WASM Runtime

WASM is a first-class runtime.

Requirements:

- Sandboxed execution
- Memory isolation
- Resource limits
- Capability injection
- No implicit filesystem access
- No implicit network access
- No implicit wallet access

A WASM module should receive capabilities explicitly.

Conceptually:

```rust
App {
    wasm_module,
    capabilities: vec![
        Capability::Gpu,
        Capability::Storage,
        Capability::Network(...),
    ]
}
```

---

# 18. Capability System

This is one of the most important subsystems.

Define:

```rust
Capability
CapabilityToken
CapabilityScope
CapabilityManager
CapabilityPolicy
```

Example:

```text
Network
  └── https://api.tcc.network

Storage
  └── app://example/data

Wallet
  └── sign_transaction

Identity
  └── prove_unique_human
```

Capabilities must be:

- Explicit
- Scoped
- Revocable
- Auditable
- Origin-bound
- Application-bound
- User-controlled

---

# 19. Capability URI

Create a standardized internal identifier.

Example:

```text
cap://network/api.tcc.network
cap://storage/example.app
cap://wallet/sign
cap://identity/unique-human
```

Do not expose internal implementation details.

---

# 20. Permission UX

Permissions should be understandable.

Bad:

```text
Application requests access to:
[Permission 0x83A91]
```

Good:

```text
example.tcc wants to:

✓ Connect to TCC Network
✓ Read your public wallet address

⚠ Sign transactions

[Cancel] [Allow]
```

Signing transactions should always require explicit user confirmation unless the user has deliberately configured a trusted automation capability.

---

# 21. Process Model

Do not build a single-process browser.

Recommended architecture:

```text
TCC Browser
│
├── Browser Process
│
├── Renderer Process
│
├── WASM Process / Sandbox
│
├── Network Process
│
├── GPU Process
│
├── Wallet Process
│
└── Storage Process
```

The exact number of processes can evolve.

The important principle is:

> Compromise of one subsystem must not automatically compromise the others.

---

# 22. Wallet Isolation

The wallet must be a separate security boundary.

Never store private keys in:

```text
DOM
JavaScript
WASM application memory
renderer memory
browser UI state
localStorage
cookies
```

Preferred:

```text
Renderer
   ↓
Restricted IPC
   ↓
Wallet Service
   ↓
Secure OS storage
```

The wallet service should expose only high-level operations.

Example:

```text
get_public_address()
get_balance()
create_transaction()
sign_transaction()
```

Never:

```text
get_private_key()
get_seed()
export_raw_key()
```

unless explicitly implementing a secure user-initiated backup workflow.

---

# 23. Secure Key Storage

Use platform secure storage when possible.

macOS:

```text
Keychain
```

Windows:

```text
Windows credential/security facilities
```

Linux:

```text
Secret Service / appropriate OS secure storage
```

The cryptographic abstraction must remain platform-independent.

---

# 24. Cryptography

Create:

```text
tcc-crypto
```

Do not implement cryptographic algorithms manually unless absolutely necessary.

Use audited libraries.

The crypto abstraction should support:

- Hashing
- Digital signatures
- Key derivation
- Secure random generation
- TCC transaction signing
- Future post-quantum algorithms
- Hybrid cryptography

---

# 25. Post-Quantum Architecture

Because TCC is a blockchain project, the engine should be designed so that cryptographic algorithms are replaceable.

Do not hard-code:

```rust
if algorithm == "dilithium" { ... }
```

Instead:

```rust
trait SignatureScheme {
    fn sign(...);
    fn verify(...);
}
```

The engine should be capable of supporting:

```text
Classical
Post-Quantum
Hybrid
```

without redesigning the wallet architecture.

---

# 26. Identity

Identity should be capability-based.

Possible identity levels:

```text
Anonymous
Authenticated
Verified
Unique Human
Organization
Wallet-bound
```

Applications should request proofs rather than raw personal information.

Example:

```text
Application:
"I need proof that this user is a unique verified human."

Identity service:
"Here is a cryptographic proof."
```

The application should not automatically receive:

```text
name
address
date of birth
KYC documents
email
phone number
```

unless explicitly authorized.

---

# 27. Zero-Knowledge Integration

The engine should eventually support ZK proof capabilities.

Example:

```text
prove_age_over_18()
prove_unique_human()
prove_membership()
prove_balance_above()
prove_credential()
```

The application receives:

```text
true + proof
```

rather than unnecessary personal data.

---

# 28. Networking

Networking should be asynchronous.

Use Rust async infrastructure where appropriate.

Initial requirements:

- HTTP/1.1
- HTTP/2
- HTTP/3
- TLS
- DNS
- WebSocket compatibility
- QUIC

Future:

- P2P
- decentralized networking
- TCC protocol
- content-addressed resources

---

# 29. Network Capability

Network access must be scoped.

Bad:

```text
Network::All
```

Preferred:

```text
Network("https://api.tcc.network")
```

or:

```text
Network("https://*.example.com")
```

Applications must not receive unrestricted network access by default.

---

# 30. Storage

Provide isolated application storage.

Conceptually:

```text
app://example.tcc/
```

Applications cannot access:

```text
app://other-app/
```

Storage should support:

- Key/value
- Structured data
- Files
- Cache
- Secure storage

Do not expose raw filesystem access to applications by default.

---

# 31. Browser Navigation Model

The browser shell should support:

```text
Tabs
Windows
History
Bookmarks
Downloads
Profiles
Private browsing
Permissions
Developer tools
```

But these should sit above the engine.

The engine itself should not depend on the browser UI.

---

# 32. Profiles

A profile should contain:

```text
Profile
├── Identity
├── Permissions
├── Cookies
├── Storage
├── History
├── Bookmarks
└── Wallet association
```

Wallets should not automatically be shared across every profile.

---

# 33. Origin Model

The native runtime should not blindly inherit the old browser origin model.

Define a stronger application identity:

```text
Application ID
+
Publisher identity
+
Version
+
Capabilities
```

Example:

```text
app:tcc.example.wallet
publisher:tcc
version:1.2.0
```

This should become part of security policy.

---

# 34. Native Application Manifest

Define a manifest.

Example:

```toml
[application]
name = "TCC Wallet"
id = "com.tcc.wallet"
version = "1.0.0"

[security]
sandbox = true

[capabilities]
network = ["https://api.tcc.network"]
storage = true
wallet = ["read_address", "sign_transaction"]
identity = ["verified_human"]
gpu = true
```

The runtime should validate this manifest before launching.

---

# 35. Application Signing

Native applications should eventually support publisher signatures.

Conceptually:

```text
Application
    ↓
Hash
    ↓
Publisher Signature
    ↓
Manifest
    ↓
Runtime Verification
```

This enables:

- Publisher identity
- Version verification
- Update verification
- Revocation
- Trust levels

---

# 36. Updates

Security updates are critical.

The browser must eventually support:

```text
Signed update
↓
Verify signature
↓
Verify version
↓
Verify package hash
↓
Atomic installation
↓
Rollback if failure
```

Never silently execute an unsigned update.

---

# 37. Compatibility Web Layer

This should be developed separately.

Architecture:

```text
Compatibility Web
│
├── HTML parser
├── CSS parser
├── DOM
├── JavaScript runtime
├── Web APIs
└── Compatibility renderer
```

Do not pollute the native engine with compatibility assumptions.

The compatibility layer should communicate with the core through well-defined interfaces.

---

# 38. JavaScript

JavaScript should initially be optional from the core perspective.

Do not design the entire engine around JavaScript.

A future JS engine can provide:

```text
JavaScript
    ↓
Compatibility APIs
    ↓
Capability system
```

JavaScript must never bypass capabilities.

---

# 39. HTML

HTML should be treated as an input/compatibility format.

The native application model should NOT require:

```text
HTML
+
DOM
+
CSS
```

for every application.

---

# 40. CSS

CSS compatibility should be implemented incrementally.

Do not attempt to reproduce every historical CSS quirk in the first version.

Prioritize modern:

- Flexbox
- Grid
- typography
- transforms
- animations
- responsive layout
- modern color
- modern media

---

# 41. DOM

The DOM should remain isolated from the native component system.

Do not create:

```text
Everything = DOM
```

Instead:

```text
Native Component Tree

and

Compatibility DOM
```

are separate abstractions.

---

# 42. Browser Extensions

Do not copy the extension model immediately.

First define a capability-based application model.

Extensions should eventually become:

```text
Sandboxed Application
+
Additional Capabilities
```

rather than arbitrary code with broad browser privileges.

---

# 43. TCC DApp Runtime

TCC DApps should have first-class support.

Example:

```text
DApp
│
├── UI
├── WASM
├── TCC RPC
├── Wallet
├── Identity
├── Storage
└── Network
```

The DApp should not need browser extensions to interact with TCC.

---

# 44. TCC Provider API

Define a stable provider abstraction.

Conceptually:

```rust
trait TccProvider {
    fn get_chain_id();
    fn get_account();
    fn get_balance();
    fn send_transaction();
    fn sign_message();
}
```

The actual API can later be exposed to:

- WASM
- JavaScript
- native applications

---

# 45. Transaction Signing

Never allow arbitrary silent signing.

The normal flow:

```text
DApp
 ↓
Build transaction
 ↓
Wallet service
 ↓
Validate transaction
 ↓
Display human-readable summary
 ↓
User approval
 ↓
Sign
 ↓
Broadcast
```

---

# 46. Human-readable Transaction UI

The wallet should display:

```text
Application:
DEX Example

Action:
Swap

From:
100 TCC

To:
USDT

Network fee:
0.01 TCC

Destination:
0x....

[Reject] [Approve]
```

Avoid displaying only raw serialized transactions.

---

# 47. Developer Tools

Eventually build native developer tools.

Required:

```text
Inspector
Console
Network
Storage
WASM
GPU
Performance
Security
Capabilities
Blockchain
```

The capability inspector should show:

```text
Application
│
├── Network ✓
├── Storage ✓
├── Wallet
│   ├── Read address ✓
│   └── Sign transaction ✓
├── Camera ✗
└── Microphone ✗
```

---

# 48. Diagnostics

All subsystems should have structured logging.

Use levels:

```text
TRACE
DEBUG
INFO
WARN
ERROR
```

Do not log:

```text
private keys
seed phrases
authentication secrets
session tokens
personal KYC data
```

---

# 49. Security Logging

Security events should be separately identifiable.

Examples:

```text
CAPABILITY_GRANTED
CAPABILITY_REVOKED
WALLET_SIGN_REQUEST
WALLET_SIGN_APPROVED
WALLET_SIGN_REJECTED
IDENTITY_PROOF_REQUEST
SANDBOX_VIOLATION
PROCESS_CRASH
INVALID_SIGNATURE
UPDATE_VERIFICATION_FAILED
```

---

# 50. Testing Strategy

Every subsystem must have tests.

Required:

```text
Unit tests
Integration tests
Security tests
Fuzz tests
Property tests
Rendering tests
Performance tests
Compatibility tests
```

---

# 51. Fuzzing

Fuzz:

- HTML parser
- CSS parser
- URL parser
- network protocol
- manifest parser
- transaction parser
- capability parser
- wallet transaction decoder
- serialization
- WASM boundary

A browser engine should assume malicious input everywhere.

---

# 52. Security Invariants

Create explicit invariants.

Examples:

```text
Renderer cannot read wallet private keys.

WASM cannot access filesystem without capability.

Application cannot access another application's storage.

Network access cannot exceed granted scope.

Untrusted content cannot execute native OS commands.

A DApp cannot sign a transaction without wallet authorization.

Unsigned applications cannot receive trusted capabilities.

Compromising a renderer must not directly compromise the wallet process.
```

These should become automated tests.

---

# 53. Memory Safety

Rust should be the default.

Avoid:

```rust
unsafe
```

unless necessary.

When unsafe is necessary:

```rust
// SAFETY:
// Explain precisely why this operation is safe.
```

No unexplained unsafe code.

---

# 54. Concurrency

The engine should be asynchronous and parallel by design.

Do not create one giant event loop for everything.

Possible model:

```text
UI Thread
GPU Thread
Renderer Workers
Network Runtime
WASM Workers
Storage Worker
Wallet Process
```

Use message passing where appropriate.

Avoid unnecessary shared mutable state.

---

# 55. Error Handling

Do not use:

```rust
unwrap()
expect()
```

in production paths unless justified.

Use structured error types.

Example:

```rust
enum EngineError {
    Network(NetworkError),
    Rendering(RenderError),
    Capability(CapabilityError),
    Wallet(WalletError),
    Runtime(RuntimeError),
}
```

Errors should retain context.

---

# 56. Performance Goals

Initial goals:

- Fast startup
- Low idle memory
- GPU accelerated rendering
- Parallel parsing
- Async networking
- Incremental rendering
- Incremental layout
- Efficient image decoding
- WASM startup optimization

Do not prematurely optimize.

Measure before optimizing.

---

# 57. Performance Architecture

The main UI path should avoid:

```text
UI
 ↓
blocking filesystem
 ↓
blocking network
 ↓
blocking crypto
```

Instead:

```text
UI
 ↓
async request
 ↓
worker/service
 ↓
result
 ↓
UI update
```

---

# 58. Determinism

Where possible:

- deterministic serialization
- deterministic transaction representation
- reproducible builds
- stable tests
- deterministic capability evaluation

This is especially important for blockchain operations.

---

# 59. Reproducible Builds

Eventually support:

```text
Source
 ↓
Pinned dependencies
 ↓
Deterministic build
 ↓
Binary hash
 ↓
Signed release
```

Document the build environment.

---

# 60. Dependency Policy

Do not add dependencies merely for convenience.

Before adding a crate evaluate:

- Security history
- Maintenance
- License
- Dependency tree
- Build complexity
- Unsafe usage
- Performance
- WASM compatibility

Avoid unnecessary dependency chains.

---

# 61. No Node.js Requirement

The core engine should not require Node.js.

Node.js may be used for tooling if necessary, but:

```text
Runtime
Browser
Wallet
Security
Networking
```

must not depend on Node.js.

---

# 62. JavaScript Tooling

JavaScript may still be used for:

```text
build tools
documentation
developer tooling
compatibility layer
```

but not as a requirement for the core runtime.

---

# 63. Cross-platform Strategy

Phase 1:

```text
macOS
Linux
```

Phase 2:

```text
Windows
```

The engine architecture must not make platform assumptions.

---

# 64. Initial MVP

Do NOT implement everything immediately.

MVP 1:

```text
Rust
+
Window
+
GPU
+
Scene Graph
+
Component System
+
Text
+
Input
+
WASM
+
Capability System
```

The first application should be:

```text
Hello TCC
```

rendered entirely by the new engine.

---

# 65. MVP 2

Add:

```text
Networking
HTTP
HTTPS
Storage
Application manifest
Application identity
Developer tools
```

Create:

```text
TCC Example App
```

that can:

```text
render UI
call HTTPS
store data
run WASM
```

---

# 66. MVP 3

Add:

```text
Wallet Service
TCC RPC
TCC Provider
Transaction builder
Transaction signing
Permission UI
```

Create a simple:

```text
TCC Wallet DApp
```

---

# 67. MVP 4

Add:

```text
Identity
Credential
ZK proof interface
Native DApp model
Application signing
Updates
```

---

# 68. MVP 5

Begin compatibility layer:

```text
HTML
CSS
JavaScript
DOM
HTTP websites
```

Start with a deliberately limited modern subset.

Do not attempt full browser compatibility.

---

# 69. Browser Shell

Only after the underlying engine is stable should the project build the complete browser UI.

Browser:

```text
Tabs
Address bar
Back
Forward
Reload
Bookmarks
History
Downloads
Profiles
Settings
Developer tools
Wallet
Identity
```

---

# 70. Example Native TCC Application

Conceptual application:

```rust
fn app() -> App {
    App::new()
        .title("TCC Wallet")
        .capability(Wallet::ReadAddress)
        .capability(Wallet::SignTransaction)
        .capability(Network::Host("api.tcc.network"))
        .view(|| {
            Column::new()
                .text("TCC Wallet")
                .button("Send")
        })
}
```

The API is illustrative.

The actual API should be designed carefully before implementation.

---

# 71. Example Capability Request

Application:

```text
Request:

wallet.sign_transaction
```

Runtime:

```text
Is capability declared?
        |
       Yes
        ↓
Is publisher trusted?
        |
       Yes
        ↓
Is user permission granted?
        |
       Yes
        ↓
Wallet confirmation
        |
       Yes
        ↓
Sign
```

---

# 72. Threat Model

Assume:

```text
Malicious websites
Malicious DApps
Compromised dependencies
Malicious WASM
Malicious JavaScript
Malformed documents
Network attackers
Compromised renderer
Compromised extension/application
Supply-chain attacks
```

The engine must be designed under the assumption that all external content is hostile.

---

# 73. Security Boundary

Primary boundaries:

```text
OS
 |
Browser process
 |
Renderer
 |
Application/WASM
 |
Capabilities
 |
Privileged services
```

Wallet and identity services should be treated as highly privileged.

---

# 74. Zero Trust Between Components

Do not trust IPC messages simply because they originate from another internal process.

Every privileged service should verify:

```text
sender identity
application identity
capability
request parameters
session
origin
```

---

# 75. IPC

Create an explicit IPC protocol.

Messages should be:

```text
typed
versioned
validated
authenticated
auditable
```

Avoid passing arbitrary serialized objects.

---

# 76. Browser-to-Wallet IPC

Never expose:

```text
sign(raw_bytes)
```

without validation.

Prefer:

```text
sign_transaction(
    application_id,
    chain_id,
    transaction
)
```

Wallet service should validate:

```text
chain
recipient
amount
fee
nonce
contract
```

before displaying confirmation.

---

# 77. Future P2P Networking

The architecture should allow:

```text
HTTP
HTTPS
QUIC
P2P
TCC Network
```

without redesigning the application model.

A future resource may be:

```text
tcc://resource
```

or another decentralized URI scheme.

Do not commit to the exact scheme until properly designed.

---

# 78. Blockchain-Native Resource Model

Eventually applications may consume:

```text
https://...
tcc://...
ipfs://...
content://...
```

The runtime should abstract resource resolution.

---

# 79. Content Addressing

Future support could include:

```text
content hash
 ↓
immutable resource
 ↓
verified content
```

This could become useful for:

- DApps
- decentralized websites
- application packages
- static assets

---

# 80. Application Distribution

Eventually:

```text
Developer
 ↓
Build
 ↓
Sign
 ↓
Publish
 ↓
Registry
 ↓
User
 ↓
Verify
 ↓
Install
```

The registry does not need to be centralized.

It may eventually support:

```text
TCC ecosystem registry
```

alongside normal HTTPS distribution.

---

# 81. Governance of Runtime APIs

Do not rapidly add APIs.

Every new privileged capability should have:

```text
Threat model
Security review
API design
Permission model
Test
Documentation
```

---

# 82. API Stability

Mark APIs:

```text
Experimental
Stable
Deprecated
Removed
```

Never silently change security semantics.

---

# 83. Backward Compatibility

Compatibility is important, but should not dictate core architecture.

Rule:

> Compatibility belongs at the edge.

Not:

> Compatibility defines the engine.

---

# 84. Development Workflow for Claude Code

Claude Code must NOT attempt to implement the entire project in one operation.

It must work incrementally.

For every phase:

```text
1. Read architecture
2. Inspect repository
3. Plan
4. Implement
5. Compile
6. Test
7. Review security
8. Benchmark
9. Document
10. Commit/checkpoint
```

Never skip compilation and testing.

---

# 85. Claude Code Rules

Claude Code must:

1. Never rewrite large parts of the architecture without explaining why.
2. Never introduce a dependency without justification.
3. Never introduce unsafe Rust without documentation.
4. Never store private keys in application memory longer than necessary.
5. Never expose privileged APIs directly to untrusted content.
6. Never bypass the capability system.
7. Never implement security as UI-only permission checks.
8. Never use global mutable state unless justified.
9. Never create a monolithic crate.
10. Never silently change public APIs.
11. Write tests alongside new functionality.
12. Keep documentation synchronized with architecture.
13. Prefer small incremental commits.
14. Measure performance instead of guessing.
15. Treat every external input as untrusted.

---

# 86. Claude Code Must Ask Before Major Architectural Changes

Before making changes that affect:

```text
process model
security model
capability model
wallet architecture
cryptographic architecture
public API
serialization format
application identity
IPC
```

Claude Code must stop and explain the proposed change before proceeding.

---

# 87. Definition of Done

A feature is NOT complete merely because it compiles.

It is complete only when:

```text
Code
+
Tests
+
Error handling
+
Security review
+
Documentation
+
Examples
```

are present.

---

# 88. First Development Task

Claude Code should NOT start by implementing the browser.

First create:

```text
tcc-engine/
```

with the Cargo workspace and foundational crates.

Implement:

```text
tcc-core
tcc-platform
tcc-window
tcc-gpu
tcc-renderer
tcc-component
tcc-runtime
tcc-capability
```

Create a minimal executable:

```text
tcc-example
```

that opens a window and renders:

```text
TCC Engine
Modern Internet Runtime
```

using the GPU.

---

# 89. Second Development Task

Implement:

```text
Component Tree
```

with:

```text
Text
Rectangle
Button
Container
Image placeholder
```

Implement input:

```text
mouse
keyboard
window resize
```

No HTML yet.

---

# 90. Third Development Task

Implement reactive state:

```text
Signal
Computed
Effect
```

Create a counter example.

Verify that changing state only updates affected components.

---

# 91. Fourth Development Task

Implement WASM runtime.

Create:

```text
hello.wasm
```

and execute it inside a sandbox.

WASM must have no filesystem/network access by default.

---

# 92. Fifth Development Task

Implement capabilities.

Create:

```text
StorageCapability
NetworkCapability
GpuCapability
```

Demonstrate:

```text
Application A
```

cannot access:

```text
Application B storage
```

---

# 93. Sixth Development Task

Implement networking.

Create:

```text
NetworkCapability("https://example.com")
```

Attempting:

```text
https://another-domain.com
```

must fail if not authorized.

---

# 94. Seventh Development Task

Implement secure IPC.

Create:

```text
Browser Process
Renderer/WASM Process
Network Service
```

Demonstrate that renderer compromise does not automatically grant network service privileges.

---

# 95. Eighth Development Task

Implement TCC Wallet Service.

First implement:

```text
public address
balance
transaction construction
```

Only later implement signing.

---

# 96. Ninth Development Task

Implement transaction signing.

Create a complete test suite around:

```text
valid transaction
invalid transaction
wrong chain
wrong recipient
wrong nonce
wrong amount
wrong signature
malformed transaction
```

---

# 97. Tenth Development Task

Implement native TCC DApp.

The DApp must:

```text
display balance
request wallet connection
request transaction
show confirmation
receive result
```

without requiring a browser extension.

---

# 98. Eleventh Development Task

Implement Identity.

Start with:

```text
Anonymous identity
Wallet identity
Verified credential abstraction
```

Then add ZK proof integration.

---

# 99. Twelfth Development Task

Only now begin compatibility web.

First:

```text
HTML parser
```

Then:

```text
DOM
```

Then:

```text
CSS
```

Then:

```text
JavaScript
```

Do not allow compatibility implementation to contaminate the native engine.

---

# 100. Long-Term Architecture

The final conceptual architecture should be:

```text
                         TCC ENGINE
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
    Native Apps         Compatibility Web      Browser
        │                    │                    │
     WASM               HTML/CSS/JS             Tabs
        │                    │                   History
        │                    │                   Bookmarks
        └────────────┬───────┴───────────────────┘
                     │
              Capability Runtime
                     │
      ┌──────────────┼───────────────────┐
      │              │                   │
     GPU          Network             Storage
      │              │                   │
      ├──────────────┼───────────────────┤
      │              │                   │
   Identity        Wallet              TCC
      │              │                Blockchain
      └──────────────┼───────────────────┘
                     │
                  Secure OS
```

---

# 101. What This Project Is NOT

This project is NOT:

- A Chromium clone
- A Firefox clone
- A WebKit clone
- An Electron replacement
- A JavaScript framework
- A blockchain wallet with a browser attached
- A simple HTML renderer
- A single-process desktop application

It is:

> A new Rust-native runtime for secure Internet applications.

---

# 102. Strategic Objective

The ultimate objective is to enable applications that look more like:

```text
TCC App
│
├── UI
├── 3D
├── WASM
├── Identity
├── Wallet
├── Payment
├── Network
├── Storage
└── Decentralized resources
```

rather than traditional:

```text
HTML
+
CSS
+
JavaScript
+
Browser extensions
+
Wallet extension
+
Third-party authentication
```

---

# 103. Important Design Principle

Do not attempt to eliminate the Web.

Instead:

> Build a better runtime while preserving the Web as a compatibility layer.

The user should be able to:

```text
Open Google
Open GitHub
Open YouTube
Open normal websites
```

while also being able to launch:

```text
Native TCC Applications
```

that use the modern runtime.

---

# 104. Final Vision

The long-term vision is:

```text
             OLD WEB
                │
        HTML / CSS / JS
                │
          Compatibility
                │
                ▼
        ┌───────────────┐
        │   TCC ENGINE  │
        │               │
        │ Rust          │
        │ WASM          │
        │ GPU           │
        │ Capabilities  │
        │ Identity      │
        │ Wallet        │
        │ Blockchain    │
        └───────┬───────┘
                │
          NEXT-GENERATION
           APPLICATIONS
```

The goal is not to ask:

> "How do we make another browser?"

The goal is to ask:

> **"If we were designing the Internet application runtime today, without being constrained by 30 years of browser history, what would we build?"**

That question should drive the architecture.

---

# 105. Final Instruction to Claude Code

Before writing substantial code, Claude Code must:

1. Read this entire specification.
2. Inspect the existing repository.
3. Produce an implementation plan.
4. Identify architectural risks.
5. Identify security risks.
6. Identify decisions that need clarification.
7. Propose the initial Cargo workspace.
8. Implement ONLY Phase 1 after the plan is approved.
9. Keep every subsystem modular.
10. Never sacrifice the security architecture for short-term convenience.

The project must evolve incrementally.

The priority order is:

```text
Architecture
    ↓
Security
    ↓
Correctness
    ↓
Testability
    ↓
Performance
    ↓
Compatibility
    ↓
Convenience
```

Do not reverse this order.

---

# 106. Phase 1 Acceptance Criteria

Phase 1 is complete only when all of the following are true:

- [ ] Cargo workspace builds successfully
- [ ] macOS build works
- [ ] Linux build works
- [ ] Window opens
- [ ] GPU renderer works
- [ ] Component tree works
- [ ] Text renders
- [ ] Input works
- [ ] Basic layout works
- [ ] Reactive state works
- [ ] No unnecessary unsafe Rust
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Architecture documentation exists
- [ ] Security assumptions are documented
- [ ] Example application works
- [ ] No wallet/private-key functionality exists yet
- [ ] No compatibility browser functionality is mixed into the core engine

---

# 107. The Most Important Rule

Do not rush.

The project should be built as a sequence of small, independently testable systems.

A small, elegant, secure rendering engine is more valuable than a huge incomplete browser.

The first milestone is not:

> "Open YouTube."

The first milestone is:

> **"We have built a fundamentally new secure runtime architecture that can eventually host an entire class of modern applications."**

Once that foundation is correct, the browser, DApps, wallet, identity, blockchain integration, games and other applications can be built on top of it.