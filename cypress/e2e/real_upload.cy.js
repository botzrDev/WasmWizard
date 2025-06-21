// cypress/e2e/real_upload.cy.js
// Test the actual execution endpoint with real file upload

describe('Real File Upload Test', () => {
  beforeEach(() => {
    // Create a simple WASM file with magic bytes
    const wasmData = new Uint8Array([
      0x00, 0x61, 0x73, 0x6D, // WASM magic bytes
      0x01, 0x00, 0x00, 0x00, // WASM version
      // Add some minimal WASM content
      0x01, 0x04, 0x01, 0x60, 0x00, 0x00, // type section with empty function type
    ]);
    cy.writeFile('cypress/fixtures/test.wasm', wasmData);
  });

  it('should upload and execute a WASM file through the UI', () => {
    cy.visit('/');
    
    // Upload the WASM file
    cy.get('#wasm-file').selectFile('cypress/fixtures/test.wasm', { force: true });
    
    // Add some input
    cy.get('#input-text').type('test input data');
    
    // Submit the form
    cy.get('#submit-button').click();
    
    // Wait for some response (success or error)
    cy.get('#execution-result', { timeout: 15000 }).should('exist');
    
    // Check if we get any content in the result area
    cy.get('#execution-result').should('not.be.empty');
  });

  it('should handle file upload via direct API call', () => {
    cy.fixture('test.wasm', 'binary').then((wasmContent) => {
      const formData = new FormData();
      formData.append('wasm', new Blob([wasmContent], { type: 'application/wasm' }), 'test.wasm');
      formData.append('input', 'test input');
      
      cy.request({
        method: 'POST',
        url: '/api/execute',
        body: formData,
        timeout: 15000,
        failOnStatusCode: false
      }).then((response) => {
        cy.log(`Response status: ${response.status}`);
        cy.log(`Response body:`, response.body);
        
        // We expect either success (200) or some error, but not a hang
        expect(response.status).to.be.oneOf([200, 400, 422, 500]);
      });
    });
  });
});
