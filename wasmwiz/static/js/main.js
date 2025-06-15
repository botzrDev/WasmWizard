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
            }
        });
        
        // File input change
        fileInput.addEventListener('change', function(e) {
            if (e.target.files.length > 0) {
                updateFileDisplay(e.target.files[0]);
            }
        });
    }
    
    // Form validation
    const executeForm = document.querySelector('#execute-form');
    if (executeForm) {
        executeForm.addEventListener('submit', function(e) {
            e.preventDefault();
            executeWasm();
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
    
    // Validate inputs
    if (!fileInput.files[0]) {
        displayAlert('Please select a WASM file', 'error');
        return;
    }
    
    if (!validateFile(fileInput.files[0])) {
        return;
    }
    
    if (!validateInput(inputText.value)) {
        displayAlert('Input validation failed', 'error');
        return;
    }
    
    // Show loading state
    submitButton.disabled = true;
    submitButton.innerHTML = '<span class="spinner"></span> Executing...';
    resultContainer.innerHTML = '<div class="alert">Executing WASM module...</div>';
    
    try {
        const formData = new FormData();
        formData.append('wasm', fileInput.files[0]);
        formData.append('input', inputText.value);
        
        const response = await fetch('/api/execute', {
            method: 'POST',
            body: formData,
            headers: {
                'Authorization': `Bearer ${getApiKey()}`
            }
        });
        
        const result = await response.json();
        
        if (response.ok) {
            displayExecutionResult(result);
        } else {
            displayAlert(`Execution failed: ${result.error || 'Unknown error'}`, 'error');
        }
    } catch (error) {
        displayAlert(`Network error: ${error.message}`, 'error');
    } finally {
        // Reset button state
        submitButton.disabled = false;
        submitButton.innerHTML = 'Execute WASM';
    }
}

function displayExecutionResult(result) {
    const resultContainer = document.querySelector('#execution-result');
    
    if (result.error) {
        resultContainer.innerHTML = `
            <div class="alert alert-error">
                <strong>Execution Error:</strong><br>
                <div class="code-output">${escapeHtml(result.error)}</div>
            </div>
        `;
    } else {
        resultContainer.innerHTML = `
            <div class="alert alert-success">
                <strong>Execution Successful!</strong>
            </div>
            <div class="card">
                <div class="card-header">
                    <h3 class="card-title">Output</h3>
                </div>
                <div class="code-output">${escapeHtml(result.output || '(no output)')}</div>
            </div>
        `;
    }
}

function displayAlert(message, type = 'error') {
    const resultContainer = document.querySelector('#execution-result');
    resultContainer.innerHTML = `
        <div class="alert alert-${type}">
            ${escapeHtml(message)}
        </div>
    `;
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
