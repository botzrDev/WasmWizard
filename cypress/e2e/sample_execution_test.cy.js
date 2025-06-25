describe('Sample Loading and Execution', () => {
  beforeEach(() => {
    cy.visit('http://127.0.0.1:8081');
    cy.wait(1000); // Wait for page to load
  });

  it('should load echo sample and execute successfully', () => {
    // Click on Echo Module sample
    cy.get('[data-sample="echo"]').click();
    
    // Wait for the sample to load
    cy.wait(3000);
    
    // Verify file info is displayed
    cy.get('#file-info').should('be.visible');
    cy.get('#file-name').should('contain', 'echo.wasm');
    
    // Add input text
    cy.get('#input-text').clear().type('Hello, WasmWiz Test!');
    
    // Execute the module
    cy.get('#submit-button').click();
    
    // Wait for execution to complete (increase timeout)
    cy.wait(8000);
    
    // Check if execution result appears
    cy.get('#execution-result', { timeout: 15000 }).should('exist').and('not.be.empty');
    
    // Check if buttons appear in the result
    cy.get('#execution-result').within(() => {
      cy.get('button').contains('Clear Results').should('be.visible');
      cy.get('button').contains('Download Results').should('be.visible');
    });
    
    // Test the Clear Results button
    cy.get('#execution-result').within(() => {
      cy.get('button').contains('Clear Results').click();
    });
    
    // Check that results are cleared
    cy.wait(1000);
    cy.get('#execution-result').should('be.empty');
  });

  it('should load hello_world sample and execute', () => {
    // Click on Hello World sample
    cy.get('[data-sample="hello_world"]').click();
    
    // Wait for the sample to load
    cy.wait(3000);
    
    // Verify file info is displayed
    cy.get('#file-info').should('be.visible');
    cy.get('#file-name').should('contain', 'hello_world.wasm');
    
    // Execute without input (hello world doesn't need input)
    cy.get('#submit-button').click();
    
    // Wait for execution
    cy.wait(8000);
    
    // Check result
    cy.get('#execution-result', { timeout: 15000 }).should('exist').and('not.be.empty');
    
    // Check buttons
    cy.get('#execution-result').within(() => {
      cy.get('button').contains('Clear Results').should('be.visible');
      cy.get('button').contains('Download Results').should('be.visible');
    });
  });
});
