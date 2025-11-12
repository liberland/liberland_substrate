# Security Analysis Report: PR #448 - Peace Accords Implementation

**Date**: November 12, 2025  
**Analyzed By**: GitHub Copilot Security Review  
**PR Title**: "newly released merits go to congress"  
**PR Number**: #448  
**Status**: 🔴 **CRITICAL SECURITY ISSUES FOUND**

---

## Executive Summary

This PR implements significant governance changes as part of "Peace Accords" for Liberland's blockchain. While the political motivations are documented, the implementation contains **several critical security vulnerabilities** that could result in **permanent loss of funds** and **governance centralization**.

### Risk Level: 🔴 **CRITICAL**

---

## Critical Security Issues

### 1. ⚠️ CRITICAL: Treasury Account Migration - Permanent Fund Loss Risk

**Severity**: CRITICAL  
**Impact**: Potential permanent loss of all funds in old treasury account

#### Description
The PR changes the treasury PalletID from `lltreasu` to `councilc`, which changes the derived account address:

- **Old Treasury Account**: `5EYCAe5hveooUENA5d7dwq3caqM4LLBzktNumMKmhNRXu4JE` (PalletID: `lltreasu`)
- **New Treasury Account**: `5EYCAe5g8CDuMsTief7QBxfvzDFEfws6ueXTUhsbx5V81nGH` (PalletID: `councilc`)

#### Vulnerability
When the runtime is upgraded, the code will immediately start using the new treasury account address. **Any LLM tokens remaining in the old treasury account will become permanently inaccessible** unless they are transferred BEFORE the runtime upgrade.

#### Exploitation Scenario
1. Runtime is upgraded with new PalletID
2. Code now references new treasury address (`councilc`)  
3. Old treasury address (`lltreasu`) still contains funds
4. **No code path exists to access funds in old treasury**
5. Funds are permanently locked/lost

#### Evidence from Code
```rust
// substrate/frame/llm/src/lib.rs (lines 599-605)
/// AccountId of **Treasury** account. **Treasury** account stores
/// prereleased amount of LLM on genesis and part of LLM from **Vault**
/// on LLM Release Events.
pub fn get_llm_treasury_account() -> T::AccountId {
-    PalletId(*b"lltreasu").into_account_truncating()
+    PalletId(*b"councilc").into_account_truncating()
}
```

#### PR Description Claims
The PR description (step 2) states:
> "Move the existing merits for senate, into a scheduler call via new sudo that distributes those merits to congress (70%) and senate (30%) after the next election."

However:
- ❌ No migration code in the PR
- ❌ No runtime migration hooks implemented
- ❌ Relies on manual sudo calls BEFORE upgrade
- ⚠️ **Single point of failure**: If manual migration fails or is forgotten, funds are lost permanently

#### Recommendation
**BLOCK MERGE until addressed**. Implement one of:

1. **Automated Runtime Migration** (RECOMMENDED):
   ```rust
   // Add to runtime upgrade logic
   pub struct MigrateTreasuryFunds<T>(PhantomData<T>);
   impl<T: Config> OnRuntimeUpgrade for MigrateTreasuryFunds<T> {
       fn on_runtime_upgrade() -> Weight {
           let old_treasury = PalletId(*b"lltreasu").into_account_truncating();
           let new_treasury = PalletId(*b"councilc").into_account_truncating();
           let balance = Assets::<T>::balance(AssetId::get(), &old_treasury);
           if balance > Zero::zero() {
               Assets::<T>::transfer(AssetId::get(), &old_treasury, &new_treasury, balance, false)
                   .expect("Treasury migration failed");
           }
           // Return appropriate weight
       }
   }
   ```

2. **Manual Pre-Upgrade Checklist** (NOT RECOMMENDED):
   - Document exact steps required
   - Verify migration completed on-chain
   - Include governance vote for confirmation
   - Add emergency rollback plan

---

### 2. ⚠️ HIGH: Centralized Control via PeaceAccordsOrigin

**Severity**: HIGH  
**Impact**: Centralization of treasury control

#### Description
The PR introduces a new `PeaceAccordsOrigin` that replaces `SenateOrigin` for treasury-related functions. This origin is configured as `EnsureRoot`, meaning **only the Root/Sudo account can transfer from treasury**.

#### Vulnerability
```rust
// substrate/bin/node/runtime/src/lib.rs (line 1390)
type PeaceAccordsOrigin = frame_system::EnsureRoot<AccountId>;
```

Three critical functions now require Root instead of Senate:
1. `treasury_llm_transfer` - Transfer from treasury
2. `treasury_llm_transfer_to_politipool` - Transfer to politics pool
3. `treasury_lld_transfer` - Transfer LLD from treasury

#### Before vs After

**Before (v32)**:
- Treasury transfers: Requires Senate majority OR Root
- Decentralized: Multiple senators can approve

**After (v34)**:
- Treasury transfers: Requires ONLY Root
- Centralized: Single account (or 2/2 multisig)

#### Risk Assessment
While the PR description mentions this is "temporary" and describes a 2/2 multisig, the code doesn't enforce this:
- No timelocks for reverting to Senate control
- No checks that PeaceAccordsOrigin is actually a multisig
- Comment says "Temporary" but no expiration mechanism

#### From PR Description (step 6):
> "Sudo handover to 5GX4fJ8YzZbEva8JfihD2yfvxhYLD5wopMMV9jcvaW4TKfnp, a 2/2 multisig of Navid, the Minister of Finance and Dorian, the Secretary of Technology and rebel leader."

#### Concerns:
1. **Single Point of Failure**: If either multisig key is lost, treasury is locked
2. **No Accountability**: Root can transfer without on-chain governance
3. **Centralization Risk**: Goes against blockchain decentralization principles
4. **No Sunset Clause**: "Temporary" but no automatic reversion

#### Recommendation
1. Add sunset clause: Auto-revert to Senate after X blocks
2. Implement timelocked transitions back to decentralized control
3. Add governance parameter to configure PeaceAccordsOrigin
4. Consider using `EitherOfDiverse` to allow both Root AND Senate
5. Document the multisig address and key holders on-chain

---

### 3. ⚠️ MEDIUM-HIGH: Extended Voting Periods - Governance Lockout

**Severity**: MEDIUM-HIGH  
**Impact**: Potential governance paralysis

#### Description
Voting periods massively extended:
- **Regular Voting Period**: 14 days → **70 days** (5x increase)
- **Fast Track Voting**: 3 days → **75 days** (25x increase!)

#### Rationale from PR (step 5.2):
> "increase referendum(both types) time to 70+ days so no referendum can roll back meritocracy and reinstall dictatorship before the next election"

#### Vulnerabilities

##### A. Fast Track is Slower Than Regular
```rust
pub const VotingPeriod: BlockNumber = 70 * DAYS;      // Regular
pub const FastTrackVotingPeriod: BlockNumber = 75 * DAYS;  // "Fast" track
```
**Fast track is now 5 days SLOWER than regular voting** - this is illogical and defeats the purpose of fast track.

##### B. Emergency Response Capability Destroyed
- **Old system**: 3-day fast track for emergencies
- **New system**: 75-day minimum for ANY governance action
- **Risk**: Cannot respond quickly to:
  - Security vulnerabilities
  - Economic attacks
  - Critical bugs requiring emergency patches

##### C. Governance Capture Window
The stated goal is preventing "dictatorship reinstallation" before an election, but:
- 70+ days allows ample time for governance attacks
- Legitimate emergency responses are blocked
- Attackers can prepare 70-day campaigns

#### Recommendation
1. **Keep fast track meaningful**: Set FastTrackVotingPeriod < VotingPeriod
2. **Add emergency sudo bypass**: For critical security issues
3. **Consider tiered approach**:
   - Regular proposals: 70 days (as intended)
   - Fast track (non-critical): 14 days
   - Emergency track (security): 3 days (requires supermajority)
4. **Add governance proposal categorization** to prevent abuse

---

### 4. ⚠️ MEDIUM: Direct Storage Manipulation

**Severity**: MEDIUM  
**Impact**: Bypasses runtime logic and auditing

#### Description
PR description includes direct storage manipulation via sudo:

```
Sudo system setstorage
0xeb380e7bb68d925ba7614cda49c81fb1c8c230bf4f1f1ef47d3e5f6feef8fe6c...
0x8cc3c901
```

#### Concerns
1. **No runtime validation**: Bypasses all safety checks
2. **Opaque operations**: Storage keys not clearly documented
3. **Difficult to audit**: Changes not visible in code
4. **Error-prone**: Typos in hex strings cause irreversible damage
5. **Sets precedent**: Future admins may abuse this pattern

#### Context
These appear to set election/withdraw locks for specific accounts, but:
- No way to verify correctness without deep chain state knowledge
- Could accidentally lock wrong accounts
- No automated tests for these operations

#### Recommendation
1. Implement proper extrinsic calls instead of raw storage manipulation
2. Add runtime functions like `set_election_lock` with proper validation
3. Include comprehensive testing
4. Document what each storage key represents
5. Use try-runtime for validation before mainnet

---

### 5. ⚠️ LOW-MEDIUM: Version Skipping

**Severity**: LOW-MEDIUM  
**Impact**: Confusing version history, possible migration issues

#### Description
Runtime version jumps from v32 to v34, skipping v33:
```rust
-spec_version: 32,
+spec_version: 34,
```

#### PR Description Mentions:
> "4 - Upgrade to v32 (EVM update, not related to the revolution)
> 5 - Upgrade to v33 (Revolution update)"

But the code only shows v34. This suggests:
- v32 may have been deployed
- v33 may have been deployed  
- This is actually v34
- OR version numbers are confused

#### Concerns
1. **Unclear state**: What's actually deployed in production?
2. **Migration tracking**: Runtime migrations rely on version numbers
3. **Audit trail**: Missing versions make audit more difficult

#### Recommendation
1. Document which versions are deployed where
2. Explain why v33 is skipped in code
3. Ensure runtime migrations handle version skipping correctly

---

## Additional Security Concerns

### 6. No Storage Migration Code

The PR changes critical storage account addresses but includes **zero storage migration logic**. All migrations rely on manual intervention.

**Risk**: Human error during manual migration could cause:
- Partial fund transfers
- Incorrect account targeting
- Lost transaction ordering
- Irreversible mistakes

### 7. Senate Asset Ownership Concerns

From PR description (step 3):
> "Remove senate technical ownership from LLM. Senate was never meant to be the TECHNICAL owner of LLM, for example right now it can print infinite merits and freeze them."

**This reveals a SEPARATE critical issue**: Senate can currently "print infinite merits."

**Questions**:
- Is this already being exploited?
- Should this be fixed BEFORE the Peace Accords changes?
- Is there an audit trail of Senate asset operations?

### 8. Test Coverage

The PR modifies test mocks but doesn't add:
- Tests for treasury migration scenarios
- Tests for PeaceAccordsOrigin behavior
- Tests for extended voting periods
- Integration tests for the full upgrade flow

---

## Recommendations Summary

### Must Fix Before Merge (CRITICAL)
1. ✅ **Implement automated treasury migration** or prove manual migration completed
2. ✅ **Add sunset clause** for PeaceAccordsOrigin centralization
3. ✅ **Fix FastTrackVotingPeriod logic** (should be faster than regular)

### Should Fix Before Merge (HIGH)
4. Replace direct storage manipulation with proper extrinsics
5. Add comprehensive test coverage
6. Document version numbering strategy
7. Address Senate asset ownership vulnerability separately

### Consider for Future (MEDIUM)
8. Implement tiered governance voting periods
9. Add emergency response mechanisms
10. Create governance proposal categorization
11. Add monitoring for centralized control abuse

---

## Conclusion

While this PR implements politically-motivated "Peace Accords," it does so with **critical security flaws**:

1. **Treasury migration is a single point of failure** - could lose all funds
2. **Governance is centralized to Root** - removes decentralization guarantees
3. **Fast track voting is broken** - 75 days is not "fast"
4. **Manual operations are error-prone** - no automated safety nets

### Final Verdict: ⛔ **DO NOT MERGE** until critical issues are resolved

The PR requires substantial rework to be production-safe. The current implementation trades security and decentralization for speed, which is inappropriate for a blockchain managing real value.

---

## Audit Trail

- **Initial Review**: 2025-11-12
- **Reviewer**: GitHub Copilot Security Analysis
- **PR**: https://github.com/liberland/liberland_substrate/pull/448
- **Status**: Awaiting fixes

---

## Appendix: Code References

### Treasury Account Change
- `substrate/frame/llm/src/lib.rs`: Lines 602, 305-306, 323, 380
- `substrate/frame/llm/README.md`: Line 36

### Origin Changes  
- `substrate/bin/node/runtime/src/lib.rs`: Line 1390
- `substrate/frame/llm/src/lib.rs`: Lines 179, 305, 323, 380

### Voting Period Changes
- `substrate/bin/node/runtime/src/lib.rs`: Lines 344, 865

### Version Changes
- `substrate/bin/node/runtime/src/lib.rs`: Lines 217, 234
- `Cargo.toml` files: Multiple locations
