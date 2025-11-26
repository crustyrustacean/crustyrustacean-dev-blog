// auth.js - Authentication form handling

/**
 * Authentication form manager
 */
class AuthManager {
    constructor() {
        this.init();
    }

    init() {
        this.setupLoginForm();
        this.setupRegisterForm();
        this.setupPasswordToggle();
        this.checkForRegistrationMessage();
    }

    /**
     * Check for registration success message in URL params
     */
    checkForRegistrationMessage() {
        const urlParams = new URLSearchParams(window.location.search);
        const registered = urlParams.get('registered');
        const email = urlParams.get('email');

        if (registered === 'true') {
            const message = email
                ? `Registration successful! Please check your email (${decodeURIComponent(email)}) for the verification link before logging in.`
                : 'Registration successful! Please check your email for the verification link before logging in.';
            this.showVerificationInfo(message);

            // Clean up URL without reloading
            const cleanUrl = window.location.pathname;
            window.history.replaceState({}, document.title, cleanUrl);
        }
    }

    /**
     * Show verification info message (persistent, doesn't auto-dismiss)
     */
    showVerificationInfo(message) {
        const alert = document.createElement('div');
        alert.className = 'alert alert-info';
        alert.innerHTML = `<i class="fas fa-envelope me-2"></i>${message}`;

        const form = document.querySelector('form');
        if (form) {
            form.parentNode.insertBefore(alert, form);
        }
    }

    /**
     * Setup login form handling
     */
    setupLoginForm() {
        const loginForm = document.getElementById('loginForm');
        if (!loginForm) return;

        loginForm.addEventListener('submit', async (e) => {
            e.preventDefault();

            const formData = new FormData(loginForm);
            const loginData = {
                user: {
                    email: formData.get('email'),
                    password: formData.get('password')
                }
            };

            try {
                const response = await fetch('/api/users/login', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify(loginData)
                });

                if (response.ok) {
                    const data = await response.json();
                    this.storeAuthToken(data.user.token);
                    window.location.href = '/admin';
                } else {
                    const errorData = await response.json();
                    this.showError(errorData.message || 'Login failed. Please try again.');
                }
            } catch (error) {
                this.showError('Network error. Please check your connection and try again.');
            }
        });
    }

    /**
     * Setup registration form handling
     */
    setupRegisterForm() {
        const registerForm = document.getElementById('registerForm');
        if (!registerForm) return;

        registerForm.addEventListener('submit', async (e) => {
            e.preventDefault();

            const formData = new FormData(registerForm);
            const password = formData.get('password');
            const confirmPassword = formData.get('confirmPassword');

            // Client-side validation
            if (password !== confirmPassword) {
                this.showError('Passwords do not match.');
                return;
            }

            const registerData = {
                user: {
                    username: formData.get('username'),
                    email: formData.get('email'),
                    password: password
                }
            };

            // Disable submit button during request
            const submitBtn = registerForm.querySelector('button[type="submit"]');
            if (submitBtn) {
                submitBtn.disabled = true;
                submitBtn.innerHTML = '<i class="fas fa-spinner fa-spin me-2"></i>Creating Account...';
            }

            try {
                const response = await fetch('/api/users', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify(registerData)
                });

                if (response.ok) {
                    const data = await response.json();
                    // Registration successful - redirect to login with verification message
                    const email = encodeURIComponent(data.email);
                    window.location.href = `/login?registered=true&email=${email}`;
                } else {
                    const errorData = await response.json();
                    const errorMessage = this.parseRegistrationErrors(errorData);
                    this.showError(errorMessage);
                    // Re-enable submit button
                    if (submitBtn) {
                        submitBtn.disabled = false;
                        submitBtn.innerHTML = '<i class="fas fa-user-plus me-2"></i>Create Account';
                    }
                }
            } catch (error) {
                this.showError('Network error. Please check your connection and try again.');
                // Re-enable submit button
                if (submitBtn) {
                    submitBtn.disabled = false;
                    submitBtn.innerHTML = '<i class="fas fa-user-plus me-2"></i>Create Account';
                }
            }
        });
    }

    /**
     * Setup password visibility toggle
     */
    setupPasswordToggle() {
        const toggleButtons = document.querySelectorAll('[id^="togglePassword"]');
        
        toggleButtons.forEach(toggleButton => {
            const targetId = toggleButton.getAttribute('data-target') || 
                            (toggleButton.id === 'togglePassword' ? 'password' : 
                             toggleButton.id === 'toggleConfirmPassword' ? 'confirmPassword' : null);
            
            if (!targetId) return;
            
            const passwordInput = document.getElementById(targetId);
            if (!passwordInput) return;

            toggleButton.addEventListener('click', () => {
                const type = passwordInput.getAttribute('type') === 'password' ? 'text' : 'password';
                passwordInput.setAttribute('type', type);

                const icon = toggleButton.querySelector('i');
                if (icon) {
                    icon.classList.toggle('fa-eye');
                    icon.classList.toggle('fa-eye-slash');
                }
            });
        });
    }

    /**
     * Store authentication token
     */
    storeAuthToken(token) {
        if (!token) return;

        // Store in localStorage
        localStorage.setItem('authToken', token);

        // Store in cookie with 24 hour expiration
        const expires = new Date();
        expires.setTime(expires.getTime() + (24 * 60 * 60 * 1000)); // 24 hours
        document.cookie = `authToken=${encodeURIComponent(token)}; expires=${expires.toUTCString()}; path=/; SameSite=Strict`;
    }

    /**
     * Parse registration error messages
     */
    parseRegistrationErrors(errorData) {
        if (errorData.errors) {
            const errors = [];
            if (errorData.errors.username) errors.push(`Username: ${errorData.errors.username.join(', ')}`);
            if (errorData.errors.email) errors.push(`Email: ${errorData.errors.email.join(', ')}`);
            if (errorData.errors.password) errors.push(`Password: ${errorData.errors.password.join(', ')}`);
            return errors.length > 0 ? errors.join(' | ') : 'Registration failed.';
        }
        return errorData.message || 'Registration failed. Please try again.';
    }

    /**
     * Show error message
     */
    showError(message) {
        // Remove existing alerts
        const existingAlert = document.querySelector('.alert-danger');
        if (existingAlert && !existingAlert.hasAttribute('data-server-error')) {
            existingAlert.remove();
        }

        // Create new alert
        const alert = document.createElement('div');
        alert.className = 'alert alert-danger';
        alert.innerHTML = `<i class="fas fa-exclamation-triangle me-2"></i>${message}`;

        // Find insertion point (before first form)
        const form = document.querySelector('form');
        if (form) {
            form.parentNode.insertBefore(alert, form);
        }

        // Auto-dismiss after 5 seconds
        setTimeout(() => {
            if (alert.parentNode) {
                alert.remove();
            }
        }, 5000);
    }

    /**
     * Show success message
     */
    showSuccess(message) {
        const alert = document.createElement('div');
        alert.className = 'alert alert-success';
        alert.innerHTML = `<i class="fas fa-check-circle me-2"></i>${message}`;

        const form = document.querySelector('form');
        if (form) {
            form.parentNode.insertBefore(alert, form);
        }

        setTimeout(() => {
            if (alert.parentNode) {
                alert.remove();
            }
        }, 5000);
    }
}

// Initialize when DOM is loaded
document.addEventListener('DOMContentLoaded', function() {
    new AuthManager();
});

// Export for manual initialization if needed
window.AuthManager = AuthManager;