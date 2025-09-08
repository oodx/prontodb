# Phase 5: Certification Protocol

## Introduction

**Purpose**: Final verification and merge to production. The "release manager" ensures everything is ready.

**Duration**: 30 minutes - 1 hour (should be quick if previous phases done well)

**Output**: Code merged to main, feature branch deleted, deployment ready.

---

## Roles in Certification Phase

### Full Team Scenario
- **Release Manager (Lead)**: Orchestrates certification
- **DevOps Engineer**: Validates CI/CD readiness
- **Security Officer**: Final security scan
- **QA Lead**: Confirms test results
- **Technical Lead**: Approves architecture
- **Product Owner**: Signs off on delivery

### Compressed Scenarios

**Solo Developer Mode**
```
You = Release Manager + DevOps + QA
- First, be Release Manager: Is everything complete?
- Then, be DevOps: Will this deploy cleanly?
- Finally, be QA: Are we confident in quality?
```

**AI Assistant + Human Mode**
```
AI = Release Preparer (checks everything)
Human = Release Approver (makes merge decision)
AI must NEVER merge directly to main
```

**Startup Mode**
```
Founder = Approver
Dev = Release Manager + Basic DevOps
Skip: Security scan, Formal QA signoff
```

---

## Quick Start Guide

### 1. Pre-Certification Checklist (5 minutes)
```bash
# On feature branch
git checkout feature/[component-name]

# Rebase on latest main
git fetch origin
git rebase origin/main

# Run all checks
cargo fmt --check
cargo clippy -- -D warnings
cargo test --all
cargo build --release
```

### 2. Set Context (2 minutes)
```markdown
You are in CERTIFICATION PHASE.
Role: Release Manager preparing for merge
Task: Verify production readiness
Constraint: Fix only critical issues
Output: Merge-ready PR
```

### 3. Certification Steps
```markdown
1. All checks green
2. Squash commits appropriately
3. Write merge commit message
4. Merge to main
5. Delete feature branch
6. Tag release if needed
```

---

## Deep Dive: The Certification Process

### Stage 1: Automated Checks (DevOps Hat)

**CI/CD Pipeline Must Pass:**
```yaml
# .github/workflows/ci.yml should verify:
- Format: cargo fmt --check
- Linting: cargo clippy -- -D warnings
- Tests: cargo test --all
- Build: cargo build --release
- Coverage: cargo tarpaulin
- Benchmarks: cargo bench
- Docs: cargo doc --no-deps
```

**Verification Commands:**
```bash
# Local verification before pushing
./scripts/pre-merge-check.sh

# Script contents:
#!/bin/bash
set -e
echo "Running pre-merge certification..."

echo "1. Format check..."
cargo fmt --check

echo "2. Clippy check..."
cargo clippy -- -D warnings

echo "3. Test suite..."
cargo test --all

echo "4. Release build..."
cargo build --release

echo "5. Documentation..."
cargo doc --no-deps

echo "✅ All checks passed!"
```

### Stage 2: Code Review Completeness (Tech Lead Hat)

**Review Checklist Verification:**
```markdown
## Code Review Complete
- [ ] Architecture approved
- [ ] No unresolved comments
- [ ] Requested changes implemented
- [ ] Follow-up tickets created for deferred items

## Technical Debt Assessment
- [ ] No new warnings introduced
- [ ] Complexity metrics acceptable
- [ ] Dependencies justified
- [ ] Performance impact measured
```

### Stage 3: Git History Cleanup (Release Manager Hat)

**Squash Strategy:**
```bash
# Interactive rebase to clean history
git rebase -i origin/main

# Squash pattern:
# - Keep major feature commits
# - Squash fix commits
# - Squash WIP commits

# Example transformation:
# Before:
# - WIP: starting retry logic
# - checkpoint: types defined
# - fix: typo
# - checkpoint: implementation complete
# - fix: PR feedback

# After:
# - feat: add retry logic with exponential backoff
```

**Commit Message Standards:**
```markdown
feat(component): add retry logic with exponential backoff

- Implements automatic retry for transient failures
- Configurable max attempts and delays
- Supports exponential, linear, and fixed strategies

Closes #123
BREAKING CHANGE: None
```

### Stage 4: Security & Compliance (Security Officer Hat)

**Security Checklist:**
```markdown
## Security Verification
- [ ] No secrets in code
- [ ] Dependencies audited
- [ ] No unsafe code without justification
- [ ] Input validation present
- [ ] Error messages don't leak sensitive info
```

**Dependency Audit:**
```bash
# Check for known vulnerabilities
cargo audit

# Check for outdated dependencies
cargo outdated

# Verify license compatibility
cargo license
```

### Stage 5: Merge Execution (Release Manager Hat)

**Merge Protocol:**
```bash
# Final sync with main
git checkout main
git pull origin main
git checkout feature/[component-name]
git rebase origin/main

# If conflicts, resolve and re-test
cargo test --all

# Merge via PR (preferred)
# GitHub/GitLab UI: Squash and merge

# OR manual merge
git checkout main
git merge --squash feature/[component-name]
git commit -m "feat(component): comprehensive message"
git push origin main
```

### Stage 6: Post-Merge Cleanup (DevOps Hat)

**Cleanup Tasks:**
```bash
# Delete local feature branch
git branch -d feature/[component-name]

# Delete remote feature branch
git push origin --delete feature/[component-name]

# Tag if significant release
git tag -a v1.2.0 -m "Add retry logic"
git push origin v1.2.0

# Update changelog if maintained
echo "## v1.2.0 - Add retry logic" >> CHANGELOG.md
```

---

## Rollback Protocol

### If Issues Found Post-Merge

```bash
# Quick revert
git revert HEAD
git push origin main

# OR reset if caught immediately (dangerous)
git reset --hard HEAD~1
git push --force-with-lease origin main

# Create hotfix branch
git checkout -b hotfix/[issue-name]
# Fix issue using Five-Phase process (abbreviated)
```

---

## Release Tagging Strategy

### Semantic Versioning
```markdown
MAJOR.MINOR.PATCH

MAJOR: Breaking API changes
MINOR: New features, backward compatible
PATCH: Bug fixes only

Example progression:
- v1.0.0 - Initial release
- v1.1.0 - Add retry logic (new feature)
- v1.1.1 - Fix retry delay calculation
- v2.0.0 - Change retry API (breaking)
```

### Tag Creation
```bash
# Annotated tag with message
git tag -a v1.1.0 -m "feat: add retry logic

- Exponential backoff strategy
- Configurable max attempts
- Plugin architecture support"

# Push tag
git push origin v1.1.0
```

---

## Intra-Phase Certification

### Certification Exit Criteria
```markdown
## Ready to Complete

### Quality Gates
- [ ] All CI checks green
- [ ] Code review approved
- [ ] UAT sign-off received
- [ ] No critical bugs open

### Documentation
- [ ] CHANGELOG updated
- [ ] README current
- [ ] API docs generated
- [ ] Migration guide (if needed)

### Repository State
- [ ] Feature branch rebased on main
- [ ] Commits squashed appropriately
- [ ] Commit message follows standards

### Deployment Ready
- [ ] Build artifacts created
- [ ] Version number updated
- [ ] Release notes written

If any unchecked: FIX BEFORE MERGE
If all checked: PROCEED WITH MERGE
```

---

## Common Certification Failures

### Failure: Merge Conflicts
```markdown
❌ WRONG: Force merge without resolution
"It works on my branch"

✅ RIGHT: 
1. Rebase on latest main
2. Resolve conflicts carefully
3. Re-run all tests
4. Get re-review if significant changes
```

### Failure: Broken Main
```markdown
❌ WRONG: "I'll fix it in the next PR"

✅ RIGHT: 
1. Immediately revert if main broken
2. Fix in feature branch
3. Re-certify before merge
```

### Failure: Skipping Checks
```markdown
❌ WRONG: "Tests pass locally, good enough"
Merge despite CI failures

✅ RIGHT: 
ALL checks must be green:
- CI pipeline
- Security scan
- Performance tests
- Documentation build
```

### Failure: Poor Git History
```markdown
❌ WRONG: 50 commits including:
"fix"
"more fixes"  
"why doesn't this work"
"IT WORKS!"

✅ RIGHT: Clean, semantic commits:
"feat: add retry logic"
"test: comprehensive retry tests"
"docs: retry configuration guide"
```

---

## AI Assistant Instructions

### Certification Preparation
```markdown
When user says: "Prepare for certification"

AI Response:
"Starting CERTIFICATION PHASE for retry logic feature.

Running pre-certification checks:

1. **Format Check**
```bash
cargo fmt --check
```
✅ Passed

2. **Linting**
```bash
cargo clippy -- -D warnings
```
✅ No warnings

3. **Tests**
```bash
cargo test --all
```
✅ 47 tests passed

4. **Build**
```bash
cargo build --release
```
✅ Build successful

5. **Git History**
Current commits:
- checkpoint: types defined
- checkpoint: implementation complete  
- fix: PR feedback

Recommend squashing to:
- feat: add retry logic with exponential backoff

Ready for merge. Shall I prepare the squash commit message?"
```

### Merge Preparation
```markdown
When ready to merge:

AI Response:
"Merge preparation complete:

**Commit Message:**
```
feat(retry): add configurable retry logic

- Implements exponential, linear, and fixed backoff
- Configurable max attempts and delays
- Plugin architecture integration
- Comprehensive test coverage

Closes #123
```

**Pre-merge checklist:**
✅ All CI checks passing
✅ Code review approved
✅ UAT sign-off received
✅ Rebased on latest main
✅ No conflicts

**Next steps:**
1. You merge via GitHub PR interface
2. I'll prepare cleanup commands
3. I'll draft release notes

Ready for you to execute merge."
```

### Post-Merge
```markdown
After merge:

AI Response:
"Merge successful! Cleanup tasks:

```bash
# Delete local branch
git branch -d feature/retry-logic

# Delete remote branch  
git push origin --delete feature/retry-logic

# Pull latest main
git checkout main
git pull origin main
```

**Release tag** (if needed):
```bash
git tag -a v1.1.0 -m "feat: add retry logic"
git push origin v1.1.0
```

**CHANGELOG entry:**
```markdown
## [1.1.0] - 2024-01-15
### Added
- Configurable retry logic with multiple strategies
- Exponential backoff implementation
- Plugin architecture support for retry strategies
```

Certification complete! Feature successfully delivered."
```

---

## Emergency Procedures

### If Main Breaks After Merge
```markdown
## IMMEDIATE ACTION REQUIRED

1. **Verify the break**
```bash
git checkout main
git pull
cargo test --all
```

2. **Quick revert**
```bash
git revert HEAD -m "revert: retry logic breaking tests"
git push origin main
```

3. **Notify team**
"Main broken by retry logic merge, reverted in commit abc123"

4. **Fix properly**
- Create hotfix branch
- Fix issue
- Fast-track through phases
- Re-merge when stable
```

---

## Certification Metrics

### Success Indicators
- Merge on first attempt: >80%
- No reverts required: >95%
- CI passing pre-merge: 100%
- Clean git history: 100%

### Warning Signs
- Multiple merge attempts needed
- Frequent post-merge hotfixes
- CI failures discovered late
- Messy commit history

### Process Health
- Time from PR to merge: <2 days
- Certification phase duration: <1 hour
- Rollback frequency: <5%