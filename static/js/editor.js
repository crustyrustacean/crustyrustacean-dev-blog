// Article Editor JavaScript Module
document.addEventListener('DOMContentLoaded', function() {
    // Load categories on page load
    loadCategories();

    // Handle form submission
    const articleForm = document.getElementById('articleForm');
    const submitBtn = document.getElementById('submitBtn');
    const submitText = document.getElementById('submitText');
    const loadingText = document.getElementById('loadingText');
    const saveDraftBtn = document.getElementById('saveDraftBtn');
    const draftText = document.getElementById('draftText');
    const draftLoadingText = document.getElementById('draftLoadingText');

    // Handle save draft button
    if (saveDraftBtn) {
        saveDraftBtn.addEventListener('click', async function(e) {
            e.preventDefault();
            console.log('Save Draft button clicked');
            await saveArticle(true); // true = save as draft
        });
    }

    // Handle publish/update button
    if (articleForm) {
        articleForm.addEventListener('submit', async function(e) {
            e.preventDefault();
            console.log('Form submitted (Publish)');
            await saveArticle(false); // false = publish
        });
    } else {
        console.error('Article form not found!');
    }

    async function saveArticle(asDraft) {
        console.log('saveArticle called, asDraft:', asDraft);

        const targetBtn = asDraft ? saveDraftBtn : submitBtn;
        const targetText = asDraft ? draftText : submitText;
        const targetLoadingText = asDraft ? draftLoadingText : loadingText;

        if (!targetBtn) {
            console.error('Target button not found!');
            return;
        }

        // Show loading state
        targetBtn.disabled = true;
        targetText.classList.add('d-none');
        targetLoadingText.classList.remove('d-none');

        const formData = new FormData(articleForm);

        // Parse tags
        const tagsInput = formData.get('tags');
        const tagList = tagsInput ? tagsInput.split(',').map(tag => tag.trim()).filter(tag => tag.length > 0) : [];

        // Get category
        const category = formData.get('category');

        const articleData = {
            article: {
                title: formData.get('title'),
                description: formData.get('description'),
                body: formData.get('body'),
                tagList: tagList,
                draft: asDraft
            }
        };

        // Add category if selected
        if (category) {
            articleData.article.category = category;
        }

        try {
            // Get auth token from localStorage or cookies
            const token = localStorage.getItem('authToken') || getAuthTokenFromCookies();

            if (!token) {
                console.error('No auth token found');
                showError('You must be logged in to save articles.');
                resetButton(targetBtn, targetText, targetLoadingText);
                return;
            }

            const isEditMode = articleForm.getAttribute('data-edit-mode') === 'true';
            const articleSlug = articleForm.getAttribute('data-article-slug');

            const url = isEditMode ? `/api/articles/${articleSlug}` : '/api/articles';
            const method = isEditMode ? 'PUT' : 'POST';

            console.log('Making request:', { url, method, asDraft, articleData });

            const response = await fetch(url, {
                method: method,
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${token}`
                },
                body: JSON.stringify(articleData)
            });

            console.log('Response status:', response.status);

            if (response.ok) {
                const data = await response.json();

                // Show success message
                let successMessage;
                if (asDraft) {
                    successMessage = isEditMode ? 'Draft updated successfully!' : 'Draft saved successfully!';
                } else {
                    successMessage = isEditMode ? 'Article published successfully!' : 'Article published successfully!';
                }
                showSuccess(successMessage);

                // Redirect based on draft status
                setTimeout(() => {
                    if (asDraft) {
                        // Redirect to drafts admin page
                        window.location.href = '/admin/drafts';
                    } else {
                        // Redirect to the published article page
                        window.location.href = `/articles/${data.article.slug}`;
                    }
                }, 1500);
            } else {
                console.error('Request failed:', response.status);
                const errorData = await response.json().catch(() => ({ message: null }));
                console.error('Error data:', errorData);
                const errorMessage = asDraft ? 'Failed to save draft. Please try again.' : 'Failed to publish article. Please try again.';
                showError(errorData.message || errorMessage);
                resetButton(targetBtn, targetText, targetLoadingText);
            }
        } catch (error) {
            console.error('Fetch error:', error);
            showError('Network error. Please check your connection and try again.');
            resetButton(targetBtn, targetText, targetLoadingText);
        }
    }

    function resetButton(btn, textEl, loadingEl) {
        btn.disabled = false;
        textEl.classList.remove('d-none');
        loadingEl.classList.add('d-none');
    }

    function resetSubmitButton() {
        resetButton(submitBtn, submitText, loadingText);
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
        // Remove existing dynamic alerts (draft mode warning doesn't have alert-dismissible)
        const existingAlert = document.querySelector('.alert.alert-dismissible');
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

// Load categories from API and populate dropdown
async function loadCategories() {
    try {
        const response = await fetch('/api/categories');
        const data = await response.json();

        const categorySelect = document.getElementById('category');
        if (!categorySelect) return;

        // Keep the "No category" option
        categorySelect.innerHTML = '<option value="">No category</option>';

        // Add each category as an option
        data.categories.forEach(category => {
            const option = document.createElement('option');
            option.value = category.slug;
            option.textContent = category.name;
            categorySelect.appendChild(option);
        });

        // If we're in edit mode and there's a current category, select it
        loadCurrentCategory();
    } catch (error) {
        console.error('Error loading categories:', error);
    }
}

// Load and select the current category if editing an article
async function loadCurrentCategory() {
    const articleForm = document.getElementById('articleForm');
    const isEditMode = articleForm.getAttribute('data-edit-mode') === 'true';

    if (!isEditMode) return;

    const articleSlug = articleForm.getAttribute('data-article-slug');
    if (!articleSlug) return;

    try {
        const response = await fetch(`/api/articles/${articleSlug}`);
        if (!response.ok) return;

        const data = await response.json();
        const category = data.article.category;

        if (category) {
            const categorySelect = document.getElementById('category');
            if (categorySelect) {
                categorySelect.value = category;
            }
        }
    } catch (error) {
        console.error('Error loading current category:', error);
    }
}