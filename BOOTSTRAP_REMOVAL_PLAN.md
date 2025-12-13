# Bootstrap Removal Plan

## Overview

This plan outlines the systematic removal of Bootstrap 5.1.3 from the CrustyRustacean Dev Blog, transitioning to a modern, standards-compliant CSS and JavaScript architecture built on the recently implemented modular theming system.

### Current State
- **Bootstrap Version**: 5.1.3 (CDN loaded)
- **Bootstrap Class Usage**: ~1,031 instances across 35 HTML templates
- **Bootstrap JS Features Used**: Modals, Dropdowns, Collapse, Alert dismiss
- **Theming System**: Already implemented with CSS custom properties and `@layer` support

### Goals
1. Remove Bootstrap CSS and JS dependencies entirely
2. Implement modern CSS replacements using CSS Grid, Flexbox, and custom properties
3. Create vanilla JavaScript replacements for interactive components
4. Maintain current visual design and functionality
5. Improve performance (smaller bundle, no external CSS framework)

---

## Task Breakdown

### LOW DIFFICULTY TASKS (Quick wins, minimal risk)

#### L1. Remove Bootstrap CDN References
**Files**: `themes/*/theme.toml`, `templates/base.html`
**Effort**: ~30 minutes

Remove the Bootstrap CDN links from all theme configuration files and update base.html to no longer include external Bootstrap stylesheets.

- [ ] Remove `stylesheets = ["https://cdn.jsdelivr.net/npm/bootstrap@5.1.3/dist/css/bootstrap.min.css"]` from all `theme.toml` files
- [ ] Remove Bootstrap JS bundle script tag from `templates/base.html`
- [ ] Update `theme_stylesheet()` Tera function to not return Bootstrap URL

---

#### L2. Replace Bootstrap Spacing Utilities
**Files**: All 35 templates
**Effort**: ~1-2 hours

Replace Bootstrap margin/padding classes with CSS custom properties.

**Bootstrap Classes to Replace**:
- `mb-*`, `mt-*`, `my-*`, `mx-*` (margin)
- `pb-*`, `pt-*`, `py-*`, `px-*` (padding)
- `me-*`, `ms-*`, `pe-*`, `ps-*` (inline margin/padding)

**Implementation**:
```css
/* static/css/utilities.css */
.m-0 { margin: 0; }
.m-1 { margin: var(--space-1, 0.25rem); }
.m-2 { margin: var(--space-2, 0.5rem); }
.m-3 { margin: var(--space-3, 1rem); }
.m-4 { margin: var(--space-4, 1.5rem); }
.m-5 { margin: var(--space-5, 3rem); }
/* ... similar for p-*, mb-*, mt-*, etc. */
```

- [ ] Create `static/css/utilities.css` with spacing utilities
- [ ] Define spacing scale as CSS custom properties in themes
- [ ] Include utilities.css in base.html

---

#### L3. Replace Text Utilities
**Files**: All templates using `text-*` classes
**Effort**: ~1 hour

**Bootstrap Classes to Replace**:
- `text-center`, `text-start`, `text-end`
- `text-muted`
- `text-primary`, `text-secondary`, `text-danger`, etc.
- `text-nowrap`, `text-truncate`
- `fw-bold`, `fw-normal`, `fst-italic`

**Implementation**:
```css
/* static/css/utilities.css */
.text-center { text-align: center; }
.text-start { text-align: left; }
.text-end { text-align: right; }
.text-muted { color: var(--color-text-muted); }
.text-primary { color: var(--color-primary); }
.text-truncate { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.fw-bold { font-weight: 700; }
```

- [ ] Add text utilities to utilities.css
- [ ] Ensure color utilities use theme custom properties

---

#### L4. Replace Display Utilities
**Files**: Templates using `d-*` classes
**Effort**: ~30 minutes

**Bootstrap Classes to Replace**:
- `d-none`, `d-block`, `d-inline`, `d-inline-block`
- `d-flex`, `d-inline-flex`
- `d-grid`
- Responsive variants: `d-md-none`, `d-lg-block`, etc.

**Implementation**:
```css
.d-none { display: none; }
.d-block { display: block; }
.d-flex { display: flex; }
.d-grid { display: grid; }

@media (min-width: 768px) {
  .d-md-none { display: none; }
  .d-md-block { display: block; }
  .d-md-flex { display: flex; }
}
```

- [ ] Add display utilities with responsive variants
- [ ] Use consistent breakpoint variables

---

#### L5. Replace Sizing Utilities
**Files**: Templates using `w-*`, `h-*` classes
**Effort**: ~30 minutes

**Bootstrap Classes to Replace**:
- `w-100`, `w-75`, `w-50`, `w-25`, `w-auto`
- `h-100`, `h-auto`
- `mw-100`, `mh-100`

**Implementation**:
```css
.w-100 { width: 100%; }
.w-auto { width: auto; }
.h-100 { height: 100%; }
.mw-100 { max-width: 100%; }
```

- [ ] Add sizing utilities to utilities.css

---

#### L6. Replace Flex Utilities
**Files**: Templates using flexbox classes
**Effort**: ~1 hour

**Bootstrap Classes to Replace**:
- `flex-row`, `flex-column`
- `flex-wrap`, `flex-nowrap`
- `justify-content-*` (start, end, center, between, around, evenly)
- `align-items-*` (start, end, center, baseline, stretch)
- `align-self-*`
- `flex-grow-*`, `flex-shrink-*`
- `gap-*`

**Implementation**:
```css
.flex-row { flex-direction: row; }
.flex-column { flex-direction: column; }
.justify-content-center { justify-content: center; }
.justify-content-between { justify-content: space-between; }
.align-items-center { align-items: center; }
.gap-1 { gap: var(--space-1); }
.gap-2 { gap: var(--space-2); }
.gap-3 { gap: var(--space-3); }
```

- [ ] Add flexbox utilities to utilities.css
- [ ] Include responsive variants where needed

---

#### L7. Replace Border & Rounded Utilities
**Files**: Templates using `border-*`, `rounded-*` classes
**Effort**: ~30 minutes

**Bootstrap Classes to Replace**:
- `border`, `border-0`, `border-top`, etc.
- `rounded`, `rounded-0`, `rounded-circle`, `rounded-pill`

**Implementation**:
```css
.border { border: 1px solid var(--color-border); }
.border-0 { border: 0; }
.rounded { border-radius: var(--border-radius); }
.rounded-circle { border-radius: 50%; }
.rounded-pill { border-radius: 9999px; }
```

- [ ] Add border utilities to utilities.css

---

#### L8. Replace Shadow Utilities
**Files**: Templates using `shadow-*` classes
**Effort**: ~15 minutes

**Bootstrap Classes to Replace**:
- `shadow`, `shadow-sm`, `shadow-lg`, `shadow-none`

**Implementation**:
```css
.shadow { box-shadow: var(--shadow); }
.shadow-sm { box-shadow: var(--shadow-sm); }
.shadow-lg { box-shadow: var(--shadow-lg); }
.shadow-none { box-shadow: none; }
```

- [ ] Add shadow utilities using theme variables

---

### MEDIUM DIFFICULTY TASKS (Require more careful implementation)

#### M1. Implement Custom Grid System
**Files**: All templates using `container`, `row`, `col-*`
**Effort**: ~3-4 hours

Replace Bootstrap's 12-column grid with CSS Grid-based layout system.

**Bootstrap Classes to Replace**:
- `container`, `container-fluid`, `container-*`
- `row`, `row-cols-*`
- `col`, `col-*`, `col-sm-*`, `col-md-*`, `col-lg-*`, `col-xl-*`
- `g-*`, `gx-*`, `gy-*` (gutters)
- `offset-*`

**Implementation**:
```css
/* static/css/grid.css */
.container {
  width: 100%;
  max-width: var(--container-max-width, 1320px);
  margin-inline: auto;
  padding-inline: var(--container-padding, 1rem);
}

.row {
  display: grid;
  grid-template-columns: repeat(12, 1fr);
  gap: var(--grid-gap, 1.5rem);
}

.col { grid-column: span 12; }
.col-1 { grid-column: span 1; }
.col-2 { grid-column: span 2; }
/* ... */
.col-12 { grid-column: span 12; }

@media (min-width: 768px) {
  .col-md-4 { grid-column: span 4; }
  .col-md-6 { grid-column: span 6; }
  .col-md-8 { grid-column: span 8; }
}

@media (min-width: 992px) {
  .col-lg-3 { grid-column: span 3; }
  .col-lg-4 { grid-column: span 4; }
  .col-lg-8 { grid-column: span 8; }
  .col-lg-9 { grid-column: span 9; }
}
```

- [ ] Create `static/css/grid.css` with CSS Grid-based layout
- [ ] Define breakpoint variables for consistency
- [ ] Support responsive column classes
- [ ] Test all page layouts for grid regressions

---

#### M2. Implement Custom Button Styles
**Files**: All templates using `btn-*` classes
**Effort**: ~2-3 hours

**Bootstrap Classes to Replace**:
- `btn`, `btn-primary`, `btn-secondary`, `btn-success`, `btn-danger`, `btn-warning`, `btn-info`
- `btn-outline-*`
- `btn-link`
- `btn-sm`, `btn-lg`
- `btn-close`
- `btn-group`

**Implementation**:
```css
/* static/css/components/buttons.css */
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  font-weight: 500;
  text-decoration: none;
  border: 1px solid transparent;
  border-radius: var(--border-radius);
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

.btn-primary {
  background: var(--color-primary);
  color: var(--color-text-on-primary);
}

.btn-primary:hover {
  background: var(--color-primary-hover);
}

.btn-outline-primary {
  border-color: var(--color-primary);
  color: var(--color-primary);
  background: transparent;
}

.btn-outline-primary:hover {
  background: var(--color-primary);
  color: var(--color-text-on-primary);
}

.btn-sm { padding: 0.25rem 0.5rem; font-size: 0.875rem; }
.btn-lg { padding: 0.75rem 1.5rem; font-size: 1.125rem; }

.btn-close {
  background: transparent;
  border: 0;
  padding: 0.5rem;
  cursor: pointer;
  opacity: 0.5;
}

.btn-close::before {
  content: '✕';
}

.btn-close:hover { opacity: 1; }
```

- [ ] Create `static/css/components/buttons.css`
- [ ] Implement all button variants using theme colors
- [ ] Add focus states for accessibility
- [ ] Test all button interactions

---

#### M3. Implement Custom Card Component
**Files**: Templates using `card-*` classes
**Effort**: ~2 hours

**Bootstrap Classes to Replace**:
- `card`, `card-body`, `card-header`, `card-footer`
- `card-title`, `card-text`, `card-img-top`
- `card-group`

**Implementation**:
```css
/* static/css/components/cards.css */
.card {
  background: var(--color-bg-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius);
  overflow: hidden;
}

.card-header {
  padding: 1rem;
  background: var(--color-bg-elevated);
  border-bottom: 1px solid var(--color-border);
}

.card-body {
  padding: 1rem;
}

.card-footer {
  padding: 1rem;
  background: var(--color-bg-elevated);
  border-top: 1px solid var(--color-border);
}

.card-title {
  margin-bottom: 0.5rem;
  font-weight: 600;
}

.card-text {
  color: var(--color-text-secondary);
}

.card-img-top {
  width: 100%;
  height: auto;
}
```

- [ ] Create `static/css/components/cards.css`
- [ ] Style cards to match current design
- [ ] Add hover states for clickable cards

---

#### M4. Implement Custom Form Styles
**Files**: Templates with forms (login, register, editor, search, etc.)
**Effort**: ~3-4 hours

**Bootstrap Classes to Replace**:
- `form-control`, `form-select`
- `form-label`
- `form-check`, `form-check-input`, `form-check-label`
- `form-text`
- `input-group`, `input-group-text`
- `form-floating`
- `is-valid`, `is-invalid`
- `invalid-feedback`, `valid-feedback`

**Implementation**:
```css
/* static/css/components/forms.css */
.form-control {
  display: block;
  width: 100%;
  padding: 0.5rem 0.75rem;
  font-size: 1rem;
  line-height: 1.5;
  color: var(--color-text-primary);
  background: var(--color-bg-body);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius);
  transition: border-color 0.2s, box-shadow 0.2s;
}

.form-control:focus {
  border-color: var(--color-primary);
  outline: 0;
  box-shadow: 0 0 0 3px var(--color-primary-alpha);
}

.form-label {
  display: block;
  margin-bottom: 0.5rem;
  font-weight: 500;
}

.form-select {
  display: block;
  width: 100%;
  padding: 0.5rem 2rem 0.5rem 0.75rem;
  background-image: url("data:image/svg+xml,...");
  background-repeat: no-repeat;
  background-position: right 0.75rem center;
  background-size: 16px 12px;
  appearance: none;
}

.is-invalid {
  border-color: var(--color-danger);
}

.invalid-feedback {
  display: none;
  color: var(--color-danger);
  font-size: 0.875rem;
  margin-top: 0.25rem;
}

.is-invalid ~ .invalid-feedback {
  display: block;
}
```

- [ ] Create `static/css/components/forms.css`
- [ ] Style all form elements consistently
- [ ] Implement validation states
- [ ] Test all forms for functionality

---

#### M5. Implement Custom Badge Component
**Files**: Templates using `badge` classes
**Effort**: ~30 minutes

**Bootstrap Classes to Replace**:
- `badge`, `badge-*` color variants
- `rounded-pill` (for badges)

**Implementation**:
```css
/* static/css/components/badges.css */
.badge {
  display: inline-flex;
  align-items: center;
  padding: 0.25em 0.5em;
  font-size: 0.75em;
  font-weight: 600;
  line-height: 1;
  border-radius: var(--border-radius-sm);
}

.badge-primary { background: var(--color-primary); color: var(--color-text-on-primary); }
.badge-secondary { background: var(--color-secondary); color: white; }
.badge-success { background: var(--color-success); color: white; }
.badge-danger { background: var(--color-danger); color: white; }
```

- [ ] Create badge styles in components CSS
- [ ] Support all color variants

---

#### M6. Implement Custom Alert Component
**Files**: Templates using `alert-*` classes (flash messages)
**Effort**: ~1 hour

**Bootstrap Classes to Replace**:
- `alert`, `alert-*` color variants
- `alert-dismissible`
- `alert-heading`, `alert-link`

**Implementation**:
```css
/* static/css/components/alerts.css */
.alert {
  position: relative;
  padding: 1rem;
  margin-bottom: 1rem;
  border: 1px solid transparent;
  border-radius: var(--border-radius);
}

.alert-success {
  color: var(--color-success-text);
  background: var(--color-success-bg);
  border-color: var(--color-success-border);
}

.alert-danger {
  color: var(--color-danger-text);
  background: var(--color-danger-bg);
  border-color: var(--color-danger-border);
}

.alert-dismissible {
  padding-right: 3rem;
}

.alert-dismissible .btn-close {
  position: absolute;
  top: 0;
  right: 0;
  padding: 1rem;
}
```

- [ ] Create alert styles with dismissible support
- [ ] Wire up dismiss functionality in vanilla JS

---

#### M7. Implement Custom Pagination Component
**Files**: `articles/list.html`, `admin/dashboard.html`
**Effort**: ~1-2 hours

**Bootstrap Classes to Replace**:
- `pagination`
- `page-item`, `page-link`
- `active`, `disabled` states

**Implementation**:
```css
/* static/css/components/pagination.css */
.pagination {
  display: flex;
  list-style: none;
  padding: 0;
  margin: 0;
  gap: 0.25rem;
}

.page-item {
  display: inline-flex;
}

.page-link {
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 2.5rem;
  height: 2.5rem;
  padding: 0.5rem;
  text-decoration: none;
  color: var(--color-text-primary);
  background: var(--color-bg-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-sm);
  transition: all 0.2s;
}

.page-link:hover {
  background: var(--color-bg-elevated);
}

.page-item.active .page-link {
  background: var(--color-primary);
  color: var(--color-text-on-primary);
  border-color: var(--color-primary);
}

.page-item.disabled .page-link {
  opacity: 0.5;
  pointer-events: none;
}
```

- [ ] Create pagination component styles
- [ ] Test pagination on articles list and admin pages

---

#### M8. Implement Custom Table Styles
**Files**: Admin templates with tables
**Effort**: ~1 hour

**Bootstrap Classes to Replace**:
- `table`, `table-striped`, `table-hover`
- `table-responsive`
- `thead-*`, `tbody-*`

**Implementation**:
```css
/* static/css/components/tables.css */
.table {
  width: 100%;
  border-collapse: collapse;
}

.table th,
.table td {
  padding: 0.75rem;
  text-align: left;
  border-bottom: 1px solid var(--color-border);
}

.table thead th {
  font-weight: 600;
  background: var(--color-bg-elevated);
}

.table-striped tbody tr:nth-child(odd) {
  background: var(--color-bg-surface);
}

.table-hover tbody tr:hover {
  background: var(--color-bg-elevated);
}

.table-responsive {
  overflow-x: auto;
}
```

- [ ] Create table component styles
- [ ] Ensure responsive behavior on mobile

---

### HIGH DIFFICULTY TASKS (Complex, require careful testing)

#### H1. Implement Custom Navbar Component
**Files**: `templates/base.html`, `static/js/navbar.js`
**Effort**: ~6-8 hours

The navbar is the most complex Bootstrap component in use. It requires:
- Responsive collapse behavior
- Dropdown menus
- Mobile hamburger toggle
- Search form integration

**Bootstrap Classes to Replace**:
- `navbar`, `navbar-expand-lg`, `navbar-dark`, `navbar-light`
- `navbar-brand`, `navbar-nav`, `nav-item`, `nav-link`
- `navbar-toggler`, `navbar-toggler-icon`
- `navbar-collapse`, `collapse`
- All dropdown classes used in navbar

**Implementation**:

```css
/* static/css/components/navbar.css */
.navbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 1rem;
  background: var(--color-bg-surface);
}

.navbar-brand {
  font-size: 1.25rem;
  font-weight: 700;
  text-decoration: none;
  color: var(--color-primary);
}

.navbar-nav {
  display: flex;
  flex-direction: column;
  list-style: none;
  padding: 0;
  margin: 0;
}

@media (min-width: 992px) {
  .navbar-nav {
    flex-direction: row;
    gap: 0.5rem;
  }
}

.nav-link {
  display: block;
  padding: 0.5rem 1rem;
  text-decoration: none;
  color: var(--color-text-secondary);
  transition: color 0.2s;
}

.nav-link:hover,
.nav-link.active {
  color: var(--color-primary);
}

.navbar-toggler {
  display: flex;
  padding: 0.5rem;
  background: transparent;
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-sm);
  cursor: pointer;
}

@media (min-width: 992px) {
  .navbar-toggler {
    display: none;
  }
}

.navbar-collapse {
  display: none;
  width: 100%;
  flex-basis: 100%;
}

.navbar-collapse.show {
  display: block;
}

@media (min-width: 992px) {
  .navbar-collapse {
    display: flex !important;
    flex-basis: auto;
    width: auto;
  }
}
```

```javascript
// static/js/navbar.js
class Navbar {
  constructor() {
    this.toggler = document.querySelector('.navbar-toggler');
    this.collapse = document.querySelector('.navbar-collapse');

    if (this.toggler && this.collapse) {
      this.toggler.addEventListener('click', () => this.toggle());
    }
  }

  toggle() {
    this.collapse.classList.toggle('show');
    const isExpanded = this.collapse.classList.contains('show');
    this.toggler.setAttribute('aria-expanded', isExpanded);
  }
}

document.addEventListener('DOMContentLoaded', () => new Navbar());
```

- [ ] Create `static/css/components/navbar.css`
- [ ] Create `static/js/navbar.js` with toggle functionality
- [ ] Update `templates/base.html` to use new structure
- [ ] Test on all screen sizes
- [ ] Ensure keyboard accessibility

---

#### H2. Implement Custom Modal Component
**Files**: All templates with modals, `static/js/modal.js`
**Effort**: ~6-8 hours

There are 10+ modals across the application that need custom implementation.

**Bootstrap Classes to Replace**:
- `modal`, `modal-dialog`, `modal-content`
- `modal-header`, `modal-body`, `modal-footer`
- `modal-title`
- `modal-lg`, `modal-xl`, `modal-sm`
- `modal-backdrop`, `fade`, `show`

**Bootstrap Data Attributes to Replace**:
- `data-bs-toggle="modal"`
- `data-bs-target="#modalId"`
- `data-bs-dismiss="modal"`

**Implementation**:

```css
/* static/css/components/modal.css */
.modal {
  position: fixed;
  inset: 0;
  z-index: 1050;
  display: none;
  align-items: center;
  justify-content: center;
  padding: 1rem;
  overflow-y: auto;
}

.modal.show {
  display: flex;
}

.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 1040;
  background: rgba(0, 0, 0, 0.5);
  opacity: 0;
  transition: opacity 0.15s;
}

.modal-backdrop.show {
  opacity: 1;
}

.modal-dialog {
  position: relative;
  width: 100%;
  max-width: 500px;
  margin: auto;
  z-index: 1060;
  transform: translateY(-20px);
  opacity: 0;
  transition: transform 0.3s, opacity 0.3s;
}

.modal.show .modal-dialog {
  transform: translateY(0);
  opacity: 1;
}

.modal-lg .modal-dialog { max-width: 800px; }
.modal-xl .modal-dialog { max-width: 1140px; }

.modal-content {
  background: var(--color-bg-body);
  border-radius: var(--border-radius);
  box-shadow: var(--shadow-lg);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem;
  border-bottom: 1px solid var(--color-border);
}

.modal-title {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 600;
}

.modal-body {
  padding: 1rem;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  padding: 1rem;
  border-top: 1px solid var(--color-border);
}
```

```javascript
// static/js/modal.js
class Modal {
  static instances = new Map();

  constructor(element) {
    this.element = element;
    this.backdrop = null;
    this.isShown = false;

    // Handle close buttons
    element.querySelectorAll('[data-dismiss="modal"]').forEach(btn => {
      btn.addEventListener('click', () => this.hide());
    });

    // Close on backdrop click
    element.addEventListener('click', (e) => {
      if (e.target === element) this.hide();
    });

    // Close on Escape key
    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape' && this.isShown) this.hide();
    });
  }

  static getInstance(element) {
    if (!Modal.instances.has(element)) {
      Modal.instances.set(element, new Modal(element));
    }
    return Modal.instances.get(element);
  }

  show() {
    // Create backdrop
    this.backdrop = document.createElement('div');
    this.backdrop.className = 'modal-backdrop';
    document.body.appendChild(this.backdrop);

    // Show modal
    this.element.classList.add('show');
    document.body.style.overflow = 'hidden';

    // Trigger reflow for animation
    requestAnimationFrame(() => {
      this.backdrop.classList.add('show');
    });

    this.isShown = true;
    this.element.dispatchEvent(new CustomEvent('shown.modal'));
  }

  hide() {
    this.element.classList.remove('show');
    this.backdrop?.classList.remove('show');
    document.body.style.overflow = '';

    setTimeout(() => {
      this.backdrop?.remove();
      this.backdrop = null;
    }, 150);

    this.isShown = false;
    this.element.dispatchEvent(new CustomEvent('hidden.modal'));
  }

  toggle() {
    this.isShown ? this.hide() : this.show();
  }
}

// Auto-initialize modal triggers
document.addEventListener('DOMContentLoaded', () => {
  document.querySelectorAll('[data-toggle="modal"]').forEach(trigger => {
    const targetId = trigger.dataset.target;
    const modal = document.querySelector(targetId);

    if (modal) {
      trigger.addEventListener('click', (e) => {
        e.preventDefault();
        Modal.getInstance(modal).show();
      });
    }
  });
});

// Global API for programmatic access
window.Modal = Modal;
```

- [ ] Create `static/css/components/modal.css`
- [ ] Create `static/js/modal.js` with full functionality
- [ ] Update all templates to use `data-toggle`/`data-target` instead of `data-bs-*`
- [ ] Test all 10+ modals across the application
- [ ] Ensure accessibility (focus trap, ARIA attributes)
- [ ] Test keyboard navigation

**Modals to test**:
1. Search modal (`searchModal`)
2. API key create modal (`createKeyModal`)
3. Category modal (`categoryModal`)
4. Tag modal (`tagModal`)
5. Draft publish modal (`publishModal`)
6. Media upload modal
7. User edit modal
8. Newsletter issue editor modal
9. Confirmation dialogs
10. Article preview modal

---

#### H3. Implement Custom Dropdown Component
**Files**: `templates/base.html`, `static/js/dropdown.js`
**Effort**: ~4-6 hours

Dropdowns are used in the navbar for user menu and admin menus.

**Bootstrap Classes to Replace**:
- `dropdown`, `dropup`, `dropstart`, `dropend`
- `dropdown-toggle`
- `dropdown-menu`, `dropdown-menu-end`
- `dropdown-item`, `dropdown-divider`

**Bootstrap Data Attributes to Replace**:
- `data-bs-toggle="dropdown"`

**Implementation**:

```css
/* static/css/components/dropdown.css */
.dropdown {
  position: relative;
  display: inline-flex;
}

.dropdown-toggle {
  cursor: pointer;
}

.dropdown-toggle::after {
  content: '';
  display: inline-block;
  margin-left: 0.5rem;
  border-top: 0.3em solid;
  border-right: 0.3em solid transparent;
  border-left: 0.3em solid transparent;
  vertical-align: middle;
}

.dropdown-menu {
  position: absolute;
  top: 100%;
  left: 0;
  z-index: 1000;
  display: none;
  min-width: 10rem;
  padding: 0.5rem 0;
  margin: 0.125rem 0 0;
  background: var(--color-bg-body);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius);
  box-shadow: var(--shadow);
}

.dropdown-menu.show {
  display: block;
}

.dropdown-menu-end {
  left: auto;
  right: 0;
}

.dropdown-item {
  display: block;
  width: 100%;
  padding: 0.5rem 1rem;
  color: var(--color-text-primary);
  text-decoration: none;
  background: transparent;
  border: 0;
  cursor: pointer;
  text-align: left;
}

.dropdown-item:hover {
  background: var(--color-bg-elevated);
}

.dropdown-divider {
  height: 0;
  margin: 0.5rem 0;
  border-top: 1px solid var(--color-border);
}
```

```javascript
// static/js/dropdown.js
class Dropdown {
  constructor(toggle) {
    this.toggle = toggle;
    this.menu = toggle.nextElementSibling;
    this.isOpen = false;

    toggle.addEventListener('click', (e) => {
      e.preventDefault();
      e.stopPropagation();
      this.isOpen ? this.close() : this.open();
    });

    // Close on outside click
    document.addEventListener('click', (e) => {
      if (this.isOpen && !this.toggle.contains(e.target) && !this.menu.contains(e.target)) {
        this.close();
      }
    });

    // Close on Escape
    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape' && this.isOpen) this.close();
    });
  }

  open() {
    // Close other dropdowns
    document.querySelectorAll('.dropdown-menu.show').forEach(menu => {
      menu.classList.remove('show');
    });

    this.menu.classList.add('show');
    this.toggle.setAttribute('aria-expanded', 'true');
    this.isOpen = true;
  }

  close() {
    this.menu.classList.remove('show');
    this.toggle.setAttribute('aria-expanded', 'false');
    this.isOpen = false;
  }
}

// Auto-initialize
document.addEventListener('DOMContentLoaded', () => {
  document.querySelectorAll('[data-toggle="dropdown"]').forEach(toggle => {
    new Dropdown(toggle);
  });
});
```

- [ ] Create `static/css/components/dropdown.css`
- [ ] Create `static/js/dropdown.js`
- [ ] Update templates to use `data-toggle="dropdown"` instead of `data-bs-toggle`
- [ ] Test user profile dropdown
- [ ] Test admin navigation dropdowns
- [ ] Ensure keyboard accessibility

---

#### H4. Implement Custom Collapse Component
**Files**: Navbar mobile menu, potentially other collapsible sections
**Effort**: ~2-3 hours

**Bootstrap Classes to Replace**:
- `collapse`
- `collapsing` (animation state)
- `show`

**Bootstrap Data Attributes to Replace**:
- `data-bs-toggle="collapse"`
- `data-bs-target="#id"`

**Implementation**:

```css
/* static/css/components/collapse.css */
.collapse {
  display: none;
}

.collapse.show {
  display: block;
}

.collapsing {
  overflow: hidden;
  height: 0;
  transition: height 0.35s ease;
}
```

```javascript
// static/js/collapse.js
class Collapse {
  constructor(element) {
    this.element = element;
    this.isShown = element.classList.contains('show');
  }

  show() {
    this.element.classList.add('show');
    this.isShown = true;
  }

  hide() {
    this.element.classList.remove('show');
    this.isShown = false;
  }

  toggle() {
    this.isShown ? this.hide() : this.show();
  }
}

// Auto-initialize
document.addEventListener('DOMContentLoaded', () => {
  document.querySelectorAll('[data-toggle="collapse"]').forEach(trigger => {
    const targetId = trigger.dataset.target;
    const target = document.querySelector(targetId);

    if (target) {
      const collapse = new Collapse(target);
      trigger.addEventListener('click', (e) => {
        e.preventDefault();
        collapse.toggle();
      });
    }
  });
});
```

- [ ] Create collapse CSS and JS
- [ ] Test navbar mobile collapse
- [ ] Test any other collapsible sections

---

#### H5. Update All JavaScript Files
**Files**: All 10+ JS files that use Bootstrap data attributes
**Effort**: ~4-6 hours

Update all JavaScript files to use the new custom component APIs instead of Bootstrap's.

**Files to Update**:
- `drafts.js`
- `article.js`
- `search.js`
- `users-admin.js`
- `article-page.js`
- `tags-admin.js`
- `categories-admin.js`
- `admin.js`
- `media-library.js`
- `api-keys.js`

**Changes Required**:
1. Replace `data-bs-toggle` → `data-toggle`
2. Replace `data-bs-target` → `data-target`
3. Replace `data-bs-dismiss` → `data-dismiss`
4. Update any Bootstrap JS API calls to use custom component APIs
5. Replace `new bootstrap.Modal(element)` → `Modal.getInstance(element)`

- [ ] Audit all JS files for Bootstrap dependencies
- [ ] Update data attributes in templates
- [ ] Update JS API calls
- [ ] Test all interactive features

---

#### H6. Update All 35 HTML Templates
**Files**: All templates in `/templates/`
**Effort**: ~8-12 hours

Systematic update of all templates to use new class names and data attributes.

**Process for each template**:
1. Replace Bootstrap utility classes with custom equivalents
2. Update data attributes from `data-bs-*` to `data-*`
3. Ensure component structure matches new CSS expectations
4. Test visual appearance
5. Test all interactive features

- [ ] `base.html` - Main layout (navbar, footer, modals)
- [ ] `index.html` - Homepage
- [ ] `articles/list.html` - Articles listing
- [ ] `articles/article.html` - Article detail
- [ ] `articles/editor.html` - Article editor
- [ ] `auth/login.html` - Login page
- [ ] `auth/register.html` - Registration page
- [ ] `admin/dashboard.html` - Admin dashboard
- [ ] `admin/categories.html` - Categories management
- [ ] `admin/drafts.html` - Drafts management
- [ ] `admin/media.html` - Media library
- [ ] `admin/newsletters.html` - Newsletter management
- [ ] `admin/users.html` - User management
- [ ] `profile/profile.html` - User profiles
- [ ] `profile/favorites.html` - Favorites page
- [ ] `profile/authors.html` - Authors discovery
- [ ] `newsletter/subscribe.html` - Newsletter subscription
- [ ] `newsletter/confirmed.html` - Subscription confirmation
- [ ] `newsletter/unsubscribed.html` - Unsubscribe confirmation
- [ ] All remaining templates (errors, API keys, debug, tags, etc.)

---

#### H7. Comprehensive Testing & QA
**Effort**: ~4-6 hours

Thorough testing of all functionality after Bootstrap removal.

- [ ] Run all 323 integration tests
- [ ] Manual testing of all pages on desktop
- [ ] Manual testing of all pages on mobile (responsive)
- [ ] Test all forms (login, register, editor, search, settings)
- [ ] Test all modals (10+ modals)
- [ ] Test all dropdowns
- [ ] Test navbar collapse on mobile
- [ ] Test pagination
- [ ] Test flash messages / alerts
- [ ] Cross-browser testing (Chrome, Firefox, Safari, Edge)
- [ ] Accessibility audit (keyboard navigation, screen readers)
- [ ] Performance comparison (page load, CSS size)

---

## Implementation Order

### Recommended Sequence

**Phase 1: Foundation (Days 1-2)**
1. L2-L8: Create utilities.css with all utility classes
2. M1: Implement grid system

**Phase 2: Components (Days 3-5)**
3. M2: Button styles
4. M3: Card styles
5. M4: Form styles
6. M5-M8: Badge, Alert, Pagination, Table styles

**Phase 3: Complex Components (Days 6-8)**
7. H3: Dropdown component (needed for navbar)
8. H4: Collapse component (needed for navbar)
9. H1: Navbar component
10. H2: Modal component

**Phase 4: Integration (Days 9-11)**
11. H5: Update all JavaScript files
12. H6: Update all HTML templates
13. L1: Remove Bootstrap CDN references (final step)

**Phase 5: QA (Days 12-13)**
14. H7: Comprehensive testing

---

## File Structure After Migration

```
static/
├── css/
│   ├── base.css              # Theme-aware base styles (existing)
│   ├── utilities.css         # NEW: Spacing, text, display, flex utilities
│   ├── grid.css              # NEW: CSS Grid layout system
│   ├── styles.css            # Existing custom styles
│   └── components/
│       ├── navbar.css        # NEW
│       ├── buttons.css       # NEW
│       ├── cards.css         # NEW
│       ├── forms.css         # NEW
│       ├── modal.css         # NEW
│       ├── dropdown.css      # NEW
│       ├── collapse.css      # NEW
│       ├── alerts.css        # NEW
│       ├── badges.css        # NEW
│       ├── pagination.css    # NEW
│       └── tables.css        # NEW
├── js/
│   ├── navbar.js             # NEW
│   ├── modal.js              # NEW
│   ├── dropdown.js           # NEW
│   ├── collapse.js           # NEW
│   ├── theme-switcher.js     # Existing
│   └── ... (existing JS files, updated)
```

---

## Benefits After Migration

1. **Smaller Bundle Size**: No 160KB Bootstrap CSS, custom CSS ~30-50KB
2. **Better Performance**: Faster page loads, less CSS parsing
3. **Full Control**: No framework constraints, easier customization
4. **Modern CSS**: Using latest features (CSS Grid, @layer, color-mix)
5. **Consistent Theming**: All components use CSS custom properties
6. **Maintainability**: Clear component structure, no Bootstrap version lock-in
7. **Accessibility**: Custom components with proper ARIA support

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Visual regressions | Screenshot comparison testing, incremental rollout |
| Missing functionality | Thorough audit of Bootstrap features in use |
| Browser compatibility | Use fallbacks for modern CSS features |
| Accessibility issues | Audit ARIA attributes, keyboard navigation |
| Extended timeline | Prioritize critical components, defer edge cases |

---

## Success Criteria

- [ ] All 323 integration tests pass
- [ ] No Bootstrap CSS or JS loaded
- [ ] Visual appearance matches current design
- [ ] All interactive features work (modals, dropdowns, collapse)
- [ ] Mobile responsive design works correctly
- [ ] Theme switching continues to work
- [ ] Page load performance improved or maintained
- [ ] Cross-browser compatibility maintained
