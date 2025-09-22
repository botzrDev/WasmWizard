describe('WasmWizard health endpoint', () => {
  it('responds with detailed health information', () => {
    cy.request({
      url: '/health',
      retryOnNetworkFailure: true,
      retryOnStatusCodeFailure: true,
    }).then((response) => {
      expect(response.status).to.eq(200);
      expect(response.headers['content-type']).to.include('application/json');
      expect(response.body).to.have.property('status', 'healthy');
      expect(response.body).to.have.property('version').that.is.a('string');
      expect(response.body)
        .to.have.nested.property('checks.database.status')
        .that.matches(/healthy|degraded/);
    });
  });
});
