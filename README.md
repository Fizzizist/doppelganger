# Doppelganger

![logo](./assets/doppelganger.png)

## Use Cases
Why would anyone in their right mind use this thing?

- In my case: I want a GitHub/GitLab type experience _locally_ with just me and my agents so I am not spamming AI chat all over my GitHub repo.
- Maybe you don't host your repo on any GitHub/GitLab service, but you want a local issue/branch conversation interface for you and your agents to collaborate.

### More detail on my specific use case

This app solve a very specific problem that likely wouldn't help the majority of the population.

I use the [mach6 skillset](https://github.com/Fizzizist/illustrious-manager/tree/trunk/.claude/skills) for agentic development. I like most things about it, but I wanted a local version where I didn't have to use GitHub as the main message bus between me and my agents.

Doppelganger solves this with a local interface for local issue/branch conversations between you and your team of agents.

I went from this:
```mermaid
sequenceDiagram
    actor Me
    participant Local Git Repo
    participant Issue Assessor Agent
    participant Planner Agent
    participant Implementer Agent
    participant Reviewer Agent
    participant Reviewer Subagents
    participant GitHub
    
    Me->>GitHub: Create Issue 
    Me->>Issue Assessor Agent: /mach6-issue
    Issue Assessor Agent->>GitHub: reads issue
    Issue Assessor Agent->>GitHub: Posts issue assessment
    Me->>GitHub: Address issue assessment ambiguities
    Me->>Planner Agent: /mach6-plan
    Planner Agent->>GitHub: reads issue and ambiguity resolution
    Planner Agent->>Local Git Repo: creates local branch
    Planner Agent->>GitHub: Posts PR draft with plan
    Me->>Implementer Agent: /mach6-implement
    Implementer Agent->>GitHub: reads PR plan
    Implementer Agent->>Local Git Repo: Implements the plan
    Me->>Implementer Agent: /mach6-push
    Implementer Agent->>Local Git Repo: commits changes
    Implementer Agent->>GitHub: Pushes changes
    Me->>Reviewer Agent: /mach6-review
    Reviewer Agent->>Reviewer Subagents: spins up sub reviewers
    Reviewer Subagents->>Local Git Repo: analyze aspects of changes
    Reviewer Subagents->>Reviewer Agent: report back with confidence scores
    Reviewer Agent->>GitHub: Post review and review assessment
    Me->>Implementer Agent: /mach6-implement (for review)
    Implementer Agent->>Local Git Repo: makes changes based on review
    Me->>Implementer Agent: /mach6-push
    Implementer Agent->>Local Git Repo: commits changes
    Implementer Agent->>GitHub: pushes changes
    Me->>Local Git Repo: Perform my own review
    Me->>GitHub: commit and push maybe
    Me->>GitHub: rewrite description and close most comments
    Me->>GitHub: set PR ready for review
```

to this:

```mermaid
sequenceDiagram
    actor Me
    participant Local Git Repo
    participant Doppelganger
    participant Issue Assessor Agent
    participant Planner Agent
    participant Implementer Agent
    participant Reviewer Agent
    participant Reviewer Subagents
    participant GitHub
    
    Me->>GitHub: Create Issue
    GitHub->>Doppelganger: sync issue 
    Me->>Issue Assessor Agent: /dg-issue
    Issue Assessor Agent->>Doppelganger: reads issue
    Issue Assessor Agent->>Doppelganger: Posts issue assessment
    Me->>Doppelganger: Address issue assessment ambiguities
    Me->>Planner Agent: /dg-plan
    Planner Agent->>Doppelganger: reads issue and ambiguity resolution
    Planner Agent->>Local Git Repo: creates local branch
    Planner Agent->>Doppelganger: Creates `dg branch`
    Me->>Implementer Agent: /dg-implement
    Implementer Agent->>Doppelganger: reads dg branch plan
    Implementer Agent->>Local Git Repo: Implements the plan
    Me->>Implementer Agent: /dg-commit
    Implementer Agent->>Local Git Repo: commits changes
    Me->>Reviewer Agent: /dg-review
    Reviewer Agent->>Reviewer Subagents: spins up sub reviewers
    Reviewer Subagents->>Local Git Repo: analyze aspects of changes
    Reviewer Subagents->>Reviewer Agent: report back with confidence scores
    Reviewer Agent->>Doppelganger: Post review and review assessment
    Me->>Implementer Agent: /dg-implement (for review)
    Implementer Agent->>Local Git Repo: makes changes based on review
    Me->>Implementer Agent: /dg-commit
    Implementer Agent->>Local Git Repo: commits changes
    Me->>Local Git Repo: Perform my own review
    Me->>Local Git Repo: maybe commit
    Me->>Implementer Agent: /dg-publish
    Implementer Agent->>GitHub: Opens PR
    Me->>GitHub: Refine description and set as ready for review
```

## Controls

### Issue List

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `Enter` / `l` | Open selected issue thread |
| `n` | Create new issue (opens name input) |
| `a` | Toggle archive on selected issue |
| `A` | Toggle showing archived issues |

### Thread — Navigating

| Key | Action |
|-----|--------|
| `q` / `Esc` / `h` | Return to issue list |
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `e` | Edit selected item (description or comment) |
| `H` | Toggle hidden on selected comment |
| `Ctrl+u` | Scroll up |
| `Ctrl+d` | Scroll down |
| `Ctrl+w` `j` | Focus the comment input box |
| `Ctrl+w` `k` | Focus the thread |

### Thread — Comment Input Box

The input box is a [vim-mode text editor](https://crates.io/crates/hjkl-engine). All standard hjkl-engine keybindings apply. Of note:

| Key | Action |
|-----|--------|
| `Esc` | Exit insert/visual mode (or return focus to thread if already in normal mode) |
| `Enter` (normal mode) | Submit comment |
| `Enter` (insert mode) | Insert newline |

### Modals

| Key | Name Input Modal | Error Modal |
|-----|-------------------|------------|
| `Esc` | Cancel | Dismiss |
| `Enter` | Confirm | Dismiss |
| `Backspace` | Delete last character | — |
| Printable chars | Append to name | — |

## Installation

```sh
cargo install --git https://github.com/Fizzizist/doppelganger.git
```
