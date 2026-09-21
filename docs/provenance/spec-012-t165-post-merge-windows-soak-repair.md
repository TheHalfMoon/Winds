# Spec 012 T165 Post-Merge Windows Soak Repair

## Trigger

```text
T165_MERGE=84c10ba7493e5ce797699efca607bb7aa414cd19
POST_MERGE_T160_NATIVE_PLATFORM_RUN=35559526761
POST_MERGE_T160_NATIVE_PLATFORM_RESULT=FAILURE
FAILED_JOB=persistent-runtime (windows)
FAILED_STEP=Run 100-cycle terminal lifecycle resource cleanup
FAILED_TEST=git::t063_soak_tests::active_close_and_windows_child_resize_guard
```

The exact T165 merge passed the direct persistent-runtime qualification, domain evidence, reconnect churn, native Windows ConPTY qualification, and the full 100-cycle T063 lifecycle soak. The supplemental Windows child-resize guard then timed out while waiting for the exact child-observed `WINDS_T063_CHILD_SIZE_33 101` marker. The captured output proves that `cmd.exe` accepted and launched the PowerShell probe, but the marker had not arrived inside the fixture's fixed 10-second wait.

This failure remains negative canonical evidence. It is not relabelled, retried, or discarded.

## Repair scope

The repair changes only `src/t063_soak_tests.rs`.

- Production terminal, ConPTY, multiplexer, owner, persistence, protocol, UI, detector, Git/worktree, and authority behavior are unchanged.
- The default T063 output-marker wait remains 10 seconds.
- Only the hosted-Windows PowerShell child-resize probe receives a 30-second bounded wait.
- The exact child-observed size marker remains mandatory.
- No retry, skip, alternate success condition, dependency, workflow-semantic change, or product behavior is introduced.

The repair addresses hosted-runner PowerShell cold-start variance after the 100-cycle ConPTY soak without weakening the resize proof.

## Governance state

```text
T165=CLOSEOUT_REPAIR
T166..T185=BLOCKED_BY_PREDECESSOR
PROTOCOL_V2_IMPLEMENTATION_AUTHORIZED=NO
AGENT_START_AUTHORIZED=NO
WORKTREE_MUTATION_AUTHORIZED=NO
REMOTE_EXECUTION_AUTHORIZED=NO
PLUGIN_RUNTIME_AUTHORIZED=NO
MARKETPLACE_AUTHORIZED=NO
AUTOMATIC_LANDING_AUTHORIZED=NO
```

T165 may become `CLOSED_CANONICAL` only after this repair is independently reviewed, exact-head qualification succeeds, the repair lands through a guarded normal merge, and every actually-triggered post-merge push workflow succeeds on the resulting canonical `main`.
