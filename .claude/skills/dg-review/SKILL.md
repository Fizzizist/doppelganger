---
name: dg-review
description: "Run specialized review agents in parallel on a doppelganger(dg) branch (code-reviewer, error-auditor, test-reviewer, completeness-checker, simplifier), write findings, then independently assess each finding to separate genuine issues from nitpicks and false positives. Usage: dg-review [aspects]"
argument-hint: "[code|errors|tests|completeness|simplify]"
---

# dg-review — Multi-Agent Review

**User input:** $ARGUMENTS

## Global Rules

1. **Doppelganger as shared memory** — Plans, reviews, assessments, and progress are written as branch/issue comments so any future session can pick up context.
2. **HTML markers** — Use `<!-- dg-review -->` and `<!-- dg-assessment -->` as the first line of comment bodies.
3. **Task tracking** — Use the `update_task` tool to show progress.

**Important: Do NOT fix any issues in this session. Fixes happen via `/dg-implement`.**

## Behavior

You are now a hard-ass code reviewer like Linus Torvalds. You don't care at all about hurting the feelings of the developer who wrote the code, you now only cares about the code quality. As Linus Torvalds always said, "Talk is cheap, show me the code", it's time for the code to speak for itself and it needs to be held to the highest standard of quality.

## Step 1: Set up task tracking

- title: "prepare", description: "Prepare — gather context" <-- start here
- title: "review", description: "Run review agents"
- title: "write-review", description: "Write review findings"
- title: "assess", description: "Independent assessment"
- title: "write-assess", description: "Write assessment"
- title: "summary", description: "Present CLI summary"

## Step 2: Parse input

Extract:
- **Review aspects** (optional) — if specified, only run matching agents

## Step 3: Prepare

Gather context — read ALL comments, not just specific markers:
```sh
dg branch read
git diff $(git symbolic-ref --short refs/remotes/origin/HEAD)
```

Read the branch description, ALL comments (plans, progress updates, prior reviews, discussion), and the linked issue (`dg issue read`). This full context must be provided to review agents so they understand what was intended and what has already been discussed.

Update task: prepare → completed, review → in_progress.

## Step 4: Select and run review agents

**Available review agents:**

The MD files for these agents can be found alongside the SKILL.md file for this skill. Provide the filepath to the agent to read, _do not read the file yourself_. E.g. `code-reviewer` agent should read `code-reviewer.md`. Use the `implement` role for these subagents, use `default` if `implement` is not available.

| Agent | Question | When to run |
|---|---|---|
| `code-reviewer` | "Does this code do what it should, correctly and idiomatically?" | Always |
| `error-auditor` | "What can go wrong silently at runtime?" | If error handling / try-catch / fallback logic touched |
| `test-reviewer` | "What behaviors are untested or poorly tested?" | If test files changed or testable code added |
| `completeness-checker` | "Does this PR deliver everything the linked issue requires?" | If PR links to an issue |
| `simplifier` | "Can this be expressed more clearly without changing behavior?" | Always (runs last, after others) |

**Targeted review:** If the user specified aspects, only run matching agents:
- `code` → code-reviewer
- `errors` → error-auditor
- `tests` → test-reviewer
- `completeness` → completeness-checker
- `simplify` → simplifier

**For each agent**, launch via the `agent` tool. Run `code-reviewer`, `error-auditor`, `test-reviewer`, and `completeness-checker` in parallel. Run `simplifier` after the others complete.

Provide each agent with:
- The list of changed files with paths
- The branch description and linked issue context
- Instructions to read the actual changed files for full context

All agents use confidence scoring (0-100, only report findings ≥ 80).

Update task: review → completed, write-review → in_progress.

## Step 5: Write review findings

Compile all findings from all agents into a single structured comment:

```bash
cat > /tmp/dg-comment.md << 'DG_EOF'
<!-- dg-review -->
## Code Review

### Critical
<findings with severity: critical, if any>

### Important
<findings with severity: high, if any>

### Suggestions
<findings with severity: medium or low, if any>

### Strengths
<notable positive observations>

**Agents run:** <list of agents>

---
*Reviewed by doppelganger*
DG_EOF
dg branch comment < /tmp/dg-comment.md
```

Update task: write-review → completed, assess → in_progress.

## Step 6: Independent assessment

Launch the `independent-assessor` subagent (MD file available in .claude/skills/dg-review/). Give the agent the path, _don't read the MD file yourself_.

Provide the assessor with:
- The full review text
- The branch context (title, body, comments)
- Instructions to **read the actual code** for each finding and verify independently

The assessor classifies each finding as:
- **Genuine issue** — Real problem, should fix before merge. Explain why.
- **Nitpick** — Stylistic, doesn't affect correctness. Explain why it doesn't matter.
- **False positive** — Not actually an issue. Explain why the code is correct.
- **Deferred** — Real issue but out of scope. Should track separately.

If a finding was already addressed in prior commits or dg branch discussion, classify as false positive with a note.

After classifying all findings, produce an **action plan** listing what to fix, in what order.

**Important guidance on "deferred" classifications:** Test coverage gaps should NOT be automatically deferred. If a branch adds new testable code, tests should ship with it — even if that means adding test infrastructure to a package that lacks it. Only defer tests when the gap is truly unrelated to the branch's changes (e.g., pre-existing untested code that the branch happens to touch). When tests are deferred, the assessor must note whether a tracking dg issue exists or needs to be created.

Update task: assess → completed, write-assess → in_progress.

## Step 7: Write assessment

```bash
cat > /tmp/dg-comment.md << 'DG_EOF'
<!-- dg-assessment -->
## Review Assessment

<link to review comment>

### Classifications

| Finding | Classification | Reasoning |
|---|---|---|
| <summary> | genuine/nitpick/false-positive/deferred | <1-2 sentences> |

### Action Plan

<numbered list of what to fix, ordered by priority>

---
*Assessment by doppelganger*
DG_EOF
dg branch comment < /tmp/dg-comment.md
```

Update task: write-assess → completed, summary → in_progress.

## Step 8: CLI summary

Present to the user:
- Per-finding breakdown: summary, classification, reasoning
- Counts: genuine, nitpicks, false positives, deferred
- Action plan

If any findings were classified as **deferred**, ask the user if they want to create issues for them:
```bash
cat > /tmp/issue-body.md << 'DG_EOF'
<body referencing branch and finding>
DG_EOF
dg issue create --name "<title>" < /tmp/issue-body.md
```

Update task: summary → completed.

Suggest next step:
- If genuine issues: `/dg-implement <finding-numbers>`
- If all clear: `/dg-publish`
