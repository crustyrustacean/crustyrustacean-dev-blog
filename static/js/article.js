// article.js - Article management functionality

/**
 * Article manager class
 */
class ArticleManager {
    constructor(articleSlug) {
        this.articleSlug = articleSlug;
        this.init();
    }

    init() {
        this.setupDeleteHandler();
        this.setupFavoriteHandler();
    }

    /**
     * Setup article deletion functionality
     */
    setupDeleteHandler() {
        const confirmDeleteBtn = document.getElementById('confirmDeleteBtn');
        if (confirmDeleteBtn) {
            confirmDeleteBtn.addEventListener('click', () => this.handleDeleteArticle());
        }

        // Setup modal trigger
        window.confirmDeleteArticle = () => {
            const modal = new bootstrap.Modal(document.getElementById('deleteModal'));
            modal.show();
        };
    }

    /**
     * Handle article deletion
     */
    async handleDeleteArticle() {
        const token = getAuthToken();
        const confirmDeleteBtn = document.getElementById('confirmDeleteBtn');
        
        if (!token) {
            showError('You must be logged in to delete articles.');
            return;
        }
        
        try {
            showButtonLoading(confirmDeleteBtn, 'Deleting...');
            
            const response = await fetch(`/api/articles/${this.articleSlug}`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${token}`
                }
            });
            
            if (response.ok) {
                // Redirect to home page after successful deletion
                window.location.href = '/';
            } else {
                const errorData = await response.json();
                throw new Error(errorData.errors?.body?.[0] || 'Failed to delete article.');
            }
        } catch (error) {
            console.error('Error deleting article:', error);
            showError('Network error. Please try again.');
            resetButtonLoading(confirmDeleteBtn);
        }
    }

    /**
     * Setup favorite/unfavorite functionality
     */
    setupFavoriteHandler() {
        const favoriteBtn = document.getElementById('favoriteBtn');
        if (favoriteBtn) {
            favoriteBtn.addEventListener('click', () => this.handleToggleFavorite());
        }

        // Make function globally available for onclick handlers
        window.toggleFavorite = () => this.handleToggleFavorite();
    }

    /**
     * Handle toggling favorite status
     */
    async handleToggleFavorite() {
        const btn = document.getElementById('favoriteBtn');
        const text = document.getElementById('favoriteText');
        const countEl = document.getElementById('favoriteCount');
        const icon = btn.querySelector('i');
        const isFavorited = btn.dataset.favorited === 'true';
        const token = getAuthToken();
        
        if (!token) {
            showError('You must be logged in to favorite articles.');
            return;
        }
        
        // Disable button during request
        btn.disabled = true;
        
        const method = isFavorited ? 'DELETE' : 'POST';
        const url = `/api/articles/${this.articleSlug}/favorite`;
        
        try {
            const response = await fetch(url, {
                method: method,
                headers: {
                    'Authorization': `Bearer ${token}`,
                    'Content-Type': 'application/json'
                }
            });
            
            if (!response.ok) {
                throw new Error('Network response was not ok');
            }
            
            const data = await response.json();
            const article = data.article;
            const newFavorited = article.favorited;
            const newCount = article.favoritesCount;
            
            // Update button state
            btn.dataset.favorited = newFavorited.toString();
            if (text) text.textContent = newFavorited ? 'Unfavorite' : 'Favorite';
            if (countEl) countEl.textContent = newCount;
            
            // Update icon
            if (icon) {
                if (newFavorited) {
                    icon.className = 'fas fa-heart';
                    btn.className = 'btn btn-danger me-3';
                } else {
                    icon.className = 'fas fa-heart-o';
                    btn.className = 'btn btn-outline-danger me-3';
                }
            }
            
        } catch (error) {
            console.error('Error updating favorite status:', error);
            showError('Error updating favorite status. Please try again.');
        } finally {
            btn.disabled = false;
        }
    }
}

// Export for use in templates
window.ArticleManager = ArticleManager;