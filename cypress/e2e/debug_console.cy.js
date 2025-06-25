describe('Debug Console Logs', () => {
  beforeEach(() => {
    // Capture all console logs
    cy.window().then((win) => {
      cy.stub(win.console, 'log').as('consoleLog');
      cy.stub(win.console, 'error').as('consoleError');
      cy.stub(win.console, 'warn').as('consoleWarn');
    });
    
    cy.visit('http://localhost:8080');
  });

  it('should capture console logs during sample execution', () => {
    // Wait for page to load completely
    cy.get('.sample-card').should('be.visible');
    
    // Click on echo sample
    cy.get('.sample-card').contains('Echo').click();
    
    // Wait a moment for any logs to appear
    cy.wait(1000);
    
    // Check for console logs
    cy.get('@consoleLog').then((stub) => {
      console.log('Console logs captured:', stub.getCalls().map(call => call.args));
    });
    
    cy.get('@consoleError').then((stub) => {
      console.log('Console errors captured:', stub.getCalls().map(call => call.args));
    });
    
    // Try to execute
    cy.get('#execute-btn').click();
    
    // Wait for execution to complete or fail
    cy.wait(5000);
    
    // Check console logs again
    cy.get('@consoleLog').then((stub) => {
      console.log('Console logs after execution:', stub.getCalls().map(call => call.args));
    });
    
    cy.get('@consoleError').then((stub) => {
      console.log('Console errors after execution:', stub.getCalls().map(call => call.args));
    });
    
    // Check DOM state
    cy.get('body').then(($body) => {
      console.log('Body HTML after execution:', $body.html());
    });
    
    // Check the execution result div specifically
    cy.get('#execution-result').then(($result) => {
      console.log('Execution result div content:', $result.html());
      console.log('Execution result div is empty:', $result.is(':empty'));
    });
  });
});
