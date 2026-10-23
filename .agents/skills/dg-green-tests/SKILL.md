---
name: dg-green-tests
description: "Implement code changes to make failing tests pass from a doppelganger(dg) branch"
---

# dg-implement — Implement Plans or Fix Findings

## Global Rules

1. **Doppelganger as shared memory** — Plans, reviews, assessments, and progress are written as branch/issue comments so any future session can pick up context.
2. **Safe git** — Never use `git add -A` or `git add .`. Stage files by name. Never stage secrets.
3. **Task tracking** — Use the `update_task` tool to show progress.

## Specific Rules

1. **No new test code** - Only implement code to make existing tests pass. Do not write _any_ new tests or modify existing tests. If you believe it to be impossible to make a test pass then you shall halt and inform the user of your findings.

## Step 1: Read the dg branch

Read dg branch plan and comments to get complete context:

```bash
dg branch read
```

Also read any progress updates, and discussion — all of this context informs implementation.

## Step 2: Set up task tracking

Create tasks based on the failing tests for each abstraction layer in the plan. Example:
- title: "read", description: "Read dg branch and codebase" <-- start here
- title: "frontend layer", description: "Implement frontend changes"
- title: "service layer", description: "Implement service layer changes"
- title: "verify", description: "Test and verify"

## Step 3: Read the codebase

Read all relevant files and understand the existing code before making changes.

## Step 4: Implement

Use the `implement` role subagent to implement each necessary changes for each abstraction layer. Use `default` model role if `implement` is not defined.

**For each layer in the plan**, launch an `implement` subagent in parallel via the `Agent` tool. Provide each agent with:
- The specific behavioral changes to implement (files to modify, what to change, expected behavior)
- The full plan context and any relevant discussion
- The list of files to read for understanding existing patterns
- Instructions to run tests and linting after making changes

**Small plans (1-2 simple abstractions):** You may implement directly instead of delegating, if the changes are straightforward enough that subagent overhead isn't justified.

Update task tracking as each deliverable completes.

## Step 5: Verify

After all `implement` agents complete:
- Run the project's test suite
- Run any linting/formatting tools
- Verify all tests are passing and that no tests were modified in the process
- If any agent reported issues or partial completion, address the gaps

Suggest next step: `/dg-commit`
