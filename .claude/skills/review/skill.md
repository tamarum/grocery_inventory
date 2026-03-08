# Review Skill

Review code changes for quality.

## Commands

- **View staged changes**: `git diff --cached`
- **View all changes**: `git diff`
- **View changes vs branch**: `git diff main...HEAD`

## Quality Checklist

1. Error handling follows `rust-style.md` conventions
2. New public APIs have corresponding tests
3. No `.unwrap()` outside test code
4. Feature gates used correctly for web code
5. Mutex lock scopes are minimal
6. Commit message follows `git-workflow.md` conventions
