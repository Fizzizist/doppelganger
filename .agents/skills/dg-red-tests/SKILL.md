---
name: dg-red-tests
description: "Write failing tests for a plan from a doppelganger(dg) branch. Usage: dg-red-tests"
---

# dg-red-tests — Write Red Tests for a Plan

## Global Rules

1. **Doppelganger as shared memory** — Plans, reviews, assessments, and progress are written as branch/issue comments so any future session can pick up context.
2. **Safe git** — Never use `git add -A` or `git add .`. Stage files by name. Never stage secrets.
3. **Task tracking** — Use the `update_task` tool to show progress.

## Specific Rules

- **Tests Only** - No production code is allowed to change. Only new and existing test files are allowed to be changed. All new and modified tests must fail.

## Step 1: Read the plan and comments

Read dg branch plan and comments to get complete context:

```bash
dg branch read
```

The plan is marked with `<!-- dg-plan -->` and is the body of the dg branch.

Also read any progress updates, and discussion — all of this context informs the tests.

## Step 2: Set up task tracking

Create tasks based on the plan's abstraction layout. Example:
- title: "read", description: "Read plan and codebase" <-- start here
- title: "frontend layer tests", description: "Write frontend layer tests"
- title: "service layer tests", description: "Write service layer tests"
- title: "verify", description: "Run test suite and verify failures"

## Step 3: Read the codebase

Read all files relevant to the plan's abstractions. Understand the existing code before writing any tests.

## Step 4: Write Red Tests

Use the `implement` role subagent to write tests for each abstraction layer listed in the plan. Use `default` model role if `implement` is not defined.

**For each abstraction layer in the plan**, launch a parallel `implement` subagent via the `Agent` tool. Provide each agent with:
- The specific abstraction layer and new behavioral changes
- The full plan context and any relevant PR discussion
- The list of files to read for understanding existing patterns
- Instructions to run tests

**Small plans (1-2 abstraction layers):** You may write tests directly instead of delegating, if the changes are straightforward enough that subagent overhead isn't justified.

Update task tracking as each deliverable completes.

## Step 5: Verify

After all `implement` agents complete:
- Run the project's test suite
- Run any linting/formatting tools
- Verify each new behavior is covered and has a failing test
- If any agent reported issues or partial completion, address the gaps

Suggest next step: `/dg-commit`
