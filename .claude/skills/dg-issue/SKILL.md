---
name: dg-issue
description: "Assess an existing doppelganger issue (explore codebase, identify scope/risks/ambiguities, post assessment) or create a new structured issue. Usage: dg-issue 42 (assess) or dg-issue (create) or dg-issue <description> (create with context)"
argument-hint: "[issue-number | description]"
---

# dg-issue — Assess or Create Issue

**User input:** $ARGUMENTS

## Global Rules

1. **doppelganger as shared memory** — Plans, reviews, assessments, and progress are written as branch/issue comments using `dg` so any future session can pick up context.
2. **HTML markers** — Use `<!-- dg-assessment -->`, `<!-- dg-plan -->`, `<!-- dg-review -->`, `<!-- dg-progress -->` as the first line of comment bodies for reliable discovery.
3. **Safe git** — Never use `git add -A` or `git add .`. Stage files by name. Never stage secrets (.env, credentials, tokens, keys).
4. **Task tracking** — Use the `update_task` tool to show progress through multi-step commands.
5. **Project conventions** — Check for CLAUDE.md, AGENTS.md, .dreb/CONTEXT.md, and CONTRIBUTING.md before planning or implementing.

## Determine Mode

If the input is a number, run **ASSESS** mode. Otherwise, run **CREATE** mode.

---

## ASSESS Mode

**Assess an existing doppelganger issue — explore the codebase, identify scope/risks/ambiguities, post assessment.**

### Step 1: Set up task tracking

- title: "read", description: "read issue and comments" <-- start on this one
- title: "explore", description: "Explore relevant codebase"
- title: "assess", description: "Analyze and assess"
- title: "post", description: "Post assessment"

### Step 2: Read the issue

```sh
dg issue read <number>
```

Parse: problem statement, constraints, requirements, acceptance criteria, existing comments.

Update task: read → completed, explore → in_progress.

### Step 3: Explore the codebase

Launch 2 Explore subagents in parallel targeting different aspects. Use the `implement` role for these agents. If there isn't an implement role configured, then they can just use default.
- **Relevant code**: Find existing code related to the issue, trace implementation patterns
- **Architecture**: Map relevant architecture layers, abstractions, data flow

Each agent should return 5-10 key files. After agents complete, read all identified files.

Update task: explore → completed, assess → in_progress.

### Step 4: Assess

Present to the user:
1. **Summary**: The issue in your own words
2. **Current state**: What exists today that's relevant
3. **Gaps**: What's missing, broken, or unclear
4. **Ambiguities**: Underspecified aspects or open questions
5. **Scope**: Size and complexity estimate
6. **Risks**: Pitfalls, edge cases, architectural concerns

### Step 5: Post assessment

Post as an issue comment:

```sh
cat > /tmp/dg-comment.md << 'DG_EOF'
<!-- dg-assessment -->
## Issue Assessment

<assessment content>

---
*Automated assessment by doppelganger*
DG_EOF
dg issue comment <number> < /tmp/dg-comment.md
```

Update task: post → completed.

Suggest next step: `/skill:dg-plan <number>`

---

## CREATE Mode

**Create a new structured GitHub issue from context or description.**

### Step 1: Gather context

If a description was provided, use it as the starting point. Otherwise, ask the user what they want to create an issue for.

Explore the codebase if needed to understand the relevant area.

### Step 2: Draft the issue

Create a structured issue with:
- **Title**: Clear, concise, action-oriented (under 80 chars, imperative form)
- **Summary**: 2-3 sentences describing the problem or feature
- **Current Behavior** (for bugs/improvements): What happens now
- **Proposed Behavior**: What should happen
- **Acceptance Criteria**: Bullet list of verifiable conditions that define "done"
- **Context**: Links to related PRs, issues, or discussions
- **Technical Notes**: Implementation hints, relevant files, architectural considerations
- **Labels**: Suggest appropriate labels based on the issue type

Present the draft to the user for approval.

### Step 3: Create the issue

```sh
cat > /tmp/dg-body.md << 'DG_EOF'
<body>
DG_EOF
dg issue create --name "<title>" < /tmp/dg-body.md
```

Report the issue number and URL. Suggest next step: `/dg-plan <number>`
