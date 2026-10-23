---
name: dg-test-plan
description: "Explore codebase, create abstraction test plan, create feature branch, create doppelganger(dg) branch, post plan as dg branch comment. Everything lives on the dg branch from this point forward. Usage: dg-test-plan 42"
argument-hint: "<issue-number>"
---

# dg-test-plan - Plan, Branch, and Comment on Doppelganger Branch

**User input:** $ARGUMENTS

This command is strictly for **planning**. Do NOT write any code - no file edits, no file writes.

## Global Rules

1. **Doppelganger as shared memory** — Plans, reviews, assessments, and progress are written as branch/issue comments so any future session can pick up context.
2. **HTML markers** — Use `<!-- dg-plan -->` as the first line of plan comment bodies for reliable discovery.
3. **Safe git** — Never use `git add -A` or `git add .`. Stage files by name. Never stage secrets.
4. **Task tracking** — Use the `update_task` tool to show progress through multi-step commands.
5. **Project conventions** — Check for CLAUDE.md, AGENTS.md, and CONTRIBUTING.md before planning.

## Step 1: Set up task tracking

- title: "read", description: "Read issue and context" <-- start here
- title: "explore", description: "Explore codebase"
- title: "test plan", description: "Draft test plan"
- title: "branch", description: "Create branch and write dg branch"

## Step 2: Read the issue

```bash
dg issue read <number>
```

Parse everything: problem statement, constraints, requirements, acceptance criteria, prior discussion, any existing assessment comments (look for `<!-- dg-assessment -->`).

Update task: read → completed, explore → in_progress.

## Step 3: Read project conventions

Check for and read (first found):
- CONTRIBUTING.md, DEVELOPMENT.md, .github/CONTRIBUTING.md
- CLAUDE.md, AGENTS.md

Extract relevant guidance: abstraction layers, testing expectations, coding conventions.

## Step 4: Explore the codebase

Launch 2-3 Explore subagents in parallel. Use `implement` role or `default` if not defined.
- **Similar features**: Find existing code that solves related problems, trace implementation patterns
- **Architecture**: Map out the project's abstraction layers.
- **Integration points**: Identify where new code connects to existing systems

Include project conventions in each agent's context. Each agent returns 5-10 key files. Read all identified files.

Update task: explore → completed, plan → in_progress.

## Step 5: Draft the plan

Draft a test plan for each abstraction layer of the program:

- For each layer, identify the behavioral changes needed to achieve the acceptance criteria in the issue.
- Your plan should list a set of test cases per abstraction layer with the specific behavioral changes to be tested. 

Present the plan to the user. Discuss and revise if they have feedback.

Update task: plan → completed, branch → in_progress.

## Step 6: Create branch and draft PR

```bash
# Derive branch name from issue
# Format: <issue-type>/issue-<N>-<slug> (slug = 3-5 words from title, lowercase, hyphens) (issue-type = feature, bug, etc.)
git checkout -b <issue-type>/issue-<N>-<slug>

cat > /tmp/dg-plan.md << 'DG_EOF'
<!-- dg-plan -->
## Implementation Plan

<full plan content>

---
*Plan created by doppelganger*
DG_EOF
dg branch create <issue-number> < /tmp/dg-plan.md
```

Update task: branch → completed.

Suggest next step: implement the plan with `/dg-red-tests` when ready.
