# Testing Plan for PR #14 - CPU Usage Optimization

## Summary
PR #14 claims to reduce CPU usage from ~15% to ~5-9% through:
1. Using minimal `System::new_with_specifics()` instead of `System::new_all()`
2. Reusing `SessionDetector` across poll cycles
3. Increasing poll interval from 2s to 3.5s

## Test Procedure

### 1. Measure Baseline (Current Code)
```bash
# Start c9watch from current branch (feature/mobile-web)
npm run tauri dev

# In another terminal, measure CPU for 2 minutes
./scripts/measure-cpu.sh 120
```

**Record results:**
- Average CPU: ______%
- Min CPU: ______%
- Max CPU: ______%

### 2. Checkout PR #14 Branch
```bash
# Fetch the PR branch
git fetch origin pull/14/head:pr-14-test

# Create a test branch
git checkout -b test/pr14-cpu-optimization pr-14-test

# Rebuild
npm run tauri build --debug
```

### 3. Measure After Optimization
```bash
# Start c9watch with optimized code
npm run tauri dev

# Measure again for 2 minutes
./scripts/measure-cpu.sh 120
```

**Record results:**
- Average CPU: ______%
- Min CPU: ______%
- Max CPU: ______%

### 4. Test Different Scenarios

#### Scenario A: Single Active Session
- Start 1 Claude Code session
- Measure CPU: `./scripts/measure-cpu.sh 60`
- Result: ______%

#### Scenario B: Multiple Active Sessions
- Start 3-5 Claude Code sessions in different projects
- Measure CPU: `./scripts/measure-cpu.sh 60`
- Result: ______%

#### Scenario C: Notification Responsiveness
- Start a Claude session
- Send a prompt that requires permission (e.g., run a bash command)
- Time from permission needed → notification appears
- Before PR #14 (2s interval): max ~2s delay
- After PR #14 (3.5s interval): max ~3.5s delay
- Acceptable? ☐ Yes ☐ No

### 5. Verify Functionality
- [ ] Sessions are detected correctly
- [ ] Session status updates properly (Working/Waiting/Permission)
- [ ] Notifications fire when status changes
- [ ] Session list updates in the UI
- [ ] Custom names/titles still work
- [ ] WebSocket clients receive updates

## Expected Results

On my system (553 processes, 1-2 active Claude sessions):
- **Before**: ~1.8% average CPU
- **After**: ~0.9-1.2% average CPU (50% reduction)

If the contributor saw 15% → 5-9% (system with more processes/sessions):
- Our reduction might be smaller in absolute terms but similar in percentage

## Success Criteria

✅ PR #14 is good to merge if:
- CPU usage decreases by at least 30%
- All functionality tests pass
- Notification delay (3.5s) is acceptable
- No crashes or errors during 2+ minute runs

❌ PR #14 needs work if:
- CPU usage increases or stays the same
- Sessions aren't detected properly
- Notifications stop working

## After Testing

Once verified, leave comment on PR #14 with:
- Your measured results (before/after)
- System specs for context
- Thanks and approval for merge
