# CLAUDE.md

This file provides guidance to any LLM agent when working with code in this repository.

## Engineering Policies

These are _strict_ policies that must be followed by all engineers and developers in this project. MRs and PRs will be rejected if these policies are violated.

### Dependency Management

- All dependencies _must_ be added, removed and updated using `cargo` on the command line.
- Under no circumstances should the Cargo.toml be manually edited with regard to dependencies.

### Coding

- The use of `.unwrap()` is forbidden under _all_ circumstances. The program should _never_ panic.
- In a case where something needs to be unwrapped and it is _logically impossible_ for a panic to occur, the use of `.expect()` with an informative message is permitted.
- The use of `pub(crate)` is forbidden. It is a leaky boundary that exposes implementation details to the rest of the crate without forcing a real API decision. If something needs to cross a module boundary, either make it fully `pub` with a thought-out interface, or restructure so the caller doesn't need access at all.
- Always run `cargo fmt` before committing code.
- Always run `cargo clippy` before committing code.
- Keep code comments to a minimum. Only comment in cases where something is unable to be gleaned from the code itself.
- The use of `eprintln!`, `println!`, `eprint!`, `print!`, and `dbg!` is forbidden in TUI code as it conflicts with TUI rendering. All diagnostic output must go through a `logging` module. Direct stdout/stderr writes corrupt the TUI render.

### Testing

- Frontend REPL ratatui testing should be done with insta snapshots.
- It is expected that Test Driven Development will be the main way that code is implemented in this repo, so most code should have tests that test _behavior_.
- Any bug fix should include regression at least one regression test.

## Project

This project is in its infancy. See `./docs/initial-design.md` for the high level initial design plan.

## Architecture

### Database Concurrency

Multiple doppelganger processes can access the same `.doppelganger.db` file concurrently (e.g. a TUI process and a CLI agent). This is enabled by:

1. **`LIMBO_DISABLE_FILE_LOCK=1`** — disables turso's exclusive `flock` on the database file at open time, which would otherwise prevent concurrent access.
2. **`experimental_multiprocess_wal(true)`** — enables turso's shared WAL coordination, which uses `fcntl` byte-range locks for cross-process coordination instead of exclusive file locks.
3. **`Connection::busy_timeout(Duration::from_secs(5))`** — retries on SQLite-level busy conditions for up to 5 seconds.

Do not remove any of these without understanding the concurrency implications.
