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
Refer to ERD and DOCS for backend integration details and data flows.
