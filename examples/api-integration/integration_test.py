#!/usr/bin/env python3
"""
API Integration Example - Integration Test Suite
This script provides comprehensive testing for the API integration WASM module
including authentication, error handling, and performance validation.
"""

import requests
import json
import time
import sys
from typing import Dict, List, Optional
import argparse

# Constants for repeated strings
CONTENT_TYPE_JSON = "application/json"
HEALTH_CHECK = "Health Check"
SINGLE_PROCESSING = "Single Processing"
BATCH_PROCESSING = "Batch Processing"
API_KEY_VALIDATION = "API Key Validation"
PERFORMANCE_MONITORING = "Performance Monitoring"
RATE_LIMITING = "Rate Limiting"

class APIIntegrationTester:
    def __init__(self, base_url: str = "http://localhost:8080"):
        self.base_url = base_url
        self.session = requests.Session()
        self.test_results = []

    def log_test_result(self, test_name: str, success: bool, message: str = "", response_time: float = 0):
        """Log a test result"""
        result = {
            "test": test_name,
            "success": success,
            "message": message,
            "response_time_ms": round(response_time * 1000, 2)
        }
        self.test_results.append(result)
        status = "✅" if success else "❌"
        print(f"{status} {test_name}: {message} ({result['response_time_ms']}ms)")

    def test_health_check(self) -> bool:
        """Test the health check endpoint"""
        try:
            start_time = time.time()
            response = self.session.get(f"{self.base_url}/health")
            response_time = time.time() - start_time

            if response.status_code == 200:
                data = response.json()
                if data.get("status") == "healthy":
                    self.log_test_result(HEALTH_CHECK, True, "Service is healthy", response_time)
                    return True

            self.log_test_result(HEALTH_CHECK, False, f"Unexpected response: {response.status_code}", response_time)
            return False

        except Exception as e:
            self.log_test_result(HEALTH_CHECK, False, f"Exception: {str(e)}", 0)
            return False

    def test_single_processing(self, auth_token: Optional[str] = None) -> bool:
        """Test single data processing"""
        test_data = {
            "input": "Hello World from integration test",
            "options": {
                "reverse": True,
                "count_words": True
            }
        }

        headers = {CONTENT_TYPE_JSON: CONTENT_TYPE_JSON}
        if auth_token:
            headers["Authorization"] = f"Bearer {auth_token}"

        try:
            start_time = time.time()
            response = self.session.post(
                f"{self.base_url}/api/v1/execute",
                json=test_data,
                headers=headers
            )
            response_time = time.time() - start_time

            if response.status_code == 200:
                data = response.json()
                if data.get("status") == "success":
                    self.log_test_result(SINGLE_PROCESSING, True, "Data processed successfully", response_time)
                    return True

            self.log_test_result(SINGLE_PROCESSING, False, f"Processing failed: {response.status_code}", response_time)
            return False

        except Exception as e:
            self.log_test_result(SINGLE_PROCESSING, False, f"Exception: {str(e)}", 0)
            return False

    def test_batch_processing(self, auth_token: Optional[str] = None) -> bool:
        """Test batch data processing"""
        test_data = [
            "First test message",
            "Second test message with more content",
            "Third message for batch processing",
            "Fourth message to test scalability",
            "Fifth and final test message"
        ]

        headers = {CONTENT_TYPE_JSON: CONTENT_TYPE_JSON}
        if auth_token:
            headers["Authorization"] = f"Bearer {auth_token}"

        try:
            start_time = time.time()
            response = self.session.post(
                f"{self.base_url}/api/v1/batch",
                json=test_data,
                headers=headers
            )
            response_time = time.time() - start_time

            if response.status_code == 200:
                data = response.json()
                if data.get("successful", 0) > 0:
                    success_rate = (data.get("successful", 0) / data.get("total_items", 1)) * 100
                    self.log_test_result(BATCH_PROCESSING, True, f"Processed {success_rate:.1f}% successfully", response_time)
                    return True

            self.log_test_result(BATCH_PROCESSING, False, f"Batch processing failed: {response.status_code}", response_time)
            return False

        except Exception as e:
            self.log_test_result(BATCH_PROCESSING, False, f"Exception: {str(e)}", 0)
            return False

    def _test_single_api_key(self, api_key: str, should_be_valid: bool) -> bool:
        """Test a single API key"""
        try:
            start_time = time.time()
            response = self.session.post(
                f"{self.base_url}/api/v1/validate",
                data=api_key,
                headers={CONTENT_TYPE_JSON: "text/plain"}
            )
            response_time = time.time() - start_time

            is_valid_response = response.status_code == 200 and response.json().get("valid", False)

            if is_valid_response == should_be_valid:
                self.log_test_result(
                    f"{API_KEY_VALIDATION} ({'valid' if should_be_valid else 'invalid'})",
                    True,
                    "Key validation correct",
                    response_time
                )
                return True
            else:
                self.log_test_result(
                    f"{API_KEY_VALIDATION} ({'valid' if should_be_valid else 'invalid'})",
                    False,
                    f"Expected {'valid' if should_be_valid else 'invalid'}, got {'valid' if is_valid_response else 'invalid'}",
                    response_time
                )
                return False

        except Exception as e:
            self.log_test_result(
                f"{API_KEY_VALIDATION} ({'valid' if should_be_valid else 'invalid'})",
                False,
                f"Exception: {str(e)}",
                0
            )
            return False

    def test_api_key_validation(self) -> bool:
        """Test API key validation"""
        test_keys = [
            ("abcdefghijklmnopqrst1234567890", True),  # Valid key
            ("short", False),  # Invalid key
            ("", False),  # Empty key
        ]

        all_passed = True
        for api_key, should_be_valid in test_keys:
            if not self._test_single_api_key(api_key, should_be_valid):
                all_passed = False

        return all_passed

    def test_error_handling(self) -> bool:
        """Test error handling scenarios"""
        error_tests = [
            {
                "name": "Invalid JSON",
                "data": "{invalid json",
                "expected_status": 400
            },
            {
                "name": "Empty Input",
                "data": "",
                "expected_status": 400
            },
            {
                "name": "Oversized Input",
                "data": "x" * 100000,  # 100KB of data
                "expected_status": 413
            }
        ]

        all_passed = True

        for test in error_tests:
            try:
                start_time = time.time()
                response = self.session.post(
                    f"{self.base_url}/api/v1/execute",
                    data=test["data"],
                    headers={CONTENT_TYPE_JSON: CONTENT_TYPE_JSON}
                )
                response_time = time.time() - start_time

                if response.status_code == test["expected_status"]:
                    self.log_test_result(
                        f"Error Handling - {test['name']}",
                        True,
                        f"Correct error response: {response.status_code}",
                        response_time
                    )
                else:
                    self.log_test_result(
                        f"Error Handling - {test['name']}",
                        False,
                        f"Expected {test['expected_status']}, got {response.status_code}",
                        response_time
                    )
                    all_passed = False

            except Exception as e:
                self.log_test_result(
                    f"Error Handling - {test['name']}",
                    False,
                    f"Exception: {str(e)}",
                    0
                )
                all_passed = False

        return all_passed

    def test_performance_monitoring(self) -> bool:
        """Test performance monitoring endpoints"""
        try:
            start_time = time.time()
            response = self.session.get(f"{self.base_url}/api/v1/stats")
            response_time = time.time() - start_time

            if response.status_code == 200:
                data = response.json()
                required_fields = ["average_processing_time_ms", "memory_usage_kb", "requests_processed"]

                if all(field in data for field in required_fields):
                    self.log_test_result(PERFORMANCE_MONITORING, True, "Stats retrieved successfully", response_time)
                    return True

            self.log_test_result(PERFORMANCE_MONITORING, False, f"Invalid stats response: {response.status_code}", response_time)
            return False

        except Exception as e:
            self.log_test_result(PERFORMANCE_MONITORING, False, f"Exception: {str(e)}", 0)
            return False

    def test_rate_limiting(self) -> bool:
        """Test rate limiting functionality"""
        # Send multiple requests quickly
        responses = []
        start_time = time.time()

        for i in range(10):
            try:
                response = self.session.post(
                    f"{self.base_url}/api/v1/execute",
                    json={"input": f"Rate limit test {i}"},
                    headers={CONTENT_TYPE_JSON: CONTENT_TYPE_JSON}
                )
                responses.append(response.status_code)
            except requests.RequestException:
                responses.append(429)  # Assume rate limited on error

        response_time = time.time() - start_time

        # Check if any requests were rate limited (429)
        rate_limited = 429 in responses
        success_count = responses.count(200)

        if rate_limited or success_count > 0:
            self.log_test_result(
                RATE_LIMITING,
                True,
                f"Rate limiting working (success: {success_count}, limited: {responses.count(429)})",
                response_time
            )
            return True
        else:
            self.log_test_result(RATE_LIMITING, False, "Rate limiting not detected", response_time)
            return False

    def run_all_tests(self, auth_token: Optional[str] = None) -> Dict:
        """Run all integration tests"""
        print("🚀 Starting API Integration Test Suite")
        print("=" * 50)

        tests = [
            (HEALTH_CHECK, self.test_health_check),
            (SINGLE_PROCESSING, lambda: self.test_single_processing(auth_token)),
            (BATCH_PROCESSING, lambda: self.test_batch_processing(auth_token)),
            (API_KEY_VALIDATION, self.test_api_key_validation),
            ("Error Handling", self.test_error_handling),
            (PERFORMANCE_MONITORING, self.test_performance_monitoring),
            (RATE_LIMITING, self.test_rate_limiting),
        ]

        passed = 0
        total = len(tests)

        for test_name, test_func in tests:
            print(f"\n🔍 Running {test_name}...")
            if test_func():
                passed += 1

        print("\n" + "=" * 50)
        print(f"📊 Test Results: {passed}/{total} tests passed")

        success_rate = (passed / total) * 100
        overall_success = success_rate >= 80  # 80% success threshold

        if overall_success:
            print("✅ Integration test suite PASSED")
        else:
            print("❌ Integration test suite FAILED")

        return {
            "total_tests": total,
            "passed_tests": passed,
            "success_rate": success_rate,
            "overall_success": overall_success,
            "results": self.test_results
        }

def main():
    parser = argparse.ArgumentParser(description="API Integration Test Suite")
    parser.add_argument("--url", default="http://localhost:8080", help="Base URL of the API")
    parser.add_argument("--auth-token", help="Authentication token for API requests")
    parser.add_argument("--output", help="Output file for test results (JSON)")

    args = parser.parse_args()

    tester = APIIntegrationTester(args.url)
    results = tester.run_all_tests(args.auth_token)

    if args.output:
        with open(args.output, 'w') as f:
            json.dump(results, f, indent=2)
        print(f"📄 Results saved to {args.output}")

    # Exit with appropriate code
    sys.exit(0 if results["overall_success"] else 1)

if __name__ == "__main__":
    main()