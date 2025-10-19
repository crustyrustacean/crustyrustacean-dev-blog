// utils.js - Core utility functions

/**
 * Get authentication token from localStorage or cookies
 */
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

/**
 * Get cookie value by name
 */
function getCookie(name) {
    const value = `; ${document.cookie}`;
    const parts = value.split(`; ${name}=`);
    if (parts.length === 2) return parts.pop().split(';').shift();
}

/**
 * Format date for display
 */
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

/**
 * Show loading state for a button
 */
function showButtonLoading(button, loadingText = 'Loading...') {
    button.disabled = true;
    button.dataset.originalText = button.innerHTML;
    button.innerHTML = `<i class="fas fa-spinner fa-spin me-2"></i>${loadingText}`;
}

/**
 * Reset button from loading state
 */
function resetButtonLoading(button) {
    button.disabled = false;
    if (button.dataset.originalText) {
        button.innerHTML = button.dataset.originalText;
    }
}

/**
 * Display error message to user
 */
function showError(message) {
    // Could be enhanced with toast notifications in the future
    alert(message);
}

/**
 * Display success message to user
 */
function showSuccess(message) {
    // Could be enhanced with toast notifications in the future
    alert(message);
}