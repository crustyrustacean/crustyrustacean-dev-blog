// admin.js - Admin dashboard functionality

/**
 * Admin dashboard manager
 */
class AdminManager {
    constructor() {
        this.init();
    }

    init() {
        this.setupArticleManagement();
        this.setupUserManagement();
        this.loadDashboardData();
    }

    /**
     * Setup article management functionality
     */
    setupArticleManagement() {
        // Delete article buttons
        document.querySelectorAll('.delete-article-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const articleSlug = e.target.dataset.slug;
                if (confirm('Are you sure you want to delete this article?')) {
                    this.deleteArticle(articleSlug);
                }
            });
        });

        // Edit article buttons are handled by navigation (no JS needed)
    }

    /**
     * Setup user management functionality
     */
    setupUserManagement() {
        // User action buttons
        document.querySelectorAll('.user-action-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const action = e.target.dataset.action;
                const userId = e.target.dataset.userId;
                this.handleUserAction(action, userId);
            });
        });
    }

    /**
     * Load dashboard statistics and data
     */
    async loadDashboardData() {
        try {
            // Load basic stats that might be available via API
            await this.loadArticleStats();
            await this.loadUserStats();
        } catch (error) {
            console.error('Error loading dashboard data:', error);
        }
    }

    /**
     * Delete an article
     */
    async deleteArticle(slug) {
        const token = getAuthToken();
        if (!token) {
            showError('Authentication required');
            return;
        }

        try {
            const response = await fetch(`/api/articles/${slug}`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${token}`
                }
            });

            if (response.ok) {
                showSuccess('Article deleted successfully');
                // Remove the article row from the table
                const articleRow = document.querySelector(`[data-slug="${slug}"]`).closest('tr');
                if (articleRow) {
                    articleRow.remove();
                }
                // Update counts
                this.updateArticleCount(-1);
            } else {
                const errorData = await response.json();
                showError(errorData.message || 'Failed to delete article');
            }
        } catch (error) {
            console.error('Error deleting article:', error);
            showError('Network error. Please try again.');
        }
    }

    /**
     * Handle user management actions
     */
    async handleUserAction(action, userId) {
        const token = getAuthToken();
        if (!token) {
            showError('Authentication required');
            return;
        }

        let endpoint = '';
        let method = 'POST';
        let confirmMessage = '';

        switch (action) {
            case 'suspend':
                confirmMessage = 'Are you sure you want to suspend this user?';
                endpoint = `/api/admin/users/${userId}/suspend`;
                break;
            case 'unsuspend':
                confirmMessage = 'Are you sure you want to unsuspend this user?';
                endpoint = `/api/admin/users/${userId}/unsuspend`;
                break;
            case 'delete':
                confirmMessage = 'Are you sure you want to delete this user? This cannot be undone.';
                endpoint = `/api/admin/users/${userId}`;
                method = 'DELETE';
                break;
            default:
                showError('Unknown action');
                return;
        }

        if (!confirm(confirmMessage)) {
            return;
        }

        try {
            const response = await fetch(endpoint, {
                method,
                headers: {
                    'Authorization': `Bearer ${token}`,
                    'Content-Type': 'application/json'
                }
            });

            if (response.ok) {
                showSuccess(`User ${action} successful`);
                // Refresh the user list or update the UI accordingly
                setTimeout(() => {
                    window.location.reload();
                }, 1000);
            } else {
                const errorData = await response.json();
                showError(errorData.message || `Failed to ${action} user`);
            }
        } catch (error) {
            console.error(`Error ${action} user:`, error);
            showError('Network error. Please try again.');
        }
    }

    /**
     * Load article statistics
     */
    async loadArticleStats() {
        try {
            const response = await fetch('/api/articles');
            if (response.ok) {
                const data = await response.json();
                this.updateArticleCount(data.articlesCount, true);
            }
        } catch (error) {
            console.error('Error loading article stats:', error);
        }
    }

    /**
     * Load user statistics
     */
    async loadUserStats() {
        // This would require an admin API endpoint
        // For now, we'll skip this unless the API is available
    }

    /**
     * Update article count display
     */
    updateArticleCount(count, isAbsolute = false) {
        const countElement = document.querySelector('.article-count');
        if (countElement) {
            const currentCount = parseInt(countElement.textContent) || 0;
            const newCount = isAbsolute ? count : currentCount + count;
            countElement.textContent = Math.max(0, newCount);
        }
    }

    /**
     * Show dashboard notifications
     */
    showNotification(message, type = 'info') {
        const notification = document.createElement('div');
        notification.className = `alert alert-${type} alert-dismissible fade show`;
        notification.innerHTML = `
            ${message}
            <button type="button" class="btn-close" data-bs-dismiss="alert"></button>
        `;

        const container = document.querySelector('.dashboard-notifications') || 
                         document.querySelector('.container-fluid') || 
                         document.body;
        
        container.insertBefore(notification, container.firstChild);

        // Auto-dismiss after 5 seconds
        setTimeout(() => {
            if (notification.parentNode) {
                notification.remove();
            }
        }, 5000);
    }
}

// Initialize when DOM is loaded
document.addEventListener('DOMContentLoaded', function() {
    // Only initialize on admin pages
    if (document.querySelector('.admin-dashboard, .admin-page')) {
        new AdminManager();
    }
});

// Export for manual initialization
window.AdminManager = AdminManager;