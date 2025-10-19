// Feed page JavaScript Module
let currentOffset = 0;
    
// Handle favorite/unfavorite functionality
document.addEventListener('DOMContentLoaded', function() {
    // Get current offset from articles count
    const articlesContainer = document.querySelector('.col-lg-8');
    if (articlesContainer) {
        currentOffset = articlesContainer.querySelectorAll('.card').length;
    }

    // Handle favorite buttons
    document.querySelectorAll('.favorite-btn').forEach(btn => {
        btn.addEventListener('click', function(e) {
            e.preventDefault();
            toggleFavorite(this);
        });
    });

    // Handle load more button
    const loadMoreBtn = document.getElementById('loadMoreBtn');
    if (loadMoreBtn) {
        loadMoreBtn.addEventListener('click', loadMore);
    }

    // Animate cards on scroll
    const observerOptions = {
        threshold: 0.1,
        rootMargin: '0px 0px -50px 0px'
    };

    const observer = new IntersectionObserver(function(entries) {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.style.opacity = '1';
                entry.target.style.transform = 'translateY(0)';
            }
        });
    }, observerOptions);

    document.querySelectorAll('.card').forEach(card => {
        card.style.opacity = '0';
        card.style.transform = 'translateY(20px)';
        card.style.transition = 'opacity 0.6s ease, transform 0.6s ease';
        observer.observe(card);
    });
});

async function toggleFavorite(btn) {
    const slug = btn.getAttribute('data-slug');
    const favorited = btn.getAttribute('data-favorited') === 'true';
    const method = favorited ? 'DELETE' : 'POST';
    
    try {
        const response = await fetch(`/api/articles/${slug}/favorite`, {
            method: method,
            headers: {
                'Authorization': `Bearer ${localStorage.getItem('jwt_token')}`,
                'Content-Type': 'application/json',
            }
        });
        
        if (response.ok) {
            const data = await response.json();
            const article = data.article;
            
            // Update button state
            const icon = btn.querySelector('i');
            if (article.favorited) {
                icon.className = 'fas fa-heart';
                btn.setAttribute('data-favorited', 'true');
            } else {
                icon.className = 'far fa-heart';
                btn.setAttribute('data-favorited', 'false');
            }
            
            // Update favorites count in the same card
            const favoritesCount = btn.closest('.card').querySelector('.text-muted');
            if (favoritesCount && favoritesCount.textContent.includes('favorites')) {
                favoritesCount.innerHTML = `<i class="fas fa-heart text-danger me-1"></i>${article.favoritesCount} favorites`;
            }
        } else {
            console.error('Failed to toggle favorite');
        }
    } catch (error) {
        console.error('Error toggling favorite:', error);
    }
}

async function loadMore() {
    try {
        const response = await fetch(`/api/articles/feed?offset=${currentOffset}`, {
            headers: {
                'Authorization': `Bearer ${localStorage.getItem('jwt_token')}`,
            }
        });
        
        if (response.ok) {
            const data = await response.json();
            // TODO: Implement append new articles to the page
            console.log('More articles:', data.articles);
            currentOffset += data.articles.length;
        }
    } catch (error) {
        console.error('Error loading more articles:', error);
    }
}