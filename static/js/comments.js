// comments.js - Comment management functionality

/**
 * Comment manager class
 */
class CommentManager {
    constructor(articleSlug, currentUser = null) {
        this.articleSlug = articleSlug;
        this.currentUser = currentUser;
        this.commentsList = document.getElementById('commentsList');
        this.init();
    }

    init() {
        this.loadComments();
        this.setupFormHandler();
    }

    /**
     * Load and display comments
     */
    async loadComments() {
        if (!this.commentsList) return;

        try {
            const response = await fetch(`/api/articles/${this.articleSlug}/comments`);
            
            if (!response.ok) {
                throw new Error('Failed to load comments');
            }
            
            const data = await response.json();
            const comments = data.comments || [];
            
            this.renderComments(comments);
            
        } catch (error) {
            console.error('Error loading comments:', error);
            this.commentsList.innerHTML = `
                <div class="alert alert-danger" role="alert">
                    <i class="fas fa-exclamation-circle me-2"></i>
                    Failed to load comments. Please refresh the page to try again.
                </div>
            `;
        }
    }

    /**
     * Render comments HTML
     */
    renderComments(comments) {
        if (comments.length === 0) {
            this.commentsList.innerHTML = `
                <div class="text-center text-muted py-4">
                    <i class="fas fa-comment-slash fa-2x mb-2"></i>
                    <p>No comments yet. Be the first to comment!</p>
                </div>
            `;
            return;
        }

        let commentsHtml = '';
        for (const comment of comments) {
            commentsHtml += this.renderSingleComment(comment);
        }
        
        this.commentsList.innerHTML = commentsHtml;
        this.setupDeleteHandlers();
    }

    /**
     * Render a single comment
     */
    renderSingleComment(comment) {
        const canDelete = this.currentUser === comment.author.username;
        const avatarHtml = comment.author.image 
            ? `<img src="${comment.author.image}" alt="${comment.author.username}" class="rounded-circle" width="40" height="40">`
            : `<div class="bg-secondary rounded-circle d-flex align-items-center justify-content-center" style="width: 40px; height: 40px;">
                 <span class="text-white fw-bold">${comment.author.username.charAt(0).toUpperCase()}</span>
               </div>`;
        
        const deleteButton = canDelete 
            ? `<button class="btn btn-sm btn-outline-danger delete-comment-btn" data-comment-id="${comment.id}">
                 <i class="fas fa-trash"></i>
               </button>`
            : '';
        
        return `
            <div class="card mb-3" data-comment-id="${comment.id}">
                <div class="card-body">
                    <div class="d-flex">
                        <div class="me-3">${avatarHtml}</div>
                        <div class="flex-grow-1">
                            <div class="d-flex justify-content-between align-items-start mb-2">
                                <div>
                                    <strong>${comment.author.username}</strong>
                                    <span class="text-muted small ms-2">${formatDate(comment.createdAt)}</span>
                                </div>
                                ${deleteButton}
                            </div>
                            <p class="mb-0">${this.escapeHtml(comment.body)}</p>
                        </div>
                    </div>
                </div>
            </div>
        `;
    }

    /**
     * Setup form submission handler
     */
    setupFormHandler() {
        const form = document.getElementById('addCommentForm');
        if (form) {
            form.addEventListener('submit', (e) => this.handleAddComment(e));
        }
    }

    /**
     * Setup delete button handlers
     */
    setupDeleteHandlers() {
        document.querySelectorAll('.delete-comment-btn').forEach(btn => {
            btn.addEventListener('click', (e) => this.handleDeleteComment(e));
        });
    }

    /**
     * Handle adding a new comment
     */
    async handleAddComment(event) {
        event.preventDefault();
        
        const form = event.target;
        const submitBtn = document.getElementById('submitCommentBtn');
        const commentBody = document.getElementById('commentBody');
        const token = getAuthToken();
        
        if (!token) {
            showError('You must be logged in to comment.');
            return;
        }
        
        const body = commentBody.value.trim();
        if (!body) {
            showError('Please enter a comment.');
            return;
        }
        
        try {
            showButtonLoading(submitBtn, 'Posting...');
            
            const response = await fetch(`/api/articles/${this.articleSlug}/comments`, {
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
            await this.loadComments();
            
        } catch (error) {
            console.error('Error posting comment:', error);
            showError(error.message || 'Failed to post comment. Please try again.');
        } finally {
            resetButtonLoading(submitBtn);
        }
    }

    /**
     * Handle deleting a comment
     */
    async handleDeleteComment(event) {
        const commentId = event.currentTarget.dataset.commentId;
        const token = getAuthToken();
        
        if (!token) {
            showError('You must be logged in to delete comments.');
            return;
        }
        
        if (!confirm('Are you sure you want to delete this comment?')) {
            return;
        }
        
        try {
            const response = await fetch(`/api/articles/${this.articleSlug}/comments/${commentId}`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${token}`
                }
            });
            
            if (!response.ok) {
                throw new Error('Failed to delete comment');
            }
            
            // Reload comments
            await this.loadComments();
            
        } catch (error) {
            console.error('Error deleting comment:', error);
            showError('Failed to delete comment. Please try again.');
        }
    }

    /**
     * Escape HTML to prevent XSS
     */
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
}

// Export for use in templates
window.CommentManager = CommentManager;