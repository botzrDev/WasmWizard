describe('WasmWizard health endpoint', () => {
  it('responds with health information', () => {
    cy.request('/health').then((response) => {
      expect(response.status).to.eq(200);
      expect(response.body).to.have.property('status');
    });
  });
});
