// static/js/media-library.js

let currentPage = 1;
let currentMediaId = null;
let allMedia = [];
let filteredMedia = [];
const itemsPerPage = 12;

document.addEventListener('DOMContentLoaded', () => {
    loadMedia();
    setupEventListeners();
});

function setupEventListeners() {
    // Upload button handlers
    document.getElementById('uploadBtn').addEventListener('click', showUploadForm);
    document.getElementById('emptyUploadBtn')?.addEventListener('click', showUploadForm);
    document.getElementById('cancelUploadBtn').addEventListener('click', hideUploadForm);

    // Form submission
    document.getElementById('uploadForm').addEventListener('submit', handleUpload);
    document.getElementById('saveMediaBtn').addEventListener('click', handleSaveMedia);
    document.getElementById('deleteMediaBtn').addEventListener('click', showDeleteConfirmation);
    document.getElementById('confirmDeleteBtn').addEventListener('click', handleDeleteMedia);

    // Copy URL button
    document.getElementById('copyUrlBtn').addEventListener('click', copyMediaUrl);
    document.getElementById('copyMarkdownBtn').addEventListener('click', copyMediaMarkdown);

    // Search and filter
    document.getElementById('searchInput').addEventListener('input', filterMedia);
    document.getElementById('mimeTypeFilter').addEventListener('change', filterMedia);
    document.getElementById('sortOrder').addEventListener('change', sortMedia);
}

function showUploadForm() {
    document.getElementById('uploadCard').style.display = 'block';
    document.getElementById('uploadBtn').style.display = 'none';
    document.getElementById('uploadForm').reset();
}

function hideUploadForm() {
    document.getElementById('uploadCard').style.display = 'none';
    document.getElementById('uploadBtn').style.display = 'inline-block';
}

async function loadMedia() {
    try {
        const token = localStorage.getItem('authToken');
        if (!token) {
            window.location.href = '/login';
            return;
        }

        document.getElementById('loadingSpinner').style.display = 'block';
        document.getElementById('mediaGrid').innerHTML = '';
        document.getElementById('emptyState').style.display = 'none';

        const response = await fetch('/api/media?limit=100', {
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });

        if (!response.ok) {
            throw new Error('Failed to load media');
        }

        const data = await response.json();
        allMedia = data.media || [];
        filteredMedia = [...allMedia];

        document.getElementById('loadingSpinner').style.display = 'none';

        if (allMedia.length === 0) {
            document.getElementById('emptyState').style.display = 'block';
        } else {
            sortMedia();
            displayMedia();
        }
    } catch (error) {
        console.error('Error loading media:', error);
        document.getElementById('loadingSpinner').style.display = 'none';
        showAlert('Failed to load media library', 'danger');
    }
}

function filterMedia() {
    const searchTerm = document.getElementById('searchInput').value.toLowerCase();
    const mimeType = document.getElementById('mimeTypeFilter').value;

    filteredMedia = allMedia.filter(media => {
        const matchesSearch = !searchTerm ||
            (media.filename?.toLowerCase().includes(searchTerm)) ||
            (media.title?.toLowerCase().includes(searchTerm)) ||
            (media.description?.toLowerCase().includes(searchTerm));

        const matchesMimeType = !mimeType || media.mime_type === mimeType;

        return matchesSearch && matchesMimeType;
    });

    currentPage = 1;
    sortMedia();
    displayMedia();
}

function sortMedia() {
    const sortOrder = document.getElementById('sortOrder').value;

    filteredMedia.sort((a, b) => {
        switch (sortOrder) {
            case 'newest':
                return new Date(b.uploaded_at) - new Date(a.uploaded_at);
            case 'oldest':
                return new Date(a.uploaded_at) - new Date(b.uploaded_at);
            case 'largest':
                return b.file_size - a.file_size;
            case 'smallest':
                return a.file_size - b.file_size;
            default:
                return 0;
        }
    });

    displayMedia();
}

function displayMedia() {
    const grid = document.getElementById('mediaGrid');
    grid.innerHTML = '';

    const startIndex = (currentPage - 1) * itemsPerPage;
    const endIndex = startIndex + itemsPerPage;
    const pageMedia = filteredMedia.slice(startIndex, endIndex);

    if (filteredMedia.length === 0) {
        grid.innerHTML = '<div class="col-12 text-center py-5"><p class="text-muted">No media files match your search criteria.</p></div>';
        document.getElementById('paginationNav').style.display = 'none';
        return;
    }

    pageMedia.forEach(media => {
        const card = createMediaCard(media);
        grid.appendChild(card);
    });

    updatePagination();
}

function createMediaCard(media) {
    const col = document.createElement('div');
    col.className = 'col-md-4 col-lg-3 mb-4';

    const fileSize = formatFileSize(media.file_size);
    const uploadDate = new Date(media.uploaded_at).toLocaleDateString();
    const title = media.title || media.filename;

    col.innerHTML = `
        <div class="card media-card h-100" data-media-id="${media.id}">
            <div class="media-card-image">
                <img src="${media.download_url}" alt="${escapeHtml(media.alt_text || title)}" class="card-img-top">
                <div class="media-card-overlay">
                    <button class="btn btn-sm btn-primary" onclick="viewMedia('${media.id}')">
                        <i class="fas fa-eye"></i> View
                    </button>
                </div>
            </div>
            <div class="card-body">
                <h6 class="card-title text-truncate" title="${escapeHtml(title)}">${escapeHtml(title)}</h6>
                <p class="card-text small text-muted mb-1">
                    <i class="fas fa-file"></i> ${fileSize}
                </p>
                <p class="card-text small text-muted mb-0">
                    <i class="fas fa-calendar"></i> ${uploadDate}
                </p>
            </div>
        </div>
    `;

    return col;
}

function updatePagination() {
    const totalPages = Math.ceil(filteredMedia.length / itemsPerPage);
    const paginationNav = document.getElementById('paginationNav');
    const pagination = document.getElementById('pagination');

    if (totalPages <= 1) {
        paginationNav.style.display = 'none';
        return;
    }

    paginationNav.style.display = 'block';
    pagination.innerHTML = '';

    // Previous button
    const prevLi = document.createElement('li');
    prevLi.className = `page-item ${currentPage === 1 ? 'disabled' : ''}`;
    prevLi.innerHTML = `<a class="page-link" href="#" onclick="changePage(${currentPage - 1}); return false;">Previous</a>`;
    pagination.appendChild(prevLi);

    // Page numbers
    for (let i = 1; i <= totalPages; i++) {
        const li = document.createElement('li');
        li.className = `page-item ${i === currentPage ? 'active' : ''}`;
        li.innerHTML = `<a class="page-link" href="#" onclick="changePage(${i}); return false;">${i}</a>`;
        pagination.appendChild(li);
    }

    // Next button
    const nextLi = document.createElement('li');
    nextLi.className = `page-item ${currentPage === totalPages ? 'disabled' : ''}`;
    nextLi.innerHTML = `<a class="page-link" href="#" onclick="changePage(${currentPage + 1}); return false;">Next</a>`;
    pagination.appendChild(nextLi);
}

function changePage(page) {
    const totalPages = Math.ceil(filteredMedia.length / itemsPerPage);
    if (page < 1 || page > totalPages) return;
    currentPage = page;
    displayMedia();
    window.scrollTo({ top: 0, behavior: 'smooth' });
}

async function handleUpload(e) {
    e.preventDefault();

    const fileInput = document.getElementById('fileInput');
    const file = fileInput.files[0];

    if (!file) {
        showAlert('Please select a file', 'warning');
        return;
    }

    const formData = new FormData();
    formData.append('file', file);

    const title = document.getElementById('titleInput').value;
    const altText = document.getElementById('altTextInput').value;
    const caption = document.getElementById('captionInput').value;
    const description = document.getElementById('descriptionInput').value;

    if (title) formData.append('title', title);
    if (altText) formData.append('alt_text', altText);
    if (caption) formData.append('caption', caption);
    if (description) formData.append('description', description);

    try {
        const token = localStorage.getItem('authToken');
        if (!token) {
            window.location.href = '/login';
            return;
        }

        const progressBar = document.querySelector('#uploadProgress .progress-bar');
        document.getElementById('uploadProgress').style.display = 'block';
        progressBar.style.width = '0%';

        const xhr = new XMLHttpRequest();

        xhr.upload.addEventListener('progress', (e) => {
            if (e.lengthComputable) {
                const percentComplete = (e.loaded / e.total) * 100;
                progressBar.style.width = percentComplete + '%';
            }
        });

        xhr.addEventListener('load', () => {
            if (xhr.status === 200 || xhr.status === 201) {
                showAlert('Media uploaded successfully', 'success');
                hideUploadForm();
                loadMedia();
            } else {
                const error = JSON.parse(xhr.responseText);
                showAlert(error.errors?.body?.[0] || 'Upload failed', 'danger');
            }
            document.getElementById('uploadProgress').style.display = 'none';
        });

        xhr.addEventListener('error', () => {
            showAlert('Upload failed', 'danger');
            document.getElementById('uploadProgress').style.display = 'none';
        });

        xhr.open('POST', '/api/media');
        xhr.setRequestHeader('Authorization', `Bearer ${token}`);
        xhr.send(formData);

    } catch (error) {
        console.error('Error uploading media:', error);
        showAlert('Failed to upload media', 'danger');
        document.getElementById('uploadProgress').style.display = 'none';
    }
}

async function viewMedia(mediaId) {
    try {
        const token = localStorage.getItem('authToken');
        if (!token) {
            window.location.href = '/login';
            return;
        }

        const response = await fetch(`/api/media/${mediaId}`, {
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });

        if (!response.ok) {
            throw new Error('Failed to load media details');
        }

        const data = await response.json();
        const media = data.media;

        currentMediaId = media.id;

        // Populate modal
        document.getElementById('modalMediaId').value = media.id;
        document.getElementById('modalImage').src = media.download_url;
        document.getElementById('modalImage').alt = media.alt_text || media.title || media.filename;
        document.getElementById('downloadLink').href = media.download_url;
        document.getElementById('modalFilename').textContent = media.filename;
        document.getElementById('modalFileSize').textContent = formatFileSize(media.file_size);
        document.getElementById('modalMimeType').textContent = media.mime_type;
        document.getElementById('modalUploadedAt').textContent = new Date(media.uploaded_at).toLocaleString();
        document.getElementById('modalTitle').value = media.title || '';
        document.getElementById('modalAltText').value = media.alt_text || '';
        document.getElementById('modalCaption').value = media.caption || '';
        document.getElementById('modalDescription').value = media.description || '';

        const modal = new bootstrap.Modal(document.getElementById('mediaModal'));
        modal.show();

    } catch (error) {
        console.error('Error loading media details:', error);
        showAlert('Failed to load media details', 'danger');
    }
}

async function handleSaveMedia() {
    if (!currentMediaId) return;

    try {
        const token = localStorage.getItem('authToken');
        if (!token) {
            window.location.href = '/login';
            return;
        }

        const payload = {
            title: document.getElementById('modalTitle').value || null,
            alt_text: document.getElementById('modalAltText').value || null,
            caption: document.getElementById('modalCaption').value || null,
            description: document.getElementById('modalDescription').value || null
        };

        const response = await fetch(`/api/media/${currentMediaId}`, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            },
            body: JSON.stringify(payload)
        });

        if (!response.ok) {
            throw new Error('Failed to update media');
        }

        showAlert('Media updated successfully', 'success');
        bootstrap.Modal.getInstance(document.getElementById('mediaModal')).hide();
        loadMedia();

    } catch (error) {
        console.error('Error updating media:', error);
        showAlert('Failed to update media', 'danger');
    }
}

function showDeleteConfirmation() {
    bootstrap.Modal.getInstance(document.getElementById('mediaModal')).hide();
    const deleteModal = new bootstrap.Modal(document.getElementById('deleteModal'));
    deleteModal.show();
}

async function handleDeleteMedia() {
    if (!currentMediaId) return;

    try {
        const token = localStorage.getItem('authToken');
        if (!token) {
            window.location.href = '/login';
            return;
        }

        const response = await fetch(`/api/media/${currentMediaId}`, {
            method: 'DELETE',
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });

        if (!response.ok) {
            throw new Error('Failed to delete media');
        }

        showAlert('Media deleted successfully', 'success');
        bootstrap.Modal.getInstance(document.getElementById('deleteModal')).hide();
        currentMediaId = null;
        loadMedia();

    } catch (error) {
        console.error('Error deleting media:', error);
        showAlert('Failed to delete media', 'danger');
    }
}

function copyMediaUrl() {
    const url = document.getElementById('downloadLink').href;
    navigator.clipboard.writeText(url).then(() => {
        showAlert('URL copied to clipboard', 'success');
    }).catch(() => {
        showAlert('Failed to copy URL', 'danger');
    });
}

function copyMediaMarkdown() {
    const url = document.getElementById('downloadLink').href;
    const altText = document.getElementById('modalAltText').value ||
                    document.getElementById('modalTitle').value ||
                    document.getElementById('modalFilename').textContent;

    const markdown = `![${altText}](${url})`;

    navigator.clipboard.writeText(markdown).then(() => {
        showAlert('Markdown copied to clipboard', 'success');
    }).catch(() => {
        showAlert('Failed to copy Markdown', 'danger');
    });
}

function formatFileSize(bytes) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
}

function showAlert(message, type) {
    const alertDiv = document.createElement('div');
    alertDiv.className = `alert alert-${type} alert-dismissible fade show position-fixed top-0 start-50 translate-middle-x mt-3`;
    alertDiv.style.zIndex = '9999';
    alertDiv.innerHTML = `
        ${escapeHtml(message)}
        <button type="button" class="btn-close" data-bs-dismiss="alert"></button>
    `;
    document.body.appendChild(alertDiv);

    setTimeout(() => {
        alertDiv.remove();
    }, 5000);
}

function escapeHtml(text) {
    if (!text) return '';
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}
