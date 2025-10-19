// Authors page JavaScript Module
let currentOffset = 0;
let currentSearch = "";

// Initialize variables from template data
document.addEventListener('DOMContentLoaded', function() {
    // Get data from DOM
    const profilesContainer = document.querySelector('.row');
    if (profilesContainer) {
        currentOffset = profilesContainer.querySelectorAll('.col-md-6').length;
    }
    
    const searchInput = document.getElementById('searchInput');
    if (searchInput) {
        currentSearch = searchInput.value || "";
    }

    // Search functionality
    if (searchInput) {
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

    document.querySelectorAll('.author-card').forEach(card => {
        card.style.opacity = '0';
        card.style.transform = 'translateY(20px)';
        card.style.transition = 'opacity 0.6s ease, transform 0.6s ease';
        observer.observe(card);
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

async function toggleFollow(btn) {
    const username = btn.getAttribute('data-username');
    const isFollowing = btn.getAttribute('data-following') === 'true';
    const method = isFollowing ? 'DELETE' : 'POST';
    const url = `/api/profiles/${username}/follow`;

    try {
        btn.disabled = true;
        btn.innerHTML = '<i class="fas fa-spinner fa-spin me-1"></i>Working...';

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
            } else {
                btn.className = 'btn btn-outline-primary btn-sm follow-btn';
                btn.innerHTML = '<i class="fas fa-user-plus me-1"></i>Follow';
            }
        } else {
            // Handle error
            console.error('Follow/unfollow failed');
            btn.innerHTML = isFollowing ? 
                '<i class="fas fa-user-check me-1"></i>Following' : 
                '<i class="fas fa-user-plus me-1"></i>Follow';
        }
    } catch (error) {
        console.error('Error toggling follow:', error);
        btn.innerHTML = isFollowing ? 
            '<i class="fas fa-user-check me-1"></i>Following' : 
            '<i class="fas fa-user-plus me-1"></i>Follow';
    } finally {
        btn.disabled = false;
    }
}

async function loadMore() {
    try {
        let url = `/api/profiles?offset=${currentOffset}`;
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
            // TODO: Implement append new profiles to the page
            console.log('More profiles:', data.profiles);
            currentOffset += data.profiles.length;
        }
    } catch (error) {
        console.error('Error loading more profiles:', error);
    }
}