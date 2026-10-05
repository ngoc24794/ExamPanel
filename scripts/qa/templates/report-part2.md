## 7. Prioritised fix backlog (suggested regression test per item)

Hypotheses are marked (H). "Mock gap" = the existing mock suite passes the same step, so the test must use the real service (tauri-driver smoke or service-level test), or a mock that models latency / real-shaped errors.

### Must fix before v0.2.0
| Area | Items | Regression test |
|---|---|---|
| app shell | **RA-001** startup dialogs hang the event loop on Linux (H: `blocking_show()` inside `setup()`); also confirm on Windows | CI job: launch release exe under Xvfb (a) in a read-only folder, (b) with `user_version=99`, (c) with a corrupted DB; assert a dialog window appears within 5 s and the process exits/continues after the button (xdotool); or move the checks before `Builder::run` and unit-test the decision function |
| app shell / packaging | **RA-036** `default = ["dev-tools"]` ships `seed_demo`; **RA-034** `opener:default` lacks `open_path` (data-folder button always errors; H: same on Windows) | tauri-driver smoke: `invoke('seed_demo')` must return `not_supported`; clicking "Mở thư mục dữ liệu" must not show an ACL error; build script uses `--no-default-features` |
| core | **RA-019** candidate Δ computed for the other setter when DB positions ≠ engine order (H: `IncrementalState::new` canonical ordering); **RA-023** spurious H2 after absence (H: availability-aware eligibility used by H2) | core unit tests with a plan whose setter positions are reversed vs teacher-id order: for every candidate and both positions `delta == full evaluate − base`; validator test "absent teacher ⇒ exactly one h5, no h2" |
| service + ui | **RA-011** Cancel saves partial plans (real backend returns Ok, only the mock throws `cancelled`) | service test: cancel flag ⇒ `Err(cancelled)` and `save_optimize_result` never called; tauri-driver test: Cancel at 0.7 s ⇒ plan count unchanged, single toast |
| storage | **RA-031** rotation/ordering by file name, same-second overwrite; **RA-007** no pre-migration backup (should-fix if Q accepts the written spec) | unit tests: 16 backups of 2 reasons with interleaved timestamps ⇒ kept set == newest 10, list sorted by time desc, two backups in one second both survive; opening `v4_synthetic.db` creates a backup |
| ui (editing) | **RA-020** header/“Hợp lệ” not recomputed after replace/swap, invalid swap accepted, cryptic save error; **RA-018** copy not selected (query-invalidation race); **RA-022** stale badge cache | component tests with an async mock (50–200 ms latency, delayed list refetch); Playwright against the real-shaped fake; assert header equals `evaluate_assignments` after each edit and swap creating H1 is blocked |
| multi-subject views/exports | **RA-013** compare, **RA-027** statistics (also truncated to 11 teachers), **RA-028** Excel "Cùng ban với", **RA-009** Excel "Tiêu chí" names/units | Vitest with the 2-subject Q fixture: distance == #changed seats (39), VL/CN panels shown separately, heatmaps 12×12 with names; calamine test comparing both Excel sheets to the engine for the Q plan |
| ui (settings) | **RA-033** language from Settings not persisted | Vitest: Settings buttons call `setLanguage`; e2e restart test |

### Should fix
- **Q-fit:** RA-014 (grid at 1280×720), RA-015 (GK1/CK1 labels, blank second CN cell), RA-017 (S3 constant 144 for one campus), RA-026, RA-029 (invented org defaults), RA-039 (focusable seats, visible focus).
- **i18n:** RA-040 (10 missing keys — add a CI test that every `t('…')` key exists in both locales), RA-042 (hard-coded Vietnamese / English "Close"), RA-002, RA-010, RA-021, RA-025, RA-032 (map engine codes/messages to translated text).
- **Robustness / data:** RA-012 (aggregate progress), RA-024 (ISO dates — fixes WebKitGTK), RA-041 (disable submit while pending), RA-008 (seed_defaults without sequence bump).
- **Docs:** RA-006, RA-037, RA-038, RA-005 (route test artefacts away from tracked docs).

### Could fix
RA-003, RA-004, RA-016, RA-035, RA-030 (in-app "Save as PDF" if Linux/macOS builds are ever supported).

## 8. Hypotheses to confirm
1. RA-001: `blocking_show()` on the main thread also deadlocks (or not) on Windows — manual W5/W1.
2. RA-019: root cause is the setter ordering in `IncrementalState` (candidates for position 0 equal the full Δ of position 1 in every probed case); confirm by logging `old_t_idx`.
3. RA-034: capability `opener:default` has no `allow-open-path` ⇒ same ACL error on Windows.
4. RA-024: WebView2 parses `YYYY-MM-DD HH:MM:SS`; Linux/macOS do not.
5. RA-023: H2 reuses an availability-aware eligibility helper.
6. RA-018/RA-022: React Query invalidation timing (mock resolves synchronously) — try `await queryClient.invalidateQueries` before `setSelectedPlanId`.
7. Windows print: `window.print()` honours `@page` landscape and background graphics (watermark / grey headers) — manual W9.

## 9. Questions for Q (raised by what the app showed)
1. Total penalties include S3 = 144 for a school without campuses (Q's plan 260.36 vs 116.36 without S3). Should S3 be hidden/disabled when there is a single "Chưa phân hiệu" campus?
2. The paper sheet says GK1/CK1/GK2/CK2 — should the on-screen grid and totals use these codes (the Excel export already does) instead of "Giữa kỳ 1"?
3. Is the order of the two setters ("Đề", "Đề") meaningful on paper? The optimiser/engine seems to treat it as unordered (kept seats come back in another order).
4. Statistics/compare: should a "ban đề" mean the per-subject panel (24 panels, VL 3 members, CN 2) or the merged exam×grade? Should T Nghĩa appear in the heatmaps?
5. Printing: which header text, signer name and place should be pre-filled when Settings is empty (today invented text is printed)?
6. Automatic backups: keep the newest 10 overall, or 10 per kind (import / restore / migration)? Is a backup before a version upgrade expected?
7. "Tránh làm chung" (avoid pair) was in the release checklist — is that feature still wanted?
8. Does the school ever run on Linux/macOS? (Decides whether RA-030 and WebKit-specific items matter.)
9. For the real data: is H3 really off (no campus)? Should C Quí's "3 việc trong một kỳ" exception be kept as a hard override, or should the tool warn instead of showing a violation?
