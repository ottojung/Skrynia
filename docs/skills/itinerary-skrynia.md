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

**Never stop or disable this scheduled task.** Treat errors and blockers in an individual work item as recoverable. Preserve useful state, recover or choose other actionable work as appropriate, and continue producing useful progress rather than terminating the recurring orchestrator.

## Work selection

Apply Lubko's `docs/skills/scheduled.md`, with these Skrynia-specific choices:

- Prefer inheriting abandoned issue-tracked work over selecting a new issue.
- If there is no abandoned work to inherit, choose an actionable open Skrynia issue that is neither actively owned nor already completed under the shared scheduled-work protocol.
- Prefer the lowest-numbered actionable issue when several are otherwise equally suitable. This makes selection deterministic and reduces unnecessary races.
- Do not invent speculative feature work merely to keep the schedule busy. If there is no actionable issue, end that scheduled invocation without changing the repository.
- Once an issue is selected, drive it through the completion condition below rather than merely investigating it.
