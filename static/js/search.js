// static/js/search.js

/**
 * Search functionality with modal display and result highlighting
 */

// Initialize search functionality when DOM is loaded
document.addEventListener('DOMContentLoaded', function () {
    const searchForm = document.getElementById('searchForm');
    const searchInput = document.getElementById('searchInput');
    const searchModal = new bootstrap.Modal(
        document.getElementById('searchModal')
    );
    const searchResults = document.getElementById('searchResults');
    const searchResultCount = document.getElementById('searchResultCount');

    // Handle search form submission
    if (searchForm) {
        searchForm.addEventListener('submit', async function (e) {
            e.preventDefault();

            const query = searchInput.value.trim();

            if (!query) {
                showError('Please enter a search query');
                return;
            }

            // Show the modal
            searchModal.show();

            // Show loading state
            searchResults.innerHTML = `
                <div class="text-center">
                    <div class="spinner-border text-primary" role="status">
                        <span class="visually-hidden">Searching...</span>
                    </div>
                    <p class="mt-3">Searching for "${escapeHtml(query)}"...</p>
                </div>
            `;
            searchResultCount.textContent = '';

            try {
                // Perform the search
                const response = await fetch(
                    `/api/search?q=${encodeURIComponent(query)}`
                );

                if (!response.ok) {
                    const errorData = await response.json();
                    const errorMessage =
                        errorData.errors?.body?.[0] || 'Search failed';
                    throw new Error(errorMessage);
                }

                const data = await response.json();

                // Display results
                displayResults(data.results, query);
                searchResultCount.textContent = `${data.resultsCount} result${data.resultsCount !== 1 ? 's' : ''} found`;
            } catch (error) {
                console.error('Search error:', error);
                showError(error.message || 'An error occurred while searching');
            }
        });
    }

    /**
     * Display search results with highlighting
     */
    function displayResults(results, query) {
        if (results.length === 0) {
            searchResults.innerHTML = `
                <div class="text-center text-muted">
                    <i class="fas fa-search fa-3x mb-3"></i>
                    <p>No articles found matching "${escapeHtml(query)}"</p>
                    <p class="small">Try different keywords or check your spelling</p>
                </div>
            `;
            return;
        }

        const resultsHtml = results
            .map(
                (article) => `
            <div class="card mb-3 search-result-card">
                <div class="card-body">
                    <h5 class="card-title">
                        <a href="/articles/${article.slug}" class="text-decoration-none">
                            ${highlightText(escapeHtml(article.title), query)}
                        </a>
                    </h5>
                    <p class="card-text text-muted small">
                        By <strong>${escapeHtml(article.author.username)}</strong>
                        • ${formatDate(article.createdAt)}
                        ${article.tagList.length > 0 ? ` • ${article.tagList.map((tag) => `<span class="badge bg-secondary">${escapeHtml(tag)}</span>`).join(' ')}` : ''}
                    </p>
                    <p class="card-text">
                        ${highlightText(escapeHtml(article.description), query)}
                    </p>
                    ${article.body ? `<p class="card-text small text-muted">${highlightText(truncateText(escapeHtml(article.body), 150), query)}</p>` : ''}
                    <div class="d-flex align-items-center text-muted small">
                        <span class="me-3">
                            <i class="fas fa-heart${article.favorited ? ' text-danger' : ''}"></i>
                            ${article.favoritesCount}
                        </span>
                    </div>
                </div>
            </div>
        `
            )
            .join('');

        searchResults.innerHTML = resultsHtml;
    }

    /**
     * Highlight search query in text
     */
    function highlightText(text, query) {
        if (!query) return text;

        // Escape special regex characters in the query
        const escapedQuery = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

        // Create a case-insensitive regex
        const regex = new RegExp(`(${escapedQuery})`, 'gi');

        // Replace matches with highlighted version
        return text.replace(
            regex,
            '<mark class="bg-warning bg-opacity-50">$1</mark>'
        );
    }

    /**
     * Show error message in the modal
     */
    function showError(message) {
        searchResults.innerHTML = `
            <div class="alert alert-danger" role="alert">
                <i class="fas fa-exclamation-circle me-2"></i>
                ${escapeHtml(message)}
            </div>
        `;
        searchResultCount.textContent = '';
    }

    /**
     * Escape HTML to prevent XSS
     */
    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    /**
     * Truncate text to a specified length
     */
    function truncateText(text, maxLength) {
        if (text.length <= maxLength) return text;
        return text.substring(0, maxLength).trim() + '...';
    }

    /**
     * Format date for display
     */
    function formatDate(dateString) {
        const date = new Date(dateString);
        const now = new Date();
        const diffInMs = now - date;
        const diffInDays = Math.floor(diffInMs / (1000 * 60 * 60 * 24));

        if (diffInDays === 0) return 'Today';
        if (diffInDays === 1) return 'Yesterday';
        if (diffInDays < 7) return `${diffInDays} days ago`;
        if (diffInDays < 30) return `${Math.floor(diffInDays / 7)} weeks ago`;
        if (diffInDays < 365)
            return `${Math.floor(diffInDays / 30)} months ago`;

        return date.toLocaleDateString('en-US', {
            year: 'numeric',
            month: 'short',
            day: 'numeric',
        });
    }

    // Allow searching by pressing Enter in the input field
    if (searchInput) {
        searchInput.addEventListener('keypress', function (e) {
            if (e.key === 'Enter') {
                e.preventDefault();
                searchForm.dispatchEvent(new Event('submit'));
            }
        });
    }

    // Clear input when modal is hidden
    document
        .getElementById('searchModal')
        .addEventListener('hidden.bs.modal', function () {
            searchInput.value = '';
            searchResults.innerHTML = `
            <div class="text-center text-muted">
                <i class="fas fa-search fa-3x mb-3"></i>
                <p>Enter a search query to find articles</p>
            </div>
        `;
            searchResultCount.textContent = '';
        });
});
