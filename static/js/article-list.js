// Article List page JavaScript Module
document.addEventListener('DOMContentLoaded', function() {
    // Fetch and display categories from API
    fetch('/api/categories')
        .then(response => response.json())
        .then(data => {
            const categoriesContainer = document.getElementById('filter-categories-container');
            if (data.categories && data.categories.length > 0) {
                categoriesContainer.innerHTML = data.categories.map(category => {
                    return `<a href="/articles?category=${encodeURIComponent(category.slug)}" class="badge bg-primary text-decoration-none">
                        <i class="fas fa-folder me-1"></i>${category.name}
                    </a>`;
                }).join('');
            } else {
                categoriesContainer.innerHTML = '<span class="text-muted small">No categories yet</span>';
            }
        })
        .catch(error => {
            console.error('Error fetching categories:', error);
            document.getElementById('filter-categories-container').innerHTML = '<span class="text-muted small">Unable to load categories</span>';
        });

    // Fetch and display tags from API
    fetch('/api/tags')
        .then(response => response.json())
        .then(data => {
            const tagsContainer = document.getElementById('filter-tags-container');
            if (data.tags && data.tags.length > 0) {
                const badgeColors = ['primary', 'secondary', 'success', 'warning', 'info', 'dark', 'danger'];
                tagsContainer.innerHTML = data.tags.map((tag, index) => {
                    const color = badgeColors[index % badgeColors.length];
                    return `<a href="/articles?tag=${encodeURIComponent(tag)}" class="badge bg-${color} text-decoration-none">${tag}</a>`;
                }).join('');
            } else {
                tagsContainer.innerHTML = '<span class="text-muted small">No tags yet</span>';
            }
        })
        .catch(error => {
            console.error('Error fetching tags:', error);
            document.getElementById('filter-tags-container').innerHTML = '<span class="text-muted small">Unable to load tags</span>';
        });

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