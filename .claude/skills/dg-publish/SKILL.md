---
name: dg-publish
description: "Pre-merge checks, version bump, create PR/MR. Version bump happens BEFORE merge (on the feature branch) because master requires PRs. Usage: dg-publish"
---

# dg-publish — Version Bump, Push, PR/MR

**User input:** $ARGUMENTS

## Global Rules

1. **Doppelganger as shared memory** — Plans, reviews, assessments, and progress are written as branch/issue comments so any future session can pick up context.
3. **Safe git** — Never use `git add -A` or `git add .`. Stage files by name. Never stage secrets.
4. **Task tracking** — Use the `update_task` tool to show progress.
5. **Non-interactive `gh`/`glab`** — Set `GH_PAGER=cat` and `GH_EDITOR=cat` before all `gh` commands to prevent interactive prompts from hanging the agent. Use `--body-file` instead of inline `--body` for all `gh pr comment`, `gh pr create`, and `gh issue create` calls to avoid shell interpretation of backticks.

## Step 1: Set up task tracking

- title: "version", description: "Version bump (on feature branch)"
- title: "docs", description: "Update documentation"
- title: "post", description: "Post PR/MR"
- title: "checks", description: "Pre-merge checks"

## Step 2: Version bump (on the feature branch — BEFORE PR/MR)

**This step is mandatory for projects with versioning.** The version bump MUST happen on the feature branch and be pushed as part of the PR, because the default branch requires PRs for all changes — you cannot push commits directly to it.

1. Detect versioning:
   ```bash
   # Check for version files: package.json, Cargo.toml, pyproject.toml, version.txt, etc.
   ```

2. Determine the current version and what the new version should be. If the bump level isn't obvious from the PR context, **ask the user**:
   - **Patch**: Bug fixes, minor improvements
   - **Minor**: New features, non-breaking changes
   - **Major**: Breaking changes

3. Apply the version bump. Check AGENTS.md / CONTRIBUTING.md for project-specific version bump procedures (e.g., sync scripts, build steps that embed the version).

4. Commit on the feature branch:
   ```bash
   git add <version-files>
   git commit -m "chore: bump version to <new-version>"
   ```

If the project doesn't use versioning, skip this step.

Update task: version → completed, docs → in_progress.

## Step 3: Update documentation

Proactively review and update ALL documentation affected by the branch's changes. Check _everything_ in the repo.

1. Identify changed features by reading the PR diff:
   ```sh
    git diff $(git symbolic-ref --short refs/remotes/origin/HEAD)
   ```

2. Scan all documentation for references to changed features:
   - README.md files across all packages
   - docs/ directory (all .md files)
   - AGENTS.md, CLAUDE.md, CONTRIBUTING.md, .dreb/CONTEXT.md
   - Example files and their READMEs
   - Help text and CLI flag documentation
   - CHANGELOG.md or CHANGES.md

3. For each doc, verify:
   - CLI flags, settings, and commands match current code
   - Code snippets and examples are accurate
   - Cross-references point to real files and valid anchors
   - No stale descriptions of removed or changed features
   - Environment variables listed match current code

4. Fix all inaccuracies found. Commit and push:
   ```bash
   git add <doc-files>
   git commit -m "docs: update documentation for <version>"
   ```

If no documentation changes are needed (rare), skip this step.

Update task: docs → completed, post → in_progress.

## Step 4: Post PR/MR

Use `glab` is place of `gh` here if the repo is hosted on GitLab.

```sh
git push -u origin feature/issue-<N>-<slug>

# Open PR
cat > /tmp/gh-body.md << 'DG_EOF'
Closes #<remote_issue_id>

# What

<list of changes made>

# Why

<description of _why_ these changes matter for the project>

# Local Testing Instructions

<list of instructions for a _human_ reviewer to take to reproduce the behavior locally>

MACH6_EOF
gh pr create --title "<title>" --body-file /tmp/gh-body.md
```

Update task: post → completed, checks → in_progress.

## Step 5: Pre-merge checks

```bash
gh pr checks <pr-number>
```

**Note:** `gh pr checks` returns exit code 8 while checks are still pending — this is expected, not a failure. Wait and re-run if needed.

Verify:
- [ ] CI is passing
- [ ] No merge conflicts
- [ ] All review findings addressed (check for genuine items in latest assessment)

If there are blocking issues, report them and suggest fixes:
- **Failed CI/Unaddressed findings**: `/dg-implement`
- **Merge conflicts**: Suggest resolving manually or rebasing

### Pre-merge checklist

Check for contributing guidelines first:
```bash
# Read first found: CONTRIBUTING.md, DEVELOPMENT.md, .github/CONTRIBUTING.md, AGENTS.md, CLAUDE.md
```

Then check if these need attention:
- [ ] Documentation updated (if behavior changed) — check README, docs/, docstrings, help text
- [ ] Changelog updated (if project maintains one — check CHANGELOG.md, CHANGES.md)
- [ ] Tests passing locally

Present the checklist to the user. If items need attention, address them before proceeding.

Update task: checks → completed.

## Step 6: Report

Report: what was done throughout. Link to the PR.
