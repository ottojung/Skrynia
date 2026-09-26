# Skrynia scheduled-work itinerary

## Scope

This is the project-specific execution, integration, verification, and completion policy for recurring work on Skrynia.

Target repository:

<https://github.com/ottojung/Skrynia>

Before acting, study and obey:

- Antonina's canonical board-orchestrator skill: <https://github.com/ottojung/antonina/blob/main/docs/skills/orchestrator.md>
- Lubko's execution skill: <https://github.com/ottojung/lubko/blob/main/docs/SKILL.md>
- <https://github.com/ottojung/Skrynia/blob/main/AGENTS.md>

The Antonina board owns queue selection, claims, recovery, progress, blockers, and handoff. This itinerary owns only Skrynia-specific repository policy. GitHub issue order and mutable GitHub status comments are not orchestration state.

For normal development work, operate through the Lubko server `phoebe-dev`.

**Never stop or disable the recurring task merely because one work item is blocked.** Record the blocker on the Antonina board and let the generic orchestrator choose another actionable issue.

## Work selection

Use Antonina's canonical board-selection algorithm. Continue recoverable ongoing Skrynia work before starting duplicate work. Otherwise select the highest-priority actionable Skrynia issue represented on the board.

When a board issue mirrors a GitHub issue, the GitHub issue is specification/context and the Antonina board thread is the coordination history.

Do not invent speculative feature work merely to keep the schedule busy. If there is no actionable Skrynia board issue, end that invocation without changing the repository.

## Release integration

Scheduled Skrynia work accumulates in one current active `release/*` branch. A human promotes that release branch into `main`.

- The active release branch is the latest `release/*` branch that has **never** been promoted into `main`.
- A release branch is permanently retired after its first promotion into `main`, even if commits are accidentally added to it later.
- If no active release branch exists, create one from current `main`.
- Reuse the same active release branch across scheduled issues; do not create one release branch per issue.
- Before starting issue work, merge current `main` into the active release branch via a pull request and verify the resulting release head.
- Start each issue branch from the active release branch in an isolated Lubko worktree.
- Open the issue PR against the active release branch, **not `main`**.
- Push issue branches early and keep the PR usable as the review surface.
- After implementation, exact-head verification, and orchestrator-owned GitHub review, merge the issue PR into the active release branch.
- After that merge, reconcile the latest `main` into the active release branch if needed and verify the exact resulting release head.
- If work is accidentally added to a retired release branch, preserve any unique work on the active release branch, then stop using the retired branch.
- There must be at most one open release-promotion PR targeting `main`, and it must come from the current active release branch.

Scheduled orchestrators must **not** merge issue/task PRs into `main` and must **not** promote `release/*` into `main`. Promotion is the human review boundary.

## Verification

For every scheduled issue:

- obey `AGENTS.md` and relevant live intent records;
- run `make test` on the exact candidate head;
- require relevant GitHub CI checks to pass on the exact pushed head;
- independently review the GitHub PR diff;
- treat tests as evidence rather than proof and inspect changed invariants directly.

## Completion

A Skrynia board issue is complete when:

- its reviewed work is merged into the current active release branch;
- the release branch is reconciled with current `main`;
- `make test` and required GitHub CI pass on the exact resulting release head;
- no unresolved review blocker remains.

Then append the completed board comment, close the Antonina board issue, and verify that it has left the queue according to the canonical orchestrator skill.
