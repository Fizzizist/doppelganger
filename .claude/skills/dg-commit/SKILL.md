---
name: dg-commit
description: "Commit changes, and post a progress comment in doppelganger(dg). Stages files by name (never git add -A), matches existing commit style. Usage: dg-commit [optional commit message]"
argument-hint: "[commit message]"
---

# dg-commit — Commit, Progress Comment

**User input:** $ARGUMENTS

## Global Rules

1. **Doppelganger as shared memory** — Plans, reviews, assessments, and progress are written as branch/issue comments so any future session can pick up context.
2. **HTML markers** — Use `<!-- dg-progress -->` as the first line of progress comment bodies.
3. **Safe git** — Never use `git add -A` or `git add .`. Stage files by name. Never stage secrets (.env, credentials, tokens, keys).
4. **Task tracking** — Use the `update_task` tool to show progress.

## Step 1: Set up task tracking

- title: "stage", description: "Stage changes"
- title: "commit", description: "Commit"
- title: "comment", description: "Post progress comment"

## Step 2: Stage changes

Run `git status` and `git diff` to understand the current state.

- If you have context from this session about which files were modified, stage those by name.
- If files are already staged and look correct, proceed.
- If unclear (fresh session, no context), review all changes and ask the user what to stage.
- **Never** use `git add -A` or `git add .`
- **Never** stage secrets (.env, credentials, tokens, keys)

Update task: stage → completed, commit → in_progress.

## Step 3: Commit

Check recent commit style:
```bash
git log --oneline -10
```

Generate a commit message that:
- Follows the repository's existing style
- Summarizes the nature of the changes
- Uses the user's override message if provided

```bash
git commit -m "<message>"
```

Update task: commit → completed, comment → in_progress.

## Step 4: Post progress comment

Post a progress comment:
```bash
cat > /tmp/dg-comment.md << 'DG_EOF'
<!-- dg-progress -->
## Progress Update

<summary of changes in this batch>

**Commit:** \`<hash>\`

---
*Progress tracked by doppelganger*
DG_EOF
dg branch comment < /tmp/dg-comment.md
```

Update task: comment → completed.

Report: what was committed.

Suggest next step: `/dg-review`
