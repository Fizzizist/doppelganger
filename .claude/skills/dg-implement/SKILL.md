---
name: dg-implement
description: "Implement a plan from a doppelganger(dg) branch, or fix review findings. Usage: dg-implement [finding-numbers]"
argument-hint: "[finding-numbers]"
---

# dg-implement — Implement Plans or Fix Findings

**User input:** $ARGUMENTS

This skill has two modes:
- **Implement mode** (no arguments given): reads the plan and implements it
- **Fix mode** (with finding numbers): fixes specific review findings

## Global Rules

1. **Doppelganger as shared memory** — Plans, reviews, assessments, and progress are written as branch/issue comments so any future session can pick up context.
2. **Safe git** — Never use `git add -A` or `git add .`. Stage files by name. Never stage secrets.
3. **Task tracking** — Use the `update_task` tool to show progress.

## Step 1: Parse input

Extract:
- **Finding numbers** to fix (optional — e.g., `1,2,3`)

If no arguments were given → **implement mode**.
If finding numbers → **fix mode**.

---

## Implement Mode

### Step 3i: Read the plan and comments

Read dg branch plan and comments to get complete context:
```bash
dg branch read
```

The plan is marked with `<!-- dg-plan -->` and is the body of the dg branch.

Also read any progress updates, prior review findings, assessments, inline review comments, and discussion — all of this context informs implementation.

### Step 4i: Set up task tracking

Create tasks based on the plan's deliverables/features. Example:
- title: "read", description: "Read plan and codebase" <-- start here
- title: "feature-1", description: "Implement feature 1"
- title: "feature-2", description: "Implement feature 2"
- title: "test", description: "Add/update tests"
- title: "verify", description: "Build and verify"

### Step 5i: Read the codebase

Read all files mentioned in the plan. Understand the existing code before making changes.

### Step 6i: Implement

Use the `implement` role subagent to implement each deliverable. Use `default` model role if `implement` is not defined.

**For each deliverable in the plan**, launch a `implement` subagent via the `Agent` tool. Provide each agent with:
- The specific deliverable to implement (files to modify, what to change, expected behavior)
- The full plan context and any relevant PR discussion
- The list of files to read for understanding existing patterns
- Instructions to run tests and linting after making changes
- **If the plan includes tests for this deliverable, tests MUST be written as part of the implementation — not deferred**

**Test coverage is part of the deliverable, not an afterthought.** If the plan specifies tests for a deliverable, the feature-dev agent must implement them. If the target package lacks test infrastructure, add it.

**Parallelism:** If deliverables are independent (don't modify the same files), run their `implement` agents in parallel.

**Small plans (1-2 simple deliverables):** You may implement directly instead of delegating, if the changes are straightforward enough that subagent overhead isn't justified.

Update task tracking as each deliverable completes.

### Step 7i: Verify

After all `implement` agents complete:
- Run the project's test suite
- Run any linting/formatting tools
- Build the project if applicable
- Verify each deliverable from the plan is addressed
- If any agent reported issues or partial completion, address the gaps

Suggest next step: `/dg-commit` then `/dg-review` for review.

---

## Fix Mode (finding numbers or `ci`)

### Step 3f: Set up task tracking

- title: "gather", description: "Gather findings to fix" <-- start here
- title: "fix", description: "Implement fixes"
- title: "verify", description: "Verify fixes"

### Step 4f: Gather context

#### If finding numbers were provided:

Read ALL dg branch comments to get full context:
```bash
dg branch read
```

Find the review (`<!-- dg-review -->`) and assessment (`<!-- dg-assessment -->`) comments, then extract the specific findings to fix. Also review inline review comments for any additional context or feedback. Prior progress comments and discussion may also provide useful context.

#### If no finding numbers:

Read ALL dg branch comments AND inline review comments, find review/assessment comments, present genuine findings, and ask which to fix. Use both commands above.

### Step 5f: Batch sizing

- **Simple fixes** (typos, naming, imports): ~10 per batch
- **Moderate fixes** (logic changes, refactors): ~6 per batch
- **Complex fixes** (architecture, new features): ~3 per batch

If more than batch size, fix first batch and tell user to re-run.

### Step 6f: Implement fixes

Use the `implement` subagent to implement fixes. `implement` is a model role. If it isn't specified, then just use `default`.

**For each finding** (or batch of related findings), launch a `implement` subagent with:
- The finding description and the assessment's classification/reasoning
- The specific files and code locations involved
- Instructions on what to fix and how
- Instructions to run tests after fixing

**Parallelism:** If findings touch different files, run their `implement` agents in parallel. If findings overlap (same file/function), batch them into a single agent.

**Simple fixes** (typos, naming, one-line changes): You may fix these directly instead of delegating.

Defer out-of-scope items to new dg issues. Update task tracking per finding.

### Step 7f: Verify

After all `implement` agents complete:
- Run tests and linting
- Verify each fix addresses its finding
- If any agent reported issues, address the gaps

Suggest next step: `/dg-commit` then `/dg-review` for re-review.
