// API Keys Management JavaScript

// Set the upload URL
document.addEventListener('DOMContentLoaded', () => {
    const uploadUrl = `${window.location.origin}/api/mobile/upload`;
    const uploadUrlElement = document.getElementById('uploadUrl');
    if (uploadUrlElement) {
        uploadUrlElement.textContent = uploadUrl;
    }

    loadApiKeys();
});

// Load API keys
async function loadApiKeys() {
    try {
        const response = await fetch('/api/keys', {
            method: 'GET',
            headers: {
                'Authorization': `Bearer ${getAuthToken()}`,
            },
        });

        if (!response.ok) {
            throw new Error('Failed to load API keys');
        }

        const data = await response.json();
        displayApiKeys(data.api_keys);
    } catch (error) {
        console.error('Error loading API keys:', error);
        displayError('Failed to load API keys. Please try refreshing the page.');
    }
}

// Display API keys
function displayApiKeys(apiKeys) {
    const container = document.getElementById('apiKeysList');

    if (!apiKeys || apiKeys.length === 0) {
        container.innerHTML = `
            <div class="text-center py-5">
                <i class="fas fa-key fa-5x text-muted mb-3"></i>
                <h3 class="text-muted mb-3">No API Keys Yet</h3>
                <p class="text-muted mb-4">Generate your first API key to start posting from your phone.</p>
                <button type="button" class="btn btn-primary btn-lg" data-bs-toggle="modal" data-bs-target="#createKeyModal">
                    <i class="fas fa-plus me-2"></i>
                    Generate Your First Key
                </button>
            </div>
        `;
        return;
    }

    const html = `
        <div class="table-responsive">
            <table class="table table-striped table-hover">
                <thead class="table-dark">
                    <tr>
                        <th scope="col">Name</th>
                        <th scope="col">Created</th>
                        <th scope="col">Last Used</th>
                        <th scope="col">Actions</th>
                    </tr>
                </thead>
                <tbody>
                    ${apiKeys.map(key => `
                        <tr>
                            <td>
                                <strong>${escapeHtml(key.name)}</strong>
                            </td>
                            <td>
                                <small>${formatDate(key.created_at)}</small>
                            </td>
                            <td>
                                <small>${key.last_used_at ? formatDate(key.last_used_at) : '<span class="text-muted">Never</span>'}</small>
                            </td>
                            <td>
                                <button type="button" class="btn btn-sm btn-outline-danger delete-key-btn"
                                        data-key-id="${key.id}"
                                        data-key-name="${escapeHtml(key.name)}">
                                    <i class="fas fa-trash"></i>
                                    Delete
                                </button>
                            </td>
                        </tr>
                    `).join('')}
                </tbody>
            </table>
        </div>
    `;

    container.innerHTML = html;

    // Add event listeners to delete buttons
    document.querySelectorAll('.delete-key-btn').forEach(btn => {
        btn.addEventListener('click', handleDeleteClick);
    });
}

// Handle delete button click
function handleDeleteClick(event) {
    const keyId = event.currentTarget.getAttribute('data-key-id');
    const keyName = event.currentTarget.getAttribute('data-key-name');

    document.getElementById('keyName').textContent = keyName;

    const modal = new bootstrap.Modal(document.getElementById('deleteKeyModal'));
    modal.show();

    // Set up the confirm button
    const confirmBtn = document.getElementById('confirmDeleteKeyBtn');
    confirmBtn.onclick = () => deleteApiKey(keyId, modal);
}

// Delete API key
async function deleteApiKey(keyId, modal) {
    try {
        const response = await fetch(`/api/keys/${keyId}`, {
            method: 'DELETE',
            headers: {
                'Authorization': `Bearer ${getAuthToken()}`,
            },
        });

        if (!response.ok) {
            throw new Error('Failed to delete API key');
        }

        // Close modal and reload keys
        modal.hide();
        showSuccess('API key deleted successfully');
        loadApiKeys();
    } catch (error) {
        console.error('Error deleting API key:', error);
        showError('Failed to delete API key. Please try again.');
    }
}

// Create new API key
document.getElementById('createKeyBtn').addEventListener('click', async () => {
    const keyName = document.getElementById('keyName').value.trim();

    if (!keyName) {
        showError('Please enter a name for your API key');
        return;
    }

    try {
        const response = await fetch('/api/keys', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${getAuthToken()}`,
            },
            body: JSON.stringify({ name: keyName }),
        });

        if (!response.ok) {
            throw new Error('Failed to create API key');
        }

        const data = await response.json();

        // Hide create modal
        const createModal = bootstrap.Modal.getInstance(document.getElementById('createKeyModal'));
        createModal.hide();

        // Clear form
        document.getElementById('keyName').value = '';

        // Show the new key
        document.getElementById('newApiKey').value = data.key;
        const showModal = new bootstrap.Modal(document.getElementById('showKeyModal'));
        showModal.show();

        // Reload keys after modal is hidden
        document.getElementById('showKeyModal').addEventListener('hidden.bs.modal', () => {
            loadApiKeys();
        }, { once: true });

    } catch (error) {
        console.error('Error creating API key:', error);
        showError('Failed to create API key. Please try again.');
    }
});

// Copy API key to clipboard
document.getElementById('copyKeyBtn').addEventListener('click', async () => {
    const keyInput = document.getElementById('newApiKey');

    try {
        await navigator.clipboard.writeText(keyInput.value);

        // Change button text temporarily
        const btn = document.getElementById('copyKeyBtn');
        const originalHTML = btn.innerHTML;
        btn.innerHTML = '<i class="fas fa-check me-1"></i>Copied!';
        btn.classList.remove('btn-outline-secondary');
        btn.classList.add('btn-success');

        setTimeout(() => {
            btn.innerHTML = originalHTML;
            btn.classList.remove('btn-success');
            btn.classList.add('btn-outline-secondary');
        }, 2000);
    } catch (error) {
        console.error('Failed to copy:', error);
        // Fallback for older browsers
        keyInput.select();
        document.execCommand('copy');
        showSuccess('API key copied to clipboard');
    }
});

// Helper: Get auth token from cookie
function getAuthToken() {
    const name = 'authToken=';
    const decodedCookie = decodeURIComponent(document.cookie);
    const cookies = decodedCookie.split(';');

    for (let cookie of cookies) {
        cookie = cookie.trim();
        if (cookie.indexOf(name) === 0) {
            return cookie.substring(name.length);
        }
    }
    return '';
}

// Helper: Format date
function formatDate(dateString) {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now - date;
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) {
        const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
        if (diffHours === 0) {
            const diffMins = Math.floor(diffMs / (1000 * 60));
            return diffMins === 0 ? 'Just now' : `${diffMins} minute${diffMins === 1 ? '' : 's'} ago`;
        }
        return `${diffHours} hour${diffHours === 1 ? '' : 's'} ago`;
    } else if (diffDays === 1) {
        return 'Yesterday';
    } else if (diffDays < 7) {
        return `${diffDays} days ago`;
    } else {
        return date.toLocaleDateString();
    }
}

// Helper: Escape HTML
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Helper: Display error
function displayError(message) {
    const container = document.getElementById('apiKeysList');
    container.innerHTML = `
        <div class="alert alert-danger" role="alert">
            <i class="fas fa-exclamation-circle me-2"></i>
            ${escapeHtml(message)}
        </div>
    `;
}

// Helper: Show success message
function showSuccess(message) {
    // You can implement a toast notification here
    console.log('Success:', message);
}

// Helper: Show error message
function showError(message) {
    // You can implement a toast notification here
    alert(message);
}
