# Phase 1 Issue Templates

This directory contains GitHub issue templates for Phase 1 of the WordPress-like CMS transformation project.

## How to Create Issues

Since the GitHub CLI is restricted, you'll need to manually create issues using these templates:

### Option 1: Copy & Paste (Recommended)

1. Go to your GitHub repository: https://github.com/crustyrustacean/crustyrustacean-dev-blog/issues
2. Click "New Issue"
3. Open one of the markdown files below
4. Copy the content (excluding the frontmatter between `---`)
5. Paste into GitHub issue
6. Add the labels, milestone, and assignees from the frontmatter

### Option 2: Upload as Templates

1. Copy these files to `.github/ISSUE_TEMPLATE/`
2. GitHub will automatically recognize them as issue templates
3. Users can select templates when creating new issues

## Phase 1 Issues

### Issue 1: Tag Editing for Articles
- **File:** `issue-01-tag-editing.md`
- **Priority:** CRITICAL (Quick win)
- **Time:** 2 hours
- **Labels:** enhancement, phase-1, quick-win, priority-critical
- **Summary:** Fix tag editing in article updates

### Issue 2: Media Library & Upload System
- **File:** `issue-02-media-library.md`
- **Priority:** HIGH
- **Time:** 3-4 weeks
- **Labels:** enhancement, phase-1, priority-high
- **Summary:** Integrate OpenDAL storage API for media management

### Issue 3: Draft/Publish Workflow
- **File:** `issue-03-draft-publish-workflow.md`
- **Priority:** HIGH
- **Time:** 2 weeks
- **Labels:** enhancement, phase-1, priority-high
- **Summary:** Add article status and scheduled publishing

### Issue 4: User Roles & Permissions
- **File:** `issue-04-user-roles-permissions.md`
- **Priority:** HIGH
- **Time:** 2-3 weeks
- **Labels:** enhancement, phase-1, priority-high
- **Summary:** Implement RBAC with subscriber/author/editor/admin roles

### Issue 5: Settings/Configuration UI
- **File:** `issue-05-settings-ui.md`
- **Priority:** HIGH
- **Time:** 1-2 weeks
- **Labels:** enhancement, phase-1, priority-high
- **Summary:** Database-backed settings with admin UI

## Recommended Order

1. **Issue 1** - Tag Editing (2 hours) - Do this first as a quick win
2. **Issue 2** - Media Library (3-4 weeks) - Foundation for rich content
3. **Issue 3** - Draft/Publish Workflow (2 weeks) - Can be done in parallel with #2
4. **Issue 4** - User Roles & Permissions (2-3 weeks) - Required for multi-user features
5. **Issue 5** - Settings UI (1-2 weeks) - Requires #4 for admin-only access

## Creating All Issues at Once

If you want to create all issues quickly, you can use the GitHub API or create them via the web interface. Here's a quick command reference for when GitHub CLI becomes available:

```bash
# When gh CLI is available:
gh issue create --title "[Phase 1] Tag Editing for Articles" --body-file .github/issues/phase1/issue-01-tag-editing.md --label "enhancement,phase-1,quick-win,priority-critical"

gh issue create --title "[Phase 1] Media Library & Upload System" --body-file .github/issues/phase1/issue-02-media-library.md --label "enhancement,phase-1,priority-high"

gh issue create --title "[Phase 1] Draft/Publish Workflow" --body-file .github/issues/phase1/issue-03-draft-publish-workflow.md --label "enhancement,phase-1,priority-high"

gh issue create --title "[Phase 1] User Roles & Permissions" --body-file .github/issues/phase1/issue-04-user-roles-permissions.md --label "enhancement,phase-1,priority-high"

gh issue create --title "[Phase 1] Settings/Configuration UI" --body-file .github/issues/phase1/issue-05-settings-ui.md --label "enhancement,phase-1,priority-high"
```

## Related Documents

- `PROJECT_PLAN.md` - Full project plan with all phases
- `TAG_EDITING_PLAN.md` - Detailed implementation plan for Issue #1
- `CODEBASE_INDEX.md` - Current codebase documentation

## Labels to Create (if not already existing)

- `enhancement` - New feature or request
- `phase-1` - Part of Phase 1 implementation
- `phase-2` - Part of Phase 2 implementation
- `quick-win` - Can be completed quickly
- `priority-critical` - Must be done ASAP
- `priority-high` - Important feature
- `priority-medium` - Moderate importance
- `priority-low` - Nice to have

## Milestone

Create a milestone called "Phase 1 - Content Management Foundations" to track all Phase 1 issues together.
