# Phase 17: Fork CLI - Context

**Gathered:** 2026-04-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement `3gs fork` CLI subcommand that scaffolds a ready-to-run 3GS node. The command generates a fresh PKARR keypair, creates skeleton data files, writes a .env, and prints instructions. The arg check happens before Config::load() so no env vars are needed. This is the onboarding mechanism for the federation demand test.

</domain>

<decisions>
## Implementation Decisions

### CLI Architecture (DIST-02)
- **D-01:** Fork module lives in `src/fork.rs` — separate from server logic
- **D-02:** `main.rs` checks `std::env::args()` for "fork" as first positional arg BEFORE calling `Config::load()`
- **D-03:** If "fork" detected: call `fork::run(args)` and `std::process::exit(0)` — never enters tokio runtime
- **D-04:** No clap dependency — simple manual arg parsing with `std::env::args().collect::<Vec<_>>()`
- **D-05:** Flags: `--endorse <pubkey>` (required), `--url <peer_url>` (required), `--name <curator_name>` (optional, defaults to "New Curator"), `--output <dir>` (optional, defaults to `./3gs-fork-{pubkey_prefix}`)

### Keypair Generation (DIST-01)
- **D-06:** Generate keypair via `pkarr::Keypair::random()`
- **D-07:** Extract secret key: `hex::encode(keypair.secret_key())` for PKARR_SECRET_KEY
- **D-08:** Extract public key: `keypair.public_key().to_z32()` for display and registry curator pubkey

### Generated Files (DIST-01)
- **D-09:** Output directory: `./3gs-fork-{first_8_chars_of_pubkey_z32}/`
- **D-10:** If output dir exists: error with message, do not overwrite
- **D-11:** `registry.json` — skeleton with curator identity and parent endorsement:
  ```json
  {
    "version": "0.1.0",
    "updated": "{today ISO date}",
    "curator": { "name": "{curator_name}", "pubkey": "{pubkey_z32}" },
    "endorsements": [{ "pubkey": "{parent_pubkey}", "url": "{parent_url}", "name": null, "since": "{today}" }],
    "categories": {}
  }
  ```
- **D-12:** `identities.json` — empty object `{}`
- **D-13:** `contributions.json` — `{ "proposals": {} }` (matches current contributions loader format, NOT raw HashMap — check actual format)
- **D-14:** `audit_log.json` — empty array `[]`
- **D-15:** `.env` — all required env vars:
  ```
  REGISTRY_PATH=registry.json
  AUDIT_LOG_PATH=audit_log.json
  IDENTITIES_PATH=identities.json
  CONTRIBUTIONS_PATH=contributions.json
  PKARR_SECRET_KEY={hex_secret_key}
  PORT=3000
  ```

### Output (DIST-01)
- **D-16:** Print to stdout after successful scaffold:
  - Your PKARR public key (z32)
  - Your PKARR secret key (hex) — with WARNING to keep it safe
  - How to run: `cd {output_dir} && cargo run` (from repo root) or Docker one-liner
  - How to add categories: brief instructions
- **D-17:** Exit code 0 on success, 1 on error

### Error Handling
- **D-18:** Missing required flags (--endorse, --url): print usage and exit 1
- **D-19:** Output directory already exists: error message and exit 1
- **D-20:** File write failures: error message with path and exit 1

### Claude's Discretion
- Exact usage/help text formatting
- Whether to validate --endorse as a valid z32 key format
- Whether to include a README.md in the scaffolded directory

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Keypair Generation
- `src/pubky/identity.rs` — generate_or_load_keypair() for reference. Fork uses Keypair::random() directly.

### Current main.rs Structure
- `src/main.rs` — Fork arg check goes at the very top, before Config::load() at line 39

### Registry Format
- `registry.json` — Current format with endorsements array (populated by Phase 15)
- `src/registry/types.rs` — Registry, Endorsement structs that the skeleton must match

### Config Requirements
- `src/config.rs` — Config struct shows required env vars: REGISTRY_PATH, AUDIT_LOG_PATH, IDENTITIES_PATH, CONTRIBUTIONS_PATH

### Eng Review Plan
- `~/.claude/plans/purring-marinating-taco.md` — Step 5 covers fork CLI in detail

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `pkarr::Keypair::random()` — generates new keypair
- `pkarr::Keypair::secret_key()` → `[u8; 32]` — for hex encoding
- `pkarr::Keypair::public_key().to_z32()` → String — for display
- `hex::encode()` — already a dependency

### Established Patterns
- Module declaration in main.rs (10 existing mods + federation from Phase 16)
- Config::load() at line 39 — fork check must go before this

### Integration Points
- `src/main.rs` top of `fn main()` — insert fork arg check before `Config::load()`
- `src/fork.rs` — new file, `pub fn run(args: Vec<String>) -> Result<(), ...>`
- `src/lib.rs` — add `pub mod fork;`

</code_context>

<specifics>
## Specific Ideas

No specific requirements — standard CLI scaffolding. The key insight is that `fork` must work without any environment setup, which is why arg parsing goes before Config::load().

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 17-fork-cli*
*Context gathered: 2026-04-03*
