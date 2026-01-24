
``` bash
git stash push -u -m "backup_with_untracked" && git checkout main && git add .stable/ && git commit --allow-empty -m "Update stable folder" && git push origin main && git checkout - && git merge main --no-ff -m "Sync stable from main" && git stash pop
```
