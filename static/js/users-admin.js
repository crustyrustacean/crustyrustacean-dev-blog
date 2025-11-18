// static/js/users-admin.js

let currentUserToDelete = null;
let currentFilters = {
    search: '',
    status: 'active',
};

document.addEventListener('DOMContentLoaded', () => {
    loadUsers();
    setupEventListeners();
});

function setupEventListeners() {
    // Apply filters button
    document.getElementById('applyFilters').addEventListener('click', () => {
        currentFilters.search = document.getElementById('searchInput').value.trim();
        currentFilters.status = document.getElementById('statusFilter').value;
        loadUsers();
    });

    // Enter key in search input
    document.getElementById('searchInput').addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            document.getElementById('applyFilters').click();
        }
    });

    // Save user button
    document.getElementById('saveUserBtn').addEventListener('click', saveUser);

    // Delete user button
    document.getElementById('confirmDeleteBtn').addEventListener('click', deleteUser);
}

async function loadUsers() {
    try {
        const loadingIndicator = document.getElementById('loading-indicator');
        const usersList = document.getElementById('users-list');

        loadingIndicator.style.display = 'block';
        usersList.innerHTML = '';

        const token = getAuthToken();
        if (!token) {
            window.location.href = '/login';
            return;
        }

        // Build query parameters
        const params = new URLSearchParams();
        if (currentFilters.search) {
            params.append('search', currentFilters.search);
        }
        if (currentFilters.status && currentFilters.status !== 'all') {
            params.append('status', currentFilters.status);
        }
        params.append('limit', '100'); // Load more users for admin view

        const response = await fetch(`/api/admin/users?${params}`, {
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });

        if (!response.ok) {
            throw new Error('Failed to load users');
        }

        const result = await response.json();
        const users = result.data.users;
        const usersCount = result.data.usersCount;

        // Update statistics
        document.getElementById('total-users').textContent = usersCount;

        // Calculate active/disabled counts
        let activeCount = 0;
        let disabledCount = 0;
        users.forEach(user => {
            if (user.disabled) {
                disabledCount++;
            } else {
                activeCount++;
            }
        });
        document.getElementById('active-users').textContent = activeCount;
        document.getElementById('disabled-users').textContent = disabledCount;

        loadingIndicator.style.display = 'none';

        if (users.length === 0) {
            usersList.innerHTML = '<tr><td colspan="6" class="text-center">No users found</td></tr>';
            return;
        }

        users.forEach(user => {
            const row = document.createElement('tr');
            const createdDate = new Date(user.createdAt).toLocaleDateString();
            const statusBadge = user.disabled
                ? '<span class="badge bg-danger">Disabled</span>'
                : '<span class="badge bg-success">Active</span>';

            row.innerHTML = `
                <td><strong>${escapeHtml(user.username)}</strong></td>
                <td>${escapeHtml(user.email)}</td>
                <td>${user.articleCount}</td>
                <td>${createdDate}</td>
                <td>${statusBadge}</td>
                <td>
                    <button class="btn btn-sm btn-primary edit-user-btn"
                            data-id="${escapeHtml(user.id)}"
                            data-username="${escapeHtml(user.username)}"
                            data-email="${escapeHtml(user.email)}"
                            data-bio="${user.bio ? escapeHtml(user.bio) : ''}"
                            data-image="${user.image ? escapeHtml(user.image) : ''}"
                            data-disabled="${user.disabled}">
                        <i class="bi bi-pencil"></i> Edit
                    </button>
                    ${user.disabled
                        ? `<button class="btn btn-sm btn-success enable-user-btn" data-id="${escapeHtml(user.id)}">
                               <i class="bi bi-check-circle"></i> Enable
                           </button>`
                        : `<button class="btn btn-sm btn-warning disable-user-btn" data-id="${escapeHtml(user.id)}">
                               <i class="bi bi-ban"></i> Disable
                           </button>`
                    }
                    <button class="btn btn-sm btn-danger delete-user-btn"
                            data-id="${escapeHtml(user.id)}"
                            data-username="${escapeHtml(user.username)}">
                        <i class="bi bi-trash"></i> Delete
                    </button>
                </td>
            `;
            usersList.appendChild(row);
        });

        // Attach event listeners to new buttons
        document.querySelectorAll('.edit-user-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                openEditModal(
                    btn.dataset.id,
                    btn.dataset.username,
                    btn.dataset.email,
                    btn.dataset.bio,
                    btn.dataset.image,
                    btn.dataset.disabled === 'true'
                );
            });
        });

        document.querySelectorAll('.disable-user-btn').forEach(btn => {
            btn.addEventListener('click', () => toggleUserStatus(btn.dataset.id, true));
        });

        document.querySelectorAll('.enable-user-btn').forEach(btn => {
            btn.addEventListener('click', () => toggleUserStatus(btn.dataset.id, false));
        });

        document.querySelectorAll('.delete-user-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                openDeleteModal(btn.dataset.id, btn.dataset.username);
            });
        });

    } catch (error) {
        console.error('Error loading users:', error);
        document.getElementById('loading-indicator').style.display = 'none';
        showAlert('Failed to load users', 'danger');
    }
}

function openEditModal(id, username, email, bio, image, disabled) {
    document.getElementById('editUserId').value = id;
    document.getElementById('editUsername').value = username;
    document.getElementById('editEmail').value = email;
    document.getElementById('editBio').value = bio;
    document.getElementById('editImage').value = image;
    const modal = new bootstrap.Modal(document.getElementById('editUserModal'));
    modal.show();
}

function openDeleteModal(id, username) {
    currentUserToDelete = id;
    document.getElementById('deleteUsername').textContent = username;
    const modal = new bootstrap.Modal(document.getElementById('deleteUserModal'));
    modal.show();
}

async function saveUser() {
    const id = document.getElementById('editUserId').value;
    const username = document.getElementById('editUsername').value.trim();
    const email = document.getElementById('editEmail').value.trim();
    const bio = document.getElementById('editBio').value.trim();
    const image = document.getElementById('editImage').value.trim();

    if (!username || !email) {
        showAlert('Username and email are required', 'danger');
        return;
    }

    try {
        const token = getAuthToken();
        if (!token) {
            window.location.href = '/login';
            return;
        }

        const response = await fetch(`/api/admin/users/${encodeURIComponent(id)}`, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            },
            body: JSON.stringify({
                username,
                email,
                bio: bio || undefined,
                image: image || undefined
            })
        });

        if (response.ok) {
            showAlert('User updated successfully', 'success');
            bootstrap.Modal.getInstance(document.getElementById('editUserModal')).hide();
            loadUsers();
        } else {
            const error = await response.json();
            showAlert(error.errors?.body?.[0] || 'Failed to update user', 'danger');
        }
    } catch (error) {
        console.error('Error updating user:', error);
        showAlert('Failed to update user', 'danger');
    }
}

async function toggleUserStatus(id, disabled) {
    const action = disabled ? 'disable' : 'enable';
    if (!confirm(`Are you sure you want to ${action} this user?`)) {
        return;
    }

    try {
        const token = getAuthToken();
        if (!token) {
            window.location.href = '/login';
            return;
        }

        const response = await fetch(`/api/admin/users/${encodeURIComponent(id)}`, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            },
            body: JSON.stringify({
                disabled
            })
        });

        if (response.ok) {
            showAlert(`User ${action}d successfully`, 'success');
            loadUsers();
        } else {
            const error = await response.json();
            showAlert(error.errors?.body?.[0] || `Failed to ${action} user`, 'danger');
        }
    } catch (error) {
        console.error(`Error ${action}ing user:`, error);
        showAlert(`Failed to ${action} user`, 'danger');
    }
}

async function deleteUser() {
    if (!currentUserToDelete) return;

    try {
        const token = getAuthToken();
        if (!token) {
            window.location.href = '/login';
            return;
        }

        const response = await fetch(`/api/admin/users/${encodeURIComponent(currentUserToDelete)}`, {
            method: 'DELETE',
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });

        if (response.ok) {
            showAlert('User deleted successfully', 'success');
            bootstrap.Modal.getInstance(document.getElementById('deleteUserModal')).hide();
            currentUserToDelete = null;
            loadUsers();
        } else {
            const error = await response.json();
            showAlert(error.errors?.body?.[0] || 'Failed to delete user', 'danger');
        }
    } catch (error) {
        console.error('Error deleting user:', error);
        showAlert('Failed to delete user', 'danger');
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
