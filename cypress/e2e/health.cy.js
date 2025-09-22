describe('WasmWizard health endpoint', () => {
  it('responds with health information', () => {
    cy.request({
      url: '/health',
      failOnStatusCode: false,
      timeout: 10000,
    }).then((response) => {
      expect(response.status).to.eq(200);
      expect(response.headers['content-type']).to.include('application/json');
      expect(response.body).to.have.property('status');
      expect(response.body.status).to.be.a('string').and.to.equal('healthy');
      expect(response.body).to.have.nested.property('checks.database.status');
      expect(response.body.checks.database.status).to.be.a('string');
    });
  });
});
