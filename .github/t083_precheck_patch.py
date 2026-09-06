from pathlib import Path

path = Path("src/store.rs")
text = path.read_text()
marker = '''        let worktree = PathBuf::from(&persisted.5);
        let timeout_secs = u64::try_from(persisted.7)?;
        let check_run = crate::check::run_check(
'''
replacement = '''        let worktree = PathBuf::from(&persisted.5);
        let head_before = repo.worktree_head(&worktree)?;
        if head_before != persisted.3 {
            return Err("verification worktree HEAD does not match persisted candidate OID".into());
        }
        let tree_before = repo.tree_oid(&head_before)?;
        if tree_before != persisted.4 {
            return Err("verification worktree tree does not match persisted candidate tree".into());
        }
        if !repo.worktree_is_clean(&worktree)? {
            return Err("verification worktree must be clean before the required check".into());
        }

        let timeout_secs = u64::try_from(persisted.7)?;
        let check_run = crate::check::run_check(
'''
assert text.count(marker) == 1, "T083 precheck insertion marker drifted"
text = text.replace(marker, replacement, 1)
path.write_text(text)
