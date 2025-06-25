describe('Debug File Loading', () => {
  beforeEach(() => {
    cy.visit('http://127.0.0.1:8081');
    cy.wait(1000); // Wait for page to load
  });

  it('should load sample module and check file input state', () => {
    // Click on Echo Module sample
    cy.contains('Echo Module').click();
    
    // Wait for the sample to load
    cy.wait(2000);
    
    // Check if file input has files
    cy.get('#wasm-file').then($input => {
      const fileInput = $input[0];
      cy.log(`File input files length: ${fileInput.files.length}`);
      
      if (fileInput.files.length > 0) {
        cy.log(`File name: ${fileInput.files[0].name}`);
        cy.log(`File size: ${fileInput.files[0].size}`);
      }
      
      // File input should have a file
      expect(fileInput.files.length).to.be.greaterThan(0);
    });
    
    // Check if file info is displayed
    cy.get('#file-info').should('be.visible');
    cy.get('#file-name').should('contain', 'echo.wasm');
    
    // Add some input text
    cy.get('#input-text').type('Hello, WasmWiz!');
    
    // Try to execute
    cy.get('#submit-button').click();
    
    // Wait for execution to complete
    cy.wait(5000);
    
    // Check if result container appears
    cy.get('#execution-result', { timeout: 10000 }).should('exist');
    
    // Check if buttons appear
    cy.get('#execution-result').within(() => {
      cy.get('button').contains('Clear Results').should('be.visible');
      cy.get('button').contains('Download Results').should('be.visible');
    });
  });

  it('should manually upload a file and execute', () => {
    // Check if we have a sample wasm file available
    cy.readFile('wasmwiz/static/wasm_modules/echo.wasm', { encoding: null }).then((fileContent) => {
      // Create a file from the content
      const file = new File([fileContent], 'echo.wasm', { type: 'application/wasm' });
      
      // Get the file input and set the file
      cy.get('#wasm-file').then($input => {
        const fileInput = $input[0];
        const dataTransfer = new DataTransfer();
        dataTransfer.items.add(file);
        fileInput.files = dataTransfer.files;
        
        // Trigger change event
        cy.get('#wasm-file').trigger('change');
      });
      
      // Wait a moment
      cy.wait(1000);
      
      // Add input text
      cy.get('#input-text').type('Hello from manual upload!');
      
      // Execute
      cy.get('#submit-button').click();
      
      // Wait for execution
      cy.wait(5000);
      
      // Check result
      cy.get('#execution-result', { timeout: 10000 }).should('exist');
      cy.get('#execution-result').within(() => {
        cy.get('button').contains('Clear Results').should('be.visible');
        cy.get('button').contains('Download Results').should('be.visible');
      });
    });
  });
});
