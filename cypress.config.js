const { defineConfig } = require('cypress');

module.exports = defineConfig({
  e2e: {
    baseUrl: 'http://localhost:8081', // Updated to match your frontend port
    supportFile: false,
    specPattern: 'cypress/e2e/**/*.cy.js',
  },
});
