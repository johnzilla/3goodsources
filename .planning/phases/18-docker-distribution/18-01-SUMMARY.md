---
plan: 18-01
phase: 18-docker-distribution
status: complete
started: 2026-04-03
completed: 2026-04-03
tasks_completed: 2
tasks_total: 2
---

# Plan 18-01: Docker Distribution — Summary

## What Shipped

Replaced the GitHub Actions multi-platform workflow with a local `scripts/docker-publish.sh` script that builds natively on amd64 and pushes to GHCR.

## Deviation from Plan

**Original plan:** GitHub Actions workflow with QEMU cross-compilation for amd64 + arm64.
**What happened:** GitHub Actions build was extremely slow for Rust (cargo compiles from scratch on every push, QEMU arm64 emulation compounds it). User opted for amd64-only local build script instead.
**Impact:** Faster publish cycle, simpler infrastructure. arm64 support deferred — most curators will run x86.

## Key Files

### Created
- `scripts/docker-publish.sh` — Local build-and-push script for GHCR

### Deleted
- `.github/workflows/docker.yml` — Removed slow CI workflow

## Commits

| Hash | Message |
|------|---------|
| 9613db9 | feat(18-01): add GitHub Actions Docker workflow for GHCR |
| 9812298 | feat(18-01): replace GHA workflow with local docker-publish script |

## Verification

- Script exists and is executable
- Builds use existing Dockerfile (unchanged)
- Repeatable: run script any time to publish
- Human verification deferred to when user is ready to push first image
