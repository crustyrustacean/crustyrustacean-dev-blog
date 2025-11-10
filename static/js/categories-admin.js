// static/js/categories-admin.js

let currentCategoryToDelete = null;

document.addEventListener('DOMContentLoaded', () => {
    loadCategories();
    setupEventListeners();
});

async function loadCategories() {
    try {
        const response = await fetch('/api/categories');
        const data = await response.json();

        const categoriesList = document.getElementById('categories-list');
        categoriesList.innerHTML = '';

        if (data.categories.length === 0) {
            categoriesList.innerHTML = '<tr><td colspan="5" class="text-center">No categories found</td></tr>';
            return;
        }

        data.categories.forEach(category => {
            const row = document.createElement('tr');

            row.innerHTML = `
                <td><strong>${escapeHtml(category.name)}</strong></td>
                <td><code>${escapeHtml(category.slug)}</code></td>
                <td>${category.description ? escapeHtml(category.description) : '<em class="text-muted">No description</em>'}</td>
                <td>${category.articleCount} article${category.articleCount !== 1 ? 's' : ''}</td>
                <td>
                    <button class="btn btn-sm btn-primary edit-category-btn"
                            data-slug="${escapeHtml(category.slug)}"
                            data-name="${escapeHtml(category.name)}"
                            data-description="${category.description ? escapeHtml(category.description) : ''}">
                        <i class="bi bi-pencil"></i> Edit
                    </button>
                    <button class="btn btn-sm btn-danger delete-category-btn"
                            data-slug="${escapeHtml(category.slug)}"
                            data-name="${escapeHtml(category.name)}">
                        <i class="bi bi-trash"></i> Delete
                    </button>
                </td>
            `;
            categoriesList.appendChild(row);
        });

        // Attach event listeners to new buttons
        document.querySelectorAll('.edit-category-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                openEditModal(btn.dataset.slug, btn.dataset.name, btn.dataset.description);
            });
        });

        document.querySelectorAll('.delete-category-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                openDeleteModal(btn.dataset.slug, btn.dataset.name);
            });
        });

    } catch (error) {
        console.error('Error loading categories:', error);
        showAlert('Failed to load categories', 'danger');
    }
}

function setupEventListeners() {
    // Create category form
    document.getElementById('createCategoryForm').addEventListener('submit', createCategory);

    // Edit category modal
    document.getElementById('saveCategoryBtn').addEventListener('click', saveCategory);

    // Delete category modal
    document.getElementById('confirmDeleteBtn').addEventListener('click', deleteCategory);
}

async function createCategory(e) {
    e.preventDefault();

    const name = document.getElementById('categoryName').value.trim();
    const description = document.getElementById('categoryDescription').value.trim();

    if (!name) {
        showAlert('Category name cannot be empty', 'danger');
        return;
    }

    try {
        const token = localStorage.getItem('authToken');
        if (!token) {
            window.location.href = '/login';
            return;
        }

        const response = await fetch('/api/categories', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            },
            body: JSON.stringify({
                category: {
                    name,
                    description: description || undefined
                }
            })
        });

        if (response.ok) {
            showAlert(`Category "${name}" created successfully`, 'success');
            document.getElementById('createCategoryForm').reset();
            loadCategories();
        } else {
            const error = await response.json();
            showAlert(error.errors?.body?.[0] || 'Failed to create category', 'danger');
        }
    } catch (error) {
        console.error('Error creating category:', error);
        showAlert('Failed to create category', 'danger');
    }
}

function openEditModal(slug, name, description) {
    document.getElementById('oldCategorySlug').value = slug;
    document.getElementById('editCategoryName').value = name;
    document.getElementById('editCategoryDescription').value = description;
    const modal = new bootstrap.Modal(document.getElementById('editCategoryModal'));
    modal.show();
}

function openDeleteModal(slug, name) {
    currentCategoryToDelete = slug;
    document.getElementById('deleteCategoryName').textContent = name;
    const modal = new bootstrap.Modal(document.getElementById('deleteCategoryModal'));
    modal.show();
}

async function saveCategory() {
    const oldSlug = document.getElementById('oldCategorySlug').value;
    const newName = document.getElementById('editCategoryName').value.trim();
    const newDescription = document.getElementById('editCategoryDescription').value.trim();

    if (!newName) {
        showAlert('Category name cannot be empty', 'danger');
        return;
    }

    try {
        const token = localStorage.getItem('authToken');
        if (!token) {
            window.location.href = '/login';
            return;
        }

        const response = await fetch(`/api/categories/${encodeURIComponent(oldSlug)}`, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            },
            body: JSON.stringify({
                category: {
                    name: newName,
                    description: newDescription || undefined
                }
            })
        });

        if (response.ok) {
            showAlert(`Category updated successfully`, 'success');
            bootstrap.Modal.getInstance(document.getElementById('editCategoryModal')).hide();
            loadCategories();
        } else {
            const error = await response.json();
            showAlert(error.errors?.body?.[0] || 'Failed to update category', 'danger');
        }
    } catch (error) {
        console.error('Error updating category:', error);
        showAlert('Failed to update category', 'danger');
    }
}

async function deleteCategory() {
    if (!currentCategoryToDelete) return;

    try {
        const token = localStorage.getItem('authToken');
        if (!token) {
            window.location.href = '/login';
            return;
        }

        const response = await fetch(`/api/categories/${encodeURIComponent(currentCategoryToDelete)}`, {
            method: 'DELETE',
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });

        if (response.ok) {
            showAlert(`Category deleted successfully`, 'success');
            bootstrap.Modal.getInstance(document.getElementById('deleteCategoryModal')).hide();
            currentCategoryToDelete = null;
            loadCategories();
        } else {
            const error = await response.json();
            showAlert(error.errors?.body?.[0] || 'Failed to delete category', 'danger');
        }
    } catch (error) {
        console.error('Error deleting category:', error);
        showAlert('Failed to delete category', 'danger');
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
