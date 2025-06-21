// WasmWiz Main JavaScript
document.addEventListener('DOMContentLoaded', function() {
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
        inputText.addEventListener('input', function(e) {
            validateInputAndShowFeedback(e.target.value);
        });
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
        apiKeyInput.addEventListener('input', function(e) {
            validateApiKeyAndShowFeedback(e.target.value);
        });
        
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
});

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
    const inputType = document.querySelector('input[name="input-type"]:checked').value;
    
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
    
    // Validate file is selected
    if (!fileInput.files || fileInput.files.length === 0) {
        displayAlert('Please select a WebAssembly (.wasm) file', 'error');
        return false;
    }
    
    // Validate file
    if (!validateFile(fileInput.files[0])) {
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
    const form = document.querySelector('#execute-form');
    const fileInput = document.querySelector('#wasm-file');
    const inputText = document.querySelector('#input-text');
    const inputType = document.querySelector('input[name="input-type"]:checked').value;
    const memoryLimit = document.querySelector('#memory-limit').value;
    const timeout = document.querySelector('#timeout').value;
    const resultContainer = document.querySelector('#execution-result');
    const submitButton = document.querySelector('#submit-button');
    const progressContainer = document.querySelector('#progress-container');
    
    // Validate form first
    if (!validateForm()) {
        return;
    }
    
    // Update progress steps
    if (progressContainer) {
        progressContainer.style.display = 'flex';
        updateProgressStep('step-upload', 'active');
    }
    
    // Show enhanced loading state
    setLoadingState(submitButton, true);
    
    try {
        const formData = new FormData();
        formData.append('wasm', fileInput.files[0]);
        formData.append('input', inputText.value);
        formData.append('input_type', inputType);
        formData.append('memory_limit', memoryLimit);
        formData.append('timeout', timeout);
        
        // In development mode, API key is optional
        const apiKey = getApiKey();
        const headers = {};
        if (apiKey && apiKey.trim() !== '') {
            headers['Authorization'] = `Bearer ${apiKey}`;
        }
        
        updateProgressStep('step-upload', 'completed');
        updateProgressStep('step-execute', 'active');
        
        const response = await fetch('/api/execute', {
            method: 'POST',
            body: formData,
            headers: headers
        });
        
        updateProgressStep('step-execute', 'completed');
        updateProgressStep('step-results', 'active');
        
        if (!response.ok) {
            throw new Error(`Server error: ${response.status} ${response.statusText}`);
        }
        
        const result = await response.json();
        
        updateProgressStep('step-results', 'completed');
        
        if (response.ok) {
            displayExecutionResult(result, response.status);
            showToast('Execution completed successfully', 'success');
        } else {
            displayAlert(`Execution failed: ${result.error || 'Unknown error'}`, 'error');
        }
    } catch (error) {
        updateProgressStep('step-execute', 'error');
        updateProgressStep('step-results', 'error');
        displayAlert(`Network error: ${error.message}`, 'error');
    } finally {
        setLoadingState(submitButton, false);
        setTimeout(() => {
            if (progressContainer) progressContainer.style.display = 'none';
        }, 3000);
    }
}

function updateProgressStep(stepId, status) {
    const step = document.getElementById(stepId);
    if (step) {
        // Remove all status classes first
        step.classList.remove('active', 'completed', 'error');
        // Add the new status class
        step.classList.add(status);
    }
}

function setLoadingState(button, loading) {
    if (loading) {
        button.disabled = true;
        button.classList.add('loading');
        button.setAttribute('data-original-text', button.textContent);
        button.innerHTML = '<span class="spinner"></span> Executing...';
    } else {
        button.disabled = false;
        button.classList.remove('loading');
        button.textContent = button.getAttribute('data-original-text') || 'Execute WASM';
    }
}

// Enhanced execution result display
function displayExecutionResult(result, statusCode) {
    const resultContainer = document.querySelector('#execution-result');
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
            </div>
            
            <div class="result-actions">
                <button onclick="clearResults()" class="btn btn-secondary">Clear Results</button>
                <button onclick="downloadResults()" class="btn btn-primary">💾 Download Results</button>
            </div>
        </div>
    `;
    
    resultContainer.innerHTML = resultHtml;
    resultContainer.scrollIntoView({ behavior: 'smooth' });
}

function clearResults() {
    const resultContainer = document.querySelector('#execution-result');
    resultContainer.innerHTML = '';
}

function downloadResults() {
    const resultContainer = document.querySelector('#execution-result');
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
    
    if (content) {
        const blob = new Blob([content], { type: 'text/markdown' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `wasmwiz-result-${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.md`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    }
}

function copyToClipboard(text, type) {
    navigator.clipboard.writeText(text).then(function() {
        showToast(`${type.charAt(0).toUpperCase() + type.slice(1)} copied to clipboard!`, 'success');
    }, function(err) {
        console.error('Could not copy text: ', err);
        showToast('Failed to copy to clipboard', 'error');
    });
}

function displayAlert(message, type = 'info') {
    const notificationArea = document.getElementById('notification-area');
    if (!notificationArea) return;
    
    const alertId = 'alert-' + Date.now();
    const alertHtml = `
        <div id="${alertId}" class="alert alert-${type} alert-animated">
            <div class="alert-content">
                <span class="alert-icon">${getAlertIcon(type)}</span>
                <span class="alert-message">${message}</span>
            </div>
            <button class="alert-close" onclick="dismissAlert('${alertId}')">×</button>
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

async function loadSampleModule(sampleName) {
    // Show loading state
    displayAlert(`Loading sample module: ${sampleName}...`, 'info');
    
    try {
        // Fetch the sample WASM file
        const response = await fetch(`/static/wasm_modules/${sampleName}.wasm`);
        if (!response.ok) {
            throw new Error(`Failed to load sample module: ${response.statusText}`);
        }
        
        const blob = await response.blob();
        
        // Create a File object from the blob
        const file = new File([blob], `${sampleName}.wasm`, { type: 'application/wasm' });
        
        // Set the file input
        const dataTransfer = new DataTransfer();
        dataTransfer.items.add(file);
        
        const fileInput = document.querySelector('#wasm-file');
        fileInput.files = dataTransfer.files;
        
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
        
        // Show success message
        displayAlert(`Sample module "${sampleName}" loaded successfully!`, 'success');
    } catch (error) {
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
    
    // Disable submit button
    submitButton.disabled = true;
    submitButton.textContent = 'Creating...';
    
    try {
        const response = await fetch('/admin/api-keys', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                user_email: formData.get('user_email'),
                tier_name: formData.get('tier_name')
            })
        });
        
        const result = await response.json();
        
        if (response.ok) {
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
        } else {
            resultDiv.innerHTML = `
                <div class="result error">
                    <h4>Error Creating API Key</h4>
                    <p>${result.error || 'Unknown error occurred'}</p>
                </div>
            `;
        }
    } catch (error) {
        resultDiv.innerHTML = `
            <div class="result error">
                <h4>Network Error</h4>
                <p>Failed to create API key: ${error.message}</p>
            </div>
        `;
    } finally {
        // Re-enable submit button
        submitButton.disabled = false;
        submitButton.textContent = 'Create API Key';
    }
}

async function handleListApiKeys(event) {
    event.preventDefault();
    
    const form = event.target;
    const formData = new FormData(form);
    const submitButton = form.querySelector('button[type="submit"]');
    const resultDiv = document.getElementById('list-keys-result');
    
    // Disable submit button
    submitButton.disabled = true;
    submitButton.textContent = 'Loading...';
    
    try {
        const email = formData.get('user_email');
        const response = await fetch(`/admin/api-keys/${encodeURIComponent(email)}`, {
            method: 'GET',
        });
        
        const result = await response.json();
        
        if (response.ok && result.api_keys && result.api_keys.length > 0) {
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
        } else if (response.ok && (!result.api_keys || result.api_keys.length === 0)) {
            resultDiv.innerHTML = `
                <div class="result info">
                    <h4>No API Keys Found</h4>
                    <p>No API keys found for ${email}.</p>
                </div>
            `;
        } else {
            resultDiv.innerHTML = `
                <div class="result error">
                    <h4>Error Fetching API Keys</h4>
                    <p>${result.error || 'Unknown error occurred'}</p>
                </div>
            `;
        }
    } catch (error) {
        resultDiv.innerHTML = `
            <div class="result error">
                <h4>Network Error</h4>
                <p>Failed to fetch API keys: ${error.message}</p>
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
        
        const result = await response.json();
        
        if (response.ok) {
            showToast('API key revoked successfully', 'success');
            return true;
        } else {
            displayAlert(`Failed to revoke API key: ${result.error || 'Unknown error'}`, 'error');
            return false;
        }
    } catch (error) {
        displayAlert(`Network error: ${error.message}`, 'error');
        return false;
    }
}
