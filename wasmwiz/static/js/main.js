/**
 * WasmWiz Main JavaScript - Production Version
 * @version 1.0.0
 * @author WasmWiz Team
 * @description Main frontend functionality for the WasmWiz platform
 */

// Browser compatibility check
function checkBrowserCompatibility() {
    const isWebAssemblySupported = (typeof WebAssembly === 'object');
    const isFileReaderSupported = (typeof FileReader === 'function');
    const isFetchSupported = (typeof fetch === 'function');
    
    if (!isWebAssemblySupported || !isFileReaderSupported || !isFetchSupported) {
        displayAlert('Your browser does not support all features required for WasmWiz. Please upgrade to a modern browser.', 'error');
        return false;
    }
    
    return true;
}

// Get CSRF token from form
function getCsrfToken() {
    const tokenInput = document.querySelector('input[name="csrf_token"]');
    return tokenInput ? tokenInput.value : '';
}

// Performance monitoring
const performanceMetrics = {
    pageLoadTime: 0,
    uploadTime: 0,
    executionTime: 0,
    startTiming: function(metric) {
        this[`${metric}Start`] = performance.now();
    },
    endTiming: function(metric) {
        this[metric] = performance.now() - this[`${metric}Start`];
        console.debug(`Performance: ${metric} = ${this[metric].toFixed(2)}ms`);
    },
    reset: function() {
        this.pageLoadTime = 0;
        this.uploadTime = 0;
        this.executionTime = 0;
    }
};

// Initialize performance tracking
performanceMetrics.startTiming('pageLoadTime');

// Error tracking and reporting
function reportError(error, context) {
    console.error(`Error in ${context}:`, error);
    
    // In production, you would send this to your error monitoring service
    // Example: Sentry.captureException(error);
    
    // For now, we'll just log it
    const errorData = {
        message: error.message,
        stack: error.stack,
        context: context,
        url: window.location.href,
        timestamp: new Date().toISOString(),
        userAgent: navigator.userAgent
    };
    
    // In development, log the full error details
    console.debug('Error details:', errorData);
    
    // Show a user-friendly message
    displayAlert(`An error occurred in ${context}. Please try again.`, 'error');
}

// Initialize offline detection
function setupOfflineDetection() {
    window.addEventListener('online', () => {
        displayAlert('You are back online. All features are available.', 'success');
    });
    
    window.addEventListener('offline', () => {
        displayAlert('You are offline. Some features may not work correctly.', 'warning');
    });
    
    // Check initial state
    if (!navigator.onLine) {
        displayAlert('You are currently offline. Some features may not work correctly.', 'warning');
    }
}

// Application initialization
document.addEventListener('DOMContentLoaded', function() {
    // Check browser compatibility
    if (!checkBrowserCompatibility()) {
        return; // Stop initialization if browser is not compatible
    }
    
    // Set up offline detection
    setupOfflineDetection();
    
    // File upload handling
    const fileUpload = document.querySelector('.file-upload');
    const fileInput = document.querySelector('#wasm-file');
    
    if (fileUpload && fileInput) {
        // Drag and drop functionality
        fileUpload.addEventListener('dragover', function(e) {
            e.preventDefault();
            fileUpload.classList.add('dragover');
        });
        
        fileUpload.addEventListener('dragleave', function(e) {
            e.preventDefault();
            fileUpload.classList.remove('dragover');
        });
        
        fileUpload.addEventListener('drop', function(e) {
            e.preventDefault();
            fileUpload.classList.remove('dragover');
            
            const files = e.dataTransfer.files;
            if (files.length > 0) {
                fileInput.files = files;
                updateFileDisplay(files[0]);
                validateFileAndShowFeedback(files[0]);
            }
        });
        
        // File input change
        fileInput.addEventListener('change', function(e) {
            if (e.target.files.length > 0) {
                updateFileDisplay(e.target.files[0]);
                validateFileAndShowFeedback(e.target.files[0]);
            }
        });
        
        // Click to select file
        fileUpload.addEventListener('click', function() {
            fileInput.click();
        });
        
        // Remove file button
        document.getElementById('remove-file')?.addEventListener('click', function(e) {
            e.preventDefault();
            e.stopPropagation();
            fileInput.value = '';
            document.getElementById('file-info').style.display = 'none';
            document.getElementById('file-name').textContent = '';
            document.getElementById('file-size').textContent = '';
            fileUpload.classList.remove('has-file');
        });
    }
    
    // Real-time input validation
    const inputText = document.querySelector('#input-text');
    if (inputText) {
        inputText.addEventListener('input', debounce(function(e) {
            validateInputAndShowFeedback(e.target.value);
        }, 300));
    }
    
    // Input type selector
    const inputTypeRadios = document.querySelectorAll('input[name="input-type"]');
    inputTypeRadios.forEach(radio => {
        radio.addEventListener('change', function() {
            updateInputPlaceholder(this.value);
            validateInputAndShowFeedback(inputText.value);
        });
    });
    
    // API key input validation
    const apiKeyInput = document.querySelector('#api-key');
    if (apiKeyInput) {
        apiKeyInput.addEventListener('input', debounce(function(e) {
            validateApiKeyAndShowFeedback(e.target.value);
        }, 300));
        
        // Load saved API key
        const savedApiKey = localStorage.getItem('wasmwiz-api-key');
        if (savedApiKey) {
            apiKeyInput.value = savedApiKey;
            validateApiKeyAndShowFeedback(savedApiKey);
        }
        
        // Save API key when changed
        apiKeyInput.addEventListener('change', function() {
            localStorage.setItem('wasmwiz-api-key', this.value);
        });
    }
    
    // Range sliders
    initRangeSliders();
    
    // Form validation
    const executeForm = document.querySelector('#execute-form');
    if (executeForm) {
        executeForm.addEventListener('submit', function(e) {
            e.preventDefault();
            if (validateForm()) {
                executeWasm();
            }
        });
    }
    
    // Sample gallery handling
    const sampleButtons = document.querySelectorAll('.use-sample');
    sampleButtons.forEach(button => {
        button.addEventListener('click', function() {
            const sampleCard = this.closest('.sample-card');
            const sampleName = sampleCard.getAttribute('data-sample');
            loadSampleModule(sampleName);
        });
    });
    
    // Language selector
    const languageSelector = document.getElementById('language-selector');
    if (languageSelector) {
        languageSelector.addEventListener('change', function() {
            const language = this.value;
            localStorage.setItem('wasmwiz-language', language);
            showToast(`Language set to ${language}`, 'info');
        });
        
        // Load saved language
        const savedLanguage = localStorage.getItem('wasmwiz-language');
        if (savedLanguage) {
            languageSelector.value = savedLanguage;
        }
    }
    
    // Sign-in link
    const signInLink = document.getElementById('sign-in-link');
    if (signInLink) {
        signInLink.addEventListener('click', function(e) {
            e.preventDefault();
            showToast('Sign-in feature will be available in the next update', 'info');
        });
    }
    
    // API Key Management
    initializeApiKeyManagement();
    
    // End page load timing
    performanceMetrics.endTiming('pageLoadTime');
    
    // Initial accessibility check
    runAccessibilityCheck();
});

// Debounce function to limit function calls
function debounce(func, wait) {
    let timeout;
    return function(...args) {
        const context = this;
        clearTimeout(timeout);
        timeout = setTimeout(() => func.apply(context, args), wait);
    };
}

// Throttle function for events that fire rapidly
function throttle(func, limit) {
    let inThrottle;
    return function(...args) {
        const context = this;
        if (!inThrottle) {
            func.apply(context, args);
            inThrottle = true;
            setTimeout(() => inThrottle = false, limit);
        }
    };
}

// Basic accessibility check
function runAccessibilityCheck() {
    // Check for contrast issues and missing alt text
    const potentialIssues = [];
    
    // Check images for alt text
    document.querySelectorAll('img').forEach(img => {
        if (!img.hasAttribute('alt')) {
            potentialIssues.push(`Image missing alt text: ${img.src}`);
        }
    });
    
    // Check form fields for labels
    document.querySelectorAll('input, textarea, select').forEach(field => {
        if (!field.id) return;
        
        const hasLabel = document.querySelector(`label[for="${field.id}"]`);
        const hasAriaLabel = field.getAttribute('aria-label');
        
        if (!hasLabel && !hasAriaLabel) {
            potentialIssues.push(`Form field missing label: ${field.id}`);
        }
    });
    
    // Log issues in development
    if (potentialIssues.length > 0) {
        console.warn('Accessibility issues detected:', potentialIssues);
    }
}

function updateFileDisplay(file) {
    const fileInfo = document.getElementById('file-info');
    const fileNameElement = document.getElementById('file-name');
    const fileSizeElement = document.getElementById('file-size');
    const dropArea = document.getElementById('drop-area');
    
    if (fileInfo && fileNameElement && fileSizeElement) {
        fileNameElement.textContent = file.name;
        fileSizeElement.textContent = formatFileSize(file.size);
        fileInfo.style.display = 'block';
        dropArea.classList.add('has-file');
    }
    
    // Validate file
    validateFile(file);
}

function validateFile(file) {
    const errors = [];
    const maxSize = 10 * 1024 * 1024; // 10MB
    
    if (!file.name.endsWith('.wasm')) {
        errors.push('File must have a .wasm extension');
    }
    
    if (file.size > maxSize) {
        errors.push('File size must be less than 10MB');
    }
    
    // Check WASM header bytes if possible
    if (window.FileReader && file.size > 4) {
        const reader = new FileReader();
        reader.onload = function(e) {
            const arrayBuffer = e.target.result;
            const headerBytes = new Uint8Array(arrayBuffer, 0, 4);
            
            // Check for WASM magic bytes: 0x00, 0x61, 0x73, 0x6D
            const isValidWasm = (
                headerBytes[0] === 0x00 &&
                headerBytes[1] === 0x61 &&
                headerBytes[2] === 0x73 &&
                headerBytes[3] === 0x6D
            );
            
            if (!isValidWasm) {
                displayAlert('The selected file does not appear to be a valid WebAssembly module.', 'error');
            }
        };
        reader.readAsArrayBuffer(file.slice(0, 4));
    }
    
    displayValidationErrors(errors);
    return errors.length === 0;
}

function validateFileAndShowFeedback(file) {
    if (!validateFile(file)) {
        displayAlert('Please fix the validation errors before continuing', 'warning');
        return false;
    }
    return true;
}

function validateInput(input) {
    const errors = [];
    const maxSize = 1024 * 1024; // 1MB
    const inputType = document.querySelector('input[name="input-type"]:checked')?.value || 'text';
    
    if (new Blob([input]).size > maxSize) {
        errors.push('Input size must be less than 1MB');
    }
    
    // Validate based on input type
    if (inputType === 'json' && input.trim() !== '') {
        try {
            JSON.parse(input);
        } catch (e) {
            errors.push('Invalid JSON format: ' + e.message);
        }
    } else if (inputType === 'binary') {
        // Basic validation for Base64
        const base64Regex = /^[A-Za-z0-9+/=]*$/;
        if (input.trim() !== '' && !base64Regex.test(input)) {
            errors.push('Invalid Base64 format');
        }
    }
    
    displayValidationErrors(errors);
    return errors.length === 0;
}

function validateInputAndShowFeedback(input) {
    return validateInput(input);
}

function validateApiKey(apiKey) {
    // Simple validation for API key format
    if (apiKey && apiKey.trim() !== '') {
        const apiKeyRegex = /^[A-Za-z0-9_-]{20,64}$/;
        if (!apiKeyRegex.test(apiKey)) {
            return false;
        }
    }
    return true;
}

function validateApiKeyAndShowFeedback(apiKey) {
    if (apiKey && apiKey.trim() !== '' && !validateApiKey(apiKey)) {
        displayAlert('Invalid API key format', 'warning');
        return false;
    }
    return true;
}

function displayValidationErrors(errors) {
    const errorContainer = document.querySelector('#validation-errors');
    if (errorContainer) {
        if (errors.length > 0) {
            errorContainer.innerHTML = `
                <div class="alert alert-error">
                    <ul>
                        ${errors.map(error => `<li>${error}</li>`).join('')}
                    </ul>
                </div>
            `;
        } else {
            errorContainer.innerHTML = '';
        }
    }
}

function validateForm() {
    const fileInput = document.querySelector('#wasm-file');
    const inputText = document.querySelector('#input-text');
    const apiKeyInput = document.querySelector('#api-key');
    
    console.log('[DEBUG] validateForm called');
    console.log('[DEBUG] fileInput:', fileInput);
    console.log('[DEBUG] fileInput.files:', fileInput.files);
    console.log('[DEBUG] fileInput.files?.length:', fileInput.files?.length);
    console.log('[DEBUG] currentSampleFile:', currentSampleFile);
    
    // Check if we have a file from either file input or sample loading
    let selectedFile = null;
    if (fileInput.files && fileInput.files.length > 0) {
        selectedFile = fileInput.files[0];
        console.log('[DEBUG] Using file from input:', selectedFile);
    } else if (currentSampleFile) {
        selectedFile = currentSampleFile;
        console.log('[DEBUG] Using current sample file:', selectedFile);
    }
    
    // Validate file is selected
    if (!selectedFile) {
        console.log('[DEBUG] No files selected, showing error');
        displayAlert('Please select a WebAssembly (.wasm) file', 'error');
        return false;
    }
    
    console.log('[DEBUG] File selected:', selectedFile);
    
    // Validate file
    if (!validateFile(selectedFile)) {
        console.log('[DEBUG] File validation failed');
        return false;
    }
    
    // Validate input
    if (!validateInput(inputText.value)) {
        return false;
    }
    
    // Validate API key if provided
    if (apiKeyInput.value.trim() !== '' && !validateApiKey(apiKeyInput.value)) {
        displayAlert('Invalid API key format', 'warning');
        return false;
    }
    
    return true;
}

async function executeWasm() {
    performanceMetrics.startTiming('executionTime');
    
    const form = document.querySelector('#execute-form');
    const fileInput = document.querySelector('#wasm-file');
    const inputText = document.querySelector('#input-text');
    const inputType = document.querySelector('input[name="input-type"]:checked')?.value || 'text';
    const memoryLimit = document.querySelector('#memory-limit')?.value || '128';
    const timeout = document.querySelector('#timeout')?.value || '5';
    const resultContainer = document.querySelector('#execution-result');
    const submitButton = document.querySelector('#submit-button');
    const progressContainer = document.querySelector('#progress-container');
    
    // Validate form first
    if (!validateForm()) {
        performanceMetrics.endTiming('executionTime');
        return;
    }
    
    // Get the selected file (either from input or sample)
    let selectedFile = null;
    if (fileInput.files && fileInput.files.length > 0) {
        selectedFile = fileInput.files[0];
    } else if (currentSampleFile) {
        selectedFile = currentSampleFile;
    }
    
    console.log('[DEBUG] executeWasm - selectedFile:', selectedFile);
    
    // Update progress steps
    if (progressContainer) {
        progressContainer.style.display = 'flex';
        updateProgressStep('step-upload', 'active');
    }
    
    // Show enhanced loading state
    setLoadingState(submitButton, true);
    
    try {
        // Check if online
        if (!navigator.onLine) {
            throw new Error('You are currently offline. Please check your internet connection and try again.');
        }
        
        const formData = new FormData();
        formData.append('wasm', selectedFile);
        formData.append('input', inputText.value);
        formData.append('input_type', inputType);
        formData.append('memory_limit', memoryLimit);
        formData.append('timeout', timeout);
        
        // Add session and analytics data
        formData.append('session_id', getSessionId());
        formData.append('client_timestamp', new Date().toISOString());
        
        // In development mode, API key is optional
        const apiKey = getApiKey();
        const headers = {
            'X-CSRF-Token': getCsrfToken()
        };
        if (apiKey && apiKey.trim() !== '') {
            headers['Authorization'] = `Bearer ${apiKey}`;
        }
        
        performanceMetrics.startTiming('uploadTime');
        updateProgressStep('step-upload', 'completed');
        updateProgressStep('step-execute', 'active');
        
        // Set up request timeout
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 30000); // 30-second timeout
        
        const response = await fetch('/api/execute', {
            method: 'POST',
            body: formData,
            headers: headers,
            signal: controller.signal
        });
        
        clearTimeout(timeoutId);
        performanceMetrics.endTiming('uploadTime');
        
        updateProgressStep('step-execute', 'completed');
        updateProgressStep('step-results', 'active');
        
        if (!response.ok) {
            // Try to parse error response
            let errorMessage;
            try {
                const errorData = await response.json();
                errorMessage = errorData.error || `Server error: ${response.status} ${response.statusText}`;
            } catch (e) {
                errorMessage = `Server error: ${response.status} ${response.statusText}`;
            }
            throw new Error(errorMessage);
        }
        
        const result = await response.json();
        
        updateProgressStep('step-results', 'completed');
        
        if (response.ok) {
            console.log('[DEBUG] About to call displayExecutionResult with:', result, response.status);
            console.log('Execution completed successfully, displaying results:', result);
            displayExecutionResult(result, response.status);
            showToast('Execution completed successfully', 'success');
            
            // Log successful execution
            console.debug('Execution successful:', {
                fileSize: selectedFile.size,
                inputSize: new Blob([inputText.value]).size,
                executionTime: result.execution_time_ms || 0,
                memoryUsage: result.memory_usage_mb || 0
            });
        } else {
            console.log('[DEBUG] Response not OK, showing error alert');
            displayAlert(`Execution failed: ${result.error || 'Unknown error'}`, 'error');
        }
    } catch (error) {
        updateProgressStep('step-execute', 'error');
        updateProgressStep('step-results', 'error');
        
        // Handle specific error types
        if (error.name === 'AbortError') {
            displayAlert('The request timed out. The server may be experiencing high load.', 'error');
        } else if (error.message.includes('Failed to fetch')) {
            displayAlert('Network error: Failed to connect to the server. Please check your internet connection.', 'error');
        } else {
            displayAlert(`Error: ${error.message}`, 'error');
        }
        
        // Report error
        reportError(error, 'WASM Execution');
    } finally {
        setLoadingState(submitButton, false);
        setTimeout(() => {
            if (progressContainer) progressContainer.style.display = 'none';
        }, 3000);
        
        performanceMetrics.endTiming('executionTime');
    }
}

// --- DEBUG: Test function to manually trigger result display ---
function testResultDisplay() {
    console.log('[DEBUG] testResultDisplay called');
    const mockResult = {
        output: "Test output",
        execution_time_ms: 100,
        memory_usage_mb: 1.5
    };
    displayExecutionResult(mockResult, 200);
    console.log('[DEBUG] testResultDisplay finished');
}

// Enhanced execution result display
function displayExecutionResult(result, statusCode) {
    const resultContainer = document.querySelector('#execution-result');
    console.log('[DEBUG] displayExecutionResult called', { result, statusCode, resultContainer });
    if (!resultContainer) {
        console.error('[DEBUG] #execution-result not found in DOM');
        return;
    }
    
    const hasOutput = result.output && result.output.trim().length > 0;
    const hasError = result.error && result.error.trim().length > 0;
    const executionTime = result.execution_time_ms || 0;
    const memoryUsage = result.memory_usage_mb || 0;
    
    let statusBadge = '';
    if (statusCode === 200) {
        statusBadge = '<span class="status-badge success">✅ Success</span>';
    } else if (statusCode >= 400) {
        statusBadge = '<span class="status-badge error">❌ Error</span>';
    }
    
    const resultHtml = `
        <div class="execution-result-card">
            <div class="result-header">
                <h3>Execution Result</h3>
                ${statusBadge}
            </div>
            
            ${hasOutput ? `
                <div class="result-section">
                    <h4>Program Output</h4>
                    <div class="code-output">${escapeHtml(result.output)}</div>
                    <button onclick="copyToClipboard('${escapeHtml(result.output).replace(/'/g, "\\'")}', 'output')" 
                            class="btn btn-secondary btn-sm">
                        📋 Copy Output
                    </button>
                </div>
            ` : ''}
            
            ${hasError ? `
                <div class="result-section error-section">
                    <h4>Error Details</h4>
                    <div class="error-output">${escapeHtml(result.error)}</div>
                </div>
            ` : ''}
            
            ${!hasOutput && !hasError ? `
                <div class="result-section">
                    <p class="no-output">The program executed but produced no output.</p>
                </div>
            ` : ''}
            
            <div class="result-metadata">
                <div class="metadata-item">
                    <span class="metadata-label">Execution Time:</span>
                    <span class="metadata-value">${executionTime} ms</span>
                </div>
                <div class="metadata-item">
                    <span class="metadata-label">Memory Usage:</span>
                    <span class="metadata-value">${memoryUsage} MB</span>
                </div>
                <div class="metadata-item">
                    <span class="metadata-label">Frontend Processing:</span>
                    <span class="metadata-value">${performanceMetrics.executionTime.toFixed(2)} ms</span>
                </div>
            </div>
            
            <div class="result-actions">
                <button onclick="clearResults()" class="btn btn-secondary">Clear Results</button>
                <button onclick="downloadResults()" class="btn btn-primary">💾 Download Results</button>
            </div>
        </div>
    `;
    
    resultContainer.innerHTML = resultHtml;
    console.log('[DEBUG] resultHtml injected into #execution-result', resultHtml);
    resultContainer.scrollIntoView({ behavior: 'smooth' });
    // Check if buttons exist after injection
    const clearBtn = resultContainer.querySelector('.btn.btn-secondary');
    const downloadBtn = resultContainer.querySelector('.btn.btn-primary');
    console.log('[DEBUG] Buttons after injection', { clearBtn, downloadBtn });
}

function clearResults() {
    console.log('[DEBUG] clearResults called');
    const resultContainer = document.querySelector('#execution-result');
    if (resultContainer) {
        resultContainer.innerHTML = '';
        console.log('[DEBUG] #execution-result cleared');
    }
}

function downloadResults() {
    console.log('[DEBUG] downloadResults called');
    const resultContainer = document.querySelector('#execution-result');
    if (!resultContainer) {
        console.error('[DEBUG] Result container not found!');
        return;
    }
    
    const outputElement = resultContainer.querySelector('.code-output');
    const errorElement = resultContainer.querySelector('.error-output');
    
    let content = '# WasmWiz Execution Results\n\n';
    content += `Date: ${new Date().toLocaleString()}\n\n`;
    
    if (outputElement) {
        content += '## Program Output\n\n```\n' + outputElement.textContent + '\n```\n\n';
    }
    
    if (errorElement) {
        content += '## Error Details\n\n```\n' + errorElement.textContent + '\n```\n\n';
    }
    
    // Add metadata
    const metadataItems = resultContainer.querySelectorAll('.metadata-item');
    if (metadataItems.length > 0) {
        content += '## Execution Metadata\n\n';
        metadataItems.forEach(item => {
            const label = item.querySelector('.metadata-label').textContent;
            const value = item.querySelector('.metadata-value').textContent;
            content += `${label} ${value}\n`;
        });
    }
    
    // Add system information
    content += '\n## System Information\n\n';
    content += `Browser: ${navigator.userAgent}\n`;
    content += `Platform: ${navigator.platform}\n`;
    content += `Language: ${navigator.language}\n`;
    content += `Viewport: ${window.innerWidth}x${window.innerHeight}\n`;
    content += `Date: ${new Date().toISOString()}\n`;
    
    if (content) {
        try {
            const blob = new Blob([content], { type: 'text/markdown' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `wasmwiz-result-${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.md`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
        } catch (error) {
            reportError(error, 'Download Results');
            displayAlert('Failed to download results. Please try again.', 'error');
        }
    }
}

function copyToClipboard(text, type) {
    try {
        navigator.clipboard.writeText(text).then(function() {
            showToast(`${type.charAt(0).toUpperCase() + type.slice(1)} copied to clipboard!`, 'success');
        }, function(err) {
            throw err;
        });
    } catch (error) {
        // Fallback for browsers that don't support clipboard API
        const textarea = document.createElement('textarea');
        textarea.value = text;
        textarea.style.position = 'fixed';  // Prevent scrolling to bottom
        document.body.appendChild(textarea);
        textarea.focus();
        textarea.select();
        
        try {
            const successful = document.execCommand('copy');
            if (successful) {
                showToast(`${type.charAt(0).toUpperCase() + type.slice(1)} copied to clipboard!`, 'success');
            } else {
                throw new Error('Copy command failed');
            }
        } catch (err) {
            reportError(err, 'Copy to Clipboard');
            showToast('Failed to copy to clipboard', 'error');
        }
        
        document.body.removeChild(textarea);
    }
}

function displayAlert(message, type = 'info') {
    const notificationArea = document.getElementById('notification-area');
    if (!notificationArea) return;
    
    const alertId = 'alert-' + Date.now();
    const alertHtml = `
        <div id="${alertId}" class="alert alert-${type} alert-animated" role="alert" aria-live="assertive">
            <div class="alert-content">
                <span class="alert-icon" aria-hidden="true">${getAlertIcon(type)}</span>
                <span class="alert-message">${message}</span>
            </div>
            <button class="alert-close" onclick="dismissAlert('${alertId}')" aria-label="Dismiss">×</button>
        </div>
    `;
    
    notificationArea.insertAdjacentHTML('beforeend', alertHtml);
    
    // Auto-dismiss after 5 seconds
    setTimeout(() => {
        dismissAlert(alertId);
    }, 5000);
}

function dismissAlert(alertId) {
    const alert = document.getElementById(alertId);
    if (alert) {
        alert.classList.add('alert-dismissing');
        setTimeout(() => {
            alert.remove();
        }, 300);
    }
}

function getAlertIcon(type) {
    switch (type) {
        case 'success': return '✅';
        case 'error': return '❌';
        case 'warning': return '⚠️';
        case 'info': default: return 'ℹ️';
    }
}

function showToast(message, type = 'info') {
    const toast = document.createElement('div');
    toast.className = `toast toast-${type}`;
    toast.setAttribute('role', 'alert');
    toast.setAttribute('aria-live', 'polite');
    toast.textContent = message;
    
    document.body.appendChild(toast);
    
    // Trigger animation
    setTimeout(() => toast.classList.add('show'), 100);
    
    // Remove after 3 seconds
    setTimeout(() => {
        toast.classList.remove('show');
        setTimeout(() => document.body.removeChild(toast), 300);
    }, 3000);
}

function getApiKey() {
    // For development mode, return empty if not set (no-auth mode)
    // In production, this would be handled by authentication
    return localStorage.getItem('wasmwiz-api-key') || '';
}

function formatFileSize(bytes) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function escapeHtml(text) {
    if (!text) return '';
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function updateInputPlaceholder(inputType) {
    const inputTextarea = document.querySelector('#input-text');
    if (!inputTextarea) return;
    
    switch (inputType) {
        case 'text':
            inputTextarea.placeholder = 'Enter plain text input for your WebAssembly module...';
            break;
        case 'json':
            inputTextarea.placeholder = 'Enter JSON input for your WebAssembly module...\nExample: {"name": "value"}';
            break;
        case 'binary':
            inputTextarea.placeholder = 'Enter Base64-encoded binary input for your WebAssembly module...';
            break;
    }
}

function initRangeSliders() {
    const memorySlider = document.querySelector('#memory-limit');
    const memoryDisplay = document.querySelector('#memory-display');
    const timeoutSlider = document.querySelector('#timeout');
    const timeoutDisplay = document.querySelector('#timeout-display');
    
    if (memorySlider && memoryDisplay) {
        memorySlider.addEventListener('input', function() {
            memoryDisplay.textContent = `${this.value} MB`;
        });
    }
    
    if (timeoutSlider && timeoutDisplay) {
        timeoutSlider.addEventListener('input', function() {
            timeoutDisplay.textContent = `${this.value} sec`;
        });
    }
}

// Global variable to store current sample file
let currentSampleFile = null;

async function loadSampleModule(sampleName) {
    // Show loading state
    displayAlert(`Loading sample module: ${sampleName}...`, 'info');
    
    try {
        performanceMetrics.startTiming('loadSampleTime');
        
        // Fetch the sample WASM file
        const response = await fetch(`/static/wasm_modules/${sampleName}.wasm`);
        if (!response.ok) {
            throw new Error(`Failed to load sample module: ${response.statusText}`);
        }
        
        const blob = await response.blob();
        
        // Create a File object from the blob
        const file = new File([blob], `${sampleName}.wasm`, { type: 'application/wasm' });
        
        console.log('[DEBUG] loadSampleModule - created file:', file);
        console.log('[DEBUG] loadSampleModule - file.name:', file.name);
        console.log('[DEBUG] loadSampleModule - file.size:', file.size);
        
        // Store the file globally so we can access it in validation
        currentSampleFile = file;
        
        // Try to set the file input using DataTransfer
        try {
            const dataTransfer = new DataTransfer();
            dataTransfer.items.add(file);
            
            const fileInput = document.querySelector('#wasm-file');
            fileInput.files = dataTransfer.files;
            
            console.log('[DEBUG] loadSampleModule - fileInput after setting:', fileInput);
            console.log('[DEBUG] loadSampleModule - fileInput.files after setting:', fileInput.files);
            console.log('[DEBUG] loadSampleModule - fileInput.files.length:', fileInput.files.length);
        } catch (error) {
            console.warn('[DEBUG] loadSampleModule - DataTransfer failed:', error);
            // Continue anyway, we have the file stored globally
        }
        
        // Update the file display
        updateFileDisplay(file);
        
        // Set default input based on sample
        const inputText = document.querySelector('#input-text');
        switch (sampleName) {
            case 'calc_add':
                inputText.value = '2 3';
                break;
            case 'echo':
                inputText.value = 'Hello, WasmWiz!';
                break;
            case 'hello_world':
                inputText.value = '';
                break;
        }
        
        performanceMetrics.endTiming('loadSampleTime');
        
        // Show success message
        displayAlert(`Sample module "${sampleName}" loaded successfully!`, 'success');
        
        // Log analytics
        console.debug(`Sample module loaded: ${sampleName}`, {
            loadTime: performanceMetrics.loadSampleTime,
            fileSize: blob.size
        });
    } catch (error) {
        reportError(error, 'Load Sample Module');
        displayAlert(`Error loading sample: ${error.message}`, 'error');
    }
}

function initializeApiKeyManagement() {
    const createKeyForm = document.getElementById('create-api-key-form');
    const listKeysForm = document.getElementById('list-api-keys-form');
    
    if (createKeyForm) {
        createKeyForm.addEventListener('submit', handleCreateApiKey);
    }
    
    if (listKeysForm) {
        listKeysForm.addEventListener('submit', handleListApiKeys);
    }
    
    // Revoke key buttons
    document.querySelectorAll('.revoke-key-button').forEach(button => {
        button.addEventListener('click', async function(e) {
            e.preventDefault();
            const keyId = this.getAttribute('data-key-id');
            if (confirm('Are you sure you want to revoke this API key? This action cannot be undone.')) {
                await revokeApiKey(keyId);
            }
        });
    });
}

async function handleCreateApiKey(event) {
    event.preventDefault();
    
    const form = event.target;
    const formData = new FormData(form);
    const submitButton = form.querySelector('button[type="submit"]');
    const resultDiv = document.getElementById('create-key-result');
    
    if (!resultDiv) return;
    
    // Validate form
    const userEmail = formData.get('user_email');
    if (!userEmail || !isValidEmail(userEmail)) {
        resultDiv.innerHTML = `
            <div class="result error">
                <h4>Validation Error</h4>
                <p>Please enter a valid email address.</p>
            </div>
        `;
        return;
    }
    
    // Disable submit button
    submitButton.disabled = true;
    submitButton.textContent = 'Creating...';
    
    try {
        const response = await fetch('/admin/api-keys', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-CSRF-Token': getCsrfToken()
            },
            body: JSON.stringify({
                user_email: formData.get('user_email'),
                tier_name: formData.get('tier_name')
            })
        });
        
        if (!response.ok) {
            throw new Error(`Server error: ${response.status} ${response.statusText}`);
        }
        
        const result = await response.json();
        
        resultDiv.innerHTML = `
            <div class="result success">
                <h4>API Key Created Successfully!</h4>
                <p><strong>API Key:</strong> <code class="api-key-display">${result.api_key}</code></p>
                <p><strong>Key ID:</strong> ${result.api_key_id}</p>
                <p><strong>Created:</strong> ${new Date(result.created_at).toLocaleString()}</p>
                <div class="warning">
                    ⚠️ <strong>Important:</strong> Save this API key now. You won't be able to see it again!
                </div>
                <button onclick="copyToClipboard('${result.api_key}', 'API Key')" class="btn btn-secondary">
                    📋 Copy API Key
                </button>
            </div>
        `;
        form.reset();
    } catch (error) {
        reportError(error, 'Create API Key');
        
        resultDiv.innerHTML = `
            <div class="result error">
                <h4>Error Creating API Key</h4>
                <p>${error.message}</p>
            </div>
        `;
    } finally {
        // Re-enable submit button
        submitButton.disabled = false;
        submitButton.textContent = 'Create API Key';
    }
}

function isValidEmail(email) {
    const re = /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/;
    return re.test(String(email).toLowerCase());
}

async function handleListApiKeys(event) {
    event.preventDefault();
    
    const form = event.target;
    const formData = new FormData(form);
    const submitButton = form.querySelector('button[type="submit"]');
    const resultDiv = document.getElementById('list-keys-result');
    
    if (!resultDiv) return;
    
    // Validate form
    const userEmail = formData.get('user_email');
    if (!userEmail || !isValidEmail(userEmail)) {
        resultDiv.innerHTML = `
            <div class="result error">
                <h4>Validation Error</h4>
                <p>Please enter a valid email address.</p>
            </div>
        `;
        return;
    }
    
    // Disable submit button
    submitButton.disabled = true;
    submitButton.textContent = 'Loading...';
    
    try {
        const email = formData.get('user_email');
        const response = await fetch(`/admin/api-keys/${encodeURIComponent(email)}`, {
            method: 'GET',
        });
        
        if (!response.ok) {
            throw new Error(`Server error: ${response.status} ${response.statusText}`);
        }
        
        const result = await response.json();
        
        if (result.api_keys && result.api_keys.length > 0) {
            resultDiv.innerHTML = `
                <div class="result success">
                    <h4>API Keys for ${email}</h4>
                    <table class="api-keys-table">
                        <thead>
                            <tr>
                                <th>Key ID</th>
                                <th>Created</th>
                                <th>Tier</th>
                                <th>Status</th>
                                <th>Actions</th>
                            </tr>
                        </thead>
                        <tbody>
                            ${result.api_keys.map(key => `
                                <tr>
                                    <td>${key.id}</td>
                                    <td>${new Date(key.created_at).toLocaleString()}</td>
                                    <td>${key.tier_name}</td>
                                    <td>${key.active ? '<span class="badge success">Active</span>' : '<span class="badge error">Revoked</span>'}</td>
                                    <td>
                                        ${key.active ? 
                                            `<button class="btn btn-small btn-danger revoke-key-button" data-key-id="${key.id}">Revoke</button>` : 
                                            '<span class="text-muted">Revoked</span>'
                                        }
                                    </td>
                                </tr>
                            `).join('')}
                        </tbody>
                    </table>
                </div>
            `;
            
            // Re-initialize revoke buttons
            resultDiv.querySelectorAll('.revoke-key-button').forEach(button => {
                button.addEventListener('click', async function(e) {
                    e.preventDefault();
                    const keyId = this.getAttribute('data-key-id');
                    if (confirm('Are you sure you want to revoke this API key? This action cannot be undone.')) {
                        await revokeApiKey(keyId);
                        // Refresh the list
                        handleListApiKeys(event);
                    }
                });
            });
        } else {
            resultDiv.innerHTML = `
                <div class="result info">
                    <h4>No API Keys Found</h4>
                    <p>No API keys found for ${email}.</p>
                </div>
            `;
        }
    } catch (error) {
        reportError(error, 'List API Keys');
        
        resultDiv.innerHTML = `
            <div class="result error">
                <h4>Error Fetching API Keys</h4>
                <p>${error.message}</p>
            </div>
        `;
    } finally {
        // Re-enable submit button
        submitButton.disabled = false;
        submitButton.textContent = 'List API Keys';
    }
}

async function revokeApiKey(keyId) {
    try {
        const response = await fetch(`/admin/api-keys/${keyId}/deactivate`, {
            method: 'POST',
        });
        
        if (!response.ok) {
            throw new Error(`Server error: ${response.status} ${response.statusText}`);
        }
        
        const result = await response.json();
        
        showToast('API key revoked successfully', 'success');
        return true;
    } catch (error) {
        reportError(error, 'Revoke API Key');
        displayAlert(`Failed to revoke API key: ${error.message}`, 'error');
        return false;
    }
}

// Generate a persistent session ID for analytics
function getSessionId() {
    let sessionId = localStorage.getItem('wasmwiz-session-id');
    if (!sessionId) {
        sessionId = 'session_' + Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15);
        localStorage.setItem('wasmwiz-session-id', sessionId);
    }
    return sessionId;
}

// Error handling for uncaught exceptions
window.addEventListener('error', function(event) {
    reportError(event.error || new Error(event.message), 'Uncaught Exception');
    event.preventDefault();
});

// Handle unhandled promise rejections
window.addEventListener('unhandledrejection', function(event) {
    reportError(event.reason || new Error('Unhandled Promise Rejection'), 'Unhandled Promise Rejection');
    event.preventDefault();
});

// Missing helper functions for form execution
function setLoadingState(button, isLoading) {
    if (!button) return;
    
    if (isLoading) {
        button.disabled = true;
        button.innerHTML = '⏳ Executing...';
        button.classList.add('loading');
    } else {
        button.disabled = false;
        button.innerHTML = '🚀 Execute WebAssembly Module';
        button.classList.remove('loading');
    }
}

function updateProgressStep(stepId, state) {
    // This is a placeholder - implement if you have progress steps in your UI
    console.log(`[DEBUG] Progress step ${stepId}: ${state}`);
}
