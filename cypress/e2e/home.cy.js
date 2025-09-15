// Basic E2E test for Wasm Wizard frontend

describe('Wasm Wizard Frontend E2E', () => {
  it('loads the homepage', () => {
    cy.visit('/');
    cy.contains('Wasm Wizard'); // Adjust if your homepage has a different heading
  });

  // Add more tests here for upload, execute, and result display
});
