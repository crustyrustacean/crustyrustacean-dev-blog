## Summary

This PR adds **Phase 0.5: Theme Foundation** to the project plan based on the insight that theming should be established early to prevent technical debt.

## Changes

### PROJECT_PLAN.md
- Updated to version 1.1.0
- Added Phase 0.5 section between Phase 0 (Quick Wins) and Phase 1 (Content Management)
- Updated timeline from 6-7 months to 7-8 months
- Renumbered all issues from #3-#19 (Phase 0.5 becomes Issue #2)
- Added Phase 0.5 to Success Metrics
- Updated revision history and next actions

### New Phase 0.5: Theme Foundation (1-2 weeks)

**Goals:**
- Move templates to `themes/default/` directory structure
- Create theme metadata system (`theme.toml`)
- Build backend theme loader (`src/lib/theme.rs`)
- Extract reusable components (navbar, footer, card, button, form)
- Create semantic CSS classes abstracting Bootstrap
- Use CSS custom properties for theme variables

**Why This Matters:**

Currently, Bootstrap classes are hardcoded throughout all 22+ templates. If we build Phase 1 features (media library, settings, admin UIs) with hardcoded Bootstrap, we'll need to refactor everything when implementing the full theme system.

By establishing theme foundation NOW:
- ✅ One-time refactor instead of continuous rework
- ✅ All Phase 1 features built theme-agnostic from day one
- ✅ Easy to switch from Bootstrap to Tailwind/custom CSS later
- ✅ Better developer experience with reusable components
- ✅ Aligns with WordPress model (themes are foundational)
- ✅ Future-proof architecture

### New Issue Template

**File:** `.github/issues/phase0/issue-02-theme-foundation.md`

Comprehensive 2-week implementation plan including:
- Week 1: Structure, migration, backend loader
- Week 2: Components, CSS abstraction, documentation
- Complete code examples
- Testing requirements
- Success criteria

## Implementation Order

1. **Phase 0:** Tag Editing (2 hours) - Quick win
2. **Phase 0.5:** Theme Foundation (1-2 weeks) - **NEW** ⭐
3. **Phase 1:** Media Library, Drafts, Roles, Settings (2 months)
4. **Phase 2:** Pages, Categories, Revisions (2 months)
5. **Phase 3:** Full Theme System, Menus, Widgets (2 months)
6. **Phase 4:** Plugins, Advanced Features (ongoing)

## Related Documents

- `TAG_EDITING_PLAN.md` - Existing detailed plan for Phase 0
- `CODEBASE_INDEX.md` - Current codebase documentation
- `.github/issues/phase1/` - Phase 1 issue templates (already created)

## Next Steps

1. Review and merge this PR
2. Create GitHub issue from `.github/issues/phase0/issue-02-theme-foundation.md`
3. Implement tag editing (2 hours)
4. Implement theme foundation (1-2 weeks)
5. Begin Phase 1 with theme-agnostic approach

🤖 Generated with [Claude Code](https://claude.com/claude-code)
