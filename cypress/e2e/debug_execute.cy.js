// cypress/e2e/debug_execute.cy.js
// Test the debug endpoint to isolate multipart parsing from WASM execution

describe('Debug Execute Endpoint', () => {
  it('should handle multipart form data in debug endpoint', () => {
    // Test using direct HTTP request instead of FormData
    const testWasmData = 'test wasm data';
    const testInput = 'test input data';
    
    cy.request({
      method: 'POST',
      url: '/api/debug-execute',
      form: true,
      body: {
        wasm: testWasmData,
        input: testInput,
      },
      timeout: 10000,
    }).then((response) => {
      expect(response.status).to.eq(200);
      expect(response.body).to.have.property('status', 'debug_success');
      expect(response.body).to.have.property('fields');
      expect(response.body.fields).to.be.an('array');
      
      // Check that both fields were received
      const fieldNames = response.body.fields.map(field => field.split(':')[0]);
      expect(fieldNames).to.include('wasm');
      expect(fieldNames).to.include('input');
    });
  });

  it('should handle file upload via fixture', () => {
    // Create a simple test file
    cy.writeFile('cypress/fixtures/test.wasm', 'test wasm content');
    
    cy.fixture('test.wasm', 'binary').then((wasmContent) => {
      const formData = new FormData();
      const blob = new Blob([wasmContent], { type: 'application/wasm' });
      formData.append('wasm', blob, 'test.wasm');
      formData.append('input', 'test input');
      
      cy.request({
        method: 'POST',
        url: '/api/debug-execute',
        body: formData,
        timeout: 10000,
      }).then((response) => {
        expect(response.status).to.eq(200);
        expect(response.body).to.have.property('status', 'debug_success');
      });
    });
  });

  it('should timeout if debug endpoint hangs', () => {
    cy.request({
      method: 'POST',
      url: '/api/debug-execute',
      form: true,
      body: {
        wasm: 'test',
        input: 'test',
      },
      timeout: 5000,
      failOnStatusCode: false
    }).then((response) => {
      // If we get here, the endpoint responded within timeout
      cy.log('Debug endpoint responded successfully');
      expect(response.status).to.be.oneOf([200, 408]); // 200 for success, 408 for timeout
    });
  });
});
