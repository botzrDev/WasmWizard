// cypress/e2e/full_flow.cy.js
// E2E test for upload → execute → display results for all sample modules

describe('Wasm Wizard Full E2E Flow', () => {
  const samples = [
    { name: 'calc_add', input: '2 3', expected: '5' },
    { name: 'echo', input: 'Hello, Wasm Wizard!', expected: 'Hello, Wasm Wizard!' },
    { name: 'hello_world', input: '', expected: 'Hello, World!' },
  ];

  samples.forEach(({ name, input, expected }) => {
    it(`executes sample module: ${name}`, () => {
      cy.visit('/');
      // Use the sample gallery button
      cy.get(`.sample-card[data-sample="${name}"] .use-sample`).click();
      // If input is required, fill it
      if (input) {
        cy.get('#input-text').clear().type(input);
      }
      // Submit the form
      cy.get('#submit-button').click();
      // Wait for result
      cy.get('#execution-result', { timeout: 10000 }).should('be.visible');
      // Check for expected output
      cy.get('#execution-result').should('contain.text', expected);
    });
  });
});
