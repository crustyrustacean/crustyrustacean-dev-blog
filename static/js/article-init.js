// Article initialization script
document.addEventListener('DOMContentLoaded', function() {
    // Get article slug from URL path or favorite button
    let articleSlug = null;
    
    // Try to get from favorite button first (logged in users)
    const favoriteBtn = document.getElementById('favoriteBtn');
    if (favoriteBtn && favoriteBtn.dataset.slug) {
        articleSlug = favoriteBtn.dataset.slug;
    } else {
        // Extract from URL path (fallback for non-logged in users)
        const path = window.location.pathname;
        const parts = path.split('/');
        if (parts.length >= 2 && parts[1] === 'articles' && parts[2]) {
            articleSlug = parts[2];
        }
    }
    
    if (articleSlug) {
        // Initialize comment manager
        const userElement = document.querySelector('[data-current-user]');
        const currentUser = userElement ? userElement.dataset.currentUser : null;
        
        if (typeof CommentManager !== 'undefined') {
            const commentManager = new CommentManager(articleSlug, currentUser);
        }
        
        // Initialize article manager for favorite functionality (only if user is logged in)
        if (favoriteBtn && typeof ArticleManager !== 'undefined') {
            const articleManager = new ArticleManager(articleSlug);
        }
    }
});
