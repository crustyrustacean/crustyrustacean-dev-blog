// Article Editor JavaScript Module
document.addEventListener('DOMContentLoaded', function() {
    // Handle form submission
    const articleForm = document.getElementById('articleForm');
    const submitBtn = document.getElementById('submitBtn');
    const submitText = document.getElementById('submitText');
    const loadingText = document.getElementById('loadingText');

    articleForm.addEventListener('submit', async function(e) {
        e.preventDefault();
        
        // Show loading state
        submitBtn.disabled = true;
        submitText.classList.add('d-none');
        loadingText.classList.remove('d-none');

        const formData = new FormData(this);
        
        // Parse tags
        const tagsInput = formData.get('tags');
        const tagList = tagsInput ? tagsInput.split(',').map(tag => tag.trim()).filter(tag => tag.length > 0) : [];

        const articleData = {
            article: {
                title: formData.get('title'),
                description: formData.get('description'),
                body: formData.get('body'),
                tagList: tagList
            }
        };

        try {
            // Get auth token from localStorage or cookies
            const token = localStorage.getItem('authToken') || getAuthTokenFromCookies();
            
            if (!token) {
                showError('You must be logged in to publish articles.');
                resetSubmitButton();
                return;
            }

            const isEditMode = articleForm.getAttribute('data-edit-mode') === 'true';
            const articleSlug = articleForm.getAttribute('data-article-slug');
            
            const url = isEditMode ? `/api/articles/${articleSlug}` : '/api/articles';
            const method = isEditMode ? 'PUT' : 'POST';

            const response = await fetch(url, {
                method: method,
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${token}`
                },
                body: JSON.stringify(articleData)
            });

            if (response.ok) {
                const data = await response.json();
                // Show success message
                const successMessage = isEditMode ? 'Article updated successfully!' : 'Article published successfully!';
                showSuccess(successMessage);
                
                // Redirect to the article page after a short delay
                setTimeout(() => {
                    window.location.href = `/articles/${data.article.slug}`;
                }, 1500);
            } else {
                const errorData = await response.json();
                const errorMessage = isEditMode ? 'Failed to update article. Please try again.' : 'Failed to publish article. Please try again.';
                showError(errorData.message || errorMessage);
                resetSubmitButton();
            }
        } catch (error) {
            showError('Network error. Please check your connection and try again.');
            resetSubmitButton();
        }
    });

    function resetSubmitButton() {
        submitBtn.disabled = false;
        submitText.classList.remove('d-none');
        loadingText.classList.add('d-none');
    }

    function getAuthTokenFromCookies() {
        // Simple cookie parsing - in production you might want to use a library
        const cookies = document.cookie.split(';');
        for (let cookie of cookies) {
            const [name, value] = cookie.trim().split('=');
            if (name === 'authToken') {
                return decodeURIComponent(value);
            }
        }
        return null;
    }

    function showError(message) {
        showAlert(message, 'danger');
    }

    function showSuccess(message) {
        showAlert(message, 'success');
    }

    function showAlert(message, type) {
        // Remove existing alerts
        const existingAlert = document.querySelector('.alert');
        if (existingAlert) {
            existingAlert.remove();
        }

        // Create new alert
        const alert = document.createElement('div');
        alert.className = `alert alert-${type} alert-dismissible fade show`;
        alert.innerHTML = `
            <i class="fas fa-${type === 'danger' ? 'exclamation-triangle' : 'check-circle'} me-2"></i>
            ${message}
            <button type="button" class="btn-close" data-bs-dismiss="alert"></button>
        `;

        // Insert before form
        const form = document.getElementById('articleForm');
        form.parentNode.insertBefore(alert, form);

        // Auto-dismiss success messages after 3 seconds
        if (type === 'success') {
            setTimeout(() => {
                if (alert.parentNode) {
                    const bsAlert = new bootstrap.Alert(alert);
                    bsAlert.close();
                }
            }, 3000);
        }
    }

    // Character counter for title and description
    const titleInput = document.getElementById('title');
    const descriptionInput = document.getElementById('description');

    function addCharacterCounter(input, maxLength) {
        const counter = document.createElement('div');
        counter.className = 'form-text text-end';
        counter.style.marginTop = '0.25rem';
        
        function updateCounter() {
            const remaining = maxLength - input.value.length;
            counter.textContent = `${remaining} characters remaining`;
            counter.classList.toggle('text-danger', remaining < 10);
        }
        
        input.addEventListener('input', updateCounter);
        updateCounter();
        
        input.parentNode.appendChild(counter);
    }

    addCharacterCounter(titleInput, 200);
    addCharacterCounter(descriptionInput, 500);

    // Handle publish from preview button
    const publishFromPreviewBtn = document.getElementById('publishFromPreviewBtn');
    if (publishFromPreviewBtn) {
        publishFromPreviewBtn.addEventListener('click', publishFromPreview);
    }
});

// Function to be called from preview modal
function publishFromPreview() {
    const modal = bootstrap.Modal.getInstance(document.getElementById('previewModal'));
    modal.hide();
    document.getElementById('articleForm').dispatchEvent(new Event('submit'));
}