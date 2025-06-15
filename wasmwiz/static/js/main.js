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
    }
    
    // Real-time input validation
    const inputText = document.querySelector('#input-text');
    if (inputText) {
        inputText.addEventListener('input', function(e) {
            validateInputAndShowFeedback(e.target.value);
        });
    }
    
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
    
    // API Key Management
    initializeApiKeyManagement();
});

function updateFileDisplay(file) {
    const fileInfo = document.querySelector('.file-info');
    if (fileInfo) {
        fileInfo.innerHTML = `
            <div class="file-details">
                <strong>Selected file:</strong> ${file.name}<br>
                <strong>Size:</strong> ${formatFileSize(file.size)}<br>
                <strong>Type:</strong> ${file.type || 'application/wasm'}
            </div>
        `;
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

function validateInput(input) {
    const errors = [];
    const maxSize = 1024 * 1024; // 1MB
    
    if (new Blob([input]).size > maxSize) {
        errors.push('Input size must be less than 1MB');
    }
    
    return errors.length === 0;
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

async function executeWasm() {
    const form = document.querySelector('#execute-form');
    const fileInput = document.querySelector('#wasm-file');
    const inputText = document.querySelector('#input-text');
    const resultContainer = document.querySelector('#execution-result');
    const submitButton = document.querySelector('#submit-button');
    
    // Validate form first
    if (!validateForm()) {
        return;
    }
    
    // Show enhanced loading state
    setLoadingState(submitButton, true);
    showProgressBar(resultContainer);
    
    try {
        const formData = new FormData();
        formData.append('wasm', fileInput.files[0]);
        formData.append('input', inputText.value);
        
        const apiKey = getApiKey();
        if (!apiKey) {
            throw new Error('API key is required');
        }
        
        const response = await fetch('/api/execute', {
            method: 'POST',
            body: formData,
            headers: {
                'Authorization': `Bearer ${apiKey}`
            }
        });
        
        const result = await response.json();
        
        if (response.ok) {
            displayExecutionResult(result, response.status);
        } else {
            displayAlert(`Execution failed: ${result.error || 'Unknown error'}`, 'error');
        }
    } catch (error) {
        displayAlert(`Network error: ${error.message}`, 'error');
    } finally {
        setLoadingState(submitButton, false);
        hideProgressBar();
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

function showProgressBar(container) {
    const progressHtml = `
        <div id="execution-progress" class="progress-container">
            <div class="progress-bar indeterminate">
                <div class="progress-bar-fill"></div>
            </div>
            <p class="progress-text">Executing WebAssembly module...</p>
        </div>
    `;
    container.innerHTML = progressHtml;
}

function hideProgressBar() {
    const progressElement = document.getElementById('execution-progress');
    if (progressElement) {
        progressElement.remove();
    }
}

// Enhanced execution result display
function displayExecutionResult(result, statusCode) {
    const resultContainer = document.querySelector('#execution-result');
    const hasOutput = result.output && result.output.trim().length > 0;
    const hasError = result.error && result.error.trim().length > 0;
    
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
            
            <div class="result-actions">
                <button onclick="clearResults()" class="btn btn-secondary">Clear Results</button>
                <button onclick="downloadResults()" class="btn btn-secondary">💾 Download Results</button>
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
    
    let content = '';
    if (outputElement) {
        content += 'Program Output:\n' + outputElement.textContent + '\n\n';
    }
    if (errorElement) {
        content += 'Error Details:\n' + errorElement.textContent + '\n';
    }
    
    if (content) {
        const blob = new Blob([content], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `wasmwiz-result-${new Date().toISOString().slice(0, 19)}.txt`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    }
}

function copyToClipboard(text, type) {
    navigator.clipboard.writeText(text).then(function() {
        showToast(`${type} copied to clipboard!`, 'success');
    }, function(err) {
        console.error('Could not copy text: ', err);
        showToast('Failed to copy to clipboard', 'error');
    });
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
    // For now, get from localStorage or prompt user
    // In production, this would be handled by authentication
    return localStorage.getItem('wasmwiz-api-key') || prompt('Enter your API key:');
}

function formatFileSize(bytes) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
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
    const userEmail = formData.get('user_email');
    
    // Disable submit button
    submitButton.disabled = true;
    submitButton.textContent = 'Loading...';
    
    try {
        const response = await fetch(`/admin/api-keys/${encodeURIComponent(userEmail)}`);
        const apiKeys = await response.json();
        
        if (response.ok) {
            if (apiKeys.length === 0) {
                resultDiv.innerHTML = `
                    <div class="result info">
                        <p>No API keys found for user: ${userEmail}</p>
                    </div>
                `;
            } else {
                const keysHtml = apiKeys.map(key => `
                    <div class="api-key-item">
                        <div class="key-info">
                            <span class="key-hash">${key.key_hash}</span>
                            <span class="tier-badge tier-${key.tier_name.toLowerCase()}">${key.tier_name}</span>
                            <span class="status ${key.is_active ? 'active' : 'inactive'}">
                                ${key.is_active ? '✓ Active' : '✗ Inactive'}
                            </span>
                        </div>
                        <div class="key-meta">
                            <small>Created: ${new Date(key.created_at).toLocaleString()}</small>
                            ${key.is_active ? `<button onclick="deactivateApiKey('${key.id}')" class="btn-deactivate">Deactivate</button>` : ''}
                        </div>
                    </div>
                `).join('');
                
                resultDiv.innerHTML = `
                    <div class="result success">
                        <h4>API Keys for ${userEmail}</h4>
                        <div class="api-keys-list">
                            ${keysHtml}
                        </div>
                    </div>
                `;
            }
        } else {
            resultDiv.innerHTML = `
                <div class="result error">
                    <h4>Error Loading API Keys</h4>
                    <p>${apiKeys.error || 'Unknown error occurred'}</p>
                </div>
            `;
        }
    } catch (error) {
        resultDiv.innerHTML = `
            <div class="result error">
                <h4>Network Error</h4>
                <p>Failed to load API keys: ${error.message}</p>
            </div>
        `;
    } finally {
        // Re-enable submit button
        submitButton.disabled = false;
        submitButton.textContent = 'List API Keys';
    }
}

async function deactivateApiKey(keyId) {
    if (!confirm('Are you sure you want to deactivate this API key?')) {
        return;
    }
    
    try {
        const response = await fetch(`/admin/api-keys/${keyId}/deactivate`, {
            method: 'POST'
        });
        
        const result = await response.json();
        
        if (response.ok) {
            // Refresh the list
            const listForm = document.getElementById('list-api-keys-form');
            if (listForm) {
                listForm.dispatchEvent(new Event('submit'));
            }
        } else {
            alert(`Error deactivating API key: ${result.error || 'Unknown error'}`);
        }
    } catch (error) {
        alert(`Network error: ${error.message}`);
    }
}

function validateFileAndShowFeedback(file) {
    const errors = [];
    
    if (!validateFile(file)) {
        errors.push('Invalid file selected');
    }
    
    showValidationFeedback('file', errors);
    return errors.length === 0;
}

function validateInputAndShowFeedback(input) {
    const errors = [];
    
    if (!validateInput(input)) {
        errors.push('Input too large (max 1MB)');
    }
    
    showValidationFeedback('input', errors);
    return errors.length === 0;
}

function validateApiKeyAndShowFeedback(apiKey) {
    const errors = [];
    
    if (!apiKey || apiKey.trim().length === 0) {
        errors.push('API key is required');
    } else if (!apiKey.startsWith('ww_')) {
        errors.push('Invalid API key format (should start with "ww_")');
    } else if (apiKey.length < 30) {
        errors.push('API key appears to be too short');
    }
    
    showValidationFeedback('api-key', errors);
    return errors.length === 0;
}

function showValidationFeedback(fieldType, errors) {
    const fieldElement = document.querySelector(`#${fieldType.replace('api-key', 'api-key')}`);
    if (!fieldElement) return;
    
    // Remove existing feedback
    const existingFeedback = fieldElement.parentElement.querySelector('.validation-feedback');
    if (existingFeedback) {
        existingFeedback.remove();
    }
    
    if (errors.length > 0) {
        const feedback = document.createElement('div');
        feedback.className = 'validation-feedback error';
        feedback.innerHTML = errors.map(error => `<small>❌ ${error}</small>`).join('<br>');
        fieldElement.parentElement.appendChild(feedback);
        fieldElement.classList.add('error');
    } else if (fieldElement.value || fieldElement.files?.length > 0) {
        const feedback = document.createElement('div');
        feedback.className = 'validation-feedback success';
        feedback.innerHTML = '<small>✅ Valid</small>';
        fieldElement.parentElement.appendChild(feedback);
        fieldElement.classList.remove('error');
        fieldElement.classList.add('success');
    }
}

function validateForm() {
    const fileInput = document.querySelector('#wasm-file');
    const inputText = document.querySelector('#input-text');
    const apiKeyInput = document.querySelector('#api-key');
    
    let isValid = true;
    
    // Validate file
    if (!fileInput.files[0]) {
        showValidationFeedback('file', ['Please select a WASM file']);
        isValid = false;
    } else {
        isValid = validateFileAndShowFeedback(fileInput.files[0]) && isValid;
    }
    
    // Validate input
    isValid = validateInputAndShowFeedback(inputText.value) && isValid;
    
    // Validate API key
    isValid = validateApiKeyAndShowFeedback(apiKeyInput.value) && isValid;
    
    return isValid;
}
