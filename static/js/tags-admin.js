// static/js/tags-admin.js

let currentTagToDelete = null;

document.addEventListener('DOMContentLoaded', () => {
    loadTags();
    setupEventListeners();
});

async function loadTags() {
    try {
        const response = await fetch('/api/tags');
        const data = await response.json();
        
        const tagsList = document.getElementById('tags-list');
        tagsList.innerHTML = '';

        if (data.tags.length === 0) {
            tagsList.innerHTML = '<tr><td colspan="3" class="text-center">No tags found</td></tr>';
            return;
        }

        // Get article counts for each tag
        const articleCounts = await getArticleCountsForTags(data.tags);

        data.tags.forEach(tag => {
            const row = document.createElement('tr');
            const count = articleCounts[tag] || 0;
            
            row.innerHTML = `
                <td><span class="badge bg-secondary">${escapeHtml(tag)}</span></td>
                <td>${count} article${count !== 1 ? 's' : ''}</td>
                <td>
                    <button class="btn btn-sm btn-primary edit-tag-btn" data-tag="${escapeHtml(tag)}">
                        <i class="bi bi-pencil"></i> Edit
                    </button>
                    <button class="btn btn-sm btn-danger delete-tag-btn" data-tag="${escapeHtml(tag)}">
                        <i class="bi bi-trash"></i> Delete
                    </button>
                </td>
            `;
            tagsList.appendChild(row);
        });

        // Attach event listeners to new buttons
        document.querySelectorAll('.edit-tag-btn').forEach(btn => {
            btn.addEventListener('click', () => openEditModal(btn.dataset.tag));
        });

        document.querySelectorAll('.delete-tag-btn').forEach(btn => {
            btn.addEventListener('click', () => openDeleteModal(btn.dataset.tag));
        });

    } catch (error) {
        console.error('Error loading tags:', error);
        showAlert('Failed to load tags', 'danger');
    }
}

async function getArticleCountsForTags(tags) {
    const counts = {};
    
    try {
        // Fetch articles and count tags
        const response = await fetch('/api/articles?limit=100');
        const data = await response.json();
        
        tags.forEach(tag => counts[tag] = 0);
        
        data.articles.forEach(article => {
            article.tagList.forEach(tag => {
                if (counts.hasOwnProperty(tag)) {
                    counts[tag]++;
                }
            });
        });
    } catch (error) {
        console.error('Error getting article counts:', error);
    }
    
    return counts;
}

function setupEventListeners() {
    document.getElementById('saveTagBtn').addEventListener('click', saveTag);
    document.getElementById('confirmDeleteBtn').addEventListener('click', deleteTag);
}

function openEditModal(tagName) {
    document.getElementById('oldTagName').value = tagName;
    document.getElementById('newTagName').value = tagName;
    const modal = new bootstrap.Modal(document.getElementById('editTagModal'));
    modal.show();
}

function openDeleteModal(tagName) {
    currentTagToDelete = tagName;
    document.getElementById('deleteTagName').textContent = tagName;
    const modal = new bootstrap.Modal(document.getElementById('deleteTagModal'));
    modal.show();
}

async function saveTag() {
    const oldName = document.getElementById('oldTagName').value;
    const newName = document.getElementById('newTagName').value.trim();

    if (!newName) {
        showAlert('Tag name cannot be empty', 'danger');
        return;
    }

    if (oldName === newName) {
        bootstrap.Modal.getInstance(document.getElementById('editTagModal')).hide();
        return;
    }

    try {
        const token = localStorage.getItem('authToken');
        if (!token) {
            window.location.href = '/login';
            return;
        }

        const response = await fetch(`/api/tags/${encodeURIComponent(oldName)}`, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            },
            body: JSON.stringify({
                tag: { name: newName }
            })
        });

        if (response.ok) {
            showAlert(`Tag renamed from "${oldName}" to "${newName}"`, 'success');
            bootstrap.Modal.getInstance(document.getElementById('editTagModal')).hide();
            loadTags();
        } else {
            const error = await response.json();
            showAlert(error.errors?.body?.[0] || 'Failed to update tag', 'danger');
        }
    } catch (error) {
        console.error('Error updating tag:', error);
        showAlert('Failed to update tag', 'danger');
    }
}

async function deleteTag() {
    if (!currentTagToDelete) return;

    try {
        const token = localStorage.getItem('authToken');
        if (!token) {
            window.location.href = '/login';
            return;
        }

        const response = await fetch(`/api/tags/${encodeURIComponent(currentTagToDelete)}`, {
            method: 'DELETE',
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });

        if (response.ok) {
            showAlert(`Tag "${currentTagToDelete}" deleted successfully`, 'success');
            bootstrap.Modal.getInstance(document.getElementById('deleteTagModal')).hide();
            currentTagToDelete = null;
            loadTags();
        } else {
            const error = await response.json();
            showAlert(error.errors?.body?.[0] || 'Failed to delete tag', 'danger');
        }
    } catch (error) {
        console.error('Error deleting tag:', error);
        showAlert('Failed to delete tag', 'danger');
    }
}

function showAlert(message, type) {
    const alertDiv = document.createElement('div');
    alertDiv.className = `alert alert-${type} alert-dismissible fade show position-fixed top-0 start-50 translate-middle-x mt-3`;
    alertDiv.style.zIndex = '9999';
    alertDiv.innerHTML = `
        ${message}
        <button type="button" class="btn-close" data-bs-dismiss="alert"></button>
    `;
    document.body.appendChild(alertDiv);

    setTimeout(() => {
        alertDiv.remove();
    }, 5000);
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}