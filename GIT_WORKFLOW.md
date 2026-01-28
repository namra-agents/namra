# Git Workflow for Nexus Development

**Last Updated**: January 28, 2026

---

## Branch Structure

```
main                          ← Stable releases (Week 1-2 complete)
  │
  ├── feature/week3-tools     ← Week 3: Built-in Tools (CURRENT)
  │
  ├── feature/week4-runtime   ← Week 4: Agent Runtime (FUTURE)
  │
  ├── feature/week5-api       ← Week 5: gRPC/HTTP API (FUTURE)
  │
  └── ...                     ← Future weekly feature branches
```

---

## Workflow

### For Each Week's Development

1. **Start from main**
   ```bash
   git checkout main
   git pull origin main  # If working with remote
   ```

2. **Create feature branch**
   ```bash
   git checkout -b feature/weekN-name
   # Example: feature/week3-tools
   ```

3. **Develop incrementally**
   ```bash
   # Make changes
   git add .
   git commit -m "Implement X feature

   - Added Y
   - Fixed Z

   Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
   ```

4. **Commit frequently**
   - Commit after each logical unit of work
   - Write descriptive commit messages
   - Test before committing

5. **Week complete: Merge to main**
   ```bash
   # Ensure all tests pass
   cargo test
   cargo build --release

   # Switch to main
   git checkout main

   # Merge feature branch
   git merge feature/week3-tools --no-ff

   # Tag the release
   git tag -a v0.3.0 -m "Week 3 Complete: Built-in Tools"

   # Push (if using remote)
   git push origin main --tags
   ```

---

## Current Status

### ✅ Committed to `main`
- **Week 1**: Project setup, configuration system, CLI basics
- **Week 2**: LLM adapters, Anthropic integration, streaming

**Commit**: `b962ff1` - "Initial commit: Nexus MVP Foundation (Weeks 1-2)"
**Date**: January 28, 2026
**Stats**: 44 files, 8,930 lines

### 🚧 Active Branch
- **Branch**: `feature/week3-tools`
- **Goal**: Implement built-in tools (HTTP, Filesystem, Calculator)
- **Status**: Just created, ready for development

---

## Commit Message Guidelines

### Format
```
Short summary (50 chars or less)

Longer explanation if needed. Wrap at 72 characters.

- Bullet points for multiple changes
- Keep related changes together
- Reference issues if applicable

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

### Good Examples
```bash
# Feature implementation
git commit -m "Implement HTTP tool with GET/POST support

- Add HttpTool struct with reqwest client
- Implement Tool trait for HTTP operations
- Add parameter validation
- Include timeout handling

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"

# Bug fix
git commit -m "Fix streaming token count calculation

The token counter was not accumulating correctly in streaming
mode, causing incorrect cost calculations.

- Fixed accumulator in StreamChunk::Done
- Added test for streaming token counting

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"

# Documentation
git commit -m "Add tool system architecture documentation

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"

# Tests
git commit -m "Add unit tests for HTTP tool

- Test GET requests
- Test POST with JSON body
- Test timeout handling
- Test error cases

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

### What to Avoid
```bash
# Too vague
git commit -m "Update files"
git commit -m "Fix bug"
git commit -m "WIP"

# No context
git commit -m "Changes"
git commit -m "."

# Too many unrelated changes
git commit -m "Add HTTP tool, fix config parser, update docs, refactor CLI"
```

---

## Commands Reference

### Daily Workflow
```bash
# Check current branch
git branch

# Check status
git status

# Stage changes
git add src/tool.rs src/http.rs
# Or stage all
git add .

# Commit
git commit -m "Your message"

# View commit history
git log --oneline
git log --graph --oneline --all

# View changes
git diff
git diff --staged

# View file history
git log --follow src/tool.rs
```

### Branch Management
```bash
# List branches
git branch -a

# Create and switch
git checkout -b feature/new-feature

# Switch branches
git checkout main
git checkout feature/week3-tools

# Delete branch (after merging)
git branch -d feature/week3-tools

# Force delete (if not merged)
git branch -D feature/week3-tools
```

### Merging
```bash
# Merge feature into main (from main branch)
git checkout main
git merge feature/week3-tools --no-ff

# Abort merge if conflicts
git merge --abort

# View merged branches
git branch --merged
```

### Tagging
```bash
# Create annotated tag
git tag -a v0.3.0 -m "Week 3: Built-in Tools"

# List tags
git tag -l

# View tag details
git show v0.3.0

# Push tags to remote
git push origin --tags
```

### Undoing Changes
```bash
# Unstage file
git reset HEAD file.rs

# Discard changes in working directory
git checkout -- file.rs

# Undo last commit (keep changes)
git reset --soft HEAD~1

# Undo last commit (discard changes)
git reset --hard HEAD~1

# Revert a commit (create new commit)
git revert abc123
```

---

## Weekly Development Cycle

### Week Start
1. Ensure main branch is clean
2. Create feature branch: `git checkout -b feature/weekN-name`
3. Review NEXT_STEPS.md for tasks
4. Start implementing

### During Week
1. Commit frequently (multiple times per day)
2. Keep commits focused and atomic
3. Write descriptive commit messages
4. Run tests before committing: `cargo test`

### Week End
1. Ensure all tests pass: `cargo test`
2. Build release binary: `cargo build --release`
3. Update documentation:
   - PROJECT_STATUS.md
   - NEXT_STEPS.md (for next week)
   - Create WEEKN_COMPLETE.md
4. Merge to main:
   ```bash
   git checkout main
   git merge feature/weekN-name --no-ff
   ```
5. Tag release:
   ```bash
   git tag -a v0.N.0 -m "Week N Complete: Feature Name"
   ```
6. Create next week's branch:
   ```bash
   git checkout -b feature/weekN+1-name
   ```

---

## Remote Repository (Future)

When pushing to GitHub/GitLab:

```bash
# Add remote
git remote add origin https://github.com/username/nexus.git

# Push main branch
git push -u origin main

# Push feature branch
git push -u origin feature/week3-tools

# Push tags
git push origin --tags

# Pull latest changes
git pull origin main
```

---

## Best Practices

### ✅ Do
- Commit frequently (multiple times per day)
- Write clear, descriptive commit messages
- Keep commits focused on one logical change
- Test before committing
- Use feature branches for each week
- Merge to main only when week is complete
- Tag releases at week boundaries

### ❌ Don't
- Commit broken code to main
- Use vague commit messages ("fix", "update", "wip")
- Mix unrelated changes in one commit
- Commit sensitive data (API keys, passwords)
- Force push to main branch
- Delete branches until week is merged

---

## Emergency Recovery

### Accidentally committed to wrong branch
```bash
# Move last commit to another branch
git checkout correct-branch
git cherry-pick abc123
git checkout wrong-branch
git reset --hard HEAD~1
```

### Need to fix last commit message
```bash
git commit --amend -m "New commit message"
```

### Accidentally deleted file
```bash
git checkout HEAD -- deleted-file.rs
```

---

## Current Repository State

```bash
$ git log --oneline --graph --all
* b962ff1 (HEAD -> feature/week3-tools, main) Initial commit: Nexus MVP Foundation (Weeks 1-2)
```

**Next Steps**:
1. Implement tools in `feature/week3-tools` branch
2. Commit frequently during Week 3 development
3. Merge to `main` when Week 3 is complete
4. Create `feature/week4-runtime` branch

---

**For development workflow**: See [NEXT_STEPS.md](NEXT_STEPS.md)
**For project status**: See [PROJECT_STATUS.md](PROJECT_STATUS.md)
