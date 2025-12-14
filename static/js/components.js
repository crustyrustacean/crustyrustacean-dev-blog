/**
 * Vanilla JS Components
 * Replacement for Bootstrap JavaScript components
 */

// ============================================
// MODAL COMPONENT
// ============================================

class Modal {
    constructor(element) {
        if (typeof element === 'string') {
            this.element = document.querySelector(element);
        } else {
            this.element = element;
        }

        if (!this.element) {
            console.warn('Modal element not found');
            return;
        }

        this.dialog = this.element.querySelector('.modal-dialog');
        this.isStatic = this.element.dataset.bsBackdrop === 'static';
        this.keyboardDisabled = this.element.dataset.bsKeyboard === 'false';
        this._isShown = false;
        this._init();
    }

    _init() {
        // Close on backdrop click (unless static)
        this.element.addEventListener('click', (e) => {
            if (e.target === this.element && !this.isStatic) {
                this.hide();
            }
        });

        // Close on Escape key (unless disabled)
        this._escapeHandler = (e) => {
            if (e.key === 'Escape' && this._isShown && !this.keyboardDisabled) {
                this.hide();
            }
        };

        // Close button handlers
        this.element.querySelectorAll('[data-bs-dismiss="modal"]').forEach(btn => {
            btn.addEventListener('click', () => this.hide());
        });
    }

    show() {
        if (this._isShown) return;

        // Dispatch show event
        const showEvent = new CustomEvent('show.bs.modal', { cancelable: true });
        this.element.dispatchEvent(showEvent);
        if (showEvent.defaultPrevented) return;

        this._isShown = true;
        document.body.style.overflow = 'hidden';
        document.body.classList.add('modal-open');

        this.element.style.display = 'block';
        this.element.setAttribute('aria-modal', 'true');
        this.element.setAttribute('role', 'dialog');
        this.element.removeAttribute('aria-hidden');

        // Add escape key listener
        document.addEventListener('keydown', this._escapeHandler);

        // Trigger reflow for animation
        this.element.offsetHeight;

        this.element.classList.add('show');

        // Focus the modal
        const focusable = this.element.querySelector('button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])');
        if (focusable) {
            focusable.focus();
        }

        // Dispatch shown event
        setTimeout(() => {
            this.element.dispatchEvent(new CustomEvent('shown.bs.modal'));
        }, 300);
    }

    hide() {
        if (!this._isShown) return;

        // Dispatch hide event
        const hideEvent = new CustomEvent('hide.bs.modal', { cancelable: true });
        this.element.dispatchEvent(hideEvent);
        if (hideEvent.defaultPrevented) return;

        this._isShown = false;
        this.element.classList.remove('show');

        document.removeEventListener('keydown', this._escapeHandler);

        setTimeout(() => {
            this.element.style.display = 'none';
            this.element.setAttribute('aria-hidden', 'true');
            this.element.removeAttribute('aria-modal');
            this.element.removeAttribute('role');

            document.body.style.overflow = '';
            document.body.classList.remove('modal-open');

            this.element.dispatchEvent(new CustomEvent('hidden.bs.modal'));
        }, 300);
    }

    toggle() {
        if (this._isShown) {
            this.hide();
        } else {
            this.show();
        }
    }

    dispose() {
        document.removeEventListener('keydown', this._escapeHandler);
        this._isShown = false;
    }

    // Static methods for Bootstrap compatibility
    static getInstance(element) {
        if (typeof element === 'string') {
            element = document.querySelector(element);
        }
        return element?._modalInstance || null;
    }

    static getOrCreateInstance(element) {
        let instance = Modal.getInstance(element);
        if (!instance) {
            instance = new Modal(element);
            if (typeof element === 'string') {
                element = document.querySelector(element);
            }
            if (element) {
                element._modalInstance = instance;
            }
        }
        return instance;
    }
}

// ============================================
// DROPDOWN COMPONENT
// ============================================

class Dropdown {
    constructor(element) {
        if (typeof element === 'string') {
            this.element = document.querySelector(element);
        } else {
            this.element = element;
        }

        if (!this.element) return;

        this.menu = this.element.nextElementSibling;
        if (!this.menu || !this.menu.classList.contains('dropdown-menu')) {
            this.menu = this.element.parentElement.querySelector('.dropdown-menu');
        }

        this._isShown = false;
        this._init();
    }

    _init() {
        this.element.addEventListener('click', (e) => {
            e.preventDefault();
            e.stopPropagation();
            this.toggle();
        });

        // Close when clicking outside
        this._outsideClickHandler = (e) => {
            if (this._isShown && !this.element.contains(e.target) && !this.menu.contains(e.target)) {
                this.hide();
            }
        };

        document.addEventListener('click', this._outsideClickHandler);

        // Close on Escape
        this._escapeHandler = (e) => {
            if (e.key === 'Escape' && this._isShown) {
                this.hide();
                this.element.focus();
            }
        };

        document.addEventListener('keydown', this._escapeHandler);
    }

    show() {
        if (this._isShown || !this.menu) return;

        // Close other dropdowns
        document.querySelectorAll('.dropdown-menu.show').forEach(menu => {
            menu.classList.remove('show');
        });

        this._isShown = true;
        this.menu.classList.add('show');
        this.element.setAttribute('aria-expanded', 'true');
    }

    hide() {
        if (!this._isShown || !this.menu) return;

        this._isShown = false;
        this.menu.classList.remove('show');
        this.element.setAttribute('aria-expanded', 'false');
    }

    toggle() {
        if (this._isShown) {
            this.hide();
        } else {
            this.show();
        }
    }

    dispose() {
        document.removeEventListener('click', this._outsideClickHandler);
        document.removeEventListener('keydown', this._escapeHandler);
    }
}

// ============================================
// COLLAPSE COMPONENT (for Navbar)
// ============================================

class Collapse {
    constructor(element) {
        if (typeof element === 'string') {
            this.element = document.querySelector(element);
        } else {
            this.element = element;
        }

        if (!this.element) return;

        this._isShown = this.element.classList.contains('show');
    }

    show() {
        if (this._isShown) return;

        this._isShown = true;
        this.element.classList.add('show');

        // Update toggle button aria
        const toggles = document.querySelectorAll(`[data-bs-target="#${this.element.id}"]`);
        toggles.forEach(toggle => {
            toggle.setAttribute('aria-expanded', 'true');
            toggle.classList.remove('collapsed');
        });
    }

    hide() {
        if (!this._isShown) return;

        this._isShown = false;
        this.element.classList.remove('show');

        // Update toggle button aria
        const toggles = document.querySelectorAll(`[data-bs-target="#${this.element.id}"]`);
        toggles.forEach(toggle => {
            toggle.setAttribute('aria-expanded', 'false');
            toggle.classList.add('collapsed');
        });
    }

    toggle() {
        if (this._isShown) {
            this.hide();
        } else {
            this.show();
        }
    }
}

// ============================================
// TAB COMPONENT
// ============================================

class Tab {
    constructor(element) {
        if (typeof element === 'string') {
            this.element = document.querySelector(element);
        } else {
            this.element = element;
        }

        if (!this.element) return;

        this._init();
    }

    _init() {
        this.element.addEventListener('click', (e) => {
            e.preventDefault();
            this.show();
        });
    }

    show() {
        const target = this.element.getAttribute('href') || this.element.dataset.bsTarget;
        if (!target) return;

        const targetPane = document.querySelector(target);
        if (!targetPane) return;

        // Get parent tab list
        const tabList = this.element.closest('.nav-tabs, .nav-pills');

        // Deactivate all tabs
        if (tabList) {
            tabList.querySelectorAll('.nav-link').forEach(link => {
                link.classList.remove('active');
                link.setAttribute('aria-selected', 'false');
            });
        }

        // Deactivate all panes
        const tabContent = targetPane.closest('.tab-content');
        if (tabContent) {
            tabContent.querySelectorAll('.tab-pane').forEach(pane => {
                pane.classList.remove('show', 'active');
            });
        }

        // Activate this tab
        this.element.classList.add('active');
        this.element.setAttribute('aria-selected', 'true');

        // Activate target pane
        targetPane.classList.add('show', 'active');
    }
}

// ============================================
// ALERT COMPONENT
// ============================================

class Alert {
    constructor(element) {
        if (typeof element === 'string') {
            this.element = document.querySelector(element);
        } else {
            this.element = element;
        }

        if (!this.element) return;
    }

    close() {
        this.element.classList.remove('show');

        setTimeout(() => {
            this.element.remove();
        }, 150);
    }

    static close(element) {
        new Alert(element).close();
    }
}

// ============================================
// AUTO-INITIALIZATION
// ============================================

document.addEventListener('DOMContentLoaded', function() {
    // Initialize data-bs-toggle="modal" triggers
    document.querySelectorAll('[data-bs-toggle="modal"]').forEach(trigger => {
        trigger.addEventListener('click', function(e) {
            e.preventDefault();
            const target = this.dataset.bsTarget || this.getAttribute('href');
            if (target) {
                const modal = Modal.getOrCreateInstance(target);
                modal.show();
            }
        });
    });

    // Initialize data-bs-toggle="collapse" triggers
    document.querySelectorAll('[data-bs-toggle="collapse"]').forEach(trigger => {
        trigger.addEventListener('click', function(e) {
            e.preventDefault();
            const target = this.dataset.bsTarget || this.getAttribute('href');
            if (target) {
                const targetEl = document.querySelector(target);
                if (targetEl) {
                    const collapse = new Collapse(targetEl);
                    collapse.toggle();
                }
            }
        });
    });

    // Initialize data-bs-toggle="dropdown" triggers
    document.querySelectorAll('[data-bs-toggle="dropdown"]').forEach(trigger => {
        new Dropdown(trigger);
    });

    // Initialize data-bs-toggle="tab" triggers
    document.querySelectorAll('[data-bs-toggle="tab"]').forEach(trigger => {
        new Tab(trigger);
    });

    // Initialize data-bs-dismiss="alert" triggers
    document.querySelectorAll('[data-bs-dismiss="alert"]').forEach(trigger => {
        trigger.addEventListener('click', function() {
            const alert = this.closest('.alert');
            if (alert) {
                Alert.close(alert);
            }
        });
    });
});

// ============================================
// BOOTSTRAP COMPATIBILITY LAYER
// ============================================

// Create bootstrap namespace for compatibility
window.bootstrap = window.bootstrap || {};
window.bootstrap.Modal = Modal;
window.bootstrap.Dropdown = Dropdown;
window.bootstrap.Collapse = Collapse;
window.bootstrap.Tab = Tab;
window.bootstrap.Alert = Alert;

// Export for module usage
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { Modal, Dropdown, Collapse, Tab, Alert };
}
