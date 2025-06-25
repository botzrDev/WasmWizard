describe('Comprehensive WASM Execution Flow', () => {
  beforeEach(() => {
    cy.visit('http://127.0.0.1:8081')
  })

  it('should execute WASM and display result card with buttons', () => {
    // Step 1: Upload a WASM file
    cy.get('#wasm-file').should('exist')
    
    // Use the calc_add sample module that should exist
    cy.get('[data-sample="calc_add"]').should('exist').click()
    
    // Wait for file to be loaded
    cy.get('#file-info', { timeout: 10000 }).should('be.visible')
    cy.get('#file-name').should('contain', 'calc_add.wasm')
    
    // Step 2: Add input
    cy.get('#input-text').should('exist').clear().type('2 3')
    
    // Step 3: Submit the form
    cy.get('#submit-button').should('exist').should('not.be.disabled').click()
    
    // Step 4: Wait for execution to complete and check for result card
    cy.get('#execution-result', { timeout: 30000 }).should('exist')
    
    // Step 5: Verify the result card content
    cy.get('#execution-result .execution-result-card').should('exist')
    cy.get('#execution-result .result-header h3').should('contain', 'Execution Result')
    cy.get('#execution-result .status-badge').should('exist')
    
    // Step 6: Verify buttons are present and functional
    cy.get('#execution-result .result-actions').should('exist')
    cy.get('#execution-result .result-actions button').should('have.length', 2)
    
    // Check for Clear Results button
    cy.get('#execution-result .result-actions button:contains("Clear Results")').should('exist')
    
    // Check for Download Results button  
    cy.get('#execution-result .result-actions button:contains("Download Results")').should('exist')
    
    // Step 7: Test Clear Results functionality
    cy.get('#execution-result .result-actions button:contains("Clear Results")').click()
    cy.get('#execution-result').should('be.empty')
    
    // Step 8: Re-execute and test Download Results functionality
    cy.get('#submit-button').click()
    cy.get('#execution-result', { timeout: 30000 }).should('exist')
    cy.get('#execution-result .result-actions button:contains("Download Results")').should('exist')
    
    // Click download (we can't easily test file download in Cypress, but we can ensure no errors)
    cy.get('#execution-result .result-actions button:contains("Download Results")').click()
    
    // Verify no JavaScript errors occurred
    cy.window().then((win) => {
      expect(win.console.error).to.not.have.been.called
    })
  })

  it('should handle WASM execution errors gracefully', () => {
    // Upload an invalid file or trigger an error condition
    cy.get('#input-text').type('invalid input')
    
    // Try to submit without a file - should show validation error
    cy.get('#submit-button').click()
    
    // Should show an error alert instead of execution result
    cy.get('.alert-error', { timeout: 10000 }).should('exist')
  })

  it('should show debug logs in console during execution', () => {
    // Set up console spy
    cy.window().then((win) => {
      cy.spy(win.console, 'log').as('consoleLog')
    })
    
    // Load sample and execute
    cy.get('[data-sample="hello_world"]').click()
    cy.get('#submit-button', { timeout: 10000 }).click()
    
    // Wait for execution
    cy.get('#execution-result', { timeout: 30000 }).should('exist')
    
    // Check that debug logs were written
    cy.get('@consoleLog').should('have.been.calledWith', 
      Cypress.sinon.match(/\[DEBUG\] displayExecutionResult called/))
  })
})
