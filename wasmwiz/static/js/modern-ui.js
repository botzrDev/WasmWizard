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
        // Show minimal loading state
        this.showToast(`Loading ${sampleType}...`, 'info', 1500);
        
        // Show file info like the original design
        this.showLoadedModuleInfo(sampleType);
        
        // Integrate with existing sample loading logic
        if (window.loadSample) {
            window.loadSample(sampleType);
        }
        
        // Show completion with minimal toast
        setTimeout(() => {
            this.showToast(`${sampleType} loaded`, 'success', 1500);
        }, 800);
    },
    
    showLoadedModuleInfo(sampleType) {
        const uploadZone = document.querySelector('.enhanced-upload-zone');
        const fileInfo = document.querySelector('#file-info');
        const fileName = document.querySelector('#file-name');
        const fileSize = document.querySelector('#file-size');
        const removeButton = document.querySelector('#remove-file');
        
        if (!uploadZone || !fileInfo) return;
        
        // Update upload zone to show loaded state
        uploadZone.classList.add('has-file');
        
        // Map sample types to display info
        const sampleInfo = {
            'calc_add': { name: 'calc_add.wasm', size: '65.27 KB' },
            'echo': { name: 'echo.wasm', size: '23.45 KB' },
            'hello_world': { name: 'hello_world.wasm', size: '18.92 KB' }
        };
        
        const info = sampleInfo[sampleType] || { name: `${sampleType}.wasm`, size: '~25 KB' };
        
        // Show file info
        if (fileName) fileName.textContent = info.name;
        if (fileSize) fileSize.textContent = info.size;
        
        fileInfo.style.display = 'block';
        
        // Remove file handler
        if (removeButton) {
            removeButton.onclick = () => {
                this.clearLoadedModule();
            };
        }
    },
    
    clearLoadedModule() {
        const uploadZone = document.querySelector('.enhanced-upload-zone');
        const fileInfo = document.querySelector('#file-info');
        const fileInput = document.querySelector('#wasm-file');
        
        if (fileInput) fileInput.value = '';
        if (uploadZone) uploadZone.classList.remove('has-file');
        if (fileInfo) fileInfo.style.display = 'none';
        
        // Clear sample selection
        document.querySelectorAll('.sample-card-modern').forEach(card => {
            card.classList.remove('selected');
            card.style.transform = 'translateY(0) scale(1)';
        });
        
        this.state.selectedSample = null;
        this.showToast('Module cleared', 'info', 1000);
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
            this.showToast('Please select a .wasm file', 'error', 2000);
            return;
        }
        
        // Validate file size (10MB limit)
        if (file.size > 10 * 1024 * 1024) {
            this.showToast('File must be <10MB', 'error', 2000);
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
                this.clearLoadedModule();
            };
        }
        
        this.showToast('File selected', 'success', 1500);
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
            this.showToast('Select a WASM file or sample', 'error', 2000);
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
            this.showToast('Execution complete', 'success', 1500);
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
    
    // Minimal toast notification system
    showToast(message, type = 'info', duration = 2000) {
        const toast = document.createElement('div');
        toast.className = `toast toast-${type} toast-minimal`;
        toast.innerHTML = `
            <div class="toast-content-minimal">
                <div class="toast-message-minimal">${message}</div>
            </div>
        `;
        
        // Add to DOM
        document.body.appendChild(toast);
        
        // Position toast (smaller, top-right corner)
        const toasts = document.querySelectorAll('.toast');
        const offset = (toasts.length - 1) * 35;
        toast.style.top = `${15 + offset}px`;
        
        // Auto-remove (shorter duration)
        setTimeout(() => {
            this.removeToast(toast);
        }, duration);
        
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

// Minimal toast styles (injected dynamically)
const toastStyles = `
.toast {
    position: fixed;
    top: 15px;
    right: 15px;
    z-index: 10000;
    opacity: 0;
    transform: translateX(100%);
    transition: all 0.2s ease;
    min-width: 200px;
    max-width: 300px;
}

.toast-minimal .toast-content-minimal {
    display: flex;
    align-items: center;
    padding: 0.5rem 0.75rem;
    border-radius: 6px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    font-size: 0.8rem;
    font-weight: 500;
    color: white;
}

.toast-success .toast-content-minimal { background: #10b981; }
.toast-error .toast-content-minimal { background: #ef4444; }
.toast-warning .toast-content-minimal { background: #f59e0b; }
.toast-info .toast-content-minimal { background: #3b82f6; }

.toast-message-minimal {
    flex: 1;
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
