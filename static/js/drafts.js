// drafts.js - Draft articles management functionality

/**
 * Drafts manager for handling draft article operations
 */
class DraftsManager {
    constructor() {
        this.publishModal = null;
        this.deleteModal = null;
        this.currentSlug = null;
        this.currentTitle = null;
        this.init();
    }

    init() {
        this.setupModals();
        this.setupPublishButtons();
        this.setupDeleteButtons();
    }

    /**
     * Setup Bootstrap modals
     */
    setupModals() {
        const publishModalEl = document.getElementById('publishModal');
        const deleteModalEl = document.getElementById('deleteModal');

        if (publishModalEl) {
            this.publishModal = new bootstrap.Modal(publishModalEl);
            document.getElementById('confirmPublishBtn')?.addEventListener('click', () => {
                this.publishDraft(this.currentSlug);
            });
        }

        if (deleteModalEl) {
            this.deleteModal = new bootstrap.Modal(deleteModalEl);
            document.getElementById('confirmDeleteBtn')?.addEventListener('click', () => {
                this.deleteDraft(this.currentSlug);
            });
        }
    }

    /**
     * Setup publish button event listeners
     */
    setupPublishButtons() {
        document.querySelectorAll('.publish-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const button = e.currentTarget;
                this.currentSlug = button.getAttribute('data-slug');
                this.currentTitle = button.getAttribute('data-title');

                const titleEl = document.getElementById('publishArticleTitle');
                if (titleEl) {
                    titleEl.textContent = this.currentTitle;
                }

                this.publishModal?.show();
            });
        });
    }

    /**
     * Setup delete button event listeners
     */
    setupDeleteButtons() {
        document.querySelectorAll('.delete-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const button = e.currentTarget;
                this.currentSlug = button.getAttribute('data-slug');
                this.currentTitle = button.getAttribute('data-title');

                const titleEl = document.getElementById('deleteArticleTitle');
                if (titleEl) {
                    titleEl.textContent = this.currentTitle;
                }

                this.deleteModal?.show();
            });
        });
    }

    /**
     * Publish a draft article
     */
    async publishDraft(slug) {
        const token = getAuthToken();
        if (!token) {
            showError('Authentication required');
            return;
        }

        try {
            // Update the article to set draft = false
            const response = await fetch(`/api/articles/${slug}`, {
                method: 'PUT',
                headers: {
                    'Authorization': `Bearer ${token}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    article: {
                        draft: false
                    }
                })
            });

            if (response.ok) {
                showSuccess('Article published successfully');
                this.publishModal?.hide();

                // Remove the draft row from the table
                const draftRow = document.querySelector(`tr[data-slug="${slug}"]`);
                if (draftRow) {
                    // Fade out animation
                    draftRow.style.transition = 'opacity 0.3s';
                    draftRow.style.opacity = '0';
                    setTimeout(() => {
                        draftRow.remove();

                        // Check if there are any drafts left
                        const remainingDrafts = document.querySelectorAll('#draftsTable tbody tr');
                        if (remainingDrafts.length === 0) {
                            // Reload the page to show the "no drafts" message
                            setTimeout(() => {
                                window.location.reload();
                            }, 1000);
                        }
                    }, 300);
                }
            } else {
                const errorData = await response.json();
                showError(errorData.message || 'Failed to publish article');
            }
        } catch (error) {
            console.error('Error publishing article:', error);
            showError('Network error. Please try again.');
        }
    }

    /**
     * Delete a draft article
     */
    async deleteDraft(slug) {
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

            if (response.ok || response.status === 204) {
                showSuccess('Draft deleted successfully');
                this.deleteModal?.hide();

                // Remove the draft row from the table
                const draftRow = document.querySelector(`tr[data-slug="${slug}"]`);
                if (draftRow) {
                    // Fade out animation
                    draftRow.style.transition = 'opacity 0.3s';
                    draftRow.style.opacity = '0';
                    setTimeout(() => {
                        draftRow.remove();

                        // Check if there are any drafts left
                        const remainingDrafts = document.querySelectorAll('#draftsTable tbody tr');
                        if (remainingDrafts.length === 0) {
                            // Reload the page to show the "no drafts" message
                            setTimeout(() => {
                                window.location.reload();
                            }, 1000);
                        }
                    }, 300);
                }
            } else {
                const errorData = await response.json();
                showError(errorData.message || 'Failed to delete draft');
            }
        } catch (error) {
            console.error('Error deleting draft:', error);
            showError('Network error. Please try again.');
        }
    }

    /**
     * Show notification message
     */
    showNotification(message, type = 'info') {
        const notification = document.createElement('div');
        notification.className = `alert alert-${type} alert-dismissible fade show position-fixed top-0 start-50 translate-middle-x mt-3`;
        notification.style.zIndex = '9999';
        notification.innerHTML = `
            ${message}
            <button type="button" class="btn-close" data-bs-dismiss="alert"></button>
        `;

        document.body.appendChild(notification);

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
    // Only initialize on drafts page
    if (document.querySelector('#draftsTable')) {
        new DraftsManager();
    }
});

// Export for manual initialization
window.DraftsManager = DraftsManager;
