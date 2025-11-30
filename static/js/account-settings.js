// Account Settings Page JavaScript
(function() {
    'use strict';

    const authToken = getAuthToken();
    if (!authToken) {
        window.location.href = '/login';
        return;
    }

    // DOM Elements
    const profileForm = document.getElementById('profileForm');
    const passwordForm = document.getElementById('passwordForm');
    const successMessage = document.getElementById('successMessage');
    const errorMessage = document.getElementById('errorMessage');
    const successText = document.getElementById('successText');
    const errorText = document.getElementById('errorText');
    const passwordMismatch = document.getElementById('passwordMismatch');

    // Helper Functions
    function showSuccess(message) {
        successText.textContent = message;
        successMessage.classList.remove('d-none');
        errorMessage.classList.add('d-none');

        // Auto-hide after 5 seconds
        setTimeout(() => {
            successMessage.classList.add('d-none');
        }, 5000);
    }

    function showError(message) {
        errorText.textContent = message;
        errorMessage.classList.remove('d-none');
        successMessage.classList.add('d-none');

        // Auto-hide after 5 seconds
        setTimeout(() => {
            errorMessage.classList.add('d-none');
        }, 5000);
    }

    function hideMessages() {
        successMessage.classList.add('d-none');
        errorMessage.classList.add('d-none');
    }

    // Password Toggle Functions
    function setupPasswordToggle(toggleBtnId, inputId) {
        const toggleBtn = document.getElementById(toggleBtnId);
        const input = document.getElementById(inputId);

        if (toggleBtn && input) {
            toggleBtn.addEventListener('click', function() {
                const icon = this.querySelector('i');
                if (input.type === 'password') {
                    input.type = 'text';
                    icon.classList.remove('fa-eye');
                    icon.classList.add('fa-eye-slash');
                } else {
                    input.type = 'password';
                    icon.classList.remove('fa-eye-slash');
                    icon.classList.add('fa-eye');
                }
            });
        }
    }

    // Setup password toggles
    setupPasswordToggle('toggleCurrentPassword', 'currentPassword');
    setupPasswordToggle('toggleNewPassword', 'newPassword');
    setupPasswordToggle('toggleConfirmPassword', 'confirmPassword');

    // Password match validation
    const newPasswordInput = document.getElementById('newPassword');
    const confirmPasswordInput = document.getElementById('confirmPassword');

    function checkPasswordMatch() {
        const newPassword = newPasswordInput.value;
        const confirmPassword = confirmPasswordInput.value;

        if (confirmPassword && newPassword !== confirmPassword) {
            passwordMismatch.classList.remove('d-none');
        } else {
            passwordMismatch.classList.add('d-none');
        }
    }

    if (newPasswordInput && confirmPasswordInput) {
        newPasswordInput.addEventListener('input', checkPasswordMatch);
        confirmPasswordInput.addEventListener('input', checkPasswordMatch);
    }

    // Profile Form Submission
    if (profileForm) {
        profileForm.addEventListener('submit', async function(e) {
            e.preventDefault();
            hideMessages();

            const submitBtn = document.getElementById('profileSubmitBtn');
            const originalBtnText = submitBtn.innerHTML;
            submitBtn.disabled = true;
            submitBtn.innerHTML = '<i class="fas fa-spinner fa-spin me-2"></i>Updating...';

            const username = document.getElementById('username').value.trim();
            const email = document.getElementById('email').value.trim();
            const bio = document.getElementById('bio').value.trim();
            const image = document.getElementById('image').value.trim();

            const updateData = {
                user: {}
            };

            if (username) updateData.user.username = username;
            if (email) updateData.user.email = email;
            if (bio) updateData.user.bio = bio;
            if (image) updateData.user.image = image;

            try {
                const response = await fetch('/api/user', {
                    method: 'PUT',
                    headers: {
                        'Content-Type': 'application/json',
                        'Authorization': `Bearer ${authToken}`
                    },
                    body: JSON.stringify(updateData)
                });

                const data = await response.json();

                if (response.ok) {
                    showSuccess('Profile updated successfully!');

                    // Update stored token if it changed
                    if (data.user && data.user.token) {
                        localStorage.setItem('authToken', data.user.token);
                    }
                } else {
                    const errorMsg = data.errors?.body?.[0] || data.message || 'Failed to update profile. Please try again.';
                    showError(errorMsg);
                }
            } catch (error) {
                console.error('Error updating profile:', error);
                showError('Network error. Please check your connection and try again.');
            } finally {
                submitBtn.disabled = false;
                submitBtn.innerHTML = originalBtnText;
            }
        });
    }

    // Password Form Submission
    if (passwordForm) {
        passwordForm.addEventListener('submit', async function(e) {
            e.preventDefault();
            hideMessages();

            const newPassword = newPasswordInput.value;
            const confirmPassword = confirmPasswordInput.value;

            // Validate passwords match
            if (newPassword !== confirmPassword) {
                showError('Passwords do not match.');
                return;
            }

            const submitBtn = document.getElementById('passwordSubmitBtn');
            const originalBtnText = submitBtn.innerHTML;
            submitBtn.disabled = true;
            submitBtn.innerHTML = '<i class="fas fa-spinner fa-spin me-2"></i>Changing...';

            const currentPassword = document.getElementById('currentPassword').value;

            try {
                const response = await fetch('/api/account/password', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                        'Authorization': `Bearer ${authToken}`
                    },
                    body: JSON.stringify({
                        current_password: currentPassword,
                        new_password: newPassword
                    })
                });

                const data = await response.json();

                if (response.ok) {
                    showSuccess('Password changed successfully! You will be redirected to login...');
                    passwordForm.reset();
                    passwordMismatch.classList.add('d-none');

                    // Clear auth token since it's now invalid
                    localStorage.removeItem('authToken');
                    document.cookie = 'authToken=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;';

                    // Redirect to login after 2 seconds
                    setTimeout(() => {
                        window.location.href = '/login';
                    }, 2000);
                } else {
                    const errorMsg = data.errors?.body?.[0] || data.message || 'Failed to change password. Please try again.';
                    showError(errorMsg);
                }
            } catch (error) {
                console.error('Error changing password:', error);
                showError('Network error. Please check your connection and try again.');
            } finally {
                submitBtn.disabled = false;
                submitBtn.innerHTML = originalBtnText;
            }
        });
    }

    // Scroll to top on page load
    window.scrollTo(0, 0);
})();
