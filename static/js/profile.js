// Profile JavaScript Module
document.addEventListener('DOMContentLoaded', function() {
    // Follow/Unfollow functionality
    const followBtn = document.getElementById('followBtn');
    if (followBtn) {
        followBtn.addEventListener('click', async function() {
            const username = this.dataset.username;
            const isFollowing = this.dataset.following === 'true';

            try {
                const response = await fetch(`/api/profiles/${username}/follow`, {
                    method: isFollowing ? 'DELETE' : 'POST',
                    headers: {
                        'Authorization': `Bearer ${localStorage.getItem('authToken')}`,
                        'Content-Type': 'application/json'
                    }
                });

                if (response.ok) {
                    // Toggle button state
                    if (isFollowing) {
                        this.classList.remove('btn-outline-primary');
                        this.classList.add('btn-primary');
                        this.innerHTML = '<i class="fas fa-user-plus me-1"></i>Follow';
                        this.dataset.following = 'false';
                    } else {
                        this.classList.remove('btn-primary');
                        this.classList.add('btn-outline-primary');
                        this.innerHTML = '<i class="fas fa-user-minus me-1"></i>Unfollow';
                        this.dataset.following = 'true';
                    }

                    // Update follower count
                    const followersElement = document.querySelector('.col-4:nth-child(2) .fw-bold');
                    if (followersElement) {
                        const count = parseInt(followersElement.textContent);
                        followersElement.textContent = isFollowing ? count - 1 : count + 1;
                    }
                } else {
                    console.error('Follow/unfollow failed');
                }
            } catch (error) {
                console.error('Network error:', error);
            }
        });
    }

    // Handle delete draft buttons
    document.addEventListener('click', function(e) {
        if (e.target.closest('.delete-draft-btn')) {
            const btn = e.target.closest('.delete-draft-btn');
            const draftId = btn.dataset.draftId;
            deleteDraft(draftId);
        }
    });
});

// Delete draft function
async function deleteDraft(draftId) {
    if (confirm('Are you sure you want to delete this draft?')) {
        try {
            const response = await fetch(`/api/articles/${draftId}`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${localStorage.getItem('authToken')}`
                }
            });

            if (response.ok) {
                // Remove the draft element from the page
                const draftElement = event.target.closest('.card');
                draftElement.remove();

                // If no more drafts, show empty state
                const draftsContainer = document.getElementById('drafts');
                if (draftsContainer.querySelectorAll('.card').length === 0) {
                    draftsContainer.innerHTML = `
                        <div class="text-center py-5">
                            <i class="fas fa-edit fa-3x text-muted mb-3"></i>
                            <h5 class="text-muted">No drafts</h5>
                            <p class="text-muted">Your draft articles will appear here.</p>
                            <a href="/editor" class="btn btn-primary">
                                <i class="fas fa-plus me-1"></i>
                                Start Writing
                            </a>
                        </div>
                    `;
                }
            } else {
                alert('Failed to delete draft');
            }
        } catch (error) {
            console.error('Error deleting draft:', error);
            alert('Error deleting draft');
        }
    }
}