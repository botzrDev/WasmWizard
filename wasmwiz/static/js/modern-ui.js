/**
 * WasmWiz Modern UI Enhancements
 * @version 1.0.0
 * @description Enhanced UI/UX components and interactions
 */

// Modern UI state management
const ModernUI = {
    state: {
        currentInputType: 'text',
        isExecuting: false,
        selectedSample: null,
        collapsedSections: new Set(['advanced-options', 'api-key-section'])
    },
    
    init() {
        this.initializeComponents();
        this.bindEvents();
        this.loadUserPreferences();
    },
    
    initializeComponents() {
        this.initializeCollapsibleSections();
        this.initializeInputTabs();
        this.initializeSampleCards();
        this.initializeFloatingActionButton();
        this.initializeEnhancedUploadZone();
        this.initializeProgressSystem();
    },
    
    bindEvents() {
        // Enhanced form submission
        const form = document.querySelector('#execute-form');
        if (form) {
            form.addEventListener('submit', this.handleFormSubmission.bind(this));
        }
        
        // Auto-save user preferences
        this.bindUserPreferenceEvents();
        
        // Keyboard shortcuts
        this.initializeKeyboardShortcuts();
    },
    
    // Collapsible sections with smooth animations
    initializeCollapsibleSections() {
        const sections = document.querySelectorAll('.collapsible-section');
        
        sections.forEach(section => {
            const header = section.querySelector('.collapsible-header');
            const content = section.querySelector('.collapsible-content');
            const icon = section.querySelector('.collapsible-icon');
            
            if (!header || !content) return;
            
            // Set initial state
            const sectionId = section.id;
            const isCollapsed = this.state.collapsedSections.has(sectionId);
            
            if (isCollapsed) {
                content.style.maxHeight = '0px';
                icon.style.transform = 'rotate(0deg)';
                section.classList.remove('expanded');
            } else {
                content.style.maxHeight = content.scrollHeight + 'px';
                icon.style.transform = 'rotate(180deg)';
                section.classList.add('expanded');
            }
            
            header.addEventListener('click', () => {
                this.toggleCollapsibleSection(section);
            });
        });
    },
    
    toggleCollapsibleSection(section) {
        const content = section.querySelector('.collapsible-content');
        const icon = section.querySelector('.collapsible-icon');
        const sectionId = section.id;
        const isExpanded = section.classList.contains('expanded');
        
        if (isExpanded) {
            // Collapse
            content.style.maxHeight = '0px';
            icon.style.transform = 'rotate(0deg)';
            section.classList.remove('expanded');
            this.state.collapsedSections.add(sectionId);
        } else {
            // Expand
            content.style.maxHeight = content.scrollHeight + 'px';
            icon.style.transform = 'rotate(180deg)';
            section.classList.add('expanded');
            this.state.collapsedSections.delete(sectionId);
        }
        
        this.saveUserPreferences();
    },
    
    // Enhanced input tabs with smooth transitions
    initializeInputTabs() {
        const tabs = document.querySelectorAll('.input-tab');
        const inputField = document.querySelector('#input-text');
        
        tabs.forEach(tab => {
            tab.addEventListener('click', () => {
                this.switchInputTab(tab, inputField);
            });
        });
    },
    
    switchInputTab(activeTab, inputField) {
        const tabs = document.querySelectorAll('.input-tab');
        const inputType = activeTab.dataset.inputType;
        
        // Remove active class from all tabs
        tabs.forEach(tab => tab.classList.remove('active'));
        
        // Add active class to clicked tab
        activeTab.classList.add('active');
        
        // Update state
        this.state.currentInputType = inputType;
        
        // Update input field with animation
        this.updateInputField(inputType, inputField);
        
        // Save preference
        this.saveUserPreferences();
    },
    
    updateInputField(type, inputField) {
        const placeholders = {
            text: 'Enter plain text input for your WebAssembly module...\n\nExample:\nHello, WebAssembly!',
            json: '{\n  "message": "Hello, WebAssembly!",\n  "numbers": [1, 2, 3, 4, 5],\n  "config": {\n    "debug": true,\n    "timeout": 5000\n  }\n}',
            binary: 'SGVsbG8sIFdlYkFzc2VtYmx5IQ==\n\n// Base64 encoded binary data\n// Decode this in your WASM module'
        };
        
        // Add transition effect
        inputField.style.opacity = '0.5';
        
        setTimeout(() => {
            inputField.placeholder = placeholders[type] || placeholders.text;
            inputField.className = `modern-form-input input-type-${type}`;
            inputField.style.opacity = '1';
        }, 150);
    },
    
    // Enhanced sample cards with better feedback
    initializeSampleCards() {
        const sampleCards = document.querySelectorAll('.sample-card-modern');
        
        sampleCards.forEach(card => {
            card.addEventListener('click', () => {
                this.selectSampleCard(card);
            });
            
            // Add hover effects
            card.addEventListener('mouseenter', () => {
                card.style.transform = 'translateY(-2px) scale(1.02)';
            });
            
            card.addEventListener('mouseleave', () => {
                if (!card.classList.contains('selected')) {
                    card.style.transform = 'translateY(0) scale(1)';
                }
            });
        });
    },
    
    selectSampleCard(selectedCard) {
        const sampleCards = document.querySelectorAll('.sample-card-modern');
        const sampleType = selectedCard.dataset.sample;
        
        // Remove selected class from all cards
        sampleCards.forEach(card => {
            card.classList.remove('selected');
            card.style.transform = 'translateY(0) scale(1)';
        });
        
        // Add selected class and animation to clicked card
        selectedCard.classList.add('selected');
        selectedCard.style.transform = 'translateY(-2px) scale(1.02)';
        
        // Update state
        this.state.selectedSample = sampleType;
        
        // Load sample with enhanced feedback
        this.loadSampleWithFeedback(sampleType);
    },
    
    loadSampleWithFeedback(sampleType) {
        // Show loading state
        this.showToast(`Loading ${sampleType} sample...`, 'info');
        
        // Integrate with existing sample loading logic
        if (window.loadSample) {
            window.loadSample(sampleType);
        }
        
        // Simulate loading completion (replace with actual callback)
        setTimeout(() => {
            this.showToast(`${sampleType} sample loaded successfully!`, 'success');
        }, 1000);
    },
    
    // Floating Action Button with smart behavior
    initializeFloatingActionButton() {
        const fab = document.querySelector('#fab-execute');
        const submitButton = document.querySelector('#submit-button');
        let isVisible = false;
        
        if (!fab || !submitButton) return;
        
        // Smart visibility based on scroll and form state
        let scrollTimeout;
        window.addEventListener('scroll', () => {
            clearTimeout(scrollTimeout);
            
            const shouldShow = window.scrollY > 300 && !this.state.isExecuting;
            
            if (shouldShow !== isVisible) {
                isVisible = shouldShow;
                fab.style.display = shouldShow ? 'flex' : 'none';
                fab.style.opacity = shouldShow ? '1' : '0';
            }
        });
        
        // FAB click handler with smooth scroll
        fab.addEventListener('click', () => {
            window.scrollTo({ 
                top: 0, 
                behavior: 'smooth' 
            });
            
            setTimeout(() => {
                submitButton.focus();
                this.pulseElement(submitButton);
            }, 600);
        });
    },
    
    // Enhanced upload zone with better visual feedback
    initializeEnhancedUploadZone() {
        const uploadZone = document.querySelector('.enhanced-upload-zone');
        const fileInput = document.querySelector('#wasm-file');
        const fileInfo = document.querySelector('#file-info');
        
        if (!uploadZone || !fileInput) return;
        
        // Enhanced drag and drop
        uploadZone.addEventListener('dragover', (e) => {
            e.preventDefault();
            uploadZone.classList.add('dragover');
        });
        
        uploadZone.addEventListener('dragleave', (e) => {
            e.preventDefault();
            if (!uploadZone.contains(e.relatedTarget)) {
                uploadZone.classList.remove('dragover');
            }
        });
        
        uploadZone.addEventListener('drop', (e) => {
            e.preventDefault();
            uploadZone.classList.remove('dragover');
            
            const files = e.dataTransfer.files;
            if (files.length > 0) {
                this.handleFileSelection(files[0], fileInput, uploadZone, fileInfo);
            }
        });
        
        // File input change
        fileInput.addEventListener('change', (e) => {
            if (e.target.files.length > 0) {
                this.handleFileSelection(e.target.files[0], fileInput, uploadZone, fileInfo);
            }
        });
        
        // Click to browse
        uploadZone.addEventListener('click', () => {
            fileInput.click();
        });
    },
    
    handleFileSelection(file, fileInput, uploadZone, fileInfo) {
        // Validate file type
        if (!file.name.endsWith('.wasm')) {
            this.showToast('Please select a valid .wasm file', 'error');
            return;
        }
        
        // Validate file size (10MB limit)
        if (file.size > 10 * 1024 * 1024) {
            this.showToast('File size must be less than 10MB', 'error');
            return;
        }
        
        // Update UI
        uploadZone.classList.add('has-file');
        
        // Show file info
        const fileName = fileInfo.querySelector('#file-name');
        const fileSize = fileInfo.querySelector('#file-size');
        const removeButton = fileInfo.querySelector('#remove-file');
        
        if (fileName) fileName.textContent = file.name;
        if (fileSize) fileSize.textContent = this.formatFileSize(file.size);
        
        fileInfo.style.display = 'block';
        
        // Remove file handler
        if (removeButton) {
            removeButton.onclick = () => {
                fileInput.value = '';
                uploadZone.classList.remove('has-file');
                fileInfo.style.display = 'none';
                this.showToast('File removed', 'info');
            };
        }
        
        this.showToast(`File "${file.name}" selected successfully`, 'success');
    },
    
    // Enhanced progress system
    initializeProgressSystem() {
        this.progressSteps = [
            { id: 'step-upload', label: 'Uploading', percentage: 33 },
            { id: 'step-execute', label: 'Executing', percentage: 66 },
            { id: 'step-results', label: 'Processing Results', percentage: 100 }
        ];
    },
    
    showProgress() {
        const container = document.querySelector('#progress-container');
        if (container) {
            container.style.display = 'block';
            this.updateProgress(0, 0);
        }
    },
    
    updateProgress(stepIndex, percentage) {
        const progressBar = document.querySelector('#progress-bar');
        const progressPercentage = document.querySelector('#progress-percentage');
        const steps = document.querySelectorAll('.progress-step-modern');
        
        if (progressBar) {
            progressBar.style.width = percentage + '%';
        }
        
        if (progressPercentage) {
            progressPercentage.textContent = Math.round(percentage) + '%';
        }
        
        // Update step indicators
        steps.forEach((stepEl, index) => {
            stepEl.classList.remove('active', 'completed');
            
            if (index < stepIndex) {
                stepEl.classList.add('completed');
            } else if (index === stepIndex) {
                stepEl.classList.add('active');
            }
        });
    },
    
    hideProgress() {
        const container = document.querySelector('#progress-container');
        if (container) {
            setTimeout(() => {
                container.style.display = 'none';
            }, 1000);
        }
    },
    
    // Enhanced form submission with better UX
    handleFormSubmission(event) {
        event.preventDefault();
        
        if (this.state.isExecuting) {
            return;
        }
        
        // Validate form
        if (!this.validateForm()) {
            return;
        }
        
        // Start execution
        this.setExecutingState(true);
        this.showProgress();
        
        // Simulate execution steps (replace with actual API calls)
        this.simulateExecution();
    },
    
    validateForm() {
        const fileInput = document.querySelector('#wasm-file');
        
        if (!fileInput.files.length && !this.state.selectedSample) {
            this.showToast('Please select a WASM file or sample module', 'error');
            return false;
        }
        
        return true;
    },
    
    simulateExecution() {
        // Step 1: Upload
        setTimeout(() => {
            this.updateProgress(0, 33);
        }, 500);
        
        // Step 2: Execute
        setTimeout(() => {
            this.updateProgress(1, 66);
        }, 1500);
        
        // Step 3: Results
        setTimeout(() => {
            this.updateProgress(2, 100);
            this.setExecutingState(false);
            this.hideProgress();
            this.showToast('Execution completed successfully!', 'success');
        }, 3000);
    },
    
    setExecutingState(isExecuting) {
        this.state.isExecuting = isExecuting;
        
        const fab = document.querySelector('#fab-execute');
        const submitButton = document.querySelector('#submit-button');
        
        if (isExecuting) {
            if (fab) fab.classList.add('executing');
            if (submitButton) {
                submitButton.disabled = true;
                submitButton.innerHTML = '<span class="spinner"></span> Executing...';
            }
        } else {
            if (fab) fab.classList.remove('executing');
            if (submitButton) {
                submitButton.disabled = false;
                submitButton.innerHTML = '🚀 Execute WebAssembly Module';
            }
        }
    },
    
    // User preferences management
    loadUserPreferences() {
        try {
            const prefs = localStorage.getItem('wasmwiz-modern-ui-prefs');
            if (prefs) {
                const parsed = JSON.parse(prefs);
                this.state.currentInputType = parsed.inputType || 'text';
                this.state.collapsedSections = new Set(parsed.collapsedSections || []);
            }
        } catch (e) {
            console.warn('Failed to load user preferences:', e);
        }
    },
    
    saveUserPreferences() {
        try {
            const prefs = {
                inputType: this.state.currentInputType,
                collapsedSections: Array.from(this.state.collapsedSections)
            };
            localStorage.setItem('wasmwiz-modern-ui-prefs', JSON.stringify(prefs));
        } catch (e) {
            console.warn('Failed to save user preferences:', e);
        }
    },
    
    bindUserPreferenceEvents() {
        // Auto-save API key
        const apiKeyInput = document.querySelector('#api-key');
        if (apiKeyInput) {
            apiKeyInput.addEventListener('change', () => {
                localStorage.setItem('wasmwiz-api-key', apiKeyInput.value);
            });
            
            // Load saved API key
            const savedApiKey = localStorage.getItem('wasmwiz-api-key');
            if (savedApiKey) {
                apiKeyInput.value = savedApiKey;
            }
        }
    },
    
    // Keyboard shortcuts
    initializeKeyboardShortcuts() {
        document.addEventListener('keydown', (e) => {
            // Ctrl/Cmd + Enter to execute
            if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
                e.preventDefault();
                const submitButton = document.querySelector('#submit-button');
                if (submitButton && !this.state.isExecuting) {
                    submitButton.click();
                }
            }
            
            // Escape to clear file
            if (e.key === 'Escape') {
                const removeButton = document.querySelector('#remove-file');
                if (removeButton && removeButton.style.display !== 'none') {
                    removeButton.click();
                }
            }
        });
    },
    
    // Toast notification system
    showToast(message, type = 'info', duration = 3000) {
        const toast = document.createElement('div');
        toast.className = `toast toast-${type}`;
        toast.innerHTML = `
            <div class="toast-content">
                <div class="toast-icon">${this.getToastIcon(type)}</div>
                <div class="toast-message">${message}</div>
                <button class="toast-close">×</button>
            </div>
        `;
        
        // Add to DOM
        document.body.appendChild(toast);
        
        // Position toast
        const toasts = document.querySelectorAll('.toast');
        const offset = (toasts.length - 1) * 60;
        toast.style.top = `${20 + offset}px`;
        
        // Auto-remove
        setTimeout(() => {
            this.removeToast(toast);
        }, duration);
        
        // Manual close
        toast.querySelector('.toast-close').addEventListener('click', () => {
            this.removeToast(toast);
        });
        
        // Animate in
        setTimeout(() => {
            toast.style.opacity = '1';
            toast.style.transform = 'translateX(0)';
        }, 10);
    },
    
    removeToast(toast) {
        toast.style.opacity = '0';
        toast.style.transform = 'translateX(100%)';
        
        setTimeout(() => {
            if (toast.parentNode) {
                toast.parentNode.removeChild(toast);
            }
        }, 300);
    },
    
    getToastIcon(type) {
        const icons = {
            success: '✅',
            error: '❌',
            warning: '⚠️',
            info: 'ℹ️'
        };
        return icons[type] || icons.info;
    },
    
    // Utility functions
    formatFileSize(bytes) {
        if (bytes === 0) return '0 Bytes';
        
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    },
    
    pulseElement(element) {
        element.style.transform = 'scale(1.05)';
        element.style.transition = 'transform 0.2s ease';
        
        setTimeout(() => {
            element.style.transform = 'scale(1)';
        }, 200);
    }
};

// Toast styles (injected dynamically)
const toastStyles = `
.toast {
    position: fixed;
    top: 20px;
    right: 20px;
    z-index: 10000;
    opacity: 0;
    transform: translateX(100%);
    transition: all 0.3s ease;
    min-width: 300px;
    max-width: 400px;
}

.toast-content {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 1rem;
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    color: white;
}

.toast-success .toast-content { background: var(--success-color); }
.toast-error .toast-content { background: var(--error-color); }
.toast-warning .toast-content { background: var(--warning-color); }
.toast-info .toast-content { background: var(--info-color); }

.toast-icon {
    font-size: 1.25rem;
    flex-shrink: 0;
}

.toast-message {
    flex: 1;
    font-weight: 500;
}

.toast-close {
    background: transparent;
    border: none;
    color: white;
    font-size: 1.25rem;
    cursor: pointer;
    opacity: 0.8;
    padding: 0;
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
}

.toast-close:hover {
    opacity: 1;
}
`;

// Inject toast styles
const styleSheet = document.createElement('style');
styleSheet.textContent = toastStyles;
document.head.appendChild(styleSheet);

// Initialize when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => ModernUI.init());
} else {
    ModernUI.init();
}

// Export for integration with existing code
window.ModernUI = ModernUI;
