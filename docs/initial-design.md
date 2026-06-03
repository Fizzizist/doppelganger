# Doppelganger

A local conversation layer for git repositories. The thing GitHub/GitLab gives you for issue and PR discussions, but running on your machine, reading your local repo directly, with no remote service dependency.

## Why

Using GitHub or GitLab as a message bus for agent-driven development works, but has problems:

- **Single-account constraint.** Corporate GitHub billing means one account, so the human and agent appear as the same person in every thread.
- **Unnecessary round-trips.** All conversation happens on a remote web service even though all implementation and git operations happen locally. Pushing to a remote just to have a conversation with yourself is overhead.
- **Context poisoning.** The alternative — local spec files in the repo — creates stale documents that the LLM reads as ground truth instead of the actual code. GitHub avoids this by putting the conversation in a separate system.

Doppelganger keeps the separation that makes GitHub work, but eliminates the remote dependency and the account problem.

## What It Is

A Rust application with two faces:

- **A TUI** (built with Ratatui) for the human to browse threads, read conversations, edit or delete messages, view diffs, and leave inline review comments.
- **A CLI** for the LLM agent to read and post to threads programmatically.

Both interfaces sit on the same core library backed by a TursoDB (libSQL) database stored at the root of the repository tree — typically at `.doppelganger.db`. The conversation data is never in the repo, so the LLM cannot accidentally discover it during codebase operations.

Git operations are **read-only** from Doppelganger's perspective. It reads diffs and logs from the local repo to surface context, but never commits, pushes, or modifies the repository.

## Roles

Every message has a role indicating who wrote it:

Roles are configurable in the configuration. Roles can be restricted to CLI input only (agents) or TUI only (humans).

## Threads

Two types of thread:

- **Issue thread** — like a GitHub issue. No branch attached. Exists to discuss scope, plan, and assess work before starting.
- **Branch thread** — like a GitHub PR. Bound to a specific git branch. Used during implementation and review.

Any thread can be linked to any other thread through relationships, but they are never combined. The user has precise Control about what is visible to other agent roles reading the thread. If a user marks a thread or comment as hidden, the agent reading it can no longer see it. This allows for very precise control over what gets fed into context.

## Remote Issue Sync

Issue threads can be created from a GitHub or GitLab issue. The remote issue body is the source of truth — Doppelganger pulls it on sync. Anything synced from the source cannot be edited. Local comments are _not_ pushed up to the source.

The local conversation (messages, comments, review findings) stays entirely in Doppelganger. Remote comments on the GitHub/GitLab issue are included in the sync. The point is to have the local conversation just be between the user and the agents. Github/GitLab comments are for multiple humans to discuss.

## Diff Viewer

Built into the TUI. File-by-file diff browsing with the ability to leave inline comments on specific lines, pinned to a specific commit SHA + line number. This gives the reviewer agent (and the human) the same affordance as GitHub's PR review inline comments, without the round-trip.

## Architecture

```
┌──────────────┐    ┌──────────────┐
│   TUI App    │    │   CLI API    │
│  (Ratatui)   │    │  (clap)      │
└──────┬───────┘    └──────┬───────┘
       │                   │
       └───────┬───────────┘
               │
       ┌───────┴───────┐
       │   Core Lib    │
       │  Thread CRUD  │
       │  Git reading  │
       │  Remote sync  │
       └───────┬───────┘
               │
       ┌───────┴───────────┐
       │                   │
   TursoDB              Git CLI
 (conversation)      (read-only)
```

The core library owns all logic. The TUI and CLI are thin frontends.

## Implementation Phases

| Phase | Scope |
|-------|-------|
| **1** | Core library + CLI. Thread and message CRUD. TursoDB storage. Role flag. Basic `thread-list`, `thread-read`, `thread-comment`, `thread-edit`, `thread-delete` commands. |
| **2** | TUI. Thread list, conversation view, message editing and deletion. |
| **3** | Git integration. Branch binding, diff and log reading, thread status tracking against git state. |
| **4** | Diff viewer. File-by-file diff browsing in TUI, inline comments keyed to commit SHA + file + line. |
| **5** | Remote sync. GitHub and GitLab issue import, pull-on-sync for issue body. |

Phases 1–3 produce a working replacement for the GitHub conversation layer. Phases 4–5 are the "better than GitHub" features.

## Storage

Conversation data lives at `~/.local/share/doppelganger/projects/<project-hash>/` — outside any git repository. Project hash is derived from the repo path so each repo gets its own conversation space automatically.

## Key Design Decisions

- **Conversation is editable.** Messages can be edited or deleted to prune stale context. This is a deliberate departure from strict append-only logs — the point is keeping the conversation lean and current.
- **Conversation is not version-controlled.** It's ephemeral working context, not permanent record. If you need a record, the git history of the actual code is the record.
- **No daemon.** Doppelganger is a CLI tool invoked on demand. The LLM calls it, gets output, and moves on. No persistent process to manage.
- **Structured CLI output.** Human-readable for interactive (`-i` flag) use, JSON output (primary use case) for machine consumption by the LLM.
