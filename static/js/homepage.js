// homepage.js - Homepage-specific functionality

/**
 * Homepage manager for dynamic content and animations
 */
class HomepageManager {
    constructor() {
        this.init();
    }

    init() {
        this.loadTags();
        this.setupAnimations();
    }

    /**
     * Load and display tags from API
     */
    async loadTags() {
        try {
            const response = await fetch('/api/tags');
            const data = await response.json();
            const tagsContainer = document.getElementById('tags-container');
            
            if (!tagsContainer) return;

            if (data.tags && data.tags.length > 0) {
                const badgeColors = ['primary', 'secondary', 'success', 'warning', 'info', 'dark', 'danger'];
                tagsContainer.innerHTML = data.tags.map((tag, index) => {
                    const color = badgeColors[index % badgeColors.length];
                    return `<a href="/articles?tag=${encodeURIComponent(tag)}" class="badge bg-${color} text-decoration-none">${tag}</a>`;
                }).join('');
            } else {
                tagsContainer.innerHTML = '<span class="text-muted small">No tags yet</span>';
            }
        } catch (error) {
            console.error('Error fetching tags:', error);
            const tagsContainer = document.getElementById('tags-container');
            if (tagsContainer) {
                tagsContainer.innerHTML = '<span class="text-muted small">Unable to load tags</span>';
            }
        }
    }

    /**
     * Setup card animations on scroll
     */
    setupAnimations() {
        const observerOptions = {
            threshold: 0.1,
            rootMargin: '0px 0px -50px 0px'
        };

        const observer = new IntersectionObserver((entries) => {
            entries.forEach(entry => {
                if (entry.isIntersecting) {
                    entry.target.style.opacity = '1';
                    entry.target.style.transform = 'translateY(0)';
                }
            });
        }, observerOptions);

        // Animate cards on scroll
        document.querySelectorAll('.card').forEach(card => {
            card.style.opacity = '0';
            card.style.transform = 'translateY(20px)';
            card.style.transition = 'opacity 0.6s ease, transform 0.6s ease';
            observer.observe(card);
        });
    }
}

// Initialize when DOM is loaded
document.addEventListener('DOMContentLoaded', function() {
    // Only initialize on homepage
    if (document.querySelector('.homepage, .hero-section')) {
        new HomepageManager();
    }
});

// Export for manual initialization
window.HomepageManager = HomepageManager;