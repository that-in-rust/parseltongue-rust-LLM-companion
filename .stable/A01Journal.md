
# Stable folder synchronized across all branches for PRs Architecture & Design docs
``` bash
git add .stable/ && git commit -m "Update stable folder" && git stash push -u -m "backup_with_untracked" && git checkout main && git add .stable/ && git commit --allow-empty -m "Update stable folder" && git push origin main && git checkout - && git merge main --no-ff -m "Sync stable from main" && git stash pop                   ```

Your .stable folder becomes permanently synchronized across all branches
- any changes you make to stable files are automatically committed to both your current branch AND the main branch
- while preserving all your other work, creating a universal configuration folder that never gets lost when switching between projects
