// Authors page JavaScript Module
let currentOffset = 0;
let currentSearch = "";
let isLoading = false;

// Initialize variables from template data
document.addEventListener('DOMContentLoaded', function() {
    // Get data from DOM
    const authorsContainer = document.getElementById('authorsContainer');
    if (authorsContainer) {
        currentOffset = authorsContainer.querySelectorAll('.author-item').length;
    }

    const searchInput = document.getElementById('searchInput');
    if (searchInput) {
        currentSearch = searchInput.value || "";

        // Search on Enter key
        searchInput.addEventListener('keypress', function(e) {
            if (e.key === 'Enter') {
                performSearch();
            }
        });
    }

    // Search button
    const searchBtn = document.getElementById('searchBtn');
    if (searchBtn) {
        searchBtn.addEventListener('click', performSearch);
    }

    // Clear search buttons
    const clearSearchBtns = document.querySelectorAll('#clearSearchBtn');
    clearSearchBtns.forEach(btn => {
        btn.addEventListener('click', clearSearch);
    });

    // Load more button
    const loadMoreBtn = document.getElementById('loadMoreBtn');
    if (loadMoreBtn) {
        loadMoreBtn.addEventListener('click', loadMore);
    }

    // Follow/unfollow buttons with event delegation
    document.addEventListener('click', function(e) {
        if (e.target.closest('[data-toggle-follow]')) {
            const btn = e.target.closest('[data-toggle-follow]');
            toggleFollow(btn);
        }
    });
});

function performSearch() {
    const searchInput = document.getElementById('searchInput');
    const searchTerm = searchInput.value.trim();

    if (searchTerm) {
        window.location.href = `/profiles?search=${encodeURIComponent(searchTerm)}`;
    } else {
        window.location.href = '/profiles';
    }
}

function clearSearch() {
    window.location.href = '/profiles';
}

// Show toast notification
function showToast(message, type = 'success') {
    // Remove any existing toasts
    const existingToast = document.querySelector('.toast-notification');
    if (existingToast) {
        existingToast.remove();
    }

    const toast = document.createElement('div');
    toast.className = `toast-notification alert alert-${type} position-fixed`;
    toast.style.cssText = 'top: 80px; right: 20px; z-index: 9999; min-width: 250px; animation: slideIn 0.3s ease;';
    toast.innerHTML = `
        <div class="d-flex align-items-center">
            <i class="fas fa-${type === 'success' ? 'check-circle' : type === 'danger' ? 'exclamation-circle' : 'info-circle'} me-2"></i>
            <span>${message}</span>
        </div>
    `;

    document.body.appendChild(toast);

    // Auto-remove after 3 seconds
    setTimeout(() => {
        toast.style.animation = 'slideOut 0.3s ease';
        setTimeout(() => toast.remove(), 300);
    }, 3000);
}

async function toggleFollow(btn) {
    const username = btn.getAttribute('data-username');
    const isFollowing = btn.getAttribute('data-following') === 'true';
    const method = isFollowing ? 'DELETE' : 'POST';
    const url = `/api/profiles/${username}/follow`;

    // Store original state for rollback
    const originalHTML = btn.innerHTML;
    const originalClass = btn.className;

    try {
        btn.disabled = true;
        btn.innerHTML = '<i class="fas fa-spinner fa-spin me-1"></i>';

        const response = await fetch(url, {
            method: method,
            headers: {
                'Authorization': `Bearer ${localStorage.getItem('jwt_token')}`,
                'Content-Type': 'application/json',
            }
        });

        if (response.ok) {
            const data = await response.json();
            const newFollowing = data.profile.following;

            // Update button state
            btn.setAttribute('data-following', newFollowing.toString());

            if (newFollowing) {
                btn.className = 'btn btn-success btn-sm follow-btn';
                btn.innerHTML = '<i class="fas fa-user-check me-1"></i>Following';
                showToast(`You are now following ${username}`, 'success');
            } else {
                btn.className = 'btn btn-primary btn-sm follow-btn';
                btn.innerHTML = '<i class="fas fa-user-plus me-1"></i>Follow';
                showToast(`You unfollowed ${username}`, 'info');
            }
        } else {
            // Handle error
            console.error('Follow/unfollow failed:', response.status);
            btn.innerHTML = originalHTML;
            btn.className = originalClass;

            if (response.status === 401) {
                showToast('Please log in to follow authors', 'warning');
            } else {
                showToast('Failed to update follow status', 'danger');
            }
        }
    } catch (error) {
        console.error('Error toggling follow:', error);
        btn.innerHTML = originalHTML;
        btn.className = originalClass;
        showToast('Network error. Please try again.', 'danger');
    } finally {
        btn.disabled = false;
    }
}

async function loadMore() {
    if (isLoading) return;

    const loadMoreBtn = document.getElementById('loadMoreBtn');
    const authorsContainer = document.getElementById('authorsContainer');

    if (!loadMoreBtn || !authorsContainer) return;

    try {
        isLoading = true;
        const originalText = loadMoreBtn.innerHTML;
        loadMoreBtn.innerHTML = '<i class="fas fa-spinner fa-spin me-2"></i>Loading...';
        loadMoreBtn.disabled = true;

        let url = `/api/profiles?offset=${currentOffset}&limit=20`;
        if (currentSearch) {
            url += `&search=${encodeURIComponent(currentSearch)}`;
        }

        const response = await fetch(url, {
            headers: {
                'Authorization': `Bearer ${localStorage.getItem('jwt_token')}`,
            }
        });

        if (response.ok) {
            const data = await response.json();
            const profiles = data.profiles || [];

            if (profiles.length > 0) {
                // Append new profiles to the container
                profiles.forEach((profile, index) => {
                    const authorCard = createAuthorCard(profile, currentOffset + index);
                    authorsContainer.insertAdjacentHTML('beforeend', authorCard);
                });

                currentOffset += profiles.length;

                // Hide button if no more profiles
                if (profiles.length < 20) {
                    loadMoreBtn.style.display = 'none';
                } else {
                    loadMoreBtn.innerHTML = originalText;
                    loadMoreBtn.disabled = false;
                }
            } else {
                loadMoreBtn.style.display = 'none';
                showToast('No more authors to load', 'info');
            }
        } else {
            loadMoreBtn.innerHTML = originalText;
            loadMoreBtn.disabled = false;
            showToast('Failed to load more authors', 'danger');
        }
    } catch (error) {
        console.error('Error loading more profiles:', error);
        loadMoreBtn.innerHTML = '<i class="fas fa-plus-circle me-2"></i>Load More Authors';
        loadMoreBtn.disabled = false;
        showToast('Network error. Please try again.', 'danger');
    } finally {
        isLoading = false;
    }
}

function createAuthorCard(profile, index) {
    const initial = profile.username ? profile.username.charAt(0).toUpperCase() : '?';
    const avatarHtml = profile.image
        ? `<img src="${escapeHtml(profile.image)}" alt="${escapeHtml(profile.username)}" class="rounded-circle shadow-sm" width="64" height="64" style="object-fit: cover;">`
        : `<div class="rounded-circle bg-primary text-white d-flex align-items-center justify-content-center shadow-sm" style="width: 64px; height: 64px; font-size: 24px; font-weight: bold;">${initial}</div>`;

    const bioHtml = profile.bio
        ? `<p class="card-text text-muted small mb-2">${escapeHtml(truncate(profile.bio, 80))}</p>`
        : `<p class="card-text text-muted small fst-italic mb-2">No bio yet</p>`;

    const followBtnClass = profile.following ? 'btn btn-success btn-sm follow-btn' : 'btn btn-primary btn-sm follow-btn';
    const followBtnIcon = profile.following ? 'fa-user-check' : 'fa-user-plus';
    const followBtnText = profile.following ? 'Following' : 'Follow';

    return `
        <div class="col-md-6 mb-4 author-item" style="animation-delay: ${index * 0.05}s;">
            <div class="card h-100 author-card shadow-sm border-0">
                <div class="card-body">
                    <div class="d-flex align-items-start">
                        <div class="me-3 flex-shrink-0">
                            ${avatarHtml}
                        </div>
                        <div class="flex-grow-1">
                            <h5 class="card-title mb-1">
                                <a href="/profiles/${escapeHtml(profile.username)}" class="text-decoration-none text-dark">
                                    ${escapeHtml(profile.username)}
                                </a>
                            </h5>
                            ${bioHtml}
                            <div class="d-flex gap-2 mt-2">
                                <a href="/profiles/${escapeHtml(profile.username)}" class="btn btn-sm btn-outline-secondary">
                                    <i class="fas fa-user me-1"></i>
                                    Profile
                                </a>
                                <button
                                    class="${followBtnClass}"
                                    data-username="${escapeHtml(profile.username)}"
                                    data-following="${profile.following}"
                                    data-toggle-follow
                                >
                                    <i class="fas ${followBtnIcon} me-1"></i>
                                    ${followBtnText}
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    `;
}

function escapeHtml(text) {
    if (!text) return '';
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function truncate(text, length) {
    if (!text) return '';
    if (text.length <= length) return text;
    return text.substring(0, length).trim() + '...';
}

// Add CSS for toast animations
const style = document.createElement('style');
style.textContent = `
    @keyframes slideIn {
        from { transform: translateX(100%); opacity: 0; }
        to { transform: translateX(0); opacity: 1; }
    }
    @keyframes slideOut {
        from { transform: translateX(0); opacity: 1; }
        to { transform: translateX(100%); opacity: 0; }
    }
`;
document.head.appendChild(style);
