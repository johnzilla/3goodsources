---
phase: 18-docker-distribution
verified: 2026-04-02T00:00:00Z
status: gaps_found
score: 0/3 success criteria verified
gaps:
  - truth: "A multi-platform Docker image (linux/amd64 and linux/arm64) is published to GHCR"
    status: failed
    reason: "arm64 support was deferred; script builds amd64 only. Image has not been pushed yet."
    artifacts:
      - path: "scripts/docker-publish.sh"
        issue: "Builds linux/amd64 only — no QEMU, no buildx --platform flag, no arm64 target"
    missing:
      - "Add buildx multi-platform support to docker-publish.sh (or accept amd64-only and update DIST-03 scope)"
      - "Actually push the image to GHCR so it exists"

  - truth: "docker pull ghcr.io/johnzilla/3goodsources:latest succeeds and the image runs the server correctly"
    status: failed
    reason: "No image has been pushed to GHCR yet. The script is a mechanism but has not been executed."
    artifacts: []
    missing:
      - "Run scripts/docker-publish.sh to push the first image"
      - "Verify pull and smoke test succeed"

  - truth: "The GHCR publish is repeatable via CI or documented manual workflow (not a one-off manual push)"
    status: partial
    reason: "scripts/docker-publish.sh is a documented, repeatable local workflow — the script structure satisfies this criterion. However it is blocked by the two gaps above: no image exists, and arm64 is absent."
    artifacts:
      - path: "scripts/docker-publish.sh"
        issue: "Script is substantive and correctly wired to Dockerfile, but image has not been published yet"
    missing:
      - "Execute the script at least once to establish the initial GHCR publish"

human_verification:
  - test: "Push image and verify pull"
    expected: "docker pull ghcr.io/johnzilla/3goodsources:latest completes; docker run starts the server and responds on port 3000"
    why_human: "Requires GITHUB_TOKEN with write:packages scope, Docker daemon, and outbound network access — cannot verify programmatically"

  - test: "Confirm GHCR package visibility"
    expected: "Package is set to public so any curator can pull without authentication"
    why_human: "Requires GitHub UI: Settings > Package settings > Change visibility > Public"

  - test: "arm64 decision: expand or scope-reduce"
    expected: "Either (a) script is updated to build arm64 via buildx and DIST-03 is satisfied in full, or (b) DIST-03 requirement text and phase goal are updated to amd64-only"
    why_human: "This is a product scope decision — whether to invest in arm64 build time or formally defer it"
---

# Phase 18: Docker Distribution Verification Report

**Phase Goal:** Any curator can pull and run a 3GS node on any machine (Intel or ARM) using a single Docker command from GHCR
**Verified:** 2026-04-02
**Status:** gaps_found
**Re-verification:** No — initial verification

## Context: Plan Deviation

The original plan called for a GitHub Actions workflow (`.github/workflows/docker.yml`) with QEMU cross-compilation for `linux/amd64` and `linux/arm64`. After execution, this was replaced with a local script (`scripts/docker-publish.sh`) because GitHub Actions Rust builds were too slow. The workflow file was deleted. arm64 support was deferred.

This verification uses the ROADMAP success criteria as the authoritative contract, not the stale PLAN frontmatter must_haves (which reference the deleted workflow file).

---

## Goal Achievement

### Success Criteria (from ROADMAP.md)

| # | Truth | Status | Evidence |
|---|-------|--------|---------|
| 1 | Multi-platform image (amd64 + arm64) published to GHCR | FAILED | Script builds amd64 only; no image pushed |
| 2 | `docker pull ghcr.io/johnzilla/3goodsources:latest` succeeds and server runs | FAILED | No image exists in GHCR yet |
| 3 | GHCR publish is repeatable via CI or documented manual workflow | PARTIAL | Script exists and is documented; blocked pending first push |

**Score:** 0/3 success criteria verified

---

## Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `scripts/docker-publish.sh` | Local build-and-push script | WIRED — but incomplete | Exists, executable, substantive, correctly references Dockerfile and ghcr.io/johnzilla/3goodsources |
| `.github/workflows/docker.yml` | Original CI workflow | DELETED (by design) | Replaced by local script per plan deviation |
| GHCR package `ghcr.io/johnzilla/3goodsources` | Published image | MISSING | No push has been executed yet |

---

## Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `scripts/docker-publish.sh` | `Dockerfile` | `docker build -t ... .` | WIRED | Line 22: `docker build -t "$IMAGE:latest" -t "$IMAGE:sha-$SHA" .` — builds from repo root |
| `scripts/docker-publish.sh` | `ghcr.io/johnzilla/3goodsources` | `docker push` | WIRED (code only) | Lines 31-33 push `:latest`, `:sha-$SHA`, and optional tag — correct image name |
| `scripts/docker-publish.sh` | `linux/arm64` | buildx `--platform` | NOT_WIRED | No `--platform` flag, no `docker buildx` invocation — amd64 host only |

---

## Data-Flow Trace (Level 4)

Not applicable — this phase produces a Docker image and shell script, not a component that renders dynamic data.

---

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Script is executable | `ls -la scripts/docker-publish.sh` | `-rwxr-xr-x` | PASS |
| Script references correct image name | `grep IMAGE scripts/docker-publish.sh` | `IMAGE="ghcr.io/johnzilla/3goodsources"` | PASS |
| Script wires to Dockerfile (build context `.`) | `grep "docker build" scripts/docker-publish.sh` | `docker build -t "$IMAGE:latest" -t "$IMAGE:sha-$SHA" .` | PASS |
| Script pushes latest and SHA tags | `grep "docker push" scripts/docker-publish.sh` | pushes `:latest` and `:sha-$SHA` | PASS |
| arm64 platform flag present | `grep "platform\|buildx\|arm64" scripts/docker-publish.sh` | no match | FAIL |
| Image exists in GHCR | network check | no push executed | FAIL |

---

## Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|---------|
| DIST-03 | 18-01-PLAN.md | Docker image published to GHCR (multi-platform linux/amd64 + linux/arm64) | BLOCKED | Script exists but arm64 absent; image not yet pushed |

**Note on DIST-03 wording:** REQUIREMENTS.md records DIST-03 as "multi-platform linux/amd64 + linux/arm64". The current implementation is amd64-only. This is a scope mismatch that requires either (a) completing arm64 support, or (b) formally narrowing DIST-03 to amd64-only and updating the requirement text.

---

## Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `scripts/docker-publish.sh` | 15-19 | Login check uses `manifest inspect` as a proxy — silently skips auth reminder if image already exists | Info | Cosmetic — does not block publish |

No TODO/FIXME/placeholder comments found in `scripts/docker-publish.sh`. No stub implementations. Script logic is complete for amd64.

---

## Human Verification Required

### 1. Push Image to GHCR

**Test:** Authenticate to GHCR with a token holding `write:packages` scope, then run `./scripts/docker-publish.sh` from the repo root.

**Expected:** Script completes without error; `docker push` lines succeed; GHCR package page at `https://github.com/johnzilla/3goodsources/pkgs/container/3goodsources` shows the image with `:latest` and `:sha-<hash>` tags.

**Why human:** Requires live Docker daemon, network access, and a GITHUB_TOKEN with packages:write — cannot test programmatically.

### 2. Verify Pull and Smoke Test

**Test:** After push, run `docker pull ghcr.io/johnzilla/3goodsources:latest` then `docker run --rm -p 3000:3000 ghcr.io/johnzilla/3goodsources:latest`.

**Expected:** Pull succeeds without authentication (public package). Container starts and the server listens on port 3000.

**Why human:** Requires running Docker container with network access; external service state.

### 3. Set Package Visibility to Public

**Test:** In GitHub UI, navigate to Settings > Package settings for `3goodsources` container, change visibility to Public.

**Expected:** `docker pull ghcr.io/johnzilla/3goodsources:latest` succeeds for an unauthenticated user on any machine.

**Why human:** Requires GitHub UI interaction with repository admin access.

### 4. arm64 Scope Decision

**Test:** Decide whether to add `docker buildx` multi-platform support to `scripts/docker-publish.sh`, or formally scope DIST-03 to amd64-only.

**Expected:** Either (a) script updated with `--platform linux/amd64,linux/arm64` and tested on a machine with buildx, or (b) DIST-03 requirement text and ROADMAP success criteria updated to reflect amd64-only scope.

**Why human:** Product scope decision — weighing build complexity vs. curator hardware coverage.

---

## Gaps Summary

Two hard gaps prevent goal achievement:

**Gap 1 — arm64 absent:** The phase goal and DIST-03 requirement both specify multi-platform (amd64 + arm64). The script builds amd64 only. Either the implementation must be extended (add `docker buildx --platform linux/amd64,linux/arm64`) or the requirement must be formally narrowed. This is not a silent deviation — the ROADMAP success criteria and REQUIREMENTS.md both explicitly list arm64.

**Gap 2 — Image not published:** The script is the mechanism, but the mechanism has not been run. No image exists at `ghcr.io/johnzilla/3goodsources`. Until the script is executed and the push succeeds, the phase goal ("any curator can pull...") is not satisfied. This gap resolves immediately once the human verification step is completed.

**What works:** `scripts/docker-publish.sh` is well-structured, correctly wired to the Dockerfile, targets the right GHCR namespace, and handles `:latest`, `:sha-<hash>`, and optional semver tags. The Dockerfile is production-quality (multi-stage, non-root user, minimal runtime image). The repeatability criterion (SC3) is met at the code level — the script can be run any time.

**Path to closure:** Run `scripts/docker-publish.sh` to push the initial image (closes Gap 2). Then either extend the script with buildx multi-platform support or update DIST-03 scope (closes Gap 1).

---

_Verified: 2026-04-02_
_Verifier: Claude (gsd-verifier)_
