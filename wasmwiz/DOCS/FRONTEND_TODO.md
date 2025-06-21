# WasmWiz Frontend TODO

## 0. Key Requirements (Summary)
- Home page drag-and-drop WASM execution is open to all (no sign-up required)
- All other features require an account (OAuth: Google, GitHub)
- API key management for logged-in users (multiple keys, labels, collective billing)
- Input: plain text, JSON, binary (reasonable size limits)
- Logs for paid users; output only for free/anonymous
- Downloadable results for paid users
- Sample gallery (platform and user samples)
- Upgrade path and usage stats (monthly, per-key)
- Color scheme: dark royal purple, cobalt blue, white, grey (Jira/ChatGPT inspired)
- Banners for notifications
- Accessibility and top 5 language support
- Feedback, FAQ, and docs for signed-up users only

## 1. Design System & Visual Identity
- [ ] Define color palette and typography (dark royal purple, cobalt blue, white, grey)
- [ ] Add logo placeholder
- [ ] Create reusable UI components (buttons, cards, banners, modals, navigation)
- [ ] Layout and spacing standards

## 2. Main Upload Interface (index.html)
- [ ] Drag-and-drop WASM file upload (open to all)
- [ ] File validation and visual feedback
- [ ] Input data area (plain text, JSON, binary, with validation)
- [ ] Sample input dropdown and sample gallery
- [ ] Execution options (memory, timeout)
- [ ] Result display (output for all, logs for paid)
- [ ] Downloadable results (paid only)
- [ ] Copy, syntax highlighting, metadata display
- [ ] Banners for notifications/errors
- [ ] Responsive and accessible design
- [ ] Language selector (top 5 languages)

## 3. Authentication & User Management
- [ ] OAuth sign-in/registration (Google, GitHub)
- [ ] Account dashboard for logged-in users
- [ ] Access control for premium features

## 4. API Key Management
- [ ] API key dashboard (list, create, revoke, label)
- [ ] Usage stats (monthly, per-key)
- [ ] Key rotation

## 5. Subscription & Usage
- [ ] Display current tier and upgrade options
- [ ] Usage stats (monthly, per-key)
- [ ] Upgrade workflow

## 6. Feedback, FAQ, Help & Docs
- [ ] Feedback form (signed-up users only)
- [ ] FAQ section (signed-up users only)
- [ ] Help/documentation (signed-up users only)

## 7. JavaScript Architecture
- [ ] Modular JS (api.js, validation.js, ui.js)
- [ ] Robust form submission, progress tracking
- [ ] Error handling, retry logic, offline detection

## 8. Testing
- [ ] Unit tests for JS modules
- [ ] End-to-end tests for user flows
- [ ] Visual regression tests
- [ ] Cross-browser and mobile testing

## 9. Accessibility & Internationalization
- [ ] Keyboard navigation, ARIA roles, color contrast
- [ ] Language selector and translation for top 5 languages

## 10. Implementation Timeline
- [ ] Phase 1: Core UI & Design System
- [ ] Phase 2: Interactive Features
- [ ] Phase 3: JS Architecture & Error Handling
- [ ] Phase 4: Testing & Finalization

---

# Sprint 1: Core UI & Public Upload Experience ✅ COMPLETED

## Goals ✅
- ✅ Deliver a visually appealing, accessible, and responsive homepage for anonymous WASM execution.
- ✅ Lay the foundation for the design system and component library.

## Sprint 1 Tasks ✅ ALL COMPLETED
- ✅ Set up color palette, typography, and base layout (dark royal purple, cobalt blue, white, grey)
- ✅ Implement drag-and-drop WASM upload zone (with validation and feedback)
- ✅ Input data area (plain text, JSON, binary, with validation)
- ✅ Basic execution options (memory, timeout)
- ✅ Display output/results (final output only, no logs)
- ✅ Show execution metadata (time, memory)
- ✅ Add sample gallery with at least 2 platform-provided WASM modules
- ✅ Responsive design for mobile/desktop
- ✅ Accessibility: keyboard navigation, ARIA roles, color contrast
- ✅ Banner notifications for errors/status
- ✅ Language selector (UI only, no translations yet)
- ✅ Placeholder for sign-in/registration (no backend integration yet)

---

# Sprint 2: Minimal Backend Integration & Working Demo

## Goals
- Get the frontend fully functional with a working backend
- Implement anonymous WASM execution (no authentication required)
- Create a complete vertical slice from upload to execution results
- Remove database dependency for basic functionality

## Sprint 2 Tasks - Backend Integration
- [ ] Create database-optional backend mode for development/demo
- [ ] Implement anonymous WASM execution endpoint (/api/execute)
- [ ] File upload handling and validation on backend
- [ ] WASM runtime integration for actual execution
- [ ] Error handling and response formatting
- [ ] Static file serving for frontend assets
- [ ] CORS configuration for frontend-backend communication

## Sprint 2 Tasks - Frontend Integration
- [ ] Replace simulated execution with real API calls
- [ ] Implement proper file upload to backend
- [ ] Handle real execution responses and errors
- [ ] Display actual WASM execution output
- [ ] Show real execution metadata (time, memory usage)
- [ ] Implement proper error handling for API failures
- [ ] Add loading states during actual execution

## Sprint 2 Tasks - Sample Module Integration
- [ ] Integrate real sample WASM modules with backend
- [ ] Implement sample loading from backend
- [ ] Test all three sample modules (calc_add, echo, hello_world)
- [ ] Validate sample execution works end-to-end

## Sprint 2 Tasks - Development Setup
- [ ] Create development configuration without database
- [ ] Set up simple file-based storage for demos
- [ ] Configure server to serve both API and static files
- [ ] Add development documentation for running the demo

## Acceptance Criteria
- ✅ User can visit the homepage and see the frontend
- ✅ User can upload a .wasm file or select a sample
- ✅ User can provide input and adjust execution options
- ✅ User can execute WASM and see real output
- ✅ All three sample modules work correctly
- ✅ Error handling shows meaningful messages
- ✅ Frontend is fully responsive and accessible
- ✅ No authentication required for basic functionality

## Technical Requirements
- Backend runs without database dependency
- Frontend makes real API calls instead of simulations
- WASM execution uses the actual Wasmer runtime
- All static assets served correctly
- CORS properly configured
- File uploads handled securely
- Execution sandboxing and limits enforced

This sprint will deliver a fully working demo that showcases the core WasmWiz functionality!

---

# Sprint 3: Functional Vertical Slice - Frontend Working Demo

## Goals
- Get the frontend fully functional with a minimal backend in demo mode
- Complete anonymous WASM execution from upload to results display
- Fix all compilation errors and deliver a working vertical slice
- Enable users to actually use the application in a browser

## Sprint 3 Tasks - Backend Demo Mode Completion
- [ ] Fix middleware compilation errors (Optional<AuthMiddleware> issues)
- [ ] Complete demo mode configuration for all handlers
- [ ] Bypass authentication for demo mode in all middleware
- [ ] Fix database optional patterns in all handlers
- [ ] Handle Environment::Demo in logging.rs
- [ ] Ensure execute endpoint works without database in demo mode
- [ ] Add static file serving for frontend assets
- [ ] Configure CORS for frontend-backend communication

## Sprint 3 Tasks - Frontend Integration & Testing
- [ ] Replace all simulated execution with real backend API calls
- [ ] Test file upload to backend works correctly
- [ ] Verify WASM execution returns real output
- [ ] Test all three sample modules work end-to-end
- [ ] Implement proper error handling for backend failures
- [ ] Add loading states during real execution
- [ ] Test responsive design on mobile and desktop
- [ ] Verify accessibility features work correctly

## Sprint 3 Tasks - Demo Polish & Documentation
- [ ] Create simple development setup instructions
- [ ] Test the complete user flow: visit → upload/select → execute → view results
- [ ] Add demo mode indicators in the UI
- [ ] Ensure sample gallery integrates with backend samples
- [ ] Document known limitations for demo mode
- [ ] Create screenshots for documentation

## Sprint 3 Tasks - Essential UI Improvements
- [ ] Improve file upload feedback and validation messages
- [ ] Add execution time and memory usage display
- [ ] Enhance result display formatting (syntax highlighting for JSON/text)
- [ ] Add copy-to-clipboard functionality for results
- [ ] Improve notification banners for different types of messages
- [ ] Add better loading indicators during execution

## Acceptance Criteria ✅ Success Metrics
- [ ] User can visit http://localhost:8080 and see the homepage
- [ ] User can drag-and-drop a .wasm file and see upload feedback
- [ ] User can select from sample WASM modules
- [ ] User can provide input (text/JSON) and adjust execution options
- [ ] User can click "Execute WASM" and see real execution happen
- [ ] User sees actual WASM output (not simulated)
- [ ] User sees execution metadata (time, memory)
- [ ] All three sample modules (calc_add, echo, hello_world) work
- [ ] Error messages are helpful when things go wrong
- [ ] Page works on mobile and desktop
- [ ] Keyboard navigation works for accessibility
- [ ] No authentication required - fully anonymous usage

## Technical Requirements
- Backend compiles without errors
- Frontend makes real HTTP requests to backend
- WASM execution uses actual Wasmer runtime
- File uploads handled securely with size limits
- CORS configured properly for local development
- Static assets served correctly
- Error responses properly formatted and handled
- Demo mode clearly documented

## Sprint 3 Priority Order
1. **Fix compilation errors** - Backend must compile and run
2. **Basic API integration** - Frontend must connect to backend
3. **WASM execution** - Core functionality must work
4. **Sample modules** - All samples must execute correctly
5. **Error handling** - Failures must be handled gracefully
6. **UI polish** - Improve user experience and feedback
7. **Documentation** - Setup and usage instructions

This sprint will deliver the first fully working version of WasmWiz that users can actually use in their browser!
