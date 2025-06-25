// cypress/e2e/result_buttons.cy.js

describe('WasmWiz Result Buttons', () => {
  beforeEach(() => {
    cy.visit('/');
  });

  it('should display result card and both buttons after testResultDisplay()', () => {
    // Ensure the result area is empty
    cy.get('#execution-result').should('be.empty');

    // Trigger the testResultDisplay function in the app context
    cy.window().then((win) => {
      win.testResultDisplay();
    });

    // The result card should now be present
    cy.get('.execution-result-card').should('exist');

    // The Clear Results button should be visible and clickable
    cy.get('.result-actions .btn.btn-secondary').should('be.visible').and('contain', 'Clear Results');

    // The Download Results button should be visible and clickable
    cy.get('.result-actions .btn.btn-primary').should('be.visible').and('contain', 'Download Results');
  });

  it('should clear the result card when Clear Results is clicked', () => {
    cy.window().then((win) => {
      win.testResultDisplay();
    });
    cy.get('.result-actions .btn.btn-secondary').click();
    cy.get('#execution-result').should('be.empty');
  });
});
