/**
 * Theme Switcher for CrustyRustacean Dev Blog
 *
 * Handles client-side theme switching with:
 * - LocalStorage persistence for unauthenticated users
 * - Server sync for authenticated users
 * - System preference (prefers-color-scheme) respect
 * - Smooth theme transitions
 */

class ThemeSwitcher {
    constructor() {
        this.storageKey = 'theme-preference';
        this.themes = ['default', 'dark', 'high-contrast'];
        this.initialized = false;
    }

    /**
     * Initialize the theme switcher
     */
    init() {
        if (this.initialized) return;

        // Apply saved theme immediately to prevent flash
        const saved = this.getSavedTheme();
        this.apply(saved, false);

        // Listen for system preference changes
        const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
        mediaQuery.addEventListener('change', () => {
            if (this.getSavedTheme() === 'auto') {
                this.apply('auto', true);
            }
        });

        // Listen for storage changes (sync across tabs)
        window.addEventListener('storage', (e) => {
            if (e.key === this.storageKey) {
                this.apply(e.newValue || 'auto', true);
            }
        });

        this.initialized = true;
    }

    /**
     * Get the saved theme preference
     * @returns {string} Theme ID or 'auto'
     */
    getSavedTheme() {
        return localStorage.getItem(this.storageKey) || 'auto';
    }

    /**
     * Get the effective theme (resolving 'auto' to actual theme)
     * @returns {string} Theme ID
     */
    getEffectiveTheme() {
        const saved = this.getSavedTheme();
        if (saved === 'auto') {
            return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'default';
        }
        return saved;
    }

    /**
     * Apply a theme to the document
     * @param {string} themeId - Theme ID or 'auto'
     * @param {boolean} animate - Whether to animate the transition
     */
    apply(themeId, animate = true) {
        let effectiveTheme;

        if (themeId === 'auto') {
            const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
            effectiveTheme = prefersDark ? 'dark' : 'default';
        } else {
            effectiveTheme = themeId;
        }

        // Add transition class if animating
        if (animate) {
            document.documentElement.classList.add('theme-transitioning');
        }

        // Set the data attribute
        document.documentElement.dataset.theme = effectiveTheme;

        // Update color-scheme meta tag
        const colorSchemeMeta = document.querySelector('meta[name="color-scheme"]');
        if (colorSchemeMeta) {
            colorSchemeMeta.content = effectiveTheme === 'dark' ? 'dark' : 'light';
        }

        // Remove transition class after animation completes
        if (animate) {
            setTimeout(() => {
                document.documentElement.classList.remove('theme-transitioning');
            }, 300);
        }

        // Dispatch custom event for other scripts to react
        document.dispatchEvent(new CustomEvent('themechange', {
            detail: { theme: effectiveTheme, preference: themeId }
        }));
    }

    /**
     * Set the theme preference
     * @param {string} themeId - Theme ID or 'auto'
     * @param {boolean} syncToServer - Whether to sync to server
     */
    async setTheme(themeId, syncToServer = true) {
        // Save to localStorage
        localStorage.setItem(this.storageKey, themeId);

        // Apply the theme
        this.apply(themeId, true);

        // Sync to server if authenticated
        if (syncToServer) {
            await this.syncToServer(themeId);
        }
    }

    /**
     * Sync theme preference to the server
     * @param {string} themeId - Theme ID
     */
    async syncToServer(themeId) {
        try {
            const token = localStorage.getItem('token');
            if (!token) return; // Not authenticated

            const response = await fetch('/api/user/theme', {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${token}`
                },
                body: JSON.stringify({ theme: themeId })
            });

            if (!response.ok) {
                console.warn('Failed to sync theme preference to server');
            }
        } catch (e) {
            // Graceful fallback - localStorage still works
            console.warn('Could not sync theme to server:', e.message);
        }
    }

    /**
     * Load theme preference from server
     * @returns {Promise<string|null>} Theme ID or null
     */
    async loadFromServer() {
        try {
            const token = localStorage.getItem('token');
            if (!token) return null;

            const response = await fetch('/api/user/theme', {
                headers: {
                    'Authorization': `Bearer ${token}`
                }
            });

            if (response.ok) {
                const data = await response.json();
                return data.theme || null;
            }
        } catch (e) {
            console.warn('Could not load theme from server:', e.message);
        }
        return null;
    }

    /**
     * Cycle to the next theme
     */
    cycleTheme() {
        const current = this.getSavedTheme();
        const allOptions = ['auto', ...this.themes];
        const currentIndex = allOptions.indexOf(current);
        const nextIndex = (currentIndex + 1) % allOptions.length;
        this.setTheme(allOptions[nextIndex]);
    }

    /**
     * Get list of available themes
     * @returns {Array} Array of theme options
     */
    getAvailableThemes() {
        return [
            { id: 'auto', name: 'System', description: 'Follow system preference' },
            { id: 'default', name: 'Light', description: 'Default light theme' },
            { id: 'dark', name: 'Dark', description: 'Dark theme for low-light' },
            { id: 'high-contrast', name: 'High Contrast', description: 'Maximum readability' }
        ];
    }
}

// Create global instance
window.themeSwitcher = new ThemeSwitcher();

// Initialize immediately to prevent flash of wrong theme
(function() {
    // Apply theme before DOMContentLoaded to prevent flash
    const saved = localStorage.getItem('theme-preference') || 'auto';
    let effectiveTheme;

    if (saved === 'auto') {
        effectiveTheme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'default';
    } else {
        effectiveTheme = saved;
    }

    document.documentElement.dataset.theme = effectiveTheme;
})();

// Full initialization after DOM is ready
document.addEventListener('DOMContentLoaded', function() {
    window.themeSwitcher.init();

    // Try to load preference from server for authenticated users
    window.themeSwitcher.loadFromServer().then(serverTheme => {
        if (serverTheme) {
            const localTheme = window.themeSwitcher.getSavedTheme();
            // Use server preference if different from local
            if (serverTheme !== localTheme) {
                window.themeSwitcher.setTheme(serverTheme, false);
            }
        }
    });
});
