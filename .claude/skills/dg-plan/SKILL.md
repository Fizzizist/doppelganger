---
name: dg-plan
description: "Explore codebase, create implementation plan, create feature branch, create doppelganger(dg) branch, post plan as dg branch comment. Everything lives on the dg branch from this point forward. Usage: dg-plan 42"
argument-hint: "<issue-number>"
---

# dg-plan — Plan, Branch, and Open PR

**User input:** $ARGUMENTS

This command is strictly for **planning**. Do NOT implement any code changes — no file edits, no file writes.

## Global Rules

1. **Doppelganger as shared memory** — Plans, reviews, assessments, and progress are written as branch/issue comments so any future session can pick up context.
2. **HTML markers** — Use `<!-- dg-plan -->` as the first line of plan comment bodies for reliable discovery.
3. **Safe git** — Never use `git add -A` or `git add .`. Stage files by name. Never stage secrets.
4. **Task tracking** — Use the `update_task` tool to show progress through multi-step commands.
5. **Project conventions** — Check for CLAUDE.md, AGENTS.md, .dreb/CONTEXT.md, and CONTRIBUTING.md before planning.

## Step 1: Set up task tracking

- title: "read", description: "Read issue and context" <-- start here
- title: "explore", description: "Explore codebase"
- title: "plan", description: "Draft implementation plan"
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
- CLAUDE.md, AGENTS.md, .dreb/CONTEXT.md

Extract planning-relevant guidance: project layers, testing expectations, coding conventions.

## Step 4: Explore the codebase

Launch 2-3 Explore subagents in parallel. Use `implement` role or `default` if not defined.
- **Similar features**: Find existing code that solves related problems, trace implementation patterns
- **Architecture**: Map relevant architecture layers, abstractions, data flow
- **Integration points**: Identify where new code connects to existing systems

Include project conventions in each agent's context. Each agent returns 5-10 key files. Read all identified files.

Update task: explore → completed, plan → in_progress.

## Step 5: Draft the plan

Create an implementation plan with:
- Clear analysis of the problem
- **Deliverables**: What will be produced (be specific)
- **Acceptance criteria**: How to verify the work is done
- **Files to create or modify**: List each with what changes
- **Testing approach**: What tests to write, what to verify
- **Risks and open questions**: Anything that might derail implementation

The plan should be **high-level on implementation details** (avoid cascading spec errors from over-specifying) but **specific on deliverables and acceptance criteria**.

**Project-layer coverage:** Cross-check the plan against discovered project layers. Every affected layer should be addressed.

**Test coverage is mandatory, not optional.** Every new behavior, command handler, formatting function, or event wiring must include tests in the plan. If the target package lacks test infrastructure, the plan must include setting it up as a deliverable — this cannot be deferred. The testing approach should specify:
- Which test files to create or modify
- What behaviors to verify (happy paths, error paths, edge cases)
- What test infrastructure/helpers are needed (mocks, factories, fixtures)

Present the plan to the user. Discuss and revise if they have feedback.

Update task: plan → completed, branch → in_progress.

## Step 6: Create branch and draft PR

```bash
# Derive branch name from issue
# Format: feature/issue-<N>-<slug> (slug = 3-5 words from title, lowercase, hyphens)
git checkout -b feature/issue-<N>-<slug>

cat > /tmp/dg-plan.md << 'DG_EOF'
<!-- dg-plan -->
## Implementation Plan

<full plan content>

---
*Plan created by doppelganger*
DG_EOF
dg branch create <issue-number> < /tmp/gh-comment.md
```

Update task: branch → completed.

Suggest next step: implement the plan with `/dg-implement` when ready.
