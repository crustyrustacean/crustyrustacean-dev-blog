// Article Page JavaScript Module

// Helper function to get auth token
function getAuthToken() {
    const token = localStorage.getItem('authToken');
    if (token) return token;
    
    // Fallback to cookies
    const cookies = document.cookie.split(';');
    for (let cookie of cookies) {
        const [name, value] = cookie.trim().split('=');
        if (name === 'authToken') {
            return decodeURIComponent(value);
        }
    }
    return null;
}

// Helper function to format date
function formatDate(dateString) {
    const date = new Date(dateString);
    return date.toLocaleDateString('en-US', { 
        year: 'numeric', 
        month: 'long', 
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit'
    });
}

// Get article slug from page context
function getArticleSlug() {
    // Extract from URL path or data attributes
    const path = window.location.pathname;
    const parts = path.split('/');
    return parts[parts.length - 1];
}

// Load comments
async function loadComments() {
    const commentsList = document.getElementById('commentsList');
    const articleSlug = getArticleSlug();
    
    try {
        const response = await fetch(`/api/articles/${articleSlug}/comments`);
        
        if (!response.ok) {
            throw new Error('Failed to load comments');
        }
        
        const data = await response.json();
        const comments = data.comments || [];
        
        if (comments.length === 0) {
            commentsList.innerHTML = `
                <div class="text-center text-muted py-4">
                    <i class="fas fa-comment-slash fa-2x mb-2"></i>
                    <p>No comments yet. Be the first to comment!</p>
                </div>
            `;
            return;
        }
        
        // Get current user from auth token or other source
        const currentUserElement = document.querySelector('[data-current-user]');
        const currentUser = currentUserElement ? currentUserElement.dataset.currentUser : null;
        
        commentsList.innerHTML = comments.map(comment => {
            const canDelete = currentUser === comment.author.username;
            const avatarHtml = comment.author.image 
                ? `<img src="${comment.author.image}" alt="${comment.author.username}" class="rounded-circle" width="40" height="40">` 
                : `<div class="bg-secondary rounded-circle d-flex align-items-center justify-content-center" style="width: 40px; height: 40px;"><span class="text-white fw-bold">${String(comment.author.username).charAt(0).toUpperCase()}</span></div>`;
            const deleteBtn = canDelete ? `<button class="btn btn-sm btn-outline-danger delete-comment-btn" data-comment-id="${comment.id}"><i class="fas fa-trash"></i></button>` : '';
            
            return `<div class="card mb-3" data-comment-id="${comment.id}">
                <div class="card-body">
                    <div class="d-flex">
                        <div class="me-3">${avatarHtml}</div>
                        <div class="flex-grow-1">
                            <div class="d-flex justify-content-between align-items-start mb-2">
                                <div>
                                    <strong>${comment.author.username}</strong>
                                    <span class="text-muted small ms-2">${formatDate(comment.createdAt)}</span>
                                </div>
                                ${deleteBtn}
                            </div>
                            <p class="mb-0">${comment.body}</p>
                        </div>
                    </div>
                </div>
            </div>`;
        }).join('');
        
        // Add delete event listeners
        document.querySelectorAll('.delete-comment-btn').forEach(btn => {
            btn.addEventListener('click', handleDeleteComment);
        });
        
    } catch (error) {
        console.error('Error loading comments:', error);
        commentsList.innerHTML = `
            <div class="alert alert-danger" role="alert">
                <i class="fas fa-exclamation-circle me-2"></i>
                Failed to load comments. Please refresh the page to try again.
                <br><small class="text-muted">Error: ${error.message}</small>
            </div>
        `;
    }
}

// Handle add comment
async function handleAddComment(event) {
    event.preventDefault();
    
    const form = event.target;
    const submitBtn = document.getElementById('submitCommentBtn');
    const commentBody = document.getElementById('commentBody');
    const token = getAuthToken();
    const articleSlug = getArticleSlug();
    
    if (!token) {
        alert('You must be logged in to comment.');
        return;
    }
    
    const body = commentBody.value.trim();
    
    if (!body) {
        alert('Please enter a comment.');
        return;
    }
    
    try {
        submitBtn.disabled = true;
        submitBtn.innerHTML = '<i class="fas fa-spinner fa-spin me-2"></i>Posting...';
        
        const response = await fetch(`/api/articles/${articleSlug}/comments`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            },
            body: JSON.stringify({
                comment: { body }
            })
        });
        
        if (!response.ok) {
            const errorData = await response.json();
            throw new Error(errorData.errors?.body?.[0] || 'Failed to post comment');
        }
        
        // Clear form and reload comments
        commentBody.value = '';
        await loadComments();
        
    } catch (error) {
        console.error('Error posting comment:', error);
        alert(error.message || 'Failed to post comment. Please try again.');
    } finally {
        submitBtn.disabled = false;
        submitBtn.innerHTML = '<i class="fas fa-paper-plane me-2"></i>Post Comment';
    }
}

// Handle delete comment
async function handleDeleteComment(event) {
    const commentId = event.currentTarget.dataset.commentId;
    const token = getAuthToken();
    const articleSlug = getArticleSlug();
    
    if (!token) {
        alert('You must be logged in to delete comments.');
        return;
    }
    
    if (!confirm('Are you sure you want to delete this comment?')) {
        return;
    }
    
    try {
        const response = await fetch(`/api/articles/${articleSlug}/comments/${commentId}`, {
            method: 'DELETE',
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });
        
        if (!response.ok) {
            throw new Error('Failed to delete comment');
        }
        
        // Reload comments
        await loadComments();
        
    } catch (error) {
        console.error('Error deleting comment:', error);
        alert('Failed to delete comment. Please try again.');
    }
}

// Article delete functionality
function confirmDeleteArticle() {
    const modal = new bootstrap.Modal(document.getElementById('deleteModal'));
    modal.show();
}

async function handleDeleteArticle() {
    const token = getAuthToken();
    const articleSlug = getArticleSlug();
    const confirmDeleteBtn = document.getElementById('confirmDeleteBtn');
    
    if (!token) {
        alert('You must be logged in to delete articles.');
        return;
    }
    
    try {
        confirmDeleteBtn.disabled = true;
        confirmDeleteBtn.innerHTML = '<i class="fas fa-spinner fa-spin me-2"></i>Deleting...';
        
        const response = await fetch(`/api/articles/${articleSlug}`, {
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
            alert(errorData.errors?.body?.[0] || 'Failed to delete article.');
            confirmDeleteBtn.disabled = false;
            confirmDeleteBtn.innerHTML = '<i class="fas fa-trash me-2"></i>Delete Article';
        }
    } catch (error) {
        alert('Network error. Please try again.');
        confirmDeleteBtn.disabled = false;
        confirmDeleteBtn.innerHTML = '<i class="fas fa-trash me-2"></i>Delete Article';
    }
}

// Initialize article page functionality
document.addEventListener('DOMContentLoaded', function() {
    // Load comments
    loadComments();
    
    // Add comment form handler
    const addCommentForm = document.getElementById('addCommentForm');
    if (addCommentForm) {
        addCommentForm.addEventListener('submit', handleAddComment);
    }
    
    // Delete article button handler
    const deleteArticleBtn = document.getElementById('deleteArticleBtn');
    if (deleteArticleBtn) {
        deleteArticleBtn.addEventListener('click', confirmDeleteArticle);
    }
    
    // Confirm delete button handler
    const confirmDeleteBtn = document.getElementById('confirmDeleteBtn');
    if (confirmDeleteBtn) {
        confirmDeleteBtn.addEventListener('click', handleDeleteArticle);
    }
});