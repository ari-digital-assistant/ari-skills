# AssemblyScript skills

An honest appendix.

## Read this before you start

The AssemblyScript SDK exists so the WASM ABI isn't Rust-only. It is **not**
at parity with the Rust SDK, and nothing in the registry uses it.

Here's the whole surface:

| Module | Exports |
|---|---|
| `assembly` | `ari_alloc`, `input`, `respondText`, `respondAction`, `respond` (deprecated), `unpack`, `log`, `TRACE`/`DEBUG`/`INFO`/`WARN`/`ERROR`, `hasCapability`, `nowMs`, `randU64`, `RESPONSE_TAG_TEXT`, `RESPONSE_TAG_ACTION` |
| `assembly/http` | `httpFetchRaw` |
| `assembly/storage` | `storageGet`, `storageSet` |

And here's what's missing, all of which Rust has:

- **No typed presentation builder.** Cards, alerts, notifications and every
  other envelope primitive have to be hand-written JSON.
- **No i18n.** No `t()`, no `getLocale()`, no `format_*`. Your skill will be
  monolingual, which conflicts with the project's translate-from-day-one
  principle.
- **No settings access.** No `settingGet`/`settingSet`, so no user-configurable
  skills, no `settings_query`, no `settings_action`, no OAuth.
- **No `args()`** — you can't read the arguments the router extracted.
- **No wrappers** for location, tasks, calendar or media services.
- **`httpFetchRaw` returns a raw JSON string**, not a typed response. You parse
  it yourself.

**Use Rust unless you genuinely can't.** [tutorial-wasm.md](tutorial-wasm.md)
is the supported path, and everything above works there. Choose
AssemblyScript only if the JavaScript toolchain is a hard requirement for you,
and accept that you're writing a text-only, monolingual, settings-free skill.

If you want to close these gaps, the SDK is at
[`sdk/assemblyscript`](../sdk/assemblyscript) and PRs are welcome.

## Getting started

Node.js 18+, then:

```bash
cp -r templates/echo-as skills/my-skill
cd skills/my-skill
```

Rename the directory, the `name:` field and the `[package] name` — the
directory name and `name:` must match.

```bash
./build.sh          # npm install + asc → skill.wasm
```

## The code

```typescript
import { ari_alloc, input, respondText, log, INFO } from "ari-skill-sdk-as/assembly";

// REQUIRED. Without this the host cannot write your input into memory.
export { ari_alloc };

export function score(ptr: i32, len: i32): f32 {
  return 0.0;   // never called while matching.custom_score is false
}

export function execute(ptr: i32, len: i32): i64 {
  const text = input(ptr, len);
  log(INFO, "my skill executed");
  return respondText("You said: " + text);
}
```

The input you get is **normalised** — lowercased, contractions expanded,
punctuation stripped, English number words turned into digits. See
[reference-manifest.md](reference-manifest.md#input-normalisation).

## Build flags

```bash
npx asc assembly/index.ts --outFile skill.wasm --optimize --exportRuntime --use abort=
```

`--use abort=` is **mandatory**. Without it the module imports `env::abort`,
which the host doesn't provide, and instantiation fails at install time. The
template's `build.sh` includes it.

## HTTP

```typescript
import { httpFetchRaw } from "ari-skill-sdk-as/assembly/http";

const raw: string | null = httpFetchRaw("https://api.example.com/data");
// raw is {"status": 200, "body": "…"} as a string, or null. Parse it yourself.
```

Needs `capabilities: [http]`.

## Storage

```typescript
import { storageGet, storageSet } from "ari-skill-sdk-as/assembly/storage";

const value: string | null = storageGet("my_key");
const ok: bool = storageSet("my_key", "my_value");
```

Needs `capabilities: [storage_kv]`.

## Action envelopes

Hand-built:

```typescript
import { respondAction } from "ari-skill-sdk-as/assembly";

export function execute(ptr: i32, len: i32): i64 {
  return respondAction(`{"v":1,"speak":"Opening Spotify.","launch_app":"Spotify"}`);
}
```

Get the JSON exactly right — a malformed or wrong-versioned envelope is
rejected outright and the user gets "I couldn't understand that action". The
schema is in [reference-actions.md](reference-actions.md).

## Everything else

The manifest, capabilities, publishing flow and validation are identical for
every skill regardless of language:

- [reference-manifest.md](reference-manifest.md)
- [reference-capabilities.md](reference-capabilities.md) — ignore any entry
  whose SDK feature has no AssemblyScript equivalent
- [publishing.md](publishing.md)
- [reference-sdk.md](reference-sdk.md#the-abi) — the raw ABI, if you're
  porting a third language
