# Skrynia scheduled-work itinerary

## Scope

This is the **sole entry point** for recurring scheduled ChatGPT tasks that maintain the Skrynia repository.

Target repository:

<https://github.com/ottojung/Skrynia>

Use Lubko as the execution/orchestration platform. For normal development work, operate through the Lubko server `phoebe-dev`.

Before acting, study and obey:

- <https://github.com/ottojung/lubko/blob/main/docs/SKILL.md>
- <https://github.com/ottojung/lubko/blob/main/docs/skills/scheduled.md>
- <https://github.com/ottojung/Skrynia/blob/main/AGENTS.md>

`docs/skills/scheduled.md` in Lubko owns the reusable scheduled-orchestrator mechanics. This itinerary contains only Skrynia-specific work-selection, integration, verification, and completion policy; do not duplicate the shared mechanics here.

## Work selection

Apply Lubko's `docs/skills/scheduled.md`, with these Skrynia-specific choices:

- Prefer inheriting abandoned issue-tracked work over selecting a new issue.
- If there is no abandoned work to inherit, choose an actionable open Skrynia issue that is neither actively owned nor already completed under the shared scheduled-work protocol.
- Prefer the lowest-numbered actionable issue when several are otherwise equally suitable. This makes selection deterministic and reduces unnecessary races.
- Do not invent speculative feature work merely to keep the schedule busy. If there is no actionable issue, end that scheduled invocation without changing the repository.
- Once an issue is selected, drive it through the completion condition below rather than merely investigating it.

## Release integration

Scheduled Skrynia work accumulates in one current active `release/*` branch. A human promotes that release branch into `main`.

- The active release branch is the latest `release/*` branch that has **never** been promoted into `main`.
- A release branch is permanently retired after its first promotion into `main`, even if commits are accidentally added to it later.
- If no active release branch exists, create one from current `main`.
- Reuse the same active release branch across scheduled issues; do not create one release branch per issue.
- Before starting issue work, reconcile current `main` into the active release branch using a pull request when the release branch is behind.
- Start each issue branch from the active release branch in an isolated Lubko worktree.
- Open the issue PR against the active release branch, not `main`.
- Push issue branches early and keep the PR usable as the orchestrator's review surface.
- After implementation, repository verification, and orchestrator-owned GitHub review, merge the issue PR into the active release branch.
- After that merge, reconcile the latest `main` into the active release branch if needed and verify the exact resulting release head.
- There must be at most one open promotion PR from the current active release branch to `main`.
- Scheduled orchestrators must **not** merge issue/task PRs into `main` and must **not** merge `release/*` into `main`. Promotion into `main` is the human review boundary.

## Verification

For every issue:

- Obey `AGENTS.md` and the relevant live intent records.
- Run `make test` on the exact candidate head.
- Require the relevant GitHub CI checks to pass on the exact pushed head.
- The ChatGPT orchestrator must review the GitHub PR diff itself; an agent's review or summary does not satisfy this requirement.
- Treat tests as evidence rather than proof and inspect the changed invariants directly before merge.

## Completion

A scheduled Skrynia issue is complete when:

- its reviewed work is merged into the current active release branch;
- the release branch is reconciled with current `main`;
- `make test` and required GitHub CI pass on the exact resulting release head;
- no unresolved review blocker remains;
- the canonical issue orchestrator-status comment is updated to `completed` with useful durable final handles.

After those conditions hold, complete the shared orchestrator workflow according to Lubko's `docs/skills/scheduled.md`.
